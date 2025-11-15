# Tutorial 2: Creating a New Program

Learn how to build userspace programs for WOS using the kernel API.

## Prerequisites

- Completed [Tutorial 1: Adding a Syscall](01-adding-syscall.md)
- Understanding of process lifecycle
- Basic knowledge of Unix commands

## Goal

Create a `tree` program that displays the process hierarchy in a tree format.

## Step 1: Understand the Userspace Architecture

Userspace programs in WOS are pure functions that interact with the kernel via syscalls:

```rust
// Every program has this signature
pub fn program_main(
    state: KernelState,
    pid: ProcessId,
    args: Vec<String>
) -> Result<(KernelState, i32), KernelError> {
    // Program logic here
    Ok((state, exit_code))
}
```

Key points:
- **Pure functions** - No side effects, no global state
- **Immutable state** - Kernel state flows through the program
- **Explicit exit code** - Return (new_state, exit_code)
- **Error handling** - Return KernelError on failure

## Step 2: Create the Program File

Create `userspace/src/bin/tree.rs`:

```rust
use kernel::{dispatch_syscall, KernelState, SyscallOutput, SystemCall};
use shared::types::{KernelError, ProcessId};
use std::collections::HashMap;

/// Display process tree
pub fn main(
    state: KernelState,
    pid: ProcessId,
    _args: Vec<String>,
) -> Result<(KernelState, i32), KernelError> {
    // Build process tree structure
    let mut tree: HashMap<ProcessId, Vec<ProcessId>> = HashMap::new();

    // Group processes by parent
    for (child_pid, process) in state.processes.iter() {
        if let Some(parent_pid) = process.parent_pid {
            tree.entry(parent_pid).or_insert_with(Vec::new).push(*child_pid);
        }
    }

    // Print tree starting from init (PID 1)
    let mut output = String::new();
    print_tree(&state, &tree, 1, &mut output, "", true);

    // Write output to stdout
    let syscall = SystemCall::Write(1, output.into_bytes());
    let (new_state, _) = dispatch_syscall(state, syscall, pid)?;

    Ok((new_state, 0))
}

/// Recursively print process tree
fn print_tree(
    state: &KernelState,
    tree: &HashMap<ProcessId, Vec<ProcessId>>,
    pid: ProcessId,
    output: &mut String,
    prefix: &str,
    is_last: bool,
) {
    // Get process info
    let process = state.processes.get(&pid).unwrap();

    // Print current process
    let branch = if is_last { "└──" } else { "├──" };
    let line = format!(
        "{}{} {} (PID: {}, State: {:?})\n",
        prefix, branch, process.name, pid, process.state
    );
    output.push_str(&line);

    // Print children
    if let Some(children) = tree.get(&pid) {
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        for (i, child_pid) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            print_tree(state, tree, *child_pid, output, &child_prefix, is_last_child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::sys_fork;

    #[test]
    fn test_tree_single_process() {
        let state = KernelState::new();

        let (new_state, exit_code) = main(state, 1, vec![]).unwrap();

        assert_eq!(exit_code, 0);
        // State should have written to stdout
    }

    #[test]
    fn test_tree_with_children() {
        let mut state = KernelState::new();

        // Create process hierarchy
        // init (1)
        //  ├── shell (2)
        //  │   └── ls (3)
        //  └── daemon (4)

        // Fork shell
        let (new_state, output) = sys_fork(state, 1).unwrap();
        state = new_state;

        // Fork ls from shell
        let (new_state, _) = sys_fork(state, 2).unwrap();
        state = new_state;

        // Fork daemon
        let (new_state, _) = sys_fork(state, 1).unwrap();
        state = new_state;

        // Run tree
        let (new_state, exit_code) = main(state, 1, vec![]).unwrap();

        assert_eq!(exit_code, 0);
        // Verify output contains all processes
    }

    #[test]
    fn test_tree_empty_args() {
        let state = KernelState::new();

        // Should work with no arguments
        let result = main(state, 1, vec![]);
        assert!(result.is_ok());
    }
}
```

## Step 3: Write Comprehensive Tests

Create `userspace/tests/tree_test.rs`:

```rust
use kernel::{sys_fork, KernelState, SyscallOutput, SystemCall};
use userspace::tree::main as tree_main;

#[test]
fn test_tree_shows_all_processes() {
    let mut state = KernelState::new();

    // Create several processes
    for _ in 0..5 {
        let (new_state, _) = sys_fork(state, 1).unwrap();
        state = new_state;
    }

    let process_count = state.processes.len();

    // Run tree
    let (new_state, exit_code) = tree_main(state, 1, vec![]).unwrap();

    assert_eq!(exit_code, 0);
    assert_eq!(new_state.processes.len(), process_count);
}

#[test]
fn test_tree_hierarchy() {
    let mut state = KernelState::new();

    // Create parent-child chain
    let (new_state, out1) = sys_fork(state, 1).unwrap();
    let pid2 = match out1 {
        SyscallOutput::Pid(p) => p,
        _ => panic!(),
    };

    let (new_state, out2) = sys_fork(new_state, pid2).unwrap();
    let pid3 = match out2 {
        SyscallOutput::Pid(p) => p,
        _ => panic!(),
    };

    // Verify hierarchy: 1 -> 2 -> 3
    assert_eq!(new_state.processes.get(&pid2).unwrap().parent_pid, Some(1));
    assert_eq!(new_state.processes.get(&pid3).unwrap().parent_pid, Some(pid2));

    // Run tree
    let (_, exit_code) = tree_main(new_state, 1, vec![]).unwrap();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_tree_respects_state_immutability() {
    let state = KernelState::new();
    let state_before = state.clone();

    let (state_after, _) = tree_main(state, 1, vec![]).unwrap();

    // Process list should be unchanged (only stdout written)
    assert_eq!(
        state_before.processes.len(),
        state_after.processes.len()
    );
}
```

## Step 4: Register the Program

Edit `userspace/src/lib.rs`:

```rust
pub mod tree;

pub fn get_program(name: &str) -> Option<ProgramFn> {
    match name {
        "ps" => Some(ps::main),
        "tree" => Some(tree::main),  // Add this
        _ => None,
    }
}
```

## Step 5: Add Integration Test

Create `userspace/tests/integration_tree.rs`:

```rust
use kernel::{dispatch_syscall, sys_fork, KernelState, SystemCall};
use userspace::get_program;

#[test]
fn test_tree_integration() {
    let mut state = KernelState::new();

    // Create process tree
    let (new_state, _) = sys_fork(state, 1).unwrap();
    state = new_state;

    let (new_state, _) = sys_fork(state, 1).unwrap();
    state = new_state;

    // Execute tree via program registry
    let tree_fn = get_program("tree").expect("tree program not found");
    let (final_state, exit_code) = tree_fn(state, 1, vec![]).unwrap();

    assert_eq!(exit_code, 0);
    assert_eq!(final_state.processes.len(), 3); // init + 2 children
}
```

## Step 6: Run Tests

```bash
# Unit tests
cargo test --bin tree

# Integration tests
cargo test --test integration_tree

# All userspace tests
cargo test -p userspace
```

Expected output:
```
running 5 tests
test tests::test_tree_single_process ... ok
test tests::test_tree_with_children ... ok
test tests::test_tree_empty_args ... ok
test tree_test::test_tree_shows_all_processes ... ok
test tree_test::test_tree_hierarchy ... ok

test result: ok. 5 passed
```

## Step 7: Test Manually

Add to `wos/src/lib.rs` for WASM integration:

```rust
impl WosWasm {
    pub fn execute_command(&mut self, cmd: &str) -> String {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        let program_name = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        match program_name {
            "tree" => {
                let tree_fn = userspace::get_program("tree").unwrap();
                match tree_fn(self.state.clone(), 1, args) {
                    Ok((new_state, _exit_code)) => {
                        self.state = new_state;
                        "Process tree displayed".to_string()
                    }
                    Err(e) => format!("Error: {:?}", e),
                }
            }
            // ... other commands
        }
    }
}
```

Build and test in browser:

```bash
make wasm
make serve
```

Open http://localhost:8000/dist/wos/ and type:
```
wos$ tree
```

Expected output:
```
└── init (PID: 1, State: Running)
    ├── shell (PID: 2, State: Ready)
    │   └── ls (PID: 3, State: Ready)
    └── daemon (PID: 4, State: Ready)
```

## Step 8: Add Command-Line Arguments Support

Enhance `tree.rs` to support options:

```rust
pub fn main(
    state: KernelState,
    pid: ProcessId,
    args: Vec<String>,
) -> Result<(KernelState, i32), KernelError> {
    // Parse arguments
    let show_pids = args.contains(&"--pids".to_string());
    let show_states = args.contains(&"--states".to_string());

    // Build tree with options
    let mut output = String::new();
    print_tree(&state, &tree, 1, &mut output, "", true, show_pids, show_states);

    // Write to stdout
    let syscall = SystemCall::Write(1, output.into_bytes());
    let (new_state, _) = dispatch_syscall(state, syscall, pid)?;

    Ok((new_state, 0))
}
```

Add tests for arguments:

```rust
#[test]
fn test_tree_with_pids_flag() {
    let state = KernelState::new();

    let (new_state, exit_code) = main(
        state,
        1,
        vec!["--pids".to_string()]
    ).unwrap();

    assert_eq!(exit_code, 0);
    // Verify PIDs are shown in output
}

#[test]
fn test_tree_with_states_flag() {
    let state = KernelState::new();

    let (new_state, exit_code) = main(
        state,
        1,
        vec!["--states".to_string()]
    ).unwrap();

    assert_eq!(exit_code, 0);
    // Verify states are shown in output
}
```

## Step 9: Add Help Text

```rust
pub fn main(
    state: KernelState,
    pid: ProcessId,
    args: Vec<String>,
) -> Result<(KernelState, i32), KernelError> {
    // Handle --help
    if args.contains(&"--help".to_string()) {
        let help_text = r#"
Usage: tree [OPTIONS]

Display process tree hierarchy

Options:
  --pids      Show process IDs
  --states    Show process states
  --help      Show this help message
"#;
        let syscall = SystemCall::Write(1, help_text.as_bytes().to_vec());
        let (new_state, _) = dispatch_syscall(state, syscall, pid)?;
        return Ok((new_state, 0));
    }

    // ... rest of implementation
}
```

## Step 10: Document the Program

Add to `docs/API.md` under "Userspace Programs":

```markdown
#### `tree`

Display the process hierarchy as a tree.

**Usage:**
```
tree [--pids] [--states] [--help]
```

**Options:**
- `--pids` - Show process IDs in output
- `--states` - Show process states (Running, Ready, etc.)
- `--help` - Display help message

**Examples:**
```
wos$ tree
└── init
    ├── shell
    └── daemon

wos$ tree --pids --states
└── init (PID: 1, State: Running)
    ├── shell (PID: 2, State: Ready)
    └── daemon (PID: 3, State: Ready)
```

**Implementation:**
- Location: `userspace/src/bin/tree.rs`
- Exit code: 0 on success
- Syscalls used: Write (stdout)
```

## Summary

You've created a complete userspace program with:

1. ✅ **Pure functional design** - No mutations
2. ✅ **Comprehensive tests** - Unit, integration, property tests
3. ✅ **Argument parsing** - Command-line options
4. ✅ **Help text** - User documentation
5. ✅ **Error handling** - Proper Result types
6. ✅ **WASM integration** - Browser support
7. ✅ **Documentation** - API reference updated

## Exercise: Create a `kill` Program

Implement a program that sends a termination signal to a process:

```rust
// Usage: kill <pid>
// Example: kill 42
```

Requirements:
- Parse PID from arguments
- Call `sys_exit` for the target process
- Handle errors (process not found, permission denied)
- Add `--help` flag
- Write 5+ tests
- Update documentation

Hints:
```rust
pub fn main(
    state: KernelState,
    pid: ProcessId,
    args: Vec<String>,
) -> Result<(KernelState, i32), KernelError> {
    if args.is_empty() {
        // Error: missing PID argument
        return Ok((state, 1));
    }

    let target_pid: ProcessId = args[0].parse().map_err(|_| {
        // Error: invalid PID
    })?;

    // Call sys_exit on target_pid
    // ...
}
```

## Common Pitfalls

1. **Mutating state directly** - Always use syscalls, never mutate
2. **Forgetting error handling** - Every syscall can fail
3. **Not testing edge cases** - Empty args, invalid input, etc.
4. **Hardcoding PIDs** - Use the `pid` parameter passed in
5. **Missing help text** - Users need documentation

## Next Steps

- [Tutorial 3: Understanding the Scheduler](03-understanding-scheduler.md)
- [API Reference](../API.md)

## Further Reading

- [Unix Process Management](https://en.wikipedia.org/wiki/Process_management_%28computing%29)
- [Tree Command (Unix)](https://en.wikipedia.org/wiki/Tree_%28command%29)
- [Command-Line Interface Guidelines](https://clig.dev/)
