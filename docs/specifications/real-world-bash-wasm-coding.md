# Real-World Bash WASM Coding Specification
## GNU Bash Manual Examples in WOS - EXTREME TDD Roadmap

**Version:** 1.0.0
**Date:** 2025-10-15
**Authors:** WOS Core Team
**Status:** DRAFT - Roadmap for Implementation
**Inspired by:** bashrs BASH-INGESTION-ROADMAP.md and Phase 4 validation

---

## Executive Summary

This specification defines a comprehensive EXTREME TDD roadmap for implementing every relevant example from the GNU Bash manual as executable demonstrations in WOS (WebAssembly Operating System). Drawing inspiration from the bashrs project's bi-directional Bash-Rust validation methodology, we establish a systematic approach to validate that WOS can execute real-world shell scripting patterns in a browser-based WebAssembly environment.

**Goal**: 100% of applicable GNU Bash manual examples working in WOS terminal
**Methodology**: EXTREME TDD (Test-First, RED-GREEN-REFACTOR)
**Reference**: GNU Bash Manual (bash.pdf)
**Quality Standard**: 85%+ test coverage, 100% canary test pass rate, mutation testing

**Core Principles:**
1. Every example from the GNU Bash manual gets a corresponding test
2. Test-first development: Write canary test BEFORE implementation
3. Bi-directional validation: Test both in WOS and reference bash
4. Browser-based execution: All tests run via Playwright E2E testing
5. Quality gates: Coverage, mutation testing, property-based tests

---

## 1. Background & Inspiration

### 1.1 Learning from bashrs

The **bashrs project** (../bashrs) demonstrates world-class quality engineering for shell scripting:

**Key Insights Adopted:**
- **Bi-directional validation**: Test both transpiled code and reference implementation
- **Comprehensive roadmap**: Structured tracking from GNU Bash manual (120+ items)
- **EXTREME TDD methodology**: RED-GREEN-REFACTOR enforced
- **Phase 4 validation**: 19/19 examples transpile, covering 80% of use cases
- **Quality metrics**: A+ grade (98/100), 603 tests, 85% coverage
- **Toyota Way integration**: Jidoka, Hansei, Kaizen, Genchi Genbutsu

**Bashrs Achievements:**
```
Total Tasks:        ~120 items from GNU Bash manual
Completed (v0.x):   ~15 items ✅
Property Tests:     52 (~26K test cases)
Mutation Score:     ~83% (target ≥90%)
Coverage:           85.36% core
Quality Grade:      A+ (98/100)
```

### 1.2 WOS Current State

**Current WOS Capabilities (v0.1.0):**
- 11 WASM commands: ps, ls, cat, pwd, touch, mkdir, rm, echo, grep, wc, state
- Virtual File System (VFS) with in-memory storage
- Process management with PID tracking
- Browser-based terminal UI (HTML/CSS/JS + WASM)
- 60 canary tests (100% pass rate)
- 280 unit tests (100% pass rate)

**Gaps to Address:**
- Limited shell features (no pipes, no redirections, no variables)
- No shell scripting support (no functions, no control flow)
- No command chaining (&&, ||, ;)
- No environment variables or parameter expansion
- No I/O redirection (<, >, >>)
- No job control or background execution

---

## 2. Architecture & Design

### 2.1 WOS Shell Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Browser (JavaScript Runtime)                │
│  ┌───────────────────────────────────────────────────┐  │
│  │           HTML Terminal Interface                 │  │
│  │         (dist/wos/index.html + app.js)           │  │
│  └────────────────┬──────────────────────────────────┘  │
│                   │ executeCommand()                     │
│                   ▼                                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │           WOS WebAssembly Module                  │  │
│  │              (wos_bg.wasm)                        │  │
│  │                                                    │  │
│  │  ┌──────────────────────────────────────────┐    │  │
│  │  │     Shell Parser & Executor              │    │  │
│  │  │  (NEW: to be implemented)                │    │  │
│  │  │  - Tokenizer                             │    │  │
│  │  │  - Parser (variables, functions, pipes)  │    │  │
│  │  │  - Executor (control flow, redirects)    │    │  │
│  │  └────────────┬─────────────────────────────┘    │  │
│  │               │                                    │  │
│  │  ┌────────────▼─────────────────────────────┐    │  │
│  │  │     Kernel (wos-kernel)                  │    │  │
│  │  │  - Process Management                    │    │  │
│  │  │  - Virtual File System (VFS)             │    │  │
│  │  │  - Memory Management                     │    │  │
│  │  │  - Syscall Interface                     │    │  │
│  │  └────────────┬─────────────────────────────┘    │  │
│  │               │                                    │  │
│  │  ┌────────────▼─────────────────────────────┐    │  │
│  │  │     Commands (wos-userspace)             │    │  │
│  │  │  - ls, cat, grep, wc, echo, pwd, etc.   │    │  │
│  │  │  - touch, mkdir, rm                      │    │  │
│  │  │  - ps, state                             │    │  │
│  │  └──────────────────────────────────────────┘    │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Testing Architecture

```
┌─────────────────────────────────────────────────────────┐
│         Testing Infrastructure (Playwright E2E)          │
│                                                          │
│  ┌─────────────────────────────────────────────┐        │
│  │    Canary Tests (e2e/tests/canary/)        │        │
│  │  - Terminal interaction tests               │        │
│  │  - Command execution tests                  │        │
│  │  - Shell feature tests (NEW)                │        │
│  │  - GNU Bash manual example tests (NEW)      │        │
│  └───────────────┬─────────────────────────────┘        │
│                  │                                       │
│                  ▼                                       │
│  ┌─────────────────────────────────────────────┐        │
│  │    Browser Automation (Chromium)            │        │
│  │  - http://localhost:8000/                   │        │
│  │  - Terminal input/output capture            │        │
│  │  - Screenshot comparison                    │        │
│  │  - Performance measurements                 │        │
│  └───────────────┬─────────────────────────────┘        │
│                  │                                       │
│                  ▼                                       │
│  ┌─────────────────────────────────────────────┐        │
│  │    Reference Bash Validation (NEW)          │        │
│  │  - Run same examples in /bin/bash           │        │
│  │  - Compare outputs (deterministic parts)     │        │
│  │  - Validate semantics match                 │        │
│  └─────────────────────────────────────────────┘        │
│                                                          │
│  ┌─────────────────────────────────────────────┐        │
│  │    Unit Tests (Rust)                        │        │
│  │  - kernel/ tests (160 tests)                │        │
│  │  - shared/ tests (17 tests)                 │        │
│  │  - userspace/ tests (45 tests)              │        │
│  │  - wos/ tests (58 tests)                    │        │
│  └─────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────┘
```

### 2.3 Quality Gates

Following bashrs extreme quality approach:

```
┌──────────────────────────────────────────┐
│      Pre-Commit Quality Gates            │
│  (Runs automatically via git hook)       │
├──────────────────────────────────────────┤
│  1. Format Check (cargo fmt)             │
│  2. Lint Check (cargo clippy)            │
│  3. Unit Tests (cargo test)              │
│  4. Canary Tests (playwright)            │
│  Duration: <30s (fast gate)              │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│      CI/CD Quality Gates                 │
│  (Runs on push to main)                  │
├──────────────────────────────────────────┤
│  1. All Pre-Commit Gates                 │
│  2. Coverage Check (≥85%)                │
│  3. Mutation Testing (≥85%)              │
│  4. WASM Size Check (≤500KB)             │
│  5. Performance Benchmarks               │
│  6. Cross-Browser Testing                │
│  7. GNU Bash Example Validation          │
│  Duration: <5min (full gate)             │
└──────────────────────────────────────────┘
```

---

## 3. GNU Bash Manual Roadmap

### 3.1 Roadmap Structure

Following bashrs approach, we document EVERY applicable construct from the GNU Bash manual.

**Task Format:**
```markdown
- [ ] **Task**: <Feature description>
  - GNU Example: `<bash code from manual>`
  - WOS Implementation: `<how it works in WOS>`
  - Test: `test_<feature>_in_wos`
  - Status: ❌ Not Started / 🔄 In Progress / ✅ Complete
  - Sprint: <sprint number>
  - Coverage: <percentage>
```

### 3.2 Chapter 3: Basic Shell Features

#### 3.2.1 Simple Commands
- [ ] **Task**: Implement simple command execution
  - GNU Example: `echo "Hello World"`
  - WOS Implementation: Already works via `executeCommand("echo Hello World")`
  - Test: `test_simple_command_echo`
  - Status: ✅ Complete (v0.1.0)
  - Coverage: 100%

- [ ] **Task**: Command with arguments and quoting
  - GNU Example: `mkdir -p "/tmp/my directory/subdir"`
  - WOS Implementation: `mkdir` command with quoted path parsing
  - Test: `test_command_with_quoted_args`
  - Status: 🔄 Partial (mkdir exists, needs quote handling)
  - Sprint: 1
  - Coverage: 60%

#### 3.2.2 Pipelines
- [ ] **Task**: Implement pipe operator (|)
  - GNU Example: `cat file.txt | grep "pattern" | wc -l`
  - WOS Implementation: Parse pipe operator, chain command outputs
  - Test: `test_pipeline_basic`
  - Status: ❌ Not Started
  - Sprint: 2
  - Coverage: 0%

**Implementation Strategy:**
```rust
// In wos/src/lib.rs
fn execute_command_with_pipes(&mut self, cmd: String) -> String {
    let commands = parse_pipeline(&cmd); // Split by |
    let mut output = String::new();

    for command in commands {
        // Pipe previous output as stdin to next command
        output = self.execute_single_command(command, Some(&output));
    }

    output
}
```

**Test (Playwright):**
```typescript
test('C10: Pipe commands together', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#status:has-text("Ready")');

  // Create test file
  await typeCommand(page, 'touch test.txt');
  await typeCommand(page, 'echo "hello" > test.txt');

  // Test pipe
  await typeCommand(page, 'cat test.txt | grep hello');
  const output = await getLastOutput(page);
  expect(output).toBe('hello');
});
```

#### 3.2.3 Lists (Command Chaining)
- [ ] **Task**: Implement && operator (AND list)
  - GNU Example: `mkdir /tmp/test && cd /tmp/test && touch file.txt`
  - WOS Implementation: Execute commands sequentially, stop on failure
  - Test: `test_and_list_success`, `test_and_list_failure`
  - Status: ❌ Not Started
  - Sprint: 2
  - Coverage: 0%

- [ ] **Task**: Implement || operator (OR list)
  - GNU Example: `test -f config.txt || echo "Config not found"`
  - WOS Implementation: Execute second command only if first fails
  - Test: `test_or_list_fallback`
  - Status: ❌ Not Started
  - Sprint: 2
  - Coverage: 0%

- [ ] **Task**: Implement ; operator (sequential execution)
  - GNU Example: `cmd1 ; cmd2 ; cmd3`
  - WOS Implementation: Execute all commands regardless of exit status
  - Test: `test_sequential_execution`
  - Status: ❌ Not Started
  - Sprint: 2
  - Coverage: 0%

#### 3.2.4 Compound Commands

##### While Loop
- [ ] **Task**: Implement while loop
  - GNU Example:
    ```bash
    i=0
    while [ $i -lt 5 ]; do
      echo $i
      i=$((i + 1))
    done
    ```
  - WOS Implementation: Parse `while [ condition ]; do ... done` syntax
  - Test: `test_while_loop_basic`, `test_while_loop_counter`
  - Status: ❌ Not Started
  - Sprint: 3
  - Coverage: 0%

##### For Loop
- [ ] **Task**: Implement for-in loop
  - GNU Example:
    ```bash
    for file in *.txt; do
      cat "$file"
    done
    ```
  - WOS Implementation: Parse `for var in list; do ... done` syntax
  - Test: `test_for_loop_files`, `test_for_loop_range`
  - Status: ❌ Not Started
  - Sprint: 3
  - Coverage: 0%

##### If Statement
- [ ] **Task**: Implement if-then-else
  - GNU Example:
    ```bash
    if [ -f "file.txt" ]; then
      echo "File exists"
    else
      echo "File not found"
    fi
    ```
  - WOS Implementation: Parse `if [ condition ]; then ... else ... fi` syntax
  - Test: `test_if_file_exists`, `test_if_string_equality`
  - Status: ❌ Not Started
  - Sprint: 3
  - Coverage: 0%

##### Case Statement
- [ ] **Task**: Implement case-match
  - GNU Example:
    ```bash
    case $var in
      1) echo "one" ;;
      2) echo "two" ;;
      *) echo "other" ;;
    esac
    ```
  - WOS Implementation: Parse `case $var in ... esac` syntax
  - Test: `test_case_pattern_matching`
  - Status: ❌ Not Started
  - Sprint: 4
  - Coverage: 0%

### 3.3 Shell Functions

- [ ] **Task**: Implement function definition
  - GNU Example:
    ```bash
    greet() {
      echo "Hello $1"
    }
    greet "World"
    ```
  - WOS Implementation: Parse function syntax, store in symbol table
  - Test: `test_function_definition`, `test_function_with_args`
  - Status: ❌ Not Started
  - Sprint: 4
  - Coverage: 0%

### 3.4 Shell Parameters

#### Positional Parameters
- [ ] **Task**: Implement $1, $2, etc.
  - GNU Example: `echo "First: $1, Second: $2"`
  - WOS Implementation: Parse and substitute positional parameters
  - Test: `test_positional_parameters`
  - Status: ❌ Not Started
  - Sprint: 5
  - Coverage: 0%

#### Special Parameters
- [ ] **Task**: Implement $# (argument count)
  - GNU Example: `echo "Args: $#"`
  - WOS Implementation: Track argument count
  - Test: `test_arg_count`
  - Status: ❌ Not Started
  - Sprint: 5
  - Coverage: 0%

- [ ] **Task**: Implement $? (exit status)
  - GNU Example: `cmd; echo "Exit code: $?"`
  - WOS Implementation: Track last command exit status
  - Test: `test_exit_status`
  - Status: ❌ Not Started
  - Sprint: 5
  - Coverage: 0%

- [ ] **Task**: Implement $$ (process ID)
  - GNU Example: `echo "PID: $$"`
  - WOS Implementation: Return current process PID from kernel
  - Test: `test_process_id`
  - Status: ❌ Not Started
  - Sprint: 5
  - Coverage: 0%

### 3.5 Shell Expansions

#### Parameter Expansion
- [ ] **Task**: Implement ${var:-default}
  - GNU Example: `echo "${EDITOR:-vim}"`
  - WOS Implementation: Return var value or default if unset
  - Test: `test_default_value_expansion`
  - Status: ❌ Not Started
  - Sprint: 6
  - Coverage: 0%

- [ ] **Task**: Implement ${#var} (length)
  - GNU Example: `echo "${#PATH}"`
  - WOS Implementation: Return string length
  - Test: `test_string_length`
  - Status: ❌ Not Started
  - Sprint: 6
  - Coverage: 0%

- [ ] **Task**: Implement ${var%suffix}
  - GNU Example: `file="test.txt"; echo "${file%.txt}"`
  - WOS Implementation: Strip shortest suffix match
  - Test: `test_remove_suffix`
  - Status: ❌ Not Started
  - Sprint: 6
  - Coverage: 0%

#### Command Substitution
- [ ] **Task**: Implement $(command)
  - GNU Example: `now=$(date); echo "Time: $now"`
  - WOS Implementation: Execute command, capture output as string
  - Test: `test_command_substitution`
  - Status: ❌ Not Started
  - Sprint: 7
  - Coverage: 0%

#### Arithmetic Expansion
- [ ] **Task**: Implement $((expression))
  - GNU Example: `result=$((5 + 3 * 2)); echo $result`
  - WOS Implementation: Evaluate arithmetic expression
  - Test: `test_arithmetic_expansion`
  - Status: ❌ Not Started
  - Sprint: 7
  - Coverage: 0%

### 3.6 Redirections

- [ ] **Task**: Implement > (output redirection)
  - GNU Example: `echo "text" > file.txt`
  - WOS Implementation: Write command output to VFS file
  - Test: `test_output_redirection`
  - Status: ❌ Not Started
  - Sprint: 8
  - Coverage: 0%

- [ ] **Task**: Implement >> (append redirection)
  - GNU Example: `echo "more" >> file.txt`
  - WOS Implementation: Append command output to VFS file
  - Test: `test_append_redirection`
  - Status: ❌ Not Started
  - Sprint: 8
  - Coverage: 0%

- [ ] **Task**: Implement < (input redirection)
  - GNU Example: `wc -l < file.txt`
  - WOS Implementation: Read file content as command input
  - Test: `test_input_redirection`
  - Status: ❌ Not Started
  - Sprint: 8
  - Coverage: 0%

- [ ] **Task**: Implement 2> (stderr redirection)
  - GNU Example: `cmd 2> errors.log`
  - WOS Implementation: Separate stderr stream to file
  - Test: `test_stderr_redirection`
  - Status: ❌ Not Started
  - Sprint: 8
  - Coverage: 0%

- [ ] **Task**: Implement &> (stdout + stderr)
  - GNU Example: `cmd &> output.log`
  - WOS Implementation: Redirect both streams to file
  - Test: `test_combined_redirection`
  - Status: ❌ Not Started
  - Sprint: 8
  - Coverage: 0%

### 3.7 Builtin Commands

- [ ] **Task**: Implement cd (change directory)
  - GNU Example: `cd /tmp; pwd`
  - WOS Implementation: Track current working directory per process
  - Test: `test_cd_command`, `test_cd_with_pwd`
  - Status: 🔄 Partial (pwd always returns "/")
  - Sprint: 9
  - Coverage: 20%

- [ ] **Task**: Implement export (environment variables)
  - GNU Example: `export PATH="/usr/local/bin:$PATH"`
  - WOS Implementation: Store environment variables in process context
  - Test: `test_export_variable`
  - Status: ❌ Not Started
  - Sprint: 9
  - Coverage: 0%

- [ ] **Task**: Implement source (. command)
  - GNU Example: `. ./config.sh`
  - WOS Implementation: Execute script in current shell context
  - Test: `test_source_script`
  - Status: ❌ Not Started
  - Sprint: 10
  - Coverage: 0%

- [ ] **Task**: Implement test ([ command)
  - GNU Example: `[ -f file.txt ] && echo "exists"`
  - WOS Implementation: File/string test operations
  - Test: `test_file_test`, `test_string_test`, `test_numeric_test`
  - Status: ❌ Not Started
  - Sprint: 10
  - Coverage: 0%

---

## 4. Implementation Sprints

### Sprint 1: Quote Handling & Argument Parsing (1 week)
**Goal**: Properly handle quoted arguments in all commands

**Tickets:**
- [ ] WOS-101: Implement quote parser (single, double, escape)
- [ ] WOS-102: Update all commands to use quoted argument parser
- [ ] WOS-103: Add tests for spaces in filenames
- [ ] WOS-104: Test special characters ($, `, \, etc.)

**Tests:**
```typescript
test('C11: Quoted arguments with spaces', async ({ page }) => {
  await typeCommand(page, 'mkdir "my directory"');
  await typeCommand(page, 'touch "my directory/my file.txt"');
  await typeCommand(page, 'ls');
  expect(output).toContain('my directory');
});
```

**Acceptance Criteria:**
- All 11 commands handle quoted arguments
- Tests pass for spaces, special chars
- No regression in existing tests

### Sprint 2: Command Chaining (2 weeks)
**Goal**: Implement &&, ||, ; operators and pipes

**Tickets:**
- [ ] WOS-201: Implement pipe operator (|) with output chaining
- [ ] WOS-202: Implement && (AND list) operator
- [ ] WOS-203: Implement || (OR list) operator
- [ ] WOS-204: Implement ; (sequential) operator
- [ ] WOS-205: Track exit codes for operators
- [ ] WOS-206: Add 20+ canary tests for chaining

**Example Tests:**
```typescript
test('C20: Pipe multiple commands', async ({ page }) => {
  await typeCommand(page, 'echo "hello\nworld" | grep world | wc -l');
  expect(output).toBe('1');
});

test('C21: AND operator stops on failure', async ({ page }) => {
  await typeCommand(page, 'false && echo "should not print"');
  expect(output).not.toContain('should not print');
});
```

**Acceptance Criteria:**
- 4 operators working correctly
- Exit code handling correct
- 20 new canary tests passing
- Coverage ≥85%

### Sprint 3: Control Flow (3 weeks)
**Goal**: Implement while, for, if statements

**Tickets:**
- [ ] WOS-301: Implement AST for control flow
- [ ] WOS-302: Parser for while loops
- [ ] WOS-303: Parser for for-in loops
- [ ] WOS-304: Parser for if-then-else
- [ ] WOS-305: Implement test conditions ([ ])
- [ ] WOS-306: Add 30+ control flow tests

**Example Tests:**
```typescript
test('C30: While loop with counter', async ({ page }) => {
  const script = `
    i=0
    while [ $i -lt 3 ]; do
      echo $i
      i=$((i + 1))
    done
  `;
  await typeCommand(page, script);
  expect(output).toBe('0\n1\n2');
});
```

**Acceptance Criteria:**
- while, for, if working
- Nested control flow supported
- 30 new tests passing
- Coverage ≥85%

### Sprint 4: Functions & Case (2 weeks)
**Goal**: Implement function definitions and case statements

**Tickets:**
- [ ] WOS-401: Function definition parser
- [ ] WOS-402: Function call mechanism
- [ ] WOS-403: Function local scope
- [ ] WOS-404: Case statement parser
- [ ] WOS-405: Pattern matching for case
- [ ] WOS-406: Add 20+ function tests

**Acceptance Criteria:**
- Functions with arguments working
- Case statement pattern matching
- 20 new tests passing
- Coverage ≥85%

### Sprint 5: Variables & Parameters (2 weeks)
**Goal**: Implement shell variables and special parameters

**Tickets:**
- [ ] WOS-501: Variable storage and retrieval
- [ ] WOS-502: Positional parameters ($1, $2, etc.)
- [ ] WOS-503: Special parameters ($#, $?, $$, $@)
- [ ] WOS-504: Variable assignment in commands
- [ ] WOS-505: Add 25+ variable tests

**Acceptance Criteria:**
- All parameter types working
- Variable scope correct
- 25 new tests passing
- Coverage ≥85%

### Sprint 6: Parameter Expansion (2 weeks)
**Goal**: Implement ${var} expansions

**Tickets:**
- [ ] WOS-601: Basic expansion ${var}
- [ ] WOS-602: Default value ${var:-default}
- [ ] WOS-603: String length ${#var}
- [ ] WOS-604: Suffix removal ${var%pattern}
- [ ] WOS-605: Prefix removal ${var#pattern}
- [ ] WOS-606: Add 20+ expansion tests

**Acceptance Criteria:**
- 5 expansion types working
- Nested expansions supported
- 20 new tests passing
- Coverage ≥85%

### Sprint 7: Command & Arithmetic Substitution (2 weeks)
**Goal**: Implement $(command) and $((expression))

**Tickets:**
- [ ] WOS-701: Command substitution parser
- [ ] WOS-702: Arithmetic expression evaluator
- [ ] WOS-703: Nested substitutions
- [ ] WOS-704: Add 20+ substitution tests

**Acceptance Criteria:**
- Both substitution types working
- Nested/combined usage works
- 20 new tests passing
- Coverage ≥85%

### Sprint 8: I/O Redirection (2 weeks)
**Goal**: Implement >, >>, <, 2>, &> redirections

**Tickets:**
- [ ] WOS-801: Output redirection > implementation
- [ ] WOS-802: Append redirection >> implementation
- [ ] WOS-803: Input redirection < implementation
- [ ] WOS-804: Stderr redirection 2> implementation
- [ ] WOS-805: Combined redirection &> implementation
- [ ] WOS-806: Add 25+ redirection tests

**Acceptance Criteria:**
- All 5 redirection types working
- Multiple redirections per command
- 25 new tests passing
- Coverage ≥85%

### Sprint 9: Directory & Environment (2 weeks)
**Goal**: Implement cd, export, environment variables

**Tickets:**
- [ ] WOS-901: Per-process working directory
- [ ] WOS-902: cd command with relative/absolute paths
- [ ] WOS-903: Environment variable storage
- [ ] WOS-904: export command implementation
- [ ] WOS-905: $HOME, $PATH, $PWD variables
- [ ] WOS-906: Add 20+ environment tests

**Acceptance Criteria:**
- cd/pwd working correctly
- Environment variables accessible
- 20 new tests passing
- Coverage ≥85%

### Sprint 10: Advanced Builtins (2 weeks)
**Goal**: Implement test ([), source (.)

**Tickets:**
- [ ] WOS-1001: File test operators (-f, -d, -e, -r, -w, -x)
- [ ] WOS-1002: String test operators (=, !=, -z, -n)
- [ ] WOS-1003: Numeric test operators (-eq, -ne, -lt, -gt, -le, -ge)
- [ ] WOS-1004: source (.) command for script execution
- [ ] WOS-1005: Add 25+ test operator tests

**Acceptance Criteria:**
- 15+ test operators working
- source command working
- 25 new tests passing
- Coverage ≥85%

---

## 5. Testing Strategy

### 5.1 Test Categories

Following bashrs methodology:

**1. Unit Tests (Rust)**
- Test individual functions in isolation
- Mock dependencies (VFS, process table)
- Fast execution (<1s for all unit tests)
- Target: 280+ tests (current) → 500+ tests

**2. Canary Tests (Playwright E2E)**
- Test user-facing functionality in browser
- Critical workflow validation (SQLite-inspired)
- Each GNU Bash example gets a canary test
- Target: 60 tests (current) → 200+ tests

**3. Property-Based Tests (proptest)**
- Generate random inputs, verify invariants
- Example: parse(generate_command()) never panics
- Target: 25+ properties, 10,000+ test cases

**4. Mutation Tests (cargo-mutant)**
- Verify tests catch bugs
- Change code, ensure tests fail
- Target: ≥85% mutation kill rate

**5. Bi-directional Validation (NEW)**
- Run same example in WOS and /bin/bash
- Compare deterministic outputs
- Validate semantic equivalence

### 5.2 Canary Test Structure

```typescript
// e2e/tests/canary/03-bash-manual-examples.spec.ts

test.describe('GNU Bash Manual Examples', () => {
  test.describe('Chapter 3.2: Pipelines', () => {
    test('C100: Basic pipe - cat | grep', async ({ page }) => {
      await page.goto('/');
      await page.waitForSelector('#status:has-text("Ready")');

      // Setup
      await typeCommand(page, 'echo "hello\nworld" > test.txt');

      // Execute pipeline
      await typeCommand(page, 'cat test.txt | grep world');
      const output = await getLastOutput(page);

      // Validate
      expect(output).toBe('world');

      // Cleanup
      await typeCommand(page, 'rm test.txt');
    });

    test('C101: Three-stage pipe - cat | grep | wc', async ({ page }) => {
      // Similar structure...
    });
  });

  test.describe('Chapter 3.4: While Loops', () => {
    test('C110: While loop with counter', async ({ page }) => {
      // Test while loop...
    });
  });
});
```

### 5.3 Bi-directional Validation

```typescript
// e2e/tests/validation/bash-reference.spec.ts

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

test('Validate: Pipeline output matches bash', async ({ page }) => {
  // Execute in WOS
  await page.goto('/');
  await typeCommand(page, 'echo "test" | grep test');
  const wosOutput = await getLastOutput(page);

  // Execute in reference bash
  const { stdout } = await execAsync('bash -c "echo \\"test\\" | grep test"');
  const bashOutput = stdout.trim();

  // Compare
  expect(wosOutput).toBe(bashOutput);
});
```

### 5.4 Test Coverage Targets

| Component | Current | Target | Status |
|-----------|---------|--------|--------|
| Unit Tests (Rust) | 280 tests | 500 tests | 🔄 Expand |
| Canary Tests (E2E) | 60 tests | 200 tests | 🔄 Expand |
| Property Tests | 0 properties | 25+ properties | ❌ Start |
| Mutation Kill Rate | Not measured | ≥85% | ❌ Start |
| Line Coverage | ~85% | ≥85% | ✅ Maintain |
| Branch Coverage | Not measured | ≥80% | 🔄 Measure |

---

## 6. Quality Metrics

### 6.1 Metrics Dashboard

Track the following metrics (bashrs-inspired):

```toml
# quality.toml (NEW file to create)

[metrics]
# Test Metrics
unit_tests_target = 500
canary_tests_target = 200
property_tests_target = 25
test_pass_rate_target = 100.0

# Coverage Metrics
line_coverage_target = 85.0
branch_coverage_target = 80.0
function_coverage_target = 100.0

# Complexity Metrics
cyclomatic_complexity_max = 10
cognitive_complexity_max = 15

# Performance Metrics
wasm_size_max_kb = 500
command_execution_max_us = 1000

# Mutation Testing
mutation_kill_rate_target = 85.0

# Quality Scoring
satd_tolerance = 0  # Zero TODO/FIXME/HACK comments
unsafe_blocks_tolerance = 0
```

### 6.2 Quality Grade Calculation

Following bashrs A+ grade (98/100) methodology:

```
Quality Score = Weighted Average of:
  - Test Pass Rate (20%)
  - Code Coverage (20%)
  - Mutation Kill Rate (20%)
  - Complexity Score (15%)
  - Performance Score (15%)
  - Zero Violations (10%) [SATD, unsafe, clippy]

Grade Thresholds:
  A+ : ≥98
  A  : 95-97
  A- : 92-94
  B+ : 88-91
  B  : 85-87
  C  : 80-84
  F  : <80
```

**Current WOS Score (estimated):**
```
Test Pass Rate:    100% → 20/20 points
Code Coverage:     ~85% → 17/20 points
Mutation Kill:     N/A  → 0/20 points
Complexity:        Good → 14/15 points
Performance:       Good → 14/15 points
Zero Violations:   Good → 10/10 points
────────────────────────────────────────
TOTAL:                   75/100 (C)

Target after 10 sprints: 95/100 (A)
```

---

## 7. Success Criteria

### 7.1 Phase 1: Foundation (Sprints 1-3)
**Timeframe:** 6 weeks

**Deliverables:**
- [ ] Quote handling for all commands
- [ ] Command chaining (&&, ||, ;, |)
- [ ] Control flow (while, for, if)
- [ ] 70+ new canary tests
- [ ] Coverage ≥85%

**Success Metrics:**
- 100% test pass rate
- Quality score ≥80 (B)
- WASM size ≤500KB

### 7.2 Phase 2: Expansion (Sprints 4-7)
**Timeframe:** 8 weeks

**Deliverables:**
- [ ] Functions and case statements
- [ ] Variables and parameters
- [ ] Parameter expansion
- [ ] Command/arithmetic substitution
- [ ] 85+ new canary tests
- [ ] Property-based tests started

**Success Metrics:**
- 100% test pass rate
- Coverage ≥85%
- Quality score ≥85 (B+)
- 10+ property tests

### 7.3 Phase 3: Completion (Sprints 8-10)
**Timeframe:** 6 weeks

**Deliverables:**
- [ ] I/O redirection
- [ ] cd, export, environment
- [ ] test ([) and source (.)
- [ ] 70+ new canary tests
- [ ] Mutation testing infrastructure
- [ ] Bi-directional validation

**Success Metrics:**
- 100% test pass rate
- Coverage ≥85%
- Quality score ≥95 (A)
- Mutation kill rate ≥85%
- 200+ canary tests
- 25+ property tests

### 7.4 Final Goal
**Every applicable GNU Bash manual example working in WOS**

**Quantitative Targets:**
- 500+ unit tests (100% pass)
- 200+ canary tests (100% pass)
- 25+ property tests
- 85%+ line coverage
- 80%+ branch coverage
- 85%+ mutation kill rate
- Quality grade: A (95/100)

**Qualitative Success:**
- Users can run real bash scripts in browser
- Educational value: learning shell scripting via WOS
- Community showcase: "Look, bash runs in WASM!"
- Documentation: Every example explained

---

## 8. Documentation Plan

### 8.1 User-Facing Documentation

**Create: `docs/bash-examples/`**
```
docs/bash-examples/
├── README.md                    # Overview and navigation
├── 01-simple-commands.md        # Chapter 3.2.1 examples
├── 02-pipelines.md              # Chapter 3.2.2 examples
├── 03-command-chaining.md       # Chapter 3.2.3 examples
├── 04-while-loops.md            # While loop examples
├── 05-for-loops.md              # For loop examples
├── 06-if-statements.md          # If statement examples
├── 07-functions.md              # Function examples
├── 08-variables.md              # Variable examples
├── 09-expansions.md             # Parameter expansion examples
├── 10-redirections.md           # I/O redirection examples
└── 11-builtins.md               # Builtin commands examples
```

**Each file structure:**
```markdown
# <Topic>

## Overview
Brief explanation of the feature.

## Examples from GNU Bash Manual

### Example 1: <Title>
**From:** GNU Bash Manual Section X.Y.Z

**Code:**
```bash
# Original bash example
...
```

**Try in WOS:**
```bash
# Same example, runnable in WOS terminal
...
```

**Explanation:**
How it works, what to expect.

**Test Coverage:**
Link to canary test: `test_example_1`

### Example 2: ...
```

### 8.2 Developer Documentation

**Update existing docs:**
- `README.md` - Add link to Bash Examples
- `CONTRIBUTING.md` - Add testing guidelines
- `ARCHITECTURE.md` - Document shell parser design

**Create new docs:**
- `docs/testing/EXTREME-TDD.md` - TDD methodology
- `docs/testing/PROPERTY-TESTING.md` - Property test guide
- `docs/testing/MUTATION-TESTING.md` - Mutation testing guide
- `docs/quality/METRICS.md` - Quality metrics tracking

---

## 9. Risks & Mitigations

### 9.1 Technical Risks

**Risk 1: WASM size bloat**
- Problem: Adding shell parser increases WASM binary size
- Mitigation: Modular design, lazy loading, size budgets
- Monitoring: Track size per sprint, block >10% increases

**Risk 2: Performance degradation**
- Problem: Complex parsing slows down command execution
- Mitigation: Benchmark every sprint, optimize hot paths
- Monitoring: Performance tests in CI, <1ms target

**Risk 3: Browser compatibility**
- Problem: Advanced features may not work in all browsers
- Mitigation: Cross-browser testing, feature detection
- Monitoring: Playwright tests on Chrome, Firefox, Safari

### 9.2 Scope Risks

**Risk 4: Feature creep**
- Problem: Trying to implement too many features
- Mitigation: Follow 10-sprint roadmap strictly
- Monitoring: Sprint retrospectives, scope reviews

**Risk 5: Testing burden**
- Problem: 200+ tests slow down development
- Mitigation: Fast unit tests, parallel E2E tests
- Monitoring: CI time budget: <5 minutes

### 9.3 Quality Risks

**Risk 6: Insufficient test coverage**
- Problem: New features added without adequate tests
- Mitigation: EXTREME TDD, coverage gates
- Monitoring: Pre-commit hooks block <85% coverage

**Risk 7: Brittle tests**
- Problem: Tests break on unrelated changes
- Mitigation: Property-based tests, clear test intent
- Monitoring: Test flakiness tracking, <1% flake rate

---

## 10. Next Steps

### Immediate Actions (This Week)
1. [ ] Review this specification with team
2. [ ] Create `docs/bash-examples/` directory structure
3. [ ] Write first 5 bash example docs
4. [ ] Create `quality.toml` configuration
5. [ ] Set up property testing infrastructure (proptest)

### Short-term (Sprint 1 - Week 1-2)
1. [ ] Implement quote parser
2. [ ] Add 10 new canary tests for quoted arguments
3. [ ] Update all 11 commands to use quote parser
4. [ ] Achieve 100% pass rate
5. [ ] Document Sprint 1 completion

### Medium-term (Sprints 2-5 - Weeks 3-14)
1. [ ] Complete Phases 1-2 (command chaining through variables)
2. [ ] Reach 150+ canary tests
3. [ ] Implement 15+ property tests
4. [ ] Quality score ≥85 (B+)

### Long-term (Sprints 6-10 - Weeks 15-26)
1. [ ] Complete Phase 3 (all 10 sprints)
2. [ ] Reach 200+ canary tests
3. [ ] Implement mutation testing
4. [ ] Quality score ≥95 (A)
5. [ ] Celebrate: Every GNU Bash example works in WOS! 🎉

---

## 11. Appendix

### 11.1 Comparison: bashrs vs WOS

| Aspect | bashrs | WOS |
|--------|---------|-----|
| **Goal** | Rust → Bash transpiler | Browser-based OS with Bash |
| **Environment** | Native shell | WebAssembly in browser |
| **Quality Grade** | A+ (98/100) | Target: A (95/100) |
| **Test Count** | 603 tests | 280 → 500+ tests |
| **Coverage** | 85.36% | ~85% → maintain |
| **Mutation** | 83% → 90% target | 0% → 85% target |
| **Property Tests** | 52 (~26K cases) | 0 → 25+ (10K+ cases) |
| **GNU Bash Examples** | ~15/120 complete | 0/120 → 100% target |

### 11.2 Reference Materials

**Primary Reference:**
- GNU Bash Manual (bash.pdf)
- Available: https://www.gnu.org/software/bash/manual/

**Inspiration:**
- bashrs project: `../bashrs/`
- bashrs roadmap: `../bashrs/docs/BASH-INGESTION-ROADMAP.md`
- bashrs quality: `../bashrs/EXTREME_QUALITY_IMPLEMENTATION.md`

**Standards:**
- POSIX Shell Specification
- ShellCheck rules: https://www.shellcheck.net/wiki/

### 11.3 Glossary

- **Canary Test**: Critical workflow validation test (SQLite-inspired)
- **EXTREME TDD**: Test-first methodology (RED-GREEN-REFACTOR)
- **Property Test**: Generative testing with invariants
- **Mutation Testing**: Verify tests catch bugs by changing code
- **Bi-directional Validation**: Compare WOS and bash outputs
- **Quality Score**: Weighted metric (0-100 scale)

---

**Status**: DRAFT - Ready for Review
**Next Review**: After Sprint 1 completion
**Maintainer**: WOS Core Team
**Last Updated**: 2025-10-15

🤖 Generated with [Claude Code](https://claude.com/claude-code)
