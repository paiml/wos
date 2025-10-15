# Tutorial 1: Adding a New Syscall

Learn how to extend WOS with a new system call following TDD best practices.

## Prerequisites

- Rust basics (structs, enums, Result, match)
- Understanding of WOS architecture (read [ARCHITECTURE.md](../ARCHITECTURE.md))
- Familiarity with the pure functional pattern

## Goal

Add a `sys_getppid` syscall that returns the parent process ID of the calling process.

## Step 1: Design the Interface

Every syscall follows the same pattern:

```rust
fn syscall(
    state: KernelState,
    params: Params,
    calling_pid: ProcessId
) -> Result<(KernelState, SyscallOutput), KernelError>
```

For `sys_getppid`:
- **Input**: `KernelState`, `calling_pid`
- **Output**: Parent PID or error
- **Errors**: `ProcessNotFound` if process doesn't exist

## Step 2: Add the Syscall Enum Variant

Edit `kernel/src/syscall.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemCall {
    GetPid,
    GetPpid,  // Add this line
    Fork,
    Exit(i32),
    // ... rest
}
```

## Step 3: Write Tests First (TDD)

Create `kernel/tests/syscall_getppid.rs`:

```rust
use kernel::*;
use shared::types::ProcessId;

#[test]
fn test_getppid_success() {
    // Setup: Create init process (PID 1)
    let state = KernelState::new();

    // Fork to create child (PID 2)
    let (state, output) = sys_fork(state, 1).unwrap();
    let child_pid = match output {
        SyscallOutput::Pid(pid) => pid,
        _ => panic!("Expected Pid"),
    };

    // Test: Get parent of child process
    let (state, output) = sys_getppid(state, child_pid).unwrap();

    // Assert: Parent should be init (PID 1)
    match output {
        SyscallOutput::Pid(ppid) => assert_eq!(ppid, 1),
        _ => panic!("Expected Pid output"),
    }
}

#[test]
fn test_getppid_init_process() {
    let state = KernelState::new();

    // Test: Init process (PID 1) has no parent (returns 0)
    let (state, output) = sys_getppid(state, 1).unwrap();

    match output {
        SyscallOutput::Pid(ppid) => assert_eq!(ppid, 0),
        _ => panic!("Expected Pid output"),
    }
}

#[test]
fn test_getppid_process_not_found() {
    let state = KernelState::new();

    // Test: Non-existent process
    let result = sys_getppid(state, 999);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), KernelError::ProcessNotFound);
}

#[test]
fn test_getppid_orphaned_process() {
    let state = KernelState::new();

    // Create child
    let (state, output) = sys_fork(state, 1).unwrap();
    let child_pid = match output {
        SyscallOutput::Pid(pid) => pid,
        _ => panic!("Expected Pid"),
    };

    // Parent exits (child reparented to init)
    let (state, _) = sys_exit(state, 1, 0).unwrap();

    // Child's parent should now be init
    let (_, output) = sys_getppid(state, child_pid).unwrap();
    match output {
        SyscallOutput::Pid(ppid) => assert_eq!(ppid, 1),
        _ => panic!("Expected Pid output"),
    }
}

#[test]
fn test_getppid_immutability() {
    let state = KernelState::new();
    let state_clone = state.clone();

    // Call getppid
    let (new_state, _) = sys_getppid(state, 1).unwrap();

    // Original state unchanged
    assert_eq!(state_clone.processes.len(), 1);

    // New state also unchanged (read-only operation)
    assert_eq!(new_state.processes.len(), 1);
}
```

Run the tests (they should fail):

```bash
cargo test --test syscall_getppid
```

Expected output:
```
error[E0425]: cannot find function `sys_getppid` in this scope
```

Perfect! This is TDD - write tests first, then implement.

## Step 4: Implement the Syscall

Edit `kernel/src/syscall.rs`:

```rust
/// Get parent process ID
pub fn sys_getppid(
    state: KernelState,
    calling_pid: ProcessId,
) -> Result<(KernelState, SyscallOutput), KernelError> {
    // Validate process exists
    let process = state
        .processes
        .get(&calling_pid)
        .ok_or(KernelError::ProcessNotFound)?;

    // Return parent PID (0 if no parent)
    let ppid = process.parent_pid.unwrap_or(0);

    Ok((state, SyscallOutput::Pid(ppid)))
}
```

## Step 5: Update the Syscall Dispatcher

Edit `kernel/src/syscall.rs` in the `dispatch_syscall` function:

```rust
pub fn dispatch_syscall(
    state: KernelState,
    syscall: SystemCall,
    calling_pid: ProcessId,
) -> Result<(KernelState, SyscallOutput), KernelError> {
    match syscall {
        SystemCall::GetPid => sys_getpid(state, calling_pid),
        SystemCall::GetPpid => sys_getppid(state, calling_pid),  // Add this
        SystemCall::Fork => sys_fork(state, calling_pid),
        // ... rest
    }
}
```

## Step 6: Run Tests

```bash
cargo test --test syscall_getppid
```

Expected output:
```
running 5 tests
test test_getppid_success ... ok
test test_getppid_init_process ... ok
test test_getppid_process_not_found ... ok
test test_getppid_orphaned_process ... ok
test test_getppid_immutability ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All tests pass!

## Step 7: Add Property Tests

Edit `kernel/tests/syscall_getppid.rs`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_getppid_always_returns_valid_pid(seed in 0u64..1000) {
        let state = KernelState::new();

        // Fork some processes
        let mut current_state = state;
        let mut pids = vec![1];

        for _ in 0..seed % 10 {
            let (new_state, output) = sys_fork(current_state, 1).unwrap();
            if let SyscallOutput::Pid(pid) = output {
                pids.push(pid);
            }
            current_state = new_state;
        }

        // Every process should have a valid parent
        for pid in pids {
            let (_, output) = sys_getppid(current_state.clone(), pid).unwrap();
            if let SyscallOutput::Pid(ppid) = output {
                // Parent is either 0 (init) or a valid process
                prop_assert!(ppid == 0 || current_state.processes.contains_key(&ppid));
            } else {
                panic!("Expected Pid output");
            }
        }
    }

    #[test]
    fn prop_getppid_never_modifies_state(seed in 0u64..1000) {
        let state = KernelState::new();
        let state_before = state.clone();

        // Call getppid
        let (state_after, _) = sys_getppid(state, 1).unwrap();

        // State should be identical
        prop_assert_eq!(
            state_before.processes.len(),
            state_after.processes.len()
        );
        prop_assert_eq!(
            state_before.next_pid,
            state_after.next_pid
        );
    }
}
```

Run property tests:

```bash
cargo test --test syscall_getppid -- --nocapture
```

## Step 8: Update Documentation

Add to `docs/API.md`:

```markdown
#### `sys_getppid`

Get the parent process ID of the calling process.

**Signature:**
```rust
pub fn sys_getppid(
    state: KernelState,
    calling_pid: ProcessId
) -> Result<(KernelState, SyscallOutput), KernelError>
```

**Returns:**
- `Ok((state, SyscallOutput::Pid(ppid)))` - Parent PID (0 if no parent)
- `Err(KernelError::ProcessNotFound)` - Process doesn't exist

**Example:**
```rust
let (new_state, output) = sys_getppid(state, child_pid)?;
match output {
    SyscallOutput::Pid(ppid) => {
        println!("Parent PID: {}", ppid);
    }
    _ => unreachable!(),
}
```
```

## Step 9: Add WASM Binding (Optional)

If you want to expose this to JavaScript, edit `wos/src/lib.rs`:

```rust
/// Get parent process ID
#[wasm_bindgen(js_name = getParentPid)]
pub fn get_parent_pid(&mut self, pid: u32) -> Result<u32, JsValue> {
    let syscall = SystemCall::GetPpid;
    let result = dispatch_syscall(self.state.clone(), syscall, pid)
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    self.state = result.0;

    match result.1 {
        SyscallOutput::Pid(ppid) => Ok(ppid),
        _ => Err(JsValue::from_str("Unexpected output type")),
    }
}
```

## Step 10: Verify Quality Gates

Run all quality checks:

```bash
make quality
```

Expected:
- ✅ Formatting passes
- ✅ Clippy passes
- ✅ All tests pass (now 267 tests)
- ✅ Coverage increases

## Summary

You've successfully added a new syscall following WOS best practices:

1. ✅ **Design First** - Clear interface specification
2. ✅ **TDD** - Tests written before implementation
3. ✅ **Pure Functional** - No mutation, returns new state
4. ✅ **Error Handling** - Proper Result types
5. ✅ **Property Testing** - Invariants verified
6. ✅ **Documentation** - API reference updated
7. ✅ **Quality Gates** - All checks pass

## Next Steps

- [Tutorial 2: Creating a New Program](02-creating-program.md)
- [Tutorial 3: Understanding the Scheduler](03-understanding-scheduler.md)
- [API Reference](../API.md)
- [Architecture Guide](../ARCHITECTURE.md)

## Exercise

Try implementing `sys_getuid` (get user ID) following the same pattern:
- Returns the user ID of the calling process
- For simplicity, all processes have UID 0 (root) in WOS v0.1
- Add 5+ unit tests
- Add 2+ property tests
- Update documentation

## Common Pitfalls

1. **Forgetting to clone state** - Always use persistent data structures
2. **Mutating process fields** - Use `insert()` to create new version
3. **Wrong error type** - Match the specific error to the failure case
4. **Missing tests** - Aim for 5+ tests per syscall
5. **Not checking quality gates** - Run `make quality` before committing

## Further Reading

- [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Property Testing with Proptest](https://altsysrq.github.io/proptest-book/intro.html)
- [Pure Functional Programming](https://en.wikipedia.org/wiki/Purely_functional_programming)
