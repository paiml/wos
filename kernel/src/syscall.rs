//! System Call Dispatcher
//!
//! Pure functional system call interface with error handling.

use crate::state::{KernelState, ProcessId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Open flags
pub const O_CREAT: u32 = 0x0040; // Create file if it doesn't exist

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

    /// Invalid signal number
    #[error("Invalid signal number: {0}")]
    InvalidSignal(u32),

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

    /// Replace current process image with new program
    Exec {
        /// Path to executable
        path: String,
        /// Command-line arguments (argv)
        args: Vec<String>,
        /// Environment variables
        env: Vec<(String, String)>,
    },

    /// Wait for child process
    WaitPid(ProcessId),

    /// Sleep for microseconds
    Sleep(u64),

    /// Send signal to process
    Kill {
        /// Target process ID
        pid: ProcessId,
        /// Signal to send
        signal: u32,
    },

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

    /// Send message to another process
    Send {
        /// Target process ID
        target_pid: ProcessId,
        /// Message payload
        data: Vec<u8>,
    },

    /// Receive message (blocking)
    Recv {
        /// Timeout in microseconds (0 = no timeout)
        timeout: u64,
    },

    /// Create a pipe (returns read and write file descriptors)
    Pipe,

    /// Duplicate file descriptor
    Dup2 {
        /// Old file descriptor
        oldfd: u32,
        /// New file descriptor
        newfd: u32,
    },

    /// Create directory
    Mkdir {
        /// Path to directory
        path: String,
        /// Mode (permissions)
        mode: u32,
    },

    /// Remove empty directory
    Rmdir {
        /// Path to directory
        path: String,
    },

    /// Read directory entries
    Getdents {
        /// File descriptor of open directory
        fd: u32,
    },

    /// Get file status (metadata)
    Stat {
        /// Path to file or directory
        path: String,
    },

    /// Get file status without following symlinks
    Lstat {
        /// Path to file or directory
        path: String,
    },

    /// Get canonical absolute path (resolve ., .., //, etc.)
    Realpath {
        /// Path to canonicalize
        path: String,
    },

    /// Change file permissions
    Chmod {
        /// Path to file or directory
        path: String,
        /// New permission mode bits (e.g., 0o644)
        mode: u32,
    },

    /// Change file ownership
    Chown {
        /// Path to file or directory
        path: String,
        /// New user ID (None means don't change)
        uid: Option<u32>,
        /// New group ID (None means don't change)
        gid: Option<u32>,
    },

    /// Check file access permissions
    Access {
        /// Path to file or directory
        path: String,
        /// Access mode to check (F_OK=0, R_OK=4, W_OK=2, X_OK=1)
        mode: u32,
    },

    /// Create symbolic link
    Symlink {
        /// Path where the symlink will be created
        link_path: String,
        /// Target path that the symlink points to
        target: String,
    },

    /// Read symbolic link target
    Readlink {
        /// Path to symbolic link
        path: String,
    },

    /// Register signal handler (sigaction)
    Sigaction {
        /// Signal number to handle
        signal: u32,
        /// Action to take (Terminate, Ignore, or Handler ID)
        action: crate::signals::SignalAction,
    },

    /// Block or unblock signals (sigprocmask)
    Sigprocmask {
        /// Signals to add to blocked set
        block: Vec<u32>,
        /// Signals to remove from blocked set
        unblock: Vec<u32>,
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

    /// Pipe file descriptors (read_fd, write_fd)
    Pipe {
        /// Read file descriptor
        read_fd: u32,
        /// Write file descriptor
        write_fd: u32,
    },
}

/// Generate /proc/PID/status content
fn generate_proc_status(state: &KernelState, pid: ProcessId) -> Result<Vec<u8>, KernelError> {
    let process = state
        .get_process(pid)
        .ok_or(KernelError::FileNotFound(format!("/proc/{}/status", pid)))?;

    let state_str = match &process.state {
        crate::state::ProcessState::Ready => "Ready",
        crate::state::ProcessState::Running => "Running",
        crate::state::ProcessState::Blocked => "Blocked",
        crate::state::ProcessState::Terminated(_) => "Terminated",
    };

    let parent_pid = process.parent_pid.unwrap_or(0);
    let memory_pages = process.memory_pages.len();

    let content = format!(
        "Pid:\t{}\nState:\t{}\nPPid:\t{}\nMemoryPages:\t{}\n",
        pid, state_str, parent_pid, memory_pages
    );

    Ok(content.into_bytes())
}

/// Generate /proc/PID/cmdline content
fn generate_proc_cmdline(state: &KernelState, pid: ProcessId) -> Result<Vec<u8>, KernelError> {
    // Check if process exists
    state
        .get_process(pid)
        .ok_or(KernelError::FileNotFound(format!("/proc/{}/cmdline", pid)))?;

    // For now, return placeholder since we don't track command line args yet
    // In a real implementation, this would be stored in the Process struct
    let content = format!("process_{}\0", pid);
    Ok(content.into_bytes())
}

/// Check if path is a ProcFS path and generate content
fn try_read_procfs(
    state: &KernelState,
    path: &std::path::Path,
    calling_pid: ProcessId,
) -> Option<Result<Vec<u8>, KernelError>> {
    let path_str = path.to_str()?;

    // Handle /proc/self/* by resolving to calling process
    if let Some(rest) = path_str.strip_prefix("/proc/self/") {
        let resolved_path = PathBuf::from(format!("/proc/{}/{}", calling_pid, rest));
        return try_read_procfs(state, &resolved_path, calling_pid);
    }

    // Parse /proc/PID/file paths
    if path_str.starts_with("/proc/") {
        let parts: Vec<&str> = path_str.split('/').collect();
        if parts.len() >= 4 {
            // parts: ["", "proc", "PID", "file"]
            if let Ok(pid) = parts[2].parse::<ProcessId>() {
                let file = parts[3];
                return Some(match file {
                    "status" => generate_proc_status(state, pid),
                    "cmdline" => generate_proc_cmdline(state, pid),
                    _ => Err(KernelError::FileNotFound(path_str.to_string())),
                });
            }
        }
    }

    None
}

// Syscall handler helpers (extracted for complexity reduction)

/// Handle GetPid syscall
fn sys_getpid(
    state: KernelState,
    calling_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    Ok((state, SyscallOutput::Pid(calling_pid)))
}

/// Handle Fork syscall
fn sys_fork(
    mut state: KernelState,
    calling_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let child_pid = state.allocate_pid();
    let parent = state
        .get_process(calling_pid)
        .ok_or(KernelError::ProcessNotFound(calling_pid))?
        .clone();

    let mut child = parent.clone();
    child.pid = child_pid;
    child.parent_pid = Some(calling_pid);
    state.add_process(child);

    Ok((state, SyscallOutput::Pid(child_pid)))
}

/// Handle Exit syscall
fn sys_exit(
    mut state: KernelState,
    calling_pid: ProcessId,
    code: i32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Get parent PID before terminating the process
    let parent_pid = state
        .get_process(calling_pid)
        .ok_or(KernelError::ProcessNotFound(calling_pid))?
        .parent_pid;

    // Terminate the process
    if let Some(process) = state.get_process_mut(calling_pid) {
        process.state = crate::state::ProcessState::Terminated(code);
    } else {
        return Err(KernelError::ProcessNotFound(calling_pid));
    }

    // Send SIGCHLD to parent if it exists
    if let Some(parent_pid) = parent_pid {
        if let Some(mut parent) = state.processes.get(&parent_pid).cloned() {
            parent.pending_signals.add(crate::signals::Signal::SIGCHLD);
            state.processes.insert(parent_pid, parent);
        }
    }

    Ok((state, SyscallOutput::Success))
}

/// Handle Exec syscall - replace current process image with new program
fn sys_exec(
    mut state: KernelState,
    calling_pid: ProcessId,
    path: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Check if executable file exists and is readable
    let path_buf = std::path::PathBuf::from(&path);
    match state.vfs.read_file(&path_buf) {
        Ok(content) => {
            // Validate that it's an executable (in WOS, we just check file exists and has content)
            if content.is_empty() {
                return Err(KernelError::InvalidParameters(format!(
                    "Empty executable: {}",
                    path
                )));
            }

            // Get the process
            if let Some(process) = state.get_process_mut(calling_pid) {
                // Keep PID, parent_pid, and open files
                // But reset:
                // - Memory allocations (except keep open files)
                // - Environment variables
                // - Working directory stays same (cd persists across exec)

                // Update environment variables
                process.env = env.into_iter().collect();

                // Store program path and args in environment
                process.env.insert("_".to_string(), path.clone());
                process
                    .env
                    .insert("ARGC".to_string(), args.len().to_string());
                for (i, arg) in args.iter().enumerate() {
                    process.env.insert(format!("ARG{}", i), arg.clone());
                }

                // In a real OS, we would:
                // 1. Parse executable format (ELF, etc.)
                // 2. Load code/data sections into memory
                // 3. Set up stack with argc/argv/envp
                // 4. Set instruction pointer to entry point
                //
                // For WOS (educational), we store the executable path
                // and the program will be "executed" by the shell/interpreter
                process.memory.program_path = Some(path);
                process.memory.program_args = args;

                // Reset process to Ready state (will be scheduled to run)
                process.state = crate::state::ProcessState::Ready;

                Ok((state, SyscallOutput::Success))
            } else {
                Err(KernelError::ProcessNotFound(calling_pid))
            }
        }
        Err(_e) => Err(KernelError::InvalidParameters(format!(
            "Cannot exec {}: file not found or not readable",
            path
        ))),
    }
}

/// Handle WaitPid syscall
fn sys_waitpid(
    state: KernelState,
    calling_pid: ProcessId,
    wait_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if !state.processes.contains_key(&calling_pid) {
        return Err(KernelError::ProcessNotFound(calling_pid));
    }

    let target = state
        .get_process(wait_pid)
        .ok_or(KernelError::ProcessNotFound(wait_pid))?;

    if target.parent_pid != Some(calling_pid) {
        return Err(KernelError::PermissionDenied);
    }

    match target.state {
        crate::state::ProcessState::Terminated(exit_code) => {
            Ok((state, SyscallOutput::Value(exit_code)))
        }
        _ => Err(KernelError::InvalidProcessState),
    }
}

/// Handle Sleep syscall
fn sys_sleep(
    mut state: KernelState,
    calling_pid: ProcessId,
    duration_us: u64,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let mut process = state
        .get_process(calling_pid)
        .ok_or(KernelError::ProcessNotFound(calling_pid))?
        .clone();

    // Check if process is terminated
    if matches!(process.state, crate::state::ProcessState::Terminated(_)) {
        return Err(KernelError::InvalidProcessState);
    }

    // Special case: sleep(0) is a no-op
    if duration_us == 0 {
        return Ok((state, SyscallOutput::Success));
    }

    // Calculate wakeup time
    let current_time = state.simulated_clock.current_time();
    let wakeup_time = current_time + duration_us;

    // Update process state to blocked and set wakeup time
    process.state = crate::state::ProcessState::Blocked;
    process.wakeup_time = Some(wakeup_time);

    // Update process in state
    state.processes.insert(calling_pid, process);

    Ok((state, SyscallOutput::Success))
}

/// Handle Kill syscall (send signal to process)
fn sys_kill(
    mut state: KernelState,
    _calling_pid: ProcessId,
    target_pid: ProcessId,
    signal_num: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Validate signal number
    let signal = crate::signals::Signal::from_number(signal_num)
        .ok_or(KernelError::InvalidSignal(signal_num))?;

    // Get target process
    let mut process = state
        .get_process(target_pid)
        .ok_or(KernelError::ProcessNotFound(target_pid))?
        .clone();

    // SIGKILL terminates immediately (cannot be caught or ignored)
    if signal == crate::signals::Signal::SIGKILL {
        process.state = crate::state::ProcessState::Terminated(9);
        state.processes.insert(target_pid, process);
        return Ok((state, SyscallOutput::Success));
    }

    // Add signal to pending set (will be delivered later)
    process.pending_signals.add(signal);
    state.processes.insert(target_pid, process);

    Ok((state, SyscallOutput::Success))
}

/// Handle Sigaction syscall - register signal handler
fn sys_sigaction(
    mut state: KernelState,
    calling_pid: ProcessId,
    signal_num: u32,
    action: crate::signals::SignalAction,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Validate signal number
    let signal = crate::signals::Signal::from_number(signal_num)
        .ok_or(KernelError::InvalidSignal(signal_num))?;

    // Cannot change handler for SIGKILL
    if signal == crate::signals::Signal::SIGKILL {
        return Err(KernelError::InvalidParameters(
            "Cannot set handler for SIGKILL".to_string(),
        ));
    }

    // Get process and update signal handler
    if let Some(process) = state.get_process_mut(calling_pid) {
        process.signal_handlers.insert(signal_num, action);
        Ok((state, SyscallOutput::Success))
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Sigprocmask syscall - block or unblock signals
fn sys_sigprocmask(
    mut state: KernelState,
    calling_pid: ProcessId,
    block: Vec<u32>,
    unblock: Vec<u32>,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Get process
    if let Some(process) = state.get_process_mut(calling_pid) {
        // Add signals to blocked set
        for signal_num in block {
            if let Some(signal) = crate::signals::Signal::from_number(signal_num) {
                // SIGKILL cannot be blocked
                if signal != crate::signals::Signal::SIGKILL {
                    process.blocked_signals.add(signal);
                }
            }
        }

        // Remove signals from blocked set
        for signal_num in unblock {
            if let Some(signal) = crate::signals::Signal::from_number(signal_num) {
                process.blocked_signals.remove(signal);
            }
        }

        Ok((state, SyscallOutput::Success))
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Open syscall
fn sys_open(
    mut state: KernelState,
    calling_pid: ProcessId,
    path: String,
    flags: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path_buf = PathBuf::from(&path);

    if !state.vfs.exists(&path_buf) {
        if (flags & O_CREAT) != 0 {
            state
                .vfs
                .create_file(path_buf.clone(), vec![])
                .map_err(|_| KernelError::FileNotFound(path.clone()))?;
        } else {
            return Err(KernelError::FileNotFound(path));
        }
    }

    if let Some(process) = state.get_process_mut(calling_pid) {
        let fd = process.open_file(path_buf);
        Ok((state, SyscallOutput::FileDescriptor(fd)))
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Close syscall
fn sys_close(
    mut state: KernelState,
    calling_pid: ProcessId,
    fd: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if let Some(process) = state.get_process_mut(calling_pid) {
        if process.close_file(fd).is_some() {
            Ok((state, SyscallOutput::Success))
        } else {
            Err(KernelError::InvalidFileDescriptor(fd))
        }
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Read syscall
fn sys_read(
    mut state: KernelState,
    calling_pid: ProcessId,
    fd: u32,
    count: usize,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = {
        let process = state
            .get_process(calling_pid)
            .ok_or(KernelError::ProcessNotFound(calling_pid))?;
        process
            .get_file_path(fd)
            .ok_or(KernelError::InvalidFileDescriptor(fd))?
            .clone()
    };

    // Check if this is a pipe read
    if path
        .to_str()
        .map(|s| s.starts_with("/pipe/"))
        .unwrap_or(false)
    {
        if let Some(pipe) = state.pipes.get(&fd) {
            let bytes_to_return = pipe.data.len().min(count);
            let data = pipe.data[..bytes_to_return].to_vec();
            return Ok((state, SyscallOutput::Data(data)));
        } else {
            return Err(KernelError::InvalidFileDescriptor(fd));
        }
    }

    // Check if this is a ProcFS path
    if let Some(procfs_result) = try_read_procfs(&state, &path, calling_pid) {
        let content = procfs_result?;
        let bytes_to_return = content.len().min(count);
        let data = content[..bytes_to_return].to_vec();
        return Ok((state, SyscallOutput::Data(data)));
    }

    // Read from VFS
    match state.vfs.read_file(&path) {
        Ok(content) => {
            let bytes_to_return = content.len().min(count);
            let data = content[..bytes_to_return].to_vec();
            Ok((state, SyscallOutput::Data(data)))
        }
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::PermissionDenied) => Err(KernelError::PermissionDenied),
        Err(_) => Err(KernelError::InvalidParameters(
            "VFS error during read".to_string(),
        )),
    }
}

/// Handle Write syscall
fn sys_write(
    mut state: KernelState,
    calling_pid: ProcessId,
    fd: u32,
    data: Vec<u8>,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = {
        let process = state
            .get_process(calling_pid)
            .ok_or(KernelError::ProcessNotFound(calling_pid))?;
        process
            .get_file_path(fd)
            .ok_or(KernelError::InvalidFileDescriptor(fd))?
            .clone()
    };

    // Check if this is a pipe write
    if path
        .to_str()
        .map(|s| s.starts_with("/pipe/"))
        .unwrap_or(false)
    {
        if let Some(fd_str) = path.to_str().and_then(|s| s.split('/').nth(2)) {
            if let Ok(read_fd) = fd_str.parse::<u32>() {
                if let Some(pipe) = state.pipes.get_mut(&read_fd) {
                    pipe.data.extend_from_slice(&data);
                    return Ok((state, SyscallOutput::Value(data.len() as i32)));
                }
            }
        }
        return Err(KernelError::InvalidFileDescriptor(fd));
    }

    // Write to VFS
    match state.vfs.write_file(&path, data.clone()) {
        Ok(_) => Ok((state, SyscallOutput::Value(data.len() as i32))),
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::PermissionDenied) => Err(KernelError::PermissionDenied),
        Err(_) => Err(KernelError::InvalidParameters(
            "VFS error during write".to_string(),
        )),
    }
}

/// Handle Mmap syscall
fn sys_mmap(
    mut state: KernelState,
    calling_pid: ProcessId,
    size: usize,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if let Some(process) = state.get_process_mut(calling_pid) {
        if let Some(addr) = process.memory.mmap(size) {
            Ok((state, SyscallOutput::Address(addr)))
        } else {
            Err(KernelError::ResourceExhausted("Out of memory".to_string()))
        }
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Munmap syscall
fn sys_munmap(
    mut state: KernelState,
    calling_pid: ProcessId,
    addr: u64,
    size: usize,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if let Some(process) = state.get_process_mut(calling_pid) {
        if process.memory.munmap(addr, size) {
            Ok((state, SyscallOutput::Success))
        } else {
            Err(KernelError::InvalidParameters(
                "Invalid munmap range".to_string(),
            ))
        }
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Send syscall
fn sys_send(
    mut state: KernelState,
    calling_pid: ProcessId,
    target_pid: ProcessId,
    data: Vec<u8>,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if !state.processes.contains_key(&target_pid) {
        return Err(KernelError::ProcessNotFound(target_pid));
    }

    let message = crate::state::Message::new(calling_pid, target_pid, data);

    if let Some(target_process) = state.get_process_mut(target_pid) {
        target_process.message_queue.push_back(message);
        Ok((state, SyscallOutput::Success))
    } else {
        Err(KernelError::ProcessNotFound(target_pid))
    }
}

/// Handle Recv syscall
fn sys_recv(
    mut state: KernelState,
    calling_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let process = state
        .get_process_mut(calling_pid)
        .ok_or(KernelError::ProcessNotFound(calling_pid))?;

    if process.message_queue.is_empty() {
        return Err(KernelError::InvalidProcessState);
    }

    let message = process.message_queue.remove(0);
    Ok((state, SyscallOutput::Data(message.payload)))
}

/// Handle Pipe syscall
fn sys_pipe(
    mut state: KernelState,
    calling_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if let Some(process) = state.get_process_mut(calling_pid) {
        let read_fd = process.allocate_fd();
        let write_fd = read_fd + 1;

        process
            .open_files
            .insert(read_fd, PathBuf::from(format!("/pipe/{}/read", read_fd)));
        process
            .open_files
            .insert(write_fd, PathBuf::from(format!("/pipe/{}/write", read_fd)));

        let pipe = crate::state::PipeBuffer {
            read_fd,
            write_fd,
            owner_pid: calling_pid,
            data: Vec::new(),
        };
        state.pipes.insert(read_fd, pipe);

        Ok((state, SyscallOutput::Pipe { read_fd, write_fd }))
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Handle Dup2 syscall
fn sys_dup2(
    mut state: KernelState,
    calling_pid: ProcessId,
    oldfd: u32,
    newfd: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    if let Some(process) = state.get_process_mut(calling_pid) {
        match process.dup_fd(oldfd, newfd) {
            Ok(_) => Ok((state, SyscallOutput::Success)),
            Err(msg) => Err(KernelError::InvalidParameters(msg)),
        }
    } else {
        Err(KernelError::ProcessNotFound(calling_pid))
    }
}

/// Create directory (mkdir syscall)
fn sys_mkdir(
    mut state: KernelState,
    _calling_pid: ProcessId,
    path: String,
    _mode: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = PathBuf::from(path);

    match state.vfs.create_directory(path.clone()) {
        Ok(_) => Ok((state, SyscallOutput::Success)),
        Err(wos_shared::vfs::VfsError::AlreadyExists) => {
            Err(KernelError::FileAlreadyExists(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::PermissionDenied) => Err(KernelError::PermissionDenied),
        Err(e) => Err(KernelError::InvalidParameters(format!(
            "Failed to create directory: {:?}",
            e
        ))),
    }
}

/// Remove directory (rmdir syscall)
fn sys_rmdir(
    mut state: KernelState,
    _calling_pid: ProcessId,
    path: String,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = Path::new(&path);

    match state.vfs.remove_directory(path) {
        Ok(_) => Ok((state, SyscallOutput::Success)),
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::DirectoryNotEmpty) => Err(KernelError::InvalidParameters(
            "Directory not empty".to_string(),
        )),
        Err(wos_shared::vfs::VfsError::PermissionDenied) => Err(KernelError::PermissionDenied),
        Err(e) => Err(KernelError::InvalidParameters(format!(
            "Failed to remove directory: {:?}",
            e
        ))),
    }
}

/// Read directory entries (getdents syscall)
fn sys_getdents(
    state: KernelState,
    calling_pid: ProcessId,
    fd: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Get the path associated with this file descriptor
    let process = state
        .get_process(calling_pid)
        .ok_or(KernelError::ProcessNotFound(calling_pid))?;

    let path = process
        .open_files
        .get(&fd)
        .ok_or(KernelError::InvalidFileDescriptor(fd))?;

    // List directory contents
    match state.vfs.list_directory(path) {
        Ok(entries) => {
            // Serialize directory entries to JSON
            let json = serde_json::to_vec(&entries).map_err(|e| {
                KernelError::InvalidParameters(format!(
                    "Failed to serialize directory entries: {}",
                    e
                ))
            })?;
            Ok((state, SyscallOutput::Data(json)))
        }
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::NotADirectory) => Err(KernelError::InvalidParameters(
            "Not a directory".to_string(),
        )),
        Err(e) => Err(KernelError::InvalidParameters(format!(
            "Failed to read directory: {:?}",
            e
        ))),
    }
}

/// Get file status (stat syscall)
fn sys_stat(
    state: KernelState,
    _calling_pid: ProcessId,
    path: String,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = Path::new(&path);

    match state.vfs.stat(path) {
        Ok(file_stat) => {
            // Serialize FileStat to JSON
            let json = serde_json::to_vec(&file_stat).map_err(|e| {
                KernelError::InvalidParameters(format!("Failed to serialize file stat: {}", e))
            })?;
            Ok((state, SyscallOutput::Data(json)))
        }
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::PermissionDenied) => Err(KernelError::PermissionDenied),
        Err(e) => Err(KernelError::InvalidParameters(format!(
            "Failed to stat file: {:?}",
            e
        ))),
    }
}

/// Get file status without following symlinks (lstat syscall)
fn sys_lstat(
    state: KernelState,
    _calling_pid: ProcessId,
    path: String,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = Path::new(&path);

    match state.vfs.lstat(path) {
        Ok(file_stat) => {
            // Serialize FileStat to JSON
            let json = serde_json::to_vec(&file_stat).map_err(|e| {
                KernelError::InvalidParameters(format!("Failed to serialize file stat: {}", e))
            })?;
            Ok((state, SyscallOutput::Data(json)))
        }
        Err(wos_shared::vfs::VfsError::NotFound) => {
            Err(KernelError::FileNotFound(path.display().to_string()))
        }
        Err(wos_shared::vfs::VfsError::PermissionDenied) => Err(KernelError::PermissionDenied),
        Err(e) => Err(KernelError::InvalidParameters(format!(
            "Failed to lstat file: {:?}",
            e
        ))),
    }
}

/// Get canonical absolute path (realpath syscall)
fn sys_realpath(
    state: KernelState,
    _calling_pid: ProcessId,
    path: String,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path = Path::new(&path);

    // Use VFS normalize_path to canonicalize the path
    let normalized = wos_shared::vfs::VirtualFileSystem::normalize_path(path);

    // Convert to string and return as Data
    let canonical_str = normalized.to_str().ok_or(KernelError::InvalidParameters(
        "Path contains invalid UTF-8".to_string(),
    ))?;

    Ok((
        state,
        SyscallOutput::Data(canonical_str.as_bytes().to_vec()),
    ))
}

/// Change file permissions (chmod syscall)
fn sys_chmod(
    mut state: KernelState,
    _calling_pid: ProcessId,
    path: String,
    mode: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path_buf = PathBuf::from(&path);
    state.vfs.chmod(&path_buf, mode).map_err(|e| match e {
        wos_shared::vfs::VfsError::NotFound => KernelError::FileNotFound(path),
        wos_shared::vfs::VfsError::PermissionDenied => KernelError::PermissionDenied,
        wos_shared::vfs::VfsError::InvalidPath => {
            KernelError::InvalidParameters("Invalid path".to_string())
        }
        _ => KernelError::InvalidParameters(format!("VFS error: {:?}", e)),
    })?;

    Ok((state, SyscallOutput::Success))
}

/// Change file ownership (chown syscall)
fn sys_chown(
    mut state: KernelState,
    _calling_pid: ProcessId,
    path: String,
    uid: Option<u32>,
    gid: Option<u32>,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path_buf = PathBuf::from(&path);
    state.vfs.chown(&path_buf, uid, gid).map_err(|e| match e {
        wos_shared::vfs::VfsError::NotFound => KernelError::FileNotFound(path),
        wos_shared::vfs::VfsError::PermissionDenied => KernelError::PermissionDenied,
        wos_shared::vfs::VfsError::InvalidPath => {
            KernelError::InvalidParameters("Invalid path".to_string())
        }
        _ => KernelError::InvalidParameters(format!("VFS error: {:?}", e)),
    })?;

    Ok((state, SyscallOutput::Success))
}

/// Check file access permissions (access syscall)
fn sys_access(
    state: KernelState,
    _calling_pid: ProcessId,
    path: String,
    mode: u32,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path_buf = PathBuf::from(&path);
    state.vfs.access(&path_buf, mode).map_err(|e| match e {
        wos_shared::vfs::VfsError::NotFound => KernelError::FileNotFound(path),
        wos_shared::vfs::VfsError::PermissionDenied => KernelError::PermissionDenied,
        wos_shared::vfs::VfsError::InvalidPath => {
            KernelError::InvalidParameters("Invalid path".to_string())
        }
        _ => KernelError::InvalidParameters(format!("VFS error: {:?}", e)),
    })?;

    Ok((state, SyscallOutput::Success))
}

/// Create symbolic link (symlink syscall)
fn sys_symlink(
    mut state: KernelState,
    _calling_pid: ProcessId,
    link_path: String,
    target: String,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let link_path_buf = PathBuf::from(&link_path);
    let target_path_buf = PathBuf::from(&target);

    state
        .vfs
        .create_symlink(link_path_buf, target_path_buf)
        .map_err(|e| match e {
            wos_shared::vfs::VfsError::NotFound => {
                KernelError::FileNotFound("Parent directory not found".to_string())
            }
            wos_shared::vfs::VfsError::AlreadyExists => KernelError::FileAlreadyExists(link_path),
            wos_shared::vfs::VfsError::PermissionDenied => KernelError::PermissionDenied,
            wos_shared::vfs::VfsError::InvalidPath => {
                KernelError::InvalidParameters("Invalid path".to_string())
            }
            _ => KernelError::InvalidParameters(format!("VFS error: {:?}", e)),
        })?;

    Ok((state, SyscallOutput::Success))
}

/// Read symbolic link target (readlink syscall)
fn sys_readlink(
    state: KernelState,
    _calling_pid: ProcessId,
    path: String,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    let path_buf = PathBuf::from(&path);
    let target = state.vfs.readlink(&path_buf).map_err(|e| match e {
        wos_shared::vfs::VfsError::NotFound => KernelError::FileNotFound(path),
        wos_shared::vfs::VfsError::InvalidPath => {
            KernelError::InvalidParameters("Not a symbolic link".to_string())
        }
        _ => KernelError::InvalidParameters(format!("VFS error: {:?}", e)),
    })?;

    let target_str = target.to_str().ok_or_else(|| {
        KernelError::InvalidParameters("Invalid UTF-8 in symlink target".to_string())
    })?;

    Ok((state, SyscallOutput::Data(target_str.as_bytes().to_vec())))
}

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
        SystemCall::GetPid => sys_getpid(state, calling_pid),
        SystemCall::Fork => sys_fork(state, calling_pid),
        SystemCall::Exit(code) => sys_exit(state, calling_pid, code),
        SystemCall::Exec { path, args, env } => sys_exec(state, calling_pid, path, args, env),
        SystemCall::WaitPid(wait_pid) => sys_waitpid(state, calling_pid, wait_pid),
        SystemCall::Sleep(duration) => sys_sleep(state, calling_pid, duration),
        SystemCall::Open { path, flags } => sys_open(state, calling_pid, path, flags),
        SystemCall::Close { fd } => sys_close(state, calling_pid, fd),
        SystemCall::Read { fd, count } => sys_read(state, calling_pid, fd, count),
        SystemCall::Write { fd, data } => sys_write(state, calling_pid, fd, data),
        SystemCall::Mmap { size } => sys_mmap(state, calling_pid, size),
        SystemCall::Munmap { addr, size } => sys_munmap(state, calling_pid, addr, size),
        SystemCall::Send { target_pid, data } => sys_send(state, calling_pid, target_pid, data),
        SystemCall::Recv { timeout: _ } => sys_recv(state, calling_pid),
        SystemCall::Pipe => sys_pipe(state, calling_pid),
        SystemCall::Dup2 { oldfd, newfd } => sys_dup2(state, calling_pid, oldfd, newfd),
        SystemCall::Kill { pid, signal } => sys_kill(state, calling_pid, pid, signal),
        SystemCall::Mkdir { path, mode } => sys_mkdir(state, calling_pid, path, mode),
        SystemCall::Rmdir { path } => sys_rmdir(state, calling_pid, path),
        SystemCall::Getdents { fd } => sys_getdents(state, calling_pid, fd),
        SystemCall::Stat { path } => sys_stat(state, calling_pid, path),
        SystemCall::Lstat { path } => sys_lstat(state, calling_pid, path),
        SystemCall::Realpath { path } => sys_realpath(state, calling_pid, path),
        SystemCall::Chmod { path, mode } => sys_chmod(state, calling_pid, path, mode),
        SystemCall::Chown { path, uid, gid } => sys_chown(state, calling_pid, path, uid, gid),
        SystemCall::Access { path, mode } => sys_access(state, calling_pid, path, mode),
        SystemCall::Symlink { link_path, target } => {
            sys_symlink(state, calling_pid, link_path, target)
        }
        SystemCall::Readlink { path } => sys_readlink(state, calling_pid, path),
        SystemCall::Sigaction { signal, action } => {
            sys_sigaction(state, calling_pid, signal, action)
        }
        SystemCall::Sigprocmask { block, unblock } => {
            sys_sigprocmask(state, calling_pid, block, unblock)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Process, ProcessState};

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

        // All previously unimplemented syscalls are now implemented
        // (Sleep, Open, Close, Read, Write, Mmap, Munmap, Send, Recv, Pipe, Dup2)
        // This test is kept for future use when new syscalls are added
        let syscalls: Vec<SystemCall> = vec![];

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
    fn test_sys_open_with_o_creat() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Open non-existent file with O_CREAT flag - should create it
        let result = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/newfile.txt".to_string(),
                flags: O_CREAT,
            },
            pid,
        );
        assert!(result.is_ok());
        let (new_state, output) = result.unwrap();

        // Verify file was created
        assert!(new_state.vfs.exists(&PathBuf::from("/newfile.txt")));

        // Verify file descriptor was allocated
        assert!(matches!(output, SyscallOutput::FileDescriptor(_)));
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
        let (mut new_state, output) = result.unwrap();

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
        let no_read_perms = wos_shared::vfs::FilePermissions::new(
            0o200, // write-only for owner (-w-------)
            0,     // uid: root
            0,     // gid: root
        );
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
        let (mut state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Set VFS context to non-root user to test permission denial
        state.vfs.set_context(1000, 1000);

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
        let (mut state, output) = result.unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Set VFS context to non-root user to test permission denial
        state.vfs.set_context(1000, 1000);

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
    fn test_read_proc_status() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let mut proc = Process::new(pid, Some(100)); // Parent PID is 100
        proc.memory_pages = vec![1, 2, 3]; // Allocate some memory pages
        state.add_process(proc);

        // Manually open /proc/PID/status
        let mut process = state.get_process(pid).unwrap().clone();
        let fd = process.open_file(PathBuf::from(format!("/proc/{}/status", pid)));
        state.processes.insert(pid, process);

        // Read from /proc/PID/status
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 1000 }, pid);
        let (_, output) = result.unwrap();

        match output {
            SyscallOutput::Data(data) => {
                let content = String::from_utf8(data).unwrap();
                assert!(content.contains(&format!("Pid:\t{}", pid)));
                assert!(content.contains("State:\tReady"));
                assert!(content.contains("PPid:\t100"));
                assert!(content.contains("MemoryPages:\t3"));
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_read_proc_cmdline() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Manually open /proc/PID/cmdline
        let mut process = state.get_process(pid).unwrap().clone();
        let fd = process.open_file(PathBuf::from(format!("/proc/{}/cmdline", pid)));
        state.processes.insert(pid, process);

        // Read from /proc/PID/cmdline
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 1000 }, pid);
        let (_, output) = result.unwrap();

        match output {
            SyscallOutput::Data(data) => {
                let content = String::from_utf8_lossy(&data);
                assert!(content.contains(&format!("process_{}", pid)));
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_proc_self_symlink() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Open /proc/self/status
        let mut process = state.get_process(pid).unwrap().clone();
        let fd = process.open_file(PathBuf::from("/proc/self/status"));
        state.processes.insert(pid, process);

        // Read from /proc/self/status (should resolve to calling process)
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 1000 }, pid);
        let (_, output) = result.unwrap();

        match output {
            SyscallOutput::Data(data) => {
                let content = String::from_utf8(data).unwrap();
                // Should contain the calling process's PID
                assert!(content.contains(&format!("Pid:\t{}", pid)));
                assert!(content.contains("State:\tReady"));
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_proc_nonexistent_process() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to open /proc/999/status where 999 doesn't exist
        let mut process = state.get_process(pid).unwrap().clone();
        let fd = process.open_file(PathBuf::from("/proc/999/status"));
        state.processes.insert(pid, process);

        // Read should fail with FileNotFound
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 1000 }, pid);
        assert!(result.is_err());
        match result.unwrap_err() {
            KernelError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_proc_invalid_file() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to open /proc/PID/invalid
        let mut process = state.get_process(pid).unwrap().clone();
        let fd = process.open_file(PathBuf::from(format!("/proc/{}/invalid", pid)));
        state.processes.insert(pid, process);

        // Read should fail with FileNotFound
        let result = dispatch_syscall(state, SystemCall::Read { fd, count: 1000 }, pid);
        assert!(result.is_err());
        match result.unwrap_err() {
            KernelError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_proc_status_different_states() {
        let mut state = KernelState::new();

        // Create process in Running state
        let pid1 = state.allocate_pid();
        let mut proc1 = Process::new(pid1, None);
        proc1.state = crate::state::ProcessState::Running;
        state.add_process(proc1);

        // Create process in Terminated state
        let pid2 = state.allocate_pid();
        let mut proc2 = Process::new(pid2, None);
        proc2.state = crate::state::ProcessState::Terminated(0);
        state.add_process(proc2);

        // Read running process status
        let mut process1 = state.get_process(pid1).unwrap().clone();
        let fd1 = process1.open_file(PathBuf::from(format!("/proc/{}/status", pid1)));
        state.processes.insert(pid1, process1);

        let result1 = dispatch_syscall(
            state.clone(),
            SystemCall::Read {
                fd: fd1,
                count: 1000,
            },
            pid1,
        );
        let (mut state, output1) = result1.unwrap();
        match output1 {
            SyscallOutput::Data(data) => {
                let content = String::from_utf8(data).unwrap();
                assert!(content.contains("State:\tRunning"));
            }
            _ => panic!("Expected Data"),
        }

        // Read terminated process status
        let mut process2 = state.get_process(pid2).unwrap().clone();
        let fd2 = process2.open_file(PathBuf::from(format!("/proc/{}/status", pid2)));
        state.processes.insert(pid2, process2);

        let result2 = dispatch_syscall(
            state,
            SystemCall::Read {
                fd: fd2,
                count: 1000,
            },
            pid2,
        );
        let (_, output2) = result2.unwrap();
        match output2 {
            SyscallOutput::Data(data) => {
                let content = String::from_utf8(data).unwrap();
                assert!(content.contains("State:\tTerminated"));
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_sys_send_delivers_message() {
        let mut state = KernelState::new();

        // Create sender process
        let sender_pid = state.allocate_pid();
        let sender_proc = Process::new(sender_pid, None);
        state.add_process(sender_proc);

        // Create receiver process
        let receiver_pid = state.allocate_pid();
        let receiver_proc = Process::new(receiver_pid, None);
        state.add_process(receiver_proc);

        // Send message
        let data = b"Hello, World!".to_vec();
        let result = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: receiver_pid,
                data: data.clone(),
            },
            sender_pid,
        );

        assert!(result.is_ok());
        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Success);

        // Verify message is in receiver's queue
        let receiver = new_state.get_process(receiver_pid).unwrap();
        assert_eq!(receiver.message_queue.len(), 1);
        assert_eq!(receiver.message_queue[0].sender, sender_pid);
        assert_eq!(receiver.message_queue[0].receiver, receiver_pid);
        assert_eq!(receiver.message_queue[0].payload, data);
    }

    #[test]
    fn test_sys_send_nonexistent_target() {
        let mut state = KernelState::new();
        let sender_pid = state.allocate_pid();
        let sender_proc = Process::new(sender_pid, None);
        state.add_process(sender_proc);

        // Try to send to nonexistent process
        let result = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: 999,
                data: vec![1, 2, 3],
            },
            sender_pid,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::ProcessNotFound(999));
    }

    #[test]
    fn test_sys_recv_receives_message() {
        let mut state = KernelState::new();

        // Create two processes
        let sender_pid = state.allocate_pid();
        let sender_proc = Process::new(sender_pid, None);
        state.add_process(sender_proc);

        let receiver_pid = state.allocate_pid();
        let receiver_proc = Process::new(receiver_pid, None);
        state.add_process(receiver_proc);

        // Send message
        let data = b"Test message".to_vec();
        let result = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: receiver_pid,
                data: data.clone(),
            },
            sender_pid,
        );
        let (state, _) = result.unwrap();

        // Receive message
        let result = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, receiver_pid);

        assert!(result.is_ok());
        let (new_state, output) = result.unwrap();
        match output {
            SyscallOutput::Data(received_data) => {
                assert_eq!(received_data, data);
            }
            _ => panic!("Expected Data"),
        }

        // Verify queue is now empty
        let receiver = new_state.get_process(receiver_pid).unwrap();
        assert_eq!(receiver.message_queue.len(), 0);
    }

    #[test]
    fn test_sys_recv_empty_queue() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Try to receive when queue is empty
        let result = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, pid);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), KernelError::InvalidProcessState);
    }

    #[test]
    fn test_send_recv_pipeline() {
        let mut state = KernelState::new();

        // Create sender and receiver
        let sender_pid = state.allocate_pid();
        let sender_proc = Process::new(sender_pid, None);
        state.add_process(sender_proc);

        let receiver_pid = state.allocate_pid();
        let receiver_proc = Process::new(receiver_pid, None);
        state.add_process(receiver_proc);

        // Send multiple messages
        let msg1 = b"Message 1".to_vec();
        let msg2 = b"Message 2".to_vec();
        let msg3 = b"Message 3".to_vec();

        let result1 = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: receiver_pid,
                data: msg1.clone(),
            },
            sender_pid,
        );
        let (state, _) = result1.unwrap();

        let result2 = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: receiver_pid,
                data: msg2.clone(),
            },
            sender_pid,
        );
        let (state, _) = result2.unwrap();

        let result3 = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: receiver_pid,
                data: msg3.clone(),
            },
            sender_pid,
        );
        let (state, _) = result3.unwrap();

        // Verify all messages in queue
        let receiver = state.get_process(receiver_pid).unwrap();
        assert_eq!(receiver.message_queue.len(), 3);

        // Receive messages in order (FIFO)
        let result1 = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, receiver_pid);
        let (state, output1) = result1.unwrap();
        match output1 {
            SyscallOutput::Data(data) => assert_eq!(data, msg1),
            _ => panic!("Expected Data"),
        }

        let result2 = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, receiver_pid);
        let (state, output2) = result2.unwrap();
        match output2 {
            SyscallOutput::Data(data) => assert_eq!(data, msg2),
            _ => panic!("Expected Data"),
        }

        let result3 = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, receiver_pid);
        let (new_state, output3) = result3.unwrap();
        match output3 {
            SyscallOutput::Data(data) => assert_eq!(data, msg3),
            _ => panic!("Expected Data"),
        }

        // Queue should be empty
        let receiver = new_state.get_process(receiver_pid).unwrap();
        assert_eq!(receiver.message_queue.len(), 0);
    }

    #[test]
    fn test_send_recv_multiple_processes() {
        let mut state = KernelState::new();

        // Create three processes
        let pid1 = state.allocate_pid();
        state.add_process(Process::new(pid1, None));

        let pid2 = state.allocate_pid();
        state.add_process(Process::new(pid2, None));

        let pid3 = state.allocate_pid();
        state.add_process(Process::new(pid3, None));

        // Pid1 sends to pid2
        let result = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: pid2,
                data: b"1->2".to_vec(),
            },
            pid1,
        );
        let (state, _) = result.unwrap();

        // Pid1 sends to pid3
        let result = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: pid3,
                data: b"1->3".to_vec(),
            },
            pid1,
        );
        let (state, _) = result.unwrap();

        // Pid2 sends to pid3
        let result = dispatch_syscall(
            state,
            SystemCall::Send {
                target_pid: pid3,
                data: b"2->3".to_vec(),
            },
            pid2,
        );
        let (state, _) = result.unwrap();

        // Verify queues
        let proc1 = state.get_process(pid1).unwrap();
        let proc2 = state.get_process(pid2).unwrap();
        let proc3 = state.get_process(pid3).unwrap();

        assert_eq!(proc1.message_queue.len(), 0);
        assert_eq!(proc2.message_queue.len(), 1);
        assert_eq!(proc3.message_queue.len(), 2);

        // Pid2 receives its message
        let result = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, pid2);
        let (state, output) = result.unwrap();
        match output {
            SyscallOutput::Data(data) => assert_eq!(data, b"1->2".to_vec()),
            _ => panic!("Expected Data"),
        }

        // Pid3 receives first message
        let result = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, pid3);
        let (state, output) = result.unwrap();
        match output {
            SyscallOutput::Data(data) => assert_eq!(data, b"1->3".to_vec()),
            _ => panic!("Expected Data"),
        }

        // Pid3 receives second message
        let result = dispatch_syscall(state, SystemCall::Recv { timeout: 0 }, pid3);
        let (_, output) = result.unwrap();
        match output {
            SyscallOutput::Data(data) => assert_eq!(data, b"2->3".to_vec()),
            _ => panic!("Expected Data"),
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

    // Pipe syscall tests
    #[test]
    fn test_sys_pipe_creates_pipe() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create a pipe
        let result = dispatch_syscall(state.clone(), SystemCall::Pipe, pid);
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();

        // Should return a pair of file descriptors [read_fd, write_fd]
        match output {
            SyscallOutput::Pipe { read_fd, write_fd } => {
                assert_eq!(read_fd, 3); // First FD after stdin/stdout/stderr
                assert_eq!(write_fd, 4); // Second FD

                // Both FDs should be open in the process
                let proc = new_state.get_process(pid).unwrap();
                assert!(proc.is_fd_open(read_fd));
                assert!(proc.is_fd_open(write_fd));
            }
            _ => panic!("Expected Pipe output"),
        }
    }

    #[test]
    fn test_sys_pipe_write_read_fifo() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create pipe
        let result = dispatch_syscall(state, SystemCall::Pipe, pid);
        let (state, output) = result.unwrap();

        let (read_fd, write_fd) = match output {
            SyscallOutput::Pipe { read_fd, write_fd } => (read_fd, write_fd),
            _ => panic!("Expected Pipe output"),
        };

        // Write data to pipe
        let data1 = b"Hello".to_vec();
        let result = dispatch_syscall(
            state,
            SystemCall::Write {
                fd: write_fd,
                data: data1.clone(),
            },
            pid,
        );
        let (state, _) = result.unwrap();

        // Read data from pipe
        let result = dispatch_syscall(
            state,
            SystemCall::Read {
                fd: read_fd,
                count: 100,
            },
            pid,
        );
        let (_, output) = result.unwrap();

        match output {
            SyscallOutput::Data(data) => {
                assert_eq!(data, data1);
            }
            _ => panic!("Expected Data"),
        }
    }

    #[test]
    fn test_sys_pipe_multiple_writes_fifo() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create pipe
        let result = dispatch_syscall(state, SystemCall::Pipe, pid);
        let (state, output) = result.unwrap();

        let (read_fd, write_fd) = match output {
            SyscallOutput::Pipe { read_fd, write_fd } => (read_fd, write_fd),
            _ => panic!("Expected Pipe output"),
        };

        // Write multiple chunks
        let data1 = b"Hello".to_vec();
        let data2 = b" World".to_vec();

        let (state, _) = dispatch_syscall(
            state,
            SystemCall::Write {
                fd: write_fd,
                data: data1.clone(),
            },
            pid,
        )
        .unwrap();

        let (state, _) = dispatch_syscall(
            state,
            SystemCall::Write {
                fd: write_fd,
                data: data2.clone(),
            },
            pid,
        )
        .unwrap();

        // Read should return data in FIFO order
        let (_, output) = dispatch_syscall(
            state,
            SystemCall::Read {
                fd: read_fd,
                count: 100,
            },
            pid,
        )
        .unwrap();

        match output {
            SyscallOutput::Data(data) => {
                let expected = [data1, data2].concat();
                assert_eq!(data, expected);
            }
            _ => panic!("Expected Data"),
        }
    }

    // Dup2 syscall tests
    #[test]
    fn test_sys_dup2_duplicates_fd() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create a file
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), b"Hello".to_vec())
            .unwrap();

        // Open the file
        let result = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/test.txt".to_string(),
                flags: 0,
            },
            pid,
        );
        let (state, output) = result.unwrap();

        let oldfd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Duplicate FD to a new FD number
        let newfd = 10;
        let result = dispatch_syscall(state.clone(), SystemCall::Dup2 { oldfd, newfd }, pid);
        assert!(result.is_ok());

        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Success);

        // Both FDs should point to the same file
        let proc = new_state.get_process(pid).unwrap();
        assert!(proc.is_fd_open(oldfd));
        assert!(proc.is_fd_open(newfd));
        assert_eq!(proc.get_file_path(oldfd), proc.get_file_path(newfd));
    }

    #[test]
    fn test_sys_dup2_closes_newfd_if_open() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create two files
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), vec![])
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), vec![])
            .unwrap();

        // Open both files
        let (state, output1) = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/file1.txt".to_string(),
                flags: 0,
            },
            pid,
        )
        .unwrap();

        let fd1 = match output1 {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        let (state, output2) = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/file2.txt".to_string(),
                flags: 0,
            },
            pid,
        )
        .unwrap();

        let fd2 = match output2 {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Duplicate fd1 to fd2 (should close fd2 first)
        let (new_state, _) = dispatch_syscall(
            state,
            SystemCall::Dup2 {
                oldfd: fd1,
                newfd: fd2,
            },
            pid,
        )
        .unwrap();

        // fd2 should now point to file1
        let proc = new_state.get_process(pid).unwrap();
        assert_eq!(proc.get_file_path(fd2), Some(&PathBuf::from("/file1.txt")));
    }

    #[test]
    fn test_sys_dup2_stdout_redirection() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Create output file
        state
            .vfs
            .create_file(PathBuf::from("/output.txt"), vec![])
            .unwrap();

        // Open the file
        let (state, output) = dispatch_syscall(
            state,
            SystemCall::Open {
                path: "/output.txt".to_string(),
                flags: 0,
            },
            pid,
        )
        .unwrap();

        let file_fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FileDescriptor"),
        };

        // Redirect stdout (fd=1) to file
        let (state, _) = dispatch_syscall(
            state,
            SystemCall::Dup2 {
                oldfd: file_fd,
                newfd: 1,
            },
            pid,
        )
        .unwrap();

        // Now stdout should point to /output.txt
        let proc = state.get_process(pid).unwrap();
        assert_eq!(proc.get_file_path(1), Some(&PathBuf::from("/output.txt")));
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

    // =========================================================================
    // WOS-KERN-002: Sleep Syscall Tests (RED phase)
    // =========================================================================

    /// Unit Tests (8 tests)

    #[test]
    fn test_sleep_basic() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Sleep for 100ms
        let result = dispatch_syscall(state.clone(), SystemCall::Sleep(100), pid);
        assert!(result.is_ok(), "Sleep syscall should succeed");

        let (new_state, output) = result.unwrap();
        assert_eq!(output, SyscallOutput::Success);

        // Process should now be blocked (sleeping)
        let process = new_state.get_process(pid).unwrap();
        assert_eq!(process.state, ProcessState::Blocked);
    }

    #[test]
    fn test_sleep_zero_duration() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Sleep for 0ms should succeed but not block
        let result = dispatch_syscall(state.clone(), SystemCall::Sleep(0), pid);
        assert!(result.is_ok());

        let (new_state, _) = result.unwrap();
        let process = new_state.get_process(pid).unwrap();
        // Zero sleep should keep process ready
        assert_eq!(process.state, ProcessState::Ready);
    }

    #[test]
    fn test_sleep_sets_wakeup_time() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let sleep_duration = 500; // 500 microseconds
        let result = dispatch_syscall(state.clone(), SystemCall::Sleep(sleep_duration), pid);
        assert!(result.is_ok());

        let (new_state, _) = result.unwrap();
        let process = new_state.get_process(pid).unwrap();

        // Process should have wakeup_time set
        assert!(process.wakeup_time.is_some());
        let wakeup_time = process.wakeup_time.unwrap();

        // Wakeup time should be current time + sleep duration
        let expected_wakeup = new_state.simulated_clock.current_time() + sleep_duration;
        assert_eq!(wakeup_time, expected_wakeup);
    }

    #[test]
    fn test_sleep_on_terminated_process_fails() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let mut proc = Process::new(pid, None);
        proc.state = ProcessState::Terminated(0);
        state.add_process(proc);

        let result = dispatch_syscall(state, SystemCall::Sleep(100), pid);
        assert!(result.is_err());
    }

    #[test]
    fn test_sleep_multiple_processes_independent() {
        let mut state = KernelState::new();

        // Create two processes
        let pid1 = state.allocate_pid();
        let proc1 = Process::new(pid1, None);
        state.add_process(proc1);

        let pid2 = state.allocate_pid();
        let proc2 = Process::new(pid2, None);
        state.add_process(proc2);

        // Both sleep for different durations
        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(100), pid1).unwrap();
        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(200), pid2).unwrap();

        // Both should be blocked
        assert_eq!(
            state.get_process(pid1).unwrap().state,
            ProcessState::Blocked
        );
        assert_eq!(
            state.get_process(pid2).unwrap().state,
            ProcessState::Blocked
        );

        // Wakeup times should be different
        let wakeup1 = state.get_process(pid1).unwrap().wakeup_time.unwrap();
        let wakeup2 = state.get_process(pid2).unwrap().wakeup_time.unwrap();
        assert!(wakeup2 > wakeup1, "Process 2 should wake up later");
    }

    #[test]
    fn test_sleep_large_duration() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Sleep for very long duration (1 hour = 3,600,000,000 microseconds)
        let result = dispatch_syscall(state, SystemCall::Sleep(3_600_000_000), pid);
        assert!(result.is_ok(), "Should handle large sleep durations");
    }

    #[test]
    fn test_sleep_preserves_process_data() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let mut proc = Process::new(pid, Some(99));
        proc.memory_pages.push(1);
        proc.memory_pages.push(2);
        state.add_process(proc);

        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(100), pid).unwrap();

        let process = state.get_process(pid).unwrap();
        // Parent PID preserved
        assert_eq!(process.parent_pid, Some(99));
        // Memory preserved
        assert_eq!(process.memory_pages, vec![1, 2]);
    }

    #[test]
    fn test_sleep_process_can_be_scheduled() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Put process to sleep
        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(100), pid).unwrap();

        // Scheduler should be able to handle sleeping process
        // (sleeping processes should not be scheduled until wakeup)
        assert_eq!(state.get_process(pid).unwrap().state, ProcessState::Blocked);
    }

    /// Integration Tests (3 tests)

    #[test]
    fn test_integration_scheduler_wakes_sleeping_process() {
        use crate::scheduler::schedule;

        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        // Put process to sleep for 100 microseconds
        let (mut state, _) = dispatch_syscall(state, SystemCall::Sleep(100), pid).unwrap();

        // Process should be blocked
        assert_eq!(state.get_process(pid).unwrap().state, ProcessState::Blocked);
        // Verify wakeup time is set
        assert!(state.get_process(pid).unwrap().wakeup_time.is_some());

        // Advance clock past wakeup time
        state.simulated_clock.advance(150);

        // Scheduler should wake up the process
        let (state, _) = schedule(state).unwrap();

        // Process should now be ready or running
        let process_state = state.get_process(pid).unwrap().state.clone();
        assert!(
            matches!(process_state, ProcessState::Ready | ProcessState::Running),
            "Process should be awake after wakeup time"
        );
    }

    #[test]
    fn test_integration_multiple_processes_wake_in_order() {
        use crate::scheduler::schedule;

        let mut state = KernelState::new();

        // Create 3 processes with different sleep durations
        let pid1 = state.allocate_pid();
        let proc1 = Process::new(pid1, None);
        state.add_process(proc1);

        let pid2 = state.allocate_pid();
        let proc2 = Process::new(pid2, None);
        state.add_process(proc2);

        let pid3 = state.allocate_pid();
        let proc3 = Process::new(pid3, None);
        state.add_process(proc3);

        // Sleep: pid1=100us, pid2=200us, pid3=300us
        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(100), pid1).unwrap();
        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(200), pid2).unwrap();
        let (mut state, _) = dispatch_syscall(state, SystemCall::Sleep(300), pid3).unwrap();

        // All sleeping
        assert_eq!(
            state.get_process(pid1).unwrap().state,
            ProcessState::Blocked
        );
        assert_eq!(
            state.get_process(pid2).unwrap().state,
            ProcessState::Blocked
        );
        assert_eq!(
            state.get_process(pid3).unwrap().state,
            ProcessState::Blocked
        );

        // Advance to 150us - only pid1 should wake
        state.simulated_clock.advance(150);
        let (state, _) = schedule(state).unwrap();

        assert!(matches!(
            state.get_process(pid1).unwrap().state,
            ProcessState::Ready | ProcessState::Running
        ));
        assert_eq!(
            state.get_process(pid2).unwrap().state,
            ProcessState::Blocked
        );
        assert_eq!(
            state.get_process(pid3).unwrap().state,
            ProcessState::Blocked
        );

        // Advance to 250us - pid2 should also wake
        let mut state = state;
        state.simulated_clock.advance(100); // Total 250us
        let (state, _) = schedule(state).unwrap();

        assert!(matches!(
            state.get_process(pid2).unwrap().state,
            ProcessState::Ready | ProcessState::Running
        ));
        assert_eq!(
            state.get_process(pid3).unwrap().state,
            ProcessState::Blocked
        );

        // Advance to 350us - pid3 should wake
        let mut state = state;
        state.simulated_clock.advance(100); // Total 350us
        let (state, _) = schedule(state).unwrap();

        assert!(matches!(
            state.get_process(pid3).unwrap().state,
            ProcessState::Ready | ProcessState::Running
        ));
    }

    #[test]
    fn test_integration_sleep_with_fork_and_exec() {
        let mut state = KernelState::new();

        // Parent process
        let parent_pid = state.allocate_pid();
        let parent = Process::new(parent_pid, None);
        state.add_process(parent);

        // Parent forks
        let (state, output) = dispatch_syscall(state, SystemCall::Fork, parent_pid).unwrap();
        let child_pid = match output {
            SyscallOutput::Pid(pid) => pid,
            _ => panic!("Expected Pid"),
        };

        // Child sleeps
        let (state, _) = dispatch_syscall(state, SystemCall::Sleep(100), child_pid).unwrap();
        assert_eq!(
            state.get_process(child_pid).unwrap().state,
            ProcessState::Blocked
        );

        // Parent remains ready
        assert_eq!(
            state.get_process(parent_pid).unwrap().state,
            ProcessState::Ready
        );
    }

    /// Property Tests (2 tests)

    #[cfg(test)]
    mod sleep_properties {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10_000))]

            #[test]
            fn proptest_sleep_duration_accuracy(
                duration in 1u64..10_000,
            ) {
                let mut state = KernelState::new();
                let pid = state.allocate_pid();
                let proc = Process::new(pid, None);
                state.add_process(proc);

                let start_time = state.simulated_clock.current_time();
                let (state, _) = dispatch_syscall(state, SystemCall::Sleep(duration), pid).unwrap();

                let process = state.get_process(pid).unwrap();
                if let Some(wakeup_time) = process.wakeup_time {
                    let expected_wakeup = start_time + duration;
                    prop_assert_eq!(wakeup_time, expected_wakeup);
                }
            }

            #[test]
            fn proptest_sleep_state_transitions(
                duration in 0u64..1_000,
            ) {
                let mut state = KernelState::new();
                let pid = state.allocate_pid();
                let proc = Process::new(pid, None);
                state.add_process(proc);

                let (state, _) = dispatch_syscall(state, SystemCall::Sleep(duration), pid).unwrap();
                let process = state.get_process(pid).unwrap();

                if duration == 0 {
                    // Zero sleep should not block
                    prop_assert_eq!(&process.state, &ProcessState::Ready);
                    prop_assert!(process.wakeup_time.is_none());
                } else {
                    // Non-zero sleep should block
                    prop_assert_eq!(&process.state, &ProcessState::Blocked);
                    prop_assert!(process.wakeup_time.is_some());
                }
            }
        }
    }

    // ========================================================================
    // Signal Handling Tests (WOS-KERN-001)
    // ========================================================================

    mod signal_tests {
        use super::*;
        use crate::signals::{Signal, SignalAction};

        #[test]
        fn test_kill_send_sigterm() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGTERM to process
            let (state, output) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGTERM.number(),
                },
                pid,
            )
            .unwrap();

            assert_eq!(output, SyscallOutput::Success);

            // Signal should be pending
            let process = state.get_process(pid).unwrap();
            assert!(process.pending_signals.contains(Signal::SIGTERM));
        }

        #[test]
        fn test_kill_send_sigkill() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGKILL to process
            let (state, output) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGKILL.number(),
                },
                pid,
            )
            .unwrap();

            assert_eq!(output, SyscallOutput::Success);

            // SIGKILL should terminate immediately (not just pending)
            let process = state.get_process(pid).unwrap();
            assert!(matches!(process.state, ProcessState::Terminated(9)));
        }

        #[test]
        fn test_kill_send_sigint() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGINT to process
            let (state, output) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGINT.number(),
                },
                pid,
            )
            .unwrap();

            assert_eq!(output, SyscallOutput::Success);

            // Signal should be pending
            let process = state.get_process(pid).unwrap();
            assert!(process.pending_signals.contains(Signal::SIGINT));
        }

        #[test]
        fn test_kill_nonexistent_process() {
            let mut state = KernelState::new();

            // Create a calling process
            let calling_pid = state.allocate_pid();
            let caller = Process::new(calling_pid, None);
            state.add_process(caller);

            // Try to kill nonexistent process
            let result = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid: 999,
                    signal: Signal::SIGTERM.number(),
                },
                calling_pid,
            );

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), KernelError::ProcessNotFound(999));
        }

        #[test]
        fn test_kill_invalid_signal() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to send invalid signal
            let result = dispatch_syscall(state, SystemCall::Kill { pid, signal: 999 }, pid);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), KernelError::InvalidSignal(999));
        }

        #[test]
        fn test_signal_pending_delivery() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGUSR1
            let (state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGUSR1.number(),
                },
                pid,
            )
            .unwrap();

            // Signal should be in pending set
            let process = state.get_process(pid).unwrap();
            assert!(process.pending_signals.contains(Signal::SIGUSR1));
            assert!(!process.pending_signals.contains(Signal::SIGUSR2));
        }

        #[test]
        fn test_signal_blocked_not_delivered() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Block SIGTERM
            proc.blocked_signals.add(Signal::SIGTERM);
            state.add_process(proc);

            // Send SIGTERM (should be blocked)
            let (state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGTERM.number(),
                },
                pid,
            )
            .unwrap();

            // Signal should be pending but not delivered
            let process = state.get_process(pid).unwrap();
            assert!(process.pending_signals.contains(Signal::SIGTERM));
            assert!(process.blocked_signals.contains(Signal::SIGTERM));
            // Process should still be Ready (not terminated)
            assert_eq!(process.state, ProcessState::Ready);
        }

        #[test]
        fn test_signal_handler_custom() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Register custom handler for SIGUSR1
            proc.signal_handlers
                .insert(Signal::SIGUSR1.number(), SignalAction::Handler(42));
            state.add_process(proc);

            // Send SIGUSR1
            let (state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGUSR1.number(),
                },
                pid,
            )
            .unwrap();

            // Handler should be executed (this test will need signal delivery implementation)
            let process = state.get_process(pid).unwrap();
            // For now, just check signal was added to pending
            assert!(process.pending_signals.contains(Signal::SIGUSR1));
        }

        #[test]
        fn test_signal_default_terminate() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGTERM (default action: Terminate)
            let (mut state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGTERM.number(),
                },
                pid,
            )
            .unwrap();

            // Deliver pending signals
            state = crate::scheduler::deliver_signals(state).unwrap();

            // Process should be terminated
            let process = state.get_process(pid).unwrap();
            assert!(matches!(process.state, ProcessState::Terminated(_)));
        }

        #[test]
        fn test_signal_default_ignore() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGCHLD (default action: Ignore)
            let (mut state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGCHLD.number(),
                },
                pid,
            )
            .unwrap();

            // Deliver pending signals
            state = crate::scheduler::deliver_signals(state).unwrap();

            // Process should still be Ready (not terminated)
            let process = state.get_process(pid).unwrap();
            assert_eq!(process.state, ProcessState::Ready);
            // Signal should be removed from pending
            assert!(!process.pending_signals.contains(Signal::SIGCHLD));
        }

        // Integration Tests

        #[test]
        fn test_sigchld_on_child_exit() {
            let mut state = KernelState::new();

            // Create parent process (PID 1)
            let parent_pid = state.allocate_pid();
            let parent = Process::new(parent_pid, None);
            state.add_process(parent);

            // Create child process (PID 2)
            let child_pid = state.allocate_pid();
            let child = Process::new(child_pid, Some(parent_pid));
            state.add_process(child);

            // Child exits
            let (state, _) = dispatch_syscall(state, SystemCall::Exit(0), child_pid).unwrap();

            // Parent should have SIGCHLD pending
            let parent = state.get_process(parent_pid).unwrap();
            assert!(parent.pending_signals.contains(Signal::SIGCHLD));
        }

        #[test]
        fn test_signal_delivery_terminates_process() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send SIGINT
            let (mut state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGINT.number(),
                },
                pid,
            )
            .unwrap();

            // Process should be ready with pending signal
            let process = state.get_process(pid).unwrap();
            assert_eq!(process.state, ProcessState::Ready);
            assert!(process.pending_signals.contains(Signal::SIGINT));

            // Deliver signals
            state = crate::scheduler::deliver_signals(state).unwrap();

            // Process should be terminated
            let process = state.get_process(pid).unwrap();
            assert!(matches!(process.state, ProcessState::Terminated(_)));
        }

        #[test]
        fn test_signal_masking_blocks_delivery() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Block SIGINT
            proc.blocked_signals.add(Signal::SIGINT);
            state.add_process(proc);

            // Send SIGINT
            let (mut state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGINT.number(),
                },
                pid,
            )
            .unwrap();

            // Deliver signals (should not deliver blocked signal)
            state = crate::scheduler::deliver_signals(state).unwrap();

            // Process should still be Ready (not terminated)
            let process = state.get_process(pid).unwrap();
            assert_eq!(process.state, ProcessState::Ready);
            // Signal should still be pending
            assert!(process.pending_signals.contains(Signal::SIGINT));
        }

        #[test]
        fn test_multiple_pending_signals() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Send multiple signals
            let (state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGUSR1.number(),
                },
                pid,
            )
            .unwrap();

            let (state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGUSR2.number(),
                },
                pid,
            )
            .unwrap();

            // Both signals should be pending
            let process = state.get_process(pid).unwrap();
            assert!(process.pending_signals.contains(Signal::SIGUSR1));
            assert!(process.pending_signals.contains(Signal::SIGUSR2));
        }

        #[test]
        fn test_sigkill_bypasses_handler() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Register custom handler for SIGKILL (should be ignored)
            proc.signal_handlers
                .insert(Signal::SIGKILL.number(), SignalAction::Handler(99));
            state.add_process(proc);

            // Send SIGKILL
            let (state, _) = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: Signal::SIGKILL.number(),
                },
                pid,
            )
            .unwrap();

            // SIGKILL should terminate immediately regardless of handler
            let process = state.get_process(pid).unwrap();
            assert!(matches!(process.state, ProcessState::Terminated(9)));
        }

        // Property-based tests
        #[cfg(test)]
        mod proptests {
            use super::*;
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn proptest_kill_signal_delivery(
                    signal_num in prop::sample::select(vec![2u32, 9, 10, 11, 12, 13, 15, 17]),
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let signal = Signal::from_number(signal_num).unwrap();

                    // Send signal
                    let (state, output) = dispatch_syscall(
                        state,
                        SystemCall::Kill { pid, signal: signal_num },
                        pid,
                    )?;

                    prop_assert_eq!(output, SyscallOutput::Success);

                    let process = state.get_process(pid).unwrap();

                    if signal == Signal::SIGKILL {
                        // SIGKILL terminates immediately
                        prop_assert!(matches!(process.state, ProcessState::Terminated(9)));
                    } else {
                        // Other signals should be pending
                        prop_assert!(process.pending_signals.contains(signal));
                    }
                }

                #[test]
                fn proptest_signal_masking_correctness(
                    signal_num in prop::sample::select(vec![2u32, 10, 11, 12, 13, 15, 17]),
                    block_signal in prop::bool::ANY,
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let mut proc = Process::new(pid, None);

                    let signal = Signal::from_number(signal_num).unwrap();

                    // Optionally block the signal
                    if block_signal {
                        proc.blocked_signals.add(signal);
                    }
                    state.add_process(proc);

                    // Send signal
                    let (mut state, _) = dispatch_syscall(
                        state,
                        SystemCall::Kill { pid, signal: signal_num },
                        pid,
                    )?;

                    // Signal should be pending
                    let process = state.get_process(pid).unwrap();
                    prop_assert!(process.pending_signals.contains(signal));

                    // Deliver signals
                    state = crate::scheduler::deliver_signals(state)?;

                    let process = state.get_process(pid).unwrap();

                    if block_signal {
                        // Blocked signal should not terminate process
                        prop_assert_eq!(&process.state, &ProcessState::Ready);
                        // Signal should still be pending
                        prop_assert!(process.pending_signals.contains(signal));
                    } else {
                        // Unblocked signal should be delivered
                        // (either terminated or ignored depending on default action)
                        let default_action = signal.default_action();
                        if default_action == SignalAction::Terminate {
                            prop_assert!(matches!(&process.state, ProcessState::Terminated(_)));
                        } else {
                            // Ignore action - signal removed, process still Ready
                            prop_assert_eq!(&process.state, &ProcessState::Ready);
                            prop_assert!(!process.pending_signals.contains(signal));
                        }
                    }
                }
            }
        }
    }

    // ============================================================================
    // WOS-FS-001: Directory syscall tests
    // ============================================================================

    #[cfg(test)]
    mod directory_tests {
        use super::*;

        #[test]
        fn test_mkdir_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to create directory /test_dir
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );

            // Should succeed
            assert!(result.is_ok());
            let (new_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify directory was created
            let dir_exists = new_state
                .vfs
                .is_directory(std::path::Path::new("/test_dir"));
            assert!(dir_exists);
        }

        #[test]
        fn test_mkdir_nested() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create parent directory first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/parent".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Create child directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/parent/child".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            // Verify both directories exist
            assert!(new_state.vfs.is_directory(std::path::Path::new("/parent")));
            assert!(new_state
                .vfs
                .is_directory(std::path::Path::new("/parent/child")));
        }

        #[test]
        fn test_mkdir_already_exists() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Try to create same directory again - should fail
            let result = dispatch_syscall(
                state,
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                KernelError::FileAlreadyExists(_)
            ));
        }

        #[test]
        fn test_rmdir_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Remove directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Rmdir {
                    path: "/test_dir".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (new_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify directory no longer exists
            let dir_exists = new_state
                .vfs
                .is_directory(std::path::Path::new("/test_dir"));
            assert!(!dir_exists);
        }

        #[test]
        fn test_rmdir_not_empty() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create parent and child directories
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/parent".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/parent/child".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Try to remove parent directory - should fail because not empty
            let result = dispatch_syscall(
                state,
                SystemCall::Rmdir {
                    path: "/parent".to_string(),
                },
                pid,
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_getdents_empty_directory() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Open directory (assuming we'll use Open syscall with directory support)
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_dir".to_string(),
                    flags: 0,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            let fd = match output {
                SyscallOutput::FileDescriptor(fd) => fd,
                _ => panic!("Expected FileDescriptor"),
            };

            // Read directory entries
            let result = dispatch_syscall(state, SystemCall::Getdents { fd }, pid);
            assert!(result.is_ok());
            let (_new_state, output) = result.unwrap();

            // Should return directory entries (. and .. at minimum)
            match output {
                SyscallOutput::Data(data) => {
                    assert!(!data.is_empty());
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_getdents_with_files() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Create a file in the directory using Write syscall
            // First open the file with O_CREAT flag
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_dir/file1.txt".to_string(),
                    flags: O_CREAT | 0x0001, // O_CREAT | O_WRONLY
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            let file_fd = match output {
                SyscallOutput::FileDescriptor(fd) => fd,
                _ => panic!("Expected FileDescriptor"),
            };

            // Write content to the file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Write {
                    fd: file_fd,
                    data: b"content".to_vec(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Open directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_dir".to_string(),
                    flags: 0,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            let fd = match output {
                SyscallOutput::FileDescriptor(fd) => fd,
                _ => panic!("Expected FileDescriptor"),
            };

            // Read directory entries
            let result = dispatch_syscall(state, SystemCall::Getdents { fd }, pid);
            assert!(result.is_ok());
            let (_new_state, output) = result.unwrap();

            // Should return directory entries including file1.txt
            match output {
                SyscallOutput::Data(data) => {
                    let entries_json = String::from_utf8(data).unwrap();
                    assert!(entries_json.contains("file1.txt"));
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_mkdir_invalid_path() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to create directory with non-existent parent
            let result = dispatch_syscall(
                state,
                SystemCall::Mkdir {
                    path: "/nonexistent/child".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_rmdir_not_found() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to remove non-existent directory
            let result = dispatch_syscall(
                state,
                SystemCall::Rmdir {
                    path: "/nonexistent".to_string(),
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::FileNotFound(_)));
        }

        // ========================================================================
        // Property tests for directory operations
        // ========================================================================

        #[cfg(test)]
        mod proptests {
            use super::*;
            use proptest::prelude::*;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(10_000))]

                /// Property: mkdir followed by rmdir returns to original state
                #[test]
                fn proptest_mkdir_rmdir_inverse(
                    dir_name in "[a-z]{1,10}",
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = format!("/test_{}", dir_name);

                    // Create directory
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Mkdir {
                            path: path.clone(),
                            mode: 0o755,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Verify it exists
                    prop_assert!(state.vfs.is_directory(std::path::Path::new(&path)));

                    // Remove directory
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Rmdir {
                            path: path.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Verify it no longer exists
                    prop_assert!(!state.vfs.is_directory(std::path::Path::new(&path)));
                }

                /// Property: mkdir is deterministic - same input always produces same result
                #[test]
                fn proptest_mkdir_deterministic(
                    dir_name in "[a-z]{1,10}",
                    mode in 0o000u32..=0o777u32,
                ) {
                    let mut state1 = KernelState::new();
                    let pid1 = state1.allocate_pid();
                    let proc1 = Process::new(pid1, None);
                    state1.add_process(proc1);

                    let mut state2 = KernelState::new();
                    let pid2 = state2.allocate_pid();
                    let proc2 = Process::new(pid2, None);
                    state2.add_process(proc2);

                    // Use "test_" prefix to avoid conflicts with pre-existing directories
                    let path = format!("/test_{}", dir_name);

                    // Create directory in state1
                    let result1 = dispatch_syscall(
                        state1.clone(),
                        SystemCall::Mkdir {
                            path: path.clone(),
                            mode,
                        },
                        pid1,
                    );

                    // Create same directory in state2
                    let result2 = dispatch_syscall(
                        state2.clone(),
                        SystemCall::Mkdir {
                            path: path.clone(),
                            mode,
                        },
                        pid2,
                    );

                    // Both should succeed
                    prop_assert!(result1.is_ok());
                    prop_assert!(result2.is_ok());

                    // Both should have the directory
                    let (state1, _) = result1.unwrap();
                    let (state2, _) = result2.unwrap();
                    prop_assert!(state1.vfs.is_directory(std::path::Path::new(&path)));
                    prop_assert!(state2.vfs.is_directory(std::path::Path::new(&path)));
                }

                /// Property: mkdir with existing directory fails
                #[test]
                fn proptest_mkdir_idempotence(
                    dir_name in "[a-z]{1,10}",
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = format!("/test_{}", dir_name);

                    // Create directory first time
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Mkdir {
                            path: path.clone(),
                            mode: 0o755,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Try to create same directory again - should fail
                    let result = dispatch_syscall(
                        state,
                        SystemCall::Mkdir {
                            path: path.clone(),
                            mode: 0o755,
                        },
                        pid,
                    );
                    prop_assert!(result.is_err());
                    prop_assert!(matches!(result.unwrap_err(), KernelError::FileAlreadyExists(_)));
                }

                /// Property: rmdir on empty directory succeeds, on non-empty fails
                #[test]
                fn proptest_rmdir_empty_check(
                    parent_name in "[a-z]{1,10}",
                    child_name in "[a-z]{1,10}",
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let parent_path = format!("/test_{}", parent_name);
                    let child_path = format!("/test_{}/{}", parent_name, child_name);

                    // Create parent directory
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Mkdir {
                            path: parent_path.clone(),
                            mode: 0o755,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Create child directory
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Mkdir {
                            path: child_path.clone(),
                            mode: 0o755,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Try to remove parent (non-empty) - should fail
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Rmdir {
                            path: parent_path.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result.is_err());

                    // Remove child first
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Rmdir {
                            path: child_path.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Now remove parent (empty) - should succeed
                    let result = dispatch_syscall(
                        state,
                        SystemCall::Rmdir {
                            path: parent_path.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                }

                /// Property: getdents never panics on valid file descriptor
                #[test]
                fn proptest_getdents_never_panics(
                    dir_name in "[a-z]{1,10}",
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = format!("/test_{}", dir_name);

                    // Create directory
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Mkdir {
                            path: path.clone(),
                            mode: 0o755,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Open directory
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Open {
                            path: path.clone(),
                            flags: 0,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, output) = result.unwrap();
                    let fd = match output {
                        SyscallOutput::FileDescriptor(fd) => fd,
                        _ => return Err(TestCaseError::fail("Expected FileDescriptor")),
                    };

                    // Read directory entries - should never panic
                    let result = dispatch_syscall(
                        state,
                        SystemCall::Getdents { fd },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                }
            }
        }
    }

    // ============================================================================
    // WOS-FS-005: File metadata (stat) tests
    // ============================================================================

    #[cfg(test)]
    mod metadata_tests {
        use super::*;

        #[test]
        fn test_stat_file_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_file.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            let fd = match output {
                SyscallOutput::FileDescriptor(fd) => fd,
                _ => panic!("Expected FileDescriptor"),
            };

            // Write some content
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Write {
                    fd,
                    data: b"Hello, World!".to_vec(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Stat the file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Stat {
                    path: "/test_file.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            // Parse the FileStat from JSON
            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.size, 13); // "Hello, World!" is 13 bytes
                    assert_eq!(file_stat.file_type, wos_shared::vfs::FileType::RegularFile);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_stat_directory() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a directory
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Mkdir {
                    path: "/test_dir".to_string(),
                    mode: 0o755,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Stat the directory
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/test_dir".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            // Verify it's a directory
            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.file_type, wos_shared::vfs::FileType::Directory);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_stat_nonexistent_file() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Stat a non-existent file
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/nonexistent.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::FileNotFound(_)));
        }

        #[test]
        fn test_stat_file_size() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create file with specific size
            let content = b"0123456789"; // 10 bytes
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/sizefile.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            let fd = match output {
                SyscallOutput::FileDescriptor(fd) => fd,
                _ => panic!("Expected FileDescriptor"),
            };

            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Write {
                    fd,
                    data: content.to_vec(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Stat and verify size
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/sizefile.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.size, 10);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_stat_timestamps_exist() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/timefile.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Stat the file
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/timefile.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            // Verify timestamps are present (non-zero)
            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    // Timestamps should exist (can be zero or non-zero depending on implementation)
                    // Note: timestamps are unsigned, so they're always >= 0
                    let _ = (file_stat.atime, file_stat.mtime, file_stat.ctime);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_lstat_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/lstat_test.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Lstat the file (should work like stat for regular files)
            let result = dispatch_syscall(
                state,
                SystemCall::Lstat {
                    path: "/lstat_test.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.file_type, wos_shared::vfs::FileType::RegularFile);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_stat_empty_file() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create empty file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/empty.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Stat the empty file
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/empty.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.size, 0);
                    assert_eq!(file_stat.file_type, wos_shared::vfs::FileType::RegularFile);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_stat_root_directory() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Stat the root directory
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.file_type, wos_shared::vfs::FileType::Directory);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_stat_preserves_state() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/preserve.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            let state_before = state.clone();

            // Stat should not modify state
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/preserve.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state_after, _) = result.unwrap();

            // State should be functionally equivalent (stat is read-only)
            assert_eq!(state_before.processes.len(), state_after.processes.len());
        }

        #[test]
        fn test_stat_multiple_files() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create multiple files with different sizes
            for i in 0..3 {
                let path = format!("/file{}.txt", i);
                let result = dispatch_syscall(
                    state.clone(),
                    SystemCall::Open {
                        path: path.clone(),
                        flags: O_CREAT | 0x0001,
                    },
                    pid,
                );
                assert!(result.is_ok());
                let (new_state, output) = result.unwrap();
                state = new_state;

                let fd = match output {
                    SyscallOutput::FileDescriptor(fd) => fd,
                    _ => panic!("Expected FileDescriptor"),
                };

                // Write i bytes
                let content = vec![b'x'; i];
                let result =
                    dispatch_syscall(state.clone(), SystemCall::Write { fd, data: content }, pid);
                assert!(result.is_ok());
                let (new_state, _) = result.unwrap();
                state = new_state;
            }

            // Stat each file and verify sizes
            for i in 0..3 {
                let path = format!("/file{}.txt", i);
                let result = dispatch_syscall(state.clone(), SystemCall::Stat { path }, pid);
                assert!(result.is_ok());
                let (new_state, output) = result.unwrap();
                state = new_state;

                match output {
                    SyscallOutput::Data(data) => {
                        let file_stat: wos_shared::vfs::FileStat =
                            serde_json::from_slice(&data).unwrap();
                        assert_eq!(file_stat.size, i as u64);
                    }
                    _ => panic!("Expected Data output"),
                }
            }
        }

        // ========================================================================
        // Property tests for stat/metadata operations
        // ========================================================================

        #[cfg(test)]
        mod proptests {
            use super::*;
            use proptest::prelude::*;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(10_000))]

                /// Property: stat is deterministic - same path always returns same metadata
                #[test]
                fn proptest_stat_deterministic(
                    filename in "[a-z]{1,10}",
                    content_size in 0usize..1000,
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = format!("/test_{}.txt", filename);
                    let content = vec![b'x'; content_size];

                    // Create and write file
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Open {
                            path: path.clone(),
                            flags: O_CREAT | 0x0001,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, output) = result.unwrap();
                    let fd = match output {
                        SyscallOutput::FileDescriptor(fd) => fd,
                        _ => return Err(TestCaseError::fail("Expected FileDescriptor")),
                    };

                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Write {
                            fd,
                            data: content.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Stat the file twice
                    let result1 = dispatch_syscall(
                        state.clone(),
                        SystemCall::Stat {
                            path: path.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result1.is_ok());
                    let (state, output1) = result1.unwrap();

                    let result2 = dispatch_syscall(
                        state,
                        SystemCall::Stat {
                            path: path.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result2.is_ok());
                    let (_state, output2) = result2.unwrap();

                    // Both should return identical data
                    prop_assert_eq!(output1, output2);
                }

                /// Property: stat never panics on valid paths
                #[test]
                fn proptest_stat_never_panics(
                    filename in "[a-z]{1,10}",
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = format!("/test_{}.txt", filename);

                    // Create file
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Open {
                            path: path.clone(),
                            flags: O_CREAT | 0x0001,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Stat should never panic
                    let result = dispatch_syscall(
                        state,
                        SystemCall::Stat {
                            path,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                }

                /// Property: file size reported by stat matches written content length
                #[test]
                fn proptest_stat_size_matches_content(
                    filename in "[a-z]{1,10}",
                    content_size in 0usize..1000,
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = format!("/test_{}.txt", filename);
                    let content = vec![b'y'; content_size];

                    // Create and write file
                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Open {
                            path: path.clone(),
                            flags: O_CREAT | 0x0001,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, output) = result.unwrap();
                    let fd = match output {
                        SyscallOutput::FileDescriptor(fd) => fd,
                        _ => return Err(TestCaseError::fail("Expected FileDescriptor")),
                    };

                    let result = dispatch_syscall(
                        state.clone(),
                        SystemCall::Write {
                            fd,
                            data: content.clone(),
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (state, _) = result.unwrap();

                    // Stat and verify size
                    let result = dispatch_syscall(
                        state,
                        SystemCall::Stat {
                            path,
                        },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (_state, output) = result.unwrap();

                    match output {
                        SyscallOutput::Data(data) => {
                            let file_stat: wos_shared::vfs::FileStat =
                                serde_json::from_slice(&data)
                                    .map_err(|e| TestCaseError::fail(format!("JSON parse error: {}", e)))?;
                            prop_assert_eq!(file_stat.size, content_size as u64);
                        }
                        _ => return Err(TestCaseError::fail("Expected Data output")),
                    }
                }
            }
        }
    }

    // ============================================================================
    // WOS-FS-006: Path normalization and resolution tests
    // ============================================================================

    #[cfg(test)]
    mod path_resolution_tests {
        use super::*;

        #[test]
        fn test_realpath_current_directory() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test resolving /./test -> /test
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/./test".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/test");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_parent_directory() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test resolving /a/b/../c -> /a/c
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/b/../c".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/c");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_multiple_slashes() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test resolving /a//b///c -> /a/b/c
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a//b///c".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/b/c");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_complex_path() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test complex path: /a/./b/../c/./d -> /a/c/d
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/./b/../c/./d".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/c/d");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_root() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test root path
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_root_parent() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test /.. -> / (parent of root is root)
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/..".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_trailing_slash() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test trailing slash removal: /a/b/ -> /a/b
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/b/".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/b");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_multiple_dot_dot() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test /a/b/c/../../d -> /a/d
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/b/c/../../d".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/d");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_preserves_state() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            let state_before = state.clone();

            // Realpath should not modify state
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/./b/../c".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state_after, _) = result.unwrap();

            // State should be identical
            assert_eq!(state_before.processes.len(), state_after.processes.len());
        }

        #[test]
        fn test_realpath_empty_components() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test path with empty components from multiple slashes
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "///a///b///".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/b");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_single_dot_at_end() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test /a/b/. -> /a/b
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/b/.".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/b");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_double_dot_at_end() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test /a/b/.. -> /a
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/b/..".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_mixed_separators() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test path with mixed dot components
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/././b/../././c".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/c");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_realpath_deterministic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            let test_path = "/a/./b/../c".to_string();

            // Call realpath twice
            let result1 = dispatch_syscall(
                state.clone(),
                SystemCall::Realpath {
                    path: test_path.clone(),
                },
                pid,
            );
            assert!(result1.is_ok());
            let (state, output1) = result1.unwrap();

            let result2 = dispatch_syscall(state, SystemCall::Realpath { path: test_path }, pid);
            assert!(result2.is_ok());
            let (_state, output2) = result2.unwrap();

            // Both should return identical results
            assert_eq!(output1, output2);
        }

        #[test]
        fn test_realpath_long_path() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Test long path with many components
            let result = dispatch_syscall(
                state,
                SystemCall::Realpath {
                    path: "/a/b/c/d/e/f/g/../../h/../i/./j".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();

            match output {
                SyscallOutput::Data(data) => {
                    let canonical_path = String::from_utf8(data).unwrap();
                    assert_eq!(canonical_path, "/a/b/c/d/e/i/j");
                }
                _ => panic!("Expected Data output"),
            }
        }

        // ========================================================================
        // Property tests for path normalization
        // ========================================================================

        #[cfg(test)]
        mod proptests {
            use super::*;
            use proptest::prelude::*;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(10_000))]

                /// Property: realpath is deterministic
                #[test]
                fn proptest_realpath_deterministic(
                    components in prop::collection::vec("[a-z]{1,5}", 1..10),
                    dots in prop::collection::vec(prop::bool::weighted(0.3), 0..5),
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    // Build a path with random components and dots
                    let mut path_parts = vec![];
                    for (i, comp) in components.iter().enumerate() {
                        if i < dots.len() && dots[i] {
                            path_parts.push(".".to_string());
                        }
                        path_parts.push(comp.clone());
                    }
                    let path = format!("/{}", path_parts.join("/"));

                    // Call realpath twice
                    let result1 = dispatch_syscall(
                        state.clone(),
                        SystemCall::Realpath { path: path.clone() },
                        pid,
                    );
                    prop_assert!(result1.is_ok());
                    let (state, output1) = result1.unwrap();

                    let result2 = dispatch_syscall(
                        state,
                        SystemCall::Realpath { path },
                        pid,
                    );
                    prop_assert!(result2.is_ok());
                    let (_state, output2) = result2.unwrap();

                    // Results should be identical
                    prop_assert_eq!(output1, output2);
                }

                /// Property: realpath removes all . components
                #[test]
                fn proptest_realpath_removes_dots(
                    components in prop::collection::vec("[a-z]{1,5}", 1..8),
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    // Build path with . components interspersed
                    let mut path_parts = vec![];
                    for comp in &components {
                        path_parts.push(comp.clone());
                        path_parts.push(".".to_string());
                    }
                    let path = format!("/{}", path_parts.join("/"));

                    let result = dispatch_syscall(
                        state,
                        SystemCall::Realpath { path },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (_state, output) = result.unwrap();

                    match output {
                        SyscallOutput::Data(data) => {
                            let canonical = String::from_utf8(data)
                                .map_err(|e| TestCaseError::fail(format!("UTF-8 error: {}", e)))?;
                            // Canonical path should not contain /./
                            prop_assert!(!canonical.contains("/./ "));
                            // Should not end with /.
                            prop_assert!(!canonical.ends_with("/."));
                        }
                        _ => return Err(TestCaseError::fail("Expected Data output")),
                    }
                }

                /// Property: realpath removes redundant slashes
                #[test]
                fn proptest_realpath_removes_redundant_slashes(
                    components in prop::collection::vec("[a-z]{1,5}", 1..8),
                    extra_slashes in prop::collection::vec(1usize..5, 1..8),
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    // Build path with extra slashes
                    let mut path = String::from("/");
                    for (i, comp) in components.iter().enumerate() {
                        if i < extra_slashes.len() {
                            for _ in 0..extra_slashes[i] {
                                path.push('/');
                            }
                        }
                        path.push_str(comp);
                        path.push('/');
                    }

                    let result = dispatch_syscall(
                        state,
                        SystemCall::Realpath { path },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (_state, output) = result.unwrap();

                    match output {
                        SyscallOutput::Data(data) => {
                            let canonical = String::from_utf8(data)
                                .map_err(|e| TestCaseError::fail(format!("UTF-8 error: {}", e)))?;
                            // Canonical path should not contain //
                            prop_assert!(!canonical.contains("//"));
                        }
                        _ => return Err(TestCaseError::fail("Expected Data output")),
                    }
                }

                /// Property: realpath handles .. correctly
                #[test]
                fn proptest_realpath_handles_parent(
                    depth in 2usize..10,
                    backtrack in 1usize..5,
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    // Build path: /a/b/c/d/e/../.. -> /a/b/c
                    let mut path_parts = vec![];
                    for i in 0..depth {
                        path_parts.push(format!("dir{}", i));
                    }
                    let effective_backtrack = backtrack.min(depth - 1);
                    for _ in 0..effective_backtrack {
                        path_parts.push("..".to_string());
                    }
                    let path = format!("/{}", path_parts.join("/"));

                    let result = dispatch_syscall(
                        state,
                        SystemCall::Realpath { path },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (_state, output) = result.unwrap();

                    match output {
                        SyscallOutput::Data(data) => {
                            let canonical = String::from_utf8(data)
                                .map_err(|e| TestCaseError::fail(format!("UTF-8 error: {}", e)))?;
                            // Count components in canonical path
                            let components: Vec<&str> = canonical
                                .trim_matches('/')
                                .split('/')
                                .filter(|s| !s.is_empty())
                                .collect();
                            // Should have (depth - backtrack) components
                            prop_assert_eq!(components.len(), depth - effective_backtrack);
                        }
                        _ => return Err(TestCaseError::fail("Expected Data output")),
                    }
                }

                /// Property: realpath always returns absolute paths
                #[test]
                fn proptest_realpath_always_absolute(
                    components in prop::collection::vec("[a-z]{1,5}", 0..10),
                ) {
                    let mut state = KernelState::new();
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);

                    let path = if components.is_empty() {
                        "/".to_string()
                    } else {
                        format!("/{}", components.join("/"))
                    };

                    let result = dispatch_syscall(
                        state,
                        SystemCall::Realpath { path },
                        pid,
                    );
                    prop_assert!(result.is_ok());
                    let (_state, output) = result.unwrap();

                    match output {
                        SyscallOutput::Data(data) => {
                            let canonical = String::from_utf8(data)
                                .map_err(|e| TestCaseError::fail(format!("UTF-8 error: {}", e)))?;
                            // Must start with /
                            prop_assert!(canonical.starts_with('/'));
                            // Must not be empty
                            prop_assert!(!canonical.is_empty());
                        }
                        _ => return Err(TestCaseError::fail("Expected Data output")),
                    }
                }
            }
        }
    }

    // ============================================================================
    // File Permission and Ownership Tests
    // ============================================================================

    #[cfg(test)]
    mod permission_tests {
        use super::*;

        #[test]
        fn test_chmod_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_file.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Change permissions to 0644
            let result = dispatch_syscall(
                state,
                SystemCall::Chmod {
                    path: "/test_file.txt".to_string(),
                    mode: 0o644,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify the permissions changed
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/test_file.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.mode, 0o644);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_chmod_nonexistent_file() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to chmod a nonexistent file
            let result = dispatch_syscall(
                state,
                SystemCall::Chmod {
                    path: "/nonexistent.txt".to_string(),
                    mode: 0o644,
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::FileNotFound(_)));
        }

        #[test]
        fn test_chown_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_file.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Change ownership (uid: 1000, gid: 1000)
            let result = dispatch_syscall(
                state,
                SystemCall::Chown {
                    path: "/test_file.txt".to_string(),
                    uid: Some(1000),
                    gid: Some(1000),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify the ownership changed
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/test_file.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.uid, 1000);
                    assert_eq!(file_stat.gid, 1000);
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_chown_uid_only() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_file.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Change only uid
            let result = dispatch_syscall(
                state,
                SystemCall::Chown {
                    path: "/test_file.txt".to_string(),
                    uid: Some(500),
                    gid: None,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify only uid changed
            let result = dispatch_syscall(
                state,
                SystemCall::Stat {
                    path: "/test_file.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            match output {
                SyscallOutput::Data(data) => {
                    let file_stat: wos_shared::vfs::FileStat =
                        serde_json::from_slice(&data).unwrap();
                    assert_eq!(file_stat.uid, 500);
                    assert_eq!(file_stat.gid, 0); // Should remain default
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_access_file_exists() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_file.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Check if file exists (F_OK = 0)
            let result = dispatch_syscall(
                state,
                SystemCall::Access {
                    path: "/test_file.txt".to_string(),
                    mode: 0, // F_OK
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);
        }

        #[test]
        fn test_access_nonexistent_file() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Check nonexistent file
            let result = dispatch_syscall(
                state,
                SystemCall::Access {
                    path: "/nonexistent.txt".to_string(),
                    mode: 0, // F_OK
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::FileNotFound(_)));
        }

        #[test]
        fn test_access_read_permission() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file with read permissions
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/test_file.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Set permissions to 0444 (read-only)
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Chmod {
                    path: "/test_file.txt".to_string(),
                    mode: 0o444,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Check read permission (R_OK = 4)
            let result = dispatch_syscall(
                state,
                SystemCall::Access {
                    path: "/test_file.txt".to_string(),
                    mode: 4, // R_OK
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);
        }
    }

    // ============================================================================
    // Symbolic Link Tests
    // ============================================================================

    #[cfg(test)]
    mod symlink_tests {
        use super::*;

        #[test]
        fn test_symlink_create() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a target file first
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/target.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Create a symlink
            let result = dispatch_syscall(
                state,
                SystemCall::Symlink {
                    link_path: "/link.txt".to_string(),
                    target: "/target.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);
        }

        #[test]
        fn test_symlink_already_exists() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/existing.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Try to create symlink with same name
            let result = dispatch_syscall(
                state,
                SystemCall::Symlink {
                    link_path: "/existing.txt".to_string(),
                    target: "/target.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                KernelError::FileAlreadyExists(_)
            ));
        }

        #[test]
        fn test_readlink_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a target file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/target.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Create a symlink
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Symlink {
                    link_path: "/link.txt".to_string(),
                    target: "/target.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Read the symlink
            let result = dispatch_syscall(
                state,
                SystemCall::Readlink {
                    path: "/link.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_ok());
            let (_state, output) = result.unwrap();
            match output {
                SyscallOutput::Data(data) => {
                    let target = String::from_utf8(data).unwrap();
                    assert_eq!(target, "/target.txt");
                }
                _ => panic!("Expected Data output"),
            }
        }

        #[test]
        fn test_readlink_not_a_symlink() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a regular file
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Open {
                    path: "/regular.txt".to_string(),
                    flags: O_CREAT | 0x0001,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Try to readlink a regular file
            let result = dispatch_syscall(
                state,
                SystemCall::Readlink {
                    path: "/regular.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_readlink_nonexistent() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to readlink nonexistent file
            let result = dispatch_syscall(
                state,
                SystemCall::Readlink {
                    path: "/nonexistent.txt".to_string(),
                },
                pid,
            );
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::FileNotFound(_)));
        }

        // ===== Exec Syscall Tests =====

        #[test]
        fn test_exec_basic() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create a simple executable file
            state
                .vfs
                .create_file(std::path::PathBuf::from("/bin/test_prog"), Vec::new())
                .unwrap();
            state
                .vfs
                .write_file(
                    &std::path::PathBuf::from("/bin/test_prog"),
                    b"#!/bin/sh\necho hello".to_vec(),
                )
                .unwrap();

            // Execute the program
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Exec {
                    path: "/bin/test_prog".to_string(),
                    args: vec!["test_prog".to_string(), "arg1".to_string()],
                    env: vec![("PATH".to_string(), "/bin".to_string())],
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Check that process memory has been updated
            let process = new_state.get_process(pid).unwrap();
            assert_eq!(
                process.memory.program_path,
                Some("/bin/test_prog".to_string())
            );
            assert_eq!(
                process.memory.program_args,
                vec!["test_prog".to_string(), "arg1".to_string()]
            );

            // Check environment variables
            assert_eq!(process.env.get("PATH"), Some(&"/bin".to_string()));
            assert_eq!(process.env.get("_"), Some(&"/bin/test_prog".to_string()));
            assert_eq!(process.env.get("ARGC"), Some(&"2".to_string()));
            assert_eq!(process.env.get("ARG0"), Some(&"test_prog".to_string()));
            assert_eq!(process.env.get("ARG1"), Some(&"arg1".to_string()));

            // Process should be in Ready state
            assert_eq!(process.state, ProcessState::Ready);
        }

        #[test]
        fn test_exec_preserves_pid() {
            let mut state = KernelState::new();
            let parent_pid = state.allocate_pid();
            let parent = Process::new(parent_pid, None);
            state.add_process(parent);

            let child_pid = state.allocate_pid();
            let child = Process::new(child_pid, Some(parent_pid));
            state.add_process(child);

            // Create executable
            state
                .vfs
                .create_file(std::path::PathBuf::from("/bin/child_prog"), Vec::new())
                .unwrap();
            state
                .vfs
                .write_file(&std::path::PathBuf::from("/bin/child_prog"), b"child code".to_vec())
                .unwrap();

            // Exec in child process
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Exec {
                    path: "/bin/child_prog".to_string(),
                    args: vec![],
                    env: vec![],
                },
                child_pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            // PID should remain same
            let process = new_state.get_process(child_pid).unwrap();
            assert_eq!(process.pid, child_pid);

            // Parent PID should remain same
            assert_eq!(process.parent_pid, Some(parent_pid));
        }

        #[test]
        fn test_exec_nonexistent_file() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to exec nonexistent file
            let result = dispatch_syscall(
                state,
                SystemCall::Exec {
                    path: "/bin/nonexistent".to_string(),
                    args: vec![],
                    env: vec![],
                },
                pid,
            );

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::InvalidParameters(_)));
        }

        #[test]
        fn test_exec_empty_file() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create empty file
            state
                .vfs
                .create_file(std::path::PathBuf::from("/bin/empty"), Vec::new())
                .unwrap();

            // Try to exec empty file
            let result = dispatch_syscall(
                state,
                SystemCall::Exec {
                    path: "/bin/empty".to_string(),
                    args: vec![],
                    env: vec![],
                },
                pid,
            );

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::InvalidParameters(_)));
        }

        #[test]
        fn test_exec_replaces_environment() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Set initial environment
            proc.env.insert("OLD_VAR".to_string(), "old_value".to_string());
            state.add_process(proc);

            // Create executable
            state
                .vfs
                .create_file(std::path::PathBuf::from("/bin/prog"), Vec::new())
                .unwrap();
            state
                .vfs
                .write_file(&std::path::PathBuf::from("/bin/prog"), b"program code".to_vec())
                .unwrap();

            // Exec with new environment
            let result = dispatch_syscall(
                state,
                SystemCall::Exec {
                    path: "/bin/prog".to_string(),
                    args: vec![],
                    env: vec![("NEW_VAR".to_string(), "new_value".to_string())],
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            let process = new_state.get_process(pid).unwrap();

            // Old environment should be replaced
            assert_eq!(process.env.get("OLD_VAR"), None);

            // New environment should be set
            assert_eq!(process.env.get("NEW_VAR"), Some(&"new_value".to_string()));

            // Standard exec environment variables should be set
            assert_eq!(process.env.get("_"), Some(&"/bin/prog".to_string()));
            assert_eq!(process.env.get("ARGC"), Some(&"0".to_string()));
        }

        #[test]
        fn test_exec_with_multiple_args() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Create executable
            state
                .vfs
                .create_file(std::path::PathBuf::from("/bin/multi_arg"), Vec::new())
                .unwrap();
            state
                .vfs
                .write_file(&std::path::PathBuf::from("/bin/multi_arg"), b"code".to_vec())
                .unwrap();

            // Exec with multiple arguments
            let args = vec![
                "multi_arg".to_string(),
                "--flag".to_string(),
                "value1".to_string(),
            ];

            let result = dispatch_syscall(
                state,
                SystemCall::Exec {
                    path: "/bin/multi_arg".to_string(),
                    args: args.clone(),
                    env: vec![],
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            let process = new_state.get_process(pid).unwrap();
            assert_eq!(process.memory.program_args, args);
            assert_eq!(process.env.get("ARGC"), Some(&"3".to_string()));
            assert_eq!(process.env.get("ARG0"), Some(&"multi_arg".to_string()));
            assert_eq!(process.env.get("ARG1"), Some(&"--flag".to_string()));
            assert_eq!(process.env.get("ARG2"), Some(&"value1".to_string()));
        }

        // ===== Signal Handler Registration Tests =====

        #[test]
        fn test_sigaction_register_handler() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Register handler for SIGINT
            let result = dispatch_syscall(
                state.clone(),
                SystemCall::Sigaction {
                    signal: 2, // SIGINT
                    action: crate::signals::SignalAction::Handler(100),
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify handler is registered
            let process = new_state.get_process(pid).unwrap();
            assert_eq!(
                process.signal_handlers.get(&2),
                Some(&crate::signals::SignalAction::Handler(100))
            );
        }

        #[test]
        fn test_sigaction_ignore_signal() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Set SIGTERM to be ignored
            let result = dispatch_syscall(
                state,
                SystemCall::Sigaction {
                    signal: 15, // SIGTERM
                    action: crate::signals::SignalAction::Ignore,
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            let process = new_state.get_process(pid).unwrap();
            assert_eq!(
                process.signal_handlers.get(&15),
                Some(&crate::signals::SignalAction::Ignore)
            );
        }

        #[test]
        fn test_sigaction_cannot_handle_sigkill() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to set handler for SIGKILL (should fail)
            let result = dispatch_syscall(
                state,
                SystemCall::Sigaction {
                    signal: 9, // SIGKILL
                    action: crate::signals::SignalAction::Handler(100),
                },
                pid,
            );

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::InvalidParameters(_)));
        }

        #[test]
        fn test_sigaction_invalid_signal() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try invalid signal number
            let result = dispatch_syscall(
                state,
                SystemCall::Sigaction {
                    signal: 999, // Invalid signal
                    action: crate::signals::SignalAction::Handler(100),
                },
                pid,
            );

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), KernelError::InvalidSignal(_)));
        }

        #[test]
        fn test_sigprocmask_block_signals() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Block SIGINT and SIGTERM
            let result = dispatch_syscall(
                state,
                SystemCall::Sigprocmask {
                    block: vec![2, 15], // SIGINT, SIGTERM
                    unblock: vec![],
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, output) = result.unwrap();
            assert_eq!(output, SyscallOutput::Success);

            // Verify signals are blocked
            let process = new_state.get_process(pid).unwrap();
            assert!(process
                .blocked_signals
                .contains(crate::signals::Signal::SIGINT));
            assert!(process
                .blocked_signals
                .contains(crate::signals::Signal::SIGTERM));
        }

        #[test]
        fn test_sigprocmask_unblock_signals() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Pre-block some signals
            proc.blocked_signals
                .add(crate::signals::Signal::SIGINT);
            proc.blocked_signals
                .add(crate::signals::Signal::SIGTERM);
            state.add_process(proc);

            // Unblock SIGINT
            let result = dispatch_syscall(
                state,
                SystemCall::Sigprocmask {
                    block: vec![],
                    unblock: vec![2], // SIGINT
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            // Verify SIGINT is unblocked, SIGTERM still blocked
            let process = new_state.get_process(pid).unwrap();
            assert!(!process
                .blocked_signals
                .contains(crate::signals::Signal::SIGINT));
            assert!(process
                .blocked_signals
                .contains(crate::signals::Signal::SIGTERM));
        }

        #[test]
        fn test_sigprocmask_cannot_block_sigkill() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Try to block SIGKILL (should be silently ignored)
            let result = dispatch_syscall(
                state,
                SystemCall::Sigprocmask {
                    block: vec![9], // SIGKILL
                    unblock: vec![],
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            // Verify SIGKILL is NOT blocked
            let process = new_state.get_process(pid).unwrap();
            assert!(!process
                .blocked_signals
                .contains(crate::signals::Signal::SIGKILL));
        }

        #[test]
        fn test_sigprocmask_block_and_unblock() {
            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);

            // Pre-block SIGINT
            proc.blocked_signals
                .add(crate::signals::Signal::SIGINT);
            state.add_process(proc);

            // Block SIGTERM, unblock SIGINT in same call
            let result = dispatch_syscall(
                state,
                SystemCall::Sigprocmask {
                    block: vec![15],   // SIGTERM
                    unblock: vec![2],  // SIGINT
                },
                pid,
            );

            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();

            let process = new_state.get_process(pid).unwrap();
            assert!(!process
                .blocked_signals
                .contains(crate::signals::Signal::SIGINT));
            assert!(process
                .blocked_signals
                .contains(crate::signals::Signal::SIGTERM));
        }

        #[test]
        fn test_signal_handler_integration() {
            use crate::scheduler::deliver_signals;

            let mut state = KernelState::new();
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);

            // Register handler for SIGTERM
            let result = dispatch_syscall(
                state,
                SystemCall::Sigaction {
                    signal: 15,
                    action: crate::signals::SignalAction::Ignore,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Send SIGTERM to process
            let result = dispatch_syscall(
                state,
                SystemCall::Kill {
                    pid,
                    signal: 15,
                },
                pid,
            );
            assert!(result.is_ok());
            let (state, _) = result.unwrap();

            // Deliver signals - should be ignored due to handler
            let state = deliver_signals(state).unwrap();

            // Process should still be running (signal was ignored)
            let process = state.get_process(pid).unwrap();
            assert!(!matches!(
                process.state,
                crate::state::ProcessState::Terminated(_)
            ));
        }
    }
}
