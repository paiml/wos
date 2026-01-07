//! WOS Microkernel
//!
//! Minimal trusted computing base providing:
//! - Process scheduling
//! - Memory management
//! - System call dispatch
//! - IPC primitives
//! - Jidoka invariant guards
//! - MicroVM virtualization

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod apr_runtime;
pub mod jidoka;
pub mod memory;
pub mod scheduler;
pub mod shm;
pub mod signals;
pub mod state;
pub mod sync;
pub mod syscall;
pub mod trace;
pub mod vmm;

pub use apr_runtime::{AprExecutionResult, KernelAprRuntime};
pub use jidoka::{
    InvariantCheck, JidokaGuard, JidokaStatus, KernelStateView, MockKernelState, Severity,
    Violation, ViolationCategory, MAX_FDS_PER_PROCESS, MAX_MEMORY_BYTES, MAX_PROCESSES,
    MAX_TOTAL_FDS, MAX_VMS, MAX_ZOMBIES,
};
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
pub use vmm::{
    ExceptionInfo, GuestMemory, GuestPage, IoRequest, MicroVm, VcpuState, VirtioConsole,
    VirtioDeviceConfig, VmConfig, VmError, VmExitReason, VmManager, VmState, VmStatus,
    DEFAULT_GUEST_MEMORY, GUEST_PAGE_SIZE, MAX_GUEST_MEMORY,
};

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
