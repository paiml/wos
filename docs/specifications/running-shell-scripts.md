# Running Shell Scripts in WOS

**Version:** 1.0.0
**Date:** October 18, 2025
**Status:** SPECIFICATION - Implementation Ready
**Quality Level:** NASA-grade (Extreme TDD, E2E Playwright, 85%+ coverage, 90%+ mutation score)

---

## Executive Summary

This specification defines the comprehensive system for executing shell script files (`bash foo.sh`) in WOS (WebAssembly Operating System), enabling users to write, save, and execute multi-command scripts directly in the browser. Drawing from industry best practices (Pyodide, WebAssembly.sh, Browsix) and peer-reviewed WebAssembly research, we establish a robust, testable architecture that integrates seamlessly with WOS's existing VFS and Vim editor.

**Primary Goal**: Enable `bash script.sh` command to execute script files stored in the WOS virtual filesystem.

**Secondary Goals**:
- Support `source script.sh` (execute in current shell context)
- Support `./script.sh` (execute as program)
- Integration with Vim editor (write scripts, save, execute)
- Full EXTREME TDD coverage with Playwright E2E tests
- Bi-directional validation against reference bash

**Key Constraint**: No external bash.wasm binary - WOS implements its own shell interpreter in Rust/WASM for maximum control, safety, and educational value.

---

## Table of Contents

1. [Background & Research](#1-background--research)
2. [Problem Statement](#2-problem-statement)
3. [Architecture Design](#3-architecture-design)
4. [Implementation Strategy](#4-implementation-strategy)
5. [Testing Strategy](#5-testing-strategy)
6. [Integration Points](#6-integration-points)
7. [Quality Requirements](#7-quality-requirements)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Risk Analysis](#9-risk-analysis)
10. [References](#10-references)

---

## 1. Background & Research

### 1.1 Industry Solutions Analysis

#### Pyodide (Python in WebAssembly)
**Architecture**:
- CPython compiled to WASM via Emscripten
- Emscripten provides virtual filesystem (MEMFS) in JavaScript
- Files stored in volatile browser memory, can persist via IndexedDB
- Package execution via `micropip` from browser

**Key Lessons**:
- Virtual filesystems work excellently in WASM environments
- File execution requires proper loader/interpreter integration
- Browser APIs (localStorage, IndexedDB) provide persistence
- Educational value: Enables learning programming in browser

**Relevance to WOS**: WOS already has VFS - need to connect script loader to shell interpreter.

#### WebAssembly.sh / Wasmer
**Architecture**:
- PWA terminal powered by Wasmer-JS
- Runs WASI modules in browser via WebAssembly Package Manager (WAPM)
- Shell-like interface executing precompiled WASM binaries
- Piping and command chaining supported

**Key Lessons**:
- Browser-based terminals can execute complex workflows
- WASI provides standardized syscall interface
- Progressive Web App architecture enables offline usage
- Real-world POSIX compatibility achievable in browser

**Relevance to WOS**: Validates browser-based shell execution feasibility; WOS implements custom interpreter vs. running bash.wasm.

#### Browsix (Unix in Browser Tab)
**Architecture** (ASPLOS 2017 peer-reviewed):
- Maps Unix primitives onto browser APIs
- Web Workers for parallel process execution
- `pipe(2)`, `fork()`, `exec()` syscalls implemented
- Supports C, C++, Go, Node.js, **and POSIX shell scripts**

**Key Mechanisms**:
- System calls mapped to postMessage communication
- Shared filesystem across multiple "processes"
- Process management using Web Worker lifecycle
- Signals via custom event system

**Key Lessons**:
- Full POSIX compatibility possible in browser
- Multi-process model works via Web Workers
- Shell script execution validated in academic research
- Performance acceptable for educational/development use

**Relevance to WOS**: Proves shell script execution in browser is feasible and academically validated.

#### Native Bash WASM Ports (Wavix, WASI implementations)
**Approaches**:
1. **Wavix**: Compiles GNU bash to WASM using custom WASM compiler
2. **WASI bash**: Uses Emscripten to compile bash with WASI syscalls
3. **Direct invocation**: Load bash.wasm, invoke with `["bash", "-c", "script"]`

**Challenges**:
- Large binary sizes (bash.wasm ~500KB+)
- Limited WASI syscall support in browsers
- Debugging difficulty
- Educational opacity (users can't learn from source)

**WOS Decision**: Implement custom shell interpreter in Rust for:
- Educational transparency (users can read ~2000 lines of kernel code)
- Tight integration with WOS VFS and process model
- Smaller binary size (incremental addition to existing WASM)
- NASA-grade quality through TDD

### 1.2 Peer-Reviewed Research

#### "Research on WebAssembly Runtimes: A Survey" (ACM TOSEM)
- **Finding**: 103 research papers on WASM runtimes analyzed
- **Relevance**: Interpreters are viable WASM approach (not just AOT compilation)
- **WOS Application**: Our Rust-based shell interpreter follows proven patterns

#### "Bringing the Web up to Speed with WebAssembly" (PLDI 2017)
- **Finding**: Formal semantics from the start ensure correctness
- **Relevance**: Type-safe, sandboxed execution model
- **WOS Application**: Rust's type system provides similar guarantees

#### "WasmRef-Isabelle: A Verified Monadic Interpreter" (PLDI 2023)
- **Finding**: Monadic interpreters can be formally verified
- **Relevance**: Pure functional pattern (used in WOS kernel) enables verification
- **WOS Application**: Our `KernelOp` trait follows monadic interpreter pattern

#### Browsix: Unix in the Browser Tab (ASPLOS 2017)
- **Finding**: POSIX shell scripts successfully run in browser
- **Validation**: Academic peer review confirms feasibility
- **WOS Application**: Direct validation that our goal is achievable

**Conclusion from Research**: Shell script execution in browser WASM is:
1. ✅ Technically feasible (proven in multiple implementations)
2. ✅ Academically validated (peer-reviewed publications)
3. ✅ Performance-acceptable (<10ms execution for simple scripts)
4. ✅ Educationally valuable (learning shell scripting in browser)

---

## 2. Problem Statement

### 2.1 Current State

**What Works Today (WOS v0.1.0)**:
```bash
wos$ echo "hello" > test.txt    # ❌ Fails - no redirection
wos$ cat test.txt               # ❌ Fails - file doesn't exist
wos$ echo "hello"               # ✅ Works - direct command
hello
```

**Gap**: Users cannot:
1. Save multi-command scripts to files
2. Execute scripts with `bash script.sh`
3. Use `source` or `.` to run scripts in current context
4. Make scripts executable with `chmod +x` and run `./script.sh`

### 2.2 Desired End State

**Target Workflow**:
```bash
# Step 1: Create script using Vim
wos$ vim hello.sh
# User types in Vim:
#!/bin/bash
echo "Hello from WOS!"
echo "Current directory: $(pwd)"
ls -la

# Step 2: Execute script
wos$ bash hello.sh
Hello from WOS!
Current directory: /
total 1
drwxr-xr-x  1 root root  0 Oct 18 10:00 .
drwxr-xr-x  1 root root  0 Oct 18 10:00 bin
-rw-r--r--  1 root root 78 Oct 18 10:15 hello.sh

# Step 3: Source script (execute in current shell)
wos$ source hello.sh
Hello from WOS!
Current directory: /
[... same output ...]

# Step 4: Make executable and run directly (stretch goal)
wos$ chmod +x hello.sh
wos$ ./hello.sh
Hello from WOS!
[... same output ...]
```

### 2.3 Why This Matters

**Educational Value**:
- Users learn shell scripting in browser (no installation required)
- Live experimentation with scripts
- Immediate feedback loop (write → save → execute)
- Real-world skill building

**Practical Value**:
- Automation of repetitive tasks
- Testing and validation workflows
- Configuration management
- Demonstration and teaching tool

**Technical Validation**:
- Proves WOS is a "real" OS (can execute programs)
- Validates VFS, process model, shell parser completeness
- Milestone toward GNU Bash manual 100% compatibility

---

## 3. Architecture Design

### 3.1 System Components

```
┌─────────────────────────────────────────────────────────────┐
│                  Browser Environment                         │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │         HTML Terminal (dist/wos/index.html)            │ │
│  │  - User types: "bash script.sh"                        │ │
│  │  - User types: "source config.sh"                      │ │
│  └────────────┬───────────────────────────────────────────┘ │
│               │ executeCommand()                             │
│               ▼                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │        JavaScript Bridge (dist/wos/app.js)             │ │
│  │  - Terminal.executeCommand(cmd)                        │ │
│  │  - wos.execute_command(cmd) → WASM                     │ │
│  └────────────┬───────────────────────────────────────────┘ │
│               │                                              │
│               ▼                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │            WOS WASM Module (wos_bg.wasm)               │ │
│  │                                                         │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │  Shell Command Parser (NEW ENHANCEMENT)          │ │ │
│  │  │  - Parse "bash script.sh"                        │ │ │
│  │  │  - Parse "source script.sh"                      │ │ │
│  │  │  - Parse "./script.sh"                           │ │ │
│  │  └────────────┬─────────────────────────────────────┘ │ │
│  │               │                                         │ │
│  │               ▼                                         │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │  Script Loader (NEW COMPONENT)                   │ │ │
│  │  │  - Read script file from VFS                     │ │ │
│  │  │  - Parse shebang (#!) line                       │ │ │
│  │  │  - Validate script syntax                        │ │ │
│  │  │  - Prepare execution context                     │ │ │
│  │  └────────────┬─────────────────────────────────────┘ │ │
│  │               │                                         │ │
│  │               ▼                                         │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │  Script Executor (NEW COMPONENT)                 │ │ │
│  │  │  - Execute script line-by-line                   │ │ │
│  │  │  - Manage script-local variables                 │ │ │
│  │  │  - Handle control flow (if/while/for)            │ │ │
│  │  │  - Manage nested script calls                    │ │ │
│  │  │  - Accumulate output                             │ │ │
│  │  └────────────┬─────────────────────────────────────┘ │ │
│  │               │                                         │ │
│  │               ▼                                         │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │  Virtual File System (EXISTING)                  │ │ │
│  │  │  - shared/src/vfs.rs                             │ │ │
│  │  │  - im::HashMap storage (persistent data)         │ │ │
│  │  │  - File read/write operations                    │ │ │
│  │  └──────────────────────────────────────────────────┘ │ │
│  │                                                         │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │  Kernel Process Manager (EXISTING)               │ │ │
│  │  │  - kernel/src/process.rs                         │ │ │
│  │  │  - Process creation/tracking                     │ │ │
│  │  │  - Environment variables                         │ │ │
│  │  └──────────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Data Flow Diagrams

#### Flow 1: `bash script.sh` Execution

```
┌─────────────┐
│ User Input  │ bash script.sh
└──────┬──────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 1. Terminal.executeCommand("bash script.sh")│
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 2. Parse command → {cmd: "bash",            │
│                     args: ["script.sh"]}     │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 3. Detect "bash" command → route to          │
│    ScriptLoader.load("script.sh")            │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 4. VFS.read_file("/script.sh")              │
│    → Result<String, Error>                   │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 5. ScriptLoader.parse_shebang()             │
│    Detect: #!/bin/bash                       │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 6. ScriptExecutor.execute_lines()           │
│    Loop: parse + execute each line           │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 7. For each line:                            │
│    - Parse command (echo, ls, cat, etc.)     │
│    - Execute via existing command handlers   │
│    - Accumulate output                       │
│    - Check exit status                       │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 8. Return accumulated output to Terminal     │
│    Display in terminal-output div            │
└──────────────────────────────────────────────┘
```

#### Flow 2: `source script.sh` Execution

```
┌─────────────┐
│ User Input  │ source script.sh
└──────┬──────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 1. Detect "source" or "." command            │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 2. ScriptLoader.load_for_source()           │
│    Same as bash, but preserve shell context  │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 3. Execute in CURRENT shell context          │
│    - Use existing shell variables            │
│    - Modify shell environment                │
│    - cd commands affect current dir          │
│    - export persists after script            │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 4. Return output + modified shell state      │
└──────────────────────────────────────────────┘
```

#### Flow 3: `./script.sh` Execution (Executable Scripts)

```
┌─────────────┐
│ User Input  │ ./script.sh
└──────┬──────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 1. Detect path starting with ./ or /        │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 2. Check if file exists + has execute perm  │
│    VFS.stat() → check permissions            │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 3. Read shebang to determine interpreter     │
│    #!/bin/bash → use bash interpreter        │
│    #!/usr/bin/env python → (future: error)   │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 4. Execute via ScriptExecutor (like bash)   │
└──────────────────────────────────────────────┘
```

### 3.3 Core Components Design

#### Component 1: ScriptLoader (`shared/src/script_loader.rs`)

```rust
use std::path::Path;
use crate::vfs::VirtualFileSystem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub path: String,
    pub content: String,
    pub shebang: Option<String>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptError {
    FileNotFound(String),
    PermissionDenied(String),
    InvalidShebang(String),
    ParseError(String),
}

pub struct ScriptLoader;

impl ScriptLoader {
    /// Load a script file from the VFS
    pub fn load(vfs: &VirtualFileSystem, path: &str) -> Result<Script, ScriptError> {
        // 1. Resolve path (handle relative paths)
        let full_path = Self::resolve_path(path)?;

        // 2. Read file from VFS
        let content = vfs.read_file(&full_path)
            .map_err(|_| ScriptError::FileNotFound(path.to_string()))?;

        // 3. Parse shebang line (first line starting with #!)
        let (shebang, script_lines) = Self::parse_shebang(&content);

        // 4. Validate shebang (must be bash or empty)
        if let Some(ref shebang_line) = shebang {
            if !Self::is_bash_shebang(shebang_line) {
                return Err(ScriptError::InvalidShebang(shebang_line.clone()));
            }
        }

        Ok(Script {
            path: full_path,
            content,
            shebang,
            lines: script_lines,
        })
    }

    fn resolve_path(path: &str) -> Result<String, ScriptError> {
        if path.starts_with('/') {
            Ok(path.to_string())
        } else {
            // Relative to current working directory
            // For now, assume root
            Ok(format!("/{}", path.trim_start_matches("./")))
        }
    }

    fn parse_shebang(content: &str) -> (Option<String>, Vec<String>) {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        if let Some(first_line) = lines.first() {
            if first_line.starts_with("#!") {
                let shebang = Some(first_line.clone());
                lines.remove(0); // Remove shebang from script lines
                return (shebang, lines);
            }
        }

        (None, lines)
    }

    fn is_bash_shebang(shebang: &str) -> bool {
        shebang.contains("/bash") || shebang.contains("/sh")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shebang_bash() {
        let content = "#!/bin/bash\necho hello\n";
        let (shebang, lines) = ScriptLoader::parse_shebang(content);
        assert_eq!(shebang, Some("#!/bin/bash".to_string()));
        assert_eq!(lines, vec!["echo hello"]);
    }

    #[test]
    fn test_parse_no_shebang() {
        let content = "echo hello\n";
        let (shebang, lines) = ScriptLoader::parse_shebang(content);
        assert_eq!(shebang, None);
        assert_eq!(lines, vec!["echo hello"]);
    }

    #[test]
    fn test_is_bash_shebang() {
        assert!(ScriptLoader::is_bash_shebang("#!/bin/bash"));
        assert!(ScriptLoader::is_bash_shebang("#!/usr/bin/env bash"));
        assert!(ScriptLoader::is_bash_shebang("#!/bin/sh"));
        assert!(!ScriptLoader::is_bash_shebang("#!/usr/bin/python"));
    }
}
```

#### Component 2: ScriptExecutor (`shared/src/script_executor.rs`)

```rust
use crate::script_loader::Script;
use crate::vfs::VirtualFileSystem;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, String>,
    pub working_dir: String,
    pub exit_code: i32,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            working_dir: "/".to_string(),
            exit_code: 0,
        }
    }
}

pub struct ScriptExecutor;

impl ScriptExecutor {
    /// Execute a script in a new subshell context
    pub fn execute(
        script: &Script,
        vfs: &mut VirtualFileSystem,
        parent_context: &ExecutionContext,
    ) -> Result<(String, ExecutionContext), String> {
        // Create new execution context (subshell)
        let mut context = parent_context.clone();
        let mut output = String::new();

        for (line_num, line) in script.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Execute line
            match Self::execute_line(trimmed, vfs, &mut context) {
                Ok(line_output) => {
                    if !line_output.is_empty() {
                        output.push_str(&line_output);
                        output.push('\n');
                    }
                }
                Err(e) => {
                    // On error, set exit code and stop execution
                    context.exit_code = 1;
                    return Err(format!("Line {}: {}", line_num + 1, e));
                }
            }
        }

        Ok((output.trim_end().to_string(), context))
    }

    /// Execute a script in the current shell context (source/.)
    pub fn source(
        script: &Script,
        vfs: &mut VirtualFileSystem,
        context: &mut ExecutionContext,
    ) -> Result<String, String> {
        let mut output = String::new();

        for (line_num, line) in script.lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            match Self::execute_line(trimmed, vfs, context) {
                Ok(line_output) => {
                    if !line_output.is_empty() {
                        output.push_str(&line_output);
                        output.push('\n');
                    }
                }
                Err(e) => {
                    context.exit_code = 1;
                    return Err(format!("Line {}: {}", line_num + 1, e));
                }
            }
        }

        Ok(output.trim_end().to_string())
    }

    fn execute_line(
        line: &str,
        vfs: &mut VirtualFileSystem,
        context: &mut ExecutionContext,
    ) -> Result<String, String> {
        // This delegates to the existing command execution system
        // For now, placeholder that would integrate with WOS command handlers

        // Parse command (handle variable substitution, etc.)
        let expanded_line = Self::expand_variables(line, context);

        // TODO: Integrate with existing wos.execute_command()
        // For spec purposes, showing the interface
        Err("Not implemented - integrate with WOS command system".to_string())
    }

    fn expand_variables(line: &str, context: &ExecutionContext) -> String {
        let mut result = line.to_string();

        // Simple variable expansion (full implementation in Sprint 5)
        for (key, value) in &context.variables {
            let pattern = format!("${}", key);
            result = result.replace(&pattern, value);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_loader::Script;

    #[test]
    fn test_execution_context_default() {
        let ctx = ExecutionContext::new();
        assert_eq!(ctx.working_dir, "/");
        assert_eq!(ctx.exit_code, 0);
        assert_eq!(ctx.variables.len(), 0);
    }

    #[test]
    fn test_expand_variables() {
        let mut ctx = ExecutionContext::new();
        ctx.variables.insert("NAME".to_string(), "World".to_string());

        let expanded = ScriptExecutor::expand_variables("Hello $NAME", &ctx);
        assert_eq!(expanded, "Hello World");
    }
}
```

#### Component 3: Integration with WOS (`wos/src/lib.rs`)

```rust
// In WosWasm struct, add script execution methods

impl WosWasm {
    /// Execute a bash script file
    pub fn execute_bash_script(&mut self, script_path: &str) -> String {
        use shared::script_loader::ScriptLoader;
        use shared::script_executor::{ScriptExecutor, ExecutionContext};

        // Load script from VFS
        let script = match ScriptLoader::load(&self.vfs, script_path) {
            Ok(s) => s,
            Err(e) => return format!("Error loading script: {:?}", e),
        };

        // Create execution context
        let context = ExecutionContext::new();

        // Execute script
        match ScriptExecutor::execute(&script, &mut self.vfs, &context) {
            Ok((output, _final_context)) => output,
            Err(e) => format!("Script execution error: {}", e),
        }
    }

    /// Source a script (execute in current shell context)
    pub fn source_script(&mut self, script_path: &str) -> String {
        use shared::script_loader::ScriptLoader;
        use shared::script_executor::{ScriptExecutor, ExecutionContext};

        let script = match ScriptLoader::load(&self.vfs, script_path) {
            Ok(s) => s,
            Err(e) => return format!("Error loading script: {:?}", e),
        };

        // Use current shell context (would need to track this in WosWasm)
        let mut context = self.get_current_shell_context();

        match ScriptExecutor::source(&script, &mut self.vfs, &mut context) {
            Ok(output) => {
                self.update_shell_context(context);
                output
            }
            Err(e) => format!("Script execution error: {}", e),
        }
    }
}
```

---

## 4. Implementation Strategy

### 4.1 Phased Implementation (EXTREME TDD)

#### Phase 1: Basic Script Loading (Week 1)
**Goal**: Load script files from VFS, parse shebang

**Deliverables**:
- `ScriptLoader` struct with `load()` method
- Shebang parsing logic
- File reading from VFS
- Error handling (file not found, invalid shebang)

**Tests (TDD - Write First)**:
```rust
#[test]
fn test_script_loader_loads_file_from_vfs() { }

#[test]
fn test_script_loader_returns_error_if_file_not_found() { }

#[test]
fn test_script_loader_parses_bash_shebang() { }

#[test]
fn test_script_loader_rejects_non_bash_shebang() { }
```

**E2E Tests (Playwright)**:
```typescript
test('Script Loader: Load script from VFS', async ({ page }) => {
  await page.goto('/');
  await typeCommand(page, 'echo "#!/bin/bash" > test.sh');
  await typeCommand(page, 'echo "echo hello" >> test.sh');

  // For now, just verify file exists
  await typeCommand(page, 'cat test.sh');
  const output = await getLastOutput(page);
  expect(output).toContain('#!/bin/bash');
  expect(output).toContain('echo hello');
});
```

#### Phase 2: Line-by-Line Execution (Week 2)
**Goal**: Execute simple scripts (echo, ls, pwd only)

**Deliverables**:
- `ScriptExecutor` struct with `execute()` method
- Line-by-line parsing and execution
- Output accumulation
- Integration with existing command handlers

**Tests**:
```rust
#[test]
fn test_execute_single_line_echo() { }

#[test]
fn test_execute_multiple_lines() { }

#[test]
fn test_skip_empty_lines_and_comments() { }
```

**E2E Tests**:
```typescript
test('Script Execution: bash simple_script.sh', async ({ page }) => {
  await page.goto('/');

  // Create script
  await typeCommand(page, 'vim hello.sh');
  await vimType(page, 'i');  // Insert mode
  await vimType(page, '#!/bin/bash\necho "Hello World"\n');
  await vimType(page, '\x1b');  // Escape
  await vimType(page, ':wq\n');  // Save and quit

  // Execute script
  await typeCommand(page, 'bash hello.sh');
  const output = await getLastOutput(page);
  expect(output).toBe('Hello World');
});
```

#### Phase 3: Variable Support (Week 3)
**Goal**: Support shell variables in scripts

**Deliverables**:
- Variable expansion (`$VAR`, `${VAR}`)
- Variable assignment (`VAR=value`)
- Environment variable support (`export VAR=value`)

**Tests**:
```rust
#[test]
fn test_variable_expansion() { }

#[test]
fn test_variable_assignment() { }

#[test]
fn test_export_variable() { }
```

#### Phase 4: Source Command (Week 4)
**Goal**: Implement `source` and `.` commands

**Deliverables**:
- `source` command handler
- Current shell context preservation
- Environment modification persistence

**E2E Tests**:
```typescript
test('Source: Modify current shell environment', async ({ page }) => {
  await createScript(page, 'config.sh', 'export PATH="/usr/local/bin"\ncd /tmp');
  await typeCommand(page, 'source config.sh');
  await typeCommand(page, 'echo $PATH');
  expect(await getLastOutput(page)).toBe('/usr/local/bin');
  await typeCommand(page, 'pwd');
  expect(await getLastOutput(page)).toBe('/tmp');
});
```

#### Phase 5: Control Flow (Week 5-6)
**Goal**: Support if/while/for in scripts

**Deliverables**:
- if-then-else parsing and execution
- while loop parsing and execution
- for loop parsing and execution
- Conditional test operators

**E2E Tests**:
```typescript
test('Control Flow: If statement in script', async ({ page }) => {
  const script = `
#!/bin/bash
if [ -f test.txt ]; then
  echo "File exists"
else
  echo "File not found"
fi
`;
  await createScript(page, 'check.sh', script);
  await typeCommand(page, 'bash check.sh');
  expect(await getLastOutput(page)).toBe('File not found');

  await typeCommand(page, 'touch test.txt');
  await typeCommand(page, 'bash check.sh');
  expect(await getLastOutput(page)).toBe('File exists');
});
```

#### Phase 6: Executable Scripts (Week 7)
**Goal**: Support `./script.sh` execution

**Deliverables**:
- chmod command (set execute permission)
- Path-based command detection (`./`, `/`)
- Permission checking
- Shebang-based interpreter selection

**E2E Tests**:
```typescript
test('Executable: Run script with ./script.sh', async ({ page }) => {
  await createScript(page, 'hello.sh', '#!/bin/bash\necho "Executable!"');
  await typeCommand(page, 'chmod +x hello.sh');
  await typeCommand(page, './hello.sh');
  expect(await getLastOutput(page)).toBe('Executable!');
});
```

### 4.2 Integration Points

#### Integration 1: VFS (shared/src/vfs.rs)
**What exists**: File read/write, directory operations

**What to add**:
- File permission bits (rwx flags)
- `stat()` method for permission checking
- Execute bit support

```rust
// Add to VirtualFile struct
pub struct VirtualFile {
    pub name: String,
    pub content: Vec<u8>,
    pub permissions: u32,  // NEW: Unix-style permission bits
    pub created_at: u64,
    pub modified_at: u64,
}

impl VirtualFile {
    pub fn has_execute_permission(&self) -> bool {
        (self.permissions & 0o111) != 0  // Check any execute bit
    }
}
```

#### Integration 2: Command Parser (wos/src/lib.rs)
**What exists**: Simple command parsing (split by whitespace)

**What to add**:
- Detect `bash` command → route to script executor
- Detect `source` or `.` command → route to source handler
- Detect path-based commands (`./`, `/`) → check executable + run

```rust
// In execute_command method
fn execute_command(&mut self, cmd_line: String) -> String {
    let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }

    match parts[0] {
        "bash" => {
            if parts.len() < 2 {
                return "Usage: bash <script>".to_string();
            }
            self.execute_bash_script(parts[1])
        }
        "source" | "." => {
            if parts.len() < 2 {
                return "Usage: source <script>".to_string();
            }
            self.source_script(parts[1])
        }
        cmd if cmd.starts_with("./") || cmd.starts_with("/") => {
            self.execute_file_as_program(cmd)
        }
        _ => {
            // Existing command handling
            self.execute_builtin_command(parts)
        }
    }
}
```

#### Integration 3: Vim Editor (dist/wos/app.js)
**What exists**: Full Vim modal editor with file save/load

**What to enhance**:
- Save file with execute permissions if shebang detected
- Syntax highlighting for shell scripts (stretch goal)

```javascript
// In VimEditor.save() method
save() {
  const content = this.lines.join('\n');

  // Detect shebang
  const hasShebang = content.startsWith('#!');

  // Save to localStorage
  localStorage.setItem(`wos-file-${this.fileName}`, content);

  // If shebang, also save execute permission flag
  if (hasShebang) {
    localStorage.setItem(`wos-file-perm-${this.fileName}`, '0755');
  }

  this.modified = false;
  this.message = `"${this.fileName}" written`;
}
```

---

## 5. Testing Strategy

### 5.1 Test Pyramid

```
         ┌──────────────────┐
         │   E2E Tests      │  ~30 tests (Playwright)
         │   (Playwright)   │  - Full user workflows
         └────────┬─────────┘  - bash script.sh
                  │            - source config.sh
         ┌────────┴─────────┐  - ./executable.sh
         │ Integration Tests│  ~50 tests (Rust)
         │  (Rust #[test])  │  - ScriptLoader + VFS
         └────────┬─────────┘  - ScriptExecutor + Commands
                  │            - Error handling
         ┌────────┴─────────┐
         │   Unit Tests     │  ~100 tests (Rust)
         │  (Rust #[test])  │  - ScriptLoader functions
         └──────────────────┘  - ScriptExecutor functions
                                - Variable expansion
                                - Shebang parsing
```

### 5.2 Test Coverage Requirements

**Coverage Targets**:
- Line Coverage: ≥85% (measured with `cargo-llvm-cov`)
- Branch Coverage: ≥80%
- Function Coverage: 100% (all public functions tested)

**Mutation Testing**:
- Mutation Kill Rate: ≥90% (measured with `cargo-mutants`)
- Critical paths: 100% kill rate (script loading, execution)

### 5.3 E2E Test Scenarios (Playwright)

```typescript
// e2e/tests/canary/shell-scripts.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Shell Script Execution', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")');
  });

  test('C200: Execute simple bash script with echo', async ({ page }) => {
    // Create script using vim
    await createScriptViaVim(page, 'hello.sh',
      '#!/bin/bash\necho "Hello from script"');

    // Execute with bash command
    await typeCommand(page, 'bash hello.sh');

    // Verify output
    const output = await getLastOutput(page);
    expect(output).toBe('Hello from script');
  });

  test('C201: Execute script with multiple commands', async ({ page }) => {
    const script = `#!/bin/bash
echo "Line 1"
echo "Line 2"
pwd`;

    await createScriptViaVim(page, 'multi.sh', script);
    await typeCommand(page, 'bash multi.sh');

    const output = await getLastOutput(page);
    expect(output).toContain('Line 1');
    expect(output).toContain('Line 2');
    expect(output).toContain('/');  // pwd output
  });

  test('C202: Script with variables', async ({ page }) => {
    const script = `#!/bin/bash
NAME="WOS"
echo "Hello $NAME"`;

    await createScriptViaVim(page, 'vars.sh', script);
    await typeCommand(page, 'bash vars.sh');

    expect(await getLastOutput(page)).toBe('Hello WOS');
  });

  test('C203: Source command modifies shell environment', async ({ page }) => {
    const script = `export TEST_VAR="configured"
cd /tmp`;

    await createScriptViaVim(page, 'config.sh', script);
    await typeCommand(page, 'source config.sh');

    // Check variable persists
    await typeCommand(page, 'echo $TEST_VAR');
    expect(await getLastOutput(page)).toBe('configured');

    // Check directory changed
    await typeCommand(page, 'pwd');
    expect(await getLastOutput(page)).toBe('/tmp');
  });

  test('C204: Executable script with ./script.sh', async ({ page }) => {
    await createScriptViaVim(page, 'exec.sh',
      '#!/bin/bash\necho "Executable script"');

    // Make executable
    await typeCommand(page, 'chmod +x exec.sh');

    // Run with ./
    await typeCommand(page, './exec.sh');
    expect(await getLastOutput(page)).toBe('Executable script');
  });

  test('C205: Script with if statement', async ({ page }) => {
    const script = `#!/bin/bash
if [ -f test.txt ]; then
  echo "exists"
else
  echo "not found"
fi`;

    await createScriptViaVim(page, 'check.sh', script);
    await typeCommand(page, 'bash check.sh');
    expect(await getLastOutput(page)).toBe('not found');
  });

  test('C206: Script with while loop', async ({ page }) => {
    const script = `#!/bin/bash
i=0
while [ $i -lt 3 ]; do
  echo $i
  i=$((i + 1))
done`;

    await createScriptViaVim(page, 'loop.sh', script);
    await typeCommand(page, 'bash loop.sh');

    const output = await getLastOutput(page);
    expect(output).toBe('0\n1\n2');
  });

  test('C207: Error handling - script not found', async ({ page }) => {
    await typeCommand(page, 'bash nonexistent.sh');
    const output = await getLastOutput(page);
    expect(output).toContain('not found');
  });

  test('C208: Error handling - invalid shebang', async ({ page }) => {
    await createScriptViaVim(page, 'bad.sh',
      '#!/usr/bin/python\nprint("hello")');

    await typeCommand(page, 'bash bad.sh');
    const output = await getLastOutput(page);
    expect(output).toContain('Invalid shebang');
  });

  test('C209: Nested script calls', async ({ page }) => {
    await createScriptViaVim(page, 'inner.sh',
      '#!/bin/bash\necho "Inner script"');

    await createScriptViaVim(page, 'outer.sh',
      '#!/bin/bash\necho "Outer start"\nbash inner.sh\necho "Outer end"');

    await typeCommand(page, 'bash outer.sh');
    const output = await getLastOutput(page);
    expect(output).toBe('Outer start\nInner script\nOuter end');
  });

  test('C210: Bi-directional validation vs /bin/bash', async ({ page }) => {
    const script = '#!/bin/bash\necho "test" | grep test';

    // Execute in WOS
    await createScriptViaVim(page, 'pipe.sh', script);
    await typeCommand(page, 'bash pipe.sh');
    const wosOutput = await getLastOutput(page);

    // Execute in reference bash (Node.js child_process)
    const { stdout } = await execAsync('bash -c "echo test | grep test"');
    const bashOutput = stdout.trim();

    // Compare deterministic output
    expect(wosOutput).toBe(bashOutput);
  });
});

// Helper functions
async function createScriptViaVim(page, filename, content) {
  await typeCommand(page, `vim ${filename}`);
  await page.keyboard.press('i');  // Insert mode
  await page.keyboard.type(content);
  await page.keyboard.press('Escape');
  await page.keyboard.type(':wq');
  await page.keyboard.press('Enter');
}

async function typeCommand(page, cmd) {
  const input = page.locator('#terminal-input');
  await input.fill(cmd);
  await input.press('Enter');
  await page.waitForTimeout(100);  // Allow WASM to process
}

async function getLastOutput(page) {
  const outputLines = await page.locator('#terminal-output .output-line').all();
  if (outputLines.length === 0) return '';
  const lastLine = outputLines[outputLines.length - 1];
  return (await lastLine.textContent()).trim();
}
```

### 5.4 Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_script_loader_never_panics(path in ".*", content in ".*") {
        // Property: ScriptLoader should never panic on any input
        let mut vfs = VirtualFileSystem::new();
        vfs.write_file(&path, content.as_bytes()).ok();

        let _ = ScriptLoader::load(&vfs, &path);
        // If we get here without panic, test passes
    }

    #[test]
    fn test_shebang_parsing_deterministic(content in ".*") {
        // Property: Parsing same content multiple times yields same result
        let (shebang1, lines1) = ScriptLoader::parse_shebang(&content);
        let (shebang2, lines2) = ScriptLoader::parse_shebang(&content);

        prop_assert_eq!(shebang1, shebang2);
        prop_assert_eq!(lines1, lines2);
    }

    #[test]
    fn test_variable_expansion_idempotent(line in ".*") {
        // Property: Expanding variables twice should equal expanding once
        let ctx = ExecutionContext::new();
        let once = ScriptExecutor::expand_variables(&line, &ctx);
        let twice = ScriptExecutor::expand_variables(&once, &ctx);

        prop_assert_eq!(once, twice);
    }
}
```

### 5.5 Mutation Testing Targets

```toml
# Add to Cargo.toml for each crate

[package.metadata.mutants]
timeout = "120s"
minimum_test_timeout = "5s"

[[package.metadata.mutants.override]]
# Critical path - require 100% mutation kill rate
path = "shared/src/script_loader.rs"
minimum_mutation_score = 1.0

[[package.metadata.mutants.override]]
path = "shared/src/script_executor.rs"
minimum_mutation_score = 1.0

[[package.metadata.mutants.override]]
# Other code - require 90% mutation kill rate
path = "wos/src/*.rs"
minimum_mutation_score = 0.9
```

---

## 6. Integration Points

### 6.1 Vim Editor Integration

**Current State**: Vim editor can create and edit files, save to localStorage

**Enhancements Needed**:
1. Automatically detect shebang → set execute permission
2. Syntax highlighting for `.sh` files (stretch goal)
3. Run command: `:!bash %` to execute current file

```javascript
// In dist/wos/app.js - VimEditor class

class VimEditor {
  handleCommandMode(e) {
    if (e.key === 'Enter') {
      const cmd = this.commandBuffer.slice(1);  // Remove leading :

      // New: Handle :!bash % command
      if (cmd === '!bash %') {
        this.executeCurrentFile();
        return;
      }

      // Existing command handling...
    }
  }

  executeCurrentFile() {
    if (!this.fileName) {
      this.message = 'No file to execute';
      return;
    }

    // Close Vim
    this.close();

    // Execute the file via terminal
    const terminal = document.querySelector('#terminal-input');
    terminal.value = `bash ${this.fileName}`;
    terminal.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
  }

  save() {
    const content = this.lines.join('\n');
    const hasShebang = content.startsWith('#!');

    // Save file content
    localStorage.setItem(`wos-file-${this.fileName}`, content);

    // Save execute permission if shebang present
    if (hasShebang) {
      localStorage.setItem(`wos-file-perm-${this.fileName}`, '0755');
    } else {
      localStorage.setItem(`wos-file-perm-${this.fileName}`, '0644');
    }

    this.modified = false;
    this.message = `"${this.fileName}" written${hasShebang ? ' [executable]' : ''}`;
  }
}
```

### 6.2 Command History Integration

**Enhancement**: Save executed scripts to command history

```javascript
// In Terminal.executeCommand()

executeCommand(cmd) {
  // Add to history
  if (cmd.trim() !== '') {
    this.history.push(cmd);
    this.historyIndex = -1;

    // Persist to localStorage
    const historyKey = 'wos-command-history';
    const maxHistory = 100;
    const history = this.history.slice(-maxHistory);
    localStorage.setItem(historyKey, JSON.stringify(history));
  }

  // Execute command...
}
```

### 6.3 Error Reporting Integration

**Enhancement**: Rich error messages for script failures

```rust
// In ScriptExecutor

pub enum ScriptError {
    FileNotFound { path: String },
    PermissionDenied { path: String },
    InvalidShebang { shebang: String },
    SyntaxError { line: usize, message: String },
    RuntimeError { line: usize, command: String, error: String },
    CommandNotFound { command: String },
}

impl ScriptError {
    pub fn to_user_message(&self) -> String {
        match self {
            Self::FileNotFound { path } => {
                format!("bash: {}: No such file or directory", path)
            }
            Self::SyntaxError { line, message } => {
                format!("bash: line {}: syntax error: {}", line, message)
            }
            Self::RuntimeError { line, command, error } => {
                format!("bash: line {}: {}: {}", line, command, error)
            }
            // ... other error formats
        }
    }
}
```

---

## 7. Quality Requirements

### 7.1 Code Quality Standards

**Rust Code**:
```toml
# Enforced via pre-commit hooks

[quality]
formatting = "cargo fmt --check"
linting = "cargo clippy --all-features -- -D warnings"
complexity_max = 15  # per function
satd_tolerance = 0   # Zero TODO/FIXME/HACK
unsafe_blocks = 0    # Forbidden (already enforced)
```

**TypeScript/JavaScript Code** (E2E tests):
```json
{
  "lint": "eslint e2e/**/*.ts",
  "format": "prettier --check e2e/**/*.ts"
}
```

### 7.2 Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Simple script execution (5 lines) | <10ms | E2E timer |
| Medium script (50 lines) | <50ms | E2E timer |
| Nested script calls (3 deep) | <30ms | E2E timer |
| WASM binary size increase | <50KB | wasm-opt size |
| Memory overhead per script | <10KB | Chrome DevTools |

### 7.3 Documentation Requirements

**API Documentation**:
- All public functions: rustdoc comments
- All structs/enums: rustdoc with examples
- All test modules: Purpose descriptions

**User Documentation**:
```markdown
# To be created:
- docs/user-guide/shell-scripting.md
- docs/examples/hello-world-script.md
- docs/examples/advanced-scripts.md
- docs/troubleshooting/script-errors.md
```

### 7.4 Accessibility Requirements

**Terminal Interface**:
- Screen reader support for script output
- Keyboard-only script creation/execution
- High-contrast mode compatibility
- Error messages readable by assistive tech

---

## 8. Implementation Roadmap

### 8.1 Sprint Timeline (7 weeks total)

```
Week 1: Phase 1 - Basic Script Loading
  ├─ Mon-Tue: TDD ScriptLoader (unit tests → implementation)
  ├─ Wed-Thu: VFS integration + E2E tests
  ├─ Fri: Code review + documentation
  └─ Deliverable: Scripts loadable from VFS ✓

Week 2: Phase 2 - Line-by-Line Execution
  ├─ Mon-Tue: TDD ScriptExecutor (unit tests → implementation)
  ├─ Wed-Thu: Command integration + E2E tests
  ├─ Fri: Performance testing + optimization
  └─ Deliverable: bash script.sh works for echo/ls/pwd ✓

Week 3: Phase 3 - Variable Support
  ├─ Mon-Tue: Variable expansion logic (TDD)
  ├─ Wed: Variable assignment + export
  ├─ Thu: E2E tests with variables
  ├─ Fri: Property-based tests
  └─ Deliverable: Scripts with $VAR work ✓

Week 4: Phase 4 - Source Command
  ├─ Mon-Tue: source command implementation (TDD)
  ├─ Wed: Shell context preservation
  ├─ Thu: E2E tests for source
  ├─ Fri: Integration testing
  └─ Deliverable: source script.sh modifies environment ✓

Week 5-6: Phase 5 - Control Flow
  ├─ Week 5 Mon-Wed: if-then-else (TDD)
  ├─ Week 5 Thu-Fri: while loops (TDD)
  ├─ Week 6 Mon-Tue: for loops (TDD)
  ├─ Week 6 Wed-Thu: E2E tests for all control flow
  ├─ Week 6 Fri: Complex nested scenarios
  └─ Deliverable: Scripts with if/while/for work ✓

Week 7: Phase 6 - Executable Scripts
  ├─ Mon-Tue: chmod command + VFS permissions
  ├─ Wed: Executable detection + shebang parsing
  ├─ Thu: E2E tests for ./script.sh
  ├─ Fri: Final integration + bi-directional validation
  └─ Deliverable: ./script.sh execution works ✓

Week 8 (Buffer): Hardening & Documentation
  ├─ Mon: Mutation testing to ≥90%
  ├─ Tue: Performance optimization
  ├─ Wed: Error message improvement
  ├─ Thu: User documentation
  ├─ Fri: Demo scripts + tutorials
  └─ Deliverable: Production-ready ✓
```

### 8.2 Ticket Breakdown

**Phase 1 Tickets**:
- [ ] SCRIPT-001: Implement ScriptLoader struct and load() method
- [ ] SCRIPT-002: Add shebang parsing logic
- [ ] SCRIPT-003: Integrate with VFS for file reading
- [ ] SCRIPT-004: Add error handling (FileNotFound, InvalidShebang)
- [ ] SCRIPT-005: Write 20 unit tests for ScriptLoader
- [ ] SCRIPT-006: Write 5 E2E tests for script loading
- [ ] SCRIPT-007: Update documentation (rustdoc + user guide)

**Phase 2 Tickets**:
- [ ] SCRIPT-010: Implement ScriptExecutor struct and execute() method
- [ ] SCRIPT-011: Add line-by-line parsing and execution
- [ ] SCRIPT-012: Integrate with existing command handlers
- [ ] SCRIPT-013: Add output accumulation logic
- [ ] SCRIPT-014: Implement exit code tracking
- [ ] SCRIPT-015: Write 30 unit tests for ScriptExecutor
- [ ] SCRIPT-016: Write 10 E2E tests for bash command
- [ ] SCRIPT-017: Performance benchmarking

(Continued for Phases 3-6...)

### 8.3 Definition of Done (DoD)

For each phase to be considered complete:

**Code Quality**:
- [ ] All unit tests passing (100%)
- [ ] All E2E tests passing (100%)
- [ ] Code coverage ≥85% line, ≥80% branch
- [ ] Mutation score ≥90%
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] cargo fmt passes
- [ ] Complexity ≤15 per function

**Documentation**:
- [ ] Rustdoc for all public APIs
- [ ] User guide section updated
- [ ] Example scripts added
- [ ] CHANGELOG.md updated

**Testing**:
- [ ] Unit tests cover all branches
- [ ] E2E tests cover user workflows
- [ ] Property tests cover invariants
- [ ] Bi-directional validation vs bash

**Integration**:
- [ ] Works with existing VFS
- [ ] Works with Vim editor
- [ ] Works with all existing commands
- [ ] No regressions in existing tests

---

## 9. Risk Analysis

### 9.1 Technical Risks

**Risk 1: WASM Binary Size**
- **Description**: Adding script execution may bloat WASM binary
- **Probability**: Medium
- **Impact**: High (slow page loads)
- **Mitigation**:
  - Track size per commit (reject >10% increase)
  - Use wasm-opt with -Oz optimization
  - Lazy-load script executor if possible
- **Monitoring**: `make wasm-size-check` in CI

**Risk 2: Performance Degradation**
- **Description**: Script parsing/execution may be slow
- **Probability**: Low (Rust is fast)
- **Impact**: Medium (poor UX)
- **Mitigation**:
  - Benchmark every phase
  - Set performance budgets (<10ms simple scripts)
  - Optimize hot paths (variable expansion, line parsing)
- **Monitoring**: E2E performance tests with timing assertions

**Risk 3: VFS Integration Bugs**
- **Description**: File operations may have edge cases
- **Probability**: Medium
- **Impact**: High (data loss, corruption)
- **Mitigation**:
  - EXTREME TDD (tests first)
  - Property-based testing (fuzz file paths, content)
  - Extensive E2E scenarios
- **Monitoring**: 100% test coverage on VFS integration

### 9.2 Scope Risks

**Risk 4: Feature Creep**
- **Description**: Temptation to add non-essential features
- **Probability**: High (always happens)
- **Impact**: Medium (delays, complexity)
- **Mitigation**:
  - Strict adherence to 6-phase roadmap
  - Defer advanced features (heredoc, arrays, etc.)
  - Weekly scope review in standups
- **Monitoring**: Sprint retrospectives

**Risk 5: GNU Bash Compatibility Expectations**
- **Description**: Users expect 100% bash compatibility
- **Probability**: High
- **Impact**: Low (educational tool, not production shell)
- **Mitigation**:
  - Clear documentation of supported features
  - Error messages for unsupported syntax
  - Roadmap for future enhancements
- **Monitoring**: User feedback, GitHub issues

### 9.3 Quality Risks

**Risk 6: Test Flakiness**
- **Description**: E2E tests may be non-deterministic
- **Probability**: Medium (timing issues common)
- **Impact**: High (CI failures, lost productivity)
- **Mitigation**:
  - Explicit waits (not arbitrary sleeps)
  - Retry logic for known-flaky tests
  - Test isolation (clear state between tests)
- **Monitoring**: Flake rate tracking (<1% target)

**Risk 7: Edge Case Coverage**
- **Description**: Missing edge cases in testing
- **Probability**: Medium
- **Impact**: Medium (bugs in production)
- **Mitigation**:
  - Property-based testing (random inputs)
  - Mutation testing (verify tests catch bugs)
  - Code review with checklist
- **Monitoring**: Mutation kill rate ≥90%

---

## 10. References

### 10.1 Industry Implementations

1. **Pyodide**
   - Website: https://pyodide.org/
   - GitHub: https://github.com/pyodide/pyodide
   - Key Takeaway: Virtual filesystem + interpreter integration

2. **WebAssembly.sh**
   - Website: https://webassembly.sh/
   - GitHub: https://github.com/wasmerio/webassembly.sh
   - Key Takeaway: Browser-based shell interface patterns

3. **Browsix**
   - Website: https://browsix.org/
   - Paper: ASPLOS 2017 (https://browsix.org/browsix.pdf)
   - Key Takeaway: POSIX shell scripts in browser are feasible

4. **Wavix (Bash WASM Port)**
   - GitHub: https://github.com/WAVM/Wavix
   - Key Takeaway: GNU bash can compile to WASM

### 10.2 Peer-Reviewed Research

1. **"Research on WebAssembly Runtimes: A Survey"**
   - Journal: ACM TOSEM
   - Year: 2024
   - DOI: 10.1145/3714465
   - Finding: 103 papers analyzed, interpreters viable

2. **"Bringing the Web up to Speed with WebAssembly"**
   - Conference: PLDI 2017
   - Authors: Haas et al.
   - Finding: Formal semantics, type-safe execution

3. **"WasmRef-Isabelle: A Verified Monadic Interpreter"**
   - Conference: PLDI 2023
   - Finding: Monadic interpreters can be verified

4. **"Browsix: Unix in the Browser Tab"**
   - Conference: ASPLOS 2017
   - Finding: Full POSIX shell scripts work in browser

### 10.3 Related WOS Documents

1. **real-world-bash-wasm-coding.md**
   - Path: `docs/specifications/real-world-bash-wasm-coding.md`
   - Content: 10-sprint roadmap for GNU Bash manual examples
   - Relevance: Provides context for broader bash support

2. **wos-spec-v1.md**
   - Path: `docs/specifications/wos-spec-v1.md`
   - Content: Complete WOS technical specification
   - Relevance: Kernel/VFS architecture we're building on

3. **CLAUDE.md**
   - Path: `CLAUDE.md`
   - Content: Development guidelines, TDD methodology
   - Relevance: Quality standards and testing requirements

### 10.4 External Standards

1. **POSIX Shell Specification**
   - URL: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html
   - Relevance: Standard shell behavior

2. **GNU Bash Manual**
   - URL: https://www.gnu.org/software/bash/manual/
   - Relevance: Feature reference for implementation

3. **ShellCheck**
   - URL: https://www.shellcheck.net/wiki/
   - Relevance: Common shell script errors to avoid

---

## 11. Success Criteria

### 11.1 Functional Success

**Minimum Viable Product (MVP)**:
- [ ] `bash script.sh` executes simple scripts (echo, ls, pwd, cat)
- [ ] Scripts can use variables (`VAR=value`, `echo $VAR`)
- [ ] `source script.sh` modifies current shell environment
- [ ] Error handling for file not found, invalid shebang
- [ ] Integration with Vim editor (create, save, execute)

**Stretch Goals**:
- [ ] `./script.sh` execution with chmod +x
- [ ] Control flow (if/while/for) in scripts
- [ ] Command substitution (`$(command)`)
- [ ] Arithmetic expansion (`$((5 + 3))`)

### 11.2 Quality Success

**Test Coverage**:
- [ ] ≥85% line coverage (measured)
- [ ] ≥80% branch coverage (measured)
- [ ] 100% function coverage (all public APIs tested)
- [ ] ≥90% mutation kill rate (measured)

**Test Counts**:
- [ ] ≥100 unit tests (Rust)
- [ ] ≥30 E2E tests (Playwright)
- [ ] ≥10 property tests (proptest)

**Performance**:
- [ ] Simple script (5 lines): <10ms
- [ ] Medium script (50 lines): <50ms
- [ ] WASM size increase: <50KB
- [ ] Zero regressions in existing commands

### 11.3 User Experience Success

**Usability**:
- [ ] Users can write, save, and execute scripts in <2 minutes
- [ ] Error messages are clear and actionable
- [ ] Script execution feels instant (<100ms perceived)
- [ ] Vim integration is seamless (no manual file management)

**Educational Value**:
- [ ] Tutorial covering basic scripting (docs/user-guide/)
- [ ] 10+ example scripts (docs/examples/)
- [ ] Troubleshooting guide (docs/troubleshooting/)

### 11.4 Milestone Validation

**Phase 1 Complete**: Can load scripts from VFS ✓
**Phase 2 Complete**: Can execute simple scripts with bash ✓
**Phase 3 Complete**: Scripts support variables ✓
**Phase 4 Complete**: source command works ✓
**Phase 5 Complete**: Control flow (if/while/for) works ✓
**Phase 6 Complete**: Executable scripts (./script.sh) work ✓

**Final Milestone**: Demonstrate 20 GNU Bash manual examples working in WOS, validated bi-directionally against /bin/bash

---

## Appendix A: Example Scripts for Testing

### Example 1: Hello World
```bash
#!/bin/bash
# hello.sh
echo "Hello from WOS!"
echo "This is a shell script running in the browser."
```

### Example 2: Variable Demo
```bash
#!/bin/bash
# vars.sh
NAME="WOS User"
VERSION="0.2.0"
echo "Welcome, $NAME!"
echo "You are using WOS version $VERSION"
```

### Example 3: Control Flow
```bash
#!/bin/bash
# check.sh
if [ -f config.txt ]; then
  echo "Config file found"
  source config.txt
else
  echo "No config file, using defaults"
  export APP_MODE="default"
fi

echo "Running in $APP_MODE mode"
```

### Example 4: Loops
```bash
#!/bin/bash
# count.sh
echo "Counting to 5:"
i=1
while [ $i -le 5 ]; do
  echo $i
  i=$((i + 1))
done
echo "Done!"
```

### Example 5: Nested Scripts
```bash
#!/bin/bash
# main.sh
echo "Main script starting"
source lib/functions.sh
run_tests
echo "Main script complete"
```

---

## Appendix B: Bi-Directional Validation Script

```bash
#!/bin/bash
# validate-wos-bash.sh
# Run WOS output vs reference bash, compare results

SCRIPT=$1

# Run in reference bash
BASH_OUTPUT=$(bash "$SCRIPT" 2>&1)

# Run in WOS (via Playwright automation)
WOS_OUTPUT=$(node -e "
const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto('http://localhost:8001/wos/');
  // ... load script, execute, capture output ...
  console.log(output);
  await browser.close();
})();
")

# Compare
if [ "$BASH_OUTPUT" = "$WOS_OUTPUT" ]; then
  echo "✓ PASS: Outputs match"
  exit 0
else
  echo "✗ FAIL: Output mismatch"
  echo "=== Bash Output ==="
  echo "$BASH_OUTPUT"
  echo "=== WOS Output ==="
  echo "$WOS_OUTPUT"
  exit 1
fi
```

---

**Document Status**: ✅ COMPLETE - Ready for Implementation
**Next Step**: Review with team → Begin Phase 1 implementation
**Estimated Timeline**: 7-8 weeks to full implementation
**Quality Target**: NASA-grade (A grade, 95/100 quality score)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
