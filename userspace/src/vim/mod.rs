// Vim editor module for WOS
// Implements a modal text editor with NORMAL, INSERT, and COMMAND modes
// Following pure functional design with Command Pattern for undo/redo

/// Buffer management and undo/redo state
pub mod buffer;
/// Command Pattern implementation for vim operations
pub mod command;
/// High-level command execution
pub mod commands;
/// Ex command parsing and execution (:w, :q, etc.)
pub mod ex_commands;
/// Key press parsing to VimCommand
pub mod parser;
/// Vim state machine and mode management
pub mod state;

pub use buffer::{BufferMemento, CursorPos, VimBuffer};
pub use command::VimCommand;
pub use state::{
    JumpEntry, JumpList, Mark, MarkId, ParserState, Register, RegisterContent, RegisterType,
    SpecialMark, VimError, VimMode, VimState,
};
