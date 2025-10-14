//! System Call Dispatcher
//!
//! Pure functional system call interface with error handling.

use crate::state::{KernelState, ProcessId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// System call error types
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum KernelError {
    /// Process not found
    #[error("Process not found: {0}")]
    ProcessNotFound(ProcessId),

    /// Invalid process state
    #[error("Invalid process state for operation")]
    InvalidProcessState,

    /// Invalid system call parameters
    #[error("Invalid system call parameters: {0}")]
    InvalidParameters(String),

    /// Resource exhausted (e.g., PID space)
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Permission denied
    #[error("Permission denied")]
    PermissionDenied,

    /// Not implemented yet
    #[error("System call not implemented")]
    NotImplemented,
}

/// System call result type
pub type SyscallResult<T> = Result<T, KernelError>;

/// System call variants
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemCall {
    /// Get current process ID
    GetPid,

    /// Fork current process (create child)
    Fork,

    /// Exit current process with code
    Exit(i32),

    /// Wait for child process
    WaitPid(ProcessId),

    /// Sleep for microseconds
    Sleep(u64),

    /// Open file
    Open {
        /// Path to file
        path: String,
        /// Flags (read, write, create, etc.)
        flags: u32,
    },

    /// Close file descriptor
    Close {
        /// File descriptor to close
        fd: u32,
    },

    /// Read from file descriptor
    Read {
        /// File descriptor
        fd: u32,
        /// Number of bytes to read
        count: usize,
    },

    /// Write to file descriptor
    Write {
        /// File descriptor
        fd: u32,
        /// Data to write
        data: Vec<u8>,
    },

    /// Allocate memory (mmap)
    Mmap {
        /// Size in bytes
        size: usize,
    },

    /// Free memory (munmap)
    Munmap {
        /// Address to free
        addr: u64,
        /// Size in bytes
        size: usize,
    },
}

/// System call output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyscallOutput {
    /// Process ID
    Pid(ProcessId),

    /// Success with no data
    Success,

    /// Integer value
    Value(i32),

    /// Byte data
    Data(Vec<u8>),

    /// Address (for mmap)
    Address(u64),
}

/// Dispatch a system call
///
/// Pure functional dispatcher: takes kernel state and syscall, returns new state and output.
/// Never panics - all errors are returned as Results.
pub fn dispatch_syscall(
    state: KernelState,
    syscall: SystemCall,
    calling_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Verify calling process exists
    if !state.processes.contains_key(&calling_pid) {
        return Err(KernelError::ProcessNotFound(calling_pid));
    }

    match syscall {
        SystemCall::GetPid => {
            // Return calling process's PID
            Ok((state, SyscallOutput::Pid(calling_pid)))
        }

        SystemCall::Fork => {
            // Fork: create child process
            let mut new_state = state;

            // Allocate new PID for child
            let child_pid = new_state.allocate_pid();

            // Get parent process
            let parent = new_state
                .get_process(calling_pid)
                .ok_or(KernelError::ProcessNotFound(calling_pid))?
                .clone();

            // Create child process (copy of parent)
            let mut child = parent.clone();
            child.pid = child_pid;
            child.parent_pid = Some(calling_pid);

            // Add child to process table
            new_state.add_process(child);

            // Return child PID to parent
            Ok((new_state, SyscallOutput::Pid(child_pid)))
        }

        SystemCall::Exit(code) => {
            // Exit: terminate calling process
            let mut new_state = state;

            // Update process state to Terminated
            if let Some(process) = new_state.get_process_mut(calling_pid) {
                process.state = crate::state::ProcessState::Terminated(code);
            } else {
                return Err(KernelError::ProcessNotFound(calling_pid));
            }

            Ok((new_state, SyscallOutput::Success))
        }

        SystemCall::WaitPid(wait_pid) => {
            // WaitPid: wait for child process to terminate
            let state_ref = &state;

            // Verify calling process exists
            if !state_ref.processes.contains_key(&calling_pid) {
                return Err(KernelError::ProcessNotFound(calling_pid));
            }

            // Verify target process exists
            let target = state_ref
                .get_process(wait_pid)
                .ok_or(KernelError::ProcessNotFound(wait_pid))?;

            // Verify target is a child of caller
            if target.parent_pid != Some(calling_pid) {
                return Err(KernelError::PermissionDenied);
            }

            // Check if child has terminated
            match target.state {
                crate::state::ProcessState::Terminated(exit_code) => {
                    // Child has exited, return exit code
                    Ok((state, SyscallOutput::Value(exit_code)))
                }
                _ => {
                    // Child still running - in real OS, would block
                    // For now, return error to indicate blocking needed
                    Err(KernelError::InvalidProcessState)
                }
            }
        }

        SystemCall::Sleep(_duration) => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::Open { .. } => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::Close { .. } => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::Read { .. } => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::Write { .. } => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::Mmap { size } => {
            // Allocate memory for process
            let mut new_state = state;

            // Get mutable access to process
            if let Some(process) = new_state.get_process_mut(calling_pid) {
                // Allocate memory
                if let Some(addr) = process.memory.mmap(size) {
                    Ok((new_state, SyscallOutput::Address(addr)))
                } else {
                    Err(KernelError::ResourceExhausted("Out of memory".to_string()))
                }
            } else {
                Err(KernelError::ProcessNotFound(calling_pid))
            }
        }

        SystemCall::Munmap { addr, size } => {
            // Free memory for process
            let mut new_state = state;

            // Get mutable access to process
            if let Some(process) = new_state.get_process_mut(calling_pid) {
                // Free memory
                if process.memory.munmap(addr, size) {
                    Ok((new_state, SyscallOutput::Success))
                } else {
                    Err(KernelError::InvalidParameters(
                        "Invalid munmap range".to_string(),
                    ))
                }
            } else {
                Err(KernelError::ProcessNotFound(calling_pid))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Process;

    #[test]
    fn test_syscall_dispatch_routing() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Test GetPid
        let result = dispatch_syscall(state.clone(), SystemCall::GetPid, pid);
        assert!(result.is_ok());
        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Pid(pid));
        assert_eq!(new_state, state);
    }

    #[test]
    fn test_syscall_error_handling() {
        let state = KernelState::new();
        let invalid_pid = 999;

        // Calling with non-existent PID should fail
        let result = dispatch_syscall(state, SystemCall::GetPid, invalid_pid);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            KernelError::ProcessNotFound(invalid_pid)
        );
    }

    #[test]
    fn test_syscall_not_implemented() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Test unimplemented syscalls (Sleep, Open, Close, Read, Write)
        let syscalls = vec![
            SystemCall::Sleep(1000),
            SystemCall::Open {
                path: "/test".to_string(),
                flags: 0,
            },
            SystemCall::Close { fd: 0 },
            SystemCall::Read { fd: 0, count: 100 },
            SystemCall::Write {
                fd: 0,
                data: vec![1, 2, 3],
            },
        ];

        for syscall in syscalls {
            let result = dispatch_syscall(state.clone(), syscall, pid);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), KernelError::NotImplemented);
        }
    }

    #[test]
    fn test_sys_getpid() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let result = dispatch_syscall(state.clone(), SystemCall::GetPid, pid);
        assert!(result.is_ok());
        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Pid(pid));
        assert_eq!(new_state, state); // State unchanged
    }

    #[test]
    fn test_sys_fork_creates_child() {
        let mut state = KernelState::new();
        let parent_pid = state.allocate_pid();
        let parent = Process::new(parent_pid, None);
        state.add_process(parent);

        let result = dispatch_syscall(state.clone(), SystemCall::Fork, parent_pid);
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();

        // Should return child PID
        let child_pid = match output {
            SyscallOutput::Pid(pid) => pid,
            _ => panic!("Expected Pid output"),
        };

        // Child should exist in new state
        let child = new_state.get_process(child_pid).expect("Child not found");
        assert_eq!(child.pid, child_pid);
        assert_eq!(child.parent_pid, Some(parent_pid));

        // Parent should still exist
        assert!(new_state.get_process(parent_pid).is_some());

        // Process count should increase by 1
        assert_eq!(new_state.process_count(), state.process_count() + 1);
    }

    #[test]
    fn test_sys_exit_terminates_process() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let exit_code = 42;
        let result = dispatch_syscall(state, SystemCall::Exit(exit_code), pid);
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Success);

        // Process should be terminated
        let proc = new_state.get_process(pid).expect("Process not found");
        assert!(proc.is_terminated());
        assert_eq!(
            proc.state,
            crate::state::ProcessState::Terminated(exit_code)
        );
    }

    #[test]
    fn test_sys_waitpid_blocks_until_exit() {
        let mut state = KernelState::new();

        // Create parent
        let parent_pid = state.allocate_pid();
        let parent = Process::new(parent_pid, None);
        state.add_process(parent);

        // Create child (fork)
        let result = dispatch_syscall(state, SystemCall::Fork, parent_pid);
        assert!(result.is_ok());
        let (state, output) = result.unwrap();

        let child_pid = match output {
            SyscallOutput::Pid(pid) => pid,
            _ => panic!("Expected Pid output"),
        };

        // Waitpid on running child should fail (would block)
        let result = dispatch_syscall(state.clone(), SystemCall::WaitPid(child_pid), parent_pid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::InvalidProcessState);

        // Exit child
        let result = dispatch_syscall(state, SystemCall::Exit(123), child_pid);
        assert!(result.is_ok());
        let (state, _) = result.unwrap();

        // Now waitpid should succeed
        let result = dispatch_syscall(state, SystemCall::WaitPid(child_pid), parent_pid);
        assert!(result.is_ok());
        let (_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Value(123));
    }

    #[test]
    fn test_fork_wait_pipeline() {
        let mut state = KernelState::new();

        // Create init process
        let init_pid = state.allocate_pid();
        let init = Process::new(init_pid, None);
        state.add_process(init);

        // Fork
        let (state, output) = dispatch_syscall(state, SystemCall::Fork, init_pid).unwrap();
        let child_pid = match output {
            SyscallOutput::Pid(pid) => pid,
            _ => panic!("Expected Pid"),
        };

        // Child exits
        let (state, _) = dispatch_syscall(state, SystemCall::Exit(0), child_pid).unwrap();

        // Parent waits
        let (state, output) =
            dispatch_syscall(state, SystemCall::WaitPid(child_pid), init_pid).unwrap();
        assert_eq!(output, SyscallOutput::Value(0));

        // Verify state is consistent
        assert_eq!(state.process_count(), 2); // init + child (still in table)
    }

    #[test]
    fn test_waitpid_permission_denied() {
        let mut state = KernelState::new();

        // Create two unrelated processes
        let pid1 = state.allocate_pid();
        let proc1 = Process::new(pid1, None);
        state.add_process(proc1);

        let pid2 = state.allocate_pid();
        let proc2 = Process::new(pid2, None);
        state.add_process(proc2);

        // pid1 tries to wait on pid2 (not its child)
        let result = dispatch_syscall(state, SystemCall::WaitPid(pid2), pid1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::PermissionDenied);
    }

    #[test]
    fn test_sys_mmap_basic() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Allocate 4096 bytes (1 page)
        let result = dispatch_syscall(state.clone(), SystemCall::Mmap { size: 4096 }, pid);
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();
        let addr = match output {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        // Should return heap start address
        let proc = new_state.get_process(pid).unwrap();
        assert_eq!(addr, proc.memory.layout().heap_start);
        assert_eq!(proc.memory.mapped_page_count(), 1);
    }

    #[test]
    fn test_sys_mmap_multiple_allocations() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // First allocation
        let result1 = dispatch_syscall(state.clone(), SystemCall::Mmap { size: 4096 }, pid);
        assert!(result1.is_ok());
        let (state, output1) = result1.unwrap();
        let addr1 = match output1 {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        // Second allocation
        let result2 = dispatch_syscall(state.clone(), SystemCall::Mmap { size: 8192 }, pid);
        assert!(result2.is_ok());
        let (state, output2) = result2.unwrap();
        let addr2 = match output2 {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        // Second address should be after first
        assert!(addr2 > addr1);

        // Should have 3 pages total (1 + 2)
        let proc = state.get_process(pid).unwrap();
        assert_eq!(proc.memory.mapped_page_count(), 3);
    }

    #[test]
    fn test_sys_mmap_zero_size() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Zero-size allocation should fail
        let result = dispatch_syscall(state, SystemCall::Mmap { size: 0 }, pid);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KernelError::ResourceExhausted(_)
        ));
    }

    #[test]
    fn test_sys_mmap_out_of_memory() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        let heap_size = proc.memory.layout().heap_size;
        state.add_process(proc);

        // Try to allocate more than heap size
        let result = dispatch_syscall(
            state,
            SystemCall::Mmap {
                size: heap_size + 4096,
            },
            pid,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KernelError::ResourceExhausted(_)
        ));
    }

    #[test]
    fn test_sys_mmap_invalid_process() {
        let state = KernelState::new();
        let invalid_pid = 999;

        let result = dispatch_syscall(state, SystemCall::Mmap { size: 4096 }, invalid_pid);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            KernelError::ProcessNotFound(invalid_pid)
        );
    }

    #[test]
    fn test_sys_munmap_basic() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Allocate memory
        let result = dispatch_syscall(state.clone(), SystemCall::Mmap { size: 4096 }, pid);
        assert!(result.is_ok());
        let (state, output) = result.unwrap();
        let addr = match output {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        // Verify allocation
        let proc = state.get_process(pid).unwrap();
        assert_eq!(proc.memory.mapped_page_count(), 1);

        // Free memory
        let result = dispatch_syscall(state.clone(), SystemCall::Munmap { addr, size: 4096 }, pid);
        assert!(result.is_ok());
        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Success);

        // Verify freed
        let proc = new_state.get_process(pid).unwrap();
        assert_eq!(proc.memory.mapped_page_count(), 0);
    }

    #[test]
    fn test_sys_munmap_partial_range() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Allocate 3 pages
        let result = dispatch_syscall(state.clone(), SystemCall::Mmap { size: 12288 }, pid);
        assert!(result.is_ok());
        let (state, output) = result.unwrap();
        let addr = match output {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        // Free middle page
        let middle_addr = addr + 4096;
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Munmap {
                addr: middle_addr,
                size: 4096,
            },
            pid,
        );
        assert!(result.is_ok());
        let (new_state, _) = result.unwrap();

        // Should have 2 pages left
        let proc = new_state.get_process(pid).unwrap();
        assert_eq!(proc.memory.mapped_page_count(), 2);
    }

    #[test]
    fn test_sys_munmap_unmapped_fails() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        let heap_start = proc.memory.layout().heap_start;
        state.add_process(proc);

        // Try to free unmapped memory
        let result = dispatch_syscall(
            state,
            SystemCall::Munmap {
                addr: heap_start,
                size: 4096,
            },
            pid,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KernelError::InvalidParameters(_)
        ));
    }

    #[test]
    fn test_sys_munmap_invalid_process() {
        let state = KernelState::new();
        let invalid_pid = 999;

        let result = dispatch_syscall(
            state,
            SystemCall::Munmap {
                addr: 0x3000_0000,
                size: 4096,
            },
            invalid_pid,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            KernelError::ProcessNotFound(invalid_pid)
        );
    }

    #[test]
    fn test_sys_mmap_munmap_integration() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Allocate, free, allocate cycle
        let result1 = dispatch_syscall(state.clone(), SystemCall::Mmap { size: 4096 }, pid);
        assert!(result1.is_ok());
        let (state, output1) = result1.unwrap();
        let addr1 = match output1 {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        let result2 = dispatch_syscall(
            state.clone(),
            SystemCall::Munmap {
                addr: addr1,
                size: 4096,
            },
            pid,
        );
        assert!(result2.is_ok());
        let (state, _) = result2.unwrap();

        let result3 = dispatch_syscall(state, SystemCall::Mmap { size: 4096 }, pid);
        assert!(result3.is_ok());
        let (new_state, output3) = result3.unwrap();
        let addr3 = match output3 {
            SyscallOutput::Address(a) => a,
            _ => panic!("Expected Address output"),
        };

        // Note: Sequential allocator doesn't reuse freed pages
        assert!(addr3 > addr1);

        let proc = new_state.get_process(pid).unwrap();
        assert_eq!(proc.memory.mapped_page_count(), 1);
    }

    #[test]
    fn test_syscall_serialization() {
        // Test SystemCall serialization
        let syscall = SystemCall::GetPid;
        let json = serde_json::to_string(&syscall).unwrap();
        let syscall2: SystemCall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, syscall2);

        // Test with complex syscall
        let syscall = SystemCall::Write {
            fd: 1,
            data: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&syscall).unwrap();
        let syscall2: SystemCall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, syscall2);

        // Test mmap
        let syscall = SystemCall::Mmap { size: 4096 };
        let json = serde_json::to_string(&syscall).unwrap();
        let syscall2: SystemCall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, syscall2);

        // Test munmap
        let syscall = SystemCall::Munmap {
            addr: 0x3000_0000,
            size: 4096,
        };
        let json = serde_json::to_string(&syscall).unwrap();
        let syscall2: SystemCall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, syscall2);
    }

    #[test]
    fn test_syscall_output_serialization() {
        let output = SyscallOutput::Pid(42);
        let json = serde_json::to_string(&output).unwrap();
        let output2: SyscallOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(output, output2);
    }

    #[test]
    fn test_kernel_error_serialization() {
        let error = KernelError::ProcessNotFound(42);
        let json = serde_json::to_string(&error).unwrap();
        let error2: KernelError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, error2);
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: Dispatcher never panics on any input
            #[test]
            fn proptest_syscall_never_panics(
                pid in 0..10000u32,
                calling_pid in 0..10000u32,
                exit_code in -128..128i32,
                fd in 0..1000u32,
                count in 0..10000usize,
            ) {
                let mut state = KernelState::new();

                // Create a process if calling_pid is 1
                if calling_pid == 1 {
                    let proc = Process::new(calling_pid, None);
                    state.add_process(proc);
                }

                let syscalls = vec![
                    SystemCall::GetPid,
                    SystemCall::Fork,
                    SystemCall::Exit(exit_code),
                    SystemCall::WaitPid(pid),
                    SystemCall::Sleep(1000),
                    SystemCall::Open {
                        path: "/test".to_string(),
                        flags: 0,
                    },
                    SystemCall::Close { fd },
                    SystemCall::Read { fd, count },
                    SystemCall::Write {
                        fd,
                        data: vec![1, 2, 3],
                    },
                ];

                for syscall in syscalls {
                    // Should never panic, even with invalid inputs
                    let result = dispatch_syscall(state.clone(), syscall, calling_pid);

                    // Result is either Ok or Err, never panic
                    prop_assert!(result.is_ok() || result.is_err());
                }
            }

            /// Property: Valid GetPid always succeeds and returns calling PID
            #[test]
            fn proptest_getpid_correctness(pid in 1..10000u32) {
                let mut state = KernelState::new();
                let proc = Process::new(pid, None);
                state.add_process(proc);

                let result = dispatch_syscall(state.clone(), SystemCall::GetPid, pid);

                prop_assert!(result.is_ok());
                let (new_state, output) = result.unwrap();
                prop_assert_eq!(output, SyscallOutput::Pid(pid));
                prop_assert_eq!(new_state, state);
            }

            /// Property: Syscall serialization roundtrip
            #[test]
            fn proptest_syscall_serialization(
                exit_code in -128..128i32,
                pid in 1..10000u32,
            ) {
                let syscalls = vec![
                    SystemCall::GetPid,
                    SystemCall::Fork,
                    SystemCall::Exit(exit_code),
                    SystemCall::WaitPid(pid),
                ];

                for syscall in syscalls {
                    let json = serde_json::to_string(&syscall).unwrap();
                    let syscall2: SystemCall = serde_json::from_str(&json).unwrap();
                    prop_assert_eq!(syscall, syscall2);
                }
            }

            /// Property: Invalid PID always returns ProcessNotFound
            #[test]
            fn proptest_invalid_pid_error(
                invalid_pid in 10000..100000u32,
            ) {
                let state = KernelState::new();

                let result = dispatch_syscall(state, SystemCall::GetPid, invalid_pid);

                prop_assert!(result.is_err());
                prop_assert_eq!(result.unwrap_err(), KernelError::ProcessNotFound(invalid_pid));
            }

            /// Property: State is preserved on GetPid
            #[test]
            fn proptest_getpid_preserves_state(
                num_processes in 1..100usize,
            ) {
                let mut state = KernelState::new();

                // Create multiple processes
                let mut pids = Vec::new();
                for _ in 0..num_processes {
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);
                    pids.push(pid);
                }

                let original_state = state.clone();

                // Call GetPid for each process
                for pid in pids {
                    let result = dispatch_syscall(state.clone(), SystemCall::GetPid, pid);
                    prop_assert!(result.is_ok());
                    let (new_state, _) = result.unwrap();
                    prop_assert_eq!(new_state, original_state.clone());
                }
            }

            /// Property: Fork creates unique PIDs
            #[test]
            fn proptest_fork_pid_uniqueness(
                num_forks in 1..100usize,
            ) {
                let mut state = KernelState::new();

                // Create parent
                let parent_pid = state.allocate_pid();
                let parent = Process::new(parent_pid, None);
                state.add_process(parent);

                let mut child_pids = std::collections::HashSet::new();

                // Fork multiple times
                for _ in 0..num_forks {
                    let result = dispatch_syscall(state, SystemCall::Fork, parent_pid);
                    prop_assert!(result.is_ok());

                    let (new_state, output) = result.unwrap();
                    state = new_state;

                    let child_pid = match output {
                        SyscallOutput::Pid(pid) => pid,
                        _ => return Err(proptest::test_runner::TestCaseError::fail("Expected Pid")),
                    };

                    // All child PIDs must be unique
                    prop_assert!(child_pids.insert(child_pid), "Duplicate child PID");
                }

                // Should have created num_forks unique children
                prop_assert_eq!(child_pids.len(), num_forks);
            }

            /// Property: Parent-child relationships are always valid
            #[test]
            fn proptest_parent_child_relationship(
                num_children in 1..50usize,
            ) {
                let mut state = KernelState::new();

                // Create parent
                let parent_pid = state.allocate_pid();
                let parent = Process::new(parent_pid, None);
                state.add_process(parent);

                // Fork multiple children
                for _ in 0..num_children {
                    let result = dispatch_syscall(state, SystemCall::Fork, parent_pid);
                    prop_assert!(result.is_ok());

                    let (new_state, output) = result.unwrap();
                    state = new_state;

                    let child_pid = match output {
                        SyscallOutput::Pid(pid) => pid,
                        _ => return Err(proptest::test_runner::TestCaseError::fail("Expected Pid")),
                    };

                    // Verify parent-child relationship
                    let child = state.get_process(child_pid).unwrap();
                    prop_assert_eq!(child.parent_pid, Some(parent_pid));

                    // Parent should still exist
                    prop_assert!(state.get_process(parent_pid).is_some());
                }
            }

            /// Property: Exit always terminates process
            #[test]
            fn proptest_exit_terminates(
                exit_code in -128..128i32,
            ) {
                let mut state = KernelState::new();
                let pid = state.allocate_pid();
                let proc = Process::new(pid, None);
                state.add_process(proc);

                let result = dispatch_syscall(state, SystemCall::Exit(exit_code), pid);
                prop_assert!(result.is_ok());

                let (new_state, _) = result.unwrap();
                let proc = new_state.get_process(pid).unwrap();

                prop_assert!(proc.is_terminated());
                prop_assert_eq!(&proc.state, &crate::state::ProcessState::Terminated(exit_code));
            }

            /// Property: WaitPid only succeeds for parent-child relationships
            #[test]
            fn proptest_waitpid_parent_child_only(
                _seed in 0..100u64,
            ) {
                let mut state = KernelState::new();

                // Create parent
                let parent_pid = state.allocate_pid();
                let parent = Process::new(parent_pid, None);
                state.add_process(parent);

                // Create unrelated process
                let unrelated_pid = state.allocate_pid();
                let unrelated = Process::new(unrelated_pid, None);
                state.add_process(unrelated);

                // Parent cannot wait on unrelated process
                let result = dispatch_syscall(state.clone(), SystemCall::WaitPid(unrelated_pid), parent_pid);
                prop_assert!(result.is_err());
                prop_assert_eq!(result.unwrap_err(), KernelError::PermissionDenied);

                // Fork a child
                let (state, output) = dispatch_syscall(state, SystemCall::Fork, parent_pid).unwrap();
                let child_pid = match output {
                    SyscallOutput::Pid(pid) => pid,
                    _ => return Err(proptest::test_runner::TestCaseError::fail("Expected Pid")),
                };

                // Exit child
                let (state, _) = dispatch_syscall(state, SystemCall::Exit(0), child_pid).unwrap();

                // Parent can wait on its own child
                let result = dispatch_syscall(state, SystemCall::WaitPid(child_pid), parent_pid);
                prop_assert!(result.is_ok());
            }

            /// Property: Fork-Exit-Wait pipeline always works
            #[test]
            fn proptest_fork_exit_wait_pipeline(
                exit_code in -128..128i32,
            ) {
                let mut state = KernelState::new();

                // Create parent
                let parent_pid = state.allocate_pid();
                let parent = Process::new(parent_pid, None);
                state.add_process(parent);

                // Fork
                let (state, output) = dispatch_syscall(state, SystemCall::Fork, parent_pid).unwrap();
                let child_pid = match output {
                    SyscallOutput::Pid(pid) => pid,
                    _ => return Err(proptest::test_runner::TestCaseError::fail("Expected Pid")),
                };

                // Child exits
                let (state, _) = dispatch_syscall(state, SystemCall::Exit(exit_code), child_pid).unwrap();

                // Parent waits
                let result = dispatch_syscall(state, SystemCall::WaitPid(child_pid), parent_pid);
                prop_assert!(result.is_ok());

                let (_state, output) = result.unwrap();
                prop_assert_eq!(output, SyscallOutput::Value(exit_code));
            }
        }
    }
}
