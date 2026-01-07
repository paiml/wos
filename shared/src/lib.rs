//! WOS Shared Infrastructure
//!
//! Provides common types and utilities used across all WOS components:
//! - Virtual File System (VFS)
//! - Deterministic execution context
//! - Command line parser
//! - Command pipeline parser
//! - Type-safe kernel primitives
//! - APR (Aprender Portable Runtime) model types
//! - Serialization helpers

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod apr;
pub mod context;
pub mod control_flow;
pub mod parser;
pub mod pipeline;
pub mod primitives;
pub mod script;
pub mod script_loader;
pub mod vfs;

pub use apr::{
    AprError, AprFormat, AprInput, AprMetadata, AprModel, AprOutput, AprRuntime, StateCheckpoint,
    TimestampedInput, APR_VERSION,
};
pub use context::{ExecutionContext, SimulatedClock};
pub use control_flow::{
    parse_for_statement, parse_if_statement, parse_until_statement, parse_while_statement,
    ControlFlow,
};
pub use parser::parse_command;
pub use pipeline::{parse_pipeline, Command, Operator, Pipeline, PipelineStage, Redirection};
pub use primitives::{
    FileDescriptor, MemoryProtection, Pfn, PhysAddr, ProcessId, Signal, VirtAddr, VmId, MAX_FD,
    MAX_PID, MAX_VM_ID, PAGE_SIZE,
};
pub use script::{Script, ScriptError};
pub use script_loader::ScriptLoader;
pub use vfs::VirtualFileSystem;
