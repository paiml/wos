use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wos_kernel::{
    dispatch_syscall, sys_exit, sys_fork, sys_getpid, sys_mmap, sys_open, sys_read, sys_send,
    sys_waitpid, sys_write, KernelState, OpenFlags, PagePermissions, SyscallOutput, SystemCall,
};

fn bench_getpid(c: &mut Criterion) {
    c.bench_function("sys_getpid", |b| {
        let state = KernelState::new();
        b.iter(|| {
            let (new_state, output) = sys_getpid(black_box(state.clone()), 1).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_fork(c: &mut Criterion) {
    c.bench_function("sys_fork", |b| {
        let state = KernelState::new();
        b.iter(|| {
            let (new_state, output) = sys_fork(black_box(state.clone()), 1).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_exit(c: &mut Criterion) {
    c.bench_function("sys_exit", |b| {
        // Create a process to exit
        let state = KernelState::new();
        let (state, output) = sys_fork(state, 1).unwrap();
        let child_pid = match output {
            SyscallOutput::Pid(pid) => pid,
            _ => panic!("Expected Pid"),
        };

        b.iter(|| {
            let (new_state, output) = sys_exit(black_box(state.clone()), child_pid, 0).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_waitpid(c: &mut Criterion) {
    c.bench_function("sys_waitpid", |b| {
        // Create child and make it exit
        let state = KernelState::new();
        let (state, output) = sys_fork(state, 1).unwrap();
        let child_pid = match output {
            SyscallOutput::Pid(pid) => pid,
            _ => panic!("Expected Pid"),
        };
        let (state, _) = sys_exit(state, child_pid, 42).unwrap();

        b.iter(|| {
            let (new_state, output) = sys_waitpid(black_box(state.clone()), 1, child_pid).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_mmap(c: &mut Criterion) {
    c.bench_function("sys_mmap_4kb", |b| {
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

fn bench_open(c: &mut Criterion) {
    c.bench_function("sys_open", |b| {
        let state = KernelState::new();
        b.iter(|| {
            let (new_state, output) = sys_open(
                black_box(state.clone()),
                1,
                "/proc/1/status",
                OpenFlags::O_RDONLY,
            )
            .unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_read(c: &mut Criterion) {
    c.bench_function("sys_read_small", |b| {
        let state = KernelState::new();
        let (state, output) = sys_open(state, 1, "/proc/1/status", OpenFlags::O_RDONLY).unwrap();
        let fd = match output {
            SyscallOutput::FileDescriptor(fd) => fd,
            _ => panic!("Expected FD"),
        };

        b.iter(|| {
            let (new_state, output) = sys_read(black_box(state.clone()), 1, fd, 1024).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_write(c: &mut Criterion) {
    c.bench_function("sys_write_small", |b| {
        let state = KernelState::new();
        let data = vec![0u8; 1024];

        b.iter(|| {
            let (new_state, output) =
                sys_write(black_box(state.clone()), 1, 1, black_box(data.clone())).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_send(c: &mut Criterion) {
    c.bench_function("sys_send_small_message", |b| {
        let state = KernelState::new();
        let (state, _) = sys_fork(state, 1).unwrap(); // Create recipient
        let message = vec![0u8; 256];

        b.iter(|| {
            let (new_state, output) =
                sys_send(black_box(state.clone()), 1, 2, black_box(message.clone())).unwrap();
            black_box((new_state, output))
        });
    });
}

fn bench_dispatch(c: &mut Criterion) {
    c.bench_function("dispatch_syscall_getpid", |b| {
        let state = KernelState::new();
        let syscall = SystemCall::GetPid;
        b.iter(|| {
            let result =
                dispatch_syscall(black_box(state.clone()), black_box(syscall.clone()), 1).unwrap();
            black_box(result)
        });
    });
}

fn bench_dispatch_fork(c: &mut Criterion) {
    c.bench_function("dispatch_syscall_fork", |b| {
        let state = KernelState::new();
        let syscall = SystemCall::Fork;
        b.iter(|| {
            let result =
                dispatch_syscall(black_box(state.clone()), black_box(syscall.clone()), 1).unwrap();
            black_box(result)
        });
    });
}

criterion_group!(
    syscalls,
    bench_getpid,
    bench_fork,
    bench_exit,
    bench_waitpid,
    bench_mmap,
    bench_open,
    bench_read,
    bench_write,
    bench_send,
    bench_dispatch,
    bench_dispatch_fork,
);
criterion_main!(syscalls);
