// Vim buffer implementation with undo/redo support via Command Pattern

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique buffer identifier
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferId(pub u64);

/// Cursor position in a buffer
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPos {
    /// Line number (0-indexed)
    pub line: usize,

    /// Column number (0-indexed)
    pub col: usize,
}

impl CursorPos {
    /// Create a new cursor position
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    /// Create a cursor at origin (0, 0)
    pub fn zero() -> Self {
        Self { line: 0, col: 0 }
    }
}

/// Memento Pattern: Captures buffer state for undo/redo
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferMemento {
    /// Captured lines
    pub lines: im::Vector<String>,

    /// Captured cursor position
    pub cursor: CursorPos,

    /// Timestamp of capture (for debugging)
    pub timestamp: u64,
}

impl BufferMemento {
    /// Capture current buffer state
    pub fn capture(buffer: &VimBuffer) -> Self {
        Self {
            lines: buffer.lines.clone(),
            cursor: buffer.cursor,
            timestamp: 0, // Timestamp not required for undo/redo ordering
        }
    }
}

/// A single vim buffer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimBuffer {
    /// Buffer ID
    pub id: BufferId,

    /// File path
    pub file_path: PathBuf,

    /// Buffer content (lines)
    pub lines: im::Vector<String>,

    /// Current cursor position
    pub cursor: CursorPos,

    /// Visual mode anchor position (where visual selection started)
    /// None when not in visual mode
    pub visual_anchor: Option<CursorPos>,

    /// Undo stack (command, memento pairs)
    ///
    /// Note: We store the memento BEFORE the command was executed,
    /// so we can restore to that state on undo.
    pub undo_stack: im::Vector<BufferMemento>,

    /// Redo stack (command, memento pairs)
    pub redo_stack: im::Vector<BufferMemento>,

    /// Has unsaved changes
    pub modified: bool,
}

impl VimBuffer {
    /// Create a new empty buffer
    pub fn new(id: BufferId, file_path: PathBuf) -> Self {
        Self {
            id,
            file_path,
            lines: im::vector![String::new()],
            cursor: CursorPos::zero(),
            visual_anchor: None,
            undo_stack: im::Vector::new(),
            redo_stack: im::Vector::new(),
            modified: false,
        }
    }

    /// Create a buffer with initial text
    pub fn new_with_text(id: BufferId, file_path: PathBuf, text: &str) -> Self {
        let lines: im::Vector<String> = if text.is_empty() {
            im::vector![String::new()]
        } else {
            text.lines().map(String::from).collect()
        };

        Self {
            id,
            file_path,
            lines,
            cursor: CursorPos::zero(),
            visual_anchor: None,
            undo_stack: im::Vector::new(),
            redo_stack: im::Vector::new(),
            modified: false,
        }
    }

    /// Get text content as a single string
    pub fn text(&self) -> String {
        let mut result = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            result.push_str(line);
            if i < self.lines.len() - 1 {
                result.push('\n');
            }
        }
        result
    }

    /// Get current line
    pub fn current_line(&self) -> &str {
        self.lines
            .get(self.cursor.line)
            .map(|s: &String| s.as_str())
            .unwrap_or("")
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Check if cursor is at valid position
    pub fn is_cursor_valid(&self) -> bool {
        self.cursor.line < self.lines.len() && self.cursor.col <= self.current_line().len()
    }

    /// Clamp cursor to valid position
    pub fn clamp_cursor(&mut self) {
        if self.cursor.line >= self.lines.len() {
            self.cursor.line = self.lines.len().saturating_sub(1);
        }

        let line_len = self.current_line().len();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    /// Save current state to undo stack (call before making changes)
    pub fn save_undo_point(&mut self) {
        let memento = BufferMemento::capture(self);
        self.undo_stack.push_back(memento);

        // Clear redo stack on new edits
        self.redo_stack = im::Vector::new();
    }

    /// Undo last change
    pub fn undo(&mut self) -> bool {
        if let Some(memento) = self.undo_stack.last() {
            // Save current state to redo stack
            let current_memento = BufferMemento::capture(self);
            self.redo_stack.push_back(current_memento);

            // Restore previous state
            self.lines = memento.lines.clone();
            self.cursor = memento.cursor;

            // Pop from undo stack
            self.undo_stack.pop_back();

            true
        } else {
            false
        }
    }

    /// Redo last undone change
    pub fn redo(&mut self) -> bool {
        if let Some(memento) = self.redo_stack.last() {
            // Save current state to undo stack
            let current_memento = BufferMemento::capture(self);
            self.undo_stack.push_back(current_memento);

            // Restore next state
            self.lines = memento.lines.clone();
            self.cursor = memento.cursor;

            // Pop from redo stack
            self.redo_stack.pop_back();

            true
        } else {
            false
        }
    }

    /// Mark buffer as modified
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    /// Mark buffer as saved (not modified)
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = VimBuffer::new(BufferId(0), "/test.txt".into());

        assert_eq!(buffer.id, BufferId(0));
        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "");
        assert_eq!(buffer.cursor, CursorPos::zero());
        assert!(!buffer.modified);
    }

    #[test]
    fn test_buffer_with_text() {
        let buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Hello\nWorld");

        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[0], "Hello");
        assert_eq!(buffer.lines[1], "World");
    }

    #[test]
    fn test_buffer_text_extraction() {
        let buffer =
            VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Line 1\nLine 2\nLine 3");

        assert_eq!(buffer.text(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_current_line() {
        let mut buffer =
            VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "First\nSecond\nThird");

        assert_eq!(buffer.current_line(), "First");

        buffer.cursor.line = 1;
        assert_eq!(buffer.current_line(), "Second");

        buffer.cursor.line = 2;
        assert_eq!(buffer.current_line(), "Third");
    }

    #[test]
    fn test_cursor_clamping() {
        let mut buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Hello");

        buffer.cursor = CursorPos::new(10, 10);
        buffer.clamp_cursor();

        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.col, 5); // "Hello" has 5 characters
    }

    #[test]
    fn test_undo_redo_empty_stack() {
        let mut buffer = VimBuffer::new(BufferId(0), "/test.txt".into());

        assert!(!buffer.undo());
        assert!(!buffer.redo());
    }

    #[test]
    fn test_undo_redo_single_change() {
        let mut buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Hello");

        // Save undo point before edit
        buffer.save_undo_point();

        // Make a change
        buffer.lines = im::vector![String::from("Hello World")];
        buffer.mark_modified();

        assert_eq!(buffer.text(), "Hello World");
        assert!(buffer.modified);

        // Undo should restore original
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "Hello");

        // Redo should restore edit
        assert!(buffer.redo());
        assert_eq!(buffer.text(), "Hello World");
    }

    #[test]
    fn test_modified_flag() {
        let mut buffer = VimBuffer::new(BufferId(0), "/test.txt".into());

        assert!(!buffer.modified);

        buffer.mark_modified();
        assert!(buffer.modified);

        buffer.mark_saved();
        assert!(!buffer.modified);
    }

    #[test]
    fn test_memento_capture() {
        let buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Test");
        let memento = BufferMemento::capture(&buffer);

        assert_eq!(memento.lines, buffer.lines);
        assert_eq!(memento.cursor, buffer.cursor);
    }

    #[test]
    fn test_buffer_id_equality() {
        assert_eq!(BufferId(0), BufferId(0));
        assert_ne!(BufferId(0), BufferId(1));
    }

    #[test]
    fn test_cursor_pos_equality() {
        assert_eq!(CursorPos::new(0, 0), CursorPos::zero());
        assert_ne!(CursorPos::new(1, 0), CursorPos::zero());
    }

    // Edge case tests for mutation testing coverage

    #[test]
    fn test_is_cursor_valid_empty_buffer() {
        let buffer = VimBuffer::new(BufferId(0), "/test.txt".into());

        // Empty buffer has one empty line, cursor at (0,0) is valid
        assert!(buffer.is_cursor_valid());

        // Cursor at column 1 on empty line is invalid
        let mut buffer2 = buffer.clone();
        buffer2.cursor.col = 1;
        assert!(!buffer2.is_cursor_valid());
    }

    #[test]
    fn test_is_cursor_valid_at_line_end() {
        let mut buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Hello");

        // Cursor at exact end of line (col = length) should be valid
        buffer.cursor = CursorPos::new(0, 5);
        assert!(buffer.is_cursor_valid());

        // Cursor past end of line should be invalid
        buffer.cursor.col = 6;
        assert!(!buffer.is_cursor_valid());
    }

    #[test]
    fn test_is_cursor_valid_past_last_line() {
        let buffer = VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Line1\nLine2");

        // Buffer has 2 lines (0 and 1), cursor at line 2 is invalid
        let mut buffer2 = buffer.clone();
        buffer2.cursor.line = 2;
        assert!(!buffer2.is_cursor_valid());
    }

    #[test]
    fn test_is_cursor_valid_boundary_conditions() {
        let buffer =
            VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "Short\nMediumLine\nX");

        // Valid positions
        let mut test_buffer = buffer.clone();
        test_buffer.cursor = CursorPos::new(0, 0); // Start of line 0
        assert!(test_buffer.is_cursor_valid());

        test_buffer.cursor = CursorPos::new(0, 5); // End of "Short"
        assert!(test_buffer.is_cursor_valid());

        test_buffer.cursor = CursorPos::new(1, 10); // End of "MediumLine"
        assert!(test_buffer.is_cursor_valid());

        test_buffer.cursor = CursorPos::new(2, 1); // End of "X"
        assert!(test_buffer.is_cursor_valid());

        // Invalid positions
        test_buffer.cursor = CursorPos::new(0, 6); // Past end of "Short"
        assert!(!test_buffer.is_cursor_valid());

        test_buffer.cursor = CursorPos::new(1, 11); // Past end of "MediumLine"
        assert!(!test_buffer.is_cursor_valid());

        test_buffer.cursor = CursorPos::new(3, 0); // Past last line
        assert!(!test_buffer.is_cursor_valid());
    }

    #[test]
    fn test_clamp_cursor_long_to_short_line() {
        let mut buffer =
            VimBuffer::new_with_text(BufferId(0), "/test.txt".into(), "LongLine\nShort");

        // Set cursor to end of long line
        buffer.cursor = CursorPos::new(0, 8);
        assert!(buffer.is_cursor_valid());

        // Move to short line (should be clamped)
        buffer.cursor.line = 1;
        buffer.clamp_cursor();

        // Cursor should be at end of "Short" (length 5), not at column 8
        assert!(buffer.cursor.col <= 5);
        assert!(buffer.is_cursor_valid());
    }
}
