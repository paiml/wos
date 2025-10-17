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

/// Redirection type for I/O
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Redirection {
    /// Redirect stdout to file (overwrite): cmd > file
    StdoutOverwrite(String),
    /// Redirect stdout to file (append): cmd >> file
    StdoutAppend(String),
    /// Redirect stdin from file: cmd < file
    StdinFrom(String),
}

/// A single command in a pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// Command name
    pub name: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Optional redirections for this command
    pub redirections: Vec<Redirection>,
}

impl Command {
    /// Create a new command
    pub fn new(name: String, args: Vec<String>) -> Self {
        Self {
            name,
            args,
            redirections: Vec::new(),
        }
    }

    /// Create a new command with redirections
    pub fn with_redirections(
        name: String,
        args: Vec<String>,
        redirections: Vec<Redirection>,
    ) -> Self {
        Self {
            name,
            args,
            redirections,
        }
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
        // Extract redirections from command string
        let (cmd_without_redirects, redirections) = extract_redirections(&cmd_str);

        let (name, args) = parse_command(&cmd_without_redirects);
        if !name.is_empty() {
            let command = Command::with_redirections(name, args, redirections);
            let stage = PipelineStage {
                command,
                operator: op,
            };
            pipeline.add_stage(stage);
        }
    }

    pipeline
}

/// Helper: Skip whitespace characters in iterator
fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
        chars.next();
    }
}

/// Helper: Handle stdout redirection (> or >>)
/// Returns the redirection if filename extracted successfully
fn handle_stdout_redirect(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    is_append: bool,
) -> Option<Redirection> {
    skip_whitespace(chars);
    let filename = extract_filename(chars);
    if filename.is_empty() {
        None
    } else if is_append {
        Some(Redirection::StdoutAppend(filename))
    } else {
        Some(Redirection::StdoutOverwrite(filename))
    }
}

/// Helper: Handle stdin redirection (<)
/// Returns the redirection if filename extracted successfully
fn handle_stdin_redirect(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<Redirection> {
    skip_whitespace(chars);
    let filename = extract_filename(chars);
    if filename.is_empty() {
        None
    } else {
        Some(Redirection::StdinFrom(filename))
    }
}

/// Extract redirection operators from a command string
///
/// Returns (command_without_redirects, vec_of_redirections)
fn extract_redirections(input: &str) -> (String, Vec<Redirection>) {
    let mut command = String::new();
    let mut redirections = Vec::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                command.push(ch);
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                command.push(ch);
            }
            '>' if !in_single_quote && !in_double_quote => {
                // Check for >> (append) vs > (overwrite)
                let is_append = chars.peek() == Some(&'>');
                if is_append {
                    chars.next(); // consume second >
                }
                if let Some(redir) = handle_stdout_redirect(&mut chars, is_append) {
                    redirections.push(redir);
                }
            }
            '<' if !in_single_quote && !in_double_quote => {
                if let Some(redir) = handle_stdin_redirect(&mut chars) {
                    redirections.push(redir);
                }
            }
            _ => {
                command.push(ch);
            }
        }
    }

    (command.trim().to_string(), redirections)
}

/// Extract a filename from the character iterator
fn extract_filename(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut filename = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' if !in_quotes => break,
            '\'' | '"' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
                chars.next();
            }
            c if in_quotes && c == quote_char => {
                chars.next();
                break;
            }
            _ => {
                filename.push(ch);
                chars.next();
            }
        }
    }

    filename
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

    #[test]
    fn test_complex_operator_chain_parsing() {
        // Test: echo first && echo second || echo backup ; echo final
        let pipeline = parse_pipeline("echo first && echo second || echo backup ; echo final");
        assert_eq!(pipeline.stages.len(), 4);
        assert_eq!(pipeline.stages[0].operator, Some(Operator::And));
        assert_eq!(pipeline.stages[1].operator, Some(Operator::Or));
        assert_eq!(pipeline.stages[2].operator, Some(Operator::Semicolon));
        assert_eq!(pipeline.stages[3].operator, None);
    }

    // Redirection tests
    #[test]
    fn test_parse_stdout_redirect_overwrite() {
        let pipeline = parse_pipeline("echo hello > output.txt");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.name, "echo");
        assert_eq!(pipeline.stages[0].command.args, vec!["hello"]);
        assert_eq!(pipeline.stages[0].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdoutOverwrite("output.txt".to_string())
        );
    }

    #[test]
    fn test_parse_stdout_redirect_append() {
        let pipeline = parse_pipeline("echo world >> output.txt");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.name, "echo");
        assert_eq!(pipeline.stages[0].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdoutAppend("output.txt".to_string())
        );
    }

    #[test]
    fn test_parse_stdin_redirect() {
        let pipeline = parse_pipeline("cat < input.txt");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.name, "cat");
        assert_eq!(pipeline.stages[0].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdinFrom("input.txt".to_string())
        );
    }

    #[test]
    fn test_parse_multiple_redirections() {
        let pipeline = parse_pipeline("cat < input.txt > output.txt");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.redirections.len(), 2);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdinFrom("input.txt".to_string())
        );
        assert_eq!(
            pipeline.stages[0].command.redirections[1],
            Redirection::StdoutOverwrite("output.txt".to_string())
        );
    }

    #[test]
    fn test_parse_redirection_with_quoted_filename() {
        let pipeline = parse_pipeline("echo test > \"my file.txt\"");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdoutOverwrite("my file.txt".to_string())
        );
    }

    #[test]
    fn test_parse_redirection_no_spaces() {
        let pipeline = parse_pipeline("echo test>output.txt");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdoutOverwrite("output.txt".to_string())
        );
    }

    #[test]
    fn test_parse_redirection_with_pipe() {
        let pipeline = parse_pipeline("cat file.txt | grep pattern > results.txt");
        assert_eq!(pipeline.stages.len(), 2);

        // First stage: cat with no redirection
        assert_eq!(pipeline.stages[0].command.name, "cat");
        assert_eq!(pipeline.stages[0].command.redirections.len(), 0);

        // Second stage: grep with output redirection
        assert_eq!(pipeline.stages[1].command.name, "grep");
        assert_eq!(pipeline.stages[1].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[1].command.redirections[0],
            Redirection::StdoutOverwrite("results.txt".to_string())
        );
    }

    #[test]
    fn test_parse_redirection_operators_in_quotes_ignored() {
        let pipeline = parse_pipeline("echo \"test > file\" > output.txt");
        assert_eq!(pipeline.stages.len(), 1);
        // Quotes are stripped by parse_command (standard shell behavior)
        assert_eq!(pipeline.stages[0].command.args, vec!["test > file"]);
        assert_eq!(pipeline.stages[0].command.redirections.len(), 1);
        assert_eq!(
            pipeline.stages[0].command.redirections[0],
            Redirection::StdoutOverwrite("output.txt".to_string())
        );
    }

    // WOS-Q01: Tests for refactored extract_redirections helper functions
    #[test]
    fn test_handle_stdout_redirect_single() {
        let mut chars = " output.txt".chars().peekable();
        let result = handle_stdout_redirect(&mut chars, false);
        assert_eq!(
            result,
            Some(Redirection::StdoutOverwrite("output.txt".to_string()))
        );
    }

    #[test]
    fn test_handle_stdout_redirect_append() {
        let mut chars = " output.txt".chars().peekable();
        let result = handle_stdout_redirect(&mut chars, true);
        assert_eq!(
            result,
            Some(Redirection::StdoutAppend("output.txt".to_string()))
        );
    }

    #[test]
    fn test_handle_stdin_redirect() {
        let mut chars = " input.txt".chars().peekable();
        let result = handle_stdin_redirect(&mut chars);
        assert_eq!(
            result,
            Some(Redirection::StdinFrom("input.txt".to_string()))
        );
    }

    #[test]
    fn test_handle_stdin_redirect_with_quotes() {
        let mut chars = " \"my file.txt\"".chars().peekable();
        let result = handle_stdin_redirect(&mut chars);
        assert_eq!(
            result,
            Some(Redirection::StdinFrom("my file.txt".to_string()))
        );
    }

    #[test]
    fn test_skip_whitespace_helper() {
        let mut chars = "   \t  hello".chars().peekable();
        skip_whitespace(&mut chars);
        assert_eq!(chars.next(), Some('h'));
    }
}
