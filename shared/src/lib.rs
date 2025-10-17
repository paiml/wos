//! WOS Shared Infrastructure
//!
//! Provides common types and utilities used across all WOS components:
//! - Virtual File System (VFS)
//! - Deterministic execution context
//! - Command line parser
//! - Command pipeline parser
//! - Serialization helpers

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod context;
pub mod parser;
pub mod pipeline;
pub mod vfs;

pub use context::ExecutionContext;
pub use parser::parse_command;
pub use pipeline::{parse_pipeline, Command, Operator, Pipeline, PipelineStage, Redirection};
pub use vfs::VirtualFileSystem;
