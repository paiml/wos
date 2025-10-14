//! WOS Shared Infrastructure
//!
//! Provides common types and utilities used across all WOS components:
//! - Virtual File System (VFS)
//! - Deterministic execution context
//! - Serialization helpers

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod context;
pub mod vfs;

pub use context::ExecutionContext;
pub use vfs::VirtualFileSystem;
