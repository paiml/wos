//! Criterion benchmarks for wos-kernel.
//!
//! Benchmarks syscall dispatch, memory management, scheduler operations,
//! and process creation which are the hot paths in the microkernel.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wos_kernel::{
    dispatch_syscall, KernelState, MemoryLayout, PagePermissions, PageTableEntry, Scheduler,
    SystemCall, VirtualMemory,
};

fn make_kernel_with_process() -> KernelState {
    KernelState::with_init()
}

fn bench_syscall_getpid(c: &mut Criterion) {
    let state = make_kernel_with_process();
    c.bench_function("syscall_getpid", |b| {
        b.iter(|| {
            let _ = dispatch_syscall(black_box(state.clone()), SystemCall::GetPid, 2);
        });
    });
}

fn bench_syscall_fork(c: &mut Criterion) {
    let state = make_kernel_with_process();
    c.bench_function("syscall_fork", |b| {
        b.iter(|| {
            let _ = dispatch_syscall(black_box(state.clone()), SystemCall::Fork, 2);
        });
    });
}

fn bench_syscall_mmap(c: &mut Criterion) {
    let state = make_kernel_with_process();
    c.bench_function("syscall_mmap_4kb", |b| {
        b.iter(|| {
            let _ = dispatch_syscall(
                black_box(state.clone()),
                SystemCall::Mmap { size: 4096 },
                2,
            );
        });
    });
}

fn bench_scheduler_enqueue(c: &mut Criterion) {
    c.bench_function("scheduler_enqueue_10", |b| {
        b.iter(|| {
            let mut sched = Scheduler::new();
            for i in 1..=10 {
                sched.enqueue(i);
            }
            black_box(&sched);
        });
    });
}

fn bench_scheduler_schedule(c: &mut Criterion) {
    let mut sched = Scheduler::new();
    for i in 1..=10 {
        sched.enqueue(i);
    }
    c.bench_function("scheduler_round_robin", |b| {
        b.iter(|| {
            let mut s = sched.clone();
            for _ in 0..10 {
                black_box(s.schedule());
            }
        });
    });
}

fn bench_virtual_memory_create(c: &mut Criterion) {
    c.bench_function("virtual_memory_new", |b| {
        b.iter(|| {
            black_box(VirtualMemory::new());
        });
    });
}

fn bench_memory_layout_region_lookup(c: &mut Criterion) {
    let layout = MemoryLayout::default();
    c.bench_function("memory_layout_region_lookup", |b| {
        b.iter(|| {
            black_box(layout.region_for_address(black_box(0x0000_0000_3000_1000)));
        });
    });
}

fn bench_page_table_entry_create(c: &mut Criterion) {
    c.bench_function("page_table_entry_create", |b| {
        b.iter(|| {
            black_box(PageTableEntry::new(
                black_box(42),
                PagePermissions::read_write(),
            ));
        });
    });
}

fn bench_page_permissions(c: &mut Criterion) {
    c.bench_function("page_permissions_check", |b| {
        let perm = PagePermissions::read_write();
        b.iter(|| {
            black_box(perm.allows_read());
            black_box(perm.allows_write());
            black_box(perm.allows_execute());
        });
    });
}

fn bench_syscall_dispatch_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("syscall_dispatch");
    let state = make_kernel_with_process();
    let syscalls = vec![
        ("getpid", SystemCall::GetPid),
        ("fork", SystemCall::Fork),
        ("mmap", SystemCall::Mmap { size: 4096 }),
    ];
    for (name, syscall) in syscalls {
        group.bench_with_input(BenchmarkId::new("op", name), &syscall, |b, syscall| {
            b.iter(|| {
                let _ = dispatch_syscall(black_box(state.clone()), syscall.clone(), 2);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_syscall_getpid,
    bench_syscall_fork,
    bench_syscall_mmap,
    bench_scheduler_enqueue,
    bench_scheduler_schedule,
    bench_virtual_memory_create,
    bench_memory_layout_region_lookup,
    bench_page_table_entry_create,
    bench_page_permissions,
    bench_syscall_dispatch_scaling,
);
criterion_main!(benches);
