#![no_main]

use libfuzzer_sys::fuzz_target;
use wos_kernel::{sys_exit, sys_fork, sys_waitpid, KernelState, SyscallOutput};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let fork_count = (data[0] % 20) as usize; // Max 20 forks
    let exit_code = data[1] as i32;

    let mut state = KernelState::new();
    let mut pids = vec![1];

    // Create processes
    for _ in 0..fork_count {
        if let Ok((new_state, output)) = sys_fork(state, 1) {
            state = new_state;
            if let SyscallOutput::Pid(pid) = output {
                if pid != 0 {
                    pids.push(pid);
                }
            }
        }
    }

    // Exit some processes
    for (i, &pid) in pids.iter().enumerate() {
        if i % 2 == 0 && pid != 1 {
            if let Ok((new_state, _)) = sys_exit(state.clone(), pid, exit_code) {
                state = new_state;
            }
        }
    }

    // Wait for exited processes
    for (i, &pid) in pids.iter().enumerate() {
        if i % 2 == 0 && pid != 1 {
            let _ = sys_waitpid(state.clone(), 1, pid);
        }
    }
});
