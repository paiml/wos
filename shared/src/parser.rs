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
/// assert_eq!(args, vec!["\"my directory\""]);
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

/// Helper: Process escape sequence after backslash
/// Modifies the result string in place
fn process_escape_sequence(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    result: &mut String,
    in_double_quote: bool,
) {
    if let Some(next_ch) = chars.next() {
        // Special case: \$ should be preserved for variable expander
        if next_ch == '$' {
            result.push('\\');
            result.push('$');
        } else if in_double_quote {
            // Handle common escape sequences in double quotes
            match next_ch {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => result.push(next_ch),
            }
        } else {
            // Outside quotes, just push the next character
            result.push(next_ch);
        }
    } else {
        // Trailing backslash, treat as literal
        result.push('\\');
    }
}

/// Helper: Handle quote character toggle
/// Returns (new_single_quote_state, new_double_quote_state, had_quotes_flag)
fn handle_quote_toggle(
    ch: char,
    in_single_quote: bool,
    in_double_quote: bool,
) -> (bool, bool, bool) {
    match ch {
        '\'' if !in_double_quote => (!in_single_quote, in_double_quote, true),
        '"' if !in_single_quote => (in_single_quote, !in_double_quote, true),
        _ => (in_single_quote, in_double_quote, false),
    }
}

/// Helper: Check if character should end current token
fn should_end_token(
    ch: char,
    in_single_quote: bool,
    in_double_quote: bool,
    paren_depth: usize,
    brace_depth: usize,
) -> bool {
    (ch == ' ' || ch == '\t')
        && !in_single_quote
        && !in_double_quote
        && paren_depth == 0
        && brace_depth == 0
}

/// Complete current token and add to tokens list
fn complete_token(current: &mut String, tokens: &mut Vec<String>, had_quotes: &mut bool) {
    if !current.is_empty() || *had_quotes {
        tokens.push(current.clone());
        current.clear();
        *had_quotes = false;
    }
}

/// Handle ${...} parameter expansion start
/// Returns true if this was a parameter expansion start
pub(crate) fn handle_parameter_expansion(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_token: &mut String,
    in_single_quote: bool,
    brace_depth: &mut usize,
) -> bool {
    if ch == '$' && chars.peek() == Some(&'{') && !in_single_quote {
        current_token.push(ch);
        current_token.push(chars.next().unwrap()); // consume '{'
        *brace_depth += 1;
        true
    } else {
        false
    }
}

/// Handle $(...) and $((...)) expansions start
/// Returns true if this was an expansion start
pub(crate) fn handle_command_or_arithmetic_expansion(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_token: &mut String,
    in_single_quote: bool,
    paren_depth: &mut usize,
) -> bool {
    if ch == '$' && chars.peek() == Some(&'(') && !in_single_quote {
        current_token.push(ch);
        current_token.push(chars.next().unwrap()); // consume first '('

        // Check if this is $(( (arithmetic) or just $( (command substitution)
        if chars.peek() == Some(&'(') {
            current_token.push(chars.next().unwrap()); // consume second '('
            *paren_depth += 2; // Track both opening parens
        } else {
            *paren_depth += 1;
        }
        true
    } else {
        false
    }
}

/// Handle closing } for parameter expansion
/// Returns true if this was a closing brace
pub(crate) fn handle_closing_brace(
    ch: char,
    current_token: &mut String,
    in_single_quote: bool,
    brace_depth: &mut usize,
) -> bool {
    if ch == '}' && *brace_depth > 0 && !in_single_quote {
        current_token.push(ch);
        *brace_depth -= 1;
        true
    } else {
        false
    }
}

/// Handle closing ) for command/arithmetic expansion
/// Returns true if this was a closing paren
pub(crate) fn handle_closing_paren(
    ch: char,
    current_token: &mut String,
    in_single_quote: bool,
    paren_depth: &mut usize,
) -> bool {
    if ch == ')' && *paren_depth > 0 && !in_single_quote {
        current_token.push(ch);
        *paren_depth -= 1;
        true
    } else {
        false
    }
}

/// Process a single character in tokenization
/// Returns true if character was handled (no need to push to token)
#[allow(clippy::too_many_arguments)]
fn process_token_char(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_token: &mut String,
    in_single: &mut bool,
    in_double: &mut bool,
    had_quotes: &mut bool,
    tokens: &mut Vec<String>,
    paren_depth: usize,
    brace_depth: usize,
) -> bool {
    // Handle escape sequences
    if ch == '\\' && !*in_single {
        process_escape_sequence(chars, current_token, *in_double);
        return true;
    }

    // Handle quote toggles
    let (new_single, new_double, quotes_flag) = handle_quote_toggle(ch, *in_single, *in_double);
    if quotes_flag {
        *in_single = new_single;
        *in_double = new_double;
        *had_quotes = true;
        // KEEP the quote character in the token so expand_variables can see it
        current_token.push(ch);
        return true;
    }

    // Handle token-ending whitespace
    if should_end_token(ch, *in_single, *in_double, paren_depth, brace_depth) {
        complete_token(current_token, tokens, had_quotes);
        return true;
    }

    false
}

/// Tokenize a command line respecting quotes and escapes
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut had_quotes = false;
    let mut paren_depth = 0; // Track $(...) and $((...)) nesting
    let mut brace_depth = 0; // Track ${...} nesting

    while let Some(ch) = chars.next() {
        // Check for ${  - start of parameter expansion
        if handle_parameter_expansion(
            ch,
            &mut chars,
            &mut current_token,
            in_single_quote,
            &mut brace_depth,
        ) {
            continue;
        }

        // Check for $((  - start of arithmetic expansion
        if handle_command_or_arithmetic_expansion(
            ch,
            &mut chars,
            &mut current_token,
            in_single_quote,
            &mut paren_depth,
        ) {
            continue;
        }

        // Track closing } when inside ${...}
        if handle_closing_brace(ch, &mut current_token, in_single_quote, &mut brace_depth) {
            continue;
        }

        // Track closing ) when inside $(...) or $((...))
        if handle_closing_paren(ch, &mut current_token, in_single_quote, &mut paren_depth) {
            continue;
        }

        let handled = process_token_char(
            ch,
            &mut chars,
            &mut current_token,
            &mut in_single_quote,
            &mut in_double_quote,
            &mut had_quotes,
            &mut tokens,
            paren_depth,
            brace_depth,
        );

        if !handled {
            current_token.push(ch);
        }
    }

    // Add final token if any
    complete_token(&mut current_token, &mut tokens, &mut had_quotes);

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
        assert_eq!(args, vec!["\"my directory\""]);
    }

    #[test]
    fn test_parse_double_quotes_with_spaces() {
        let (cmd, args) = parse_command("touch \"file with spaces.txt\"");
        assert_eq!(cmd, "touch");
        assert_eq!(args, vec!["\"file with spaces.txt\""]);
    }

    #[test]
    fn test_parse_multiple_quoted_args() {
        let (cmd, args) = parse_command("cmd \"arg one\" \"arg two\"");
        assert_eq!(cmd, "cmd");
        assert_eq!(args, vec!["\"arg one\"", "\"arg two\""]);
    }

    #[test]
    fn test_parse_mixed_quoted_unquoted() {
        let (cmd, args) = parse_command("cp file.txt \"my folder/backup.txt\"");
        assert_eq!(cmd, "cp");
        assert_eq!(args, vec!["file.txt", "\"my folder/backup.txt\""]);
    }

    // Single quote tests
    #[test]
    fn test_parse_single_quotes() {
        let (cmd, args) = parse_command("echo 'hello world'");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["'hello world'"]);
    }

    #[test]
    fn test_parse_single_quotes_literal() {
        let (cmd, args) = parse_command("echo 'It'\"'\"'s working'");
        assert_eq!(cmd, "echo");
        // Parser keeps quotes - expand_variables will handle them
        assert_eq!(args, vec!["'It'\"'\"'s working'"]);
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
        assert_eq!(args, vec!["\"Line 1\nLine 2\""]);
    }

    #[test]
    fn test_parse_escape_tab_in_double_quotes() {
        let (cmd, args) = parse_command("echo \"Col1\\tCol2\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["\"Col1\tCol2\""]);
    }

    #[test]
    fn test_parse_escape_carriage_return_in_double_quotes() {
        let (cmd, args) = parse_command("echo \"Line1\\rLine2\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["\"Line1\rLine2\""]);
    }

    #[test]
    fn test_parse_escape_backslash_in_double_quotes() {
        let (cmd, args) = parse_command("echo \"Path\\\\is\\\\escaped\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["\"Path\\is\\escaped\""]);
    }

    #[test]
    fn test_parse_escape_quote_in_double_quotes() {
        let (cmd, args) = parse_command("echo \"She said \\\"Hello\\\"\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["\"She said \"Hello\"\""]);
    }

    #[test]
    fn test_parse_no_escape_in_single_quotes() {
        let (cmd, args) = parse_command("echo 'Line 1\\nLine 2'");
        assert_eq!(cmd, "echo");
        // Backslash is literal in single quotes
        assert_eq!(args, vec!["'Line 1\\nLine 2'"]);
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
        assert_eq!(args, vec!["\"\"", "''"]);
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
        assert_eq!(args, vec!["\"$VAR $(cmd) `backtick`\""]);
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
                    assert_eq!(arg, "\"a b c\"");
                } else if input.contains("x y z") {
                    assert_eq!(arg, "'x y z'");
                } else if input.contains("hello   world") {
                    assert_eq!(arg, "\"hello   world\"");
                }
            }
        }
    }

    // Escaped dollar sign tests (for variable expansion)
    #[test]
    fn test_parse_escaped_dollar_outside_quotes() {
        let (cmd, args) = parse_command("echo \\$VAR");
        assert_eq!(cmd, "echo");
        // Parser should preserve \$ so expander can handle it
        assert_eq!(args, vec!["\\$VAR"]);
    }

    #[test]
    fn test_parse_escaped_dollar_in_double_quotes() {
        let (cmd, args) = parse_command("echo \"\\$VAR\"");
        assert_eq!(cmd, "echo");
        // In double quotes, \$ should be preserved for variable expander
        assert_eq!(args, vec!["\"\\$VAR\""]);
    }

    #[test]
    fn test_parse_escaped_dollar_in_single_quotes() {
        let (cmd, args) = parse_command("echo '\\$VAR'");
        assert_eq!(cmd, "echo");
        // In single quotes, everything is literal including backslash
        assert_eq!(args, vec!["'\\$VAR'"]);
    }

    // WOS-Q03: Tests for refactored tokenize helper functions
    #[test]
    fn test_process_escape_sequence_dollar() {
        let mut chars = "$VAR".chars().peekable();
        let mut result = String::new();
        process_escape_sequence(&mut chars, &mut result, false);
        assert_eq!(result, "\\$");
    }

    #[test]
    fn test_process_escape_sequence_in_double_quotes() {
        let mut chars = "n".chars().peekable();
        let mut result = String::new();
        process_escape_sequence(&mut chars, &mut result, true);
        assert_eq!(result, "\n");
    }

    #[test]
    fn test_process_escape_sequence_outside_quotes() {
        let mut chars = " ".chars().peekable();
        let mut result = String::new();
        process_escape_sequence(&mut chars, &mut result, false);
        assert_eq!(result, " ");
    }

    #[test]
    fn test_process_escape_sequence_trailing() {
        let mut chars = "".chars().peekable();
        let mut result = String::new();
        process_escape_sequence(&mut chars, &mut result, false);
        assert_eq!(result, "\\");
    }

    #[test]
    fn test_handle_quote_toggle_single() {
        let (new_single, new_double, had_quotes) = handle_quote_toggle('\'', false, false);
        assert!(new_single);
        assert!(!new_double);
        assert!(had_quotes);
    }

    #[test]
    fn test_handle_quote_toggle_double() {
        let (new_single, new_double, had_quotes) = handle_quote_toggle('"', false, false);
        assert!(!new_single);
        assert!(new_double);
        assert!(had_quotes);
    }

    #[test]
    fn test_handle_quote_toggle_close_single() {
        let (new_single, new_double, had_quotes) = handle_quote_toggle('\'', true, false);
        assert!(!new_single);
        assert!(!new_double);
        assert!(had_quotes);
    }

    #[test]
    fn test_should_end_token_whitespace() {
        assert!(should_end_token(' ', false, false, 0, 0));
        assert!(should_end_token('\t', false, false, 0, 0));
    }

    #[test]
    fn test_should_end_token_in_quotes() {
        assert!(!should_end_token(' ', true, false, 0, 0));
        assert!(!should_end_token(' ', false, true, 0, 0));
    }

    #[test]
    fn test_should_end_token_in_command_substitution() {
        // Space inside $(...)  should not end token
        assert!(!should_end_token(' ', false, false, 1, 0));
        assert!(!should_end_token('\t', false, false, 1, 0));
        // Space outside $(...)  should end token
        assert!(should_end_token(' ', false, false, 0, 0));
    }

    #[test]
    fn test_should_end_token_in_parameter_expansion() {
        // Space inside ${...} should not end token
        assert!(!should_end_token(' ', false, false, 0, 1));
        assert!(!should_end_token('\t', false, false, 0, 1));
        // Space outside ${...} should end token
        assert!(should_end_token(' ', false, false, 0, 0));
    }

    // Property-based tests using proptest
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: Parser never panics on any input
            #[test]
            fn proptest_parser_never_panics(
                input in "\\PC*"
            ) {
                let _ = parse_command(&input);
                // If we get here, we didn't panic
                prop_assert!(true);
            }

            /// Property: Parser is deterministic (same input = same output)
            #[test]
            fn proptest_parser_deterministic(
                input in "[a-zA-Z0-9 \\t\\n\"'\\\\$(){}\\[\\]]{0,100}"
            ) {
                let result1 = parse_command(&input);
                let result2 = parse_command(&input);
                prop_assert_eq!(result1, result2);
            }

            /// Property: Empty and whitespace-only inputs handled correctly
            #[test]
            fn proptest_empty_input(
                whitespace in "[ \\t\\n]*"
            ) {
                let (cmd, args) = parse_command(&whitespace);
                prop_assert_eq!(cmd, "");
                prop_assert_eq!(args, Vec::<String>::new());
            }

            /// Property: Command name is always first token
            #[test]
            fn proptest_command_is_first_token(
                cmd in "[a-z]{1,20}",
                args in prop::collection::vec("[a-z0-9]{1,15}", 0..10)
            ) {
                let input = format!("{} {}", cmd, args.join(" "));
                let (parsed_cmd, parsed_args) = parse_command(&input);

                prop_assert_eq!(parsed_cmd, cmd);
                prop_assert_eq!(parsed_args.len(), args.len());
            }

            /// Property: Parsing preserves non-empty command
            #[test]
            fn proptest_nonempty_command_preserved(
                cmd in "[a-zA-Z][a-zA-Z0-9_]{0,19}"
            ) {
                let (parsed_cmd, _) = parse_command(&cmd);
                prop_assert!(!parsed_cmd.is_empty());
                prop_assert_eq!(parsed_cmd, cmd);
            }

            /// Property: Number of tokens matches expected count
            #[test]
            fn proptest_token_count(
                words in prop::collection::vec("[a-z]{1,10}", 1..20)
            ) {
                let input = words.join(" ");
                let (cmd, args) = parse_command(&input);

                prop_assert!(!cmd.is_empty());
                prop_assert_eq!(args.len(), words.len() - 1);
            }

            /// Property: Backslash in single quotes is literal
            #[test]
            fn proptest_single_quote_literal(
                text in "[a-zA-Z0-9\\\\]{1,20}"
            ) {
                let input = format!("echo '{}'", text);
                let (cmd, args) = parse_command(&input);

                prop_assert_eq!(cmd, "echo");
                prop_assert_eq!(args.len(), 1);
                // Single quotes preserve content literally (including the quotes)
                prop_assert!(args[0].contains(&text));
            }

            /// Property: Multiple spaces collapse to single separator
            #[test]
            fn proptest_whitespace_collapse(
                words in prop::collection::vec("[a-z]{1,10}", 2..10)
            ) {
                // Build input with variable spacing (1-5 spaces between words)
                let mut input = words[0].clone();
                for (i, word) in words.iter().enumerate().skip(1) {
                    let num_spaces = (i % 5) + 1; // 1-5 spaces
                    input.push_str(&" ".repeat(num_spaces));
                    input.push_str(word);
                }

                let (cmd, args) = parse_command(&input);

                prop_assert_eq!(&cmd, &words[0]);
                prop_assert_eq!(args.len(), words.len() - 1);
                for i in 0..args.len() {
                    prop_assert_eq!(&args[i], &words[i + 1]);
                }
            }

            /// Property: Escaped characters are handled without panic
            #[test]
            fn proptest_escape_handling(
                chars in prop::collection::vec(any::<char>(), 0..50)
            ) {
                let mut input = String::from("echo ");
                for ch in chars {
                    input.push('\\');
                    input.push(ch);
                }

                // Should not panic
                let _ = parse_command(&input);
                prop_assert!(true);
            }

            /// Property: Parser handles nested structures gracefully
            #[test]
            fn proptest_nested_structures(
                depth in 0..10usize,
                content in "[a-z]{1,10}"
            ) {
                let mut input = String::from("echo ");
                for _ in 0..depth {
                    input.push_str("$(");
                }
                input.push_str(&content);
                for _ in 0..depth {
                    input.push(')');
                }

                // Should not panic
                let (cmd, args) = parse_command(&input);
                prop_assert_eq!(cmd, "echo");
                prop_assert!(!args.is_empty());
            }
        }
    }
}
