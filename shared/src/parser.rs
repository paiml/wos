//! Command Line Parser
//!
//! Provides shell-like command parsing with proper quote handling.
//! Supports single quotes, double quotes, and escape sequences.

/// Parse a command line into command name and arguments
///
/// Handles:
/// - Single quotes (') - literal strings, no expansion
/// - Double quotes (") - strings with variable expansion (future)
/// - Backslash (\) - escape next character
/// - Whitespace - argument separator
///
/// # Examples
///
/// ```
/// use wos_shared::parser::parse_command;
///
/// let (cmd, args) = parse_command("echo hello world");
/// assert_eq!(cmd, "echo");
/// assert_eq!(args, vec!["hello", "world"]);
///
/// let (cmd, args) = parse_command("mkdir \"my directory\"");
/// assert_eq!(cmd, "mkdir");
/// assert_eq!(args, vec!["my directory"]);
/// ```
pub fn parse_command(input: &str) -> (String, Vec<String>) {
    let input = input.trim();
    if input.is_empty() {
        return (String::new(), vec![]);
    }

    let tokens = tokenize(input);
    if tokens.is_empty() {
        return (String::new(), vec![]);
    }

    let cmd = tokens[0].clone();
    let args = tokens[1..].to_vec();

    (cmd, args)
}

/// Tokenize a command line respecting quotes and escapes
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut had_quotes = false; // Track if we've seen quotes for empty string handling

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_single_quote => {
                // Backslash escapes next character (except in single quotes)
                if let Some(next_ch) = chars.next() {
                    // Handle common escape sequences in double quotes
                    if in_double_quote {
                        match next_ch {
                            'n' => current_token.push('\n'),
                            't' => current_token.push('\t'),
                            'r' => current_token.push('\r'),
                            '\\' => current_token.push('\\'),
                            '"' => current_token.push('"'),
                            _ => {
                                // For other characters, push the escaped char literally
                                current_token.push(next_ch);
                            }
                        }
                    } else {
                        // Outside quotes, just push the next character
                        current_token.push(next_ch);
                    }
                } else {
                    // Trailing backslash, treat as literal
                    current_token.push('\\');
                }
            }
            '\'' if !in_double_quote => {
                // Single quote toggle
                in_single_quote = !in_single_quote;
                had_quotes = true;
            }
            '"' if !in_single_quote => {
                // Double quote toggle
                in_double_quote = !in_double_quote;
                had_quotes = true;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Whitespace outside quotes - end current token
                if !current_token.is_empty() || had_quotes {
                    tokens.push(current_token.clone());
                    current_token.clear();
                    had_quotes = false;
                }
            }
            _ => {
                // Regular character
                current_token.push(ch);
            }
        }
    }

    // Add final token if any (including empty strings from quotes)
    if !current_token.is_empty() || had_quotes {
        tokens.push(current_token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic parsing tests
    #[test]
    fn test_parse_simple_command() {
        let (cmd, args) = parse_command("echo hello");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello"]);
    }

    #[test]
    fn test_parse_multiple_args() {
        let (cmd, args) = parse_command("ls -la /tmp");
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);
    }

    #[test]
    fn test_parse_empty_string() {
        let (cmd, args) = parse_command("");
        assert_eq!(cmd, "");
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let (cmd, args) = parse_command("   \t  ");
        assert_eq!(cmd, "");
        assert_eq!(args, Vec::<String>::new());
    }

    // Double quote tests
    #[test]
    fn test_parse_double_quotes() {
        let (cmd, args) = parse_command("mkdir \"my directory\"");
        assert_eq!(cmd, "mkdir");
        assert_eq!(args, vec!["my directory"]);
    }

    #[test]
    fn test_parse_double_quotes_with_spaces() {
        let (cmd, args) = parse_command("touch \"file with spaces.txt\"");
        assert_eq!(cmd, "touch");
        assert_eq!(args, vec!["file with spaces.txt"]);
    }

    #[test]
    fn test_parse_multiple_quoted_args() {
        let (cmd, args) = parse_command("cmd \"arg one\" \"arg two\"");
        assert_eq!(cmd, "cmd");
        assert_eq!(args, vec!["arg one", "arg two"]);
    }

    #[test]
    fn test_parse_mixed_quoted_unquoted() {
        let (cmd, args) = parse_command("cp file.txt \"my folder/backup.txt\"");
        assert_eq!(cmd, "cp");
        assert_eq!(args, vec!["file.txt", "my folder/backup.txt"]);
    }

    // Single quote tests
    #[test]
    fn test_parse_single_quotes() {
        let (cmd, args) = parse_command("echo 'hello world'");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_single_quotes_literal() {
        let (cmd, args) = parse_command("echo 'It'\"'\"'s working'");
        assert_eq!(cmd, "echo");
        // Single quotes preserve everything literally
        assert_eq!(args, vec!["It's working"]);
    }

    // Escape sequence tests
    #[test]
    fn test_parse_backslash_escape() {
        let (cmd, args) = parse_command("echo hello\\ world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_escape_quote() {
        let (cmd, args) = parse_command("echo \\\"quoted\\\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["\"quoted\""]);
    }

    #[test]
    fn test_parse_escape_in_double_quotes() {
        let (cmd, args) = parse_command("echo \"Line 1\\nLine 2\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["Line 1\nLine 2"]);
    }

    #[test]
    fn test_parse_no_escape_in_single_quotes() {
        let (cmd, args) = parse_command("echo 'Line 1\\nLine 2'");
        assert_eq!(cmd, "echo");
        // Backslash is literal in single quotes
        assert_eq!(args, vec!["Line 1\\nLine 2"]);
    }

    // Edge cases
    #[test]
    fn test_parse_trailing_backslash() {
        let (cmd, args) = parse_command("echo test\\");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["test\\"]);
    }

    #[test]
    fn test_parse_empty_quotes() {
        let (cmd, args) = parse_command("echo \"\" ''");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["", ""]);
    }

    #[test]
    fn test_parse_multiple_spaces() {
        let (cmd, args) = parse_command("echo    hello    world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_tabs_and_spaces() {
        let (cmd, args) = parse_command("echo\thello \t world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    // Special characters
    #[test]
    fn test_parse_special_chars_in_quotes() {
        let (cmd, args) = parse_command("echo \"$VAR $(cmd) `backtick`\"");
        assert_eq!(cmd, "echo");
        // For now, these are literal (variable expansion is future work)
        assert_eq!(args, vec!["$VAR $(cmd) `backtick`"]);
    }

    #[test]
    fn test_parse_path_with_slashes() {
        let (cmd, args) = parse_command("cat /tmp/my\\ file.txt");
        assert_eq!(cmd, "cat");
        assert_eq!(args, vec!["/tmp/my file.txt"]);
    }

    // Property: parse then join should preserve quoted strings
    #[test]
    fn test_parse_preserves_spaces_in_quotes() {
        let inputs = vec!["mkdir \"a b c\"", "touch 'x y z'", "echo \"hello   world\""];

        for input in inputs {
            let (_, args) = parse_command(input);
            for arg in args {
                // If we had spaces in the input, they should be preserved in the arg
                if input.contains("a b c") {
                    assert_eq!(arg, "a b c");
                } else if input.contains("x y z") {
                    assert_eq!(arg, "x y z");
                } else if input.contains("hello   world") {
                    assert_eq!(arg, "hello   world");
                }
            }
        }
    }
}
