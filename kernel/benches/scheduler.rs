use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wos_kernel::Scheduler;

fn bench_schedule_single(c: &mut Criterion) {
    c.bench_function("schedule_single_process", |b| {
        let mut scheduler = Scheduler::new();
        scheduler = scheduler.add_process(1);

        b.iter(|| {
            let (new_scheduler, pid) = black_box(scheduler.clone()).schedule();
            black_box((new_scheduler, pid))
        });
    });
}

fn bench_schedule_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("schedule_scaling");

    for process_count in [1, 10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(process_count),
            process_count,
            |b, &count| {
                let mut scheduler = Scheduler::new();
                for pid in 1..=count {
                    scheduler = scheduler.add_process(pid);
                }

                b.iter(|| {
                    let (new_scheduler, pid) = black_box(scheduler.clone()).schedule();
                    black_box((new_scheduler, pid))
                });
            },
        );
    }
    group.finish();
}

fn bench_yield_process(c: &mut Criterion) {
    c.bench_function("yield_process", |b| {
        let mut scheduler = Scheduler::new();
        scheduler = scheduler.add_process(1);
        let (scheduler, pid) = scheduler.schedule();
        let pid = pid.unwrap();

        b.iter(|| {
            let new_scheduler = black_box(scheduler.clone()).yield_process(pid);
            black_box(new_scheduler)
        });
    });
}

fn bench_add_process(c: &mut Criterion) {
    c.bench_function("add_process", |b| {
        let scheduler = Scheduler::new();

        b.iter(|| {
            let new_scheduler = black_box(scheduler.clone()).add_process(42);
            black_box(new_scheduler)
        });
    });
}

fn bench_remove_process(c: &mut Criterion) {
    c.bench_function("remove_process", |b| {
        let mut scheduler = Scheduler::new();
        for pid in 1..=100 {
            scheduler = scheduler.add_process(pid);
        }

        b.iter(|| {
            let new_scheduler = black_box(scheduler.clone()).remove_process(50);
            black_box(new_scheduler)
        });
    });
}

fn bench_round_robin_cycle(c: &mut Criterion) {
    c.bench_function("round_robin_full_cycle_10_processes", |b| {
        let mut scheduler = Scheduler::new();
        for pid in 1..=10 {
            scheduler = scheduler.add_process(pid);
        }

        b.iter(|| {
            let mut current_scheduler = black_box(scheduler.clone());

            // Schedule all 10 processes
            for _ in 0..10 {
                let (new_scheduler, pid) = current_scheduler.schedule();
                current_scheduler = new_scheduler.yield_process(pid.unwrap());
            }

            black_box(current_scheduler)
        });
    });
}

fn bench_scheduler_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler_clone");

    for process_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(process_count),
            process_count,
            |b, &count| {
                let mut scheduler = Scheduler::new();
                for pid in 1..=count {
                    scheduler = scheduler.add_process(pid);
                }

                b.iter(|| {
                    let cloned = black_box(scheduler.clone());
                    black_box(cloned)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    scheduler,
    bench_schedule_single,
    bench_schedule_scaling,
    bench_yield_process,
    bench_add_process,
    bench_remove_process,
    bench_round_robin_cycle,
    bench_scheduler_clone,
);
criterion_main!(scheduler);
