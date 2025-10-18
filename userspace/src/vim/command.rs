// Command Pattern for vim editing operations
// Provides undo/redo functionality through encapsulated commands

use super::buffer::{CursorPos, VimBuffer};
use super::state::VimError;

/// Enum representing all vim commands
#[derive(Clone, Debug)]
pub enum VimCommand {
    // Navigation
    /// Move cursor left (h)
    MoveLeft,
    /// Move cursor right (l)
    MoveRight,
    /// Move cursor up (k)
    MoveUp,
    /// Move cursor down (j)
    MoveDown,
    /// Move to start of line (0)
    MoveLineStart,
    /// Move to end of line ($)
    MoveLineEnd,
    /// Move forward one word (w)
    MoveWordForward,
    /// Move backward one word (b)
    MoveWordBackward,

    // Insert mode
    /// Enter insert mode before cursor (i)
    InsertBefore,
    /// Enter insert mode after cursor (a)
    InsertAfter,
    /// Insert line below and enter insert mode (o)
    InsertLineBelow,
    /// Insert line above and enter insert mode (O)
    InsertLineAbove,

    // Editing
    /// Delete character under cursor (x)
    DeleteChar,
    /// Delete entire line (dd)
    DeleteLine,
    /// Yank (copy) line (yy)
    YankLine,
    /// Put (paste) after cursor (p)
    PutAfter,

    // Undo/Redo
    /// Undo last change (u)
    Undo,
    /// Redo undone change (Ctrl+r)
    Redo,

    // Mode switching
    /// Enter insert mode
    EnterInsertMode,
    /// Enter normal mode (Esc)
    EnterNormalMode,
    /// Enter command mode (:)
    EnterCommandMode,

    // Text insertion (for insert mode)
    /// Insert a character at cursor
    InsertChar(char),
    /// Insert newline and move to next line
    InsertNewline,
    /// Delete character before cursor
    Backspace,
}

impl VimCommand {
    /// Execute the command on a buffer
    ///
    /// Returns Result<bool, VimError> where:
    /// - Ok(true) = command executed and modified buffer
    /// - Ok(false) = command executed but no modification
    /// - Err(_) = command failed
    pub fn execute(&self, buffer: &mut VimBuffer) -> Result<bool, VimError> {
        match self {
            // Navigation commands (no modification)
            VimCommand::MoveLeft => {
                if buffer.cursor.col > 0 {
                    buffer.cursor.col -= 1;
                }
                Ok(false)
            }

            VimCommand::MoveRight => {
                let line_len = buffer.current_line().len();
                if buffer.cursor.col < line_len {
                    buffer.cursor.col += 1;
                }
                Ok(false)
            }

            VimCommand::MoveUp => {
                if buffer.cursor.line > 0 {
                    buffer.cursor.line -= 1;
                    buffer.clamp_cursor();
                }
                Ok(false)
            }

            VimCommand::MoveDown => {
                if buffer.cursor.line < buffer.line_count() - 1 {
                    buffer.cursor.line += 1;
                    buffer.clamp_cursor();
                }
                Ok(false)
            }

            VimCommand::MoveLineStart => {
                buffer.cursor.col = 0;
                Ok(false)
            }

            VimCommand::MoveLineEnd => {
                buffer.cursor.col = buffer.current_line().len();
                Ok(false)
            }

            // Editing commands (with modification)
            VimCommand::InsertChar(c) => {
                buffer.save_undo_point();

                let line_idx = buffer.cursor.line;
                let col = buffer.cursor.col;
                let current_line = buffer.lines[line_idx].clone();

                let new_line = format!("{}{}{}", &current_line[..col], c, &current_line[col..]);

                buffer.lines = buffer.lines.update(line_idx, new_line);
                buffer.cursor.col += 1;
                buffer.mark_modified();

                Ok(true)
            }

            VimCommand::InsertNewline => {
                buffer.save_undo_point();

                let line_idx = buffer.cursor.line;
                let col = buffer.cursor.col;
                let current_line = buffer.lines[line_idx].clone();

                let before = current_line[..col].to_string();
                let after = current_line[col..].to_string();

                let mut new_lines = buffer.lines.update(line_idx, before);
                new_lines.insert(line_idx + 1, after);
                buffer.lines = new_lines;

                buffer.cursor.line += 1;
                buffer.cursor.col = 0;
                buffer.mark_modified();

                Ok(true)
            }

            VimCommand::Backspace => {
                if buffer.cursor.col > 0 {
                    buffer.save_undo_point();

                    let line_idx = buffer.cursor.line;
                    let col = buffer.cursor.col;
                    let current_line = buffer.lines[line_idx].clone();

                    let new_line = format!("{}{}", &current_line[..col - 1], &current_line[col..]);

                    buffer.lines = buffer.lines.update(line_idx, new_line);
                    buffer.cursor.col -= 1;
                    buffer.mark_modified();

                    Ok(true)
                } else if buffer.cursor.line > 0 {
                    // Join with previous line
                    buffer.save_undo_point();

                    let line_idx = buffer.cursor.line;
                    let prev_line = buffer.lines[line_idx - 1].clone();
                    let current_line = buffer.lines[line_idx].clone();

                    let joined = format!("{}{}", prev_line, current_line);
                    let col = prev_line.len();

                    let mut new_lines = buffer.lines.update(line_idx - 1, joined);
                    new_lines.remove(line_idx);
                    buffer.lines = new_lines;

                    buffer.cursor.line -= 1;
                    buffer.cursor.col = col;
                    buffer.mark_modified();

                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            VimCommand::DeleteChar => {
                let line_len = buffer.current_line().len();
                if buffer.cursor.col < line_len {
                    buffer.save_undo_point();

                    let line_idx = buffer.cursor.line;
                    let col = buffer.cursor.col;
                    let current_line = buffer.lines[line_idx].clone();

                    let new_line = format!("{}{}", &current_line[..col], &current_line[col + 1..]);

                    buffer.lines = buffer.lines.update(line_idx, new_line);
                    buffer.mark_modified();

                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            VimCommand::DeleteLine => {
                if buffer.line_count() > 1 {
                    buffer.save_undo_point();

                    let line_idx = buffer.cursor.line;
                    let mut new_lines = buffer.lines.clone();
                    new_lines.remove(line_idx);
                    buffer.lines = new_lines;

                    if buffer.cursor.line >= buffer.line_count() {
                        buffer.cursor.line = buffer.line_count() - 1;
                    }
                    buffer.clamp_cursor();
                    buffer.mark_modified();

                    Ok(true)
                } else {
                    // Last line - clear it instead of deleting
                    buffer.save_undo_point();

                    buffer.lines = buffer.lines.update(0, String::new());
                    buffer.cursor = CursorPos::zero();
                    buffer.mark_modified();

                    Ok(true)
                }
            }

            VimCommand::Undo => Ok(buffer.undo()),

            VimCommand::Redo => Ok(buffer.redo()),

            // Other commands - stubs for now
            _ => Ok(false),
        }
    }

    /// Get a human-readable description of the command
    pub fn description(&self) -> &str {
        match self {
            VimCommand::MoveLeft => "Move cursor left",
            VimCommand::MoveRight => "Move cursor right",
            VimCommand::MoveUp => "Move cursor up",
            VimCommand::MoveDown => "Move cursor down",
            VimCommand::MoveLineStart => "Move to line start",
            VimCommand::MoveLineEnd => "Move to line end",
            VimCommand::MoveWordForward => "Move word forward",
            VimCommand::MoveWordBackward => "Move word backward",
            VimCommand::InsertBefore => "Insert before cursor",
            VimCommand::InsertAfter => "Insert after cursor",
            VimCommand::InsertLineBelow => "Insert line below",
            VimCommand::InsertLineAbove => "Insert line above",
            VimCommand::DeleteChar => "Delete character",
            VimCommand::DeleteLine => "Delete line",
            VimCommand::YankLine => "Yank line",
            VimCommand::PutAfter => "Put after cursor",
            VimCommand::Undo => "Undo",
            VimCommand::Redo => "Redo",
            VimCommand::EnterInsertMode => "Enter insert mode",
            VimCommand::EnterNormalMode => "Enter normal mode",
            VimCommand::EnterCommandMode => "Enter command mode",
            VimCommand::InsertChar(_) => "Insert character",
            VimCommand::InsertNewline => "Insert newline",
            VimCommand::Backspace => "Backspace",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vim::buffer::BufferId;

    fn create_test_buffer() -> VimBuffer {
        VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Hello\nWorld")
    }

    #[test]
    fn test_move_left() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 3);

        let result = VimCommand::MoveLeft.execute(&mut buffer);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No modification
        assert_eq!(buffer.cursor.col, 2);
    }

    #[test]
    fn test_move_right() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 2);

        let result = VimCommand::MoveRight.execute(&mut buffer);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert_eq!(buffer.cursor.col, 3);
    }

    #[test]
    fn test_move_up_down() {
        let mut buffer = create_test_buffer();

        VimCommand::MoveDown.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 1);

        VimCommand::MoveUp.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 0);
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 5); // After "Hello"

        let result = VimCommand::InsertChar('!').execute(&mut buffer);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Modified
        assert_eq!(buffer.lines[0], "Hello!");
        assert_eq!(buffer.cursor.col, 6);
        assert!(buffer.modified);
    }

    #[test]
    fn test_insert_newline() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 2); // After "He"

        VimCommand::InsertNewline.execute(&mut buffer).unwrap();

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[0], "He");
        assert_eq!(buffer.lines[1], "llo");
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.col, 0);
    }

    #[test]
    fn test_backspace() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 5); // After "Hello"

        VimCommand::Backspace.execute(&mut buffer).unwrap();

        assert_eq!(buffer.lines[0], "Hell");
        assert_eq!(buffer.cursor.col, 4);
    }

    #[test]
    fn test_backspace_line_join() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(1, 0); // Start of "World"

        VimCommand::Backspace.execute(&mut buffer).unwrap();

        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "HelloWorld");
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.col, 5);
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 0); // At "H"

        VimCommand::DeleteChar.execute(&mut buffer).unwrap();

        assert_eq!(buffer.lines[0], "ello");
    }

    #[test]
    fn test_delete_line() {
        let mut buffer = create_test_buffer();

        VimCommand::DeleteLine.execute(&mut buffer).unwrap();

        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "World");
    }

    #[test]
    fn test_undo_redo() {
        let mut buffer = create_test_buffer();

        // Make a change
        VimCommand::InsertChar('!').execute(&mut buffer).unwrap();
        assert_eq!(buffer.lines[0], "!Hello");

        // Undo
        VimCommand::Undo.execute(&mut buffer).unwrap();
        assert_eq!(buffer.lines[0], "Hello");

        // Redo
        VimCommand::Redo.execute(&mut buffer).unwrap();
        assert_eq!(buffer.lines[0], "!Hello");
    }

    #[test]
    fn test_command_descriptions() {
        assert_eq!(VimCommand::MoveLeft.description(), "Move cursor left");
        assert_eq!(VimCommand::DeleteLine.description(), "Delete line");
        assert_eq!(VimCommand::Undo.description(), "Undo");
    }

    // Behavioral tests for mutation testing coverage

    #[test]
    fn test_move_line_start_execution() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 3); // Middle of "Hello"

        let result = VimCommand::MoveLineStart.execute(&mut buffer);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No modification
        assert_eq!(buffer.cursor.col, 0); // Moved to column 0
    }

    #[test]
    fn test_move_line_end_execution() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 0); // Start of "Hello"

        let result = VimCommand::MoveLineEnd.execute(&mut buffer);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert_eq!(buffer.cursor.col, 5); // Moved to end of "Hello" (length 5)
    }

    #[test]
    fn test_move_down_boundary() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 0);

        // Move down once - should work
        VimCommand::MoveDown.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 1);

        // Try to move down again - should stay at line 1 (last line)
        VimCommand::MoveDown.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 1);
    }

    #[test]
    fn test_move_up_boundary() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 0);

        // Try to move up - should stay at line 0 (first line)
        VimCommand::MoveUp.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 0);
    }

    #[test]
    fn test_move_left_boundary() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 0);

        // Try to move left - should stay at column 0
        VimCommand::MoveLeft.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.col, 0);
    }

    #[test]
    fn test_move_right_boundary() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 4); // At 'o' in "Hello"

        // Move right to end of line
        VimCommand::MoveRight.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.col, 5); // At end

        // Try to move right again - should stay at end
        VimCommand::MoveRight.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.col, 5);
    }

    #[test]
    fn test_move_down_with_shorter_line() {
        let mut buffer =
            VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "LongLine\nShort");
        buffer.cursor = CursorPos::new(0, 7); // At end of "LongLine"

        VimCommand::MoveDown.execute(&mut buffer).unwrap();

        // Cursor should be clamped to length of "Short" (5)
        assert_eq!(buffer.cursor.line, 1);
        assert!(buffer.cursor.col <= 5);
    }

    #[test]
    fn test_delete_char_at_line_end() {
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 5); // After "Hello"

        // Should not delete when at end of line
        VimCommand::DeleteChar.execute(&mut buffer).unwrap();
        assert_eq!(buffer.lines[0], "Hello");
    }

    // Note: InsertLineBelow and InsertLineAbove are not yet implemented
    // (they're stubs returning Ok(false)), so no execution tests for those yet

    // Additional edge case tests targeting specific mutation survivors

    #[test]
    fn test_move_down_exact_arithmetic() {
        // Target: command.rs:105 - verify exact arithmetic (- not / or +)
        let mut buffer = VimBuffer::new_with_text(
            BufferId(0),
            "/test.txt".into(),
            "line0\nline1\nline2\nline3",
        );
        buffer.cursor = CursorPos::new(2, 0); // On line 2 (of 0,1,2,3)

        // Should move to line 3 (line_count=4, so 2 < 4-1 is true)
        VimCommand::MoveDown.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 3);

        // Should NOT move (3 < 4-1 is false, already at last line)
        VimCommand::MoveDown.execute(&mut buffer).unwrap();
        assert_eq!(buffer.cursor.line, 3); // Still on line 3
    }

    #[test]
    fn test_backspace_at_line_0_col_0() {
        // Target: command.rs:175 - backspace when cursor.line = 0, should not join
        let mut buffer = create_test_buffer();
        buffer.cursor = CursorPos::new(0, 0); // At very start

        let result = VimCommand::Backspace.execute(&mut buffer);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No modification

        // Buffer should be unchanged
        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[0], "Hello");
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.col, 0);
    }

    #[test]
    fn test_delete_line_single_line_buffer() {
        // Target: command.rs:221 - DeleteLine when line_count = 1
        let mut buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "OnlyLine");
        buffer.cursor = CursorPos::new(0, 0);

        VimCommand::DeleteLine.execute(&mut buffer).unwrap();

        // Should clear the line, not delete it (buffer must have at least 1 line)
        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "");
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.col, 0);
    }

    #[test]
    fn test_delete_line_cursor_past_end_adjustment() {
        // Target: command.rs:229-230 - cursor adjustment after deletion
        let mut buffer =
            VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "line0\nline1\nline2");
        // Position cursor on last line
        buffer.cursor = CursorPos::new(2, 3);

        VimCommand::DeleteLine.execute(&mut buffer).unwrap();

        // After deleting line 2, buffer has 2 lines (0,1)
        // Cursor should be adjusted to line 1 (line_count()-1)
        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.cursor.line, 1); // Moved to line 1 (the new last line)
        assert!(buffer.cursor.col <= buffer.lines[1].len());
    }

    #[test]
    fn test_clamp_cursor_at_exact_line_end() {
        // Target: buffer.rs:154 - clamp_cursor when cursor.col = line_len (not >)
        let mut buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Short\nMedium");

        // Set cursor to exact end of "Short" (col = 5)
        buffer.cursor = CursorPos::new(0, 5);
        assert!(buffer.is_cursor_valid()); // Should be valid

        // Clamp shouldn't change it
        buffer.clamp_cursor();
        assert_eq!(buffer.cursor.col, 5); // Still at 5

        // Now set cursor past end (col = 6)
        buffer.cursor.col = 6;
        assert!(!buffer.is_cursor_valid()); // Invalid

        // Clamp should fix it
        buffer.clamp_cursor();
        assert_eq!(buffer.cursor.col, 5); // Clamped to 5
    }
}
