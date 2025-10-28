// Vim editor state machine
// Implements the three vim modes: NORMAL, INSERT, COMMAND

use serde::{Deserialize, Serialize};
use std::fmt;

use super::buffer::{BufferId, VimBuffer};

/// Visual mode type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualMode {
    /// Character-wise visual selection (v)
    Character,

    /// Line-wise visual selection (V)
    Line,

    /// Block-wise visual selection (Ctrl+v)
    Block,
}

/// Register identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Register {
    /// Unnamed register (default for yank/delete/paste)
    Unnamed,

    /// Named registers a-z
    Named(char),

    /// Numbered registers 0-9 (0 = last yank, 1-9 = delete history)
    Numbered(u8),
}

/// Type of content stored in a register
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterType {
    /// Character-wise content
    Character,

    /// Line-wise content (entire lines)
    Line,

    /// Block-wise content (rectangular selection)
    Block,
}

/// Content stored in a register
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterContent {
    /// The text content
    pub text: String,

    /// Type of content (affects paste behavior)
    pub register_type: RegisterType,
}

impl RegisterContent {
    /// Create new register content
    pub fn new(text: String, register_type: RegisterType) -> Self {
        Self {
            text,
            register_type,
        }
    }
}

/// Vim editor mode
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VimMode {
    /// Normal mode - for navigation and commands
    #[default]
    Normal,

    /// Insert mode - for text editing
    Insert,

    /// Visual mode - for text selection
    Visual(VisualMode),

    /// Command mode - for ex commands (:w, :q, etc.)
    Command {
        /// Command buffer (what user is typing after :)
        buffer: String,
    },
}

impl fmt::Display for VimMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VimMode::Normal => write!(f, "NORMAL"),
            VimMode::Insert => write!(f, "INSERT"),
            VimMode::Visual(VisualMode::Character) => write!(f, "VISUAL"),
            VimMode::Visual(VisualMode::Line) => write!(f, "VISUAL LINE"),
            VimMode::Visual(VisualMode::Block) => write!(f, "VISUAL BLOCK"),
            VimMode::Command { .. } => write!(f, "COMMAND"),
        }
    }
}

/// Main vim editor state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimState {
    /// Current mode
    pub mode: VimMode,

    /// All open buffers
    pub buffers: im::HashMap<BufferId, VimBuffer>,

    /// Currently active buffer
    pub active_buffer: BufferId,

    /// Command history (for : commands)
    pub command_history: im::Vector<String>,

    /// Last status message
    pub message: String,

    /// Whether there are unsaved changes in any buffer
    pub modified: bool,

    /// Vim registers for yank/delete/paste operations
    pub registers: im::HashMap<Register, RegisterContent>,

    /// Currently selected register for next yank/delete/paste (None = unnamed)
    pub selected_register: Option<Register>,
}

impl VimState {
    /// Create a new vim state with an empty buffer
    pub fn new() -> Self {
        let buffer_id = BufferId(0);
        let buffer = VimBuffer::new(buffer_id, "/untitled".into());

        let mut buffers = im::HashMap::new();
        buffers.insert(buffer_id, buffer);

        Self {
            mode: VimMode::Normal,
            buffers,
            active_buffer: buffer_id,
            command_history: im::Vector::new(),
            message: String::new(),
            modified: false,
            registers: im::HashMap::new(),
            selected_register: None,
        }
    }

    /// Create a new vim state with text
    pub fn new_with_text(text: &str) -> Self {
        let buffer_id = BufferId(0);
        let buffer = VimBuffer::new_with_text(buffer_id, "/untitled".into(), text);
        let is_modified = buffer.modified;

        let mut buffers = im::HashMap::new();
        buffers.insert(buffer_id, buffer);

        Self {
            mode: VimMode::Normal,
            buffers,
            active_buffer: buffer_id,
            command_history: im::Vector::new(),
            message: String::new(),
            modified: is_modified,
            registers: im::HashMap::new(),
            selected_register: None,
        }
    }

    /// Get the currently active buffer
    pub fn current_buffer(&self) -> &VimBuffer {
        self.buffers
            .get(&self.active_buffer)
            .expect("Active buffer must always exist")
    }

    /// Get mutable reference to current buffer
    pub fn current_buffer_mut(&mut self) -> &mut VimBuffer {
        self.buffers
            .get_mut(&self.active_buffer)
            .expect("Active buffer must always exist")
    }

    /// Switch to a different mode
    pub fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
    }

    /// Set status message
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Update modified flag from buffers
    pub fn update_modified(&mut self) {
        self.modified = self.buffers.values().any(|b| b.modified);
    }

    /// Yank text to a register
    pub fn yank_to_register(
        &mut self,
        register: Register,
        text: String,
        register_type: RegisterType,
    ) {
        let content = RegisterContent::new(text, register_type);
        self.registers.insert(register, content);
    }

    /// Paste text from a register
    pub fn paste_from_register(&self, register: &Register) -> Option<&RegisterContent> {
        self.registers.get(register)
    }

    /// Get the current register for yank/paste operations
    /// Returns the selected register, or Unnamed if none selected
    pub fn get_current_register(&self) -> Register {
        self.selected_register.clone().unwrap_or(Register::Unnamed)
    }

    /// Set the register for next yank/paste operation
    pub fn set_current_register(&mut self, register: Register) {
        self.selected_register = Some(register);
    }

    /// Clear the register selection (revert to unnamed register)
    pub fn clear_register_selection(&mut self) {
        self.selected_register = None;
    }

    /// Yank the current line to the selected register
    pub fn yank_line(&mut self) -> Result<(), VimError> {
        let buffer = self.current_buffer();
        let line_text = buffer.current_line().to_string();

        let register = self.get_current_register();
        self.yank_to_register(register, line_text, RegisterType::Line);

        // Clear register selection after operation
        self.clear_register_selection();

        Ok(())
    }

    /// Paste from the selected register after the cursor
    pub fn paste_after(&mut self) -> Result<(), VimError> {
        let register = self.get_current_register();
        let content = self
            .paste_from_register(&register)
            .ok_or_else(|| VimError::General("Nothing in register".to_string()))?
            .clone();

        // Clear register selection after operation
        self.clear_register_selection();

        let buffer = self.current_buffer_mut();
        buffer.save_undo_point();

        match content.register_type {
            RegisterType::Line => {
                // Line-wise paste: insert after current line
                let insert_line = buffer.cursor.line + 1;
                let mut new_lines = buffer.lines.clone();
                new_lines.insert(insert_line, content.text);
                buffer.lines = new_lines;

                // Move cursor to start of inserted line
                buffer.cursor.line = insert_line;
                buffer.cursor.col = 0;
                buffer.mark_modified();
            }
            RegisterType::Character => {
                // Character-wise paste: insert after cursor position
                let line_idx = buffer.cursor.line;
                let col = buffer.cursor.col;
                let current_line = buffer.lines[line_idx].clone();

                // Insert after current position (col + 1)
                let insert_pos = (col + 1).min(current_line.len());
                let new_line = format!(
                    "{}{}{}",
                    &current_line[..insert_pos],
                    content.text,
                    &current_line[insert_pos..]
                );

                buffer.lines = buffer.lines.update(line_idx, new_line);
                buffer.cursor.col = insert_pos + content.text.len();
                buffer.mark_modified();
            }
            RegisterType::Block => {
                // Block-wise paste: for now, treat as character-wise
                // Full block paste implementation would be more complex
                let line_idx = buffer.cursor.line;
                let col = buffer.cursor.col;
                let current_line = buffer.lines[line_idx].clone();

                let insert_pos = (col + 1).min(current_line.len());
                let new_line = format!(
                    "{}{}{}",
                    &current_line[..insert_pos],
                    content.text,
                    &current_line[insert_pos..]
                );

                buffer.lines = buffer.lines.update(line_idx, new_line);
                buffer.cursor.col = insert_pos + content.text.len();
                buffer.mark_modified();
            }
        }

        buffer.clamp_cursor();
        self.update_modified();

        Ok(())
    }

    /// Yank visual selection to the selected register
    pub fn yank_visual(&mut self) -> Result<(), VimError> {
        // First, gather all data we need from the buffer
        let (yanked_text, register_type) = {
            let buffer = self.current_buffer();

            let anchor = buffer
                .visual_anchor
                .ok_or_else(|| VimError::General("No visual selection".to_string()))?;

            let visual_mode = buffer.visual_mode.clone().unwrap_or(VisualMode::Character);

            match visual_mode {
                VisualMode::Line => {
                    // Yank complete lines
                    let start_line = anchor.line.min(buffer.cursor.line);
                    let end_line = anchor.line.max(buffer.cursor.line);

                    let yanked_lines: Vec<String> = (start_line..=end_line)
                        .map(|i| buffer.lines.get(i).cloned().unwrap_or_default())
                        .collect();
                    let yanked_text = yanked_lines.join("\n");

                    (yanked_text, RegisterType::Line)
                }
                VisualMode::Character => {
                    // Yank character range
                    let (start, end) = if anchor.line < buffer.cursor.line
                        || (anchor.line == buffer.cursor.line && anchor.col <= buffer.cursor.col)
                    {
                        (anchor, buffer.cursor)
                    } else {
                        (buffer.cursor, anchor)
                    };

                    let yanked_text = if start.line == end.line {
                        // Single line selection
                        let line = buffer.lines[start.line].clone();
                        line[start.col..=end.col.min(line.len().saturating_sub(1))].to_string()
                    } else {
                        // Multi-line character selection
                        let mut text = String::new();
                        for line_idx in start.line..=end.line {
                            let line = buffer.lines.get(line_idx).cloned().unwrap_or_default();
                            if line_idx == start.line {
                                text.push_str(&line[start.col..]);
                            } else if line_idx == end.line {
                                text.push('\n');
                                text.push_str(&line[..=end.col.min(line.len().saturating_sub(1))]);
                            } else {
                                text.push('\n');
                                text.push_str(&line);
                            }
                        }
                        text
                    };

                    (yanked_text, RegisterType::Character)
                }
                VisualMode::Block => {
                    // Block-wise yank: yank rectangular region
                    let start_line = anchor.line.min(buffer.cursor.line);
                    let end_line = anchor.line.max(buffer.cursor.line);
                    let start_col = anchor.col.min(buffer.cursor.col);
                    let end_col = anchor.col.max(buffer.cursor.col);

                    let mut yanked_lines = Vec::new();
                    for line_idx in start_line..=end_line {
                        let line = buffer.lines.get(line_idx).cloned().unwrap_or_default();
                        let actual_end = end_col.min(line.len().saturating_sub(1));
                        if start_col < line.len() {
                            yanked_lines.push(line[start_col..=actual_end].to_string());
                        }
                    }
                    let yanked_text = yanked_lines.join("\n");

                    (yanked_text, RegisterType::Block)
                }
            }
        };

        // Now we can safely use self methods
        let register = self.get_current_register();
        self.yank_to_register(register, yanked_text, register_type);

        // Clear visual mode
        let buffer = self.current_buffer_mut();
        buffer.visual_anchor = None;
        buffer.visual_mode = None;

        // Clear register selection
        self.clear_register_selection();

        Ok(())
    }
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

/// Vim-specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VimError {
    /// Invalid command
    InvalidCommand(String),

    /// Buffer not found
    BufferNotFound(BufferId),

    /// File operation error
    FileError(String),

    /// Cannot perform operation in current mode
    InvalidMode(VimMode),

    /// General error
    General(String),
}

impl fmt::Display for VimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VimError::InvalidCommand(cmd) => write!(f, "E492: Not an editor command: {}", cmd),
            VimError::BufferNotFound(id) => write!(f, "E86: Buffer {} does not exist", id.0),
            VimError::FileError(msg) => write!(f, "E212: {}", msg),
            VimError::InvalidMode(mode) => write!(f, "E488: Trailing characters: {}", mode),
            VimError::General(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for VimError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vim::buffer::CursorPos;

    #[test]
    fn test_vim_state_creation() {
        let state = VimState::new();

        assert_eq!(state.mode, VimMode::Normal);
        assert_eq!(state.buffers.len(), 1);
        assert_eq!(state.active_buffer, BufferId(0));
        assert!(!state.modified);
    }

    #[test]
    fn test_vim_state_with_text() {
        let state = VimState::new_with_text("Hello\nWorld");

        assert_eq!(state.current_buffer().lines.len(), 2);
        assert_eq!(state.current_buffer().lines[0], "Hello");
        assert_eq!(state.current_buffer().lines[1], "World");
    }

    #[test]
    fn test_mode_transitions() {
        let mut state = VimState::new();

        assert_eq!(state.mode, VimMode::Normal);

        state.set_mode(VimMode::Insert);
        assert_eq!(state.mode, VimMode::Insert);

        state.set_mode(VimMode::Command {
            buffer: String::new(),
        });
        assert!(matches!(state.mode, VimMode::Command { .. }));

        state.set_mode(VimMode::Normal);
        assert_eq!(state.mode, VimMode::Normal);
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(VimMode::Normal.to_string(), "NORMAL");
        assert_eq!(VimMode::Insert.to_string(), "INSERT");
        assert_eq!(VimMode::Visual(VisualMode::Character).to_string(), "VISUAL");
        assert_eq!(VimMode::Visual(VisualMode::Line).to_string(), "VISUAL LINE");
        assert_eq!(
            VimMode::Visual(VisualMode::Block).to_string(),
            "VISUAL BLOCK"
        );
        assert_eq!(
            VimMode::Command {
                buffer: String::new()
            }
            .to_string(),
            "COMMAND"
        );
    }

    #[test]
    fn test_status_message() {
        let mut state = VimState::new();

        state.set_message("Test message");
        assert_eq!(state.message, "Test message");

        state.set_message(String::from("Another message"));
        assert_eq!(state.message, "Another message");
    }

    #[test]
    fn test_register_yank_and_paste() {
        let mut state = VimState::new();

        // Yank to unnamed register
        state.yank_to_register(
            Register::Unnamed,
            "test content".to_string(),
            RegisterType::Character,
        );

        // Paste from unnamed register
        let content = state.paste_from_register(&Register::Unnamed);
        assert!(content.is_some());
        assert_eq!(content.unwrap().text, "test content");
        assert_eq!(content.unwrap().register_type, RegisterType::Character);
    }

    #[test]
    fn test_register_named() {
        let mut state = VimState::new();

        // Yank to named register 'a'
        state.yank_to_register(
            Register::Named('a'),
            "content a".to_string(),
            RegisterType::Line,
        );

        // Yank to named register 'b'
        state.yank_to_register(
            Register::Named('b'),
            "content b".to_string(),
            RegisterType::Block,
        );

        // Verify both registers have separate content
        let content_a = state.paste_from_register(&Register::Named('a'));
        assert_eq!(content_a.unwrap().text, "content a");
        assert_eq!(content_a.unwrap().register_type, RegisterType::Line);

        let content_b = state.paste_from_register(&Register::Named('b'));
        assert_eq!(content_b.unwrap().text, "content b");
        assert_eq!(content_b.unwrap().register_type, RegisterType::Block);
    }

    #[test]
    fn test_register_numbered() {
        let mut state = VimState::new();

        // Yank to numbered register 0
        state.yank_to_register(
            Register::Numbered(0),
            "last yank".to_string(),
            RegisterType::Character,
        );

        // Verify numbered register works
        let content = state.paste_from_register(&Register::Numbered(0));
        assert_eq!(content.unwrap().text, "last yank");
    }

    #[test]
    fn test_register_selection() {
        let mut state = VimState::new();

        // Initially no register selected (defaults to unnamed)
        assert_eq!(state.get_current_register(), Register::Unnamed);

        // Select register 'a'
        state.set_current_register(Register::Named('a'));
        assert_eq!(state.get_current_register(), Register::Named('a'));

        // Clear selection
        state.clear_register_selection();
        assert_eq!(state.get_current_register(), Register::Unnamed);
    }

    #[test]
    fn test_paste_from_empty_register() {
        let state = VimState::new();

        // Paste from register that was never written to
        let content = state.paste_from_register(&Register::Named('x'));
        assert!(content.is_none());
    }

    #[test]
    fn test_register_overwrite() {
        let mut state = VimState::new();

        // Yank to register 'a'
        state.yank_to_register(
            Register::Named('a'),
            "first".to_string(),
            RegisterType::Character,
        );

        // Overwrite register 'a'
        state.yank_to_register(
            Register::Named('a'),
            "second".to_string(),
            RegisterType::Line,
        );

        // Verify register was overwritten
        let content = state.paste_from_register(&Register::Named('a'));
        assert_eq!(content.unwrap().text, "second");
        assert_eq!(content.unwrap().register_type, RegisterType::Line);
    }

    #[test]
    fn test_yank_line() {
        let mut state = VimState::new_with_text("Hello\nWorld\nTest");

        // Yank first line to unnamed register
        state.yank_line().unwrap();

        // Verify content was yanked
        let content = state.paste_from_register(&Register::Unnamed);
        assert_eq!(content.unwrap().text, "Hello");
        assert_eq!(content.unwrap().register_type, RegisterType::Line);
    }

    #[test]
    fn test_yank_line_to_named_register() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Select register 'a'
        state.set_current_register(Register::Named('a'));

        // Yank line
        state.yank_line().unwrap();

        // Verify yanked to register 'a'
        let content = state.paste_from_register(&Register::Named('a'));
        assert_eq!(content.unwrap().text, "Hello");

        // Verify register selection was cleared
        assert_eq!(state.selected_register, None);
    }

    #[test]
    fn test_paste_after_line() {
        let mut state = VimState::new_with_text("Line 1\nLine 2");

        // Yank line
        state.yank_line().unwrap();

        // Paste after current line
        state.paste_after().unwrap();

        // Verify line was inserted
        assert_eq!(state.current_buffer().text(), "Line 1\nLine 1\nLine 2");
        assert_eq!(state.current_buffer().cursor.line, 1);
        assert_eq!(state.current_buffer().cursor.col, 0);
    }

    #[test]
    fn test_paste_after_character() {
        let mut state = VimState::new_with_text("Hello World");

        // Yank some character-wise content
        state.yank_to_register(
            Register::Unnamed,
            "TEST".to_string(),
            RegisterType::Character,
        );

        // Position cursor at 'H' (position 0)
        state.current_buffer_mut().cursor.col = 0;

        // Paste after cursor
        state.paste_after().unwrap();

        // Should insert after position 0 (after 'H')
        assert_eq!(state.current_buffer().text(), "HTESTello World");
    }

    #[test]
    fn test_paste_empty_register() {
        let mut state = VimState::new_with_text("Hello");

        // Try to paste from empty register
        let result = state.paste_after();
        assert!(result.is_err());
    }

    #[test]
    fn test_yank_visual_line() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Set up visual line selection
        let buffer = state.current_buffer_mut();
        buffer.cursor.line = 1; // At "Line 2"
        buffer.visual_anchor = Some(CursorPos::new(0, 0));
        buffer.visual_mode = Some(VisualMode::Line);

        // Yank visual selection
        state.yank_visual().unwrap();

        // Verify lines were yanked
        let content = state.paste_from_register(&Register::Unnamed);
        assert_eq!(content.unwrap().text, "Line 1\nLine 2");
        assert_eq!(content.unwrap().register_type, RegisterType::Line);

        // Verify visual mode was cleared
        assert_eq!(state.current_buffer().visual_anchor, None);
        assert_eq!(state.current_buffer().visual_mode, None);
    }

    #[test]
    fn test_yank_visual_character_single_line() {
        let mut state = VimState::new_with_text("Hello World");

        // Select "Hello" (positions 0-4)
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 4);
        buffer.visual_anchor = Some(CursorPos::new(0, 0));
        buffer.visual_mode = Some(VisualMode::Character);

        // Yank visual selection
        state.yank_visual().unwrap();

        // Verify content
        let content = state.paste_from_register(&Register::Unnamed);
        assert_eq!(content.unwrap().text, "Hello");
        assert_eq!(content.unwrap().register_type, RegisterType::Character);
    }

    #[test]
    fn test_yank_visual_block() {
        let mut state = VimState::new_with_text("ABCD\nEFGH\nIJKL");

        // Select block (cols 1-2, lines 0-1)
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 2); // At 'G' (line 1, col 2)
        buffer.visual_anchor = Some(CursorPos::new(0, 1)); // At 'B' (line 0, col 1)
        buffer.visual_mode = Some(VisualMode::Block);

        // Yank visual selection
        state.yank_visual().unwrap();

        // Verify block was yanked (should be "BC\nFG")
        let content = state.paste_from_register(&Register::Unnamed);
        assert_eq!(content.unwrap().text, "BC\nFG");
        assert_eq!(content.unwrap().register_type, RegisterType::Block);
    }
}
