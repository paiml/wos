#![no_main]

use libfuzzer_sys::fuzz_target;
use wos_kernel::{sys_mmap, sys_munmap, KernelState, PagePermissions, SyscallOutput};

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }

    let allocation_count = (data[0] % 20) as usize;
    let size_multiplier = (data[1] % 10) as usize + 1;
    let perms_byte = data[2];

    // Determine permissions from fuzzer input
    let perms = match perms_byte % 7 {
        0 => PagePermissions::READ,
        1 => PagePermissions::WRITE,
        2 => PagePermissions::EXECUTE,
        3 => PagePermissions::READ | PagePermissions::WRITE,
        4 => PagePermissions::READ | PagePermissions::EXECUTE,
        5 => PagePermissions::WRITE | PagePermissions::EXECUTE,
        _ => PagePermissions::READ | PagePermissions::WRITE | PagePermissions::EXECUTE,
    };

    let mut state = KernelState::new();
    let mut addresses = Vec::new();

    // Allocate memory
    for _ in 0..allocation_count {
        let size = size_multiplier * 4096; // Multiple of page size
        if let Ok((new_state, output)) = sys_mmap(state, 1, size, perms) {
            state = new_state;
            if let SyscallOutput::Address(addr) = output {
                addresses.push((addr, size));
            }
        }
    }

    // Free some allocations
    for (i, &(addr, size)) in addresses.iter().enumerate() {
        if i % 2 == 0 {
            if let Ok((new_state, _)) = sys_munmap(state.clone(), 1, addr, size) {
                state = new_state;
            }
        }
    }

    // Allocate again (test fragmentation)
    for _ in 0..5 {
        let size = 4096;
        if let Ok((new_state, _)) = sys_mmap(state.clone(), 1, size, perms) {
            state = new_state;
        }
    }
});
