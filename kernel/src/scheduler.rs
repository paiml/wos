//! Process Scheduler
//!
//! Round-robin scheduler with fairness guarantees and no starvation.

use crate::state::{KernelState, ProcessId};
use im::Vector;
use serde::{Deserialize, Serialize};

/// Number of priority levels (0 = highest, 7 = lowest)
pub const NUM_PRIORITY_LEVELS: usize = 8;

/// Aging threshold - boost priority after this many wait ticks
pub const AGING_THRESHOLD: u64 = 5;

/// Round-robin process scheduler with priority support
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scheduler {
    /// Ready queue (processes ready to run) - legacy round-robin
    ready_queue: Vector<ProcessId>,
    /// Current position in round-robin (for fairness tracking)
    current_index: usize,
    /// Priority queues (one per priority level 0-7)
    priority_queues: [Vector<ProcessId>; NUM_PRIORITY_LEVELS],
    /// Current index within each priority level (for round-robin within priority)
    priority_indices: [usize; NUM_PRIORITY_LEVELS],
    /// Wait ticks tracking for aging (ProcessId -> wait_ticks)
    wait_ticks: im::HashMap<ProcessId, u64>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            ready_queue: Vector::new(),
            current_index: 0,
            priority_queues: [
                Vector::new(),
                Vector::new(),
                Vector::new(),
                Vector::new(),
                Vector::new(),
                Vector::new(),
                Vector::new(),
                Vector::new(),
            ],
            priority_indices: [0; NUM_PRIORITY_LEVELS],
            wait_ticks: im::HashMap::new(),
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

        // Clear priority queues
        for i in 0..NUM_PRIORITY_LEVELS {
            self.priority_queues[i] = Vector::new();
            self.priority_indices[i] = 0;
        }
        self.wait_ticks.clear();

        // Add all runnable processes to both queues
        for (pid, process) in state.processes.iter() {
            if process.is_runnable() {
                self.ready_queue.push_back(*pid);

                // Add to priority queue
                let priority = process.priority.min(7) as usize; // Clamp to 0-7
                if !self.priority_queues[priority].contains(pid) {
                    self.priority_queues[priority].push_back(*pid);
                }
                self.wait_ticks.insert(*pid, process.wait_ticks);
            }
        }
    }

    /// Add a process to a specific priority queue
    pub fn enqueue_priority(&mut self, pid: ProcessId, priority: u8) {
        let priority_level = priority.min(7) as usize;

        // Only add if not already in queue
        if !self.priority_queues[priority_level].contains(&pid) {
            self.priority_queues[priority_level].push_back(pid);
        }

        // Initialize wait ticks
        if !self.wait_ticks.contains_key(&pid) {
            self.wait_ticks.insert(pid, 0);
        }
    }

    /// Remove a process from priority queue
    pub fn dequeue_priority(&mut self, pid: ProcessId, priority: u8) {
        let priority_level = priority.min(7) as usize;

        // Find and remove the process
        if let Some(idx) = self.priority_queues[priority_level]
            .iter()
            .position(|&p| p == pid)
        {
            self.priority_queues[priority_level].remove(idx);

            // Adjust current_index if needed
            if self.priority_indices[priority_level] >= self.priority_queues[priority_level].len()
                && !self.priority_queues[priority_level].is_empty()
            {
                self.priority_indices[priority_level] = 0;
            }
        }

        self.wait_ticks.remove(&pid);
    }

    /// Select next process using priority-based scheduling with aging
    ///
    /// Returns the next process to run, selecting from the highest priority
    /// non-empty queue. Uses round-robin within each priority level.
    /// Implements aging to prevent starvation of low-priority processes.
    pub fn schedule_priority(&mut self, state: &KernelState) -> Option<ProcessId> {
        // Apply aging - boost priority for processes that have waited too long
        self.apply_aging(state);

        // Find highest priority non-empty queue
        for priority_level in 0..NUM_PRIORITY_LEVELS {
            if !self.priority_queues[priority_level].is_empty() {
                let queue_len = self.priority_queues[priority_level].len();
                let idx = self.priority_indices[priority_level];

                let pid = self.priority_queues[priority_level][idx];

                // Move to next process in this priority level (round-robin)
                self.priority_indices[priority_level] = (idx + 1) % queue_len;

                // Reset wait ticks for scheduled process
                self.wait_ticks.insert(pid, 0);

                // Increment wait ticks for all other processes
                for other_priority in 0..NUM_PRIORITY_LEVELS {
                    for other_pid in self.priority_queues[other_priority].iter() {
                        if *other_pid != pid {
                            let ticks = self.wait_ticks.get(other_pid).copied().unwrap_or(0);
                            self.wait_ticks.insert(*other_pid, ticks + 1);
                        }
                    }
                }

                return Some(pid);
            }
        }

        None
    }

    /// Apply aging to prevent starvation
    ///
    /// Processes that have waited too long get temporarily boosted priority
    fn apply_aging(&mut self, _state: &KernelState) {
        let mut processes_to_boost = Vec::new();

        // Find processes that need priority boost
        for priority_level in 1..NUM_PRIORITY_LEVELS {
            for pid in self.priority_queues[priority_level].iter() {
                if let Some(&ticks) = self.wait_ticks.get(pid) {
                    if ticks >= AGING_THRESHOLD {
                        // Boost to next higher priority level
                        processes_to_boost.push((*pid, priority_level));
                    }
                }
            }
        }

        // Apply boosts
        for (pid, old_priority) in processes_to_boost {
            // Remove from old priority queue
            if let Some(idx) = self.priority_queues[old_priority]
                .iter()
                .position(|&p| p == pid)
            {
                self.priority_queues[old_priority].remove(idx);

                // Adjust index if needed
                if self.priority_indices[old_priority] >= self.priority_queues[old_priority].len()
                    && !self.priority_queues[old_priority].is_empty()
                {
                    self.priority_indices[old_priority] = 0;
                }
            }

            // Add to higher priority queue (one level up)
            let new_priority = old_priority.saturating_sub(1);
            if !self.priority_queues[new_priority].contains(&pid) {
                self.priority_queues[new_priority].push_back(pid);
            }

            // Reset wait ticks after boost
            self.wait_ticks.insert(pid, 0);
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

    // WOS-022: Priority Scheduling Tests
    mod priority_tests {
        use super::*;

        #[test]
        fn test_priority_scheduler_creation() {
            let scheduler = Scheduler::new();
            assert_eq!(scheduler.ready_count(), 0);
        }

        #[test]
        fn test_priority_scheduler_high_priority_first() {
            let mut state = KernelState::new();
            let mut scheduler = Scheduler::new();

            // Create processes with different priorities
            let pid_low = state.allocate_pid();
            let mut proc_low = Process::new(pid_low, None);
            proc_low.state = ProcessState::Ready;
            proc_low.priority = 7; // Lowest priority
            state.add_process(proc_low);

            let pid_high = state.allocate_pid();
            let mut proc_high = Process::new(pid_high, None);
            proc_high.state = ProcessState::Ready;
            proc_high.priority = 0; // Highest priority
            state.add_process(proc_high);

            let pid_med = state.allocate_pid();
            let mut proc_med = Process::new(pid_med, None);
            proc_med.state = ProcessState::Ready;
            proc_med.priority = 4; // Normal priority
            state.add_process(proc_med);

            // Sync scheduler with state
            scheduler.sync_with_state(&state);

            // High priority process should be scheduled first (and keeps running)
            let first = scheduler.schedule_priority(&state);
            assert_eq!(first, Some(pid_high));

            // In priority scheduling, highest priority process runs repeatedly
            // until it blocks or terminates. Let's test this behavior.
            assert_eq!(scheduler.schedule_priority(&state), Some(pid_high));
            assert_eq!(scheduler.schedule_priority(&state), Some(pid_high));

            // Remove high priority process from its queue
            scheduler.dequeue_priority(pid_high, 0);

            // Now medium priority should run
            assert_eq!(scheduler.schedule_priority(&state), Some(pid_med));

            // Remove medium priority process
            scheduler.dequeue_priority(pid_med, 4);

            // Now low priority should run
            assert_eq!(scheduler.schedule_priority(&state), Some(pid_low));
        }

        #[test]
        fn test_priority_scheduler_round_robin_within_priority() {
            let mut state = KernelState::new();
            let mut scheduler = Scheduler::new();

            // Create multiple processes at same priority
            let pid1 = state.allocate_pid();
            let mut proc1 = Process::new(pid1, None);
            proc1.state = ProcessState::Ready;
            proc1.priority = 4;
            state.add_process(proc1);

            let pid2 = state.allocate_pid();
            let mut proc2 = Process::new(pid2, None);
            proc2.state = ProcessState::Ready;
            proc2.priority = 4;
            state.add_process(proc2);

            let pid3 = state.allocate_pid();
            let mut proc3 = Process::new(pid3, None);
            proc3.state = ProcessState::Ready;
            proc3.priority = 4;
            state.add_process(proc3);

            scheduler.sync_with_state(&state);

            // Should cycle through processes at same priority level
            let first = scheduler.schedule_priority(&state);
            let second = scheduler.schedule_priority(&state);
            let third = scheduler.schedule_priority(&state);
            let fourth = scheduler.schedule_priority(&state);

            assert!(first.is_some());
            assert!(second.is_some());
            assert!(third.is_some());
            assert_eq!(first, fourth); // Should cycle back
        }

        #[test]
        fn test_priority_no_starvation_with_aging() {
            let mut state = KernelState::new();
            let mut scheduler = Scheduler::new();

            // Create high priority process
            let pid_high = state.allocate_pid();
            let mut proc_high = Process::new(pid_high, None);
            proc_high.state = ProcessState::Ready;
            proc_high.priority = 0; // Highest
            state.add_process(proc_high);

            // Create low priority process
            let pid_low = state.allocate_pid();
            let mut proc_low = Process::new(pid_low, None);
            proc_low.state = ProcessState::Ready;
            proc_low.priority = 7; // Lowest
            state.add_process(proc_low);

            scheduler.sync_with_state(&state);

            // Schedule many times - low priority should eventually run due to aging
            let mut low_priority_ran = false;
            for i in 0..100 {
                if let Some(pid) = scheduler.schedule_priority(&state) {
                    if pid == pid_low {
                        low_priority_ran = true;
                        // Should run within reasonable time (before 100 iterations)
                        assert!(i < 50, "Low priority process starved for too long");
                        break;
                    }
                }
            }

            assert!(low_priority_ran, "Low priority process was starved");
        }

        #[test]
        fn test_priority_levels_range() {
            let mut state = KernelState::new();

            // Test all priority levels 0-7
            for priority in 0..=7 {
                let pid = state.allocate_pid();
                let mut proc = Process::new(pid, None);
                proc.priority = priority;
                proc.state = ProcessState::Ready;
                state.add_process(proc);
            }

            let mut scheduler = Scheduler::new();
            scheduler.sync_with_state(&state);

            // Should have all processes
            assert_eq!(scheduler.ready_count(), 8);
        }

        #[test]
        fn test_default_priority_is_normal() {
            let proc = Process::new(1, None);
            assert_eq!(proc.priority, 4); // Normal/default priority
        }

        #[test]
        fn test_priority_enqueue_maintains_order() {
            let mut state = KernelState::new();
            let mut scheduler = Scheduler::new();

            // Add processes in mixed priority order
            let mut pids_by_priority = std::collections::HashMap::new();
            for priority in [7, 0, 4, 2, 5, 1, 6, 3] {
                let pid = state.allocate_pid();
                let mut proc = Process::new(pid, None);
                proc.priority = priority;
                proc.state = ProcessState::Ready;
                state.add_process(proc.clone());
                scheduler.enqueue_priority(pid, priority);
                pids_by_priority.insert(priority, pid);
            }

            // Scheduling should respect priority order (0 is highest)
            // Priority 0 process should always run first
            for _ in 0..3 {
                let scheduled = scheduler.schedule_priority(&state).unwrap();
                let proc = state.get_process(scheduled).unwrap();
                assert_eq!(proc.priority, 0, "Highest priority (0) should run first");
            }

            // Remove priority 0, then priority 1 should run
            scheduler.dequeue_priority(*pids_by_priority.get(&0).unwrap(), 0);
            for _ in 0..2 {
                let scheduled = scheduler.schedule_priority(&state).unwrap();
                let proc = state.get_process(scheduled).unwrap();
                assert_eq!(proc.priority, 1, "Next highest priority (1) should run");
            }
        }
    }
}
