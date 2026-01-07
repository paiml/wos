# WOS v2.0 Specification: Streamlined Pure WASM Educational OS

## Zero JavaScript, Probador TDD, APR-Enhanced Microkernel with Virtualization

**Version**: 2.0.0-draft
**Status**: SPECIFICATION
**Date**: 2026-01-05
**Author**: Claude Code (Anthropic)

---

## Executive Summary

WOS v2.0 is a complete architectural reimagining of the WebOS educational operating system, adopting a **ZERO JavaScript** mandate with **pure WASM** browser execution. Drawing from the PAIML Sovereign AI Stack (probador, batuta, pepita, pzsh, simular), this specification defines:

1. **Pure WASM Architecture**: Zero .js/.ts files - all DOM interaction via Rust→WASM→Web APIs
2. **Probador-First TDD**: 100% test coverage using jugar-probar with GUI coverage tracking
3. **APR Model Runtime**: Kernel-level support for .apr (Aprender Portable Runtime) execution
4. **Pepita-Inspired Virtualization**: Basic MicroVM primitives for process isolation
5. **Full PMAT + bashrs Compliance**: Quality gates with zero tolerance for violations

---

## 1. Architectural Principles

### 1.1 Zero JavaScript Mandate

**ABSOLUTE REQUIREMENT**: No JavaScript files in the project.

```
FORBIDDEN (DELETE IMMEDIATELY):
├── *.js              # ALL JavaScript files
├── *.ts              # ALL TypeScript files
├── *.mjs             # ES modules
├── *.cjs             # CommonJS modules
├── package.json      # Node.js package manifest
├── package-lock.json # Node.js lock file
├── node_modules/     # Node.js dependencies
├── playwright.config.*  # Playwright configs
├── jest.config.*     # Jest configs
├── cypress/          # Cypress directories
├── *.spec.js         # JS test files
└── npm/yarn/pnpm usage

ALLOWED:
├── *.rs          # Pure Rust
├── *.wasm        # Compiled WASM
├── *.html        # Static HTML (no inline JS)
├── *.css         # Stylesheets
└── *.apr         # APR model files
```

### 1.1.1 Testing Framework: Probador Only

**DELETE ALL PLAYWRIGHT/JEST/CYPRESS** - Replace with Probador:

```bash
# FORBIDDEN commands (must be removed from all scripts)
npm test
npx playwright test
npx jest
npx cypress run

# REQUIRED commands (Probador-based)
cargo nextest run          # Unit tests
probador test --all        # E2E tests
probador test --browser    # Browser automation (Chrome CDP)
probador test --gui-coverage  # GUI coverage tracking
```

### 1.2 Pure WASM DOM Interaction

Following simular's architecture but eliminating the JavaScript layer entirely:

```rust
// WOS uses web-sys and wasm-bindgen for direct DOM access
use web_sys::{Document, Element, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Pure WASM entry point - no JavaScript bootstrapping
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    // Create terminal UI entirely in Rust
    let terminal = Terminal::new(&document)?;
    let kernel = Kernel::new()?;

    // Event loop via requestAnimationFrame - all in WASM
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        kernel.tick();
        terminal.render();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
```

### 1.3 Presentar Integration

Use the `presentar` crate (from PAIML stack) for zero-JS web UI:

```toml
# Cargo.toml
[dependencies]
presentar = { version = "0.3", features = ["terminal", "canvas"] }
web-sys = { version = "0.3", features = [
    "Document", "Element", "HtmlElement", "HtmlCanvasElement",
    "CanvasRenderingContext2d", "KeyboardEvent", "MouseEvent",
    "Window", "Performance", "Storage", "console"
]}
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
```

---

## 2. Probador-First Testing Architecture

### 2.1 Testing Stack

```
┌─────────────────────────────────────────────────────────────┐
│                    probador CLI (probador)                   │
├─────────────────────────────────────────────────────────────┤
│   jugar-probar Library                                       │
│   ├── BrowserController (Chrome CDP)                         │
│   ├── WasmRuntime (wasmtime)                                │
│   ├── TuiTestBackend (ratatui)                              │
│   └── MockDriver (in-memory)                                │
├─────────────────────────────────────────────────────────────┤
│   Coverage Modules                                           │
│   ├── UX Coverage (GUI elements & interactions)             │
│   ├── Pixel Coverage (visual heatmaps)                      │
│   ├── Code Coverage (llvm-cov integration)                  │
│   └── Mutation Coverage (cargo-mutants)                     │
├─────────────────────────────────────────────────────────────┤
│   APR Integration                                            │
│   ├── Deterministic Replay (.apr files)                     │
│   ├── State Snapshots                                       │
│   └── Reproducibility Verification                          │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Test Requirements

| Metric | Threshold | Enforcement |
|--------|-----------|-------------|
| Line Coverage | ≥95% | cargo-llvm-cov |
| Branch Coverage | ≥90% | cargo-llvm-cov |
| Mutation Score | ≥90% | cargo-mutants |
| GUI Coverage | 100% | jugar-probar::gui_coverage! |
| Accessibility | WCAG 2.1 AA | probador accessibility auditor |

### 2.3 Probador Test Example

```rust
use jugar_probar::prelude::*;

#[tokio::test]
async fn test_wos_fork_syscall() {
    // Track GUI coverage
    let mut coverage = gui_coverage! {
        syscalls: ["fork", "exec", "wait", "exit"],
        screens: ["terminal", "process_list", "memory_map"]
    };

    // Launch WASM runtime
    let mut driver = WasmRuntime::new();
    driver.load_wasm("target/wasm32-unknown-unknown/release/wos_bg.wasm").await?;

    // Initialize kernel
    driver.invoke("init").await?;
    coverage.visit("terminal");

    // Test fork syscall
    let result = driver.invoke("syscall", json!({
        "op": "Fork"
    })).await?;
    coverage.click("fork");

    // Verify process created
    let state: KernelState = driver.get_state().await?;
    assert_eq!(state.processes.len(), 2, "Fork should create child process");

    // Verify PID assignment
    let child = state.processes.iter().find(|p| p.parent_pid == Some(1)).unwrap();
    assert_eq!(child.pid, 2, "Child should have PID 2");

    // Check GUI coverage
    coverage.visit("process_list");
    assert!(coverage.meets(80.0), "GUI coverage should be ≥80%");

    // APR snapshot for reproducibility
    let snapshot = driver.snapshot().await?;
    snapshot.save("tests/fixtures/fork_test.apr")?;
}
```

### 2.4 APR Model Integration

The `.apr` (Aprender Portable Runtime) format enables:

1. **Deterministic Replay**: Reproduce exact test scenarios
2. **State Snapshots**: Capture kernel state at any point
3. **Seed Control**: ChaCha8-based deterministic RNG
4. **Cross-Platform Verification**: Bit-identical results

```rust
// APR file structure
pub struct AprModel {
    pub version: String,           // "1.0.0"
    pub seed: u64,                 // Deterministic seed
    pub initial_state: KernelState,
    pub inputs: Vec<TimestampedInput>,
    pub expected_outputs: Vec<ExpectedOutput>,
    pub checkpoints: Vec<StateCheckpoint>,
}

// Kernel APR integration
impl Kernel {
    pub fn load_apr(&mut self, path: &Path) -> Result<(), KernelError> {
        let model: AprModel = serde_json::from_reader(File::open(path)?)?;
        self.rng = ChaCha8Rng::seed_from_u64(model.seed);
        self.state = model.initial_state;
        Ok(())
    }

    pub fn snapshot_apr(&self) -> AprModel {
        AprModel {
            version: "1.0.0".to_string(),
            seed: self.seed,
            initial_state: self.state.clone(),
            inputs: self.input_log.clone(),
            expected_outputs: vec![],
            checkpoints: vec![StateCheckpoint::new(&self.state)],
        }
    }
}
```

---

## 3. Kernel Architecture

### 3.1 Microkernel Design (Pepita-Inspired)

Drawing from pepita's minimal kernel interfaces:

```
┌─────────────────────────────────────────────────────────────┐
│                    WOS Microkernel (~3000 lines)             │
├─────────────────────────────────────────────────────────────┤
│  Process Manager        │  Memory Manager                   │
│  ├── Process Table      │  ├── Virtual Address Space        │
│  ├── Scheduler          │  ├── Page Tables (simulated)      │
│  └── Context Switch     │  └── Memory Mapping               │
├─────────────────────────┼───────────────────────────────────┤
│  System Call Dispatcher │  IPC Primitives                   │
│  ├── POSIX-like API     │  ├── Message Passing              │
│  ├── File Operations    │  ├── Shared Memory (im-rs)        │
│  └── Process Control    │  └── Signals                      │
├─────────────────────────┼───────────────────────────────────┤
│  Virtual Devices        │  Virtualization Layer             │
│  ├── /dev/null          │  ├── MicroVM Abstraction          │
│  ├── /dev/zero          │  ├── Guest Isolation              │
│  ├── /dev/random        │  └── Resource Limits              │
│  └── /dev/console       │                                   │
└─────────────────────────┴───────────────────────────────────┘
```

### 3.2 Type-Safe Kernel Primitives

Following pepita's poka-yoke patterns:

```rust
// Type-safe address wrappers (no confusion between physical/virtual)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtAddr(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysAddr(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pfn(u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }
    pub const fn as_u64(&self) -> u64 { self.0 }
    pub const fn page_offset(&self) -> u64 { self.0 & (PAGE_SIZE - 1) }
    pub const fn page_number(&self) -> Pfn { Pfn(self.0 / PAGE_SIZE) }
}

// Type-safe process/task IDs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileDescriptor(u32);

// Cannot accidentally use ProcessId where FileDescriptor expected
```

### 3.3 System Call Interface

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemCall {
    // Process Management
    Fork,
    Exec { path: PathBuf, args: Vec<String>, env: Vec<(String, String)> },
    Exit { code: i32 },
    WaitPid { pid: ProcessId, options: WaitOptions },
    GetPid,
    GetPpid,
    Kill { pid: ProcessId, signal: Signal },

    // File I/O
    Open { path: PathBuf, flags: OpenFlags, mode: FileMode },
    Close { fd: FileDescriptor },
    Read { fd: FileDescriptor, count: usize },
    Write { fd: FileDescriptor, data: Vec<u8> },
    Seek { fd: FileDescriptor, offset: i64, whence: SeekFrom },
    Stat { path: PathBuf },
    Fstat { fd: FileDescriptor },

    // Directory Operations
    Mkdir { path: PathBuf, mode: FileMode },
    Rmdir { path: PathBuf },
    Readdir { fd: FileDescriptor },
    Chdir { path: PathBuf },
    Getcwd,

    // IPC
    Pipe,
    Dup { fd: FileDescriptor },
    Dup2 { oldfd: FileDescriptor, newfd: FileDescriptor },

    // Memory (virtualization-ready)
    Mmap { addr: Option<VirtAddr>, length: usize, prot: MemoryProtection, flags: MmapFlags },
    Munmap { addr: VirtAddr, length: usize },
    Mprotect { addr: VirtAddr, length: usize, prot: MemoryProtection },

    // Virtualization
    VmCreate { config: VmConfig },
    VmStart { vm_id: VmId },
    VmStop { vm_id: VmId },
    VmStatus { vm_id: VmId },
}
```

---

## 4. Virtualization Layer

### 4.1 MicroVM Abstraction

Inspired by pepita's vmm module, WOS implements educational virtualization:

```rust
/// MicroVM configuration for guest OS isolation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmConfig {
    pub memory_mb: u32,           // Guest memory (simulated)
    pub vcpus: u8,                // Virtual CPUs
    pub kernel_path: PathBuf,     // Guest kernel (APR model)
    pub initrd_path: Option<PathBuf>,
    pub cmdline: String,          // Kernel command line
    pub devices: Vec<VirtioDevice>,
}

/// Virtual machine state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VmState {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed(String),
}

/// MicroVM instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicroVm {
    pub id: VmId,
    pub config: VmConfig,
    pub state: VmState,
    pub memory: GuestMemory,
    pub vcpu_states: Vec<VcpuState>,
    pub devices: Vec<VirtioDeviceState>,
    pub exit_code: Option<i32>,
}

impl MicroVm {
    /// Create a new MicroVM
    pub fn create(config: VmConfig) -> Result<Self, VmError> {
        // Allocate guest memory (simulated via im::HashMap for educational purposes)
        let memory = GuestMemory::new(config.memory_mb as usize * 1024 * 1024)?;

        // Initialize vCPUs
        let vcpu_states = (0..config.vcpus)
            .map(|id| VcpuState::new(id))
            .collect();

        Ok(Self {
            id: VmId::new(),
            config,
            state: VmState::Created,
            memory,
            vcpu_states,
            devices: vec![],
            exit_code: None,
        })
    }

    /// Load APR model as guest kernel
    pub fn load_apr_kernel(&mut self, apr: &AprModel) -> Result<(), VmError> {
        // Load APR initial state into guest memory
        self.memory.write(KERNEL_LOAD_ADDR, &apr.serialize()?)?;
        Ok(())
    }

    /// Step VM execution (educational - one instruction at a time)
    pub fn step(&mut self) -> Result<VmExitReason, VmError> {
        match self.state {
            VmState::Running => {
                // Execute next instruction on active vCPU
                let vcpu = &mut self.vcpu_states[0];
                vcpu.execute_one(&mut self.memory)?;

                // Check for VM exits (I/O, halt, etc.)
                if let Some(exit) = vcpu.pending_exit() {
                    return Ok(exit);
                }

                Ok(VmExitReason::Continue)
            }
            _ => Err(VmError::InvalidState(self.state.clone())),
        }
    }
}
```

### 4.2 Virtio Device Emulation

Educational implementation of virtio for guest I/O:

```rust
/// Virtio device types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VirtioDevice {
    Console(VirtioConsole),
    Block(VirtioBlock),
    Net(VirtioNet),
    Vsock(VirtioVsock),
}

/// Virtio console for guest terminal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtioConsole {
    pub tx_queue: VirtQueue,
    pub rx_queue: VirtQueue,
    pub buffer: Vec<u8>,
}

impl VirtioConsole {
    pub fn write(&mut self, data: &[u8]) -> Result<usize, VirtioError> {
        // Queue data for guest to read
        self.buffer.extend_from_slice(data);
        Ok(data.len())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, VirtioError> {
        // Read from guest's tx queue
        let n = self.tx_queue.pop(buf)?;
        Ok(n)
    }
}

/// Virtio block device (backed by VFS)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtioBlock {
    pub capacity_sectors: u64,
    pub queue: VirtQueue,
    pub backing: PathBuf,  // Path in host VFS
}
```

### 4.3 Guest Memory Management

```rust
/// Guest physical memory abstraction
#[derive(Clone, Debug)]
pub struct GuestMemory {
    pages: im::HashMap<Pfn, Page>,
    total_bytes: usize,
}

impl GuestMemory {
    pub fn new(size_bytes: usize) -> Result<Self, VmError> {
        Ok(Self {
            pages: im::HashMap::new(),
            total_bytes: size_bytes,
        })
    }

    pub fn read(&self, addr: PhysAddr, buf: &mut [u8]) -> Result<(), VmError> {
        let pfn = addr.page_number();
        let offset = addr.page_offset() as usize;

        if let Some(page) = self.pages.get(&pfn) {
            let end = (offset + buf.len()).min(PAGE_SIZE as usize);
            buf[..end-offset].copy_from_slice(&page.data[offset..end]);
            Ok(())
        } else {
            // Zero-fill for unallocated pages
            buf.fill(0);
            Ok(())
        }
    }

    pub fn write(&mut self, addr: PhysAddr, data: &[u8]) -> Result<(), VmError> {
        let pfn = addr.page_number();
        let offset = addr.page_offset() as usize;

        let page = self.pages.entry(pfn).or_insert_with(Page::zero);
        let end = (offset + data.len()).min(PAGE_SIZE as usize);
        page.data[offset..end].copy_from_slice(&data[..end-offset]);
        Ok(())
    }
}
```

---

## 5. Pure Functional Design

### 5.1 Immutable State with im-rs

All kernel state uses persistent data structures:

```rust
use im::{HashMap, Vector, OrdMap};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KernelState {
    // Process table (immutable clone on write)
    pub processes: HashMap<ProcessId, Process>,

    // File system (persistent tree)
    pub vfs: VirtualFileSystem,

    // Open file descriptors per process
    pub open_files: HashMap<ProcessId, HashMap<FileDescriptor, OpenFile>>,

    // Memory maps per process
    pub memory_maps: HashMap<ProcessId, OrdMap<VirtAddr, MemoryMapping>>,

    // IPC message queues
    pub message_queues: HashMap<ProcessId, Vector<Message>>,

    // MicroVMs (virtualization layer)
    pub vms: HashMap<VmId, MicroVm>,

    // Global monotonic counters
    pub next_pid: ProcessId,
    pub next_fd: FileDescriptor,
    pub next_vm_id: VmId,

    // Deterministic RNG state
    pub rng_state: [u8; 32],

    // Simulation time
    pub clock: SimulatedClock,
}

impl KernelState {
    /// Pure functional state transition
    pub fn apply(self, syscall: &SystemCall, ctx: &Context) -> Result<(Self, SyscallResult), KernelError> {
        match syscall {
            SystemCall::Fork => self.do_fork(ctx),
            SystemCall::Exec { path, args, env } => self.do_exec(ctx, path, args, env),
            SystemCall::Exit { code } => self.do_exit(ctx, *code),
            // ... all syscalls are pure transformations
        }
    }
}
```

### 5.2 Jidoka Guards

Toyota Way principle: stop on anomaly detection

```rust
/// Jidoka guard for kernel invariants
pub struct KernelJidokaGuard {
    checks: Vec<InvariantCheck>,
    violations: Vec<Violation>,
}

pub enum InvariantCheck {
    ProcessCountLimit(usize),
    MemoryLimit(usize),
    FdLimit(usize),
    NoOrphanProcesses,
    NoZombieBloat,
    VmMemoryBounds,
    DeterministicRng,
}

impl KernelJidokaGuard {
    pub fn check(&mut self, state: &KernelState) -> JidokaStatus {
        self.violations.clear();

        for check in &self.checks {
            if let Err(violation) = check.verify(state) {
                self.violations.push(violation);
            }
        }

        if self.violations.is_empty() {
            JidokaStatus::Ok
        } else {
            // STOP THE LINE - invariant violated
            JidokaStatus::Halt(self.violations.clone())
        }
    }
}

// Used in kernel tick
impl Kernel {
    pub fn tick(&mut self) -> Result<(), KernelError> {
        // Execute pending syscalls
        self.process_syscalls()?;

        // Jidoka check
        match self.jidoka.check(&self.state) {
            JidokaStatus::Ok => Ok(()),
            JidokaStatus::Halt(violations) => {
                // Log violations, pause execution
                for v in &violations {
                    console_error!("JIDOKA HALT: {}", v);
                }
                Err(KernelError::JidokaHalt(violations))
            }
        }
    }
}
```

---

## 6. APR Runtime in Kernel

### 6.1 APR Execution Engine

The kernel can load and execute `.apr` models for deterministic simulation:

```rust
/// APR model execution within WOS kernel
pub struct AprRuntime {
    model: AprModel,
    state: AprState,
    input_index: usize,
    checkpoint_index: usize,
}

impl AprRuntime {
    pub fn load(path: &Path) -> Result<Self, AprError> {
        let data = std::fs::read(path)?;
        let model: AprModel = serde_json::from_slice(&data)?;

        Ok(Self {
            state: AprState::from_initial(&model.initial_state),
            model,
            input_index: 0,
            checkpoint_index: 0,
        })
    }

    /// Execute next input from APR model
    pub fn step(&mut self) -> Result<Option<AprOutput>, AprError> {
        if self.input_index >= self.model.inputs.len() {
            return Ok(None);
        }

        let input = &self.model.inputs[self.input_index];
        let output = self.state.apply(input)?;

        // Verify against expected output if present
        if let Some(expected) = self.model.expected_outputs.get(self.input_index) {
            if output != *expected {
                return Err(AprError::OutputMismatch {
                    index: self.input_index,
                    expected: expected.clone(),
                    actual: output.clone(),
                });
            }
        }

        self.input_index += 1;
        Ok(Some(output))
    }

    /// Check against checkpoint
    pub fn verify_checkpoint(&self) -> Result<(), AprError> {
        if let Some(checkpoint) = self.model.checkpoints.get(self.checkpoint_index) {
            let current_hash = self.state.hash();
            if current_hash != checkpoint.state_hash {
                return Err(AprError::CheckpointMismatch {
                    index: self.checkpoint_index,
                    expected: checkpoint.state_hash,
                    actual: current_hash,
                });
            }
        }
        Ok(())
    }
}
```

### 6.2 APR Model Format

**File Format**: `.apr` extension ONLY (JSON content)

- Location: `/models/*.apr` (bundled with WOS)
- External: `../apr-cookbook/*.apr` (user models)
- Format: JSON with `.apr` extension (NOT `.apr.json`)

**Bundled Models**:
- `/models/tutorial.apr` - Interactive WOS tutorial
- `/models/demo-session.apr` - Demo session recording
- `/models/vm-demo.apr` - MicroVM virtualization demo

**APR Shell Commands**:
```bash
apr list      # List available .apr models
apr run <f>   # Load and execute .apr model
apr status    # Show APR runtime status
apr step      # Execute single deterministic step
apr demo      # Run built-in demo model
```

```rust
/// APR (Aprender Portable Runtime) Model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AprModel {
    /// Version identifier
    pub version: String,  // "1.0.0"

    /// Format identifier
    pub format: AprFormat,  // "wos-kernel-state"

    /// Deterministic seed for RNG
    pub seed: u64,

    /// Initial kernel/simulation state
    pub initial_state: serde_json::Value,

    /// Timestamped inputs
    pub inputs: Vec<TimestampedInput>,

    /// Expected outputs for verification
    pub expected_outputs: Vec<AprOutput>,

    /// State checkpoints for verification
    pub checkpoints: Vec<StateCheckpoint>,

    /// Metadata
    pub metadata: AprMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimestampedInput {
    pub tick: u64,
    pub input: AprInput,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AprInput {
    Syscall(SystemCall),
    KeyPress(char),
    Command(String),
    Timer(u64),
    Signal(Signal),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AprOutput {
    pub tick: u64,
    pub output_type: String,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateCheckpoint {
    pub tick: u64,
    pub state_hash: [u8; 32],  // Blake3 hash
    pub description: String,
}
```

---

## 7. PMAT + bashrs Compliance

### 7.1 PMAT Quality Gates

```toml
# .pmat-gates.toml
[quality]
rust_project_score = 90

[testing]
unit_coverage_threshold = 95
branch_coverage_threshold = 90
mutation_score_threshold = 90
property_tests = true
e2e_tests = true

[complexity]
max_cyclomatic_complexity = 10
max_cognitive_complexity = 15
max_function_lines = 50

[satd]
zero_tolerance = true  # No TODO/FIXME comments

[documentation]
doc_coverage = 100
require_examples = true

[security]
forbid_unsafe = true
audit_dependencies = true
```

### 7.2 bashrs Makefile Compliance

All Makefile targets must pass bashrs quality checks:

```makefile
# Makefile - bashrs compliant

.PHONY: all build test coverage quality wasm serve clean

# Default target
all: fmt lint test

# Build targets
build:
	cargo build --release --all-features

build-wasm:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/wos.wasm \
		--out-dir dist/wos --target web --no-typescript

# Quality gates
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo nextest run --all-features --workspace

coverage:
	cargo llvm-cov --all-features --workspace --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

coverage-check:
	cargo llvm-cov --all-features --workspace --fail-under-lines 95

mutation:
	cargo mutants --workspace

mutation-check:
	cargo mutants --workspace --minimum-kill-rate 90

# Probador testing
probador-test:
	probador test --all

probador-gui-coverage:
	probador test --gui-coverage --fail-under 100

# PMAT gates
pmat-gates:
	pmat check --config .pmat-gates.toml

pmat-satd:
	pmat satd --zero-tolerance

pmat-complexity:
	pmat complexity --max-cyclomatic 10 --max-cognitive 15

# bashrs validation
bashrs-check:
	bashrs lint Makefile
	bashrs score Makefile

bashrs-audit:
	bashrs audit scripts/

# Full quality pipeline
quality: fmt-check lint test coverage-check mutation-check pmat-gates bashrs-check

# Development server (ruchy - NOT python http.server)
serve:
	ruchy serve dist/wos --port 8000 --watch --watch-wasm --verbose

# Clean
clean:
	cargo clean
	rm -rf dist/wos/*.wasm dist/wos/*.js
```

### 7.3 Quality Gate Enforcement

```yaml
# .github/workflows/quality.yml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy, llvm-tools-preview

      - name: Install tools
        run: |
          cargo install cargo-nextest cargo-llvm-cov cargo-mutants
          cargo install probador pmat bashrs ruchy

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Lint check
        run: cargo clippy --all-features -- -D warnings

      - name: Unit tests
        run: cargo nextest run --all-features

      - name: Coverage check (≥95%)
        run: cargo llvm-cov --fail-under-lines 95

      - name: Mutation check (≥90%)
        run: cargo mutants --minimum-kill-rate 90

      - name: PMAT gates
        run: pmat check --config .pmat-gates.toml

      - name: bashrs check
        run: bashrs audit .

      - name: Probador E2E tests
        run: probador test --all --gui-coverage
```

---

## 8. Project Structure

```
wos/
├── Cargo.toml                    # Workspace root
├── Makefile                      # bashrs-compliant build
├── .pmat-gates.toml              # PMAT quality config
├── roadmap.yaml                  # Implementation roadmap
├── CLAUDE.md                     # Development guide
│
├── docs/
│   ├── specifications/
│   │   ├── wos-spec-v1.md        # Original spec
│   │   └── streamlined-*.md      # This document
│   └── tickets/
│
├── shared/                       # Shared infrastructure (~1000 lines)
│   └── src/
│       ├── lib.rs
│       ├── vfs.rs                # Virtual filesystem (im-rs)
│       ├── context.rs            # Execution context
│       ├── apr.rs                # APR model types
│       └── types.rs              # Type-safe primitives
│
├── kernel/                       # Microkernel (~3000 lines)
│   └── src/
│       ├── lib.rs
│       ├── process.rs            # Process management
│       ├── scheduler.rs          # Round-robin scheduler
│       ├── memory.rs             # Virtual memory
│       ├── syscall.rs            # System call dispatcher
│       ├── ipc.rs                # IPC primitives
│       ├── device.rs             # Virtual devices
│       ├── jidoka.rs             # Invariant checking
│       ├── vmm.rs                # MicroVM manager
│       └── apr_runtime.rs        # APR execution
│
├── userspace/                    # User space (~1500 lines)
│   └── src/
│       ├── lib.rs
│       ├── init.rs               # Init process (PID 1)
│       ├── shell.rs              # Shell
│       └── programs/             # User programs
│           ├── echo.rs
│           ├── ls.rs
│           ├── ps.rs
│           ├── cat.rs
│           ├── kill.rs
│           └── apr_run.rs        # APR model runner
│
├── wos/                          # WASM entry + UI (~800 lines)
│   └── src/
│       ├── lib.rs                # OS integration
│       ├── wasm.rs               # WASM exports (NO JS)
│       ├── dom.rs                # Pure WASM DOM manipulation
│       ├── terminal.rs           # Terminal renderer
│       └── canvas.rs             # Canvas graphics
│
├── dist/                         # Web distribution (NO .js files)
│   └── wos/
│       ├── index.html            # Static HTML (no inline JS)
│       ├── style.css
│       └── wos_bg.wasm           # Built WASM binary
│
└── tests/                        # Tests (~3000 lines)
    ├── unit/
    ├── integration/
    ├── properties/               # Property-based (proptest)
    ├── e2e/                      # Probador E2E tests
    ├── fixtures/                 # APR test models
    │   └── *.apr
    └── gui_coverage/             # GUI coverage tests
```

---

## 9. Implementation Roadmap

### Phase 1: Pure WASM Foundation (Weeks 1-3)

| Ticket | Description | Tests |
|--------|-------------|-------|
| WOS-100 | Remove all JavaScript, setup pure WASM | Build passes |
| WOS-101 | Integrate presentar for DOM | Terminal renders |
| WOS-102 | Pure WASM event handling | Key/mouse events work |
| WOS-103 | Canvas rendering in Rust | Graphics render |
| WOS-104 | Probador test infrastructure | E2E tests run |

### Phase 2: Enhanced Kernel (Weeks 4-6)

| Ticket | Description | Tests |
|--------|-------------|-------|
| WOS-110 | Type-safe kernel primitives | Unit tests |
| WOS-111 | Jidoka guards | Invariant tests |
| WOS-112 | APR model types | Serialization tests |
| WOS-113 | APR runtime integration | Model execution |
| WOS-114 | Enhanced syscall interface | Syscall tests |

### Phase 3: Virtualization (Weeks 7-9)

| Ticket | Description | Tests |
|--------|-------------|-------|
| WOS-120 | MicroVM abstraction | VM creation |
| WOS-121 | Guest memory management | Memory tests |
| WOS-122 | VirtIO console | I/O works |
| WOS-123 | VirtIO block | Block device |
| WOS-124 | VM lifecycle | Start/stop |

### Phase 4: Quality & Polish (Weeks 10-12)

| Ticket | Description | Tests |
|--------|-------------|-------|
| WOS-130 | 95% test coverage | Coverage report |
| WOS-131 | 90% mutation score | Mutation report |
| WOS-132 | 100% GUI coverage | Probador report |
| WOS-133 | PMAT gates pass | All gates green |
| WOS-134 | bashrs compliance | Score ≥9.0 |
| WOS-135 | Documentation | All APIs documented |

---

## 10. Testing Strategy

### 10.1 Probador Test Matrix

```rust
// tests/e2e/kernel_tests.rs
use jugar_probar::prelude::*;

mod kernel_e2e {
    use super::*;

    #[tokio::test]
    async fn test_process_lifecycle() {
        let mut coverage = gui_coverage! {
            syscalls: ["fork", "exec", "wait", "exit"],
            screens: ["terminal", "process_list"]
        };

        let driver = WasmRuntime::new();
        driver.load_wasm(WOS_WASM_PATH).await?;

        // Test fork
        driver.invoke("syscall", json!({"op": "Fork"})).await?;
        coverage.click("fork");

        // Test exec
        driver.invoke("syscall", json!({
            "op": "Exec",
            "path": "/bin/echo",
            "args": ["hello"]
        })).await?;
        coverage.click("exec");

        // Test wait
        driver.invoke("syscall", json!({"op": "WaitPid", "pid": 2})).await?;
        coverage.click("wait");

        // Verify coverage
        assert!(coverage.complete(), "All syscalls must be tested");
    }

    #[tokio::test]
    async fn test_apr_model_execution() {
        let driver = WasmRuntime::new();
        driver.load_wasm(WOS_WASM_PATH).await?;

        // Load APR model
        let apr = include_str!("../fixtures/shell_session.apr");
        driver.invoke("load_apr", json!({"model": apr})).await?;

        // Execute all steps
        loop {
            let result = driver.invoke("apr_step", json!({})).await?;
            if result.is_null() { break; }
        }

        // Verify final state matches APR checkpoints
        let state: KernelState = driver.get_state().await?;
        assert!(state.apr_checkpoints_valid());
    }
}
```

### 10.2 Property-Based Tests

```rust
// tests/properties/kernel_props.rs
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn kernel_never_panics_on_syscall(
        syscall in arb_syscall(),
        state in arb_kernel_state()
    ) {
        let result = std::panic::catch_unwind(|| {
            state.apply(&syscall, &Context::default())
        });
        prop_assert!(result.is_ok(), "Kernel must never panic");
    }

    #[test]
    fn fork_creates_exactly_one_child(
        state in arb_kernel_state().prop_filter("has processes", |s| !s.processes.is_empty())
    ) {
        let initial_count = state.processes.len();
        let (new_state, _) = state.apply(&SystemCall::Fork, &Context::init()).unwrap();
        prop_assert_eq!(new_state.processes.len(), initial_count + 1);
    }

    #[test]
    fn memory_operations_preserve_invariants(
        ops in prop::collection::vec(arb_memory_op(), 1..100)
    ) {
        let mut memory = GuestMemory::new(1024 * 1024).unwrap();
        for op in ops {
            let _ = memory.apply(op);  // May fail, but must not panic
        }
        prop_assert!(memory.is_valid(), "Memory invariants must hold");
    }
}
```

### 10.3 GUI Coverage Requirements

```rust
// tests/gui_coverage/full_coverage.rs
use jugar_probar::prelude::*;

#[tokio::test]
async fn verify_100_percent_gui_coverage() {
    let mut coverage = gui_coverage! {
        // All terminal elements
        terminal: ["prompt", "cursor", "command_line", "output_area"],

        // All syscalls
        syscalls: [
            "fork", "exec", "exit", "wait",
            "open", "close", "read", "write",
            "mkdir", "rmdir", "chdir", "getcwd"
        ],

        // All views
        views: [
            "terminal", "process_list", "memory_map",
            "syscall_trace", "vm_status", "apr_runner"
        ],

        // All keyboard shortcuts
        shortcuts: ["ctrl_l", "ctrl_c", "arrow_up", "arrow_down", "tab"],

        // All user programs
        programs: ["echo", "ls", "ps", "cat", "kill", "apr_run"]
    };

    let driver = BrowserController::launch().await?;
    driver.navigate("http://localhost:8000").await?;

    // Exercise all features...
    // [comprehensive test sequence]

    // Verify 100% coverage
    let report = coverage.report();
    assert!(
        report.coverage >= 100.0,
        "GUI coverage must be 100%, got {}%",
        report.coverage
    );
}
```

---

## 11. Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| WASM Binary Size | <200KB gzipped | `wc -c dist/wos/wos_bg.wasm.gz` |
| Cold Start | <100ms | probador benchmark |
| System Call Latency | <10μs | unit benchmark |
| Context Switch | <50μs | unit benchmark |
| DOM Update | <16ms (60fps) | requestAnimationFrame timing |
| APR Step | <1ms | probador benchmark |
| Memory Usage | <10MB | browser dev tools |

---

## 12. Summary

WOS v2.0 represents a fundamental evolution:

| Aspect | v1.0 (DEPRECATED) | v2.0 |
|--------|-------------------|------|
| JavaScript | Used for bootstrap | **ZERO** - Completely eliminated |
| Testing | JS-based (removed) | **Probador (Pure Rust)** |
| Coverage | 85% | ≥95% |
| E2E Testing | Node.js required | **Probador + jugar-probar** (no Node.js) |
| Virtualization | None | MicroVM layer |
| APR Support | None | Full runtime |
| PMAT Compliance | Partial | Full |
| bashrs Compliance | Partial | Full |
| GUI Coverage | Manual | 100% automated via Probador |

### Testing Framework Migration

**CRITICAL**: All JavaScript-based testing frameworks are **DELETED** and replaced with Probador:

| REMOVED (v1.0) | REPLACEMENT (v2.0) |
|----------------|-------------------|
| ~~Playwright~~ | `probador test --browser` |
| ~~Jest~~ | `cargo nextest run` |
| ~~Cypress~~ | `probador test --e2e` |
| ~~node_modules/~~ | Pure Rust crates |
| ~~package.json~~ | Cargo.toml only |
| ~~npm test~~ | `make probador-test` |

**Probador Advantages**:
- Zero JavaScript dependencies
- Pure Rust implementation
- Chrome CDP integration (no Node.js)
- Native WASM runtime testing via wasmtime
- Integrated GUI coverage tracking
- APR model-based deterministic replay

**Key Innovations**:
1. **Pure WASM browser execution** via presentar + web-sys
2. **Probador-first testing** with GUI coverage tracking
3. **APR model integration** for deterministic replay
4. **Educational virtualization** with MicroVM abstraction
5. **Full quality gate compliance** (PMAT + bashrs)

This specification creates a unique educational platform demonstrating:
- Modern Rust systems programming
- Operating system fundamentals
- Virtualization concepts
- Deterministic testing methodology
- Quality engineering practices

---

## 13. Peer-Reviewed Citations & Academic Basis

The architectural decisions in WOS v2.0 are grounded in foundational computer science research and modern systems engineering principles.

### 13.1 Systems & Microkernels
1.  **Liedtke, J. (1995).** "On Micro-Kernel Construction." *15th ACM Symposium on Operating Systems Principles (SOSP)*.
    *   *Application*: Principles of minimality used in the Pepita-inspired kernel, pushing mechanisms to userspace.
2.  **Herder, J. N., et al. (2006).** "Fault Isolation for Device Drivers." *Dependable Systems and Networks (DSN)*.
    *   *Application*: Isolation strategies for virtual devices and MicroVMs.

### 13.2 WebAssembly & Runtime
3.  **Haas, A., Rossberg, A., Schuff, D. L., et al. (2017).** "Bringing the Web up to Speed with WebAssembly." *Proceedings of the 38th ACM SIGPLAN Conference on Programming Language Design and Implementation (PLDI)*.
    *   *Application*: Validation of WASM as a high-performance, safe compilation target for OS emulation.
4.  **Jangda, A., et al. (2019).** "Not So Fast: Analyzing the Performance of WebAssembly vs. Native Code." *USENIX ATC*.
    *   *Application*: Performance baselines for the "Performance Targets" section.

### 13.3 Formal Methods & Determinism
5.  **Popper, K. (1963).** *Conjectures and Refutations: The Growth of Scientific Knowledge*. Routledge.
    *   *Application*: The "Poppian Falsification Checklist" methodology—testing seeks to falsify the hypothesis that the system is correct, rather than confirm it.
6.  **Shannon, C. E. (1948).** "A Mathematical Theory of Communication." *Bell System Technical Journal*.
    *   *Application*: Information theoretic basis for APR state serialization and entropy management in deterministic replay.

### 13.4 Performance & Reliability
7.  **Gregg, B. (2020).** *Systems Performance: Enterprise and the Cloud, 2nd Edition*. Addison-Wesley.
    *   *Application*: USE Method (Utilization, Saturation, Errors) applied to Jidoka guards and performance metrics.
8.  **Collet, A. (2021).** *Rust in Action*. Manning Publications.
    *   *Application*: Idiomatic Rust patterns for systems programming, type safety, and memory management without garbage collection.

---

## 14. Poppian Falsification Checklist (The 100-Point QA Matrix)

To certify WOS v2.0, the system must survive these 100 falsification attempts. If *any* attempt succeeds in breaking the defined invariants, the system is rejected.

### Category A: Core Architecture (1-10)
1.  [ ] **Zero JS**: Search entire source tree for `.js` files. > 0 files = FAIL.
2.  [ ] **Zero TS**: Search entire source tree for `.ts` files. > 0 files = FAIL.
3.  [ ] **WASM Only**: Verify `dist/` contains only `.wasm`, `.html`, `.css`. Any script tag in HTML = FAIL.
4.  [ ] **Build Deps**: Inspect `Cargo.toml`. Usage of `npm`, `yarn`, or `node` in build scripts = FAIL.
5.  [ ] **Startup**: Block all network requests. If app fails to boot (offline mode) = FAIL.
6.  [ ] **Browser Compat**: Run in Firefox, Chrome, Safari. Any panic/crash = FAIL.
7.  [ ] **Resize**: Resize browser window rapidly for 10s. Layout break or panic = FAIL.
8.  [ ] **Reload**: Hard refresh 100 times. Any state corruption or hang = FAIL.
9.  [ ] **Console Clean**: Open DevTools. Any JS console error or warning = FAIL.
10. [ ] **Memory**: Monitor heap for 1 hour of idle. Growth > 1MB = FAIL (Leak).

### Category B: Kernel Stability (11-25)
11. [ ] **Panic Safety**: Insert `panic!` in random syscall. Kernel must catch and log, not crash WASM = FAIL if crash.
12. [ ] **OOM Handling**: Force 100% memory allocation. Kernel must return ENOMEM, not panic = FAIL if panic.
13. [ ] **Zero Pointer**: Dereference NULL in userspace. Process crash OK, Kernel crash = FAIL.
14. [ ] **Stack Overflow**: Infinite recursion in userspace. Process crash OK, Kernel crash = FAIL.
15. [ ] **Rapid Fork**: Fork 1000 processes in loop. Kernel panic or freeze = FAIL.
16. [ ] **Zombie Flood**: Create 1000 zombies (no wait). Jidoka must halt or reap = FAIL if uncontrolled growth.
17. [ ] **FD Exhaustion**: Open 10,000 files. Return EMFILE, no panic = FAIL if panic.
18. [ ] **Invalid Syscall**: Call syscall #-1 and #9999. Return ENOSYS, no panic = FAIL.
19. [ ] **Bad Arguments**: Pass invalid pointers to all syscalls. Return EFAULT, no panic = FAIL.
20. [ ] **Buffer Overread**: Read past end of buffer in syscall. No kernel memory leak = FAIL if leak.
21. [ ] **Deadlock**: Create circular IPC dependency. Scheduler must detect or timeout = FAIL if freeze.
22. [ ] **Starvation**: High priority loop vs low priority task. Low priority must get *some* CPU = FAIL if 0 cycles.
23. [ ] **Clock Drift**: Check `clock` against host wall time after 1 hour. Drift > 1s = FAIL.
24. [ ] **Entropy**: Check `/dev/random` output. Constant or repeating pattern = FAIL.
25. [ ] **Jidoka Halt**: Manually violate invariant (e.g., duplicate PID). System must HALT immediately = FAIL if continues.

### Category C: Process Management (26-35)
26. [ ] **PID 1**: Kill PID 1. Kernel panic (expected) or immediate reboot. Hanging = FAIL.
27. [ ] **Isolation**: Process A writes to Process B memory. Success = FAIL.
28. [ ] **Env Vars**: Set 1MB env var. Process spawn failure or truncation OK, Panic = FAIL.
29. [ ] **Argv Max**: Pass 10,000 arguments. Process spawn failure OK, Panic = FAIL.
30. [ ] **Tree Kill**: Kill parent with cascading children. Orphans must be re-parented to PID 1 = FAIL if lost.
31. [ ] **Priority**: Set invalid priority. Error returned, no panic = FAIL.
32. [ ] **Exec Fail**: Exec non-existent binary. Original process must remain or exit cleanly = FAIL if corrupted state.
33. [ ] **Concurrent Wait**: Two processes wait on same PID. One succeeds, one fails/waits = FAIL if undefined/crash.
34. [ ] **Self Kill**: Process kills itself. Immediate exit = FAIL if zombie/hang.
35. [ ] **State Dump**: Request state dump during heavy load. Serialization failure = FAIL.

### Category D: Memory & VFS (36-45)
36. [ ] **Double Free**: Userspace double free. Process SEGV, Kernel OK = FAIL if Kernel panic.
37. [ ] **Use After Free**: Userspace UAF. Undefined userspace behavior OK, Kernel compromise = FAIL.
38. [ ] **Mmap Bounds**: Map beyond address space. Error returned = FAIL if allowed.
39. [ ] **Prot Violation**: Write to ReadOnly page. SIGSEGV = FAIL if write succeeds.
40. [ ] **File Persistence**: Write file, reboot (reload), read file. Data missing = FAIL (if persistence enabled).
41. [ ] **Path Traversal**: Open `../../etc/passwd` (simulated). Access outside root = FAIL.
42. [ ] **Name Length**: Create file with 256 char name. Error or truncation = FAIL if buffer overflow.
43. [ ] **Open Many**: Open same file 1000 times. Distinct FDs = FAIL if shared offset confusion.
44. [ ] **Seek Past End**: Seek to max i64. Read returns EOF, write extends (sparse) = FAIL if panic.
45. [ ] **Cross-Mount**: Rename file across mount points. Error or copy-delete = FAIL if corrupted.

### Category E: Virtualization (46-55)
46. [ ] **VM Escape**: Guest writes to Host memory. Success = FAIL.
47. [ ] **VM Starvation**: Guest consumes 100% CPU. Host UI must remain responsive = FAIL if UI freeze.
48. [ ] **Device Fuzz**: Send random bytes to VirtIO console. VM panic OK, Host panic = FAIL.
49. [ ] **Network Spam**: Guest floods virtual net. Host rate limit must engage = FAIL if Host DOS.
50. [ ] **VM Reset**: Reset VM 100 times quickly. Resource leak (RAM/FDs) = FAIL.
51. [ ] **Bad Config**: Start VM with 0 RAM. Error returned = FAIL if panic.
52. [ ] **Double Start**: Start already running VM. Error returned = FAIL.
53. [ ] **Snapshot Restore**: Snapshot VM, run, restore. State must be bit-identical = FAIL.
54. [ ] **Instruction Trap**: Guest executes privileged instruction. VM exit/trap = FAIL if executed.
55. [ ] **Nested VM**: Attempt to start VM inside VM. Fail gracefully = FAIL if host crash.

### Category F: APR & Determinism (56-65)
56. [ ] **Replay Exactness**: Record session, replay. Final state bit-difference = FAIL.
57. [ ] **Seed Sensitivity**: Change RNG seed by 1 bit. Execution path *must* diverge = FAIL if identical.
58. [ ] **Platform Indep**: Record on Linux, replay on Mac. Divergence = FAIL.
59. [ ] **Version Check**: Load v1.0 APR in v2.0 runtime. Version mismatch error = FAIL if crash/corruption.
60. [ ] **Corrupt APR**: Fuzz APR file (flip bits). Loader returns error, no panic = FAIL.
61. [ ] **Long Replay**: Replay 1-hour session. Desync > 0 = FAIL.
62. [ ] **Checkpoint Verify**: Manually alter state during replay. Checkpoint validation must fail = FAIL if ignored.
63. [ ] **Input Injection**: Inject extra input during replay. Detection error = FAIL if accepted.
64. [ ] **Time Travel**: Snapshot, run 10s, restore, run 10s. End states must match = FAIL.
65. [ ] **Export JSON**: Export state to JSON. Invalid JSON syntax = FAIL.

### Category G: Probador Testing (66-75)
66. [ ] **Coverage Hole**: Find 1 line of code not covered by tests. Success = FAIL.
67. [ ] **Mutation Survive**: Run cargo-mutants. Any mutant survives > 100 = FAIL.
68. [ ] **Flaky Test**: Run suite 100 times. Any failure = FAIL.
69. [ ] **GUI Miss**: Identify GUI element not clicked by tests. Success = FAIL.
70. [ ] **Slow Test**: Single test takes > 1s. Success = FAIL (must be fast).
71. [ ] **Mock Leak**: Mock state persists between tests. Success = FAIL.
72. [ ] **False Positive**: Break code, run test. Test passes = FAIL.
73. [ ] **Doc Test**: Run doc tests. Failure = FAIL.
74. [ ] **Lint Error**: Run clippy. Any warning = FAIL.
75. [ ] **Format**: Run rustfmt. Any change = FAIL.

### Category H: Performance & Limits (76-85)
76. [ ] **Boot Time**: Measure boot to prompt. > 100ms = FAIL.
77. [ ] **FPS Drop**: Run heavy process. UI FPS < 30 = FAIL.
78. [ ] **Input Latency**: Type fast. Render lag > 1 frame = FAIL.
79. [ ] **Binary Size**: Gzip build. > 200KB = FAIL.
80. [ ] **Syscall Rate**: Measure syscalls/sec. < 100k = FAIL.
81. [ ] **Context Switch**: Measure switch time. > 50μs = FAIL.
82. [ ] **Disk IO**: Write 10MB to VFS. Freeze > 100ms = FAIL.
83. [ ] **Max Procs**: Spawn max processes. System stable = FAIL if crash.
84. [ ] **Max Memory**: Allocate max memory. System stable = FAIL if crash.
85. [ ] **Compile Time**: Clean build. > 60s = FAIL (Development velocity).

### Category I: Security & Safety (86-95)
86. [ ] **Unsafe Audit**: Search `unsafe`. Justification missing = FAIL.
87. [ ] **XSS Check**: Input `<script>alert(1)</script>` in terminal. Execution = FAIL.
88. [ ] **Clipboard**: Copy/Paste payload. Execution = FAIL.
89. [ ] **Origin**: Check CORS/Origin policies. Permissive = FAIL (if applicable).
90. [ ] **Dep Audit**: `cargo audit` finds vulnerability. Success = FAIL.
91. [ ] **Secret Leak**: Search binary for env secrets. Found = FAIL.
92. [ ] **Privilege Esc**: `su` without password (if implemented). Success = FAIL.
93. [ ] **File Perms**: Read root-only file as user. Success = FAIL.
94. [ ] **Signal Spam**: Spam SIGKILL to init. System panic = FAIL.
95. [ ] **Resource Hog**: User process fork bomb. System lockup = FAIL.

### Category J: UX & Accessibility (96-100)
96. [ ] **No Mouse**: Complete typical workflow using only keyboard. Stuck = FAIL.
97. [ ] **Screen Reader**: Use with screen reader (ARIA). Unintelligible = FAIL.
98. [ ] **Contrast**: Check color contrast. WCAG AA fail = FAIL.
99. [ ] **Zoom**: Browser zoom 200%. Layout broken = FAIL.
100. [ ] **Mobile**: Open on mobile device. Unusable = FAIL.

---

**Document Version**: 2.0.0-draft
**Last Updated**: 2026-01-05
**Status**: Ready for Implementation
