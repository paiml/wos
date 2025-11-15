# WOS Tutorials

Hands-on tutorials for learning WOS development.

## Tutorial Series

### Beginner

1. **[Adding a New Syscall](01-adding-syscall.md)** - 45 minutes
   - Learn TDD with WOS
   - Implement `sys_getppid`
   - Write unit and property tests
   - Update documentation

2. **[Creating a New Program](02-creating-program.md)** - 60 minutes
   - Build a `tree` userspace program
   - Parse command-line arguments
   - Use syscalls from userspace
   - Integration testing

### Intermediate

3. **[Understanding the Scheduler](03-understanding-scheduler.md)** - 90 minutes
   - Deep dive into round-robin scheduling
   - Implement optimizations (priority, aging, MLFQ)
   - Property testing for fairness
   - CPU benchmarking

### Advanced

4. **Implementing Pipes** (Coming Soon)
   - Inter-process communication
   - Pipe syscalls
   - Shell pipelines

5. **Advanced Memory Management** (Coming Soon)
   - Page table internals
   - Copy-on-write
   - Memory mapped files

6. **Building a Shell** (Coming Soon)
   - Command parsing
   - Process management
   - Job control

## Prerequisites

- Rust programming experience
- Basic operating systems knowledge
- WOS development environment set up

## Getting Started

```bash
# Clone repository
git clone https://github.com/paiml/wos
cd wos

# Run tests
make test

# Build WASM
make wasm

# Start development server
make serve
```

## Learning Path

```
┌─────────────────────────────────────────────────────────────┐
│                    WOS Learning Path                         │
└─────────────────────────────────────────────────────────────┘

1. Read Documentation
   ├── README.md           (Overview)
   ├── ARCHITECTURE.md     (System design)
   └── API.md              (API reference)

2. Beginner Tutorials
   ├── 01-adding-syscall.md       (Kernel development)
   └── 02-creating-program.md     (Userspace development)

3. Intermediate Tutorials
   └── 03-understanding-scheduler.md (Scheduling internals)

4. Advanced Tutorials
   └── 06-building-shell.md

5. Contribute
   └── CONTRIBUTING.md     (Contribution guidelines)
```

## Tutorial Format

Each tutorial follows this structure:

1. **Prerequisites** - Required knowledge and completed tutorials
2. **Goal** - What you'll build
3. **Step-by-step instructions** - Detailed implementation guide
4. **Tests** - Comprehensive testing approach
5. **Summary** - Key takeaways
6. **Exercise** - Practice implementing a similar feature
7. **Common Pitfalls** - Mistakes to avoid
8. **Further Reading** - Additional resources

## Code Examples

All tutorial code is tested and ready to run:

```rust
// Example: Adding a syscall
pub fn sys_getppid(
    state: KernelState,
    calling_pid: ProcessId,
) -> Result<(KernelState, SyscallOutput), KernelError> {
    let process = state
        .processes
        .get(&calling_pid)
        .ok_or(KernelError::ProcessNotFound)?;

    let ppid = process.parent_pid.unwrap_or(0);
    Ok((state, SyscallOutput::Pid(ppid)))
}
```

## Testing Philosophy

WOS follows **extreme TDD**:

- ✅ Write tests before implementation
- ✅ Aim for 85%+ code coverage
- ✅ Property tests for invariants
- ✅ Integration tests for workflows
- ✅ Mutation testing for test quality

Example test:

```rust
#[test]
fn test_getppid_success() {
    let state = KernelState::new();

    // Fork to create parent-child relationship
    let (state, output) = sys_fork(state, 1).unwrap();
    let child_pid = match output {
        SyscallOutput::Pid(pid) => pid,
        _ => panic!(),
    };

    // Get parent of child
    let (_, output) = sys_getppid(state, child_pid).unwrap();

    // Assert parent is init (PID 1)
    assert_eq!(output, SyscallOutput::Pid(1));
}
```

## Quality Standards

All tutorial code meets these standards:

- **Coverage**: 85%+ line coverage
- **Complexity**: <15 cyclomatic complexity per function
- **Documentation**: All public APIs documented
- **Tests**: 5+ tests per feature
- **No SATD**: No "TODO", "FIXME", "HACK" comments
- **No Unsafe**: `#![forbid(unsafe_code)]`

## Getting Help

- **Issues**: GitHub Issues (check repository)
- **Discussions**: GitHub Discussions (check repository)
- **Documentation**: [docs/](../)

## Contributing

Found an error? Want to add a tutorial?

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Additional Resources

### Documentation
- [API Reference](../API.md)
- [Architecture Guide](../ARCHITECTURE.md)
- [Performance Guide](../PERFORMANCE.md)
- [Specification](../specifications/wos-spec-v1.md)

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Operating Systems: Three Easy Pieces](https://pages.cs.wisc.edu/~remzi/OSTEP/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [Property Testing with Proptest](https://altsysrq.github.io/proptest-book/)

## Roadmap

Upcoming tutorials:

- [ ] Tutorial 4: Implementing Pipes
- [ ] Tutorial 5: Advanced Memory Management
- [ ] Tutorial 6: Building a Shell
- [ ] Tutorial 7: File System Implementation
- [ ] Tutorial 8: Debugging with Time-Travel
- [ ] Tutorial 9: Performance Optimization
- [ ] Tutorial 10: WASM Integration

## License

All tutorial code is licensed under MIT License.
