# WOS Specification v1.0
## WASM Operating System - Educational Microkernel

**Project**: Rust + WASM Operating System for Teaching
**Inspired By**: Kerla OS, Starina OS, Operating System in 1000 Lines
**Quality Standard**: WASM Labs Extreme TDD/Quality Methodology
**Author**: Noah Gift
**Date**: 2025-10-14
**Status**: Specification Phase

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Core Innovation](#2-core-innovation)
3. [Architecture](#3-architecture)
4. [Quality Standards](#4-quality-standards)
5. [Development Workflow](#5-development-workflow)
6. [Implementation Phases](#6-implementation-phases)
7. [Technical Details](#7-technical-details)
8. [Testing Strategy](#8-testing-strategy)
9. [Performance Targets](#9-performance-targets)
10. [Future Enhancements](#10-future-enhancements)

---

## 1. Project Overview

### 1.1 Vision

WOS (WASM Operating System) is a **tiny but functional** educational operating system that demonstrates fundamental OS concepts in a modern, safe, and testable environment. Written entirely in Rust and compiled to WebAssembly, WOS runs completely in the browser with no server-side infrastructure.

### 1.2 Goals

- **Educational**: Teach OS concepts (process management, memory management, system calls, IPC)
- **Functional**: Actually works, not just a toy or demo
- **Tiny**: Small codebase (~5000 lines of Rust) with minimal complexity
- **Safe**: 100% safe Rust with `#![forbid(unsafe_code)]`
- **Quality**: Extreme TDD methodology with 85%+ coverage, 90%+ mutation score
- **Modern**: Rust + WASM + functional design patterns

### 1.3 Non-Goals

- **NOT** a production OS
- **NOT** Linux-compatible (no binary compatibility requirement)
- **NOT** targeting real hardware (WASM-only)
- **NO** deployment infrastructure (local development only in MVP)

### 1.4 Inspiration Sources

**Kerla OS** (3.4k stars):
- Monolithic kernel in Rust
- Process management, system calls, signals
- File system abstraction
- Good reference for Rust OS patterns

**Starina OS** (400 stars):
- Modern experimental microkernel
- Demonstrates contemporary OS design

**Operating System in 1000 Lines** (3k stars):
- Simplicity and teaching-focused
- Minimal but complete implementation

---

## 2. Core Innovation

### 2.1 Unique Value Proposition

```
Pure Functional OS Design → Deterministic Execution → Browser-Based Learning
```

**What Makes WOS Different:**

1. **Pure Functional Kernel**: All state transitions explicit, no hidden mutation
2. **Runs in Browser**: Zero setup, instant experimentation
3. **Extreme TDD**: Every OS component is tested with property-based tests
4. **Educational Focus**: Simplified concepts without sacrificing correctness
5. **Microkernel Design**: Minimal kernel, most services as user-space processes

### 2.2 Key Differentiators

| Feature | Traditional OS | WOS |
|---------|---------------|-----|
| **Language** | C/C++ | 100% Safe Rust |
| **Execution** | Hardware | Browser (WASM) |
| **State Management** | Global mutable state | Pure functional |
| **Testing** | Limited | 85%+ coverage, property tests |
| **Setup Time** | Hours (toolchain, VM) | <5 seconds (open browser) |
| **Safety** | Unsafe, UB possible | Memory safe, no UB |
| **Debugging** | GDB, printf | Browser DevTools, time-travel |

---

## 3. Architecture

### 3.1 System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser Environment                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              WOS Terminal (HTML/JS)                   │  │
│  │  - Command input                                      │  │
│  │  - Output rendering                                   │  │
│  │  - Process list view                                  │  │
│  │  - Memory map view                                    │  │
│  └────────────────────┬──────────────────────────────────┘  │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────────────┐ │
│  │          WOS WASM Binary (bash_os.wasm)                │ │
│  │                                                         │ │
│  │  ┌───────────────────────────────────────────────┐    │ │
│  │  │         Microkernel (< 2000 lines)            │    │ │
│  │  │  - Process scheduler                          │    │ │
│  │  │  - Memory manager (virtual)                   │    │ │
│  │  │  - System call dispatcher                     │    │ │
│  │  │  - IPC primitives                             │    │ │
│  │  │  - Hardware abstraction (virtual devices)     │    │ │
│  │  └───────────────────────────────────────────────┘    │ │
│  │                                                         │ │
│  │  ┌───────────────────────────────────────────────┐    │ │
│  │  │      User Space Processes (~3000 lines)       │    │ │
│  │  │  - Init process (PID 1)                       │    │ │
│  │  │  - Shell process                              │    │ │
│  │  │  - File system server                         │    │ │
│  │  │  - User programs (echo, ls, ps, cat, etc.)    │    │ │
│  │  └───────────────────────────────────────────────┘    │ │
│  │                                                         │ │
│  │  ┌───────────────────────────────────────────────┐    │ │
│  │  │   Shared Infrastructure (~1000 lines)          │    │ │
│  │  │  - Virtual File System (VFS) using im-rs      │    │ │
│  │  │  - Deterministic RNG (ChaCha8)                │    │ │
│  │  │  - Simulated Clock                            │    │ │
│  │  │  - Serialization (serde)                      │    │ │
│  │  └───────────────────────────────────────────────┘    │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Microkernel Design

**Microkernel Responsibilities (Minimal Trusted Computing Base):**

1. **Process Scheduling**
   - Round-robin scheduler
   - Process state: Ready, Running, Blocked, Terminated
   - Context switching

2. **Memory Management**
   - Virtual address space per process (simulated)
   - Page table management (simulated, no real MMU)
   - Heap allocation tracking

3. **System Call Dispatcher**
   - System call entry point
   - Argument validation
   - Capability checking

4. **IPC Primitives**
   - Message passing between processes
   - Synchronous send/receive
   - Shared memory regions

5. **Hardware Abstraction**
   - Virtual devices (console, disk, network)
   - Interrupt simulation
   - Device driver interface

**User Space Services (Untrusted):**

- File system server (VFS implementation)
- Network stack (future)
- Shell and user programs
- Device drivers (as user processes)

### 3.3 Pure Functional Design

All kernel operations follow the pure functional pattern from WASM Labs:

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

**Key Invariants:**
- No global mutable state
- All state changes visible in type signatures
- Deterministic execution (same input → same output)
- Referential transparency

### 3.4 Process Model

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BlockReason {
    WaitingForMessage(ProcessId),
    WaitingForChild,
    WaitingForIO,
}
```

### 3.5 Memory Model

**Virtual Memory (Simulated, No Real Paging):**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualMemory {
    pub pages: im::HashMap<VirtualPage, PhysicalPage>,
    pub heap_start: usize,
    pub heap_size: usize,
    pub stack_start: usize,
    pub stack_size: usize,
}

// Typical process memory layout (virtual addresses)
// 0x00000000 - 0x00400000: Code segment (read-only)
// 0x00400000 - 0x00800000: Data segment (read-write)
// 0x00800000 - 0x10000000: Heap (grows up)
// 0xC0000000 - 0xFFFFFFFF: Stack (grows down)
```

### 3.6 System Call Interface

**System Calls (Modeled after POSIX, simplified):**

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

pub struct SystemCallResult {
    pub return_value: i64,
    pub errno: Option<Errno>,
}
```

### 3.7 File System

**Virtual File System (Building on WASM Labs VFS):**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileSystem {
    pub root: VirtualFileSystem,  // From wasm-labs-shared
    pub mount_points: im::HashMap<PathBuf, MountPoint>,
}

pub enum MountPoint {
    RootFS,
    ProcFS,   // Process information (/proc)
    DevFS,    // Virtual devices (/dev)
    TmpFS,    // Temporary files (/tmp)
}
```

**File System Layout:**

```
/
├── bin/           # User programs (echo, ls, ps, cat, etc.)
├── dev/           # Virtual devices (null, zero, random, console)
├── proc/          # Process information
│   ├── 1/         # Process 1 (init)
│   │   ├── status
│   │   ├── cmdline
│   │   └── maps
│   └── self/      # Current process symlink
├── tmp/           # Temporary files
└── home/          # User files
```

---

## 4. Quality Standards

### 4.1 WASM Labs Extreme TDD Methodology

**From WASM Labs CLAUDE.md:**

- **Coverage**: 85%+ line, 90%+ branch
- **Mutation Score**: 90%+ kill rate
- **Complexity**: ≤20 cyclomatic, ≤15 cognitive per function
- **SATD**: Zero TODO/FIXME comments allowed
- **Dead Code**: Zero tolerance
- **WASM Size**: <500KB uncompressed, <100KB gzipped
- **100% Safe Rust**: `#![forbid(unsafe_code)]` enforced at crate level

### 4.2 Quality Gates (3-Level System)

**Fast Gate (<30s) - Pre-commit:**
- `cargo fmt --check`
- `cargo clippy --all-features`
- `cargo test --lib`
- PMAT complexity analysis

**Complete Gate (~5min) - Pre-push:**
- Fast gate +
- Full test suite (unit + integration)
- Coverage report (≥85%)
- WASM binary size check
- Deno web quality (HTML/CSS/JS linting)

**Extreme Gate (~10-15min) - Pre-deploy:**
- Complete gate +
- Mutation testing (≥90% kill rate)
- Property-based tests (10K inputs per test)
- E2E tests (Playwright, 3 browsers)
- PMAT deep WASM inspection
- Security audit

### 4.3 Memory Safety Verification

**Zero Unsafe Code:**

```rust
// Enforced at crate level
#![forbid(unsafe_code)]
```

**MIRI Checks:**

```bash
# Run MIRI for undefined behavior detection
cargo +nightly miri test --all-features --workspace
```

**Guarantees:**
- No undefined behavior
- No memory leaks
- No data races
- No buffer overflows
- No null pointer dereferences

### 4.4 Property-Based Testing

**Required Properties for OS Components:**

1. **Process Scheduler**
   - Property: Every ready process eventually gets CPU time (no starvation)
   - Property: Process state transitions are valid (no invalid transitions)
   - Property: PID uniqueness (no duplicate PIDs)

2. **Memory Manager**
   - Property: Allocations never overlap
   - Property: Free followed by allocate is deterministic
   - Property: Total allocated memory ≤ system memory

3. **File System**
   - Property: Operations commute where expected (mkdir then touch == touch then mkdir for different paths)
   - Property: Path resolution is deterministic
   - Property: File contents persist across operations

4. **System Calls**
   - Property: Invalid inputs never panic
   - Property: Successful operations update state correctly
   - Property: Errors leave state unchanged (atomicity)

**Proptest Configuration:**

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn scheduler_fairness(operations: Vec<SchedulerOp>) {
        // Test with 10,000 random operation sequences
    }
}
```

---

## 5. Development Workflow

### 5.1 Extreme TDD Cycle

**For Each Feature:**

1. **RED Phase**: Write failing tests
   - Unit tests for individual functions
   - Integration tests for component interaction
   - Property tests for invariants
   - E2E tests for user workflows

2. **GREEN Phase**: Minimal implementation to pass
   - Write simplest code that passes tests
   - No premature optimization
   - Focus on correctness first

3. **REFACTOR Phase**: Improve and optimize
   - Extract functions/modules
   - Add property tests
   - Optimize hot paths (with benchmarks)
   - Document complex logic

4. **COMMIT Phase**: Atomic commit
   - Format: `[WOS-TICKET-ID] Brief description`
   - Include tests, implementation, documentation
   - Push to main (no branching per CLAUDE.md)

### 5.2 Ticket Workflow

**Structure:**

```yaml
ticket_id: WOS-001
title: "Implement round-robin process scheduler"
description: |
  Implement a basic round-robin scheduler that switches between
  ready processes in a fair manner.
acceptance_criteria:
  - Process switching works correctly
  - No process starvation
  - Context switches preserve process state
  - Property tests verify fairness
test_count_target: 15
complexity_max: 15
completion_status: not_started
```

**Process:**
1. Create ticket YAML in `docs/tickets/`
2. Write RED tests
3. Implement (GREEN)
4. Refactor and optimize
5. Verify quality gates
6. Commit with ticket ID
7. Push to main

### 5.3 Build Commands

**Makefile Targets:**

```bash
# Build WASM binary for browser
make build-wos

# Run tests
make test-wos

# Run property-based tests (10K inputs per test)
make test-props-wos

# Generate coverage report
make coverage-wos
# Opens: target/coverage/html/index.html

# Run mutation tests (90%+ kill rate required)
make mutants-wos

# Run all quality gates
make quality-complete-wos

# Build optimized WASM for distribution
make wasm-wos
# Outputs to: dist/wos/wos.wasm, wos.js

# Start local development server
make serve-wos
# Opens: http://localhost:8000/wos/

# Clean build artifacts
make clean-wos
```

---

## 6. Implementation Phases

### 6.1 Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic kernel with process management

**Tickets:**

**WOS-001: Project Setup**
- Create workspace structure
- Set up Cargo.toml with workspace
- Configure WASM target
- Add quality tooling (cargo-llvm-cov, cargo-mutants, nextest)
- Create Makefile with build targets
- Set up .pmat-gates.toml
- Tests: 5 (build smoke tests)

**WOS-002: Kernel State Types**
- Define `KernelState` struct with process table
- Define `Process` struct
- Define `ProcessState` enum
- Implement Clone + Serialize + Deserialize
- Tests: 10 (serialization round-trip)

**WOS-003: Process Scheduler (Round-Robin)**
- Implement scheduler data structure (queue of ready processes)
- Implement `schedule()` function (select next process)
- Implement `context_switch()` (save/restore process state)
- Tests: 20 (unit tests + 5 property tests)
- Property: No starvation (every ready process runs)

**WOS-004: System Call Dispatcher**
- Implement system call enum
- Implement dispatcher (match on syscall type)
- Implement validation (check arguments)
- Tests: 15 (each syscall type + error cases)

**WOS-005: Basic Process Syscalls**
- Implement `sys_getpid()`
- Implement `sys_fork()` (create child process)
- Implement `sys_exit()` (terminate process)
- Implement `sys_waitpid()` (wait for child)
- Tests: 25 (each syscall + interactions)
- Property: PID uniqueness, parent-child relationship

**Phase 1 Deliverables:**
- Working kernel with process management
- ~1000 lines of kernel code
- ~500 lines of tests
- 85%+ coverage, property tests for scheduler

### 6.2 Phase 2: Memory Management (Weeks 3-4)

**Goal**: Virtual memory and heap management

**WOS-006: Virtual Memory Structures**
- Define `VirtualMemory` struct
- Define page table (using im::HashMap)
- Implement address translation (virtual → physical)
- Tests: 15 (address translation, edge cases)

**WOS-007: Memory Allocation**
- Implement `sys_mmap()` (allocate virtual pages)
- Implement `sys_munmap()` (free virtual pages)
- Track allocations per process
- Tests: 20 (allocation, deallocation, errors)
- Property: No overlapping allocations

**WOS-008: Memory Protection**
- Implement page permissions (read/write/execute)
- Validate syscall access to memory
- Tests: 15 (permission checks, violations)

**Phase 2 Deliverables:**
- Virtual memory system
- ~800 lines of kernel code
- ~400 lines of tests
- Property tests for allocation invariants

### 6.3 Phase 3: File System (Weeks 5-6)

**Goal**: VFS and file I/O

**WOS-009: Extend WASM Labs VFS**
- Integrate `wasm-labs-shared` VFS
- Add file descriptor table per process
- Implement `sys_open()`, `sys_close()`
- Tests: 20 (open/close, fd management)

**WOS-010: File I/O Operations**
- Implement `sys_read()` (read from fd)
- Implement `sys_write()` (write to fd)
- Implement standard streams (stdin, stdout, stderr)
- Tests: 25 (read/write, pipes, edge cases)

**WOS-011: Special File Systems**
- Implement ProcFS (`/proc`)
- Implement DevFS (`/dev/null`, `/dev/zero`, `/dev/random`)
- Tests: 20 (special files, read/write behavior)

**Phase 3 Deliverables:**
- Working file system
- ~600 lines of FS code
- ~450 lines of tests
- Integration tests for file operations

### 6.4 Phase 4: IPC (Weeks 7-8)

**Goal**: Inter-process communication

**WOS-012: Message Passing**
- Define `Message` type
- Implement `sys_send()` (send message to PID)
- Implement `sys_recv()` (receive message, blocking)
- Tests: 20 (send/recv, blocking, errors)
- Property: Message ordering (FIFO)

**WOS-013: Shared Memory**
- Implement shared memory regions
- Map shared region into multiple processes
- Tests: 15 (creation, mapping, access)

**WOS-014: Synchronization Primitives**
- Implement semaphores (using message passing)
- Implement mutexes (using semaphores)
- Tests: 20 (lock/unlock, contention)

**Phase 4 Deliverables:**
- IPC mechanisms
- ~500 lines of IPC code
- ~350 lines of tests
- Property tests for message ordering

### 6.5 Phase 5: User Space (Weeks 9-10)

**Goal**: Init process, shell, user programs

**WOS-015: Init Process (PID 1)**
- Implement init process (launches shell)
- Handle orphaned processes (reparent to init)
- Tests: 15 (init behavior, reparenting)

**WOS-016: Shell Process**
- Implement simple shell (parse commands, fork/exec)
- Built-in commands (cd, exit, help)
- External commands (exec user programs)
- Tests: 25 (command parsing, execution)

**WOS-017: User Programs**
- Implement `echo` (print arguments)
- Implement `ls` (list files)
- Implement `ps` (list processes)
- Implement `cat` (read file)
- Implement `kill` (send signal to process)
- Tests: 30 (each program + integration)

**Phase 5 Deliverables:**
- User space environment
- ~800 lines of user code
- ~450 lines of tests
- E2E tests for shell interaction

### 6.6 Phase 6: Browser Interface (Weeks 11-12)

**Goal**: HTML terminal, WASM integration, E2E tests

**WOS-018: WASM Bindings**
- Implement WASM exports with wasm-bindgen
- JavaScript wrapper (load WASM, call functions)
- Tests: 10 (WASM exports, JS integration)

**WOS-019: HTML Terminal**
- Create terminal UI (HTML/CSS)
- Input handling (keyboard, enter, Ctrl+C)
- Output rendering (stdout, stderr, colors)
- Tests: 20 (E2E with Playwright)

**WOS-020: Session Persistence**
- Save kernel state to localStorage
- Restore state on page load
- Reset button (clear state)
- Tests: 15 (persistence, restore)

**WOS-021: Process/Memory Visualization**
- Process list view (PIDs, state, parent)
- Memory map view (allocations, permissions)
- System call trace view
- Tests: 10 (E2E, UI rendering)

**Phase 6 Deliverables:**
- Browser terminal interface
- ~600 lines of web code (HTML/CSS/JS)
- ~350 lines of E2E tests (Playwright)
- Complete MVP ready for use

### 6.7 Total Implementation Estimate

**Code:**
- Kernel: ~2000 lines (core + scheduler + memory + syscalls)
- User space: ~1300 lines (shell + programs)
- Shared infrastructure: ~800 lines (VFS, serialization)
- WASM bindings: ~400 lines
- Web UI: ~600 lines (HTML/CSS/JS)
- **Total: ~5100 lines of code**

**Tests:**
- Unit tests: ~1500 lines
- Integration tests: ~500 lines
- Property tests: ~400 lines
- E2E tests: ~350 lines
- **Total: ~2750 lines of tests**

**Timeline:**
- 12 weeks (assuming 10-15 hours/week)
- 21 tickets total
- ~1-2 tickets per week

---

## 7. Technical Details

### 7.1 Rust Crate Structure

```
wos/
├── Cargo.toml              # Workspace root
├── Makefile                # Build system
├── .pmat-gates.toml        # Quality gates
├── docs/
│   ├── specifications/
│   │   └── wos-spec-v1.md
│   └── tickets/
│       ├── WOS-001.yaml
│       └── ...
├── shared/                 # Shared infrastructure
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── vfs.rs          # Virtual file system (from wasm-labs)
│       └── context.rs      # Execution context
├── kernel/                 # Microkernel
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Kernel entry point
│       ├── process.rs      # Process management
│       ├── scheduler.rs    # Round-robin scheduler
│       ├── memory.rs       # Virtual memory
│       ├── syscall.rs      # System call dispatcher
│       ├── ipc.rs          # IPC primitives
│       └── device.rs       # Virtual devices
├── userspace/              # User programs
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── init.rs         # Init process (PID 1)
│       ├── shell.rs        # Shell
│       └── programs/       # User programs
│           ├── echo.rs
│           ├── ls.rs
│           ├── ps.rs
│           ├── cat.rs
│           └── kill.rs
├── wos/                    # Main entry point (integrates kernel + userspace)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # OS entry point
│       └── wasm.rs         # WASM exports
├── dist/                   # Web distribution
│   └── wos/
│       ├── index.html      # Terminal UI
│       ├── style.css
│       ├── app.js
│       ├── wos.wasm        # Built by wasm-bindgen
│       └── wos.js
└── tests/                  # Integration and E2E tests
    ├── integration/
    │   ├── process_tests.rs
    │   ├── memory_tests.rs
    │   ├── filesystem_tests.rs
    │   └── ipc_tests.rs
    └── e2e/
        └── terminal.spec.js  # Playwright tests
```

### 7.2 Cargo Dependencies

**Kernel & Shared:**

```toml
[dependencies]
# Data structures (persistent, O(1) clone)
im = "15.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Deterministic RNG
rand = "0.8"
rand_chacha = "0.3"

# Error handling
thiserror = "1.0"

# WASM
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console"] }

[dev-dependencies]
# Testing
proptest = "1.4"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

**Quality Tools:**

```toml
# In workspace Cargo.toml
[dev-dependencies]
cargo-nextest = "0.9"
cargo-llvm-cov = "0.5"
cargo-mutants = "24.1"
```

### 7.3 Key Algorithms

**Round-Robin Scheduler:**

```rust
pub fn schedule(state: &KernelState) -> Option<ProcessId> {
    let ready_processes: Vec<ProcessId> = state
        .processes
        .iter()
        .filter(|(_, p)| p.state == ProcessState::Ready)
        .map(|(pid, _)| *pid)
        .collect();

    if ready_processes.is_empty() {
        return None;
    }

    // Find current process in ready queue
    let current_idx = state.current_process
        .and_then(|pid| ready_processes.iter().position(|&p| p == pid))
        .unwrap_or(0);

    // Next process (wrap around)
    let next_idx = (current_idx + 1) % ready_processes.len();
    Some(ready_processes[next_idx])
}
```

**Virtual Memory Allocation:**

```rust
pub fn allocate_pages(
    memory: &VirtualMemory,
    size: usize,
) -> Result<VirtualAddress, MemoryError> {
    let page_count = (size + PAGE_SIZE - 1) / PAGE_SIZE;

    // Find contiguous free virtual pages
    let start_page = find_free_pages(memory, page_count)?;

    // Allocate physical pages and create mapping
    let mut new_pages = memory.pages.clone();
    for i in 0..page_count {
        let vpage = start_page + i;
        let ppage = allocate_physical_page()?;
        new_pages.insert(vpage, ppage);
    }

    Ok(VirtualAddress(start_page * PAGE_SIZE))
}
```

**System Call Dispatch:**

```rust
pub fn dispatch_syscall(
    state: KernelState,
    context: KernelContext,
    syscall: SystemCall,
) -> Result<(KernelState, KernelContext, SystemCallResult), KernelError> {
    match syscall {
        SystemCall::GetPid => sys_getpid(state, context),
        SystemCall::Fork => sys_fork(state, context),
        SystemCall::Exit { code } => sys_exit(state, context, code),
        SystemCall::WaitPid { pid } => sys_waitpid(state, context, pid),
        SystemCall::Open { path, flags } => sys_open(state, context, path, flags),
        SystemCall::Read { fd, count } => sys_read(state, context, fd, count),
        SystemCall::Write { fd, data } => sys_write(state, context, fd, data),
        SystemCall::SendMessage { dest, message } =>
            sys_send(state, context, dest, message),
        // ... more syscalls
    }
}
```

---

## 8. Testing Strategy

### 8.1 Testing Pyramid

```
         /\
        /  \       E2E Tests (Playwright)
       /____\      - Full user workflows
      /      \     - Browser interaction
     /        \    - ~50 test cases
    /__________\
   /            \  Integration Tests
  /              \ - Multi-component
 /                \- Process + Memory + FS
/____________________\
|                    | Unit Tests
|  Property Tests    | - Individual functions
|  - 10K inputs      | - ~200 test cases
|  - Invariants      |
|____________________|
```

### 8.2 Test Categories

**Unit Tests (~200 tests):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_selects_ready_process() {
        let state = create_test_state_with_processes(3);
        let next_pid = schedule(&state).unwrap();
        assert!(state.processes[&next_pid].state == ProcessState::Ready);
    }

    #[test]
    fn test_fork_creates_child_process() {
        let (new_state, _, result) = sys_fork(state, context).unwrap();
        assert_eq!(new_state.processes.len(), state.processes.len() + 1);
    }
}
```

**Property Tests (~50 tests, 10K inputs each):**

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn scheduler_fairness(operations: Vec<SchedulerOp>) {
        let mut cpu_time = HashMap::new();
        let mut state = initial_state();

        for op in operations {
            state = execute_op(state, op);
            if let Some(pid) = state.current_process {
                *cpu_time.entry(pid).or_insert(0) += 1;
            }
        }

        // Property: All ready processes get approximately equal CPU time
        let times: Vec<u32> = cpu_time.values().copied().collect();
        let max = times.iter().max().unwrap();
        let min = times.iter().min().unwrap();
        prop_assert!(max - min <= times.len() as u32);
    }

    #[test]
    fn memory_no_overlaps(allocations: Vec<(usize, usize)>) {
        let mut memory = VirtualMemory::new();
        let mut allocated = Vec::new();

        for (size, _) in allocations {
            if let Ok(addr) = allocate_pages(&memory, size) {
                allocated.push((addr, size));
                memory = memory.with_allocation(addr, size);
            }
        }

        // Property: No two allocations overlap
        for i in 0..allocated.len() {
            for j in (i+1)..allocated.len() {
                let (addr1, size1) = allocated[i];
                let (addr2, size2) = allocated[j];
                prop_assert!(!ranges_overlap(addr1, size1, addr2, size2));
            }
        }
    }
}
```

**Integration Tests (~30 tests):**

```rust
#[test]
fn test_fork_exec_wait_pipeline() {
    // Test full process lifecycle
    let state = initial_kernel_state();

    // Parent forks
    let (state, _, fork_result) = sys_fork(state, context).unwrap();
    let child_pid = fork_result.child_pid.unwrap();

    // Child execs
    let state = if state.current_process == Some(child_pid) {
        let (state, _, _) = sys_exec(
            state,
            context,
            PathBuf::from("/bin/echo"),
            vec!["hello".to_string()]
        ).unwrap();
        state
    } else {
        state
    };

    // Parent waits
    let (state, _, wait_result) = sys_waitpid(state, context, child_pid).unwrap();
    assert_eq!(wait_result.exit_code, Some(0));
    assert_eq!(state.processes[&child_pid].state, ProcessState::Terminated);
}
```

**E2E Tests (Playwright, ~50 test cases):**

```javascript
// tests/e2e/terminal.spec.js
const { test, expect } = require('@playwright/test');

test('user can run commands in terminal', async ({ page }) => {
  await page.goto('http://localhost:8000/wos/');

  // Type and submit command
  await page.fill('#terminal-input', 'echo hello world');
  await page.press('#terminal-input', 'Enter');

  // Check output
  const output = await page.textContent('#terminal-output');
  expect(output).toContain('hello world');
});

test('ps command lists running processes', async ({ page }) => {
  await page.goto('http://localhost:8000/wos/');

  await page.fill('#terminal-input', 'ps');
  await page.press('#terminal-input', 'Enter');

  const output = await page.textContent('#terminal-output');
  expect(output).toContain('PID');
  expect(output).toContain('init');  // PID 1
  expect(output).toContain('shell'); // Current shell
});

test('fork creates child process visible in ps', async ({ page }) => {
  await page.goto('http://localhost:8000/wos/');

  // Run command that forks
  await page.fill('#terminal-input', 'sleep 1 &');
  await page.press('#terminal-input', 'Enter');

  await page.fill('#terminal-input', 'ps');
  await page.press('#terminal-input', 'Enter');

  const output = await page.textContent('#terminal-output');
  expect(output).toContain('sleep'); // Background process
});
```

### 8.3 Test Coverage Requirements

**Minimum Coverage by Component:**

| Component | Line Coverage | Branch Coverage | Mutation Score |
|-----------|---------------|-----------------|----------------|
| Kernel (scheduler, syscalls) | 90% | 95% | 90% |
| Memory Management | 85% | 90% | 85% |
| File System | 85% | 90% | 85% |
| IPC | 85% | 90% | 85% |
| User Space | 80% | 85% | 80% |
| **Overall** | **85%** | **90%** | **90%** |

### 8.4 Mutation Testing Focus

**Critical Mutations to Catch:**

1. **Boundary Conditions**: Off-by-one errors in loops
2. **Comparison Operators**: `==` vs `!=`, `<` vs `<=`
3. **Logic Operators**: `&&` vs `||`, negations
4. **Return Values**: Swapping success/error returns
5. **State Updates**: Forgetting to update state fields

**Example Mutation:**

```rust
// Original
if process.state == ProcessState::Ready {
    schedule_process(process);
}

// Mutant (should be caught by tests!)
if process.state != ProcessState::Ready {  // Changed == to !=
    schedule_process(process);
}
```

If this mutant survives, it means tests don't adequately verify scheduler behavior.

---

## 9. Performance Targets

### 9.1 WASM Binary Size

- **Uncompressed**: <500KB (kernel + user space)
- **Gzipped**: <100KB
- **Cold Start**: <100ms (WASM load + initialization in browser)

### 9.2 Operation Performance

| Operation | Target | Notes |
|-----------|--------|-------|
| System call dispatch | <10μs | Simple syscalls (getpid) |
| Context switch | <50μs | Save/restore process state |
| Process fork | <100μs | Copy process state (using im-rs) |
| Memory allocation | <10μs | Virtual page allocation |
| File read (1KB) | <50μs | VFS read |
| Message send | <20μs | IPC message passing |

### 9.3 Scalability Targets

- **Processes**: Support 100 concurrent processes
- **Memory**: 16MB total virtual memory (simulated)
- **Files**: 1000 files in VFS
- **Messages**: 10,000 pending IPC messages

### 9.4 Benchmarking

**Criterion Benchmarks:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_context_switch(c: &mut Criterion) {
    let state = create_benchmark_state(10); // 10 processes

    c.bench_function("context_switch", |b| {
        b.iter(|| {
            let next_pid = schedule(&state).unwrap();
            context_switch(black_box(&state), black_box(next_pid))
        });
    });
}

fn bench_sys_fork(c: &mut Criterion) {
    let (state, context) = create_benchmark_state_single_process();

    c.bench_function("sys_fork", |b| {
        b.iter(|| {
            sys_fork(black_box(state.clone()), black_box(context.clone()))
        });
    });
}

criterion_group!(benches, bench_context_switch, bench_sys_fork);
criterion_main!(benches);
```

**Performance Regression Detection:**

- Run benchmarks on every commit (CI)
- Alert if performance degrades >10%
- Store historical benchmark data

---

## 10. Future Enhancements

### 10.1 Phase 7+: Advanced Features (Post-MVP)

**Not in MVP, but documented for future:**

**WOS-022: Priority Scheduling**
- Replace round-robin with priority-based scheduler
- Dynamic priority adjustment
- Real-time scheduling classes

**WOS-023: Preemptive Multitasking**
- Timer interrupts (simulated)
- Preempt long-running processes
- Time slice per process

**WOS-024: Virtual Memory Paging**
- Implement page replacement algorithm (LRU)
- Handle page faults
- Demand paging

**WOS-025: Network Stack**
- TCP/IP implementation (using smoltcp)
- Sockets API
- Loopback and virtual network devices

**WOS-026: Exec System Call**
- Replace process image
- Load ELF binaries (simplified format)
- Command-line argument passing

**WOS-027: Signals**
- POSIX-like signal handling
- Signal masks
- Signal handlers

**WOS-028: Pipes and Redirection**
- Anonymous pipes
- Named pipes (FIFOs)
- Shell redirection (>, <, |)

**WOS-029: Multi-core Support**
- Simulate multiple CPUs
- Per-CPU run queues
- Load balancing

**WOS-030: Debugging Tools**
- Process debugger (gdb-like interface)
- System call tracing (strace-like)
- Memory inspector

### 10.2 Educational Enhancements

**Interactive Visualizations:**

1. **Process State Diagram**
   - Animated transitions (Ready → Running → Blocked)
   - Click process to inspect state

2. **Memory Map Viewer**
   - Visual representation of virtual address space
   - Color-coded regions (code, data, heap, stack)
   - Click region to see contents

3. **System Call Tracer**
   - Live feed of syscalls
   - Filter by process
   - Show arguments and return values

4. **Scheduler Timeline**
   - Gantt chart of process execution
   - Time slice visualization
   - Context switch points

**Guided Tutorials:**

1. **Tutorial 1: Your First Process**
   - Run `ps` to see processes
   - Understand init, shell processes
   - Run `echo hello` and see new process created

2. **Tutorial 2: Process Lifecycle**
   - Understand fork/exec/wait
   - Create a background process
   - Kill a process with signals

3. **Tutorial 3: File I/O**
   - Create a file with `echo hello > file.txt`
   - Read with `cat file.txt`
   - Explore `/proc` and `/dev`

4. **Tutorial 4: Inter-Process Communication**
   - Send messages between processes
   - Use shared memory
   - Synchronize with semaphores

### 10.3 Deployment (Post-MVP)

**Only after MVP is complete and tested:**

- S3 + CloudFront static hosting
- CI/CD pipeline (GitHub Actions)
- Automated deployment on main branch push
- Versioned releases

---

## Appendix A: Comparison with Related Projects

### A.1 WOS vs Kerla OS

| Aspect | Kerla OS | WOS |
|--------|----------|-----|
| **Goal** | Linux binary compatibility | Educational OS |
| **Kernel Type** | Monolithic | Microkernel |
| **Target** | x86_64 hardware, QEMU | Browser (WASM) |
| **Size** | Large (~15K lines) | Small (~5K lines) |
| **Complexity** | High (full syscall compat) | Low (simplified) |
| **Safety** | Rust with some unsafe | 100% safe Rust |
| **Testing** | Limited | Extreme TDD (85%+ coverage) |

**Lesson from Kerla**: Use Rust patterns for OS primitives, but simplify scope dramatically for education.

### A.2 WOS vs xv6

| Aspect | xv6 | WOS |
|--------|-----|-----|
| **Language** | C | Safe Rust |
| **Target** | RISC-V hardware | Browser (WASM) |
| **Teaching** | University OS course | Self-paced learning |
| **Setup** | Toolchain, QEMU | Just open browser |
| **Safety** | Unsafe (C) | Memory safe |
| **State** | Global mutable | Pure functional |

**Lesson from xv6**: Keep concepts simple and well-documented for learners.

### A.3 WOS vs WASM Labs

| Aspect | WASM Labs | WOS |
|--------|-----------|-----|
| **Focus** | Language labs (Bash, Python, Rust) | Operating system |
| **Architecture** | Lab trait, pure functions | Kernel + user space |
| **Complexity** | Simple (single-step execution) | Complex (process scheduling, IPC) |
| **Quality** | Extreme TDD (85%+, 90% mutation) | Same methodology |
| **Size** | ~300 lines per lab | ~5000 lines total |

**Lesson from WASM Labs**: Apply same quality methodology and pure functional design patterns to OS concepts.

---

## Appendix B: Learning Resources

### B.1 Recommended Reading

**Operating Systems:**
- "Operating Systems: Three Easy Pieces" by Remzi H. Arpaci-Dusseau
- "Operating System Concepts" by Silberschatz, Galvin, Gagne
- "Modern Operating Systems" by Andrew Tanenbaum
- xv6 book: https://pdos.csail.mit.edu/6.828/2020/xv6/book-riscv-rev1.pdf

**Rust + OS:**
- "Writing an OS in Rust" by Philipp Oppermann: https://os.phil-opp.com/
- "The Rust Programming Language" (official book)
- "Rust for Rustaceans" by Jon Gjengset

**Microkernel Design:**
- "The Microkernel Architecture" papers
- L4 microkernel family documentation
- Minix 3 architecture

### B.2 Codebase Study Recommendations

1. **Kerla OS**: Study Rust patterns for process management, syscall dispatch
2. **xv6**: Understand simple, clean OS design
3. **Redox OS**: Microkernel in Rust (production-focused)
4. **WASM Labs**: Pure functional design, extreme TDD methodology

---

## Appendix C: FAQ

### C.1 General Questions

**Q: Why WASM instead of native?**
A: Instant setup, safety guarantees, deterministic execution, browser DevTools debugging.

**Q: Why microkernel instead of monolithic?**
A: Educational clarity (separation of concerns), easier testing, modern design pattern.

**Q: Why 100% safe Rust?**
A: Teaching memory safety, eliminates entire classes of bugs, demonstrates modern OS design.

**Q: Is this a real OS?**
A: It's a functional OS with real concepts, but simplified and running in WASM, not on hardware.

### C.2 Technical Questions

**Q: How do you implement context switching without hardware interrupts?**
A: Cooperative scheduling - processes yield control voluntarily. Non-preemptive in MVP.

**Q: How do you simulate virtual memory without an MMU?**
A: Use software page table (HashMap) and bounds checking on every memory access.

**Q: How fast is it compared to native OS?**
A: Much slower (interpreted), but fast enough for interactive learning (<100ms commands).

**Q: Can it run real programs?**
A: Only programs written specifically for WOS. No binary compatibility with Linux/Windows.

### C.3 Development Questions

**Q: How long to implement MVP?**
A: ~12 weeks at 10-15 hours/week (120-180 hours total) for one developer.

**Q: Can I contribute?**
A: Yes! After MVP is complete. Follow WASM Labs contributing guidelines.

**Q: What skills are needed?**
A: Rust programming, basic OS concepts, testing (TDD), WASM basics.

---

## Appendix D: Glossary

**Microkernel**: Minimal kernel with most services in user space
**System Call**: Request from user program to kernel
**Context Switch**: Saving state of one process, loading state of another
**Virtual Memory**: Abstraction that gives each process its own address space
**IPC**: Inter-Process Communication (messages, shared memory)
**ProcFS**: Pseudo-file system exposing process information
**DevFS**: Pseudo-file system exposing virtual devices
**Round-Robin**: Scheduling algorithm that gives each process equal time
**Pure Function**: Function with no side effects, same input → same output
**Property Test**: Test that verifies an invariant over many random inputs
**Mutation Testing**: Insert bugs to verify tests catch them
**WASM**: WebAssembly, portable binary format for the web
**Deterministic**: Same inputs always produce same outputs

---

## Appendix E: Document Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-14 | Noah Gift | Initial specification |

---

**End of Specification**

**Next Steps**: Begin Phase 1 implementation with ticket WOS-001 (Project Setup).
