# WOS - WebAssembly Operating System

[![TDG Grade](https://img.shields.io/badge/TDG%20Grade-A%2B%20(99.3%2F100)-brightgreen)](PROGRESS.md)
[![Test Coverage](https://img.shields.io/badge/Coverage-85%25%2B-brightgreen)](PROGRESS.md)
[![Tests](https://img.shields.io/badge/Tests-546%20unit%20%7C%20211%20E2E-brightgreen)](PROGRESS.md)
[![Quality Gates](https://img.shields.io/badge/Quality%20Gates-6%2F6%20passing-brightgreen)](Makefile)

An educational microkernel operating system written in pure Rust that compiles to WebAssembly. WOS demonstrates OS concepts (processes, memory management, file systems, IPC) in a safe, testable environment that runs directly in your browser.

## Features

- **🦀 Pure Rust**: 100% safe Rust with `#![forbid(unsafe_code)]` - zero undefined behavior
- **🌐 Browser-Native**: Compiles to WASM, runs in any modern browser without plugins
- **🧪 Elite Testing**: 452 unit tests + 147 E2E tests with 85%+ coverage
- **⚡ Functional Design**: Pure functional syscalls with immutable state transitions
- **🔍 Time-Travel Debugging**: Bidirectional execution replay with full state snapshots
- **📊 Quality Metrics**: Real-time TDG dashboard (99.3/100 A+) with JSON/HTML export

## Installation

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

# Start development server with hot reload
make serve

# Open browser to http://localhost:8000/dist/wos/
# Changes to HTML/CSS/JS/WASM automatically reload in browser
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

# Run canary tests (SQLite-inspired critical workflow validation)
make canary          # Fast: Chromium only (~2-3 min)
make canary-all      # Comprehensive: All browsers (~15-20 min)

# Generate coverage report
make coverage

# Run mutation testing
make mutants

# Run benchmarks
make bench-all
```

## Usage

Once WOS is running in your browser (after `make serve`), you can interact with the terminal:

### Basic Commands

```bash
help        # Show available commands
ps          # List processes
ls [path]   # List files
cat <file>  # Display file contents
echo <msg>  # Echo message
grep <pat>  # Search for pattern (supports stdin)
wc [file]   # Count lines/words/bytes (supports stdin)
touch <f>   # Create empty file
mkdir <dir> # Create directory
rm <file>   # Remove file
vim [file]  # Open vim editor (MVP)
version     # Show WOS version
state       # Show kernel state
reset       # Reset system
```

### Advanced Features

```bash
# Pipeline operators
cmd1 | cmd2         # Pipe stdout to stdin
cmd1 && cmd2        # Execute cmd2 if cmd1 succeeds
cmd1 || cmd2        # Execute cmd2 if cmd1 fails
cmd1 ; cmd2         # Execute both regardless

# I/O redirection
cmd > file          # Redirect stdout to file (overwrite)
cmd >> file         # Redirect stdout to file (append)
cmd < file          # Redirect file to stdin

# Variables
VAR=value           # Set variable
echo $VAR           # Expand variable
export VAR=value    # Export variable
echo $?             # Last exit code
```

For complete command documentation, see the [Terminal Commands](#terminal-commands) section below.

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
- **ruchy serve** (v3.105.0+): High-performance development server (12.13x faster than Python http.server)
  - Hot reload with file watching (300ms debounce)
  - WASM auto-compilation (.ruchy → .wasm)
  - Graceful shutdown with PID file management
  - Network access for mobile/VM testing
  - Vite-style colored output

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
- **23,376** total tests (546 unit + 22,000 property + 211 E2E scenarios + 107 canary + 411 mutation + 101 other)
- **98.5%** mutation score (411 mutants)

### Test Types

1. **Unit Tests**: 546 Rust + Frontend tests
2. **Property Tests**: 42 Rust + 22 Frontend = 64 properties generating 22,000+ test cases
3. **Integration Tests**: Syscall pipelines, fork/wait workflows
4. **E2E Tests**: 211 test scenarios across 5 browsers (Chromium, Firefox, WebKit, Mobile Chrome, Mobile Safari) = 1,055 total tests
5. **Canary Tests**: 107 SQLite-inspired critical workflow tests
6. **Fuzz Tests**: 4 targets for crash resistance
7. **Mutation Tests**: 411 mutants with 98.5% detection rate
8. **Benchmarks**: 26 Rust + 39 Frontend = 65 performance benchmarks

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

### Browser Tracing System

WOS includes a comprehensive tracing system for debugging initialization and runtime issues. The tracing system is **zero-cost when disabled** (default) and can be enabled via URL parameters or localStorage.

**Quick Start**:
```bash
# Enable tracing via URL (temporary, for debugging)
http://localhost:8000/dist/wos/?trace=DEBUG&categories=INIT,WASM,CONFIG

# Or in E2E tests
await page.goto('index.html?trace=DEBUG&categories=INIT,WASM,CONFIG');
```

**Trace Levels**: NONE (default), ERROR, WARN, INFO, DEBUG, TRACE

**Trace Categories**: INIT, WASM, CONFIG, PANEL, TERMINAL, EDITOR, SHELL, FILESYSTEM, MONITOR, IPC

**Example Output**:
```
[10.29ms] [INIT] [INFO] Application initialization started
[12.45ms] [WASM] [INFO] Calling init()
[185.67ms] [WASM] [INFO] init() completed in 173.22ms
[187.12ms] [CONFIG] [DEBUG] Loading configuration
[189.45ms] [INIT] [INFO] Initialization complete in 179.16ms
```

**Console API**:
```javascript
// Enable tracing from browser console
window.tracer.setLevel('DEBUG');
window.tracer.enableCategory('INIT');
window.tracer.disableCategory('WASM');
```

See [Tracing Specification](docs/specifications/tracing-spec.md) for complete details.

### Makefile Commands

```bash
make help            # Show all available commands
make build           # Build all crates
make wasm            # Build WASM binary
make serve           # Start development server with hot reload
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

- **Terminal Emulator**: Full-featured command-line interface with history (↑/↓)
- **File Manager**: Upload, create, edit, download files via browser UI
- **Vim Editor**: Modal text editor with syntax highlighting (MVP)
- **State Persistence**: localStorage for session continuity
- **Quality Dashboard**: Real-time TDG metrics (grade, score, tests, coverage)
- **Export Reports**: Download quality reports in JSON/HTML formats

### Terminal Commands

```bash
help        # Show available commands
ps          # List processes
ls [path]   # List files
cat <file>  # Display file contents
echo <msg>  # Echo message
grep <pat>  # Search for pattern (supports stdin)
wc [file]   # Count lines/words/bytes (supports stdin)
touch <f>   # Create empty file
mkdir <dir> # Create directory
rm <file>   # Remove file
vim [file]  # Open vim editor (MVP)
version     # Show WOS version
state       # Show kernel state
reset       # Reset system

# Pipeline operators
cmd1 | cmd2         # Pipe stdout to stdin
cmd1 && cmd2        # Execute cmd2 if cmd1 succeeds
cmd1 || cmd2        # Execute cmd2 if cmd1 fails
cmd1 ; cmd2         # Execute both regardless

# I/O redirection
cmd > file          # Redirect stdout to file (overwrite)
cmd >> file         # Redirect stdout to file (append)
cmd < file          # Redirect file to stdin

# Variables
VAR=value           # Set variable
echo $VAR           # Expand variable
export VAR=value    # Export variable
echo $?             # Last exit code
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

### Architecture & Design Specifications (229KB)
- **WOS Specification v1.0** (44KB): [Project Specification](docs/specifications/wos-spec-v1.md)
  - Complete project vision and goals
  - Implementation phases and development workflow
  - Quality standards and performance targets
  - Extreme TDD methodology (85%+ coverage, 90%+ mutation score)

- **Architecture Components** (32KB): [Architectural Specification](docs/specifications/wos-arch-spec.md)
  - Pure functional microkernel design patterns
  - L4-inspired IPC with message-passing semantics
  - Process scheduler and memory management architecture
  - VFS and ProcFS implementation strategies

- **Technical Review** (56KB): [Toyota Way Quality Framework](docs/specifications/wos-tech-review.md)
  - Technical architecture assessment (4.5/5 rating)
  - Toyota Production System principles for OS development
  - Technical Debt Grading (TDG) integration
  - Pre-implementation enhancement recommendations

- **Enhanced Features v2.0** (97KB): [Enhanced UI/UX Specification](docs/specifications/wos-enhanced-features-spec.md)
  - Research-backed Integrated Learning Environment (ILE) design
  - Monaco editor integration for accessibility
  - Visual System Monitor (Toyota Mieruka principle)
  - Time-travel debugging UI controls
  - Progressive disclosure for cognitive load reduction
  - WCAG 2.1 Level AA accessibility compliance
  - Constructivist learning theory integration
  - 4-week implementation roadmap (Phase 10)

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
- [ruchy](https://github.com/paiml/ruchy) - High-performance development server with hot reload

---

**WOS** - Where operating systems meet WebAssembly 🦀🌐
