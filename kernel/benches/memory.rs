use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wos_kernel::{sys_mmap, sys_munmap, KernelState, PagePermissions, SyscallOutput};

fn bench_mmap_single_page(c: &mut Criterion) {
    c.bench_function("mmap_4kb", |b| {
        let state = KernelState::new();
        b.iter(|| {
            let (new_state, output) = sys_mmap(
                black_box(state.clone()),
                1,
                4096,
                PagePermissions::READ | PagePermissions::WRITE,
            )
            .unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_mmap_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmap_sizes");

    for size_kb in [4, 16, 64, 256, 1024].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb", size_kb)),
            size_kb,
            |b, &kb| {
                let state = KernelState::new();
                let size = kb * 1024;

                b.iter(|| {
                    let (new_state, output) = sys_mmap(
                        black_box(state.clone()),
                        1,
                        size,
                        PagePermissions::READ | PagePermissions::WRITE,
                    )
                    .unwrap();
                    black_box((new_state, output))
                });
            },
        );
    }
    group.finish();
}

fn bench_munmap(c: &mut Criterion) {
    c.bench_function("munmap_4kb", |b| {
        let state = KernelState::new();
        let (state, output) = sys_mmap(
            state,
            1,
            4096,
            PagePermissions::READ | PagePermissions::WRITE,
        )
        .unwrap();
        let addr = match output {
            SyscallOutput::Address(addr) => addr,
            _ => panic!("Expected Address"),
        };

        b.iter(|| {
            let (new_state, output) = sys_munmap(black_box(state.clone()), 1, addr, 4096).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_mmap_munmap_cycle(c: &mut Criterion) {
    c.bench_function("mmap_munmap_cycle", |b| {
        let state = KernelState::new();

        b.iter(|| {
            let (state, output) = sys_mmap(
                black_box(state.clone()),
                1,
                4096,
                PagePermissions::READ | PagePermissions::WRITE,
            )
            .unwrap();

            let addr = match output {
                SyscallOutput::Address(addr) => addr,
                _ => panic!("Expected Address"),
            };

            let (final_state, _) = sys_munmap(state, 1, addr, 4096).unwrap();
            black_box(final_state)
        });
    });
}

fn bench_mmap_permissions(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmap_permissions");

    let permissions = [
        ("read", PagePermissions::READ),
        ("write", PagePermissions::WRITE),
        ("execute", PagePermissions::EXECUTE),
        ("rw", PagePermissions::READ | PagePermissions::WRITE),
        (
            "rwx",
            PagePermissions::READ | PagePermissions::WRITE | PagePermissions::EXECUTE,
        ),
    ];

    for (name, perms) in permissions.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), perms, |b, &perms| {
            let state = KernelState::new();

            b.iter(|| {
                let (new_state, output) =
                    sys_mmap(black_box(state.clone()), 1, 4096, perms).unwrap();
                black_box((new_state, output))
            });
        });
    }
    group.finish();
}

fn bench_mmap_multiple_allocations(c: &mut Criterion) {
    c.bench_function("mmap_10_allocations", |b| {
        let state = KernelState::new();

        b.iter(|| {
            let mut current_state = black_box(state.clone());

            for _ in 0..10 {
                let (new_state, _) = sys_mmap(
                    current_state,
                    1,
                    4096,
                    PagePermissions::READ | PagePermissions::WRITE,
                )
                .unwrap();
                current_state = new_state;
            }

            black_box(current_state)
        });
    });
}

fn bench_memory_fragmentation(c: &mut Criterion) {
    c.bench_function("memory_fragmentation_pattern", |b| {
        let state = KernelState::new();

        b.iter(|| {
            let mut current_state = black_box(state.clone());
            let mut addresses = Vec::new();

            // Allocate 10 pages
            for _ in 0..10 {
                let (new_state, output) = sys_mmap(
                    current_state,
                    1,
                    4096,
                    PagePermissions::READ | PagePermissions::WRITE,
                )
                .unwrap();

                if let SyscallOutput::Address(addr) = output {
                    addresses.push(addr);
                }
                current_state = new_state;
            }

            // Free every other page
            for (i, &addr) in addresses.iter().enumerate() {
                if i % 2 == 0 {
                    let (new_state, _) = sys_munmap(current_state, 1, addr, 4096).unwrap();
                    current_state = new_state;
                }
            }

            black_box(current_state)
        });
    });
}

fn bench_kernel_state_clone_with_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("kernel_state_clone");

    for allocation_count in [0, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_allocations", allocation_count)),
            allocation_count,
            |b, &count| {
                let mut state = KernelState::new();

                // Pre-allocate memory
                for _ in 0..count {
                    let (new_state, _) = sys_mmap(
                        state,
                        1,
                        4096,
                        PagePermissions::READ | PagePermissions::WRITE,
                    )
                    .unwrap();
                    state = new_state;
                }

                b.iter(|| {
                    let cloned = black_box(state.clone());
                    black_box(cloned)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    memory,
    bench_mmap_single_page,
    bench_mmap_sizes,
    bench_munmap,
    bench_mmap_munmap_cycle,
    bench_mmap_permissions,
    bench_mmap_multiple_allocations,
    bench_memory_fragmentation,
    bench_kernel_state_clone_with_memory,
);
criterion_main!(memory);
