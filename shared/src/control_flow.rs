//! Shell Control Flow Statements
//!
//! Implements if/else, while/until loops for shell scripting.
//! WOS-SHELL-001: If/Else Control Flow
//! WOS-SHELL-002: While/Until Loops

use serde::{Deserialize, Serialize};

/// Control flow statement types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlFlow {
    /// If statement with optional elif and else branches
    If {
        /// Condition command (exit code 0 = true)
        condition: String,
        /// Commands to execute if condition is true
        then_body: Vec<String>,
        /// Optional elif branches
        elif_branches: Vec<(String, Vec<String>)>,
        /// Optional else branch
        else_body: Option<Vec<String>>,
    },
    /// While loop
    While {
        /// Condition command (exit code 0 = continue)
        condition: String,
        /// Commands to execute in loop body
        body: Vec<String>,
    },
    /// Until loop (opposite of while)
    Until {
        /// Condition command (exit code != 0 = continue)
        condition: String,
        /// Commands to execute in loop body
        body: Vec<String>,
    },
}

/// Parse if/then/elif/else/fi statement
///
/// Syntax:
/// ```bash
/// if condition; then
///     commands
/// elif condition; then
///     commands
/// else
///     commands
/// fi
/// ```
pub fn parse_if_statement(lines: &[&str]) -> Result<ControlFlow, String> {
    if lines.is_empty() {
        return Err("Empty if statement".to_string());
    }

    // First line should be "if condition; then" or "if condition"
    let first_line = lines[0].trim();
    if !first_line.starts_with("if ") {
        return Err(format!("Expected 'if' but got: {}", first_line));
    }

    // Extract condition (between "if" and "then" or ";")
    let condition = extract_condition(first_line, "if")?;

    let mut then_body = Vec::new();
    let mut elif_branches = Vec::new();
    let mut else_body = None;
    let mut current_section = "then";
    let mut i = 1;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("elif ") {
            // Save current then_body and start elif
            let elif_condition = extract_condition(line, "elif")?;
            elif_branches.push((elif_condition, Vec::new()));
            current_section = "elif";
        } else if line == "else" {
            current_section = "else";
            else_body = Some(Vec::new());
        } else if line == "fi" {
            break;
        } else if !line.is_empty() {
            // Add command to current section
            match current_section {
                "then" => then_body.push(line.to_string()),
                "elif" => {
                    if let Some((_cond, body)) = elif_branches.last_mut() {
                        body.push(line.to_string());
                    }
                }
                "else" => {
                    if let Some(body) = &mut else_body {
                        body.push(line.to_string());
                    }
                }
                _ => {}
            }
        }

        i += 1;
    }

    if i >= lines.len() {
        return Err("Missing 'fi' to close if statement".to_string());
    }

    Ok(ControlFlow::If {
        condition,
        then_body,
        elif_branches,
        else_body,
    })
}

/// Parse while/do/done statement
///
/// Syntax:
/// ```bash
/// while condition; do
///     commands
/// done
/// ```
pub fn parse_while_statement(lines: &[&str]) -> Result<ControlFlow, String> {
    if lines.is_empty() {
        return Err("Empty while statement".to_string());
    }

    let first_line = lines[0].trim();
    if !first_line.starts_with("while ") {
        return Err(format!("Expected 'while' but got: {}", first_line));
    }

    let condition = extract_condition(first_line, "while")?;
    let mut body = Vec::new();
    let mut i = 1;

    while i < lines.len() {
        let line = lines[i].trim();
        if line == "done" {
            break;
        } else if !line.is_empty() {
            body.push(line.to_string());
        }
        i += 1;
    }

    if i >= lines.len() {
        return Err("Missing 'done' to close while loop".to_string());
    }

    Ok(ControlFlow::While { condition, body })
}

/// Parse until/do/done statement
///
/// Syntax:
/// ```bash
/// until condition; do
///     commands
/// done
/// ```
pub fn parse_until_statement(lines: &[&str]) -> Result<ControlFlow, String> {
    if lines.is_empty() {
        return Err("Empty until statement".to_string());
    }

    let first_line = lines[0].trim();
    if !first_line.starts_with("until ") {
        return Err(format!("Expected 'until' but got: {}", first_line));
    }

    let condition = extract_condition(first_line, "until")?;
    let mut body = Vec::new();
    let mut i = 1;

    while i < lines.len() {
        let line = lines[i].trim();
        if line == "done" {
            break;
        } else if !line.is_empty() {
            body.push(line.to_string());
        }
        i += 1;
    }

    if i >= lines.len() {
        return Err("Missing 'done' to close until loop".to_string());
    }

    Ok(ControlFlow::Until { condition, body })
}

/// Helper: Extract condition from control flow keyword
fn extract_condition(line: &str, keyword: &str) -> Result<String, String> {
    // Remove keyword prefix
    let after_keyword = line
        .strip_prefix(keyword)
        .map(|s| s.trim())
        .unwrap_or("");

    let mut condition = after_keyword.to_string();

    // Remove "then" or "do" keywords (handle with or without semicolon)
    // "true; then" -> "true" or "true then" -> "true"
    if let Some(idx) = condition.rfind("then") {
        condition = condition[..idx].trim().to_string();
    } else if let Some(idx) = condition.rfind("do") {
        condition = condition[..idx].trim().to_string();
    }

    // Remove trailing semicolon
    condition = condition.trim_end_matches(';').trim().to_string();

    if condition.is_empty() {
        return Err(format!("Empty condition in {} statement", keyword));
    }

    Ok(condition)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // WOS-SHELL-001: If/Else Control Flow Tests
    // ========================================================================

    #[test]
    fn test_if_then_fi_simple() {
        let lines = vec!["if true; then", "echo hello", "fi"];

        let result = parse_if_statement(&lines);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Failed to parse if statement");

        let ctrl = result.unwrap();
        match ctrl {
            ControlFlow::If {
                condition,
                then_body,
                elif_branches,
                else_body,
            } => {
                assert_eq!(condition, "true");
                assert_eq!(then_body, vec!["echo hello"]);
                assert!(elif_branches.is_empty());
                assert!(else_body.is_none());
            }
            _ => panic!("Expected If variant"),
        }
    }

    #[test]
    fn test_if_then_else_fi() {
        let lines = vec!["if false; then", "echo yes", "else", "echo no", "fi"];

        let result = parse_if_statement(&lines);
        assert!(result.is_ok());

        if let ControlFlow::If {
            condition,
            then_body,
            else_body,
            ..
        } = result.unwrap()
        {
            assert_eq!(condition, "false");
            assert_eq!(then_body, vec!["echo yes"]);
            assert_eq!(else_body, Some(vec!["echo no".to_string()]));
        } else {
            panic!("Expected If variant");
        }
    }

    #[test]
    fn test_if_elif_else_fi() {
        let lines = vec![
            "if test -f file1; then",
            "echo file1",
            "elif test -f file2; then",
            "echo file2",
            "else",
            "echo none",
            "fi",
        ];

        let result = parse_if_statement(&lines);
        assert!(result.is_ok());

        if let ControlFlow::If {
            condition,
            then_body,
            elif_branches,
            else_body,
        } = result.unwrap()
        {
            assert_eq!(condition, "test -f file1");
            assert_eq!(then_body, vec!["echo file1"]);
            assert_eq!(elif_branches.len(), 1);
            assert_eq!(elif_branches[0].0, "test -f file2");
            assert_eq!(elif_branches[0].1, vec!["echo file2"]);
            assert_eq!(else_body, Some(vec!["echo none".to_string()]));
        } else {
            panic!("Expected If variant");
        }
    }

    #[test]
    fn test_if_missing_fi_error() {
        let lines = vec!["if true; then", "echo hello"];

        let result = parse_if_statement(&lines);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'fi'"));
    }

    #[test]
    fn test_if_empty_condition_error() {
        let lines = vec!["if ; then", "echo hello", "fi"];

        let result = parse_if_statement(&lines);
        assert!(result.is_err());
    }

    // ========================================================================
    // WOS-SHELL-002: While/Until Loop Tests
    // ========================================================================

    #[test]
    fn test_while_do_done() {
        let lines = vec!["while true; do", "echo loop", "done"];

        let result = parse_while_statement(&lines);
        assert!(result.is_ok());

        if let ControlFlow::While { condition, body } = result.unwrap() {
            assert_eq!(condition, "true");
            assert_eq!(body, vec!["echo loop"]);
        } else {
            panic!("Expected While variant");
        }
    }

    #[test]
    fn test_until_do_done() {
        let lines = vec!["until false; do", "echo loop", "done"];

        let result = parse_until_statement(&lines);
        assert!(result.is_ok());

        if let ControlFlow::Until { condition, body } = result.unwrap() {
            assert_eq!(condition, "false");
            assert_eq!(body, vec!["echo loop"]);
        } else {
            panic!("Expected Until variant");
        }
    }

    #[test]
    fn test_while_missing_done_error() {
        let lines = vec!["while true; do", "echo loop"];

        let result = parse_while_statement(&lines);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'done'"));
    }

    #[test]
    fn test_control_flow_serialization() {
        let if_stmt = ControlFlow::If {
            condition: "test -f file".to_string(),
            then_body: vec!["echo yes".to_string()],
            elif_branches: vec![],
            else_body: Some(vec!["echo no".to_string()]),
        };

        let json = serde_json::to_string(&if_stmt).expect("serialization should succeed");
        let deserialized: ControlFlow =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(if_stmt, deserialized);
    }
}
