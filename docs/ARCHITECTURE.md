# WOS Architecture Guide

Complete architectural overview of the WebAssembly Operating System.

## Table of Contents

- [System Overview](#system-overview)
- [Core Principles](#core-principles)
- [Module Structure](#module-structure)
- [Data Flow](#data-flow)
- [Design Patterns](#design-patterns)
- [Memory Management](#memory-management)
- [Process Model](#process-model)
- [File System](#file-system)
- [Quality Infrastructure](#quality-infrastructure)

---

## System Overview

WOS is a microkernel operating system compiled to WebAssembly, demonstrating OS concepts in a pure Rust, safe environment.

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                 HTML Terminal UI                       │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐ │  │
│  │  │  Terminal   │  │   Quality    │  │   Process    │ │  │
│  │  │   Input     │  │  Dashboard   │  │     List     │ │  │
│  │  └─────────────┘  └──────────────┘  └──────────────┘ │  │
│  └───────────────────────────────────────────────────────┘  │
│         ▲                    │                               │
│         │ User Input         │ Display Output                │
│         │                    ▼                               │
│  ┌──────────────────────────────────────────────────┐       │
│  │            JavaScript (app.js)                    │       │
│  │  ┌────────────────────────────────────────────┐  │       │
│  │  │  Terminal Controller                        │  │       │
│  │  │  - Command history                          │  │       │
│  │  │  - Event handling                           │  │       │
│  │  │  - WASM integration                         │  │       │
│  │  └────────────────────────────────────────────┘  │       │
│  └──────────────────────────────────────────────────┘       │
│         ▲                    │                               │
│         │ JS Call            │ Return Value                  │
│         │                    ▼                               │
│  ┌──────────────────────────────────────────────────┐       │
│  │         WASM Bindings (wasm-bindgen)             │       │
│  │  ┌────────────────────────────────────────────┐  │       │
│  │  │  WosWasm Struct                             │  │       │
│  │  │  - executeSyscall()                         │  │       │
│  │  │  - executeCommand()                         │  │       │
│  │  │  - getState() / setState()                  │  │       │
│  │  │  - getQualityMetrics()                      │  │       │
│  │  └────────────────────────────────────────────┘  │       │
│  └──────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
         ▲                    │
         │ Syscall            │ Result
         │                    ▼
┌─────────────────────────────────────────────────────────────┐
│              WOS Kernel (Pure Rust WASM)                     │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                 Kernel State                           │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  processes: HashMap<PID, Process>               │  │  │
│  │  │  scheduler: Scheduler                            │  │  │
│  │  │  filesystem: VirtualFileSystem                   │  │  │
│  │  │  next_pid: PID                                   │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
│         ▲                    │                               │
│         │                    │                               │
│  ┌──────┴────────────────────┴────────┐                     │
│  │     Syscall Dispatcher             │                     │
│  │  dispatch_syscall(state, call)     │                     │
│  └─────────────────────────────────────┘                    │
│         │                                                    │
│    ┌────┴────┬────────┬─────────┬────────┐                 │
│    ▼         ▼        ▼         ▼        ▼                  │
│  ┌─────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐              │
│  │Fork │ │Exit  │ │Mmap  │ │Read  │ │Send  │              │
│  │Wait │ │GetPid│ │Munmap│ │Write │ │Recv  │              │
│  └─────┘ └──────┘ └──────┘ └──────┘ └──────┘              │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Userspace Programs                        │  │
│  │  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐   │  │
│  │  │ Init │  │Shell │  │ Echo │  │  Ls  │  │  Ps  │   │  │
│  │  │(PID1)│  │      │  │      │  │      │  │      │   │  │
│  │  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Principles

### 1. Pure Functional Design

Every operation is a pure function: input state → output state.

```rust
pub fn dispatch_syscall(
    state: KernelState,      // Input state (immutable)
    syscall: SystemCall,     // Operation to perform
    calling_pid: ProcessId,  // Calling process
) -> Result<(KernelState, SyscallOutput), KernelError>
//           ^^^^^^^^^^^^ New state returned
```

**Benefits:**
- No hidden side effects
- Easy to test
- Perfect for time-travel debugging
- Deterministic execution

### 2. Zero Unsafe Code

```rust
#![forbid(unsafe_code)]
```

Enforced at crate level. All code is memory-safe Rust.

**Benefits:**
- No segfaults
- No undefined behavior
- No use-after-free
- Compiler-verified safety

### 3. Persistent Data Structures

Using `im-rs` for O(1) cloning:

```rust
pub struct KernelState {
    pub processes: im::HashMap<ProcessId, Process>,  // O(1) clone
    pub scheduler: Scheduler,
    pub filesystem: VirtualFileSystem,
}
```

**Benefits:**
- State cloning is instant
- No deep copies
- Structural sharing
- Memory efficient

### 4. Microkernel Architecture

Minimal trusted computing base:

```
Kernel (Trusted):
- Process scheduling
- Memory management
- System call dispatch
- IPC primitives

Userspace (Untrusted):
- Init process
- Shell
- User programs
- File systems (future)
```

---

## Module Structure

```
wos/
├── kernel/                  # Core kernel (trusted)
│   ├── src/
│   │   ├── state.rs        # KernelState, Process
│   │   ├── scheduler.rs    # Round-robin scheduler
│   │   ├── memory.rs       # Virtual memory
│   │   ├── syscall.rs      # Syscall dispatcher
│   │   ├── trace.rs        # Time-travel debugging
│   │   └── lib.rs          # Public API
│   └── tests/              # 153 tests
│
├── shared/                  # Shared utilities
│   ├── src/
│   │   ├── vfs.rs          # Virtual file system
│   │   ├── context.rs      # Execution context
│   │   └── lib.rs
│   └── tests/              # 17 tests
│
├── userspace/               # Userspace programs
│   ├── src/
│   │   ├── init.rs         # PID 1, orphan reaper
│   │   ├── shell.rs        # Command interpreter
│   │   ├── programs/       # echo, ls, ps
│   │   │   ├── echo.rs
│   │   │   ├── ls.rs
│   │   │   └── ps.rs
│   │   └── lib.rs
│   └── tests/              # 45 tests
│
├── wos/                     # WASM bindings
│   ├── src/
│   │   ├── quality.rs      # Quality metrics
│   │   └── lib.rs          # WosWasm struct
│   └── tests/              # 47 tests
│
└── dist/wos/                # Browser interface
    ├── index.html          # Terminal UI
    ├── app.js              # Integration
    ├── style.css           # Styling
    └── wos_bg.wasm         # Compiled kernel (285KB)
```

---

## Data Flow

### Syscall Execution Flow

```
1. User Input (Browser)
   ↓
2. JavaScript Event Handler
   ↓
3. Terminal.executeCommand("ps")
   ↓
4. WosWasm.executeCommand("ps")
   ↓
5. Parse command → Create SystemCall
   ↓
6. dispatch_syscall(state, syscall, pid)
   ↓
7. Route to appropriate handler
   ├─ sys_getpid()
   ├─ sys_fork()
   ├─ sys_read()
   └─ etc.
   ↓
8. Execute pure function
   state → (new_state, output)
   ↓
9. Update WosWasm.state = new_state
   ↓
10. Serialize output to JSON
   ↓
11. Return to JavaScript
   ↓
12. Display in terminal
```

### State Management Flow

```
┌─────────────────────────────────────────┐
│         KernelState (Root)              │
│  ┌───────────────────────────────────┐  │
│  │ processes: HashMap<PID, Process>  │  │
│  │   ├─ PID 1 (Init)                │  │
│  │   │   ├─ state: Running          │  │
│  │   │   ├─ memory: VirtualMemory   │  │
│  │   │   ├─ files: FileTable        │  │
│  │   │   └─ children: [2]           │  │
│  │   └─ PID 2 (Shell)               │  │
│  │       ├─ state: Ready             │  │
│  │       ├─ memory: VirtualMemory   │  │
│  │       ├─ files: FileTable        │  │
│  │       └─ parent: Some(1)         │  │
│  └───────────────────────────────────┘  │
│                                          │
│  ┌───────────────────────────────────┐  │
│  │ scheduler: Scheduler              │  │
│  │   ├─ queue: [1, 2]               │  │
│  │   └─ current_index: 0            │  │
│  └───────────────────────────────────┘  │
│                                          │
│  ┌───────────────────────────────────┐  │
│  │ filesystem: VFS                   │  │
│  │   └─ files: HashMap<Path, File>  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘

Every syscall:
  Old State → Pure Function → New State
                             (structural sharing with old state)
```

---

## Design Patterns

### 1. Syscall Pattern

All syscalls follow this pattern:

```rust
pub fn sys_operation(
    state: KernelState,
    calling_pid: ProcessId,
    // ... operation-specific parameters
) -> Result<(KernelState, SyscallOutput), KernelError> {
    // 1. Validate inputs
    let process = state.get_process(calling_pid)
        .ok_or(KernelError::ProcessNotFound)?;

    // 2. Perform operation
    let mut new_state = state.clone();  // O(1) with im-rs
    // ... modify new_state

    // 3. Return new state and output
    Ok((new_state, SyscallOutput::Success))
}
```

### 2. Process State Machine

```rust
pub enum ProcessState {
    Ready,              // Waiting to run
    Running,            // Currently executing
    Blocked,            // Waiting for I/O
    Terminated(i32),    // Exited with code
}

// Transitions:
// Ready → Running (scheduler picks it)
// Running → Ready (time slice expires)
// Running → Blocked (waiting for I/O)
// Blocked → Ready (I/O completes)
// Running → Terminated (exit)
```

### 3. Resource Ownership

```rust
pub struct Process {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub state: ProcessState,
    pub memory: VirtualMemory,         // Owned by process
    pub file_descriptors: FileTable,   // Owned by process
    pub message_queue: VecDeque<Message>, // Owned by process
}

// On fork():
// - Memory is cloned (COW in real OS)
// - FDs are duplicated
// - Message queue is empty

// On exit():
// - All resources released
// - Children reparented to init
```

### 4. Error Handling

```rust
// Never panic - always return Result
pub fn safe_operation() -> Result<T, KernelError> {
    // Validate
    if !valid {
        return Err(KernelError::InvalidInput);
    }

    // Execute
    let result = operation()?;

    // Return
    Ok(result)
}

// Property tests verify: never panics on any input
proptest! {
    fn never_panics(input: ArbitraryInput) {
        let _ = operation(input); // Should not panic
    }
}
```

---

## Memory Management

### Virtual Address Space

```
Process Virtual Memory Layout:

0x0000_0000 - 0x0FFF_FFFF   Code Segment (256 MB)
  └─ Executable instructions

0x1000_0000 - 0x1FFF_FFFF   Data Segment (256 MB)
  └─ Static data

0x2000_0000 - 0x2FFF_FFFF   Heap (256 MB)
  └─ Dynamic allocations (mmap)

0x3000_0000 - 0x3FFF_FFFF   Stack (256 MB)
  └─ Function call stack
```

### Page Table Structure

```rust
pub struct VirtualMemory {
    // Maps virtual pages to physical pages
    page_table: im::HashMap<VirtualPage, PhysicalFrame>,

    // Tracks page permissions
    permissions: im::HashMap<VirtualPage, PagePermissions>,

    // Next free virtual address in heap
    next_heap_addr: VirtualAddress,
}

// Page size: 4KB (4096 bytes)
// Address translation: O(log n) via HashMap
```

### Allocation Strategy

```rust
// mmap allocates from heap region
pub fn sys_mmap(size: usize) -> VirtualAddress {
    let pages_needed = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    let addr = next_heap_addr;

    for i in 0..pages_needed {
        let vpage = VirtualPage(addr + i * PAGE_SIZE);
        let pframe = allocate_physical_frame();
        page_table.insert(vpage, pframe);
    }

    next_heap_addr += pages_needed * PAGE_SIZE;
    addr
}

// Addresses are sequential and never reused (simplified)
// munmap frees but doesn't reuse addresses
```

---

## Process Model

### Process Lifecycle

```
          [fork]
            ↓
    ┌─────Ready─────┐
    │               │
    │ [schedule]    │ [yield]
    ↓               │
 Running ───────────┘
    │
    │ [I/O wait]
    ↓
 Blocked ─── [I/O complete] ──→ Ready
    │
    │ [exit]
    ↓
Terminated
```

### Fork Semantics

```rust
// Parent (PID 5) calls fork()
let parent_state = /* PID 5's state */;

// Creates child (PID 6)
let mut child = parent_state.clone();
child.pid = 6;
child.parent_pid = Some(5);
child.state = ProcessState::Ready;
child.message_queue = VecDeque::new();  // Empty queue

// Parent receives child PID
parent_gets: SyscallOutput::Pid(6)

// Child receives 0
child_gets: SyscallOutput::Pid(0)
```

### Scheduling Algorithm

```rust
// Round-robin scheduler
pub struct Scheduler {
    queue: VecDeque<ProcessId>,  // Ready queue
    current_index: usize,         // Current position
}

impl Scheduler {
    pub fn schedule(&mut self) -> Option<ProcessId> {
        if self.queue.is_empty() {
            return None;
        }

        // Get next PID
        let pid = self.queue[self.current_index];

        // Move to next (wrap around)
        self.current_index = (self.current_index + 1) % self.queue.len();

        Some(pid)
    }
}

// Properties:
// - Fairness: Every process gets equal CPU time
// - No starvation: All processes eventually run
// - O(1) scheduling decision
```

---

## File System

### Virtual File System

```
Root (/)
├── proc/              # Process information (ProcFS)
│   ├── 1/            # PID 1 (init)
│   │   ├── status    # Process status
│   │   └── cmdline   # Command line
│   ├── 2/            # PID 2 (shell)
│   │   ├── status
│   │   └── cmdline
│   └── self/         # Symlink to current process
│
└── tmp/               # Temporary files (future)
```

### File Descriptor Table

```rust
pub struct FileTable {
    descriptors: im::HashMap<FileDescriptor, OpenFile>,
    next_fd: FileDescriptor,
}

// Standard descriptors (always open)
// 0 - stdin
// 1 - stdout
// 2 - stderr

// User descriptors start at 3
```

### ProcFS Implementation

```rust
// /proc/PID/status - dynamically generated
pub fn read_proc_status(pid: ProcessId) -> Vec<u8> {
    let process = state.get_process(pid)?;
    format!(
        "PID: {}\nState: {:?}\nParent: {:?}\n",
        process.pid,
        process.state,
        process.parent_pid
    ).into_bytes()
}

// No actual files stored - generated on demand
```

---

## Quality Infrastructure

### Testing Pyramid

```
                    ┌──────┐
                    │  1   │  E2E Tests (Browser)
                   ┌┴──────┴┐
                   │   12   │  Integration Tests
                  ┌┴────────┴┐
                  │    42    │  Property Tests
                 ┌┴──────────┴┐
                 │     207    │  Unit Tests
                └─────────────┘
                 Total: 262 tests
```

### TDG Calculation

```rust
TDG Score = (
    (test_coverage * 0.25) +              // 88% = 22.0
    (1.0 - avg_complexity/20) * 0.20 +   // 7.8 = 19.2
    (mutation_score * 0.20) +             // 90% = 18.0
    (doc_coverage * 0.15) +               // 70% = 10.5
    (1.0 - satd/10) * 0.10 +             // 0   = 10.0
    (meets_benchmarks * 0.10)             // Yes = 10.0
) * 100

Current: 95.5% (A+ grade)
```

### Quality Gates

```bash
# Pre-commit hook (< 30s)
make quality
  ├─ cargo fmt --check      # Code formatting
  ├─ cargo clippy           # Lints
  └─ cargo test --lib       # Unit tests

# CI Pipeline
make quality-complete
  ├─ make quality           # Fast gates
  ├─ cargo test --all       # All tests
  └─ cargo llvm-cov         # Coverage
```

### Time-Travel Debugging

```rust
pub struct KernelHistory {
    snapshots: Vec<KernelState>,    // State after each syscall
    traces: Vec<SystemCallTrace>,   // Syscall details
    position: usize,                 // Current position
}

// Go back in time
pub fn step_back(&mut self) {
    if self.position > 0 {
        self.position -= 1;
        // State automatically restored
    }
}

// Go forward
pub fn step_forward(&mut self) {
    if self.position < self.snapshots.len() - 1 {
        self.position += 1;
    }
}
```

---

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Syscall dispatch | O(1) | Direct function call |
| State clone | O(1) | Persistent data structures |
| Process lookup | O(log n) | HashMap lookup |
| Schedule | O(1) | Round-robin queue |
| Memory allocation | O(log n) | Page table insert |
| File open | O(log n) | VFS lookup |

### Space Complexity

| Structure | Size | Notes |
|-----------|------|-------|
| KernelState | ~1KB base | + processes |
| Process | ~500B | + memory pages |
| WASM Binary | 285KB | Compressed: ~100KB |
| Quality metrics | ~2KB | JSON |

### Benchmarks (Target)

- WASM load: <100ms
- Syscall: <10μs
- State clone: <10μs
- Schedule: <1μs
- Memory alloc: <5μs

---

## Future Architecture

### Planned Enhancements

**1. Preemptive Scheduling**
```rust
// Add time slices
pub struct Scheduler {
    queue: VecDeque<ProcessId>,
    time_slices: HashMap<ProcessId, Duration>,
    quantum: Duration,  // 100ms
}
```

**2. Demand Paging**
```rust
// Load pages on demand
pub struct VirtualMemory {
    page_table: HashMap<VirtualPage, Option<PhysicalFrame>>,
    //                                ^^^^^^ None = not loaded yet
    swap_space: SwapFile,
}
```

**3. Network Stack**
```rust
// Socket syscalls
sys_socket() -> SocketFd
sys_bind(fd, addr)
sys_connect(fd, addr)
sys_send_to(fd, data, addr)
sys_recv_from(fd) -> (data, addr)
```

---

## References

- [API Documentation](API.md)
- [Tutorials](tutorials/)
- [Roadmap](../roadmap.yaml)
- [Specification](specifications/wos-spec-v1.md)
- [Source Code](https://github.com/noahgift/wos)

---

## Glossary

- **TDG**: Technical Debt Gap - measure of code quality
- **SATD**: Self-Admitted Technical Debt (TODO/FIXME comments)
- **ProcFS**: Process file system (/proc)
- **VFS**: Virtual File System
- **COW**: Copy-On-Write
- **IPC**: Inter-Process Communication
- **Microkernel**: Minimal kernel design
- **Syscall**: System call - kernel API
- **Property Test**: Randomized testing of invariants

---

*Last Updated: 2025-10-14*
*WOS Version: 0.1.0*
*TDG Grade: A+ (95.5%)*
