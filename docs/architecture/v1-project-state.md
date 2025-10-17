# WOS v1.0 - Architectural Project State

**Document Version**: 1.0
**Date**: October 17, 2025
**Status**: MVP Complete (All 21 Features Implemented)
**Purpose**: Architectural reference for designing new features

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Architecture](#system-architecture)
3. [Codebase Structure](#codebase-structure)
4. [Core Components](#core-components)
5. [Data Flow & Patterns](#data-flow--patterns)
6. [Quality & Test Infrastructure](#quality--test-infrastructure)
7. [Extension Points](#extension-points)
8. [Constraints & Boundaries](#constraints--boundaries)
9. [Future Considerations](#future-considerations)

---

## 1. Executive Summary

### Project Overview

**WOS (WebAssembly Operating System)** is an educational microkernel OS written in pure Rust that compiles to WebAssembly and runs in web browsers. It demonstrates fundamental OS concepts (processes, memory management, file systems, IPC) in a safe, testable environment.

### Current State (v1.0 MVP)

- **Status**: ✅ Production Ready
- **Features**: 21/21 roadmap items complete (WOS-001 through WOS-021)
- **Test Coverage**: 440-441/443 tests passing (99.3-99.5%)
- **Code Quality**: TDG Grade A+ (95.0 score)
- **Safety**: 100% safe Rust (zero unsafe code)
- **Binary Size**: 342KB WASM (target: <500KB)

### Key Capabilities

1. **Process Management**: Fork/exec model, PID 1 init, shell, round-robin scheduling
2. **Memory Management**: Virtual memory, page tables, mmap/munmap, permissions (R/W/X)
3. **File Systems**: VFS abstraction, ProcFS, file redirection (>, >>, <)
4. **IPC**: Message passing, pipes, file descriptor duplication
5. **Shell Features**: Command chaining (|, &&, ||, ;), variables, builtins, stdin support
6. **Browser Integration**: Full terminal UI, state persistence, quality dashboard

---

## 2. System Architecture

### 2.1 Microkernel Design

WOS follows a classic microkernel architecture with minimal trusted computing base:

```
┌─────────────────────────────────────────────────────┐
│                  Browser (WASM Host)                 │
│  ┌────────────────────────────────────────────────┐ │
│  │            Frontend (HTML/CSS/JS)               │ │
│  │  - Terminal UI                                  │ │
│  │  - Event handling                               │ │
│  │  - State persistence (localStorage)             │ │
│  └────────────────────────────────────────────────┘ │
│                         ↕                            │
│  ┌────────────────────────────────────────────────┐ │
│  │           WASM Bindings (wos crate)             │ │
│  │  - wasm-bindgen exports                         │ │
│  │  - JS ↔ Rust interface                          │ │
│  │  - Quality metrics exports                      │ │
│  └────────────────────────────────────────────────┘ │
│                         ↕                            │
│  ┌────────────────────────────────────────────────┐ │
│  │          Userspace (userspace crate)            │ │
│  │  ┌──────────┬──────────┬────────────────────┐  │ │
│  │  │   Init   │  Shell   │  User Programs     │  │ │
│  │  │  (PID 1) │          │  (echo,ls,ps,cat)  │  │ │
│  │  └──────────┴──────────┴────────────────────┘  │ │
│  └────────────────────────────────────────────────┘ │
│                         ↕ (syscalls)                 │
│  ┌────────────────────────────────────────────────┐ │
│  │           Microkernel (kernel crate)            │ │
│  │  ┌─────────────────────────────────────────┐   │ │
│  │  │  Syscall Dispatcher                      │   │ │
│  │  ├─────────────────────────────────────────┤   │ │
│  │  │  Process Scheduler (Round-Robin)         │   │ │
│  │  ├─────────────────────────────────────────┤   │ │
│  │  │  Memory Manager (Virtual Memory)         │   │ │
│  │  ├─────────────────────────────────────────┤   │ │
│  │  │  IPC (Message Passing, Pipes)            │   │ │
│  │  ├─────────────────────────────────────────┤   │ │
│  │  │  Time-Travel Debugging (History/Trace)   │   │ │
│  │  └─────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────┘ │
│                         ↕                            │
│  ┌────────────────────────────────────────────────┐ │
│  │        Shared Infrastructure (shared crate)     │ │
│  │  - VFS (Virtual File System)                    │ │
│  │  - Command Parser                               │ │
│  │  - Pipeline Parser                              │ │
│  │  - Execution Context                            │ │
│  └────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### 2.2 Crate Dependency Graph

```
wos (WASM bindings)
  ├── depends on: kernel
  ├── depends on: userspace
  └── depends on: shared

userspace
  ├── depends on: kernel
  └── depends on: shared

kernel
  └── depends on: shared (VFS only)

shared
  └── no dependencies (foundation)
```

**Dependency Rules**:
- Shared crate is dependency-free (foundation layer)
- Kernel depends only on shared (minimal TCB)
- Userspace depends on kernel + shared
- WASM bindings orchestrate all layers

### 2.3 Pure Functional Design

All kernel operations follow a pure functional pattern:

```rust
pub fn dispatch_syscall(
    state: KernelState,          // Input: current state
    syscall: SystemCall,         // Input: operation
    calling_pid: ProcessId,      // Input: context
) -> Result<
    (KernelState, SyscallOutput), // Output: new state + result
    KernelError                   // Or error
>
```

**Key Principles**:
- ✅ No global mutable state
- ✅ All state changes visible in type signature
- ✅ Deterministic execution (same input → same output)
- ✅ Referential transparency
- ✅ Time-travel debugging enabled

---

## 3. Codebase Structure

### 3.1 Repository Layout

```
wos/
├── kernel/              # Microkernel (~2,000 lines)
│   ├── src/
│   │   ├── lib.rs       # Public API
│   │   ├── state.rs     # KernelState, Process types
│   │   ├── scheduler.rs # Round-robin scheduler
│   │   ├── memory.rs    # Virtual memory, page tables
│   │   ├── syscall.rs   # System call dispatcher
│   │   └── trace.rs     # Time-travel debugging
│   └── benches/         # Performance benchmarks
│
├── shared/              # Shared infrastructure (~800 lines)
│   └── src/
│       ├── lib.rs       # Public API
│       ├── vfs.rs       # Virtual file system
│       ├── parser.rs    # Command parser
│       ├── pipeline.rs  # Pipeline parser (|, &&, ||, ;)
│       └── context.rs   # Deterministic execution context
│
├── userspace/           # User programs (~1,300 lines)
│   └── src/
│       ├── lib.rs       # Public API
│       ├── init.rs      # Init process (PID 1)
│       ├── shell.rs     # Interactive shell
│       └── programs.rs  # User programs (echo, ls, ps, cat, grep, wc)
│
├── wos/                 # WASM bindings (~400 lines)
│   └── src/
│       ├── lib.rs       # wasm-bindgen exports
│       └── quality.rs   # TDG quality metrics
│
├── dist/wos/            # Web distribution
│   ├── index.html       # Terminal UI
│   ├── style.css        # Styling
│   ├── app.js           # Frontend logic
│   └── [WASM binaries]  # Generated by wasm-bindgen
│
├── e2e/                 # End-to-end tests
│   ├── tests/           # Playwright test suites
│   ├── PROJECT_STATUS.md
│   ├── BASH_COMPATIBILITY.md
│   └── FINAL_QUALITY_REPORT.md
│
├── docs/
│   ├── specifications/  # Technical specs
│   └── architecture/    # This document
│
├── roadmap.yaml         # Implementation roadmap
├── CLAUDE.md            # Development guidelines
└── README.md            # Project overview
```

### 3.2 Code Metrics

| Metric | Value |
|--------|-------|
| Total Lines (Rust) | ~10,896 |
| Kernel | ~2,000 lines |
| Userspace | ~1,300 lines |
| Shared | ~800 lines |
| WASM Bindings | ~400 lines |
| Tests | ~2,750 lines |
| Documentation | ~2,750 lines |
| Public Functions | ~18 |
| Public Types | ~37 |
| Total Tests | 443 |
| Test Pass Rate | 99.3-99.5% |

---

## 4. Core Components

### 4.1 Kernel Module (`kernel/`)

**Responsibilities**:
- Process lifecycle management
- CPU scheduling (round-robin)
- Memory management (virtual address space)
- System call dispatch
- IPC primitives
- Time-travel debugging

**Key Types**:

```rust
// Process state
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

// Kernel state (all processes)
pub struct KernelState {
    pub processes: im::HashMap<ProcessId, Process>,
    pub scheduler: Scheduler,
    pub next_pid: ProcessId,
    pub message_queue: im::HashMap<ProcessId, im::Vector<Message>>,
    pub pipe_buffers: im::HashMap<PipeId, PipeBuffer>,
}

// System calls (15 total)
pub enum SystemCall {
    GetPid, Fork, Exit(i32), WaitPid(ProcessId),
    Open { path: String, flags: u32 },
    Close { fd: u32 },
    Read { fd: u32, count: usize },
    Write { fd: u32, data: Vec<u8> },
    Pipe, Dup2 { oldfd: u32, newfd: u32 },
    Mmap { size: usize },
    Munmap { addr: u64, size: usize },
    Send { target_pid: ProcessId, data: Vec<u8> },
    Recv { timeout: u64 },
    Sleep(u64),
}
```

**Design Patterns**:
- Pure functional dispatch: `dispatch_syscall(state, syscall, pid) -> (state, output)`
- Persistent data structures (im-rs): O(1) cloning, structural sharing
- Process state machine: Ready → Running → Blocked → Terminated

### 4.2 Shared Module (`shared/`)

**Responsibilities**:
- Virtual file system (VFS)
- Command line parsing
- Pipeline parsing (operators: |, &&, ||, ;)
- File redirection parsing (>, >>, <)
- Deterministic execution context

**Key Types**:

```rust
// Virtual file system
pub struct VirtualFileSystem {
    files: im::HashMap<PathBuf, FileEntry>,
    // Persistent data structure for O(1) cloning
}

// Pipeline stages
pub struct Pipeline {
    pub stages: Vec<PipelineStage>,
}

pub struct PipelineStage {
    pub commands: Vec<Command>,
    pub operator: Operator, // Pipe, And, Or, Semicolon
}

pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub stdin_redirect: Option<Redirection>,
    pub stdout_redirect: Option<Redirection>,
}

// Execution context (deterministic RNG, clock)
pub struct ExecutionContext {
    rng_seed: u64,
    current_time: u64,
}
```

**VFS Directory Structure**:
```
/
├── bin/           # User programs
├── dev/           # Virtual devices (null, zero, random, console)
├── proc/          # Process information
│   ├── 1/         # Init process
│   │   ├── status
│   │   ├── cmdline
│   │   └── maps
│   └── self/      # Symlink to current process
├── tmp/           # Temporary files
└── home/          # User files
```

### 4.3 Userspace Module (`userspace/`)

**Responsibilities**:
- Init process (PID 1) - system initialization and orphan reaping
- Shell - interactive command execution
- User programs - echo, ls, ps, cat, grep, wc

**Key Programs**:

| Program | Purpose | Stdin Support | Features |
|---------|---------|---------------|----------|
| `init` | PID 1, launches shell, reaps orphans | ✗ | Process tree management |
| `shell` | Interactive shell | ✗ | Builtins, history, pipelines |
| `echo` | Print arguments | ✗ | Text output |
| `ls` | List files | ✗ | Directory listing |
| `ps` | List processes | ✗ | Process table |
| `cat` | Concatenate files | ✅ | File reading, stdin support |
| `grep` | Pattern matching | ✅ | Regex search, stdin support |
| `wc` | Word/line/byte count | ✅ | Text analysis, stdin support |

**Shell Builtins**:
- `cd [path]` - Change directory
- `pwd` - Print working directory
- `export VAR=value` - Set environment variable
- `exit [code]` - Exit shell
- `help` - Show help
- `history` - Show command history

### 4.4 WASM Module (`wos/`)

**Responsibilities**:
- wasm-bindgen exports for JavaScript interop
- OS orchestration (kernel + userspace)
- State serialization/deserialization
- Quality metrics exports

**Key Exports**:

```rust
#[wasm_bindgen]
pub struct WosWasm {
    state: KernelState,
    context: ExecutionContext,
}

#[wasm_bindgen]
impl WosWasm {
    // Core operations
    pub fn new() -> Self;
    pub fn executeCommand(&mut self, cmd: &str) -> String;
    pub fn executeSyscall(&mut self, json: &str) -> String;

    // State management
    pub fn getState(&self) -> String;
    pub fn setState(&mut self, json: &str);
    pub fn reset(&mut self);
    pub fn processCount(&self) -> usize;

    // Quality metrics
    pub fn getQualityMetrics(&self) -> String;
    pub fn exportQualityHtml(&self) -> String;
    pub fn exportQualitySarif(&self) -> String;
}
```

---

## 5. Data Flow & Patterns

### 5.1 Command Execution Flow

```
1. User types command in browser
   ↓
2. Frontend (app.js) calls wos.executeCommand(cmd)
   ↓
3. WASM bindings parse command (shared::parse_pipeline)
   ↓
4. For each pipeline stage:
   a. Fork process (kernel::syscall::sys_fork)
   b. Set up stdin/stdout redirections (kernel::syscall::sys_dup2)
   c. Execute program (userspace::shell::exec_external)
   d. Wait for completion (kernel::syscall::sys_waitpid)
   ↓
5. Collect output and return to frontend
   ↓
6. Frontend displays output in terminal
```

### 5.2 System Call Dispatch Pattern

```rust
// Pure functional dispatch
pub fn dispatch_syscall(
    state: KernelState,
    syscall: SystemCall,
    calling_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    match syscall {
        SystemCall::Fork => sys_fork(state, calling_pid),
        SystemCall::GetPid => sys_getpid(state, calling_pid),
        SystemCall::Exit(code) => sys_exit(state, calling_pid, code),
        // ... 12 more syscalls
    }
}

// Individual syscall implementation
fn sys_fork(
    state: KernelState,
    parent_pid: ProcessId,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // 1. Clone parent process (O(1) with im-rs)
    // 2. Assign new PID
    // 3. Update process table
    // 4. Return new state + child PID
}
```

### 5.3 Pipeline Execution Pattern

```rust
// Pipeline: cmd1 | cmd2 && cmd3 || cmd4 ; cmd5
// Parsed into stages with operators

pub fn execute_pipeline(
    pipeline: Pipeline,
    state: KernelState,
) -> (KernelState, i32) {
    let mut current_state = state;
    let mut last_exit_code = 0;

    for stage in pipeline.stages {
        match stage.operator {
            Operator::Pipe => {
                // Create pipe, set up stdin/stdout
                current_state = execute_with_pipe(current_state, stage);
            }
            Operator::And => {
                if last_exit_code == 0 {
                    current_state = execute_stage(current_state, stage);
                }
            }
            Operator::Or => {
                if last_exit_code != 0 {
                    current_state = execute_stage(current_state, stage);
                }
            }
            Operator::Semicolon => {
                current_state = execute_stage(current_state, stage);
            }
        }
    }

    (current_state, last_exit_code)
}
```

### 5.4 Memory Management Pattern

```rust
// Virtual memory with page tables
pub struct VirtualMemory {
    page_table: im::HashMap<VirtualPage, PageTableEntry>,
    next_heap_addr: VirtualAddress,
}

// mmap syscall (allocate memory)
pub fn sys_mmap(
    state: KernelState,
    pid: ProcessId,
    size: usize,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // 1. Round size to page boundary
    // 2. Allocate virtual pages
    // 3. Update process page table
    // 4. Return virtual address
}

// Memory permissions: R (read), W (write), X (execute)
pub struct PagePermissions(u8);
const READ: u8 = 0b001;
const WRITE: u8 = 0b010;
const EXECUTE: u8 = 0b100;
```

### 5.5 File I/O Redirection Pattern

```rust
// Redirection operators: >, >>, <
pub enum Redirection {
    StdoutOverwrite(String),  // >  filename
    StdoutAppend(String),     // >> filename
    StdinFrom(String),        // <  filename
}

// Execution flow:
// 1. Parse command with redirections
// 2. Open/create file
// 3. Dup2 to reassign FD 0 (stdin) or FD 1 (stdout)
// 4. Execute command with redirected I/O
// 5. Command reads/writes to file transparently
```

---

## 6. Quality & Test Infrastructure

### 6.1 Test Coverage

| Test Type | Count | Status | Purpose |
|-----------|-------|--------|---------|
| Unit Tests | 380 | 100% ✅ | Function-level validation |
| E2E Tests | 37 | 95% ✅ | Browser integration |
| Canary Tests | 24 | 100% ✅ | Critical workflows |
| Property Tests | 46 | 100% ✅ | Invariant checking (10K inputs each) |
| **Total** | **487** | **99%** | **Comprehensive validation** |

### 6.2 Test Pyramid

```
           E2E Tests (37)
         ┌──────────────┐
        ┌┴──────────────┴┐
       ┌┴────────────────┴┐  Canary Tests (24)
      ┌┴──────────────────┴┐
     ┌┴────────────────────┴┐ Property Tests (46 × 10K inputs)
    ┌┴──────────────────────┴┐
   └┬────────────────────────┬┘ Unit Tests (380)
    └────────────────────────┘
```

### 6.3 Quality Metrics (TDG System)

**Current Score: A+ (95.0)**

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Test Coverage | 85% | 85%+ | ✅ |
| Branch Coverage | 90% | 90%+ | ✅ |
| Mutation Score | 90% | 90%+ | ✅ |
| Cyclomatic Complexity | ≤20 | ≤20 | ✅ |
| Cognitive Complexity | ≤15 | ≤15 | ✅ |
| SATD Comments | 0 | 0 | ✅ |
| Unsafe Code | 0 | 0 | ✅ |
| WASM Size | <500KB | 342KB | ✅ |

### 6.4 Pre-commit Quality Gates

```bash
# Enforced on every commit (<30s)
1. cargo fmt --check          # Code formatting
2. cargo clippy --all-features # Linting (zero warnings)
3. cargo test --lib --workspace # Unit tests (380 tests)
4. PMAT complexity analysis    # Complexity limits
```

### 6.5 Property-Based Testing

**46 properties × 10,000 inputs each = 460,000 test cases**

Key properties verified:
- **Scheduler fairness**: Every ready process gets CPU time
- **Memory safety**: No overlapping allocations
- **Determinism**: Same input → same output
- **Syscall atomicity**: Errors leave state unchanged
- **PID uniqueness**: No duplicate PIDs
- **State cloning**: O(1) cost with im-rs

---

## 7. Extension Points

### 7.1 Adding New System Calls

**Template**:

```rust
// 1. Add variant to SystemCall enum (kernel/src/syscall.rs)
pub enum SystemCall {
    // ... existing syscalls
    YourNewSyscall { param1: Type1, param2: Type2 },
}

// 2. Add output variant to SyscallOutput
pub enum SyscallOutput {
    // ... existing outputs
    YourOutput(YourType),
}

// 3. Implement syscall function
fn sys_your_syscall(
    state: KernelState,
    pid: ProcessId,
    param1: Type1,
    param2: Type2,
) -> SyscallResult<(KernelState, SyscallOutput)> {
    // Pure functional implementation
    // Return (new_state, output)
}

// 4. Add to dispatch_syscall match
pub fn dispatch_syscall(...) -> SyscallResult<...> {
    match syscall {
        // ... existing cases
        SystemCall::YourNewSyscall { param1, param2 } => {
            sys_your_syscall(state, pid, param1, param2)
        }
    }
}

// 5. Add comprehensive tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_your_syscall_basic() { /* ... */ }

    #[test]
    fn test_your_syscall_error_cases() { /* ... */ }

    proptest! {
        #[test]
        fn proptest_your_syscall_invariants(
            inputs in your_strategy()
        ) {
            // Property-based test with 10K inputs
        }
    }
}
```

### 7.2 Adding New User Programs

**Template**:

```rust
// 1. Add to userspace/src/programs.rs
pub fn your_program_main(
    state: KernelState,
    pid: ProcessId,
    args: Vec<String>,
) -> (KernelState, i32) {
    // Parse args
    let options = parse_args(&args);

    // Execute logic (may involve syscalls)
    let (new_state, result) = do_work(state, pid, options);

    // Return (state, exit_code)
    (new_state, if result.is_ok() { 0 } else { 1 })
}

// 2. Register in shell (userspace/src/shell.rs)
fn exec_external(
    state: KernelState,
    pid: ProcessId,
    cmd: &str,
    args: Vec<String>,
) -> (KernelState, i32) {
    match cmd {
        // ... existing programs
        "yourprog" => your_program_main(state, pid, args),
        _ => (state, 127), // Command not found
    }
}

// 3. Add tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_your_program_basic() { /* ... */ }

    #[test]
    fn test_your_program_with_stdin() { /* ... */ }
}
```

### 7.3 Adding Shell Builtins

**Template**:

```rust
// In userspace/src/shell.rs
fn execute_builtin(
    state: KernelState,
    pid: ProcessId,
    cmd: &str,
    args: &[String],
) -> Option<(KernelState, String)> {
    match cmd {
        // ... existing builtins
        "yourbuiltin" => {
            let output = builtin_yourbuiltin(state, pid, args);
            Some((state, output))
        }
        _ => None, // Not a builtin
    }
}

fn builtin_yourbuiltin(
    state: &KernelState,
    pid: ProcessId,
    args: &[String],
) -> String {
    // Implement builtin logic
    format!("Output from your builtin")
}
```

### 7.4 Extending VFS

**Template**:

```rust
// In shared/src/vfs.rs

// 1. Add new file type
pub enum FileEntry {
    Regular { data: Vec<u8>, permissions: FilePermissions },
    Directory { entries: im::HashSet<String> },
    Symlink { target: PathBuf },
    YourNewType { /* fields */ },
}

// 2. Add operations
impl VirtualFileSystem {
    pub fn your_operation(
        &self,
        path: &Path,
        params: YourParams,
    ) -> Result<Self, VfsError> {
        // Return new VFS with changes (persistent)
    }
}

// 3. Add tests
#[test]
fn test_your_vfs_operation() { /* ... */ }
```

### 7.5 Adding Pipeline Operators

**Template**:

```rust
// In shared/src/pipeline.rs

// 1. Add operator variant
pub enum Operator {
    Pipe,         // |
    And,          // &&
    Or,           // ||
    Semicolon,    // ;
    YourOperator, // NEW
}

// 2. Update parser
fn parse_operator(s: &str) -> Option<Operator> {
    match s {
        "|" => Some(Operator::Pipe),
        "&&" => Some(Operator::And),
        "||" => Some(Operator::Or),
        ";" => Some(Operator::Semicolon),
        "your_op" => Some(Operator::YourOperator),
        _ => None,
    }
}

// 3. Update executor (in wos/src/lib.rs or userspace/src/shell.rs)
fn execute_pipeline_stage(
    state: KernelState,
    stage: PipelineStage,
    last_exit: i32,
) -> (KernelState, i32) {
    match stage.operator {
        // ... existing operators
        Operator::YourOperator => {
            // Implement execution logic
        }
    }
}
```

---

## 8. Constraints & Boundaries

### 8.1 Hard Constraints

**Language & Safety**:
- ❌ **NO unsafe code** (`#![forbid(unsafe_code)]` enforced)
- ✅ **100% safe Rust** - memory safety guaranteed by compiler
- ✅ **No panics on invalid input** - all errors handled gracefully

**Architecture**:
- ✅ **Pure functional kernel** - all state changes explicit
- ✅ **No global mutable state** - everything passed as parameters
- ✅ **Deterministic execution** - reproducible behavior

**Platform**:
- ✅ **WASM-only target** - runs in browser, no native code
- ✅ **No WASI dependencies** - pure WASM without OS interface
- ✅ **Single-threaded** - no concurrency, sequential execution

### 8.2 Quality Boundaries

**Test Coverage**:
- Minimum: 85% line coverage, 90% branch coverage
- Target: 90%+ coverage with property-based tests
- Mutation score: 90%+ (current: 98.5%)

**Code Complexity**:
- Cyclomatic complexity: ≤20 per function
- Cognitive complexity: ≤15 per function
- Function length: Prefer <50 lines

**Binary Size**:
- WASM uncompressed: <500KB (current: 342KB)
- WASM gzipped: <100KB target
- Minimize dependency footprint

### 8.3 Scope Boundaries (Educational Focus)

**In Scope** ✅:
- Core OS concepts (processes, memory, files, IPC)
- Educational shell (commands, pipes, redirects, variables)
- Browser-based execution (zero setup for students)
- Time-travel debugging (state inspection)
- Quality metrics dashboard (TDG grading)

**Out of Scope** ❌:
- Full POSIX compatibility (simplified syscall interface)
- Multi-threading / concurrency (educational simplicity)
- Network stack (complexity vs. educational value)
- Graphical UI (terminal-only for focus)
- Production deployment (local development only for MVP)

**Intentionally Simplified**:
- Scheduler: Round-robin only (no priorities, no preemption)
- File systems: VFS only (no real disk I/O)
- Shell: Core features only (no full GNU Bash compatibility)
- Security: Educational model (no real isolation)

### 8.4 Performance Characteristics

**Expected Performance**:
- Cold start (WASM load): <100ms (actual: 280-402ms)
- Syscall dispatch: ~100-500ns
- Process scheduling: ~1-5µs
- Memory allocation: ~2-10µs
- VFS clone: <10µs (O(1) with im-rs)
- State serialization: <50ms for typical workload

**Scalability Limits**:
- Processes: ~1,000 (educational limit, not technical)
- Memory: Limited by browser WASM heap (typically 2GB)
- File descriptors: 1,024 per process
- Message queue: Unbounded (educational, would need limits in production)

---

## 9. Future Considerations

### 9.1 Post-MVP Roadmap (Deferred Features)

**From roadmap.yaml - Post-MVP Phase**:

1. **WOS-022: Priority Scheduling**
   - Priority levels for processes
   - Preemptive scheduling
   - Real-time process support

2. **WOS-023: Advanced Memory**
   - Copy-on-write (COW) fork
   - Shared memory regions
   - Memory-mapped files

3. **WOS-024: Virtual Memory Paging**
   - Page fault handling
   - Demand paging simulation
   - Swap space (simulated)

4. **WOS-025: Network Stack**
   - TCP/IP simulation
   - Socket syscalls
   - Network protocols education

5. **WOS-026: Exec System Call**
   - Replace process image
   - ELF loading simulation
   - Dynamic linking

6. **WOS-027: Signals**
   - Signal delivery
   - Signal handlers
   - SIGCHLD, SIGKILL, etc.

7. **WOS-028: Advanced Pipes**
   - Named pipes (FIFOs)
   - Bidirectional pipes
   - Pipe buffering control

8. **WOS-029: Multi-core Support**
   - SMP simulation
   - CPU affinity
   - Load balancing

9. **WOS-030: Debugging Tools**
   - Process tracer (strace-like)
   - Memory debugger
   - Performance profiler

### 9.2 Architectural Evolutions

**Potential Enhancements**:

1. **Plugin System**:
   ```rust
   pub trait WosPlugin {
       fn init(&self, state: &mut KernelState);
       fn syscall_hook(&self, syscall: &SystemCall) -> Option<SyscallOutput>;
       fn tick(&self, state: &KernelState);
   }
   ```

2. **Capability-Based Security**:
   ```rust
   pub struct Capability {
       resource: ResourceId,
       permissions: Permissions,
       delegatable: bool,
   }
   ```

3. **Micro-Container Support**:
   ```rust
   pub struct Container {
       pid_namespace: PidNamespace,
       mount_namespace: MountNamespace,
       network_namespace: NetworkNamespace,
       resource_limits: ResourceLimits,
   }
   ```

4. **Event System**:
   ```rust
   pub enum KernelEvent {
       ProcessCreated(ProcessId),
       ProcessTerminated(ProcessId, i32),
       MemoryAllocated(ProcessId, VirtualAddress, usize),
       FileOpened(ProcessId, PathBuf),
   }

   pub trait EventListener {
       fn on_event(&mut self, event: KernelEvent);
   }
   ```

### 9.3 Integration Opportunities

**Educational Platforms**:
- Jupyter notebook integration (Python kernel calling WOS)
- VS Code extension (inline OS execution)
- Interactive tutorials with live WOS instances
- Automated grading system (test student shell scripts)

**Research Applications**:
- OS algorithm visualization (scheduling, paging)
- Distributed systems simulation (multi-WOS instances)
- Security education (capability systems, sandboxing)
- Performance analysis (time-travel profiling)

### 9.4 Technical Debt & Refactoring Opportunities

**Current Known Items**:

1. **E2E Test Flakiness** (2-3 tests):
   - Issue: Timing-sensitive auto-scroll and clear tests
   - Fix: Better event waiting, less timeout reliance
   - Impact: Low (test infrastructure only, not functionality)

2. **Syscall Error Handling**:
   - Current: String-based error messages
   - Enhancement: Structured error types with error codes
   - Benefit: Better error recovery, clearer debugging

3. **VFS Performance**:
   - Current: HashMap lookup for every path component
   - Enhancement: Path caching, directory entry cache
   - Benefit: Faster repeated lookups

4. **WASM Binary Size**:
   - Current: 342KB (within target)
   - Opportunity: Further reduction via feature flags
   - Benefit: Faster load times, better mobile experience

**No Critical Technical Debt** - All items are enhancements, not blockers.

---

## Appendix A: System Call Reference

### Complete Syscall List (15 total)

| Syscall | Purpose | Input | Output | State Change |
|---------|---------|-------|--------|--------------|
| `GetPid` | Get process ID | - | ProcessId | None |
| `Fork` | Create child | - | ProcessId (child PID) | New process |
| `Exit(i32)` | Terminate | Exit code | - | Process removed |
| `WaitPid(ProcessId)` | Wait for child | Child PID | Exit code | Process reaped |
| `Sleep(u64)` | Sleep µs | Microseconds | - | Process blocked |
| `Open{path,flags}` | Open file | Path, flags | FileDescriptor | FD allocated |
| `Close{fd}` | Close FD | FD number | - | FD released |
| `Read{fd,count}` | Read bytes | FD, count | Vec<u8> | FD position |
| `Write{fd,data}` | Write bytes | FD, data | Bytes written | FD position, file modified |
| `Pipe` | Create pipe | - | (read_fd, write_fd) | 2 FDs allocated |
| `Dup2{oldfd,newfd}` | Duplicate FD | Old/new FD | - | FD reassigned |
| `Mmap{size}` | Allocate mem | Size | VirtualAddress | Pages allocated |
| `Munmap{addr,size}` | Free memory | Addr, size | - | Pages freed |
| `Send{pid,data}` | Send message | Target PID, data | - | Message queued |
| `Recv{timeout}` | Recv message | Timeout µs | Message | Message dequeued |

---

## Appendix B: File System Layout

### Complete VFS Structure

```
/                                   [Root directory]
├── bin/                            [Executable programs]
│   ├── echo
│   ├── ls
│   ├── ps
│   ├── cat
│   ├── grep
│   └── wc
│
├── dev/                            [Virtual devices]
│   ├── null                        [Null device (discard writes)]
│   ├── zero                        [Zero device (infinite zeros)]
│   ├── random                      [Random bytes (deterministic)]
│   └── console                     [Console I/O]
│
├── proc/                           [Process information (ProcFS)]
│   ├── 1/                          [Init process]
│   │   ├── status                  [Process status]
│   │   ├── cmdline                 [Command line]
│   │   └── maps                    [Memory mappings]
│   │
│   ├── 2/                          [Shell process]
│   │   └── ...
│   │
│   ├── self -> /proc/N             [Symlink to current process]
│   └── ...
│
├── tmp/                            [Temporary files]
│   └── [user-created files]
│
└── home/                           [User home directories]
    └── [user-created files]
```

---

## Appendix C: Quality Metrics Dashboard

### TDG (Test-Driven Grade) Components

**Metrics Tracked**:
1. **Test Count**: 443 total tests
2. **Coverage**: 85%+ line, 90%+ branch
3. **Mutation Score**: 90%+ (mutant kill rate)
4. **Complexity**: Cyclomatic ≤20, Cognitive ≤15
5. **Code Quality**: Zero unsafe, zero SATD
6. **Build Status**: All gates passing

**Export Formats**:
- **JSON**: Machine-readable metrics
- **HTML**: Visual dashboard report
- **SARIF**: GitHub code scanning integration
- **Markdown**: Documentation format

**Access Points**:
- Browser UI: Quality panel in terminal
- CLI: `wos.getQualityMetrics()`
- Files: Generated reports in `dist/`

---

## Document Maintenance

**Update Triggers**:
- New feature implementation (update Extension Points)
- Architecture changes (update System Architecture)
- Performance changes (update Constraints)
- Quality metric changes (update Quality section)

**Versioning**:
- v1.0: MVP completion (this document)
- v1.1: First post-MVP feature
- v2.0: Major architectural change

**Last Updated**: October 17, 2025
**Next Review**: When implementing WOS-022+

---

*This document provides the architectural foundation for designing and implementing new WOS features. Always verify against actual codebase for implementation details.*
