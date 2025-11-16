//! Process Scheduler
//!
//! Round-robin scheduler with fairness guarantees and no starvation.

use crate::state::{KernelState, ProcessId};
use im::Vector;
use serde::{Deserialize, Serialize};

/// Round-robin process scheduler
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scheduler {
    /// Ready queue (processes ready to run)
    ready_queue: Vector<ProcessId>,
    /// Current position in round-robin (for fairness tracking)
    current_index: usize,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            ready_queue: Vector::new(),
            current_index: 0,
        }
    }

    /// Add a process to the ready queue
    pub fn enqueue(&mut self, pid: ProcessId) {
        // Only add if not already in queue
        if !self.ready_queue.contains(&pid) {
            self.ready_queue.push_back(pid);
        }
    }

    /// Remove a process from the ready queue
    pub fn dequeue(&mut self, pid: ProcessId) {
        // Find and remove the process
        if let Some(idx) = self.ready_queue.iter().position(|&p| p == pid) {
            self.ready_queue.remove(idx);
            // Adjust current_index if needed
            if self.current_index >= self.ready_queue.len() {
                self.current_index = 0;
            }
        }
    }

    /// Select next process to run using round-robin
    ///
    /// Returns None if no processes are ready.
    /// Guarantees fairness: each process gets equal CPU time.
    pub fn schedule(&mut self) -> Option<ProcessId> {
        if self.ready_queue.is_empty() {
            return None;
        }

        // Get current process
        let pid = self.ready_queue[self.current_index];

        // Move to next process (round-robin)
        self.current_index = (self.current_index + 1) % self.ready_queue.len();

        Some(pid)
    }

    /// Get the number of processes in the ready queue
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }

    /// Check if a process is in the ready queue
    pub fn is_ready(&self, pid: ProcessId) -> bool {
        self.ready_queue.contains(&pid)
    }

    /// Rebuild ready queue from kernel state
    ///
    /// Scans all processes and rebuilds the ready queue based on process states.
    /// This ensures consistency between scheduler and kernel state.
    pub fn sync_with_state(&mut self, state: &KernelState) {
        // Clear current queue
        self.ready_queue = Vector::new();
        self.current_index = 0;

        // Add all runnable processes
        for (pid, process) in state.processes.iter() {
            if process.is_runnable() {
                self.ready_queue.push_back(*pid);
            }
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Standalone schedule function that wakes up sleeping processes
///
/// Checks all blocked processes and wakes those whose wakeup time has passed.
/// Returns the updated kernel state and optionally the next process to run.
pub fn schedule(
    mut state: KernelState,
) -> Result<(KernelState, Option<ProcessId>), crate::syscall::KernelError> {
    let current_time = state.simulated_clock.current_time();

    // Wake up any sleeping processes whose wakeup time has passed
    let mut processes_to_wake = Vec::new();

    for (pid, process) in state.processes.iter() {
        if matches!(process.state, crate::state::ProcessState::Blocked) {
            if let Some(wakeup_time) = process.wakeup_time {
                if current_time >= wakeup_time {
                    processes_to_wake.push(*pid);
                }
            }
        }
    }

    // Wake up the processes
    for pid in processes_to_wake {
        if let Some(mut process) = state.processes.get(&pid).cloned() {
            process.state = crate::state::ProcessState::Ready;
            process.wakeup_time = None;
            state.processes.insert(pid, process);
        }
    }

    // Return state and no specific process (scheduler can choose)
    Ok((state, None))
}

/// Deliver pending signals to processes
///
/// Processes pending signals for all processes and takes appropriate actions
/// based on signal handlers and default actions. Blocked signals are not delivered.
pub fn deliver_signals(mut state: KernelState) -> Result<KernelState, crate::syscall::KernelError> {
    let mut processes_to_update = Vec::new();

    // Collect pending signals for all processes
    for (pid, process) in state.processes.iter() {
        if !process.pending_signals.is_empty()
            && !matches!(process.state, crate::state::ProcessState::Terminated(_))
        {
            processes_to_update.push(*pid);
        }
    }

    // Process signals for each process
    for pid in processes_to_update {
        if let Some(mut process) = state.processes.get(&pid).cloned() {
            // Get next pending signal (lowest number first)
            while let Some(signal) = process.pending_signals.next_signal() {
                // Check if signal is blocked
                if process.blocked_signals.contains(signal) {
                    // Blocked signal - skip delivery but keep pending
                    break;
                }

                // Remove signal from pending
                process.pending_signals.remove(signal);

                // Check for custom handler
                let action = process
                    .signal_handlers
                    .get(&signal.number())
                    .copied()
                    .unwrap_or_else(|| signal.default_action());

                // Execute action
                match action {
                    crate::signals::SignalAction::Terminate => {
                        process.state =
                            crate::state::ProcessState::Terminated(signal.number() as i32);
                        break; // Process terminated, no more signal processing
                    }
                    crate::signals::SignalAction::Ignore => {
                        // Signal ignored, just remove from pending (already done above)
                    }
                    crate::signals::SignalAction::Handler(_handler_id) => {
                        // Custom handler execution would go here
                        // For now, we just remove the signal from pending
                        // In a real implementation, this would invoke user-space handler
                    }
                }
            }

            // Update process in state
            state.processes.insert(pid, process);
        }
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Process, ProcessState};

    #[test]
    fn test_scheduler_creation() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.ready_count(), 0);
        assert_eq!(scheduler.schedule(), None);
    }

    #[test]
    fn test_scheduler_enqueue() {
        let mut scheduler = Scheduler::new();
        scheduler.enqueue(1);
        scheduler.enqueue(2);
        scheduler.enqueue(3);

        assert_eq!(scheduler.ready_count(), 3);
        assert!(scheduler.is_ready(1));
        assert!(scheduler.is_ready(2));
        assert!(scheduler.is_ready(3));
    }

    #[test]
    fn test_scheduler_enqueue_duplicate() {
        let mut scheduler = Scheduler::new();
        scheduler.enqueue(1);
        scheduler.enqueue(1); // Duplicate should be ignored

        assert_eq!(scheduler.ready_count(), 1);
    }

    #[test]
    fn test_scheduler_dequeue() {
        let mut scheduler = Scheduler::new();
        scheduler.enqueue(1);
        scheduler.enqueue(2);
        scheduler.enqueue(3);

        scheduler.dequeue(2);
        assert_eq!(scheduler.ready_count(), 2);
        assert!(!scheduler.is_ready(2));
    }

    #[test]
    fn test_scheduler_handles_empty_queue() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.schedule(), None);
        assert_eq!(scheduler.schedule(), None);
    }

    #[test]
    fn test_scheduler_round_robin() {
        let mut scheduler = Scheduler::new();
        scheduler.enqueue(1);
        scheduler.enqueue(2);
        scheduler.enqueue(3);

        // First round
        assert_eq!(scheduler.schedule(), Some(1));
        assert_eq!(scheduler.schedule(), Some(2));
        assert_eq!(scheduler.schedule(), Some(3));

        // Second round (should cycle back)
        assert_eq!(scheduler.schedule(), Some(1));
        assert_eq!(scheduler.schedule(), Some(2));
        assert_eq!(scheduler.schedule(), Some(3));
    }

    #[test]
    fn test_scheduler_pid_uniqueness() {
        let mut scheduler = Scheduler::new();
        scheduler.enqueue(1);
        scheduler.enqueue(2);
        scheduler.enqueue(2); // Duplicate
        scheduler.enqueue(3);

        // Should only have 3 unique PIDs
        assert_eq!(scheduler.ready_count(), 3);

        // Verify round-robin works correctly
        assert_eq!(scheduler.schedule(), Some(1));
        assert_eq!(scheduler.schedule(), Some(2));
        assert_eq!(scheduler.schedule(), Some(3));
        assert_eq!(scheduler.schedule(), Some(1)); // Back to start
    }

    #[test]
    fn test_scheduler_sync_with_state() {
        let mut state = KernelState::new();
        let mut scheduler = Scheduler::new();

        // Create some processes
        let pid1 = state.allocate_pid();
        let mut proc1 = Process::new(pid1, None);
        proc1.state = ProcessState::Ready;
        state.add_process(proc1);

        let pid2 = state.allocate_pid();
        let mut proc2 = Process::new(pid2, None);
        proc2.state = ProcessState::Running;
        state.add_process(proc2);

        let pid3 = state.allocate_pid();
        let mut proc3 = Process::new(pid3, None);
        proc3.state = ProcessState::Blocked;
        state.add_process(proc3);

        let pid4 = state.allocate_pid();
        let mut proc4 = Process::new(pid4, None);
        proc4.state = ProcessState::Terminated(0);
        state.add_process(proc4);

        // Sync scheduler with state
        scheduler.sync_with_state(&state);

        // Only Ready and Running processes should be in queue
        assert_eq!(scheduler.ready_count(), 2);
        assert!(scheduler.is_ready(pid1));
        assert!(scheduler.is_ready(pid2));
        assert!(!scheduler.is_ready(pid3)); // Blocked
        assert!(!scheduler.is_ready(pid4)); // Terminated
    }

    #[test]
    fn test_scheduler_dequeue_adjusts_index() {
        let mut scheduler = Scheduler::new();
        scheduler.enqueue(1);
        scheduler.enqueue(2);
        scheduler.enqueue(3);

        // Schedule to move index forward
        scheduler.schedule(); // index = 1
        scheduler.schedule(); // index = 2

        // Remove process at end
        scheduler.dequeue(3);

        // Index should be reset to 0
        assert_eq!(scheduler.current_index, 0);
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;
        use std::collections::HashMap;

        proptest! {
            /// Property: Scheduler never returns a PID not in the ready queue
            #[test]
            fn proptest_scheduler_returns_valid_pids(
                pids in prop::collection::vec(1..1000u32, 1..100),
                num_schedules in 1..1000usize,
            ) {
                let mut scheduler = Scheduler::new();

                // Enqueue all PIDs
                for pid in &pids {
                    scheduler.enqueue(*pid);
                }

                // Schedule many times
                for _ in 0..num_schedules {
                    if let Some(pid) = scheduler.schedule() {
                        prop_assert!(pids.contains(&pid), "Scheduler returned PID not in queue");
                    }
                }
            }

            /// Property: Round-robin fairness - all processes get equal CPU time
            #[test]
            fn proptest_scheduler_fairness(
                num_processes in 1..100usize,
                num_rounds in 1..100usize,
            ) {
                let mut scheduler = Scheduler::new();
                let mut schedule_counts = HashMap::new();

                // Enqueue processes
                for i in 1..=num_processes as u32 {
                    scheduler.enqueue(i);
                    schedule_counts.insert(i, 0);
                }

                // Schedule for multiple rounds
                let total_schedules = num_processes * num_rounds;
                for _ in 0..total_schedules {
                    if let Some(pid) = scheduler.schedule() {
                        *schedule_counts.get_mut(&pid).unwrap() += 1;
                    }
                }

                // All processes should have been scheduled equally (num_rounds times)
                for count in schedule_counts.values() {
                    prop_assert_eq!(*count, num_rounds, "Process not scheduled fairly");
                }
            }

            /// Property: No starvation - every process eventually gets scheduled
            #[test]
            fn proptest_no_starvation(
                num_processes in 1..100usize,
            ) {
                let mut scheduler = Scheduler::new();
                let mut scheduled = std::collections::HashSet::new();

                // Enqueue processes
                for i in 1..=num_processes as u32 {
                    scheduler.enqueue(i);
                }

                // Schedule until all processes have been scheduled at least once
                let max_iterations = num_processes * 2; // Should need at most num_processes iterations
                for _ in 0..max_iterations {
                    if let Some(pid) = scheduler.schedule() {
                        scheduled.insert(pid);
                    }

                    if scheduled.len() == num_processes {
                        break;
                    }
                }

                // All processes should have been scheduled
                prop_assert_eq!(scheduled.len(), num_processes, "Some process was starved");
            }

            /// Property: Enqueue/dequeue operations maintain consistency
            #[test]
            fn proptest_scheduler_operations(
                operations in prop::collection::vec((0..3usize, 1..1000u32), 0..200),
            ) {
                let mut scheduler = Scheduler::new();
                let mut expected_pids = std::collections::HashSet::new();

                for (op, pid) in operations {
                    match op {
                        0 => {
                            // Enqueue
                            scheduler.enqueue(pid);
                            expected_pids.insert(pid);
                        }
                        1 => {
                            // Dequeue
                            scheduler.dequeue(pid);
                            expected_pids.remove(&pid);
                        }
                        _ => {
                            // Schedule
                            if let Some(scheduled_pid) = scheduler.schedule() {
                                prop_assert!(expected_pids.contains(&scheduled_pid));
                            }
                        }
                    }
                }

                // Final consistency check
                prop_assert_eq!(scheduler.ready_count(), expected_pids.len());
            }

            /// Property: Cloning scheduler preserves state
            #[test]
            fn proptest_scheduler_cloning(
                pids in prop::collection::vec(1..1000u32, 0..100),
            ) {
                let mut scheduler = Scheduler::new();

                for pid in &pids {
                    scheduler.enqueue(*pid);
                }

                // Clone should preserve all data
                let scheduler2 = scheduler.clone();
                prop_assert_eq!(scheduler.ready_count(), scheduler2.ready_count());
                prop_assert_eq!(scheduler, scheduler2);
            }
        }
    }

    #[test]
    fn test_scheduler_default() {
        let scheduler1 = Scheduler::default();
        let scheduler2 = Scheduler::new();
        assert_eq!(scheduler1.ready_count(), scheduler2.ready_count());
        assert_eq!(scheduler1, scheduler2);
    }
}
