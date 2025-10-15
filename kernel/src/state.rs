//! Kernel State Types
//!
//! Core state types for the WOS microkernel with persistent data structures.

use crate::memory::VirtualMemory;
use im::HashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wos_shared::VirtualFileSystem;

/// Process identifier
pub type ProcessId = u32;

/// File descriptor
pub type FileDescriptor = u32;

/// Standard input file descriptor
pub const STDIN_FD: FileDescriptor = 0;

/// Standard output file descriptor
pub const STDOUT_FD: FileDescriptor = 1;

/// Standard error file descriptor
pub const STDERR_FD: FileDescriptor = 2;

/// Process execution state
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    /// Ready to run
    Ready,
    /// Currently executing
    Running,
    /// Blocked on I/O or IPC
    Blocked,
    /// Terminated (exit code)
    Terminated(i32),
}

/// Inter-process message
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Sender process ID
    pub sender: ProcessId,
    /// Receiver process ID
    pub receiver: ProcessId,
    /// Message payload
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(sender: ProcessId, receiver: ProcessId, payload: Vec<u8>) -> Self {
        Self {
            sender,
            receiver,
            payload,
        }
    }
}

/// Process control block
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Process {
    /// Process ID
    pub pid: ProcessId,
    /// Current execution state
    pub state: ProcessState,
    /// Parent process ID (None for init)
    pub parent_pid: Option<ProcessId>,
    /// Memory pages allocated (simulated)
    pub memory_pages: Vec<u32>,
    /// Open file descriptors
    pub open_files: HashMap<FileDescriptor, PathBuf>,
    /// Virtual memory
    pub memory: VirtualMemory,
    /// Message queue (FIFO)
    pub message_queue: im::Vector<Message>,
}

impl Process {
    /// Create a new process
    pub fn new(pid: ProcessId, parent_pid: Option<ProcessId>) -> Self {
        let mut open_files = HashMap::new();
        // Set up standard streams (stdin, stdout, stderr)
        open_files.insert(STDIN_FD, PathBuf::from("/dev/stdin"));
        open_files.insert(STDOUT_FD, PathBuf::from("/dev/stdout"));
        open_files.insert(STDERR_FD, PathBuf::from("/dev/stderr"));

        Self {
            pid,
            state: ProcessState::Ready,
            parent_pid,
            memory_pages: Vec::new(),
            open_files,
            memory: VirtualMemory::new(),
            message_queue: im::Vector::new(),
        }
    }

    /// Check if process is runnable
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, ProcessState::Ready | ProcessState::Running)
    }

    /// Check if process has terminated
    pub fn is_terminated(&self) -> bool {
        matches!(self.state, ProcessState::Terminated(_))
    }

    /// Allocate next available file descriptor
    pub fn allocate_fd(&self) -> FileDescriptor {
        // Find first available FD starting from 3 (after stdin, stdout, stderr)
        let mut fd = 3;
        while self.open_files.contains_key(&fd) {
            fd += 1;
        }
        fd
    }

    /// Open a file (add file descriptor)
    pub fn open_file(&mut self, path: PathBuf) -> FileDescriptor {
        let fd = self.allocate_fd();
        self.open_files.insert(fd, path);
        fd
    }

    /// Close a file (remove file descriptor)
    pub fn close_file(&mut self, fd: FileDescriptor) -> Option<PathBuf> {
        // Don't allow closing standard streams
        if fd == STDIN_FD || fd == STDOUT_FD || fd == STDERR_FD {
            return None;
        }
        self.open_files.remove(&fd)
    }

    /// Get file path for file descriptor
    pub fn get_file_path(&self, fd: FileDescriptor) -> Option<&PathBuf> {
        self.open_files.get(&fd)
    }

    /// Check if file descriptor is open
    pub fn is_fd_open(&self, fd: FileDescriptor) -> bool {
        self.open_files.contains_key(&fd)
    }
}

/// Complete kernel state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KernelState {
    /// Process table (persistent HashMap for O(1) cloning)
    pub processes: HashMap<ProcessId, Process>,
    /// Next available PID
    pub next_pid: ProcessId,
    /// Currently running process
    pub current_pid: Option<ProcessId>,
    /// Virtual file system
    pub vfs: VirtualFileSystem,
}

impl KernelState {
    /// Create new kernel state
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_pid: 1, // PID 0 reserved for kernel
            current_pid: None,
            vfs: VirtualFileSystem::new(),
        }
    }

    /// Create kernel state with init and shell processes
    pub fn with_init() -> Self {
        let mut state = Self::new();

        // Create init process (PID 1)
        let init = Process::new(1, None);
        state.add_process(init);
        state.next_pid = 2;

        // Create shell process (PID 2, child of init)
        let shell = Process::new(2, Some(1));
        state.add_process(shell);
        state.next_pid = 3;

        // Set current process to shell
        state.current_pid = Some(2);

        state
    }

    /// Allocate a new process ID
    pub fn allocate_pid(&mut self) -> ProcessId {
        let pid = self.next_pid;
        self.next_pid += 1;
        pid
    }

    /// Add a process to the process table
    pub fn add_process(&mut self, process: Process) {
        self.processes.insert(process.pid, process);
    }

    /// Get a process by PID
    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.get(&pid)
    }

    /// Get a mutable reference to a process
    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    /// Remove a process from the process table
    pub fn remove_process(&mut self, pid: ProcessId) -> Option<Process> {
        self.processes.remove(&pid)
    }

    /// Count of active processes
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
}

impl Default for KernelState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_state_creation() {
        let state = KernelState::new();
        assert_eq!(state.next_pid, 1);
        assert_eq!(state.process_count(), 0);
        assert_eq!(state.current_pid, None);
    }

    #[test]
    fn test_kernel_state_with_init() {
        let state = KernelState::with_init();

        // Should have 2 processes: init (PID 1) and shell (PID 2)
        assert_eq!(state.process_count(), 2);
        assert_eq!(state.next_pid, 3);
        assert_eq!(state.current_pid, Some(2));

        // Verify init process (PID 1)
        let init = state.get_process(1).expect("init process should exist");
        assert_eq!(init.pid, 1);
        assert_eq!(init.parent_pid, None);
        assert!(init.is_runnable());

        // Verify shell process (PID 2)
        let shell = state.get_process(2).expect("shell process should exist");
        assert_eq!(shell.pid, 2);
        assert_eq!(shell.parent_pid, Some(1));
        assert!(shell.is_runnable());
    }

    #[test]
    fn test_process_creation() {
        let proc = Process::new(1, None);
        assert_eq!(proc.pid, 1);
        assert_eq!(proc.state, ProcessState::Ready);
        assert_eq!(proc.parent_pid, None);
        assert!(proc.is_runnable());
        assert!(!proc.is_terminated());
    }

    #[test]
    fn test_process_state_transitions() {
        let mut proc = Process::new(1, None);

        // Ready -> Running
        proc.state = ProcessState::Running;
        assert!(proc.is_runnable());
        assert!(!proc.is_terminated());

        // Running -> Blocked
        proc.state = ProcessState::Blocked;
        assert!(!proc.is_runnable());
        assert!(!proc.is_terminated());

        // Blocked -> Ready
        proc.state = ProcessState::Ready;
        assert!(proc.is_runnable());
        assert!(!proc.is_terminated());

        // Running -> Terminated
        proc.state = ProcessState::Terminated(0);
        assert!(!proc.is_runnable());
        assert!(proc.is_terminated());
    }

    #[test]
    fn test_process_serialization_roundtrip() {
        let mut proc = Process::new(42, Some(1));
        proc.memory_pages = vec![1, 2, 3];
        proc.open_files.insert(0, PathBuf::from("/test.txt"));

        // Serialize
        let json = serde_json::to_string(&proc).expect("serialization failed");

        // Deserialize
        let proc2: Process = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(proc, proc2);
    }

    #[test]
    fn test_kernel_state_add_process() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);

        state.add_process(proc.clone());
        assert_eq!(state.process_count(), 1);
        assert_eq!(state.get_process(pid), Some(&proc));
    }

    #[test]
    fn test_kernel_state_remove_process() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);

        state.add_process(proc.clone());
        assert_eq!(state.process_count(), 1);

        let removed = state.remove_process(pid);
        assert_eq!(removed, Some(proc));
        assert_eq!(state.process_count(), 0);
    }

    #[test]
    fn test_kernel_state_clone_cheap() {
        let mut state = KernelState::new();

        // Add 100 processes
        for _ in 0..100 {
            let pid = state.allocate_pid();
            let proc = Process::new(pid, None);
            state.add_process(proc);
        }

        // Clone should be O(1) with im-rs
        let state2 = state.clone();
        assert_eq!(state.process_count(), state2.process_count());
        assert_eq!(state, state2);
    }

    // Property-based tests
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: Cloning kernel state preserves all data
            #[test]
            fn proptest_kernel_state_cloning_cheap(
                num_processes in 0..1000u32,
            ) {
                let mut state = KernelState::new();

                // Add processes
                for _ in 0..num_processes {
                    let pid = state.allocate_pid();
                    let proc = Process::new(pid, None);
                    state.add_process(proc);
                }

                // Clone should preserve all data
                let state2 = state.clone();
                prop_assert_eq!(state.process_count(), state2.process_count());
                prop_assert_eq!(state, state2);
            }

            /// Property: PID allocation is always unique and monotonic
            #[test]
            fn proptest_pid_allocation_unique(
                num_pids in 1..10000usize,
            ) {
                let mut state = KernelState::new();
                let mut pids = Vec::new();

                for _ in 0..num_pids {
                    let pid = state.allocate_pid();
                    pids.push(pid);
                }

                // All PIDs should be unique
                let unique_count = pids.iter().collect::<std::collections::HashSet<_>>().len();
                prop_assert_eq!(unique_count, num_pids);

                // PIDs should be monotonically increasing
                for i in 1..pids.len() {
                    prop_assert!(pids[i] > pids[i - 1]);
                }
            }

            /// Property: Serialization roundtrip preserves data
            #[test]
            fn proptest_process_serialization(
                pid in 1..10000u32,
                parent_pid in proptest::option::of(1..10000u32),
                exit_code in -128..128i32,
            ) {
                let mut proc = Process::new(pid, parent_pid);
                proc.state = ProcessState::Terminated(exit_code);
                proc.memory_pages = vec![1, 2, 3];

                // Serialize and deserialize
                let json = serde_json::to_string(&proc).unwrap();
                let proc2: Process = serde_json::from_str(&json).unwrap();

                prop_assert_eq!(proc, proc2);
            }

            /// Property: Process state predicates are consistent
            #[test]
            fn proptest_process_state_predicates(
                pid in 1..10000u32,
                state_choice in 0..4usize,
            ) {
                let mut proc = Process::new(pid, None);

                // Set state based on choice
                proc.state = match state_choice {
                    0 => ProcessState::Ready,
                    1 => ProcessState::Running,
                    2 => ProcessState::Blocked,
                    _ => ProcessState::Terminated(0),
                };

                // Predicates should be consistent
                match proc.state {
                    ProcessState::Ready | ProcessState::Running => {
                        prop_assert!(proc.is_runnable());
                        prop_assert!(!proc.is_terminated());
                    }
                    ProcessState::Blocked => {
                        prop_assert!(!proc.is_runnable());
                        prop_assert!(!proc.is_terminated());
                    }
                    ProcessState::Terminated(_) => {
                        prop_assert!(!proc.is_runnable());
                        prop_assert!(proc.is_terminated());
                    }
                }
            }

            /// Property: Adding and removing processes maintains consistency
            #[test]
            fn proptest_kernel_state_operations(
                operations in prop::collection::vec(0..3usize, 0..100),
            ) {
                let mut state = KernelState::new();
                let mut added_pids = Vec::new();

                for op in operations {
                    match op {
                        0 => {
                            // Add process
                            let pid = state.allocate_pid();
                            let proc = Process::new(pid, None);
                            state.add_process(proc);
                            added_pids.push(pid);
                        }
                        1 => {
                            // Remove process if any exist
                            if let Some(&pid) = added_pids.first() {
                                state.remove_process(pid);
                                added_pids.retain(|&p| p != pid);
                            }
                        }
                        _ => {
                            // Query process
                            if let Some(&pid) = added_pids.first() {
                                prop_assert!(state.get_process(pid).is_some());
                            }
                        }
                    }
                }

                // Final consistency check
                prop_assert_eq!(state.process_count(), added_pids.len());
            }
        }
    }

    #[test]
    fn test_kernel_state_default() {
        let state1 = KernelState::default();
        let state2 = KernelState::new();
        assert_eq!(state1.process_count(), state2.process_count());
        assert_eq!(state1.process_count(), 0);
    }
}
