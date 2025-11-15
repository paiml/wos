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

/// Mark identifier (local a-z, global A-Z, special marks)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarkId {
    /// Local marks (a-z) - per-buffer
    Local(char),

    /// Global marks (A-Z) - across all buffers
    Global(char),

    /// Special marks
    Special(SpecialMark),
}

/// Special predefined marks
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialMark {
    /// Last visual selection start
    VisualStart,

    /// Last visual selection end
    VisualEnd,

    /// Last edit position
    LastEdit,

    /// Last insert position
    LastInsert,
}

/// A saved position in a buffer
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    /// Buffer this mark points to
    pub buffer_id: BufferId,

    /// Line number (0-indexed)
    pub line: usize,

    /// Column number (0-indexed)
    pub col: usize,
}

impl Mark {
    /// Create a new mark at the given position
    pub fn new(buffer_id: BufferId, line: usize, col: usize) -> Self {
        Self {
            buffer_id,
            line,
            col,
        }
    }
}

/// Jump list entry for Ctrl-O/Ctrl-I navigation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JumpEntry {
    /// Buffer ID
    pub buffer_id: BufferId,

    /// Line number
    pub line: usize,

    /// Column number
    pub col: usize,
}

impl JumpEntry {
    /// Create a new jump entry
    pub fn new(buffer_id: BufferId, line: usize, col: usize) -> Self {
        Self {
            buffer_id,
            line,
            col,
        }
    }
}

/// Jump list for navigation history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JumpList {
    /// List of jump positions
    entries: im::Vector<JumpEntry>,

    /// Current position in the list
    current: usize,

    /// Maximum number of jumps to remember
    max_size: usize,
}

impl JumpList {
    /// Create a new empty jump list
    pub fn new() -> Self {
        Self {
            entries: im::Vector::new(),
            current: 0,
            max_size: 100, // Vim default
        }
    }

    /// Add a new jump position
    pub fn push(&mut self, entry: JumpEntry) {
        // If we've jumped back in history, discard the "future" entries
        // (when we make a new jump, discard everything after current)
        if self.current < self.entries.len() {
            self.entries = self.entries.take(self.current + 1);
        }

        // Add new entry
        self.entries.push_back(entry);

        // Update current to point to the new entry
        self.current = self.entries.len().saturating_sub(1);

        // Trim if exceeded max size
        if self.entries.len() > self.max_size {
            self.entries = self.entries.skip(1);
            // After skipping, adjust current
            if self.current > 0 {
                self.current -= 1;
            }
        }
    }

    /// Jump backward (Ctrl-O)
    pub fn jump_back(&mut self) -> Option<&JumpEntry> {
        if self.entries.is_empty() {
            return None;
        }
        if self.current > 0 {
            self.current -= 1;
            self.entries.get(self.current)
        } else {
            None
        }
    }

    /// Jump forward (Ctrl-I)
    pub fn jump_forward(&mut self) -> Option<&JumpEntry> {
        if self.entries.is_empty() {
            return None;
        }
        if self.current + 1 < self.entries.len() {
            self.current += 1;
            self.entries.get(self.current)
        } else {
            None
        }
    }

    /// Get current jump entry
    pub fn current(&self) -> Option<&JumpEntry> {
        if self.entries.is_empty() {
            None
        } else {
            self.entries.get(self.current)
        }
    }

    /// Check if we can jump backward
    pub fn can_jump_back(&self) -> bool {
        // Can jump back if we have entries and current points after first entry
        !self.entries.is_empty() && self.current > 0
    }

    /// Check if we can jump forward
    pub fn can_jump_forward(&self) -> bool {
        // Can jump forward if current is not at the end
        !self.entries.is_empty() && self.current + 1 < self.entries.len()
    }
}

impl Default for JumpList {
    fn default() -> Self {
        Self::new()
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

/// Parser state for handling multi-key sequences
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ParserState {
    /// Default state - waiting for next command
    #[default]
    Normal,

    /// Waiting for register name after " key
    AwaitingRegister,

    /// Waiting for mark name after m key (to set mark)
    AwaitingMarkToSet,

    /// Waiting for mark name after ' key (jump to line)
    AwaitingMarkJumpLine,

    /// Waiting for mark name after ` key (jump to exact position)
    AwaitingMarkJumpExact,

    /// Waiting for second y in yy command
    AwaitingYankTarget,
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

    /// Marks for position navigation (ma, 'a, etc.)
    pub marks: im::HashMap<MarkId, Mark>,

    /// Jump list for Ctrl-O/Ctrl-I navigation
    pub jump_list: JumpList,

    /// Parser state for multi-key command sequences
    pub parser_state: ParserState,
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
            marks: im::HashMap::new(),
            jump_list: JumpList::new(),
            parser_state: ParserState::Normal,
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
            marks: im::HashMap::new(),
            jump_list: JumpList::new(),
            parser_state: ParserState::Normal,
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

    // ===== Mark Operations =====

    /// Set a mark at the current cursor position
    pub fn set_mark(&mut self, mark_id: MarkId) -> Result<(), VimError> {
        let buffer = self.current_buffer();
        let mark = Mark::new(self.active_buffer, buffer.cursor.line, buffer.cursor.col);
        self.marks.insert(mark_id, mark);
        Ok(())
    }

    /// Jump to a previously set mark
    pub fn jump_to_mark(&mut self, mark_id: &MarkId) -> Result<(), VimError> {
        let mark = self
            .marks
            .get(mark_id)
            .ok_or_else(|| VimError::General(format!("Mark not set: {:?}", mark_id)))?
            .clone();

        // Record current position in jump list before jumping
        self.record_jump();

        // Switch buffer if needed
        if mark.buffer_id != self.active_buffer {
            if !self.buffers.contains_key(&mark.buffer_id) {
                return Err(VimError::BufferNotFound(mark.buffer_id));
            }
            self.active_buffer = mark.buffer_id;
        }

        // Move cursor to mark position
        let buffer = self.current_buffer_mut();
        buffer.cursor.line = mark.line;
        buffer.cursor.col = mark.col;
        buffer.clamp_cursor();

        Ok(())
    }

    /// Get a mark by its ID
    pub fn get_mark(&self, mark_id: &MarkId) -> Option<&Mark> {
        self.marks.get(mark_id)
    }

    // ===== Jump List Operations =====

    /// Record current position in jump list
    pub fn record_jump(&mut self) {
        let buffer = self.current_buffer();
        let entry = JumpEntry::new(self.active_buffer, buffer.cursor.line, buffer.cursor.col);
        self.jump_list.push(entry);
    }

    /// Jump backward in jump list (Ctrl-O)
    pub fn jump_back(&mut self) -> Result<(), VimError> {
        // First, record current position before jumping
        // This ensures the position we're jumping FROM is in the list
        let buffer = self.current_buffer();
        let current_pos = JumpEntry::new(self.active_buffer, buffer.cursor.line, buffer.cursor.col);

        // Only record if it's different from the last entry
        if self.jump_list.current() != Some(&current_pos) {
            self.record_jump();
        }

        let entry = self
            .jump_list
            .jump_back()
            .ok_or_else(|| VimError::General("Already at oldest jump".to_string()))?
            .clone();

        // Switch buffer if needed
        if entry.buffer_id != self.active_buffer {
            if !self.buffers.contains_key(&entry.buffer_id) {
                return Err(VimError::BufferNotFound(entry.buffer_id));
            }
            self.active_buffer = entry.buffer_id;
        }

        // Move cursor
        let buffer = self.current_buffer_mut();
        buffer.cursor.line = entry.line;
        buffer.cursor.col = entry.col;
        buffer.clamp_cursor();

        Ok(())
    }

    /// Jump forward in jump list (Ctrl-I)
    pub fn jump_forward(&mut self) -> Result<(), VimError> {
        let entry = self
            .jump_list
            .jump_forward()
            .ok_or_else(|| VimError::General("Already at newest jump".to_string()))?
            .clone();

        // Switch buffer if needed
        if entry.buffer_id != self.active_buffer {
            if !self.buffers.contains_key(&entry.buffer_id) {
                return Err(VimError::BufferNotFound(entry.buffer_id));
            }
            self.active_buffer = entry.buffer_id;
        }

        // Move cursor
        let buffer = self.current_buffer_mut();
        buffer.cursor.line = entry.line;
        buffer.cursor.col = entry.col;
        buffer.clamp_cursor();

        Ok(())
    }

    /// Check if we can jump backward
    pub fn can_jump_back(&self) -> bool {
        self.jump_list.can_jump_back()
    }

    /// Check if we can jump forward
    pub fn can_jump_forward(&self) -> bool {
        self.jump_list.can_jump_forward()
    }

    /// Handle a normal mode key press with parser state machine
    ///
    /// This method processes keys in normal mode, handling multi-key sequences
    /// like "x (register selection), mx (set mark), 'x (jump to mark), etc.
    ///
    /// Returns Ok(true) if a command was executed and may have modified the buffer.
    /// Returns Ok(false) if no modification occurred or command is incomplete.
    pub fn handle_normal_key(&mut self, key: char) -> Result<bool, VimError> {
        match self.parser_state {
            ParserState::Normal => {
                // First key of potential multi-key sequence
                match key {
                    '"' => {
                        self.parser_state = ParserState::AwaitingRegister;
                        Ok(false)
                    }
                    'm' => {
                        self.parser_state = ParserState::AwaitingMarkToSet;
                        Ok(false)
                    }
                    '\'' => {
                        self.parser_state = ParserState::AwaitingMarkJumpLine;
                        Ok(false)
                    }
                    '`' => {
                        self.parser_state = ParserState::AwaitingMarkJumpExact;
                        Ok(false)
                    }
                    'y' => {
                        self.parser_state = ParserState::AwaitingYankTarget;
                        Ok(false)
                    }
                    'p' => {
                        // Paste after cursor
                        self.paste_after()?;
                        Ok(false)
                    }
                    _ => {
                        // Try to parse as regular command
                        Err(VimError::InvalidCommand(format!(
                            "Key '{}' not handled by state machine",
                            key
                        )))
                    }
                }
            }

            ParserState::AwaitingRegister => {
                // Second key after " - register name
                self.set_current_register(Register::Named(key));
                self.parser_state = ParserState::Normal;
                Ok(false)
            }

            ParserState::AwaitingMarkToSet => {
                // Second key after m - mark name
                self.set_mark(MarkId::Local(key))?;
                self.parser_state = ParserState::Normal;
                Ok(false)
            }

            ParserState::AwaitingMarkJumpLine => {
                // Second key after ' - jump to mark line
                let mark_id = MarkId::Local(key);
                self.jump_to_mark(&mark_id)?;
                self.parser_state = ParserState::Normal;
                Ok(false)
            }

            ParserState::AwaitingMarkJumpExact => {
                // Second key after ` - jump to mark exact position
                let mark_id = MarkId::Local(key);
                self.jump_to_mark(&mark_id)?;
                self.parser_state = ParserState::Normal;
                Ok(false)
            }

            ParserState::AwaitingYankTarget => {
                // Second key after y
                match key {
                    'y' => {
                        // yy - yank line
                        self.yank_line()?;
                        self.parser_state = ParserState::Normal;
                        Ok(false)
                    }
                    _ => {
                        // Cancel yank
                        self.parser_state = ParserState::Normal;
                        Err(VimError::InvalidCommand(format!(
                            "Invalid yank target: {}",
                            key
                        )))
                    }
                }
            }
        }
    }

    /// Update special marks automatically
    pub fn update_special_marks(&mut self, special: SpecialMark) {
        let buffer = self.current_buffer();
        let mark = Mark::new(self.active_buffer, buffer.cursor.line, buffer.cursor.col);
        self.marks.insert(MarkId::Special(special), mark);
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

    // ===== Mark Tests =====

    #[test]
    fn test_set_and_get_local_mark() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Set cursor at line 1, col 3
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 3);

        // Set mark 'a'
        let mark_id = MarkId::Local('a');
        state.set_mark(mark_id.clone()).unwrap();

        // Verify mark was set correctly
        let mark = state.get_mark(&mark_id).unwrap();
        assert_eq!(mark.buffer_id, BufferId(0));
        assert_eq!(mark.line, 1);
        assert_eq!(mark.col, 3);
    }

    #[test]
    fn test_set_and_get_global_mark() {
        let mut state = VimState::new_with_text("Test content");

        // Set cursor at line 0, col 5
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 5);

        // Set global mark 'A'
        let mark_id = MarkId::Global('A');
        state.set_mark(mark_id.clone()).unwrap();

        // Verify mark was set
        let mark = state.get_mark(&mark_id).unwrap();
        assert_eq!(mark.line, 0);
        assert_eq!(mark.col, 5);
    }

    #[test]
    fn test_jump_to_local_mark() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Set mark at line 1, col 3
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 3);
        state.set_mark(MarkId::Local('a')).unwrap();

        // Move cursor somewhere else
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(2, 0);

        // Jump to mark 'a'
        state.jump_to_mark(&MarkId::Local('a')).unwrap();

        // Verify cursor moved to mark position
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.col, 3);
    }

    #[test]
    fn test_jump_to_nonexistent_mark() {
        let mut state = VimState::new_with_text("Test");

        // Try to jump to mark that doesn't exist
        let result = state.jump_to_mark(&MarkId::Local('z'));

        // Should return error
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VimError::General("Mark not set: Local('z')".to_string())
        );
    }

    #[test]
    fn test_special_mark() {
        let mut state = VimState::new_with_text("Test content");

        // Set cursor position
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 5);

        // Update special mark (last edit)
        state.update_special_marks(SpecialMark::LastEdit);

        // Verify special mark was set
        let mark = state
            .get_mark(&MarkId::Special(SpecialMark::LastEdit))
            .unwrap();
        assert_eq!(mark.line, 0);
        assert_eq!(mark.col, 5);
    }

    #[test]
    fn test_overwrite_mark() {
        let mut state = VimState::new_with_text("Line 1\nLine 2");

        // Set mark at position 1
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 3);
        state.set_mark(MarkId::Local('a')).unwrap();

        // Move and set same mark at position 2
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 5);
        state.set_mark(MarkId::Local('a')).unwrap();

        // Verify mark now points to position 2
        let mark = state.get_mark(&MarkId::Local('a')).unwrap();
        assert_eq!(mark.line, 1);
        assert_eq!(mark.col, 5);
    }

    // ===== Jump List Tests =====

    #[test]
    fn test_record_jump() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Record current position
        state.record_jump();

        // With only one jump, we can't jump back or forward
        assert!(!state.can_jump_back());
        assert!(!state.can_jump_forward());

        // Record another position
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 0);
        state.record_jump();

        // Now we can jump back (to first position) but not forward
        assert!(state.can_jump_back());
        assert!(!state.can_jump_forward());
    }

    #[test]
    fn test_jump_back() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Record position 1
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 0);
        state.record_jump();

        // Move to position 2 and record
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 5);
        state.record_jump();

        // Move to position 3
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(2, 3);

        // Jump back to position 2
        state.jump_back().unwrap();
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.col, 5);

        // Jump back to position 1
        state.jump_back().unwrap();
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.col, 0);
    }

    #[test]
    fn test_jump_forward() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Record position 1
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 0);
        state.record_jump();

        // Record position 2
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 5);
        state.record_jump();

        // Jump back
        state.jump_back().unwrap();

        // Now jump forward
        state.jump_forward().unwrap();
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.col, 5);
    }

    #[test]
    fn test_jump_back_at_oldest() {
        let mut state = VimState::new_with_text("Test");

        // Try to jump back with no jumps recorded
        let result = state.jump_back();

        // Should return error
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VimError::General("Already at oldest jump".to_string())
        );
    }

    #[test]
    fn test_jump_forward_at_newest() {
        let mut state = VimState::new_with_text("Test");

        // Record a jump
        state.record_jump();

        // Try to jump forward (no jumps ahead)
        let result = state.jump_forward();

        // Should return error
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VimError::General("Already at newest jump".to_string())
        );
    }

    #[test]
    fn test_jump_list_navigation() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3\nLine 4");

        // Record multiple positions
        for line in 0..4 {
            let buffer = state.current_buffer_mut();
            buffer.cursor = CursorPos::new(line, 0);
            state.record_jump();
        }

        // Jump back 3 times
        state.jump_back().unwrap(); // To line 2
        state.jump_back().unwrap(); // To line 1
        state.jump_back().unwrap(); // To line 0

        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 0);

        // Jump forward 2 times
        state.jump_forward().unwrap(); // To line 1
        state.jump_forward().unwrap(); // To line 2

        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 2);
    }

    #[test]
    fn test_can_jump_back() {
        let mut state = VimState::new_with_text("Test");

        // No jumps recorded yet
        assert!(!state.can_jump_back());

        // Record first jump
        state.record_jump();

        // Still can't jump back (only one position, current = 0)
        assert!(!state.can_jump_back());

        // Record second jump
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 5);
        state.record_jump();

        // Now we can jump back (we're at index 1, can go to index 0)
        assert!(state.can_jump_back());
    }

    #[test]
    fn test_can_jump_forward() {
        let mut state = VimState::new_with_text("Test");

        // Record two jumps
        state.record_jump();
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 5);
        state.record_jump();

        // Can't jump forward yet
        assert!(!state.can_jump_forward());

        // Jump back once
        state.jump_back().unwrap();

        // Now we can jump forward
        assert!(state.can_jump_forward());
    }

    #[test]
    fn test_jump_records_position_before_jumping() {
        let mut state = VimState::new_with_text("Line 1\nLine 2");

        // Set mark at line 0
        state.set_mark(MarkId::Local('a')).unwrap();

        // Move to line 1
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 0);

        // Jump to mark (should record current position first)
        state.jump_to_mark(&MarkId::Local('a')).unwrap();

        // Now jump back - should return to line 1
        state.jump_back().unwrap();
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 1);
    }

    // Parser state machine tests
    #[test]
    fn test_handle_normal_key_register_selection() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Initially in Normal state
        assert_eq!(state.parser_state, ParserState::Normal);

        // Press " to start register selection
        let result = state.handle_normal_key('"');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingRegister);

        // Press 'a' to select register a
        let result = state.handle_normal_key('a');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);
        assert_eq!(state.selected_register, Some(Register::Named('a')));
    }

    #[test]
    fn test_handle_normal_key_set_mark() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Press 'm' to start mark setting
        let result = state.handle_normal_key('m');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingMarkToSet);

        // Press 'a' to set mark a
        let result = state.handle_normal_key('a');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);

        // Verify mark was set
        let mark_id = MarkId::Local('a');
        assert!(state.marks.contains_key(&mark_id));
    }

    #[test]
    fn test_handle_normal_key_jump_to_mark_line() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Set mark at line 2
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(2, 0);
        state.set_mark(MarkId::Local('a')).unwrap();

        // Move to line 0
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 0);

        // Press ' to start mark jump (line)
        let result = state.handle_normal_key('\'');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingMarkJumpLine);

        // Press 'a' to jump to mark a
        let result = state.handle_normal_key('a');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);

        // Verify cursor jumped to mark
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 2);
    }

    #[test]
    fn test_handle_normal_key_jump_to_mark_exact() {
        let mut state = VimState::new_with_text("Line 1\nLine 2\nLine 3");

        // Set mark at line 2, col 5
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(2, 5);
        state.set_mark(MarkId::Local('b')).unwrap();

        // Move to line 0
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(0, 0);

        // Press ` to start mark jump (exact)
        let result = state.handle_normal_key('`');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingMarkJumpExact);

        // Press 'b' to jump to mark b
        let result = state.handle_normal_key('b');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);

        // Verify cursor jumped to exact position
        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor.line, 2);
        assert_eq!(buffer.cursor.col, 5);
    }

    #[test]
    fn test_handle_normal_key_yank_line() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Press 'y' to start yank
        let result = state.handle_normal_key('y');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingYankTarget);

        // Press 'y' to yank line
        let result = state.handle_normal_key('y');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);

        // Verify line was yanked
        let register = state.get_current_register();
        let content = state.paste_from_register(&register);
        assert!(content.is_some());
        assert_eq!(content.unwrap().text, "Hello");
    }

    #[test]
    fn test_handle_normal_key_yank_invalid_target() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Press 'y' to start yank
        let result = state.handle_normal_key('y');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingYankTarget);

        // Press invalid target 'x'
        let result = state.handle_normal_key('x');
        assert!(result.is_err());
        assert_eq!(state.parser_state, ParserState::Normal);
    }

    #[test]
    fn test_handle_normal_key_paste() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Yank line first
        state.yank_line().unwrap();

        // Move to line 1
        let buffer = state.current_buffer_mut();
        buffer.cursor = CursorPos::new(1, 0);

        // Press 'p' to paste
        let result = state.handle_normal_key('p');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);

        // Verify paste occurred
        let buffer = state.current_buffer();
        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[2], "Hello");
    }

    #[test]
    fn test_handle_normal_key_register_and_yank_sequence() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Select register 'a'
        state.handle_normal_key('"').unwrap();
        state.handle_normal_key('a').unwrap();

        // Yank line to register 'a'
        state.handle_normal_key('y').unwrap();
        state.handle_normal_key('y').unwrap();

        // Verify yanked to named register 'a'
        let content = state.paste_from_register(&Register::Named('a'));
        assert!(content.is_some());
        assert_eq!(content.unwrap().text, "Hello");

        // Verify register selection was cleared
        assert_eq!(state.selected_register, None);
    }

    #[test]
    fn test_handle_normal_key_invalid_in_normal_state() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Try unhandled key
        let result = state.handle_normal_key('z');
        assert!(result.is_err());
        assert_eq!(state.parser_state, ParserState::Normal);
    }

    #[test]
    fn test_handle_normal_key_state_reset_after_mark_set() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Start mark setting
        state.handle_normal_key('m').unwrap();
        assert_eq!(state.parser_state, ParserState::AwaitingMarkToSet);

        // Complete mark setting
        state.handle_normal_key('x').unwrap();
        assert_eq!(state.parser_state, ParserState::Normal);

        // Verify next command starts in Normal state
        let result = state.handle_normal_key('m');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingMarkToSet);
    }

    #[test]
    fn test_handle_normal_key_state_reset_on_error() {
        let mut state = VimState::new_with_text("Hello\nWorld");

        // Start yank
        state.handle_normal_key('y').unwrap();
        assert_eq!(state.parser_state, ParserState::AwaitingYankTarget);

        // Try invalid yank target
        let result = state.handle_normal_key('z');
        assert!(result.is_err());

        // Verify state was reset
        assert_eq!(state.parser_state, ParserState::Normal);
    }

    // ========================================================================
    // RED TESTS - Coverage improvement (vim/state.rs 83.96% → target 95%+)
    // Priority: Lines 576-583 (multi-line yank), 886-892 (error Display)
    // ========================================================================

    #[test]
    fn test_yank_visual_character_multiline() {
        // Lines 576-583: Multi-line character yank
        let mut state = VimState::new_with_text("Line1\nLine2\nLine3");

        // Enter visual character mode
        state.mode = VimMode::Visual(VisualMode::Character);
        state.current_buffer_mut().cursor = CursorPos { line: 0, col: 2 }; // "ne1"
        state.current_buffer_mut().visual_anchor = Some(CursorPos { line: 0, col: 2 });

        // Move to line 2, col 2
        state.current_buffer_mut().cursor = CursorPos { line: 2, col: 2 };

        // Yank the selection
        state.yank_visual().unwrap();

        // Verify yanked text includes newlines
        let yanked = state.registers.get(&Register::Unnamed).unwrap();
        assert!(yanked.text.contains("ne1"));
        assert!(yanked.text.contains('\n'));
        assert!(yanked.text.contains("Line2"));
        assert!(yanked.text.contains("Lin"));
    }

    #[test]
    fn test_yank_visual_character_multiline_middle() {
        // Lines 585-586: Middle line in multi-line yank
        let mut state = VimState::new_with_text("AAAA\nBBBB\nCCCC\nDDDD");

        state.mode = VimMode::Visual(VisualMode::Character);
        state.current_buffer_mut().cursor = CursorPos { line: 1, col: 1 }; // Start at "BBB"
        state.current_buffer_mut().visual_anchor = Some(CursorPos { line: 1, col: 1 });

        // Extend to line 3
        state.current_buffer_mut().cursor = CursorPos { line: 3, col: 1 };

        state.yank_visual().unwrap();

        let yanked = state.registers.get(&Register::Unnamed).unwrap();
        // Should include BBB, full CCCC line, and DD
        assert!(yanked.text.contains("BBB"));
        assert!(yanked.text.contains("CCCC"));
        assert!(yanked.text.contains("DD"));
    }

    #[test]
    fn test_vim_error_display_invalid_command() {
        // Lines 886-892: VimError Display implementation
        let err = VimError::InvalidCommand("foo".to_string());
        assert_eq!(format!("{}", err), "E492: Not an editor command: foo");
    }

    #[test]
    fn test_vim_error_display_buffer_not_found() {
        // Lines 886-892: BufferNotFound variant
        let err = VimError::BufferNotFound(BufferId(42));
        assert_eq!(format!("{}", err), "E86: Buffer 42 does not exist");
    }

    #[test]
    fn test_vim_error_display_file_error() {
        // Lines 886-892: FileError variant
        let err = VimError::FileError("Cannot write file".to_string());
        assert_eq!(format!("{}", err), "E212: Cannot write file");
    }

    #[test]
    fn test_vim_error_display_invalid_mode() {
        // Lines 886-892: InvalidMode variant
        let err = VimError::InvalidMode(VimMode::Normal);
        assert_eq!(format!("{}", err), "E488: Trailing characters: NORMAL");
    }

    #[test]
    fn test_vim_error_display_general() {
        // Lines 886-892: General variant
        let err = VimError::General("Something went wrong".to_string());
        assert_eq!(format!("{}", err), "Something went wrong");
    }
}
