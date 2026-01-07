# WOS Architectural Components Specification v1.0

**Document Status**: Pre-Implementation Technical Blueprint  
**Date**: October 14, 2025  
**Architecture**: Pure Functional, Microkernel, WASM-native

---

## Executive Overview

This specification defines the high-level architectural logic for each major component of the WASM Operating System (WOS). Each section articulates the fundamental design patterns, data flows, and implementation strategies derived from established OS theory, microkernel research, and WebAssembly computational constraints.

**Design Philosophy**: Pure functional state transitions, zero unsafe code, L4-inspired IPC minimalism, deterministic execution semantics.

---

## 1. Microkernel Foundation & Message-Passing IPC

**Core Principle**: Synchronous message passing with fast-path optimization for intra-instance communication.

### High-Level Logic

1. **Channel-Based Communication Model**  
   Processes send messages to *channels* (not directly to processes), inspired by L4 and QNX Neutrino architectures. Each channel is owned by exactly one receiver process but accepts messages from multiple senders. Channels are identified by `ChannelId` and created dynamically by receiver processes.

2. **Synchronous Send-Receive-Reply Semantics**  
   Implements the classic `Send(channel, msg) -> Reply(data)` pattern. The sender blocks until the receiver processes the message and sends a reply. This eliminates buffering overhead and enforces deterministic execution order, critical for educational clarity.

3. **Fast-Path IPC Optimization**  
   When sender and receiver reside in the same WASM instance (same process address space), bypass full message serialization. Direct memory transfer via shared `im::HashMap` structures reduces IPC overhead from O(n) copy to O(log n) persistent data structure update.

4. **Message Structure & Serialization**  
   Messages are `enum Message { Request(RequestType, Vec<u8>), Reply(Result<Vec<u8>, IpcError>) }`. All message payloads are serialized via `serde` to byte vectors, enabling uniform handling regardless of content type. Maximum message size: 4KB to prevent DoS via memory exhaustion.

5. **Capability-Based Security**  
   Channels are unforgeable capabilities. A process cannot send to a channel unless it possesses a `ChannelHandle`, obtained only through explicit delegation or kernel-mediated operations. This provides coarse-grained but effective isolation.

6. **Rendezvous Semantics & Blocking**  
   When `Send()` is called, the sender transitions to `ProcessState::BlockedOnSend(channel_id)`. When `Receive()` is called on an empty channel, receiver transitions to `ProcessState::BlockedOnReceive`. Scheduler unblocks processes only when both sides are ready, creating a *rendezvous*.

7. **IPC Error Handling**  
   Failures include: `ChannelNotFound`, `PermissionDenied`, `MessageTooLarge`, `ReceiverTerminated`. All errors return via `Result<Reply, IpcError>` to sender. Dead channel detection prevents indefinite blocking on terminated receivers.

8. **Port Abstraction Layer**  
   Higher-level "ports" bundle multiple channels for service-oriented architectures. A file system server might expose `port_fs` with channels for `open`, `read`, `write`. Ports are registered in a global `PortRegistry` for service discovery.

9. **Zero-Copy Message Passing (Future)**  
   For post-MVP: Shared memory segments as a secondary IPC mechanism. Requires careful synchronization via message-based locking protocols to maintain functional purity semantics in the API layer.

---

## 2. Process Scheduler & Lifecycle Management

**Core Principle**: Cooperative round-robin with instruction budget enforcement for pseudo-preemption.

### High-Level Logic

1. **Process State Machine**  
   Five canonical states: `Ready`, `Running`, `BlockedOnSend(ChannelId)`, `BlockedOnReceive`, `Terminated`. State transitions are pure functions: `fn transition(state: ProcessState, event: SchedulerEvent) -> ProcessState`. All transitions logged for educational visualization.

2. **Round-Robin Ready Queue**  
   Maintains `VecDeque<ProcessId>` for ready processes. Scheduler selects head of queue, executes process, then rotates to tail. Guarantees fairness: every ready process scheduled within N cycles where N = queue length. Prevents starvation by construction.

3. **Instruction Budget Pseudo-Preemption**  
   Each process receives `MAX_INSTRUCTIONS_PER_SLICE = 10,000` instruction budget per time slice. Executor counts instructions via `step_process()` increments. When budget exhausted, forcibly yield via `ProcessResult::Preempted`. Emulates preemptive multitasking without hardware timer interrupts.

4. **Cooperative Yield Points**  
   Processes voluntarily yield via `syscall::yield()` or blocking IPC operations. Yield returns immediately with state change to `Ready`. Incentivizes well-behaved process design for educational demonstrations.

5. **Process Creation & Fork Semantics**  
   `fork()` clones parent's memory space (via `im::HashMap` structural sharing for O(log n) duplication). Child receives new `ProcessId`, inherits parent's channel handles (with COW semantics). Both processes resume at fork return point with different return values (parent gets child PID, child gets 0).

6. **PID Allocation & Uniqueness**  
   PIDs assigned monotonically from `AtomicU32` counter. PID 1 reserved for `init` process. PID uniqueness enforced via `BTreeSet<ProcessId>` in process table. Dead process PIDs recycled only after reboot to prevent confusion.

7. **Priority System (MVP: Omitted)**  
   Single-priority round-robin for MVP. Post-MVP: Multi-level feedback queue with priority aging. Processes start at base priority, increase priority on I/O wait (interactive process heuristic), decrease on CPU-bound execution.

8. **Scheduler Invocation Points**  
   Scheduler runs after: (1) system call completion, (2) instruction budget exhaustion, (3) explicit yield, (4) IPC block/unblock. Never interrupts mid-instruction for WebAssembly execution atomicity guarantees.

9. **Process Termination & Cleanup**  
   `exit(code)` transitions process to `Terminated(exit_code)`. Scheduler reaps terminated processes, closes all channel handles, deallocates memory pages, removes from process table. Orphaned children reparented to `init` (PID 1) for cleanup.

---

## 3. Virtual Memory Subsystem & Page Allocation

**Core Principle**: Simulated paging for educational demonstration, not performance optimization.

### High-Level Logic

1. **Linear Address Space Simulation**  
   Each process has a 32-bit virtual address space (4GB), divided into 4KB pages. Virtual addresses map to simulated "physical" pages via `PageTable: BTreeMap<VirtualPageNumber, PhysicalPageNumber>`. WebAssembly's linear memory is single flat array—WOS adds paging *abstraction*.

2. **Page Table Structure**  
   Two-level page table: Page Directory (1024 entries) → Page Table (1024 entries). Total: 1,048,576 pages × 4KB = 4GB. Encoded as nested `BTreeMap` for sparse allocation. Unallocated pages consume zero memory via structural sharing.

3. **Memory Allocation Strategy**  
   Kernel maintains free page bitmap (`BitVec<Lsb0, u32>`) for physical pages. `alloc_pages(n: usize) -> Result<VirtualAddress>` finds n contiguous pages via bitmap scan, allocates from top-of-space downward (stack) or bottom-upward (heap) depending on request type.

4. **Page Fault Handling (Simplified)**  
   Memory access via `read_memory(vaddr: VirtualAddress) -> Result<u8, MemoryError>`. On missing page: return `PageFault(vaddr)` error. No demand paging or swapping in MVP—all memory explicitly allocated. Educational tool for understanding page fault mechanics.

5. **Memory Protection Bits**  
   Each page table entry includes: `readable`, `writable`, `executable` flags. Access violations return `PermissionDenied` error. No kernel/user mode distinction (single privilege level in MVP)—protection purely process-boundary enforcement.

6. **Copy-on-Write Semantics**  
   For `fork()`: Parent and child initially share physical pages with `writable=false`. On write attempt, trigger COW logic: allocate new physical page, copy contents, update child's page table, restore write permission. Implemented via reference counting on physical pages.

7. **Memory-Mapped I/O Regions**  
   Special virtual address ranges (`0xFFFFF000` - `0xFFFFFFFF`) reserved for device I/O. Reads/writes to these addresses dispatch to device drivers via callback table. Enables uniform memory access model for hardware abstraction.

8. **Zero-Page Optimization**  
   Global shared zero-page at `PhysicalPageNumber(0)`, mapped read-only into all processes requesting zeroed memory. Write triggers COW allocation. Saves memory for large BSS segments.

9. **Memory Leak Detection (Testing)**  
   Reference counting per physical page. On process termination, assert all pages deallocated (refcount == 0). Integration with `cargo-valgrind` for memory leak validation during test suite execution.

---

## 4. Virtual File System (VFS Extension)

**Core Principle**: Mount point abstraction over heterogeneous storage backends.

### High-Level Logic

1. **VFS Core Abstractions**  
   Four primary objects: `Superblock` (mounted filesystem metadata), `Inode` (file metadata), `Dentry` (directory entry cache), `FileDescriptor` (process-local open file handle). All objects are persistent data structures with structural sharing.

2. **Mount Point Resolution**  
   Global mount table: `BTreeMap<PathBuf, Arc<dyn Filesystem>>`. Path lookup starts at root (`/`), traverses mount table to find longest matching prefix, delegates remaining path components to mounted filesystem's implementation.

3. **Filesystem Trait Interface**  
   ```rust
   trait Filesystem: Send + Sync {
       fn lookup(&self, inode: InodeId, name: &str) -> Result<InodeId>;
       fn read(&self, inode: InodeId, offset: u64, buf: &mut [u8]) -> Result<usize>;
       fn write(&self, inode: InodeId, offset: u64, buf: &[u8]) -> Result<usize>;
       fn create(&self, parent: InodeId, name: &str, mode: FileMode) -> Result<InodeId>;
       // ... additional operations
   }
   ```
   Filesystem implementations register via trait objects, enabling dynamic dispatch and extensibility.

4. **Inode Number Allocation**  
   Each filesystem assigns unique inode numbers within its scope. Global inode identifier: `(FilesystemId, LocalInodeId)` tuple. Root inode: `(0, 1)` by convention. Inode cache maintains `HashMap<GlobalInodeId, Arc<Inode>>` for fast lookups.

5. **Directory Entry Caching (Dcache)**  
   LRU cache: `HashMap<(InodeId, String), InodeId>` mapping (parent_inode, filename) → child_inode. Cached entries expire after 5000 operations or explicit invalidation. Accelerates path resolution for deeply nested directories.

6. **File Descriptor Table**  
   Per-process table: `Vec<Option<OpenFile>>` where `OpenFile { inode: InodeId, offset: AtomicU64, flags: OpenFlags }`. FD allocation: first available index. FDs 0, 1, 2 reserved for stdin, stdout, stderr (mapped to browser console via WASM bindings).

7. **Special Filesystems**  
   - `/proc`: Process information filesystem. Virtual inodes generated on-demand from kernel state (e.g., `/proc/1/status` returns process 1's metadata as string).
   - `/dev`: Device filesystem. Inodes represent device drivers. Read/write operations dispatch to driver callbacks.
   - `tmpfs`: In-memory filesystem using `HashMap<PathBuf, Vec<u8>>` for storage. Data lost on process termination.

8. **Filesystem Registration & Discovery**  
   Kernel maintains `HashMap<String, Box<dyn FilesystemConstructor>>` for filesystem types ("tmpfs", "procfs", "devfs"). Mount syscall: `mount(type: &str, target: &Path, options: &str)` looks up constructor, instantiates filesystem, inserts into mount table.

9. **Atomicity & Consistency**  
   No transactions in MVP (single-threaded execution). File operations are atomic at syscall boundary. Crash consistency: filesystem state always recoverable by replaying syscall trace (property tested via stateful fuzzing).

---

## 5. System Call Dispatcher & Handler Registry

**Core Principle**: Function pointer table with compile-time registration and runtime dispatch.

### High-Level Logic

1. **System Call Enumeration**  
   ```rust
   #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
   #[repr(u32)]
   pub enum SyscallNumber {
       GetPid = 1,
       Fork = 2,
       Exec = 3,
       Exit = 4,
       Open = 5,
       Read = 6,
       Write = 7,
       Close = 8,
       Send = 9,
       Receive = 10,
       // ... up to ~30 syscalls
   }
   ```
   Each syscall assigned unique numeric identifier. `repr(u32)` ensures stable ABI across compilations.

2. **Handler Function Signature**  
   ```rust
   type SyscallHandler = fn(
       state: KernelState,
       context: ProcessContext,
       args: SyscallArgs,
   ) -> Result<(KernelState, ProcessContext, SyscallResult), KernelError>;
   ```
   Pure function: input (state, context, args) → output (new_state, new_context, result). All state transitions explicit and testable.

3. **Dispatch Table Construction**  
   ```rust
   pub struct SyscallDispatcher {
       handlers: HashMap<SyscallNumber, SyscallHandler>,
   }
   
   impl SyscallDispatcher {
       pub fn new() -> Self {
           let mut handlers = HashMap::new();
           handlers.insert(SyscallNumber::GetPid, sys_getpid);
           handlers.insert(SyscallNumber::Fork, sys_fork);
           // ... register all handlers
           Self { handlers }
       }
   }
   ```
   Compile-time registration via manual `insert()` calls. Post-MVP: Procedural macro for auto-registration.

4. **System Call Invocation Flow**  
   User process → `syscall(num: u32, args: [u64; 6])` → Kernel validates `num` → Lookup handler in table → Parse args into `SyscallArgs` struct → Invoke handler → Return `SyscallResult` to user.

5. **Argument Marshalling**  
   System call arguments passed as 6 × 64-bit registers (ABI: `rax=syscall_num, rdi, rsi, rdx, r10, r8, r9=args`). Complex structures passed via pointer to user memory. Kernel validates pointers are within process address space before dereference.

6. **Error Code Conventions**  
   ```rust
   pub enum KernelError {
       InvalidSyscall,
       InvalidArgument,
       PermissionDenied,
       ResourceExhausted,
       NotFound,
       AlreadyExists,
       // ... POSIX-inspired error taxonomy
   }
   ```
   Each error mapped to negative integer return code (e.g., `EINVAL = -22`). Positive return values indicate success with result data.

7. **Context Switch Semantics**  
   Each syscall invocation captures full process context: register state, instruction pointer, stack pointer. Context stored in `ProcessContext` struct. On syscall completion, restore context and resume user-mode execution.

8. **System Call Tracing & Instrumentation**  
   Every syscall invocation appends to `KernelTrace: Vec<SyscallTraceEntry>` where `SyscallTraceEntry { timestamp, pid, syscall_num, args, result, duration }`. Trace exported as JSON for performance analysis and educational visualization.

9. **Handler Complexity Management**  
   Each handler function < 50 lines, cyclomatic complexity ≤ 10. Complex operations decomposed into helper functions. Example: `sys_fork()` calls `memory::clone_address_space()`, `scheduler::create_process()`, `ipc::inherit_channels()`.

---

## 6. User Space Runtime Environment

**Core Principle**: Minimal libc-style runtime with deterministic initialization and teardown.

### High-Level Logic

1. **Process Entry Point Convention**  
   All processes start at `_start` function: `fn _start() -> !`. This function invokes process-specific `main()`, captures exit code, calls `exit(code)` syscall. No implicit process termination—all exits are explicit.

2. **Init Process (PID 1)**  
   First process created by kernel. Responsibilities: (1) Mount essential filesystems (`/proc`, `/dev`), (2) Spawn shell process, (3) Reap orphaned children. Infinite loop: `loop { wait_for_child(); reap_terminated(); }`.

3. **Shell Process Architecture**  
   Simple REPL: `loop { print!("wos$ "); let cmd = read_line(); fork_and_exec(cmd); wait_for_child(); }`. Implements basic command parsing (split on whitespace), executable lookup in `/bin`, and foreground execution semantics.

4. **Standard I/O File Descriptors**  
   Every process inherits FDs 0, 1, 2 for stdin, stdout, stderr. Mapped to browser console via WASM host calls. `read(0)` → `prompt()`, `write(1)` → `console.log()`, `write(2)` → `console.error()`.

5. **Program Loading & Execution**  
   `exec(path: &str, args: &[&str])` syscall: (1) Open executable file, (2) Parse ELF-like header, (3) Allocate memory pages for code/data/stack, (4) Copy segments into memory, (5) Reset process state, (6) Jump to entry point. MVP: Custom serialization format, not real ELF.

6. **Environment Variables**  
   Process-local `HashMap<String, String>` for environment. `getenv(key) -> Option<String>` and `setenv(key, value)`. Inherited on `fork()`, reset on `exec()`. Initial environment seeded by kernel: `PATH=/bin:/usr/bin`, `HOME=/`.

7. **Command-Line Argument Passing**  
   Arguments stored at top of stack before process start: `[argc: u32, argv[0]: *const u8, argv[1]: *const u8, ..., envp[0]: *const u8, ...]`. `main(argc, argv, envp)` accesses via stack pointer manipulation.

8. **User Space Programs (MVP)**  
   - `echo`: Reads args, prints to stdout. Demonstrates FD operations.
   - `ls`: Lists directory contents via `opendir()` / `readdir()` syscalls. Shows VFS integration.
   - `ps`: Reads `/proc` filesystem, displays running processes. Educational tool for process introspection.

9. **Signal Handling (Deferred to Post-MVP)**  
   No POSIX signals in MVP due to complexity. Post-MVP: Async signal delivery via deferred event queue. Processes check queue at syscall boundaries.

---

## 7. WebAssembly/Browser Integration Layer

**Core Principle**: Thin FFI boundary with deterministic, sandboxed host interaction.

### High-Level Logic

1. **WASM Module Structure**  
   WOS compiled to single `.wasm` module exporting: `_start() -> void` (kernel entry), `step() -> u32` (single scheduler iteration, returns status), `syscall(u32, u64, u64, u64, u64, u64, u64) -> i64` (system call interface).

2. **Linear Memory Management**  
   WASM module declares initial memory: 256 pages (16MB). Memory grows dynamically via `memory.grow` instruction up to max 1024 pages (64MB). Kernel's memory allocator (`WasmAllocator`) wraps WASM's `memory.grow`, tracks usage, enforces quotas.

3. **JavaScript Host Bindings**  
   ```javascript
   const imports = {
       env: {
           wos_console_write: (ptr, len) => { ... },
           wos_console_read: () => { return prompt(); },
           wos_timestamp: () => { return Date.now(); },
           wos_random: () => { return Math.random(); }
       }
   };
   const instance = await WebAssembly.instantiate(wasmBytes, imports);
   ```
   Host provides 4 core functions: console I/O, timestamp (for deterministic testing), random (seeded RNG for tests).

4. **Browser Event Loop Integration**  
   WOS runs in `requestAnimationFrame()` loop: each frame calls `instance.exports.step()` for one scheduler tick. Prevents blocking main thread. Paused execution when tab inactive (background tab throttling).

5. **Console-Based Terminal UI**  
   HTML `<pre id="terminal">` element displays stdout. JavaScript intercepts keyboard events, forwards to WASM via `wos_console_read()`. Cursor rendering and line editing implemented in JS, not kernel (UI concern, not OS concern).

6. **State Persistence (MVP: Omitted)**  
   No `localStorage` usage per technical review. Post-MVP: Export kernel state as JSON blob, allow manual download/upload. Auto-save to IndexedDB every 60 seconds, recovery on page reload.

7. **Deterministic Time Source**  
   For tests: `wos_timestamp()` returns simulated time from `DeterministicContext`. For browser: Returns `performance.now()` for real-time behavior. Switchable via compile feature flag: `#[cfg(test)]` vs. `#[cfg(not(test))]`.

8. **Error Reporting to Browser Console**  
   Kernel panics call `wos_panic(msg: &str) -> !` which logs to `console.error()` and halts execution. Stack traces captured via WASM debug symbols (`.wasm.map` files). Integration with browser DevTools for debugging.

9. **Performance Profiling Hooks**  
   Optional: `#[cfg(feature = "profiling")]` wraps every syscall with `performance.mark()` / `performance.measure()`. Exports trace as JSON for flame graph visualization. Overhead: ~5% when enabled.

---

## 8. Quality Infrastructure & Testing Framework

**Core Principle**: Property-based, mutation-tested, TDG-monitored extreme TDD workflow.

### High-Level Logic

1. **Property-Based State Machine Testing**  
   Uses `proptest-stateful` crate to generate sequences of syscalls, verify invariants hold after each operation. Example invariant: "Sum of all process memory ≤ total physical memory". 10,000 iterations per test.

2. **Stateful Model-Based Testing**  
   Maintain reference model (simplified scheduler simulator) alongside real kernel. Execute same syscall sequence on both, assert outputs match. Finds discrepancies between specification and implementation.

3. **Mutation Testing with Cargo-Mutants**  
   Target: 90% mutation score. Mutants focus on boundary conditions (off-by-one), comparison operators (≤ vs. <), error handling (Ok → Err). Equivalent mutants documented and excluded via `.cargo-mutants.toml`.

4. **Deterministic Execution for Tests**  
   All tests use `DeterministicContext { rng_seed: u64, simulated_time: u64 }`. Same seed → identical execution. Enables reproducible bug reports, regression testing, and test case shrinking.

5. **Coverage Tracking with Cargo-LLVM-Cov**  
   CI pipeline enforces: line coverage ≥ 85%, branch coverage ≥ 90%. Uncovered branches require explicit `#[cfg(not(test))]` annotations with justification comments.

6. **Complexity Monitoring with Cargo-Geiger**  
   Pre-commit hook blocks commits with: cyclomatic complexity > 20, cognitive complexity > 15, function lines > 50. Violations require refactoring before merge.

7. **Technical Debt Grading (TDG)**  
   Integration with PMAT toolkit: `pmat analyze tdg .` runs post-merge. Components below Grade B trigger refactoring tickets. Dashboard tracks quality trends over sprints.

8. **Continuous Integration Quality Gates**  
   GitHub Actions workflow: `make quality-gate` must pass (tests, coverage, mutation score, clippy warnings, fmt check). Failed builds block PR merge. Estimated CI time: 8-12 minutes.

9. **Time-Travel Debugging Infrastructure**  
   `KernelHistory: Vec<KernelState>` records snapshots every N syscalls. Browser UI includes "Step Back" / "Step Forward" buttons for interactive debugging. Maximum history size: 1000 snapshots (configurable).

---

## Appendix A: Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                       Browser (Host)                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              JavaScript Runtime                      │  │
│  │  - console I/O bindings                              │  │
│  │  - requestAnimationFrame() loop                      │  │
│  │  - Terminal UI rendering                             │  │
│  └────────────────┬────────────────────────────────────┬┘  │
│                   │ WASM FFI calls                     │   │
│                   ▼                                     │   │
│  ┌─────────────────────────────────────────────────────▼┐  │
│  │           WebAssembly Instance                       │  │
│  │                                                       │  │
│  │  ┌─────────────────────────────────────────────────┐ │  │
│  │  │          Kernel State (Pure Functional)         │ │  │
│  │  │  - ProcessTable                                 │ │  │
│  │  │  - MemoryManager                                │ │  │
│  │  │  - VFS & Mount Table                            │ │  │
│  │  │  - IPC Channel Registry                         │ │  │
│  │  └──────────────┬───────────────────────────────────┘ │  │
│  │                 │ State Transition                    │  │
│  │                 ▼                                     │  │
│  │  ┌──────────────────────────────────────────────────┐ │  │
│  │  │       System Call Dispatcher                     │ │  │
│  │  │  - Handler Table Lookup                          │ │  │
│  │  │  - Argument Validation                           │ │  │
│  │  └──────────────┬────────────────────────────────────┘ │  │
│  │                 │ Dispatches to                       │  │
│  │                 ▼                                     │  │
│  │  ┌──────────────────────────────────────────────────┐ │  │
│  │  │     Subsystem Handlers                           │ │  │
│  │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │ │  │
│  │  │  │Scheduler │ │ Memory   │ │   VFS    │        │ │  │
│  │  │  └──────────┘ └──────────┘ └──────────┘        │ │  │
│  │  │  ┌──────────┐ ┌──────────┐                     │ │  │
│  │  │  │   IPC    │ │ Process  │                     │ │  │
│  │  │  └──────────┘ └──────────┘                     │ │  │
│  │  └───────────────────────────────────────────────────┘ │  │
│  │                 │ Returns                            │  │
│  │                 ▼                                     │  │
│  │  ┌──────────────────────────────────────────────────┐ │  │
│  │  │     User Space Processes                         │ │  │
│  │  │  - Init (PID 1)                                  │ │  │
│  │  │  - Shell                                         │ │  │
│  │  │  - User Programs (echo, ls, ps)                 │ │  │
│  │  └───────────────────────────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Appendix B: Key Implementation Patterns

### Pattern 1: Pure State Transitions
```rust
pub fn execute_syscall(
    state: KernelState,
    pid: ProcessId,
    syscall: SystemCall,
) -> Result<(KernelState, SystemCallResult), KernelError> {
    let context = state.get_process_context(pid)?;
    let handler = state.dispatcher.get_handler(syscall.number())?;
    handler(state, context, syscall.args())
}
```

### Pattern 2: Structural Sharing for Memory Efficiency
```rust
use im::{HashMap, Vector};

#[derive(Clone)]
pub struct ProcessTable {
    processes: HashMap<ProcessId, Process>, // O(log n) clone
    ready_queue: Vector<ProcessId>,         // O(log n) updates
}
```

### Pattern 3: Property Test Invariants
```rust
proptest! {
    #[test]
    fn no_process_starvation(ops: Vec<SchedulerOp>) {
        let mut state = KernelState::new();
        let mut max_wait_times = HashMap::new();
        
        for op in ops {
            state = execute_op(state, op);
            for pid in state.ready_processes() {
                let wait_time = state.current_time() - state.last_scheduled(pid);
                max_wait_times.entry(pid).or_insert(wait_time);
            }
        }
        
        let max_wait = max_wait_times.values().max().unwrap();
        let avg_wait = max_wait_times.values().sum::<u64>() / max_wait_times.len() as u64;
        
        prop_assert!(*max_wait <= avg_wait * 2, "Process starved");
    }
}
```

---

## Appendix C: Reference Implementations

- **L4 Microkernel**: Inspiration for IPC fast-path optimization and synchronous messaging semantics.
- **QNX Neutrino**: Channel-based IPC model and capability security patterns.
- **Linux VFS**: Inode/dentry/superblock abstractions and mount point resolution logic.
- **Browsix**: WebAssembly OS concept validation and browser integration techniques.
- **WASI**: Capability-based security model for filesystem and network access.

---

## Appendix D: Performance Characteristics

| Component | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| IPC Send/Receive | O(log n) | O(m) | n = channel count, m = message size |
| Process Scheduling | O(1) amortized | O(p) | p = process count (ready queue) |
| Virtual Memory Lookup | O(log n) | O(n) | n = allocated pages (BTreeMap) |
| VFS Path Resolution | O(d·log f) | O(f) | d = path depth, f = files per dir |
| Syscall Dispatch | O(1) | O(s) | s = syscall count (HashMap lookup) |
| Fork (COW) | O(log n) | O(1) amortized | n = pages (structural sharing) |

---

**Document Status**: ✅ APPROVED for implementation commencement  
**Next Review**: Post-Phase 2 (Memory Management) completion  
**Maintenance**: Living document—update per architectural changes

---
**END OF SPECIFICATION**
