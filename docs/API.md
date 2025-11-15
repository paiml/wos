# WOS API Documentation

Complete reference for all public APIs in the WOS WebAssembly Operating System.

## Table of Contents

- [Kernel API](#kernel-api)
  - [Process Management](#process-management)
  - [Memory Management](#memory-management)
  - [File System](#file-system)
  - [Inter-Process Communication](#inter-process-communication)
- [WASM API](#wasm-api)
  - [Core Functions](#core-functions)
  - [Quality Metrics](#quality-metrics)
- [JavaScript API](#javascript-api)

---

## Kernel API

All kernel APIs follow the pure functional pattern:

```rust
fn syscall(
    state: KernelState,
    params: Params,
    calling_pid: ProcessId
) -> Result<(KernelState, Output), KernelError>
```

### Process Management

#### `sys_getpid`

Get the process ID of the calling process.

**Signature:**
```rust
pub fn sys_getpid(
    state: KernelState,
    calling_pid: ProcessId
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Returns:**
- `Ok((state, SyscallOutput::Pid(pid)))` - Success
- `Err(KernelError::ProcessNotFound)` - Process doesn't exist

**Example:**
```rust
let (new_state, output) = sys_getpid(state, 1)?;
match output {
    SyscallOutput::Pid(pid) => println!("PID: {}", pid),
    _ => unreachable!(),
}
```

#### `sys_fork`

Create a child process by duplicating the calling process.

**Signature:**
```rust
pub fn sys_fork(
    state: KernelState,
    calling_pid: ProcessId
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Behavior:**
- Creates exact copy of parent process
- Assigns new unique PID
- Establishes parent-child relationship
- Child starts in Ready state

**Returns:**
- Parent receives: `Ok((state, SyscallOutput::Pid(child_pid)))`
- Child receives: `Ok((state, SyscallOutput::Pid(0)))`
- `Err(KernelError::ProcessNotFound)` - Parent doesn't exist

**Example:**
```rust
let (new_state, output) = sys_fork(state, parent_pid)?;
match output {
    SyscallOutput::Pid(0) => {
        // We are the child
    }
    SyscallOutput::Pid(child_pid) => {
        // We are the parent
        println!("Created child PID: {}", child_pid);
    }
    _ => unreachable!(),
}
```

#### `sys_exit`

Terminate the calling process with an exit code.

**Signature:**
```rust
pub fn sys_exit(
    state: KernelState,
    calling_pid: ProcessId,
    exit_code: i32
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Behavior:**
- Sets process state to `Terminated(exit_code)`
- Reparents children to init (PID 1)
- Removes from scheduler queue
- Parent can retrieve exit code via `waitpid`

**Returns:**
- `Ok((state, SyscallOutput::Success))` - Process terminated
- `Err(KernelError::ProcessNotFound)` - Process doesn't exist

**Example:**
```rust
let (new_state, _) = sys_exit(state, pid, 0)?;
// Process is now terminated
```

#### `sys_waitpid`

Wait for a child process to terminate and retrieve its exit code.

**Signature:**
```rust
pub fn sys_waitpid(
    state: KernelState,
    calling_pid: ProcessId,
    target_pid: ProcessId
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Behavior:**
- Blocks if child is still running
- Returns immediately if child already terminated
- Only works for direct children

**Returns:**
- `Ok((state, SyscallOutput::ExitCode(code)))` - Child terminated
- `Err(KernelError::PermissionDenied)` - Not parent of target
- `Err(KernelError::ProcessNotFound)` - Target doesn't exist

**Example:**
```rust
let (new_state, output) = sys_waitpid(state, parent_pid, child_pid)?;
match output {
    SyscallOutput::ExitCode(code) => {
        println!("Child exited with code: {}", code);
    }
    _ => unreachable!(),
}
```

---

### Memory Management

#### `sys_mmap`

Allocate virtual memory pages.

**Signature:**
```rust
pub fn sys_mmap(
    state: KernelState,
    calling_pid: ProcessId,
    size: usize,
    permissions: PagePermissions
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Parameters:**
- `size` - Number of bytes to allocate (rounded up to page size)
- `permissions` - Bitflags: `READ | WRITE | EXECUTE`

**Behavior:**
- Allocates from heap region
- Returns virtual address
- Size rounded up to 4KB pages
- Addresses are sequential and non-overlapping

**Returns:**
- `Ok((state, SyscallOutput::Address(addr)))` - Success
- `Err(KernelError::OutOfMemory)` - No free pages
- `Err(KernelError::ProcessNotFound)` - Process doesn't exist

**Example:**
```rust
let (new_state, output) = sys_mmap(
    state,
    pid,
    8192,  // 2 pages
    PagePermissions::READ | PagePermissions::WRITE
)?;
match output {
    SyscallOutput::Address(addr) => {
        println!("Allocated at: 0x{:x}", addr);
    }
    _ => unreachable!(),
}
```

#### `sys_munmap`

Free virtual memory pages.

**Signature:**
```rust
pub fn sys_munmap(
    state: KernelState,
    calling_pid: ProcessId,
    addr: VirtualAddress,
    size: usize
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Parameters:**
- `addr` - Virtual address to free (must be page-aligned)
- `size` - Number of bytes to free (rounded up to page size)

**Behavior:**
- Frees pages in range [addr, addr + size)
- All pages must be currently mapped
- Fails if any page is unmapped

**Returns:**
- `Ok((state, SyscallOutput::Success))` - Pages freed
- `Err(KernelError::InvalidAddress)` - Address not mapped
- `Err(KernelError::ProcessNotFound)` - Process doesn't exist

**Example:**
```rust
let (new_state, _) = sys_munmap(state, pid, addr, 8192)?;
// Pages are now free
```

---

### File System

#### `sys_open`

Open a file and return a file descriptor.

**Signature:**
```rust
pub fn sys_open(
    state: KernelState,
    calling_pid: ProcessId,
    path: &str,
    flags: OpenFlags
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Parameters:**
- `path` - File path (absolute or relative)
- `flags` - `O_RDONLY | O_WRONLY | O_RDWR`

**Standard Descriptors:**
- `0` - stdin
- `1` - stdout
- `2` - stderr

**Returns:**
- `Ok((state, SyscallOutput::FileDescriptor(fd)))` - Success
- `Err(KernelError::FileNotFound)` - File doesn't exist
- `Err(KernelError::PermissionDenied)` - No read/write permission

**Example:**
```rust
let (new_state, output) = sys_open(
    state,
    pid,
    "/proc/1/status",
    OpenFlags::O_RDONLY
)?;
match output {
    SyscallOutput::FileDescriptor(fd) => {
        println!("Opened fd: {}", fd);
    }
    _ => unreachable!(),
}
```

#### `sys_close`

Close a file descriptor.

**Signature:**
```rust
pub fn sys_close(
    state: KernelState,
    calling_pid: ProcessId,
    fd: FileDescriptor
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Behavior:**
- Releases file descriptor
- Cannot close stdin/stdout/stderr (0, 1, 2)

**Returns:**
- `Ok((state, SyscallOutput::Success))` - Closed
- `Err(KernelError::InvalidFileDescriptor)` - FD doesn't exist
- `Err(KernelError::PermissionDenied)` - Cannot close std streams

#### `sys_read`

Read data from a file descriptor.

**Signature:**
```rust
pub fn sys_read(
    state: KernelState,
    calling_pid: ProcessId,
    fd: FileDescriptor,
    count: usize
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Returns:**
- `Ok((state, SyscallOutput::Data(bytes)))` - Data read
- `Err(KernelError::PermissionDenied)` - No read permission

#### `sys_write`

Write data to a file descriptor.

**Signature:**
```rust
pub fn sys_write(
    state: KernelState,
    calling_pid: ProcessId,
    fd: FileDescriptor,
    data: Vec<u8>
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Returns:**
- `Ok((state, SyscallOutput::BytesWritten(n)))` - Bytes written
- `Err(KernelError::PermissionDenied)` - No write permission

---

### Inter-Process Communication

#### `sys_send`

Send a message to another process.

**Signature:**
```rust
pub fn sys_send(
    state: KernelState,
    calling_pid: ProcessId,
    target_pid: ProcessId,
    message: Vec<u8>
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Behavior:**
- Adds message to target's queue
- Messages delivered in FIFO order
- No size limit (in-memory)

**Returns:**
- `Ok((state, SyscallOutput::Success))` - Message sent
- `Err(KernelError::ProcessNotFound)` - Target doesn't exist

#### `sys_recv`

Receive a message from the queue.

**Signature:**
```rust
pub fn sys_recv(
    state: KernelState,
    calling_pid: ProcessId,
    timeout: Option<Duration>
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Behavior:**
- Returns oldest message from queue
- Blocks if queue empty (in real implementation)
- Returns immediately if message available

**Returns:**
- `Ok((state, SyscallOutput::Message(sender, data)))` - Message received
- `Err(KernelError::WouldBlock)` - No messages (if non-blocking)

---

## WASM API

JavaScript-callable functions exported via wasm-bindgen.

### Core Functions

#### `WosWasm::new()`

Create a new WOS instance with initial kernel state.

**JavaScript:**
```javascript
import { WosWasm } from './wos.js';
const wos = new WosWasm();
```

#### `executeSyscall(syscall_json, calling_pid)`

Execute a system call.

**Parameters:**
- `syscall_json: string` - JSON-serialized SystemCall
- `calling_pid: number` - Process ID making the call

**Returns:**
- `string` - JSON-serialized SyscallOutput

**JavaScript:**
```javascript
const syscall = JSON.stringify({ GetPid: null });
const result = wos.executeSyscall(syscall, 1);
const output = JSON.parse(result);
console.log('PID:', output.Pid);
```

#### `executeCommand(command)`

Execute a shell command (high-level interface).

**Parameters:**
- `command: string` - Command to execute

**Returns:**
- `string` - Command output

**JavaScript:**
```javascript
const output = wos.executeCommand('ps');
console.log(output);
```

#### `getState()`

Get current kernel state as JSON.

**Returns:**
- `string` - JSON-serialized KernelState

**JavaScript:**
```javascript
const stateJson = wos.getState();
const state = JSON.parse(stateJson);
console.log('Process count:', Object.keys(state.processes).length);
```

#### `setState(state_json)`

Restore kernel state from JSON.

**Parameters:**
- `state_json: string` - JSON-serialized KernelState

**JavaScript:**
```javascript
const savedState = localStorage.getItem('wos-state');
wos.setState(savedState);
```

#### `processCount()`

Get number of active processes.

**Returns:**
- `number` - Process count

**JavaScript:**
```javascript
const count = wos.processCount();
console.log(`${count} processes running`);
```

#### `reset()`

Reset kernel to initial state.

**JavaScript:**
```javascript
wos.reset();
```

### Quality Metrics

#### `getQualityMetrics()`

Get quality metrics as JSON.

**Returns:**
- `string` - JSON-serialized QualityMetrics

**JavaScript:**
```javascript
const metricsJson = wos.getQualityMetrics();
const metrics = JSON.parse(metricsJson);
console.log('TDG Grade:', metrics.tdg_grade);
console.log('Test Count:', metrics.test_count);
console.log('Coverage:', metrics.coverage);
```

#### `exportQualityHtml()`

Generate HTML quality report.

**Returns:**
- `string` - Complete HTML document

**JavaScript:**
```javascript
const html = wos.exportQualityHtml();
const blob = new Blob([html], { type: 'text/html' });
const url = URL.createObjectURL(blob);
window.open(url);
```

#### `exportQualityMarkdown()`

Generate Markdown quality report.

**Returns:**
- `string` - Markdown document

#### `exportQualitySarif()`

Generate SARIF quality report.

**Returns:**
- `string` - JSON-formatted SARIF report

---

## JavaScript API

Terminal application API (in `app.js`).

### Terminal Class

#### `new Terminal()`

Create terminal instance.

```javascript
const terminal = new Terminal();
```

#### `executeCommand(cmd)`

Execute command in terminal.

```javascript
terminal.executeCommand('help');
```

#### `printLine(text, className)`

Print line to terminal.

**Parameters:**
- `text: string` - Text to display
- `className: string` - CSS class ('output', 'error', 'success', 'command')

```javascript
terminal.printLine('Hello, WOS!', 'output');
terminal.printLine('Error occurred', 'error');
terminal.printLine('Success!', 'success');
```

#### `clear()`

Clear terminal output.

```javascript
terminal.clear();
```

#### `saveState()`

Save kernel state to localStorage.

```javascript
terminal.saveState();
```

#### `loadState()`

Load kernel state from localStorage.

```javascript
terminal.loadState();
```

#### `reset()`

Reset kernel to initial state.

```javascript
terminal.reset();
```

#### `exportQualityJson()`

Download quality metrics as JSON.

```javascript
terminal.exportQualityJson();
```

---

## Error Handling

All kernel functions return `Result<T, KernelError>`.

### KernelError Variants

```rust
pub enum KernelError {
    ProcessNotFound,
    InvalidAddress,
    OutOfMemory,
    PermissionDenied,
    FileNotFound,
    InvalidFileDescriptor,
    WouldBlock,
    InvalidSyscall,
}
```

### Error Handling Pattern

```rust
match sys_fork(state, pid) {
    Ok((new_state, output)) => {
        // Success - use new_state and output
    }
    Err(KernelError::ProcessNotFound) => {
        // Handle missing process
    }
    Err(e) => {
        // Handle other errors
        eprintln!("Error: {:?}", e);
    }
}
```

---

## Testing APIs

All APIs are thoroughly tested:

- **Unit Tests**: Each function has 5+ tests
- **Property Tests**: Invariants verified with 10K+ iterations
- **Integration Tests**: End-to-end syscall pipelines
- **Test Coverage**: 88%+ across all modules

See `*/tests/` directories for examples.

---

## Performance

All APIs are optimized for performance:

- **Syscall Dispatch**: <10μs per call
- **State Clone**: O(1) using persistent data structures
- **Memory Allocation**: O(log n) via HashMap
- **Scheduler**: O(1) round-robin selection

Run benchmarks:
```bash
cargo bench
```

---

## Version Information

```rust
// Get version strings
let version = wos_version();           // "WOS v0.1.0 ..."
let kernel_ver = kernel_version();     // "0.1.0"
let userspace_ver = userspace_version(); // "0.1.0"
```

---

## References

- [Architecture Guide](ARCHITECTURE.md)
- [Tutorials](tutorials/)
- Source Code (see repository)
- [Specification](specifications/wos-spec-v1.md)
