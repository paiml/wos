//! System Call Dispatcher
//!
//! Pure functional system call interface with error handling.

use crate::state::{KernelState, ProcessId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// File already exists
    #[error("File already exists: {0}")]
    FileAlreadyExists(String),

    /// Invalid file descriptor
    #[error("Invalid file descriptor: {0}")]
    InvalidFileDescriptor(u32),

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

    /// File descriptor
    FileDescriptor(u32),
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

        SystemCall::Open { path, flags: _ } => {
            // Open file: create file descriptor
            let mut new_state = state;
            let path_buf = PathBuf::from(&path);

            // Check if file exists in VFS
            if !new_state.vfs.exists(&path_buf) {
                // File doesn't exist - for now, return error
                // TODO: Handle O_CREAT flag to create file if it doesn't exist
                return Err(KernelError::FileNotFound(path));
            }

            // Get mutable access to process
            if let Some(process) = new_state.get_process_mut(calling_pid) {
                // Open file (allocate file descriptor)
                let fd = process.open_file(path_buf);
                Ok((new_state, SyscallOutput::FileDescriptor(fd)))
            } else {
                Err(KernelError::ProcessNotFound(calling_pid))
            }
        }

        SystemCall::Close { fd } => {
            // Close file: remove file descriptor
            let mut new_state = state;

            // Get mutable access to process
            if let Some(process) = new_state.get_process_mut(calling_pid) {
                // Close file descriptor
                if process.close_file(fd).is_some() {
                    Ok((new_state, SyscallOutput::Success))
                } else {
                    // FD not found or is a standard stream
                    Err(KernelError::InvalidFileDescriptor(fd))
                }
            } else {
                Err(KernelError::ProcessNotFound(calling_pid))
            }
        }

        SystemCall::Read { fd, count } => {
            // Read from file descriptor
            let new_state = state;

            // Get the process
            let process = new_state
                .get_process(calling_pid)
                .ok_or(KernelError::ProcessNotFound(calling_pid))?;

            // Get file path from FD table
            let path = process
                .get_file_path(fd)
                .ok_or(KernelError::InvalidFileDescriptor(fd))?;

            // Read from VFS
            match new_state.vfs.read_file(path) {
                Ok(content) => {
                    // Return up to 'count' bytes
                    let bytes_to_return = content.len().min(count);
                    let data = content[..bytes_to_return].to_vec();
                    Ok((new_state, SyscallOutput::Data(data)))
                }
                Err(wos_shared::vfs::VfsError::NotFound) => {
                    Err(KernelError::FileNotFound(path.display().to_string()))
                }
                Err(wos_shared::vfs::VfsError::PermissionDenied) => {
                    Err(KernelError::PermissionDenied)
                }
                Err(_) => Err(KernelError::InvalidParameters(
                    "VFS error during read".to_string(),
                )),
            }
        }

        SystemCall::Write { fd, data } => {
            // Write to file descriptor
            let mut new_state = state;

            // Get file path from FD table
            let path = {
                let process = new_state
                    .get_process(calling_pid)
                    .ok_or(KernelError::ProcessNotFound(calling_pid))?;

                process
                    .get_file_path(fd)
                    .ok_or(KernelError::InvalidFileDescriptor(fd))?
                    .clone()
            };

            // Write to VFS
            match new_state.vfs.write_file(&path, data.clone()) {
                Ok(_) => {
                    // Return number of bytes written
                    Ok((new_state, SyscallOutput::Value(data.len() as i32)))
                }
                Err(wos_shared::vfs::VfsError::NotFound) => {
                    Err(KernelError::FileNotFound(path.display().to_string()))
                }
                Err(wos_shared::vfs::VfsError::PermissionDenied) => {
                    Err(KernelError::PermissionDenied)
                }
                Err(_) => Err(KernelError::InvalidParameters(
                    "VFS error during write".to_string(),
                )),
            }
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

        // Test unimplemented syscalls (Sleep)
        // Note: Open, Close, Read, Write are now implemented
        let syscalls = vec![SystemCall::Sleep(1000)];

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
    fn test_sys_open_creates_fd() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create a file in VFS
        let path = PathBuf::from("/test.txt");
        state
            .vfs
            .create_file(path.clone(), b"Hello".to_vec())
            .unwrap();

        // Open the file
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor output"),
        };

        // FD should be 3 (after stdin=0, stdout=1, stderr=2)
        assert_eq!(fd, 3);

        // Verify FD is open in process
        let proc = new_state.get_process(pid).unwrap();
        assert!(proc.is_fd_open(fd));
        assert_eq!(proc.get_file_path(fd), Some(&path));
    }

    #[test]
    fn test_sys_open_nonexistent_file() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to open non-existent file
        let result = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/nonexistent.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KernelError::FileNotFound(_)));
    }

    #[test]
    fn test_sys_open_multiple_files() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create files in VFS
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), vec![])
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), vec![])
            .unwrap();

        // Open first file
        let result1 = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/file1.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output1) = result1.unwrap();
        let fd1 = match output1 {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Open second file
        let result2 = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/file2.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (new_state, output2) = result2.unwrap();
        let fd2 = match output2 {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // FDs should be different
        assert_ne!(fd1, fd2);
        assert_eq!(fd1, 3);
        assert_eq!(fd2, 4);

        // Both should be open
        let proc = new_state.get_process(pid).unwrap();
        assert!(proc.is_fd_open(fd1));
        assert!(proc.is_fd_open(fd2));
    }

    #[test]
    fn test_sys_close_releases_fd() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create and open file
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), vec![])
            .unwrap();
        let result = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Close the file
        let result = dispatch_syscall(state.clone(), SystemCall::Close { fd }, pid);
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Success);

        // FD should be closed
        let proc = new_state.get_process(pid).unwrap();
        assert!(!proc.is_fd_open(fd));
    }

    #[test]
    fn test_sys_close_invalid_fd() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to close invalid FD
        let result = dispatch_syscall(state, SystemCall::Close { fd: 999 }, pid);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KernelError::InvalidFileDescriptor(999)
        ));
    }

    #[test]
    fn test_sys_close_standard_streams() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to close stdin
        let result = dispatch_syscall(state.clone(), SystemCall::Close { fd: 0 }, pid);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KernelError::InvalidFileDescriptor(0)
        ));

        // Try to close stdout
        let result = dispatch_syscall(state.clone(), SystemCall::Close { fd: 1 }, pid);
        assert!(result.is_err());

        // Try to close stderr
        let result = dispatch_syscall(state, SystemCall::Close { fd: 2 }, pid);
        assert!(result.is_err());
    }

    #[test]
    fn test_standard_streams_initialized() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Verify standard streams are initialized
        let proc = state.get_process(pid).unwrap();
        assert!(proc.is_fd_open(0)); // stdin
        assert!(proc.is_fd_open(1)); // stdout
        assert!(proc.is_fd_open(2)); // stderr

        assert_eq!(proc.get_file_path(0), Some(&PathBuf::from("/dev/stdin")));
        assert_eq!(proc.get_file_path(1), Some(&PathBuf::from("/dev/stdout")));
        assert_eq!(proc.get_file_path(2), Some(&PathBuf::from("/dev/stderr")));
    }

    #[test]
    fn test_fd_table_per_process() {
        let mut state = KernelState::new();

        // Create two processes
        let pid1 = state.allocate_pid();
        let proc1 = Process::new(pid1, None);
        state.add_process(proc1);

        let pid2 = state.allocate_pid();
        let proc2 = Process::new(pid2, None);
        state.add_process(proc2);

        // Create file in VFS
        state
            .vfs
            .create_file(PathBuf::from("/shared.txt"), vec![])
            .unwrap();

        // Open file in process 1
        let result1 = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/shared.txt".to_string(),
                flags: 0,
            },
            pid1,
        );
        let (state, output1) = result1.unwrap();
        let fd1 = match output1 {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Open same file in process 2
        let result2 = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/shared.txt".to_string(),
                flags: 0,
            },
            pid2,
        );
        let (new_state, output2) = result2.unwrap();
        let fd2 = match output2 {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Both should have FD 3 (independent FD tables)
        assert_eq!(fd1, 3);
        assert_eq!(fd2, 3);

        // FD should be open in both processes
        let proc1 = new_state.get_process(pid1).unwrap();
        let proc2 = new_state.get_process(pid2).unwrap();
        assert!(proc1.is_fd_open(fd1));
        assert!(proc2.is_fd_open(fd2));
    }

    #[test]
    fn test_sys_read_from_file() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create file with content
        let content = b"Hello, World!".to_vec();
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), content.clone())
            .unwrap();

        // Open file
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Read from file
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 100 }, pid);
        let (_, output) = result.unwrap();
        match output {
            SyscallOutput::Data(data) => {
                assert_eq!(data, content);
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_sys_read_partial() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create file with content
        let content = b"Hello, World!".to_vec();
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), content.clone())
            .unwrap();

        // Open file
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Read only 5 bytes
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 5 }, pid);
        let (_, output) = result.unwrap();
        match output {
            SyscallOutput::Data(data) => {
                assert_eq!(data, b"Hello".to_vec());
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_sys_read_invalid_fd() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to read from invalid FD
        let result = dispatch_syscall(state, SystemCall::Read { fd: 99, count: 100 }, pid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::InvalidFileDescriptor(99));
    }

    #[test]
    fn test_sys_read_nonexistent_file() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let mut proc = Process::new(pid, None);

        // Manually add FD pointing to nonexistent file
        proc.open_file(PathBuf::from("/nonexistent.txt"));
        state.add_process(proc);

        // Try to read
        let result = dispatch_syscall(state, SystemCall::Read { fd: 3, count: 100 }, pid);
        assert!(result.is_err());
        match result.unwrap_err() {
            KernelError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_sys_write_to_file() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create empty file
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), vec![])
            .unwrap();

        // Open file
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Write to file
        let data = b"Hello, World!".to_vec();
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Write {
                fd,
                data: data.clone(),
            },
            pid,
        );
        let (new_state, output) = result.unwrap();

        // Should return number of bytes written
        match output {
            SyscallOutput::Value(bytes_written) => {
                assert_eq!(bytes_written, data.len() as i32);
            }
            _ => panic!("Expected Value"),
        }

        // Verify file was written
        let file_content = new_state
            .vfs
            .read_file(&PathBuf::from("/test.txt"))
            .unwrap();
        assert_eq!(file_content, data);
    }

    #[test]
    fn test_sys_write_invalid_fd() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to write to invalid FD
        let result = dispatch_syscall(
            state,
            SystemCall::Write {
                fd: 99,
                data: vec![1, 2, 3],
            },
            pid,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::InvalidFileDescriptor(99));
    }

    #[test]
    fn test_sys_write_nonexistent_file() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let mut proc = Process::new(pid, None);

        // Manually add FD pointing to nonexistent file
        proc.open_file(PathBuf::from("/nonexistent.txt"));
        state.add_process(proc);

        // Try to write
        let result = dispatch_syscall(
            state,
            SystemCall::Write {
                fd: 3,
                data: vec![1, 2, 3],
            },
            pid,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            KernelError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_read_write_cycle() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create empty file
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), vec![])
            .unwrap();

        // Open file
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Write data
        let write_data = b"Test data".to_vec();
        let result = dispatch_syscall(
            state,
            SystemCall::Write {
                fd,
                data: write_data.clone(),
            },
            pid,
        );
        let (state, _) = result.unwrap();

        // Read data back
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 100 }, pid);
        let (_, output) = result.unwrap();
        match output {
            SyscallOutput::Data(data) => {
                assert_eq!(data, write_data);
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_read_permission_denied() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create file with no read permission
        state
            .vfs
            .create_file(PathBuf::from("/secret.txt"), b"secret".to_vec())
            .unwrap();
        let no_read_perms = wos_shared::vfs::FilePermissions {
            read: false,
            write: true,
            execute: false,
        };
        state
            .vfs
            .set_permissions(&PathBuf::from("/secret.txt"), no_read_perms)
            .unwrap();

        // Open file (doesn't check permissions)
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/secret.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Try to read (should fail with permission denied)
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 100 }, pid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::PermissionDenied);
    }

    #[test]
    fn test_write_permission_denied() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create file with no write permission
        state
            .vfs
            .create_file(PathBuf::from("/readonly.txt"), vec![])
            .unwrap();
        state
            .vfs
            .set_permissions(
                &PathBuf::from("/readonly.txt"),
                wos_shared::vfs::FilePermissions::read_only(),
            )
            .unwrap();

        // Open file
        let result = dispatch_syscall(
            state.clone(),
            SystemCall::Open {
                path: "/readonly.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Try to write (should fail with permission denied)
        let result = dispatch_syscall(
            state,
            SystemCall::Write {
                fd,
                data: b"data".to_vec(),
            },
            pid,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::PermissionDenied);
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

        // Test open
        let syscall = SystemCall::Open {
            path: "/test.txt".to_string(),
            flags: 0,
        };
        let json = serde_json::to_string(&syscall).unwrap();
        let syscall2: SystemCall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, syscall2);

        // Test close
        let syscall = SystemCall::Close { fd: 3 };
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
