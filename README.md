# WOS - WebAssembly Operating System

[![TDG Grade](https://img.shields.io/badge/TDG%20Grade-A%2B%20(96--97%25)-brightgreen)](PROGRESS.md)
[![Test Coverage](https://img.shields.io/badge/Coverage-94.11%25-brightgreen)](PROGRESS.md)
[![Mutation Score](https://img.shields.io/badge/Mutation%20Score-98.5%25-brightgreen)](PROGRESS.md)
[![Tests](https://img.shields.io/badge/Tests-22%2C320%20passing-brightgreen)](PROGRESS.md)

An educational microkernel operating system written in pure Rust that compiles to WebAssembly. WOS demonstrates OS concepts (processes, memory management, file systems, IPC) in a safe, testable environment that runs directly in your browser.

## Features

- **🦀 Pure Rust**: 100% safe Rust with `#![forbid(unsafe_code)]` - zero undefined behavior
- **🌐 Browser-Native**: Compiles to WASM, runs in any modern browser without plugins
- **🧪 Elite Testing**: 22,320 tests with 94% coverage and 98.5% mutation score
- **⚡ Functional Design**: Pure functional syscalls with immutable state transitions
- **🔍 Time-Travel Debugging**: Bidirectional execution replay with full state snapshots
- **📊 Quality Metrics**: Built-in TDG (Test-Driven Grade) dashboard with A+ rating

## Quick Start

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Deno 1.37+ (for frontend testing/linting)
- Make (optional, for convenient commands)

### Build and Run

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build WASM binary and generate JS bindings
make wasm

# Start local development server
make serve

# Open browser to http://localhost:8000/dist/wos/
```

### Testing

```bash
# Run all tests (Rust + Frontend + E2E)
make test-all

# Run Rust tests only (277 tests)
make test

# Run frontend tests (22,043 tests)
make test-frontend-all

# Run property tests (22,000 test cases)
make test-frontend-property

# Run E2E tests across all browsers
make e2e

# Generate coverage report
make coverage

# Run mutation testing
make mutants

# Run benchmarks
make bench-all
```

## Architecture

### Microkernel Design

WOS follows a classic microkernel architecture with minimal kernel functionality:

- **Kernel**: Process scheduling, memory management, syscall dispatch, IPC
- **Userspace**: Init process (PID 1), shell, user programs (echo, ls, ps)
- **File System**: Virtual file system (VFS) with dynamic ProcFS
- **IPC**: Message passing with FIFO ordering

### Key Technologies

- **Rust**: Systems programming language with memory safety
- **WebAssembly**: Binary format for safe, portable execution
- **wasm-bindgen**: Rust/JavaScript interop layer
- **im-rs**: Persistent data structures for O(1) cloning
- **Deno**: Modern JavaScript runtime for frontend testing

### Persistent Data Structures

WOS uses persistent (immutable) data structures via `im-rs` for efficient state management:

```rust
// Clone is O(1) thanks to structural sharing
let new_state = old_state.clone();

// Modifications create new versions without copying
let updated_state = new_state.update(|s| { /* changes */ });
```

### Pure Functional Syscalls

All syscalls follow a pure functional pattern:

```rust
pub fn dispatch_syscall(
    state: KernelState,
    syscall: SystemCall,
    calling_pid: ProcessId,
) -> Result<(KernelState, SyscallOutput), KernelError>
```

State flows in, new state + output flow out. No global state, no side effects.

## System Calls

WOS implements 11 core syscalls:

| Syscall | Description |
|---------|-------------|
| `GetPid` | Get current process ID |
| `Fork` | Create child process |
| `Exit` | Terminate process |
| `WaitPid` | Wait for child termination |
| `Open` | Open file descriptor |
| `Close` | Close file descriptor |
| `Read` | Read from file descriptor |
| `Write` | Write to file descriptor |
| `Mmap` | Map memory pages |
| `Munmap` | Unmap memory pages |
| `Send` / `Recv` | Message-passing IPC |

## Project Structure

```
wos/
├── kernel/          # Core microkernel (159 tests)
│   ├── state.rs     # Process and kernel state types
│   ├── scheduler.rs # Round-robin scheduler
│   ├── memory.rs    # Virtual memory management
│   ├── syscall.rs   # System call implementations
│   └── trace.rs     # Time-travel debugging
├── shared/          # Shared types (17 tests)
│   ├── vfs.rs       # Virtual file system
│   └── context.rs   # Execution context
├── userspace/       # User programs (45 tests)
│   ├── init.rs      # PID 1 init process
│   ├── shell.rs     # Interactive shell
│   └── programs.rs  # User programs (echo, ls, ps)
├── wos/             # WASM bindings (56 tests)
│   ├── lib.rs       # wasm-bindgen wrapper
│   └── quality.rs   # TDG quality metrics
├── dist/wos/        # Frontend (22,043 tests)
│   ├── index.html   # Terminal UI
│   ├── app.js       # Frontend logic
│   ├── app.test.ts  # Unit tests (43)
│   ├── app.property.test.ts  # Property tests (22,000)
│   └── app.bench.ts # Benchmarks (39)
├── e2e/             # End-to-end tests (29 tests)
└── fuzz/            # Fuzz testing (4 targets)
```

## Testing Philosophy

WOS follows extreme Test-Driven Development (TDD):

### Test Coverage

- **94.11%** Rust backend coverage
- **22,320** total tests
- **98.5%** mutation score (411 mutants)

### Test Types

1. **Unit Tests**: 277 Rust + 43 Frontend = 320 tests
2. **Property Tests**: 42 Rust + 22 Frontend = 64 properties generating 22,000+ test cases
3. **Integration Tests**: Syscall pipelines, fork/wait workflows
4. **E2E Tests**: 29 cross-browser tests (Chromium, Firefox, WebKit)
5. **Fuzz Tests**: 4 targets for crash resistance
6. **Mutation Tests**: 411 mutants with 98.5% detection rate
7. **Benchmarks**: 26 Rust + 39 Frontend = 65 performance benchmarks

### Quality Metrics (TDG)

WOS includes a Test-Driven Grade (TDG) system:

```bash
# View quality metrics in browser
make wasm && make serve

# Export quality reports
# - JSON: machine-readable metrics
# - HTML: visual dashboard
# - Markdown: documentation
# - SARIF: GitHub integration
```

**Current TDG Grade: A+ (96-97%)**

## Development

### Pre-commit Hooks

Install quality gates that run before each commit:

```bash
make hooks-install
```

All commits must pass:
- ✅ Code formatting (`cargo fmt`)
- ✅ Clippy lints (zero warnings)
- ✅ Unit tests (277 tests)
- ✅ Fast quality gate (<30s)

### Makefile Commands

```bash
make help            # Show all available commands
make build           # Build all crates
make wasm            # Build WASM binary
make test            # Run Rust tests
make test-frontend   # Run frontend unit tests
make test-all        # Run all tests (Rust + Frontend + E2E)
make coverage        # Generate coverage report
make bench           # Run performance benchmarks
make quality         # Fast quality checks
make quality-complete  # Complete quality validation
make mutants         # Run mutation testing
make fuzz            # Run fuzz tests
make e2e             # Run E2E tests
```

## Browser Interface

The WOS terminal runs entirely in your browser:

- **Terminal Emulator**: Full-featured command-line interface
- **Command History**: Arrow key navigation
- **State Persistence**: localStorage for session continuity
- **Quality Dashboard**: Real-time TDG metrics display
- **Multi-Format Export**: Download quality reports (JSON/HTML/Markdown/SARIF)

### Terminal Commands

```bash
help        # Show available commands
ps          # List processes
echo <msg>  # Echo message
version     # Show WOS version
state       # Show kernel state
reset       # Reset system
```

## Performance

### Benchmarks

WOS includes comprehensive performance benchmarks:

```bash
# Run all benchmarks
make bench-all

# Run specific benchmarks
make bench-syscalls   # Syscall performance
make bench-scheduler  # Scheduler performance
make bench-memory     # Memory operations
make bench-frontend   # Frontend operations
```

Sample results:
- Syscall dispatch: ~100-500ns
- Process scheduling: ~1-5µs
- Memory allocation: ~2-10µs
- Frontend operations: 4.5µs - 3.7ms

## Educational Use

WOS is designed for teaching operating systems concepts:

### Topics Covered

1. **Process Management**: fork/exec, process trees, parent-child relationships
2. **Memory Management**: Virtual memory, page tables, permissions (R/W/X)
3. **File Systems**: VFS abstraction, file descriptors, ProcFS
4. **Inter-Process Communication**: Message passing, synchronous/asynchronous patterns
5. **Scheduling**: Round-robin algorithm, fairness, no starvation
6. **System Calls**: Kernel/user boundary, parameter validation, error handling
7. **Time-Travel Debugging**: State snapshots, execution replay

### Why WebAssembly?

- **Safety**: WASM sandbox prevents crashes/corruption
- **Accessibility**: Runs in any browser, no installation
- **Debuggability**: Full state inspection at any point
- **Testability**: Deterministic execution, reproducible tests
- **Visualization**: Browser DevTools for live inspection

## Contributing

This is an educational project. Contributions welcome!

### Guidelines

1. Maintain `#![forbid(unsafe_code)]` - no unsafe code allowed
2. Add tests for all new functionality (target: 85%+ coverage)
3. Run `make quality` before committing
4. Follow conventional commit format: `feat:`, `fix:`, `docs:`, etc.
5. Update PROGRESS.md for significant changes

## License

MIT License - see LICENSE file for details

## Links

### Project Documentation
- **Development History**: [PROGRESS.md](PROGRESS.md) - Complete development timeline and metrics
- **Architecture Decisions**: [CLAUDE.md](CLAUDE.md) - Design rationale and technical choices

### Testing Strategy Documentation (186KB)
- **Main Guide** (88KB): [Testing Strategy & Architecture](docs/specifications/testing-implementation-strategy-architecture.md)
  - Complete testing guide covering all 10 testing types
  - 80+ code examples, 5 visual diagrams
  - Tool versions, troubleshooting FAQ, bug case studies
  - Grade: A+ (99/100)

- **Canary Testing Spec** (40KB): [WASM Canary & Functional UX Testing](docs/specifications/wasm-canary-testing-spec.md)
  - SQLite-inspired testing methodology (608:1 test-to-code ratio)
  - Four-harness framework: BCT, CVS, DTS, CES
  - 80%+ user action coverage + 100% critical path coverage
  - Complete Playwright test examples and implementation roadmap

- **Quality Review** (22KB): [Document Quality Assessment](docs/specifications/testing-strategy-document-quality-review.md)
  - Comprehensive quality analysis
  - Industry standards comparison
  - Ready for publication

- **Research Review** (32KB): [Research-Based Improvements](docs/specifications/testing-strategy-review.md)
  - 10 advanced testing techniques
  - Academic literature review
  - Performance optimization recommendations

- **Navigation Guide** (4.5KB): [How to Read the Documentation](docs/specifications/README-testing-reviews.md)
  - Helps choose which document to read first
  - Audience-specific recommendations

### External Links
- **GitHub Repository**: https://github.com/paiml/wos
- **Issue Tracker**: https://github.com/paiml/wos/issues

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [WebAssembly](https://webassembly.org/) - Portable binary format
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - Rust/JS interop
- [im-rs](https://github.com/bodil/im-rs) - Persistent data structures
- [Deno](https://deno.land/) - Modern JavaScript runtime
- [Playwright](https://playwright.dev/) - E2E testing framework

---

**WOS** - Where operating systems meet WebAssembly 🦀🌐
