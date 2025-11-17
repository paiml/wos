//! WOS Microkernel
//!
//! Minimal trusted computing base providing:
//! - Process scheduling
//! - Memory management
//! - System call dispatch
//! - IPC primitives

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod memory;
pub mod scheduler;
pub mod shm;
pub mod signals;
pub mod state;
pub mod sync;
pub mod syscall;
pub mod trace;

pub use memory::{
    MemoryAccess, MemoryLayout, MemoryRegion, PagePermissions, PageTableEntry, PhysicalAddress,
    PhysicalPage, VirtualAddress, VirtualMemory, VirtualPage, PAGE_SIZE,
};
pub use scheduler::Scheduler;
pub use shm::{ProcessMapping, SharedMemoryId, SharedMemoryManager, SharedMemorySegment};
pub use signals::{Signal, SignalAction, SignalSet};
pub use state::{FileDescriptor, KernelState, Message, Process, ProcessId, ProcessState};
pub use sync::{
    Mutex, MutexId, MutexLockResult, MutexState, Semaphore, SemaphoreId, SemaphoreWaitResult,
    SyncManager,
};
pub use syscall::{dispatch_syscall, KernelError, SyscallOutput, SyscallResult, SystemCall};
pub use trace::{KernelHistory, SystemCallTrace};

/// Placeholder for kernel implementation
pub fn kernel_version() -> &'static str {
    "0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_version() {
        assert_eq!(kernel_version(), "0.1.0");
    }
}
