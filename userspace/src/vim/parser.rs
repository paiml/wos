// Vim command parser
// Parses key presses and command strings into VimCommand enums

use super::command::VimCommand;
use super::state::VimError;

/// Parse a normal mode key press into a vim command
///
/// Note: Multi-key sequences like "x, mx, 'x, `x, yy require parser state tracking
/// in VimState.parser_state. This function handles single-key commands and
/// the first key of multi-key sequences.
pub fn parse_normal_key(key: char) -> Result<VimCommand, VimError> {
    match key {
        // Navigation
        'h' => Ok(VimCommand::MoveLeft),
        'j' => Ok(VimCommand::MoveDown),
        'k' => Ok(VimCommand::MoveUp),
        'l' => Ok(VimCommand::MoveRight),
        '0' => Ok(VimCommand::MoveLineStart),
        '$' => Ok(VimCommand::MoveLineEnd),

        // Insert mode
        'i' => Ok(VimCommand::InsertBefore),
        'a' => Ok(VimCommand::InsertAfter),
        'o' => Ok(VimCommand::InsertLineBelow),
        'O' => Ok(VimCommand::InsertLineAbove),

        // Editing
        'x' => Ok(VimCommand::DeleteChar),
        'p' => Ok(VimCommand::PutAfter),
        'u' => Ok(VimCommand::Undo),

        // Mode commands
        ':' => Ok(VimCommand::EnterCommandMode),

        // Multi-key sequence markers (handled by VimState)
        // These return InvalidCommand to signal they need state handling
        '"' => Err(VimError::InvalidCommand("Awaiting register name".into())),
        'm' => Err(VimError::InvalidCommand("Awaiting mark name".into())),
        '\'' => Err(VimError::InvalidCommand(
            "Awaiting mark for line jump".into(),
        )),
        '`' => Err(VimError::InvalidCommand(
            "Awaiting mark for exact jump".into(),
        )),
        'y' => Err(VimError::InvalidCommand("Awaiting yank target".into())),

        _ => Err(VimError::InvalidCommand(format!("Unknown key: {}", key))),
    }
}

/// Parse an insert mode key press
pub fn parse_insert_key(key: char, is_special: bool) -> VimCommand {
    if is_special {
        match key {
            '\x1b' => VimCommand::EnterNormalMode, // Escape
            '\x7f' => VimCommand::Backspace,       // Backspace
            '\n' | '\r' => VimCommand::InsertNewline,
            _ => VimCommand::InsertChar(key),
        }
    } else {
        VimCommand::InsertChar(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_normal_key_navigation() {
        assert!(matches!(parse_normal_key('h'), Ok(VimCommand::MoveLeft)));
        assert!(matches!(parse_normal_key('j'), Ok(VimCommand::MoveDown)));
        assert!(matches!(parse_normal_key('k'), Ok(VimCommand::MoveUp)));
        assert!(matches!(parse_normal_key('l'), Ok(VimCommand::MoveRight)));
    }

    #[test]
    fn test_parse_normal_key_editing() {
        assert!(matches!(
            parse_normal_key('i'),
            Ok(VimCommand::InsertBefore)
        ));
        assert!(matches!(parse_normal_key('a'), Ok(VimCommand::InsertAfter)));
        assert!(matches!(parse_normal_key('x'), Ok(VimCommand::DeleteChar)));
    }

    #[test]
    fn test_parse_normal_key_line_navigation() {
        assert!(matches!(
            parse_normal_key('0'),
            Ok(VimCommand::MoveLineStart)
        ));
        assert!(matches!(parse_normal_key('$'), Ok(VimCommand::MoveLineEnd)));
    }

    #[test]
    fn test_parse_normal_key_insert_line() {
        assert!(matches!(
            parse_normal_key('o'),
            Ok(VimCommand::InsertLineBelow)
        ));
        assert!(matches!(
            parse_normal_key('O'),
            Ok(VimCommand::InsertLineAbove)
        ));
    }

    #[test]
    fn test_parse_normal_key_undo() {
        assert!(matches!(parse_normal_key('u'), Ok(VimCommand::Undo)));
    }

    #[test]
    fn test_parse_normal_key_command_mode() {
        assert!(matches!(
            parse_normal_key(':'),
            Ok(VimCommand::EnterCommandMode)
        ));
    }

    #[test]
    fn test_parse_normal_key_invalid() {
        assert!(parse_normal_key('z').is_err());
    }

    #[test]
    fn test_parse_insert_key() {
        assert!(matches!(
            parse_insert_key('a', false),
            VimCommand::InsertChar('a')
        ));
        assert!(matches!(
            parse_insert_key('\n', true),
            VimCommand::InsertNewline
        ));
        assert!(matches!(
            parse_insert_key('\x7f', true),
            VimCommand::Backspace
        ));
    }

    #[test]
    fn test_parse_insert_key_escape() {
        assert!(matches!(
            parse_insert_key('\x1b', true),
            VimCommand::EnterNormalMode
        ));
    }

    // ========================================================================
    // RED TESTS - Coverage improvement (vim/parser.rs 71.88% → 100%)
    // ========================================================================

    #[test]
    fn test_parse_normal_key_put_after() {
        // Line 30: 'p' => Ok(VimCommand::PutAfter)
        assert!(matches!(parse_normal_key('p'), Ok(VimCommand::PutAfter)));
    }

    #[test]
    fn test_parse_normal_key_register_sequence() {
        // Lines 38-41: '"' => Err(VimError::InvalidCommand("Awaiting register name".into()))
        let result = parse_normal_key('"');
        assert!(result.is_err());
        if let Err(VimError::InvalidCommand(msg)) = result {
            assert_eq!(msg, "Awaiting register name");
        }
    }

    #[test]
    fn test_parse_normal_key_mark_line_jump() {
        // Lines 40-42: '\'' => Err(VimError::InvalidCommand("Awaiting mark for line jump".into()))
        let result = parse_normal_key('\'');
        assert!(result.is_err());
        if let Err(VimError::InvalidCommand(msg)) = result {
            assert_eq!(msg, "Awaiting mark for line jump");
        }
    }

    #[test]
    fn test_parse_normal_key_mark_exact_jump() {
        // Lines 43-45: '`' => Err(VimError::InvalidCommand("Awaiting mark for exact jump".into()))
        let result = parse_normal_key('`');
        assert!(result.is_err());
        if let Err(VimError::InvalidCommand(msg)) = result {
            assert_eq!(msg, "Awaiting mark for exact jump");
        }
    }

    #[test]
    fn test_parse_normal_key_yank_sequence() {
        // Line 46: 'y' => Err(VimError::InvalidCommand("Awaiting yank target".into()))
        let result = parse_normal_key('y');
        assert!(result.is_err());
        if let Err(VimError::InvalidCommand(msg)) = result {
            assert_eq!(msg, "Awaiting yank target");
        }
    }

    #[test]
    fn test_parse_normal_key_mark_set() {
        // Lines 39: 'm' => Err(VimError::InvalidCommand("Awaiting mark name".into()))
        let result = parse_normal_key('m');
        assert!(result.is_err());
        if let Err(VimError::InvalidCommand(msg)) = result {
            assert_eq!(msg, "Awaiting mark name");
        }
    }

    #[test]
    fn test_parse_insert_key_special_unknown() {
        // Line 59: _ => VimCommand::InsertChar(key) for special keys
        // Test that unknown special keys still insert the character
        assert!(matches!(
            parse_insert_key('\x01', true), // Ctrl+A (special but unknown)
            VimCommand::InsertChar('\x01')
        ));
    }

    #[test]
    fn test_parse_insert_key_carriage_return() {
        // Line 58: '\r' => VimCommand::InsertNewline
        assert!(matches!(
            parse_insert_key('\r', true),
            VimCommand::InsertNewline
        ));
    }
}
