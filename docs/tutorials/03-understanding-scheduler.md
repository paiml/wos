# Tutorial 3: Understanding the Scheduler

Deep dive into WOS's round-robin scheduler and process scheduling.

## Prerequisites

- Completed [Tutorial 1](01-adding-syscall.md) and [Tutorial 2](02-creating-program.md)
- Understanding of process states
- Basic knowledge of operating system scheduling

## Goal

Understand how WOS schedules processes and implement scheduler optimizations.

## The Scheduler Architecture

### Core Concepts

WOS uses a **preemptive round-robin scheduler** with these properties:

- **O(1) selection** - Constant time next process selection
- **Fair scheduling** - Each process gets equal CPU time
- **State-aware** - Only schedules `Ready` processes
- **Pure functional** - Returns new state, never mutates

### Scheduler State

```rust
pub struct Scheduler {
    /// Queue of ready processes (PIDs only)
    pub ready_queue: VecDeque<ProcessId>,

    /// Currently running process
    pub current_pid: Option<ProcessId>,

    /// Time quantum in milliseconds (default: 100ms)
    pub quantum: u64,
}
```

### Process States

```rust
pub enum ProcessState {
    Ready,           // Can be scheduled
    Running,         // Currently executing
    Blocked,         // Waiting for I/O or event
    Terminated(i32), // Exited with code
}
```

## How Scheduling Works

### Step-by-Step Execution

```rust
// 1. Initialize scheduler with init process
let mut scheduler = Scheduler::new();
// ready_queue: [1]
// current_pid: None

// 2. Schedule next process
let (new_scheduler, next_pid) = scheduler.schedule();
// ready_queue: []
// current_pid: Some(1)
// Returns: PID 1

// 3. Process executes syscall (fork)
let (new_state, _) = sys_fork(state, 1).unwrap();
// New process created: PID 2

// 4. Scheduler updated with new process
let new_scheduler = scheduler.add_process(2);
// ready_queue: [2]
// current_pid: Some(1)

// 5. Time quantum expires, reschedule
let new_scheduler = scheduler.yield_process(1);
// ready_queue: [1]  (PID 1 moved to back)
// current_pid: None

let (new_scheduler, next_pid) = scheduler.schedule();
// ready_queue: [1]
// current_pid: Some(2)
// Returns: PID 2
```

### Visualization

```
Time: 0ms          100ms         200ms         300ms
      |-------------|-------------|-------------|
PID 1 [===RUNNING===][---READY---][===RUNNING===]
PID 2               [---READY---][===RUNNING===]
PID 3                           [---READY---][===]

Ready Queue Evolution:
t=0ms:   [1]
t=100ms: [2, 1]      (1 yields, 2 scheduled)
t=200ms: [3, 1, 2]   (2 yields, 3 added, 1 scheduled)
t=300ms: [1, 2, 3]   (1 yields, 3 scheduled)
```

## Scheduler Implementation

### Core Methods

#### `schedule()` - Select Next Process

```rust
pub fn schedule(mut self) -> (Self, Option<ProcessId>) {
    // Pop from front of queue
    match self.ready_queue.pop_front() {
        Some(pid) => {
            self.current_pid = Some(pid);
            (self, Some(pid))
        }
        None => {
            self.current_pid = None;
            (self, None)  // No ready processes
        }
    }
}
```

**Complexity**: O(1)
**Invariants**:
- If queue is empty, returns None
- Selected process removed from ready queue
- Current PID updated

#### `yield_process()` - Yield CPU

```rust
pub fn yield_process(mut self, pid: ProcessId) -> Self {
    // Add to back of queue
    if self.current_pid == Some(pid) {
        self.ready_queue.push_back(pid);
        self.current_pid = None;
    }
    self
}
```

**Complexity**: O(1)
**Invariants**:
- Process moves to back of queue (fairness)
- Only current process can yield

#### `add_process()` - Add New Process

```rust
pub fn add_process(mut self, pid: ProcessId) -> Self {
    // New processes start at back of queue
    self.ready_queue.push_back(pid);
    self
}
```

#### `remove_process()` - Remove Process

```rust
pub fn remove_process(mut self, pid: ProcessId) -> Self {
    // Remove from queue (terminated or blocked)
    self.ready_queue.retain(|&p| p != pid);

    if self.current_pid == Some(pid) {
        self.current_pid = None;
    }

    self
}
```

**Complexity**: O(n) where n is queue length
**Note**: Could be optimized with HashSet

## Testing the Scheduler

### Unit Tests

```rust
#[test]
fn test_schedule_round_robin() {
    let mut scheduler = Scheduler::new();

    // Add processes 1, 2, 3
    scheduler = scheduler.add_process(1);
    scheduler = scheduler.add_process(2);
    scheduler = scheduler.add_process(3);

    // Schedule in round-robin order
    let (scheduler, pid1) = scheduler.schedule();
    assert_eq!(pid1, Some(1));

    scheduler = scheduler.yield_process(1);
    let (scheduler, pid2) = scheduler.schedule();
    assert_eq!(pid2, Some(2));

    scheduler = scheduler.yield_process(2);
    let (scheduler, pid3) = scheduler.schedule();
    assert_eq!(pid3, Some(3));

    scheduler = scheduler.yield_process(3);
    let (scheduler, pid4) = scheduler.schedule();
    assert_eq!(pid4, Some(1));  // Back to PID 1
}

#[test]
fn test_schedule_empty_queue() {
    let scheduler = Scheduler::new();

    let (scheduler, pid) = scheduler.schedule();
    assert_eq!(pid, None);  // No processes to schedule
}

#[test]
fn test_remove_process() {
    let mut scheduler = Scheduler::new();
    scheduler = scheduler.add_process(1);
    scheduler = scheduler.add_process(2);

    // Remove PID 1
    scheduler = scheduler.remove_process(1);

    // Only PID 2 should be scheduled
    let (scheduler, pid) = scheduler.schedule();
    assert_eq!(pid, Some(2));
}
```

### Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_scheduler_fairness(process_count in 1usize..20) {
        let mut scheduler = Scheduler::new();

        // Add N processes
        for pid in 1..=process_count {
            scheduler = scheduler.add_process(pid as ProcessId);
        }

        // Schedule N times
        let mut scheduled = Vec::new();
        for _ in 0..process_count {
            let (new_sched, pid) = scheduler.schedule();
            scheduler = new_sched;
            scheduled.push(pid.unwrap());
            scheduler = scheduler.yield_process(pid.unwrap());
        }

        // Each process scheduled exactly once
        for pid in 1..=process_count {
            prop_assert!(scheduled.contains(&(pid as ProcessId)));
        }
    }

    #[test]
    fn prop_scheduler_never_loses_processes(ops in prop::collection::vec(0u8..3, 1..50)) {
        let mut scheduler = Scheduler::new();
        let mut added = std::collections::HashSet::new();

        for op in ops {
            match op % 3 {
                0 => {
                    // Add process
                    let pid = (added.len() + 1) as ProcessId;
                    scheduler = scheduler.add_process(pid);
                    added.insert(pid);
                }
                1 => {
                    // Schedule
                    let (new_sched, _) = scheduler.schedule();
                    scheduler = new_sched;
                }
                2 => {
                    // Yield current
                    if let Some(pid) = scheduler.current_pid {
                        scheduler = scheduler.yield_process(pid);
                    }
                }
                _ => unreachable!(),
            }
        }

        // All added processes still in system
        let total_processes = scheduler.ready_queue.len()
            + if scheduler.current_pid.is_some() { 1 } else { 0 };
        prop_assert_eq!(total_processes, added.len());
    }
}
```

## Scheduler Optimizations

### Optimization 1: Priority Scheduling

Add priority levels to processes:

```rust
pub struct Process {
    pub pid: ProcessId,
    pub priority: u8,  // 0 = highest, 255 = lowest
    // ... other fields
}

pub struct Scheduler {
    // Multiple queues, one per priority
    pub queues: HashMap<u8, VecDeque<ProcessId>>,
    pub current_pid: Option<ProcessId>,
}
```

Implementation:

```rust
impl Scheduler {
    pub fn schedule(mut self) -> (Self, Option<ProcessId>) {
        // Schedule from highest priority queue
        for priority in 0..=255 {
            if let Some(queue) = self.queues.get_mut(&priority) {
                if let Some(pid) = queue.pop_front() {
                    self.current_pid = Some(pid);
                    return (self, Some(pid));
                }
            }
        }

        (self, None)  // No ready processes
    }
}
```

**Tests for priority**:

```rust
#[test]
fn test_priority_scheduling() {
    let mut scheduler = Scheduler::new();

    // Add high priority process
    scheduler = scheduler.add_process_with_priority(1, 0);

    // Add low priority process
    scheduler = scheduler.add_process_with_priority(2, 10);

    // High priority scheduled first
    let (scheduler, pid) = scheduler.schedule();
    assert_eq!(pid, Some(1));
}
```

### Optimization 2: Aging (Prevent Starvation)

Increase priority of processes that wait too long:

```rust
pub struct Process {
    pub pid: ProcessId,
    pub priority: u8,
    pub wait_time: u64,  // Milliseconds in ready queue
}

impl Scheduler {
    pub fn age_processes(mut self, delta_ms: u64) -> Self {
        for (pid, process) in self.processes.iter_mut() {
            if self.ready_queue.contains(pid) {
                process.wait_time += delta_ms;

                // Increase priority every 500ms
                if process.wait_time >= 500 && process.priority > 0 {
                    process.priority -= 1;
                    process.wait_time = 0;
                }
            }
        }
        self
    }
}
```

### Optimization 3: Multi-Level Feedback Queue

Processes start at high priority, demoted if CPU-bound:

```rust
impl Scheduler {
    pub fn yield_process(mut self, pid: ProcessId) -> Self {
        if self.current_pid == Some(pid) {
            // Get process's current priority
            let priority = self.get_priority(pid);

            // Demote if still has lower levels
            let new_priority = std::cmp::min(priority + 1, 255);

            // Add to new priority queue
            self.queues
                .entry(new_priority)
                .or_insert_with(VecDeque::new)
                .push_back(pid);

            self.current_pid = None;
        }
        self
    }
}
```

## Practical Example: CPU Benchmarking

Create a program that demonstrates scheduling fairness:

```rust
// userspace/src/bin/cpubench.rs

pub fn main(
    state: KernelState,
    pid: ProcessId,
    args: Vec<String>,
) -> Result<(KernelState, i32), KernelError> {
    let iterations: u64 = args
        .get(0)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000000);

    // Simulate CPU work
    let mut counter = 0u64;
    for i in 0..iterations {
        counter = counter.wrapping_add(i);

        // Yield every 10000 iterations
        if i % 10000 == 0 {
            let syscall = SystemCall::Yield;
            let (new_state, _) = dispatch_syscall(state, syscall, pid)?;
            state = new_state;
        }
    }

    // Print result
    let output = format!("Completed {} iterations\n", iterations);
    let syscall = SystemCall::Write(1, output.into_bytes());
    let (new_state, _) = dispatch_syscall(state, syscall, pid)?;

    Ok((new_state, 0))
}
```

Run multiple instances concurrently to test fairness:

```rust
#[test]
fn test_scheduler_fairness_cpubench() {
    let mut state = KernelState::new();

    // Fork 4 processes
    let mut pids = vec![1];
    for _ in 0..3 {
        let (new_state, output) = sys_fork(state, 1).unwrap();
        state = new_state;
        if let SyscallOutput::Pid(pid) = output {
            pids.push(pid);
        }
    }

    // Each runs cpubench
    for pid in pids {
        let cpubench_fn = get_program("cpubench").unwrap();
        let (new_state, _) = cpubench_fn(
            state,
            pid,
            vec!["100000".to_string()]
        ).unwrap();
        state = new_state;
    }

    // All should complete (fairness)
    // Measure timing variance
}
```

## Summary

You now understand:

1. ✅ **Round-robin scheduling** - O(1) fair process selection
2. ✅ **Process states** - Ready, Running, Blocked, Terminated
3. ✅ **Scheduler invariants** - Fairness, no process loss
4. ✅ **Testing strategies** - Unit and property tests
5. ✅ **Optimizations** - Priority, aging, MLFQ
6. ✅ **Practical benchmarking** - Testing fairness

## Exercise: Implement Shortest Job First (SJF)

Implement a SJF scheduler:

```rust
pub struct Process {
    pub pid: ProcessId,
    pub estimated_time: u64,  // Estimated execution time
}

pub struct SJFScheduler {
    pub ready_queue: BinaryHeap<Process>,  // Min-heap by estimated_time
}
```

Requirements:
- Schedule process with shortest estimated time first
- Add `sys_set_estimated_time` syscall
- Write 5+ tests comparing SJF to round-robin
- Measure average turnaround time

## Common Pitfalls

1. **Forgetting to yield** - CPU-bound processes must yield periodically
2. **Priority inversion** - Low priority process holds resource needed by high priority
3. **Starvation** - Low priority processes never scheduled
4. **Race conditions** - In real systems (not WOS, we're pure functional)
5. **Not testing fairness** - Always verify each process gets CPU time

## Next Steps

- [Tutorial 4: Implementing Pipes](04-implementing-pipes.md)
- [Tutorial 5: Advanced Memory Management](05-advanced-memory.md)
- [Architecture Guide](../ARCHITECTURE.md)

## Further Reading

- [Process Scheduling Algorithms](https://en.wikipedia.org/wiki/Scheduling_(computing))
- [Round-Robin Scheduling](https://en.wikipedia.org/wiki/Round-robin_scheduling)
- [Multi-Level Feedback Queue](https://en.wikipedia.org/wiki/Multilevel_feedback_queue)
- [Linux CFS Scheduler](https://www.kernel.org/doc/html/latest/scheduler/sched-design-CFS.html)
