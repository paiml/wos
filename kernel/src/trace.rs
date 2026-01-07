//! Tracing and Time-Travel Debugging
//!
//! Records all system calls and kernel state transitions for debugging and replay.

use crate::state::{KernelState, ProcessId};
use crate::syscall::{SyscallOutput, SystemCall};
use im::Vector;
use serde::{Deserialize, Serialize};

/// System call trace entry
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemCallTrace {
    /// Sequential trace ID
    pub trace_id: usize,
    /// Process that made the syscall
    pub calling_pid: ProcessId,
    /// The system call
    pub syscall: SystemCall,
    /// System call result
    pub result: Result<SyscallOutput, String>, // String instead of KernelError for serialization
    /// Timestamp (simulated microseconds)
    pub timestamp_us: u64,
}

/// Kernel history with state snapshots for time-travel debugging
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KernelHistory {
    /// State snapshots (one per syscall)
    snapshots: Vector<KernelState>,
    /// System call traces
    traces: Vector<SystemCallTrace>,
    /// Current position in history (0 = initial state)
    current_position: usize,
}

impl KernelHistory {
    /// Create new history with initial state
    pub fn new(initial_state: KernelState) -> Self {
        Self {
            snapshots: Vector::unit(initial_state),
            traces: Vector::new(),
            current_position: 0,
        }
    }

    /// Record a system call and new state
    pub fn record_syscall(
        &mut self,
        calling_pid: ProcessId,
        syscall: SystemCall,
        result: Result<SyscallOutput, String>,
        new_state: KernelState,
        timestamp_us: u64,
    ) {
        // Create trace entry
        let trace = SystemCallTrace {
            trace_id: self.traces.len(),
            calling_pid,
            syscall,
            result,
            timestamp_us,
        };

        // Add trace and snapshot
        self.traces.push_back(trace);
        self.snapshots.push_back(new_state);

        // Move to latest position
        self.current_position = self.snapshots.len() - 1;
    }

    /// Get current kernel state
    pub fn current_state(&self) -> &KernelState {
        &self.snapshots[self.current_position]
    }

    /// Get current position in history
    pub fn position(&self) -> usize {
        self.current_position
    }

    /// Get total number of states (including initial)
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if history is empty (only initial state)
    pub fn is_empty(&self) -> bool {
        self.snapshots.len() == 1
    }

    /// Get number of traces
    pub fn trace_count(&self) -> usize {
        self.traces.len()
    }

    /// Step back one state
    pub fn step_back(&mut self) -> bool {
        if self.current_position > 0 {
            self.current_position -= 1;
            true
        } else {
            false
        }
    }

    /// Step forward one state
    pub fn step_forward(&mut self) -> bool {
        if self.current_position < self.snapshots.len() - 1 {
            self.current_position += 1;
            true
        } else {
            false
        }
    }

    /// Jump to specific position
    pub fn jump_to(&mut self, position: usize) -> bool {
        if position < self.snapshots.len() {
            self.current_position = position;
            true
        } else {
            false
        }
    }

    /// Get trace at specific index
    pub fn get_trace(&self, index: usize) -> Option<&SystemCallTrace> {
        self.traces.get(index)
    }

    /// Get all traces
    pub fn traces(&self) -> &Vector<SystemCallTrace> {
        &self.traces
    }

    /// Export traces to JSON
    pub fn export_traces_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.traces)
    }

    /// Export full history (traces + snapshots) to JSON
    pub fn export_full_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct HistoryExport<'a> {
            traces: &'a Vector<SystemCallTrace>,
            snapshot_count: usize,
            current_position: usize,
        }

        let export = HistoryExport {
            traces: &self.traces,
            snapshot_count: self.snapshots.len(),
            current_position: self.current_position,
        };

        serde_json::to_string_pretty(&export)
    }
}

impl Default for KernelHistory {
    fn default() -> Self {
        Self::new(KernelState::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Process;

    #[test]
    fn test_kernel_history_creation() {
        let state = KernelState::new();
        let history = KernelHistory::new(state.clone());

        assert_eq!(history.len(), 1);
        assert_eq!(history.position(), 0);
        assert_eq!(history.trace_count(), 0);
        assert!(history.is_empty());
        assert_eq!(history.current_state(), &state);
    }

    #[test]
    fn test_syscall_tracing() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let mut history = KernelHistory::new(state.clone());

        // Record a syscall
        history.record_syscall(
            pid,
            SystemCall::GetPid,
            Ok(SyscallOutput::Pid(pid)),
            state.clone(),
            1000,
        );

        assert_eq!(history.trace_count(), 1);
        assert_eq!(history.len(), 2); // initial + 1 syscall

        let trace = history.get_trace(0).unwrap();
        assert_eq!(trace.trace_id, 0);
        assert_eq!(trace.calling_pid, pid);
        assert_eq!(trace.syscall, SystemCall::GetPid);
        assert_eq!(trace.timestamp_us, 1000);
    }

    #[test]
    fn test_time_travel_step_back() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let mut history = KernelHistory::new(state.clone());

        // Record multiple syscalls
        for i in 0..5 {
            let mut new_state = state.clone();
            new_state.allocate_pid(); // Change state
            history.record_syscall(
                pid,
                SystemCall::GetPid,
                Ok(SyscallOutput::Pid(pid)),
                new_state.clone(),
                (i + 1) * 1000,
            );
            state = new_state;
        }

        assert_eq!(history.position(), 5); // At latest

        // Step back
        assert!(history.step_back());
        assert_eq!(history.position(), 4);

        assert!(history.step_back());
        assert_eq!(history.position(), 3);

        // Step back to beginning
        assert!(history.step_back());
        assert!(history.step_back());
        assert!(history.step_back());
        assert_eq!(history.position(), 0);

        // Can't step back further
        assert!(!history.step_back());
        assert_eq!(history.position(), 0);
    }

    #[test]
    fn test_time_travel_step_forward() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let mut history = KernelHistory::new(state.clone());

        // Record syscalls
        for i in 0..3 {
            let mut new_state = state.clone();
            new_state.allocate_pid();
            history.record_syscall(
                pid,
                SystemCall::GetPid,
                Ok(SyscallOutput::Pid(pid)),
                new_state.clone(),
                (i + 1) * 1000,
            );
            state = new_state;
        }

        // Go back to start
        history.jump_to(0);
        assert_eq!(history.position(), 0);

        // Step forward
        assert!(history.step_forward());
        assert_eq!(history.position(), 1);

        assert!(history.step_forward());
        assert_eq!(history.position(), 2);

        assert!(history.step_forward());
        assert_eq!(history.position(), 3);

        // Can't step forward further
        assert!(!history.step_forward());
        assert_eq!(history.position(), 3);
    }

    #[test]
    fn test_jump_to_position() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let mut history = KernelHistory::new(state.clone());

        // Record syscalls
        for i in 0..10 {
            let mut new_state = state.clone();
            new_state.allocate_pid();
            history.record_syscall(
                pid,
                SystemCall::GetPid,
                Ok(SyscallOutput::Pid(pid)),
                new_state.clone(),
                (i + 1) * 1000,
            );
            state = new_state;
        }

        // Jump to middle
        assert!(history.jump_to(5));
        assert_eq!(history.position(), 5);

        // Jump to start
        assert!(history.jump_to(0));
        assert_eq!(history.position(), 0);

        // Jump to end
        assert!(history.jump_to(10));
        assert_eq!(history.position(), 10);

        // Can't jump beyond
        assert!(!history.jump_to(11));
        assert_eq!(history.position(), 10);
    }

    #[test]
    fn test_trace_export() {
        let mut state = KernelState::new();
        let pid = state.allocate_pid();
        let proc = Process::new(pid, None);
        state.add_process(proc);

        let mut history = KernelHistory::new(state.clone());

        // Record syscalls
        history.record_syscall(
            pid,
            SystemCall::GetPid,
            Ok(SyscallOutput::Pid(pid)),
            state.clone(),
            1000,
        );

        history.record_syscall(
            pid,
            SystemCall::Fork,
            Ok(SyscallOutput::Pid(2)),
            state.clone(),
            2000,
        );

        // Export traces
        let json = history.export_traces_json().unwrap();
        assert!(json.contains("GetPid"));
        assert!(json.contains("Fork"));

        // Export full history
        let full_json = history.export_full_json().unwrap();
        assert!(full_json.contains("traces"));
        assert!(full_json.contains("snapshot_count"));
    }

    #[test]
    fn test_trace_serialization() {
        let trace = SystemCallTrace {
            trace_id: 0,
            calling_pid: 1,
            syscall: SystemCall::GetPid,
            result: Ok(SyscallOutput::Pid(1)),
            timestamp_us: 1000,
        };

        let json = serde_json::to_string(&trace).unwrap();
        let trace2: SystemCallTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace, trace2);
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: History position is always in bounds
            #[test]
            fn proptest_history_position_in_bounds(
                num_syscalls in 0..100usize,
            ) {
                let state = KernelState::new();
                let mut history = KernelHistory::new(state.clone());

                // Record syscalls
                for i in 0..num_syscalls {
                    history.record_syscall(
                        1,
                        SystemCall::GetPid,
                        Ok(SyscallOutput::Pid(1)),
                        state.clone(),
                        (i as u64 + 1) * 1000,
                    );
                }

                // Position should always be valid
                prop_assert!(history.position() < history.len());
            }

            /// Property: Step back/forward never exceeds bounds
            #[test]
            fn proptest_time_travel_bounds(
                num_syscalls in 1..100usize,
                num_backs in 0..200usize,
                num_forwards in 0..200usize,
            ) {
                let state = KernelState::new();
                let mut history = KernelHistory::new(state.clone());

                // Record syscalls
                for i in 0..num_syscalls {
                    history.record_syscall(
                        1,
                        SystemCall::GetPid,
                        Ok(SyscallOutput::Pid(1)),
                        state.clone(),
                        (i as u64 + 1) * 1000,
                    );
                }

                // Step back many times
                for _ in 0..num_backs {
                    history.step_back();
                    prop_assert!(history.position() < history.len());
                }

                // Step forward many times
                for _ in 0..num_forwards {
                    history.step_forward();
                    prop_assert!(history.position() < history.len());
                }
            }

            /// Property: Trace IDs are sequential
            #[test]
            fn proptest_trace_ids_sequential(
                num_syscalls in 1..100usize,
            ) {
                let state = KernelState::new();
                let mut history = KernelHistory::new(state.clone());

                // Record syscalls
                for i in 0..num_syscalls {
                    history.record_syscall(
                        1,
                        SystemCall::GetPid,
                        Ok(SyscallOutput::Pid(1)),
                        state.clone(),
                        (i as u64 + 1) * 1000,
                    );
                }

                // Verify trace IDs
                for i in 0..num_syscalls {
                    let trace = history.get_trace(i).unwrap();
                    prop_assert_eq!(trace.trace_id, i);
                }
            }

            /// Property: History export is valid JSON
            #[test]
            fn proptest_export_valid_json(
                num_syscalls in 0..50usize,
            ) {
                let state = KernelState::new();
                let mut history = KernelHistory::new(state.clone());

                // Record syscalls
                for i in 0..num_syscalls {
                    history.record_syscall(
                        1,
                        SystemCall::GetPid,
                        Ok(SyscallOutput::Pid(1)),
                        state.clone(),
                        (i as u64 + 1) * 1000,
                    );
                }

                // Export should always succeed
                let json = history.export_traces_json();
                prop_assert!(json.is_ok());

                let full_json = history.export_full_json();
                prop_assert!(full_json.is_ok());
            }
        }
    }
}
