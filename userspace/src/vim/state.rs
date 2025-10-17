// Vim editor state machine
// Implements the three vim modes: NORMAL, INSERT, COMMAND

use serde::{Deserialize, Serialize};
use std::fmt;

use super::buffer::{BufferId, VimBuffer};

/// Vim editor mode
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VimMode {
    /// Normal mode - for navigation and commands
    #[default]
    Normal,

    /// Insert mode - for text editing
    Insert,

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
}
