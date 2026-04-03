# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tiny but functional educational OS demonstrating fundamental OS concepts in a modern, safe, and testable environment. Written entirely in Rust and compiled to WebAssembly, WOS (WASM Operating System) runs completely in the browser with no server-side infrastructure.

**Core Innovation**: Educational microkernel OS with pure functional design, deterministic execution, and extreme TDD methodology achieving <100ms cold start and <5 second setup time.

**Production Deployment**: ✅ Live at https://interactive.paiml.com/wos/
- Deployed: 2025-10-23
- Catalog integration: interactive.paiml.com homepage
- Rapid iteration workflow: symlink-based development
- Demo video: docs/webos-showcase-demo.webm

## Build Commands

```bash
# Build WASM binary for browser
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo nextest run --all-features --workspace

# Run property-based tests (10K inputs per test)
cargo nextest run --all-features proptest

# Generate coverage report
make coverage
# Opens: target/coverage/html/index.html

# Run mutation tests (90%+ kill rate required)
cargo mutants --workspace --json --output mutants.json

# Run all quality gates
make quality

# Build optimized WASM with wasm-bindgen
make wasm
# Outputs to: dist/wos/wos_bg.wasm, wos.js

# Start development server with hot reload (REQUIRED: Use ruchy serve ONLY)
ruchy serve dist/wos --port 8000 --watch --watch-wasm --verbose
# Open: http://127.0.0.1:8000/
# Features: Auto-reload on file changes, WASM auto-compilation, graceful shutdown
```

## Local HTTP Server Policy (CRITICAL - STOP THE LINE)

**MANDATORY**: This project uses `ruchy serve` for ALL local development.

**FORBIDDEN**: The following commands are BANNED across the entire project:
- ❌ `python -m http.server`
- ❌ `python3 -m http.server`
- ❌ `npx http-server`
- ❌ `npx serve`
- ❌ Any other HTTP server tool

**ENFORCEMENT**:
If you encounter ANY reference to Python http.server or npx-based servers:
1. **STOP THE LINE** immediately
2. Replace with `ruchy serve`
3. Report the violation in commit message
4. If `ruchy` is not installed, file bug report upstream

**Installation** (if needed):
```bash
# Install ruchy from source
cargo install ruchy

# Verify installation
ruchy --version
```

**Why ruchy serve?**
- ✅ **12.13x faster** than Python http.server (empirically validated)
- ✅ **WASM-optimized** with automatic COOP/COEP headers for SharedArrayBuffer
- ✅ **Memory safe** (Rust guarantees - no segfaults)
- ✅ **Energy efficient** (16x better req/CPU% ratio)
- ✅ **Auto MIME detection** for HTML, CSS, JS, JSON, WASM files
- ✅ **Hot Reload** (v3.105.0+): File watching with debouncing (300ms default)
- ✅ **WASM Auto-Compilation**: .ruchy files → .wasm on save
- ✅ **Graceful Shutdown**: Ctrl+C cleanup with PID file management
- ✅ **Network Access**: Local + network IPs for mobile/VM testing
- ✅ **Vite-Style Output**: Color-coded logging for better DX

**Usage Examples**:
```bash
# Development with hot reload (RECOMMENDED)
ruchy serve dist/wos --port 8000 --watch --watch-wasm --verbose

# Production mode (no file watching)
ruchy serve dist/wos --port 8000

# Custom port with hot reload
ruchy serve dist/wos --port 3000 --watch --watch-wasm

# Bind to all interfaces for mobile/VM testing
ruchy serve dist/wos --port 8000 --host 0.0.0.0 --watch --watch-wasm

# Quick start (default port 8080, no hot reload)
ruchy serve dist/wos
```

## Development Workflow

### PRIORITY: PMAT Quality Gates (CRITICAL)
**Current Status**: 8 quality violations blocking PMAT gates
- 5 SATD violations (TODO comments - zero tolerance policy)
- 3 complexity violations (functions exceeding max complexity 10)

**All work must pass PMAT quality gates before commit**:
- `make pmat-satd` - Zero TODO/FIXME comments
- `make pmat-complexity` - Max cyclomatic/cognitive complexity 10
- `make pmat-tdg` - Technical Debt Grade ≥ 0.90

See `quality-issues.yaml` for detailed action plan.

### Test-Driven Development (TDD)
This project follows **Extreme TDD** - no production code without failing test first:

1. **RED**: Write failing test (unit + property + mutation + E2E)
2. **GREEN**: Minimum implementation to pass
3. **REFACTOR**: Optimize while maintaining all test passes
4. **VERIFY**: Run `make quality` + `make pmat-gates`
5. **COMMIT**: Atomic commit with `[WOS-XXX]` ticket prefix
6. **PUSH**: Push to main after each completed ticket

**Testing Requirements (ALL required)**:
- Unit tests: 85%+ coverage
- Property tests: 10K inputs per test (proptest)
- Mutation tests: 90%+ kill rate
- Fuzz tests: For all parsers and input handlers
- E2E tests: Probador for browser functionality (NO Playwright/JS)

### Ticket Workflow

For each ticket in `roadmap.yaml`:

1. **Read ticket YAML** in `roadmap.yaml` (find ticket by ID)
2. **Write RED tests** - comprehensive test suite that fails
3. **Implement (GREEN)** - minimal code to pass tests
4. **REFACTOR** - optimize, document, clean up
5. **Verify quality gates** - format, clippy, tests, coverage
6. **Update documentation** - REQUIRED before commit:
   - Update `roadmap.yaml` with ticket status and completion date
   - OR update `CHANGELOG.md` with feature/fix details
   - OR update `docs/ui-roadmap.yaml` for UI changes
   - Document code line references for traceability
7. **Commit** with format: `[WOS-XXX] Brief description`
8. **Push to main**: `git push origin main`

**IMPORTANT**: Always walk off main - no branching per project guidelines.

**DOCUMENTATION ENFORCEMENT**: Pre-commit hooks will remind you to update roadmap/changelog if you modify source code without updating documentation. This prevents undocumented changes and ensures traceability.

### Quality Requirements

- **Coverage**: 85%+ line, 90%+ branch
- **Mutation Score**: 90%+ kill rate
- **Complexity**: ≤20 cyclomatic, ≤15 cognitive per function
- **SATD**: Zero TODO/FIXME comments allowed
- **Dead Code**: Zero tolerance
- **WASM Size**: <500KB uncompressed, <100KB gzipped

### Memory Safety

**100% Safe Rust**: This project forbids all unsafe code.

```rust
#![forbid(unsafe_code)]  // Enforced at crate level
```

**Verification**:
```bash
make miri-test       # Run MIRI checks
cargo test --test memory_safety --workspace
```

**Guarantees**:
- ✅ No undefined behavior
- ✅ No memory leaks
- ✅ No data races
- ✅ No buffer overflows
- ✅ No null pointer dereferences

## Architecture

### Microkernel Design

WOS implements a microkernel architecture with minimal trusted computing base:

**Microkernel (kernel/ crate, ~2000 lines)**:
- Process scheduler (round-robin)
- Memory manager (virtual address space)
- System call dispatcher
- IPC primitives (message passing)
- Hardware abstraction (virtual devices)

**User Space (userspace/ crate, ~1300 lines)**:
- Init process (PID 1)
- Shell process
- File system server
- User programs (echo, ls, ps, cat, kill)

**Shared Infrastructure (shared/ crate, ~800 lines)**:
- Virtual File System (VFS) using im-rs
- Deterministic RNG (ChaCha8)
- Simulated Clock
- Serialization helpers

### Pure Functional Design

All kernel operations follow pure functional pattern:

```rust
pub trait KernelOp {
    type State: Clone + Serialize + DeserializeOwned;
    type Context: Clone + Serialize + DeserializeOwned;
    type Input: Clone + Serialize + DeserializeOwned;
    type Output: Clone + Serialize + DeserializeOwned;
    type Error: std::error::Error;

    fn execute(
        state: Self::State,
        context: Self::Context,
        input: Self::Input
    ) -> Result<(Self::State, Self::Context, Self::Output), Self::Error>;
}
```

**Key Principle**: All state changes must be visible in the type signature. No hidden mutation.

### Process Model

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Process {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub state: ProcessState,
    pub memory: VirtualMemory,
    pub open_files: im::HashMap<FileDescriptor, FileHandle>,
    pub env: im::HashMap<String, String>,
    pub cwd: PathBuf,
    pub exit_code: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked(BlockReason),
    Terminated,
}
```

### System Call Interface

Modeled after POSIX but simplified:

```rust
pub enum SystemCall {
    // Process Management
    Fork,
    Exec { path: PathBuf, args: Vec<String> },
    Exit { code: i32 },
    WaitPid { pid: ProcessId },
    GetPid,
    Kill { pid: ProcessId, signal: Signal },

    // File I/O
    Open { path: PathBuf, flags: OpenFlags },
    Close { fd: FileDescriptor },
    Read { fd: FileDescriptor, count: usize },
    Write { fd: FileDescriptor, data: Vec<u8> },

    // IPC
    SendMessage { dest: ProcessId, message: Message },
    ReceiveMessage { timeout_ms: Option<u64> },

    // Memory
    Mmap { size: usize, flags: MmapFlags },
    Munmap { addr: usize, size: usize },
}
```

### Virtual File System

```
/
├── bin/           # User programs (echo, ls, ps, cat)
├── dev/           # Virtual devices (null, zero, random, console)
├── proc/          # Process information (/proc/PID/status, /proc/PID/maps)
│   ├── 1/         # Init process
│   ├── self/      # Current process symlink
│   └── ...
├── tmp/           # Temporary files
└── home/          # User files
```

## Project Structure

```
wos/
├── Cargo.toml              # Workspace root
├── Makefile                # Build orchestration
├── .pmat-gates.toml        # Quality enforcement
├── roadmap.yaml            # Implementation roadmap
├── CLAUDE.md               # This file
├── docs/
│   ├── specifications/
│   │   ├── wos-spec-v1.md       # Complete technical spec
│   │   └── wos-tech-review.md   # Toyota Way enhancements
│   └── tickets/            # Per-ticket documentation
├── shared/                 # Shared infrastructure (~800 lines)
│   └── src/
│       ├── lib.rs
│       ├── vfs.rs          # Virtual filesystem (im-rs)
│       └── context.rs      # Execution context
├── kernel/                 # Microkernel (~2000 lines)
│   └── src/
│       ├── lib.rs
│       ├── process.rs      # Process management
│       ├── scheduler.rs    # Round-robin scheduler
│       ├── memory.rs       # Virtual memory
│       ├── syscall.rs      # System call dispatcher
│       ├── ipc.rs          # IPC primitives
│       └── device.rs       # Virtual devices
├── userspace/              # User space (~1300 lines)
│   └── src/
│       ├── lib.rs
│       ├── init.rs         # Init process (PID 1)
│       ├── shell.rs        # Shell
│       └── programs/       # User programs
├── wos/                    # WASM entry point (~400 lines)
│   └── src/
│       ├── lib.rs          # OS integration
│       └── wasm.rs         # WASM exports
├── dist/                   # Web distribution
│   └── wos/
│       ├── index.html
│       ├── style.css
│       ├── app.js
│       ├── wos_bg.wasm     # Built by wasm-bindgen
│       └── wos.js
└── tests/                  # Tests (~2750 lines)
    ├── unit/
    ├── integration/
    ├── properties/         # Property-based tests (proptest)
    └── e2e/                # E2E tests (Probador - NO Playwright)
```

## Property-Based Testing

Every function must have property tests verifying invariants:

**Required Properties**:
- **Determinism**: Same input always produces same output
- **Referential Transparency**: Pure functions with no side effects
- **Sandbox Isolation**: State changes only from explicit operations
- **Robustness**: Must never panic on any input

**Scheduler Properties**:
- Every ready process eventually gets CPU time (no starvation)
- Process state transitions are valid
- PID uniqueness

**Memory Manager Properties**:
- Allocations never overlap
- Total allocated memory ≤ system memory
- Free followed by allocate is deterministic

**File System Properties**:
- Path resolution is deterministic
- File contents persist across operations
- Operations commute where expected

**System Call Properties**:
- Invalid inputs never panic
- Successful operations update state correctly
- Errors leave state unchanged (atomicity)

Use `proptest` with 10,000 inputs per test minimum:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn scheduler_fairness(operations: Vec<SchedulerOp>) {
        let result = run_scheduler_test(operations);
        prop_assert!(result.is_fair());
    }
}
```

## WASM Quality Gates

Automated checks in quality gate pipeline:

```rust
#[test]
fn wasm_binary_has_no_wasi_imports()        // Must pass
fn wasm_binary_size_under_threshold()       // <500KB
fn wasm_binary_compressed_size_under_threshold()  // <100KB
fn wasm_functions_under_complexity_limit()  // ≤20 per function
```

## Performance Targets

- **Cold Start**: <100ms (WASM load in browser)
- **System Call**: <10μs for simple syscalls (getpid)
- **Context Switch**: <50μs (save/restore process state)
- **Process Fork**: <100μs (clone with im-rs)
- **VFS Clone**: <10μs (O(1) persistent data structures)

## Coverage Workflow

```bash
# Generate coverage (handles mold linker compatibility)
make coverage

# Check thresholds
make coverage-check  # Fails if <85% line, <90% branch

# Open HTML report
xdg-open target/coverage/html/index.html
```

**Note**: Temporarily disables mold linker during coverage collection due to llvm-cov incompatibility.

## Mutation Testing

```bash
# Run mutation tests (~10-15min for full workspace)
cargo mutants --workspace

# Fast mode (recent changes only)
cargo mutants --workspace --in-diff HEAD~1

# Check score
make mutants-check  # Fails if <90% kill rate
```

## Debugging

**Development**: Keep DWARF symbols in debug builds
**Production**: Strip symbols from release WASM

```bash
# Inspect WASM with debug symbols
llvm-dwarfdump target/wasm32-unknown-unknown/debug/wos.wasm

# Verify WASM structure
wasm-objdump -h target/wasm32-unknown-unknown/release/wos.wasm

# Check for WASI imports (should be zero)
wasm-objdump -x target/wasm32-unknown-unknown/release/wos.wasm | grep wasi
```

### Browser Tracing System

WOS includes a comprehensive tracing system for debugging initialization and runtime issues. The tracing system is **zero-cost when disabled** (default) and can be enabled via URL parameters or localStorage.

**Quick Start**:
```bash
# Enable tracing via URL (temporary, for debugging)
http://127.0.0.1:8000/?trace=DEBUG&categories=INIT,WASM,CONFIG

# Enable tracing via localStorage (persistent across reloads)
# In browser console:
localStorage.setItem('wos-trace-level', 'DEBUG');
localStorage.setItem('wos-trace-categories', 'INIT,WASM,CONFIG');

# Disable tracing
localStorage.removeItem('wos-trace-level');
localStorage.removeItem('wos-trace-categories');

# Or via console:
window.tracer.clear();
```

**Trace Levels**:
- `NONE` - Disabled (default, zero overhead)
- `ERROR` - Only critical errors
- `WARN` - Warnings and errors
- `INFO` - Important lifecycle events
- `DEBUG` - Detailed execution flow
- `TRACE` - Verbose diagnostic output

**Trace Categories**:
- `INIT` - Application initialization
- `WASM` - WebAssembly module loading and operations
- `CONFIG` - Configuration management
- `PANEL` - Panel management and layout
- `TERMINAL` - Terminal operations
- `VIM` - Vim editor operations
- `PROCESS` - Process management
- `MEMORY` - Memory operations
- `SYSCALL` - System calls
- `FILE` - File operations
- `EVENT` - Event handling
- `RENDER` - UI rendering

**Example Output**:
```
[0.21ms] [WASM] [INFO] Calling init()
[8.17ms] [WASM] [INFO] init() completed in 7.90ms
[8.21ms] [CONFIG] [DEBUG] ConfigManager constructor called
[8.41ms] [CONFIG] [DEBUG] ConfigManager initialized {version: 0.1.0, ...}
[10.49ms] [INIT] [INFO] Status set to Ready
```

**Console API**:
```javascript
// Access tracer from console
window.tracer.setLevel('DEBUG');
window.tracer.setCategories(['INIT', 'WASM']);
window.tracer.info('INIT', 'Custom trace message', {data: 'optional'});
```

**Full Documentation**: See `docs/specifications/tracing-spec.md` for complete implementation details and integration examples.

## Browser Interface

The HTML terminal interface provides:
- Terminal-like REPL with command history
- Process list view (PIDs, state, parent-child)
- Memory map view (virtual address space)
- System call trace view (live monitoring)
- localStorage persistence (save/restore state)
- Keyboard shortcuts (↑/↓ history, Ctrl+L clear)

```bash
# Start with hot reload and WASM auto-compilation
ruchy serve dist/wos --port 8000 --watch --watch-wasm --verbose
# Open: http://127.0.0.1:8000/
# Changes to HTML/CSS/JS/WASM auto-reload in browser
```

## Development Scope

**CRITICAL**: Nothing is out of scope. NEVER leave defects or unfinished work.

**Core Principles**:
- Complete all tickets to 100% quality
- Full browser interface implementation required
- Pure WASM terminal interface (NO JavaScript)
- E2E testing with Probador (NOT Playwright)
- All features fully implemented and tested
- Zero TODOs, zero FIXMEs, zero incomplete work

**Local Development Focus**:
- `make wasm` - perfect local build
- `make serve` - development server with hot reload (12.13x faster than Python)
- `ruchy serve dist/wos --port 8000 --watch --watch-wasm` - manual server start with hot reload
- `http://127.0.0.1:8000/` - fully functional browser interface
- All quality gates running locally
- Perfect developer experience with auto-reload and WASM auto-compilation
- E2E tests running locally with Probador (NO Playwright/Node.js)

**Deployment (Future)**:
- S3/CloudFront deployment scripts can be added later
- CI/CD pipelines already supported via GitHub Actions
- Production builds fully functional locally first

## Pre-commit Hooks

Fast checks only (<30s):

```bash
make hooks-install
```

Installs git pre-commit hook:
- `cargo fmt --check`
- `cargo clippy --all-features`
- `cargo nextest run --lib`
- PMAT complexity analysis
- **bashrs**: Lints bash scripts and Makefiles for quality and best practices

**bashrs Integration** (Full Capabilities):
bashrs provides comprehensive shell script quality tooling:

**Core Features**:
- ✅ **Lint**: Checks shell script syntax, Makefile conventions, shellcheck violations
- ✅ **Score**: Grades scripts (A+ to F scale, 0-10.0 scoring)
- ✅ **Audit**: Comprehensive quality reports (parse, lint, test, score)
- ✅ **Test**: Bash script test framework with coverage tracking
- ✅ **Format**: Auto-format shell scripts (consistent style)
- ✅ **Purify**: Enforce determinism, idempotency, and safety patterns
- ✅ **Coverage**: Generate coverage reports for bash scripts

**Makefile Targets**:
```bash
make bashrs-check     # Fast validation (lint + score) - runs in pre-commit
make bashrs-audit     # Comprehensive audit of all scripts
make bashrs-score     # Display quality scores for all scripts
make bashrs-test      # Run bashrs test framework
make bashrs-coverage  # Generate coverage reports
make bashrs-format    # Auto-format shell scripts
make bashrs-purify    # Purify scripts (determinism + safety)
make bashrs-fix       # Auto-fix all issues (format + purify)
```

**Quality Standards**:
- **Makefile**: Target F (4.2/10) → A (9.0/10+)
- **Scripts**: Individual scores tracked, minimum B- (7.0/10) required
- **Pre-commit**: bashrs-check runs automatically on commit
- **CI/CD**: bashrs-audit runs in quality gates

**Installation** (if needed):
```bash
# Install bashrs from source
cargo install bashrs

# Verify installation
bashrs --version
```

**Bug Reports & Feature Requests**:
If you encounter bugs or need features in bashrs:
1. Open an issue at: https://github.com/paiml/bashrs
2. Include: bashrs version, OS, reproduction steps, expected vs actual behavior
3. Tag with appropriate labels: `bug`, `enhancement`, `documentation`
4. Link to the issue in your commit message when working around bashrs limitations

Slow checks (mutation, full coverage, E2E) run manually or in CI.

## Implementation Status

**Current Phase**: Specification Complete

**Next Steps**: Begin Phase 1 implementation
- WOS-001: Project setup with quality gates
- WOS-002: Kernel state types
- WOS-003: Round-robin scheduler
- WOS-004: System call dispatcher
- WOS-005: Basic process syscalls

See `roadmap.yaml` for complete 17-week implementation plan.

## Educational Goals

WOS teaches OS concepts through hands-on experimentation:

1. Run simple commands (echo, ls, ps)
2. Understand process creation (fork)
3. Explore file system (/proc, /dev)
4. Experiment with IPC (message passing)
5. Visualize memory and process state
6. Read kernel source code (only ~2000 lines!)

## References

**Specifications**:
- `docs/specifications/wos-spec-v1.md` - Complete technical specification
- `docs/specifications/wos-tech-review.md` - Toyota Way quality enhancements
- `roadmap.yaml` - Detailed implementation roadmap

**Related Projects**:
- Kerla OS: https://github.com/nuta/kerla (Rust kernel with Linux compatibility)
- WASM Labs: ../wasm-labs/ (extreme TDD methodology source)
- xv6: https://pdos.csail.mit.edu/6.828/2020/xv6.html (educational UNIX)

**Learning Resources**:
- "Operating Systems: Three Easy Pieces" - Remzi H. Arpaci-Dusseau
- "The Rust Programming Language" - Official book
- "Writing an OS in Rust" - Philipp Oppermann

## Deployment Workflow

**Production URL**: https://interactive.paiml.com/wos/

### Rapid Iteration Workflow

```bash
# One-time setup: Create symlink
cd /home/noah/src/wos
make link-dev

# Daily development
cd /home/noah/src/wos
make build  # Changes appear instantly via symlink!

# Deploy to production
cd /home/noah/src/interactive.paiml.com
make deploy  # Quality gates + E2E + S3 upload
```

### Deployment Architecture
- **Symlink**: `/home/noah/src/interactive.paiml.com/dist/wos` → `/home/noah/src/wos/dist/wos`
- **S3 Bucket**: `interactive.paiml.com-production-mces4cme`
- **CloudFront**: Distribution ID `ELY820FVFXAFF`
- **Catalog Integration**: WebOS appears on interactive.paiml.com homepage

See `docs/DEPLOYMENT-CHECKLIST.md` and `docs/RAPID-ITERATION-WORKFLOW.md` for complete guides.

## Summary

WOS is a **tiny but functional educational OS** (~5000 lines) demonstrating real OS concepts using modern, safe, and testable Rust patterns. Runs entirely in browser with zero setup, following extreme TDD methodology for maximum quality.

**Key Principles**:
- 100% safe Rust (`#![forbid(unsafe_code)]`)
- Pure functional design (no hidden state)
- Extreme TDD (85%+ coverage, 90%+ mutation score)
- Microkernel architecture (minimal TCB)
- Educational focus (simplified but correct)
- Production deployment (https://interactive.paiml.com/wos/)

## Contract-First Development

All code changes require a prior contract (provable-contract YAML or pmat work contract.json).
NEVER write code before defining the contract. Run `pmat comply check` before completing any task.
CB-1400 enforces agent contract existence; verification_level must be satisfied before merge.
