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
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::Exit(_code) => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
        }

        SystemCall::WaitPid(_pid) => {
            // Not implemented yet - placeholder
            Err(KernelError::NotImplemented)
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

        // Test unimplemented syscalls
        let syscalls = vec![
            SystemCall::Fork,
            SystemCall::Exit(0),
            SystemCall::WaitPid(1),
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
        }
    }
}
