//! Command Pipeline Parser
//!
//! Parses shell command pipelines with operators: |, &&, ||, ;
//! Respects quote boundaries when identifying operators.

use crate::parser::parse_command;

/// Operator type for command chaining
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    /// Pipe: cmd1 | cmd2 (pass stdout of cmd1 to stdin of cmd2)
    Pipe,
    /// AND: cmd1 && cmd2 (run cmd2 only if cmd1 succeeds)
    And,
    /// OR: cmd1 || cmd2 (run cmd2 only if cmd1 fails)
    Or,
    /// Sequential: cmd1 ; cmd2 (run both regardless of exit status)
    Semicolon,
}

/// A single command in a pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// Command name
    pub name: String,
    /// Command arguments
    pub args: Vec<String>,
}

impl Command {
    /// Create a new command
    pub fn new(name: String, args: Vec<String>) -> Self {
        Self { name, args }
    }
}

/// A pipeline stage: command + operator to next stage
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStage {
    /// The command to execute
    pub command: Command,
    /// The operator connecting to the next command (if any)
    pub operator: Option<Operator>,
}

/// A complete command pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    /// All stages in the pipeline
    pub stages: Vec<PipelineStage>,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the pipeline
    pub fn add_stage(&mut self, stage: PipelineStage) {
        self.stages.push(stage);
    }

    /// Check if this is a simple command (single stage, no operators)
    pub fn is_simple(&self) -> bool {
        self.stages.len() == 1 && self.stages[0].operator.is_none()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a command line into a pipeline
///
/// Handles operators: |, &&, ||, ;
/// Respects quotes when splitting
///
/// # Examples
///
/// ```
/// use wos_shared::pipeline::{parse_pipeline, Operator};
///
/// // Simple command
/// let pipeline = parse_pipeline("echo hello");
/// assert_eq!(pipeline.stages.len(), 1);
/// assert!(pipeline.is_simple());
///
/// // Pipe
/// let pipeline = parse_pipeline("cat file.txt | grep pattern");
/// assert_eq!(pipeline.stages.len(), 2);
/// assert_eq!(pipeline.stages[0].operator, Some(Operator::Pipe));
/// ```
pub fn parse_pipeline(input: &str) -> Pipeline {
    let mut pipeline = Pipeline::new();
    let input = input.trim();

    if input.is_empty() {
        return pipeline;
    }

    // Split by operators while respecting quotes
    let commands = split_by_operators(input);

    for (cmd_str, op) in commands {
        let (name, args) = parse_command(&cmd_str);
        if !name.is_empty() {
            let command = Command::new(name, args);
            let stage = PipelineStage {
                command,
                operator: op,
            };
            pipeline.add_stage(stage);
        }
    }

    pipeline
}

/// Split command line by operators, respecting quotes
///
/// Returns vector of (command_string, next_operator)
fn split_by_operators(input: &str) -> Vec<(String, Option<Operator>)> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                current.push(ch);
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                current.push(ch);
            }
            '\\' if !in_single_quote => {
                // Escape sequence - include backslash and next char
                current.push(ch);
                if let Some(next_ch) = chars.next() {
                    current.push(next_ch);
                }
            }
            '|' if !in_single_quote && !in_double_quote => {
                // Check for ||
                if chars.peek() == Some(&'|') {
                    chars.next(); // consume second |
                    result.push((current.trim().to_string(), Some(Operator::Or)));
                    current.clear();
                } else {
                    result.push((current.trim().to_string(), Some(Operator::Pipe)));
                    current.clear();
                }
            }
            '&' if !in_single_quote && !in_double_quote => {
                // Check for &&
                if chars.peek() == Some(&'&') {
                    chars.next(); // consume second &
                    result.push((current.trim().to_string(), Some(Operator::And)));
                    current.clear();
                } else {
                    // Single & is not supported (background execution)
                    current.push(ch);
                }
            }
            ';' if !in_single_quote && !in_double_quote => {
                result.push((current.trim().to_string(), Some(Operator::Semicolon)));
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Add final command (no operator after it)
    if !current.trim().is_empty() {
        result.push((current.trim().to_string(), None));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple command tests
    #[test]
    fn test_parse_simple_command() {
        let pipeline = parse_pipeline("echo hello");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.name, "echo");
        assert_eq!(pipeline.stages[0].command.args, vec!["hello"]);
        assert_eq!(pipeline.stages[0].operator, None);
        assert!(pipeline.is_simple());
    }

    #[test]
    fn test_parse_empty_string() {
        let pipeline = parse_pipeline("");
        assert_eq!(pipeline.stages.len(), 0);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let pipeline = parse_pipeline("   \t  ");
        assert_eq!(pipeline.stages.len(), 0);
    }

    // Pipe operator tests
    #[test]
    fn test_parse_pipe_two_commands() {
        let pipeline = parse_pipeline("cat file.txt | grep pattern");
        assert_eq!(pipeline.stages.len(), 2);

        assert_eq!(pipeline.stages[0].command.name, "cat");
        assert_eq!(pipeline.stages[0].command.args, vec!["file.txt"]);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::Pipe));

        assert_eq!(pipeline.stages[1].command.name, "grep");
        assert_eq!(pipeline.stages[1].command.args, vec!["pattern"]);
        assert_eq!(pipeline.stages[1].operator, None);
        assert!(!pipeline.is_simple());
    }

    #[test]
    fn test_parse_pipe_three_commands() {
        let pipeline = parse_pipeline("cat file | grep foo | wc -l");
        assert_eq!(pipeline.stages.len(), 3);

        assert_eq!(pipeline.stages[0].operator, Some(Operator::Pipe));
        assert_eq!(pipeline.stages[1].operator, Some(Operator::Pipe));
        assert_eq!(pipeline.stages[2].operator, None);
    }

    #[test]
    fn test_parse_pipe_with_quotes() {
        let pipeline = parse_pipeline("echo \"hello | world\" | grep hello");
        assert_eq!(pipeline.stages.len(), 2);

        // First command should have the full quoted string
        assert_eq!(pipeline.stages[0].command.name, "echo");
        assert_eq!(pipeline.stages[0].command.args, vec!["hello | world"]);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::Pipe));
    }

    // AND operator tests
    #[test]
    fn test_parse_and_operator() {
        let pipeline = parse_pipeline("mkdir test && cd test");
        assert_eq!(pipeline.stages.len(), 2);

        assert_eq!(pipeline.stages[0].command.name, "mkdir");
        assert_eq!(pipeline.stages[0].operator, Some(Operator::And));

        assert_eq!(pipeline.stages[1].command.name, "cd");
        assert_eq!(pipeline.stages[1].operator, None);
    }

    #[test]
    fn test_parse_and_chain() {
        let pipeline = parse_pipeline("cmd1 && cmd2 && cmd3");
        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::And));
        assert_eq!(pipeline.stages[1].operator, Some(Operator::And));
        assert_eq!(pipeline.stages[2].operator, None);
    }

    // OR operator tests
    #[test]
    fn test_parse_or_operator() {
        let pipeline = parse_pipeline("test -f file || echo \"not found\"");
        assert_eq!(pipeline.stages.len(), 2);

        assert_eq!(pipeline.stages[0].command.name, "test");
        assert_eq!(pipeline.stages[0].operator, Some(Operator::Or));

        assert_eq!(pipeline.stages[1].command.name, "echo");
        assert_eq!(pipeline.stages[1].command.args, vec!["not found"]);
        assert_eq!(pipeline.stages[1].operator, None);
    }

    // Semicolon operator tests
    #[test]
    fn test_parse_semicolon_operator() {
        let pipeline = parse_pipeline("echo one ; echo two");
        assert_eq!(pipeline.stages.len(), 2);

        assert_eq!(pipeline.stages[0].command.name, "echo");
        assert_eq!(pipeline.stages[0].operator, Some(Operator::Semicolon));

        assert_eq!(pipeline.stages[1].command.name, "echo");
        assert_eq!(pipeline.stages[1].operator, None);
    }

    #[test]
    fn test_parse_semicolon_chain() {
        let pipeline = parse_pipeline("cmd1 ; cmd2 ; cmd3");
        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::Semicolon));
        assert_eq!(pipeline.stages[1].operator, Some(Operator::Semicolon));
        assert_eq!(pipeline.stages[2].operator, None);
    }

    // Mixed operator tests
    #[test]
    fn test_parse_mixed_and_or() {
        let pipeline = parse_pipeline("cmd1 && cmd2 || cmd3");
        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::And));
        assert_eq!(pipeline.stages[1].operator, Some(Operator::Or));
        assert_eq!(pipeline.stages[2].operator, None);
    }

    #[test]
    fn test_parse_pipe_and_and() {
        let pipeline = parse_pipeline("cat file | grep foo && echo found");
        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::Pipe));
        assert_eq!(pipeline.stages[1].operator, Some(Operator::And));
        assert_eq!(pipeline.stages[2].operator, None);
    }

    // Quote handling tests
    #[test]
    fn test_operators_in_single_quotes_ignored() {
        let pipeline = parse_pipeline("echo 'cmd1 | cmd2 && cmd3'");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.args, vec!["cmd1 | cmd2 && cmd3"]);
        assert!(pipeline.is_simple());
    }

    #[test]
    fn test_operators_in_double_quotes_ignored() {
        let pipeline = parse_pipeline("echo \"cmd1 || cmd2 ; cmd3\"");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.args, vec!["cmd1 || cmd2 ; cmd3"]);
        assert!(pipeline.is_simple());
    }

    // Edge cases
    #[test]
    fn test_multiple_spaces_around_operators() {
        let pipeline = parse_pipeline("cmd1   |   cmd2");
        assert_eq!(pipeline.stages.len(), 2);
        assert_eq!(pipeline.stages[0].command.name, "cmd1");
        assert_eq!(pipeline.stages[1].command.name, "cmd2");
    }

    #[test]
    fn test_no_spaces_around_operators() {
        let pipeline = parse_pipeline("cmd1|cmd2");
        assert_eq!(pipeline.stages.len(), 2);
        assert_eq!(pipeline.stages[0].command.name, "cmd1");
        assert_eq!(pipeline.stages[1].command.name, "cmd2");
    }

    #[test]
    fn test_trailing_operator_ignored() {
        let pipeline = parse_pipeline("cmd1 |");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.name, "cmd1");
    }

    #[test]
    fn test_leading_operator_ignored() {
        let pipeline = parse_pipeline("| cmd1");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.name, "cmd1");
    }

    // Command structure tests
    #[test]
    fn test_command_with_args_in_pipeline() {
        let pipeline = parse_pipeline("ls -la /tmp | grep test");
        assert_eq!(pipeline.stages.len(), 2);
        assert_eq!(pipeline.stages[0].command.name, "ls");
        assert_eq!(pipeline.stages[0].command.args, vec!["-la", "/tmp"]);
        assert_eq!(pipeline.stages[1].command.name, "grep");
        assert_eq!(pipeline.stages[1].command.args, vec!["test"]);
    }

    #[test]
    fn test_quoted_args_preserved_in_pipeline() {
        let pipeline = parse_pipeline("mkdir \"my dir\" && cd \"my dir\"");
        assert_eq!(pipeline.stages.len(), 2);
        assert_eq!(pipeline.stages[0].command.args, vec!["my dir"]);
        assert_eq!(pipeline.stages[1].command.args, vec!["my dir"]);
    }
}
