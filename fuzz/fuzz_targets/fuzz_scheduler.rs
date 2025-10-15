#![no_main]

use libfuzzer_sys::fuzz_target;
use wos_kernel::Scheduler;

#[derive(Clone, Copy, Debug)]
enum SchedulerOp {
    AddProcess(u32),
    Schedule,
    YieldProcess(u32),
    RemoveProcess(u32),
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let mut scheduler = Scheduler::new();
    let mut current_pid = None;
    let mut next_pid = 1u32;

    // Parse fuzzer input as sequence of operations
    for &byte in data.iter().take(100) {
        // Limit operations
        let op = match byte % 4 {
            0 => SchedulerOp::AddProcess(next_pid),
            1 => SchedulerOp::Schedule,
            2 => SchedulerOp::YieldProcess(current_pid.unwrap_or(1)),
            _ => SchedulerOp::RemoveProcess((byte as u32 % next_pid.max(1)) + 1),
        };

        match op {
            SchedulerOp::AddProcess(pid) => {
                scheduler = scheduler.add_process(pid);
                next_pid += 1;
            }
            SchedulerOp::Schedule => {
                let (new_scheduler, pid) = scheduler.schedule();
                scheduler = new_scheduler;
                current_pid = pid;
            }
            SchedulerOp::YieldProcess(pid) => {
                if current_pid.is_some() {
                    scheduler = scheduler.yield_process(pid);
                    current_pid = None;
                }
            }
            SchedulerOp::RemoveProcess(pid) => {
                scheduler = scheduler.remove_process(pid);
                if current_pid == Some(pid) {
                    current_pid = None;
                }
            }
        }
    }

    // Invariant: ready_queue + current_pid should never exceed processes added
    let queue_len = scheduler.ready_queue.len();
    let current_count = if scheduler.current_pid.is_some() {
        1
    } else {
        0
    };
    assert!(queue_len + current_count <= next_pid as usize);
});
