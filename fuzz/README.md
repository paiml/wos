# WOS Fuzz Testing

Fuzz testing infrastructure for WOS using libFuzzer via cargo-fuzz.

## Overview

Fuzz testing automatically generates random inputs to find bugs, crashes, and edge cases that traditional tests might miss.

## Setup

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Or use the Makefile
make fuzz-install
```

## Fuzz Targets

### 1. `fuzz_syscall_dispatch`

Tests syscall dispatcher with random SystemCall inputs.

**What it tests:**
- Syscall deserialization from random JSON
- Dispatch logic with malformed inputs
- Error handling for invalid syscalls

**Run:**
```bash
make fuzz-syscalls
# Or: cargo fuzz run fuzz_syscall_dispatch
```

### 2. `fuzz_process_creation`

Tests process lifecycle (fork, exit, waitpid).

**What it tests:**
- Random fork/exit patterns
- Parent-child relationships with random operations
- Process state transitions
- Orphan process handling

**Run:**
```bash
make fuzz-processes
# Or: cargo fuzz run fuzz_process_creation
```

### 3. `fuzz_memory_allocation`

Tests memory management (mmap, munmap).

**What it tests:**
- Random allocation sizes
- Random permission combinations
- Fragmentation patterns
- Allocation/deallocation cycles

**Run:**
```bash
make fuzz-memory
# Or: cargo fuzz run fuzz_memory_allocation
```

### 4. `fuzz_scheduler`

Tests scheduler with random operation sequences.

**What it tests:**
- Add/remove/schedule/yield operations
- Queue invariants (no process loss)
- Round-robin fairness under stress
- Edge cases (empty queue, single process, etc.)

**Run:**
```bash
make fuzz-scheduler
# Or: cargo fuzz run fuzz_scheduler
```

## Running All Fuzz Tests

```bash
# Run all targets for 60 seconds each
make fuzz

# Run indefinitely (Ctrl+C to stop)
cargo fuzz run fuzz_syscall_dispatch &
cargo fuzz run fuzz_process_creation &
cargo fuzz run fuzz_memory_allocation &
cargo fuzz run fuzz_scheduler &
```

## Analyzing Results

### Crashes

If a crash is found, libFuzzer saves it to:
```
fuzz/artifacts/fuzz_<target>/<crash_file>
```

Reproduce crash:
```bash
cargo fuzz run fuzz_syscall_dispatch fuzz/artifacts/fuzz_syscall_dispatch/crash-abc123
```

### Coverage

Generate coverage report:
```bash
make fuzz-coverage

# Or manually:
cargo fuzz coverage fuzz_syscall_dispatch
```

### Statistics

Fuzzer shows real-time statistics:
```
#1234567 NEW    cov: 89 ft: 156 corp: 45/12kb exec/s: 12345 rss: 45Mb
#       └─ Iteration number
         └─ NEW: new coverage found
             └─ cov: edge coverage count
                 └─ ft: feature count
                     └─ corp: corpus size
                         └─ exec/s: executions per second
                             └─ rss: memory usage
```

## Integration with CI

Add to `.github/workflows/fuzz.yml`:

```yaml
name: Fuzz Testing

on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Run fuzz tests
        run: make fuzz
        timeout-minutes: 60

      - name: Upload artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-artifacts
          path: fuzz/artifacts/
```

## Best Practices

### 1. Limit Fuzzer Input

Prevent resource exhaustion:

```rust
fuzz_target!(|data: &[u8]| {
    // Limit input size
    if data.len() > 1024 {
        return;
    }

    // Limit iterations
    let iterations = (data[0] % 20) as usize;  // Max 20
});
```

### 2. Check Invariants

Verify system properties:

```rust
fuzz_target!(|data: &[u8]| {
    // ... perform operations ...

    // Invariant: total processes = queue + current
    let total = scheduler.ready_queue.len()
        + if scheduler.current_pid.is_some() { 1 } else { 0 };
    assert!(total <= max_processes);
});
```

### 3. Use Structured Input

For complex inputs, use `arbitrary` crate:

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    operation: SchedulerOp,
    pid: u32,
    count: u8,
}

fuzz_target!(|input: FuzzInput| {
    // Use structured input
});
```

## Troubleshooting

### Fuzzer Hangs

If fuzzer appears stuck:
- Check for infinite loops in code
- Verify test completes quickly (no I/O, no sleep)
- Use `-max_total_time` to limit runtime

### Low Coverage

If coverage doesn't increase:
- Add more interesting seed inputs to `corpus/`
- Simplify fuzz target to focus on specific code paths
- Check if code is reachable from fuzz entry point

### Memory Issues

If fuzzer uses too much memory:
- Limit input size
- Reduce iteration count
- Use `-rss_limit_mb` flag

## Corpus Management

### Seed Corpus

Add interesting inputs to `corpus/<target>/`:

```bash
# Example: Add valid syscall JSON
echo '{"GetPid":null}' > fuzz/corpus/fuzz_syscall_dispatch/seed1.json
```

### Minimize Corpus

Remove redundant inputs:

```bash
cargo fuzz cmin fuzz_syscall_dispatch
```

### Merge Corpora

Combine multiple runs:

```bash
cargo fuzz cmin --merge fuzz_syscall_dispatch
```

## Performance Tips

### Optimize for Speed

```bash
# Use release mode for faster execution
cargo fuzz run --release fuzz_syscall_dispatch

# Increase job count (parallel fuzzing)
cargo fuzz run fuzz_syscall_dispatch -- -jobs=8
```

### Dictionary

Add common patterns to help fuzzer:

```bash
# Create dictionary file
cat > fuzz/dict/syscalls.dict <<EOF
"GetPid"
"Fork"
"Exit"
"Mmap"
"Munmap"
EOF

# Use dictionary
cargo fuzz run fuzz_syscall_dispatch -- -dict=fuzz/dict/syscalls.dict
```

## Continuous Fuzzing

### OSS-Fuzz Integration

WOS can be integrated with OSS-Fuzz for continuous fuzzing:

1. Add project to OSS-Fuzz
2. Configure build script
3. Receive daily reports

See: https://google.github.io/oss-fuzz/

## Cleanup

```bash
# Remove all fuzzing artifacts
make fuzz-clean

# Or manually
cargo fuzz clean
rm -rf fuzz/corpus/
rm -rf fuzz/artifacts/
```

## References

- [cargo-fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing Rust Code](https://rust-fuzz.github.io/book/)
- [AFL vs libFuzzer](https://aflplus.plus/docs/fuzzing_in_depth/)
