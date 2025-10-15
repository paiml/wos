#![no_main]

use libfuzzer_sys::fuzz_target;
use wos_kernel::{dispatch_syscall, KernelState, SystemCall};

fuzz_target!(|data: &[u8]| {
    // Try to deserialize random data as a SystemCall
    if let Ok(syscall_json) = std::str::from_utf8(data) {
        if let Ok(syscall) = serde_json::from_str::<SystemCall>(syscall_json) {
            let state = KernelState::new();

            // Execute syscall - should never panic
            let _ = dispatch_syscall(state, syscall, 1);
        }
    }
});
