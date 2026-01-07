//! Kernel State Types
//!
//! Core state types for the WOS microkernel with persistent data structures.

use crate::memory::VirtualMemory;
use im::HashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wos_shared::{AprModel, AprOutput, VirtualFileSystem};

/// APR execution state (serializable for kernel state persistence)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AprExecutionState {
    /// The APR model being executed
    pub model: AprModel,
    /// Current input index
    pub input_index: usize,
    /// Current tick
    pub current_tick: u64,
    /// Whether the runtime is active
    pub active: bool,
    /// Outputs collected during execution
    pub outputs: Vec<AprOutput>,
    /// Steps executed so far
    pub steps_executed: u64,
}

impl AprExecutionState {
    /// Create from an APR model
    pub fn new(model: AprModel) -> Self {
        Self {
            model,
            input_index: 0,
            current_tick: 0,
            active: false,
            outputs: Vec::new(),
            steps_executed: 0,
        }
    }

    /// Start execution
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Stop execution
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if execution is finished
    pub fn is_finished(&self) -> bool {
        self.input_index >= self.model.inputs.len()
    }

    /// Execute one step
    pub fn step(&mut self) -> Option<AprOutput> {
        if !self.active || self.is_finished() {
            return None;
        }

        if self.input_index < self.model.inputs.len() {
            let timestamped_input = &self.model.inputs[self.input_index];
            self.input_index += 1;
            self.current_tick = timestamped_input.tick;
            self.steps_executed += 1;

            // Create output based on input type
            let output = match &timestamped_input.input {
                wos_shared::AprInput::Syscall(ref syscall) => AprOutput {
                    tick: self.current_tick,
                    output_type: "syscall".to_string(),
                    data: syscall.clone(),
                },
                wos_shared::AprInput::Command(ref cmd) => AprOutput {
                    tick: self.current_tick,
                    output_type: "command".to_string(),
                    data: serde_json::Value::String(cmd.clone()),
                },
                wos_shared::AprInput::KeyPress(key) => AprOutput {
                    tick: self.current_tick,
                    output_type: "keypress".to_string(),
                    data: serde_json::Value::String(key.to_string()),
                },
                wos_shared::AprInput::Timer(ms) => AprOutput {
                    tick: self.current_tick,
                    output_type: "timer".to_string(),
                    data: serde_json::Value::Number(serde_json::Number::from(*ms)),
                },
                wos_shared::AprInput::Signal(ref sig) => AprOutput {
                    tick: self.current_tick,
                    output_type: "signal".to_string(),
                    data: serde_json::to_value(sig).unwrap_or_default(),
                },
                wos_shared::AprInput::ExternalEvent(ref event) => AprOutput {
                    tick: self.current_tick,
                    output_type: "external_event".to_string(),
                    data: serde_json::Value::String(event.clone()),
                },
                wos_shared::AprInput::Custom { ref name, ref data } => AprOutput {
                    tick: self.current_tick,
                    output_type: format!("custom:{}", name),
                    data: data.clone(),
                },
            };

            self.outputs.push(output.clone());
            Some(output)
        } else {
            None
        }
    }
}

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
    /// Wakeup time in microseconds (for sleeping processes)
    pub wakeup_time: Option<u64>,
    /// Pending signals waiting to be delivered
    pub pending_signals: crate::signals::SignalSet,
    /// Blocked signals that won't be delivered
    pub blocked_signals: crate::signals::SignalSet,
    /// Signal handlers (signal number -> action)
    pub signal_handlers: im::HashMap<u32, crate::signals::SignalAction>,
    /// Process priority (0 = highest, 7 = lowest, 4 = normal)
    pub priority: u8,
    /// Wait ticks for aging (prevents starvation)
    pub wait_ticks: u64,
    /// Environment variables
    pub env: im::HashMap<String, String>,
    /// APR execution state (for deterministic replay)
    pub apr_state: Option<AprExecutionState>,
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
            wakeup_time: None,
            pending_signals: crate::signals::SignalSet::new(),
            blocked_signals: crate::signals::SignalSet::new(),
            signal_handlers: im::HashMap::new(),
            priority: 4, // Normal priority by default
            wait_ticks: 0,
            env: im::HashMap::new(),
            apr_state: None,
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

    /// Duplicate file descriptor (for dup2)
    pub fn dup_fd(&mut self, oldfd: FileDescriptor, newfd: FileDescriptor) -> Result<(), String> {
        // Check if oldfd exists
        let path = self
            .open_files
            .get(&oldfd)
            .ok_or_else(|| format!("Invalid oldfd: {}", oldfd))?
            .clone();

        // Close newfd if it's open (except standard streams which can be overwritten)
        if self.open_files.contains_key(&newfd) && newfd >= 3 {
            self.open_files.remove(&newfd);
        }

        // Duplicate oldfd to newfd
        self.open_files.insert(newfd, path);
        Ok(())
    }
}

/// Pipe buffer for inter-process communication
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipeBuffer {
    /// Pipe read file descriptor
    pub read_fd: FileDescriptor,
    /// Pipe write file descriptor
    pub write_fd: FileDescriptor,
    /// Process ID that owns this pipe
    pub owner_pid: ProcessId,
    /// Buffer data
    pub data: Vec<u8>,
}

/// Complete kernel state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelState {
    /// Process table (persistent HashMap for O(1) cloning)
    pub processes: HashMap<ProcessId, Process>,
    /// Next available PID
    pub next_pid: ProcessId,
    /// Currently running process
    pub current_pid: Option<ProcessId>,
    /// Virtual file system
    pub vfs: VirtualFileSystem,
    /// Pipe buffers (keyed by read FD)
    pub pipes: HashMap<FileDescriptor, PipeBuffer>,
    /// Simulated clock for time tracking
    pub simulated_clock: wos_shared::SimulatedClock,
    /// VM Manager for MicroVM operations
    #[serde(skip, default)]
    pub vm_manager: crate::vmm::VmManager,
}

impl PartialEq for KernelState {
    fn eq(&self, other: &Self) -> bool {
        // Compare all serializable fields, skip vm_manager
        self.processes == other.processes
            && self.next_pid == other.next_pid
            && self.current_pid == other.current_pid
            && self.vfs == other.vfs
            && self.pipes == other.pipes
            && self.simulated_clock == other.simulated_clock
    }
}

impl KernelState {
    /// Create new kernel state
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_pid: 1, // PID 0 reserved for kernel
            current_pid: None,
            vfs: VirtualFileSystem::new(),
            pipes: HashMap::new(),
            simulated_clock: wos_shared::SimulatedClock::new(),
            vm_manager: crate::vmm::VmManager::new(),
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

    /// Compute a hash of the kernel state for APR checkpoints
    ///
    /// Uses a simple hash of the serialized state for deterministic verification.
    pub fn compute_hash(&self) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Serialize state to JSON for consistent hashing
        let json = serde_json::to_string(self).unwrap_or_default();

        // Use a simple hash (for educational purposes - production would use Blake3)
        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let hash = hasher.finish();

        // Expand to 32 bytes by repeating the hash
        let mut result = [0u8; 32];
        for (i, chunk) in result.chunks_mut(8).enumerate() {
            let rotated_hash = hash.rotate_left((i * 16) as u32);
            chunk.copy_from_slice(&rotated_hash.to_le_bytes());
        }
        result
    }
}

impl Default for KernelState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Jidoka Integration
// ============================================================================

impl crate::jidoka::KernelStateView for KernelState {
    fn process_count(&self) -> usize {
        self.processes.len()
    }

    fn memory_used(&self) -> usize {
        // Sum memory pages across all processes
        // Each page is 4KB (4096 bytes)
        self.processes
            .values()
            .map(|p| p.memory_pages.len() * 4096)
            .sum()
    }

    fn total_fd_count(&self) -> usize {
        self.processes.values().map(|p| p.open_files.len()).sum()
    }

    fn has_orphan_processes(&self) -> bool {
        for process in self.processes.values() {
            if let Some(parent_pid) = process.parent_pid {
                // Check if parent exists (init process with pid 1 is an exception)
                if parent_pid != 0 && !self.processes.contains_key(&parent_pid) {
                    return true;
                }
            }
            // Processes without parent_pid are init processes (allowed)
        }
        false
    }

    fn zombie_count(&self) -> usize {
        self.processes
            .values()
            .filter(|p| matches!(p.state, ProcessState::Terminated(_)))
            .count()
    }

    fn vm_count(&self) -> usize {
        self.vm_manager.count()
    }

    fn vm_memory_within_bounds(&self) -> bool {
        // All VMs report their memory within bounds via their status
        self.vm_manager
            .list()
            .iter()
            .all(|status| status.memory_used <= status.memory_max)
    }

    fn rng_state_valid(&self) -> bool {
        // SimulatedClock is always valid in our implementation
        true
    }

    fn all_pids_unique(&self) -> bool {
        // HashMap keys are inherently unique, so check if any PIDs mismatch
        // their key (which would indicate corruption)
        self.processes
            .iter()
            .all(|(key, process)| *key == process.pid)
    }

    fn has_memory_overlaps(&self) -> bool {
        // Check each process's memory for overlaps
        // In our simplified model, we track page numbers which are unique per process
        // Overlaps would only happen if the same page is allocated twice within a process
        for process in self.processes.values() {
            let mut seen_pages = std::collections::HashSet::new();
            for &page in &process.memory_pages {
                if !seen_pages.insert(page) {
                    return true; // Duplicate page detected
                }
            }
        }
        false
    }

    fn memory_totals_consistent(&self) -> bool {
        // Verify that memory page counts are reasonable
        // Each process shouldn't have negative or impossibly large memory
        const MAX_PAGES_PER_PROCESS: usize = 65536; // 256MB per process
        self.processes
            .values()
            .all(|p| p.memory_pages.len() <= MAX_PAGES_PER_PROCESS)
    }

    fn all_process_states_valid(&self) -> bool {
        // Verify process state consistency
        for process in self.processes.values() {
            // Running process must be the current process
            if matches!(process.state, ProcessState::Running)
                && self.current_pid != Some(process.pid)
            {
                return false;
            }
        }

        // If there's a current process, it must be Running
        if let Some(current) = self.current_pid {
            if let Some(process) = self.processes.get(&current) {
                if !matches!(process.state, ProcessState::Running) {
                    return false;
                }
            } else {
                return false; // Current PID doesn't exist
            }
        }

        true
    }

    fn scheduler_consistent(&self) -> bool {
        // Verify scheduler invariants:
        // 1. At most one process should be Running
        let running_count = self
            .processes
            .values()
            .filter(|p| matches!(p.state, ProcessState::Running))
            .count();

        if running_count > 1 {
            return false;
        }

        // 2. If current_pid is set, exactly one process should be Running
        if self.current_pid.is_some() && running_count != 1 {
            return false;
        }

        // 3. If no current_pid, no process should be Running
        if self.current_pid.is_none() && running_count != 0 {
            return false;
        }

        true
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

    // ========================================================================
    // Jidoka Integration Tests
    // ========================================================================
    mod jidoka_tests {
        use super::*;
        use crate::jidoka::{JidokaGuard, KernelStateView, MAX_PROCESSES};

        #[test]
        fn test_kernel_state_view_empty() {
            let state = KernelState::new();
            assert_eq!(state.process_count(), 0);
            assert_eq!(state.memory_used(), 0);
            assert_eq!(state.total_fd_count(), 0);
            assert!(!state.has_orphan_processes());
            assert_eq!(state.zombie_count(), 0);
            assert_eq!(state.vm_count(), 0);
            assert!(state.vm_memory_within_bounds());
            assert!(state.rng_state_valid());
            assert!(state.all_pids_unique());
            assert!(!state.has_memory_overlaps());
            assert!(state.memory_totals_consistent());
            assert!(state.all_process_states_valid());
            assert!(state.scheduler_consistent());
        }

        #[test]
        fn test_kernel_state_view_with_processes() {
            let mut state = KernelState::new();

            // Add a process
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);
            proc.memory_pages = vec![1, 2, 3]; // 3 pages = 12KB
            state.add_process(proc);

            assert_eq!(state.process_count(), 1);
            assert_eq!(state.memory_used(), 3 * 4096);
            assert_eq!(state.total_fd_count(), 3); // stdin, stdout, stderr
        }

        #[test]
        fn test_kernel_state_view_zombie_count() {
            let mut state = KernelState::new();

            // Add terminated process (zombie)
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);
            proc.state = ProcessState::Terminated(0);
            state.add_process(proc);

            assert_eq!(state.zombie_count(), 1);
        }

        #[test]
        fn test_kernel_state_view_orphan_detection() {
            let mut state = KernelState::new();

            // Add process with non-existent parent
            let pid = state.allocate_pid();
            let proc = Process::new(pid, Some(9999)); // Parent doesn't exist
            state.add_process(proc);

            assert!(state.has_orphan_processes());
        }

        #[test]
        fn test_kernel_state_view_memory_overlaps() {
            let mut state = KernelState::new();

            // Add process with duplicate pages
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);
            proc.memory_pages = vec![1, 2, 1]; // Duplicate page 1
            state.add_process(proc);

            assert!(state.has_memory_overlaps());
        }

        #[test]
        fn test_kernel_state_view_scheduler_consistency() {
            let mut state = KernelState::new();

            // Add running process without setting current_pid
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);
            proc.state = ProcessState::Running;
            state.add_process(proc);

            // Inconsistent: running process but no current_pid
            assert!(!state.scheduler_consistent());

            // Fix by setting current_pid
            state.current_pid = Some(pid);
            assert!(state.scheduler_consistent());
        }

        #[test]
        fn test_kernel_state_view_process_states_valid() {
            let mut state = KernelState::new();

            // Add process and set it as running
            let pid = state.allocate_pid();
            let mut proc = Process::new(pid, None);
            proc.state = ProcessState::Running;
            state.add_process(proc);
            state.current_pid = Some(pid);

            assert!(state.all_process_states_valid());

            // Now add another running process (invalid)
            let pid2 = state.allocate_pid();
            let mut proc2 = Process::new(pid2, None);
            proc2.state = ProcessState::Running;
            state.add_process(proc2);

            // Two running processes is invalid
            assert!(!state.scheduler_consistent());
        }

        #[test]
        fn test_jidoka_guard_with_kernel_state() {
            let mut guard = JidokaGuard::new();
            let state = KernelState::new();

            let status = guard.check(&state);
            assert!(status.is_ok());
        }

        #[test]
        fn test_jidoka_guard_process_limit() {
            let mut guard = JidokaGuard::new();
            let mut state = KernelState::new();

            // Add too many processes
            for _ in 0..=MAX_PROCESSES {
                let pid = state.allocate_pid();
                let proc = Process::new(pid, None);
                state.add_process(proc);
            }

            let status = guard.check(&state);
            assert!(status.is_halt());
        }

        #[test]
        fn test_jidoka_guard_detects_orphans() {
            let mut guard = JidokaGuard::new();
            let mut state = KernelState::new();

            // Add orphan process
            let pid = state.allocate_pid();
            let proc = Process::new(pid, Some(9999));
            state.add_process(proc);

            let status = guard.check(&state);
            assert!(status.is_halt());
        }

        #[test]
        fn test_jidoka_guard_with_init_state() {
            let mut guard = JidokaGuard::new();
            let mut state = KernelState::with_init();

            // with_init creates processes without proper state management
            // Set the current process to running for consistency
            if let Some(current) = state.current_pid {
                if let Some(proc) = state.get_process_mut(current) {
                    proc.state = ProcessState::Running;
                }
            }

            let status = guard.check(&state);
            // Should be ok now with proper state
            assert!(
                status.is_ok(),
                "Jidoka check failed: {:?}",
                status.violations()
            );
        }
    }
}
