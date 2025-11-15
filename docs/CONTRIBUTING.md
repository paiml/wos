# Contributing to WOS

Thank you for your interest in contributing to WOS! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Quality Standards](#quality-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Project Structure](#project-structure)
- [Common Tasks](#common-tasks)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inspiring community for everyone.

### Our Standards

- **Be respectful** - Treat all contributors with respect
- **Be constructive** - Provide actionable feedback
- **Be collaborative** - Work together toward common goals
- **Be professional** - Maintain professional communication

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Publishing others' private information
- Other conduct inappropriate in a professional setting

## Getting Started

### Prerequisites

- **Rust** 1.70+ with `wasm32-unknown-unknown` target
- **wasm-bindgen-cli** 0.2.104+
- **Git** for version control
- **Make** for build automation

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/paiml/wos
cd wos

# Install Rust target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen
cargo install wasm-bindgen-cli

# Install development tools
cargo install cargo-llvm-cov
cargo install cargo-mutants

# Run tests to verify setup
make test

# Build WASM
make wasm
```

### Project Layout

```
wos/
├── shared/           # Shared types and utilities
├── kernel/           # Kernel implementation
├── userspace/        # Userspace programs
├── wos/              # WASM bindings
├── dist/wos/         # Browser interface
├── docs/             # Documentation
└── Makefile          # Build system
```

## Development Workflow

### 1. Find or Create an Issue

- Check existing issues (check repository)
- Comment that you're working on it
- Create new issue if needed with clear description

### 2. Create a Branch

We work off `main` - no separate development branches.

```bash
# Always work on main
git checkout main
git pull origin main
```

### 3. Follow TDD

**Always write tests before implementation:**

```rust
// 1. Write failing test
#[test]
fn test_new_feature() {
    let result = new_feature();
    assert_eq!(result, expected);
}

// 2. Run tests (should fail)
cargo test

// 3. Implement feature
pub fn new_feature() -> Result {
    // Implementation
}

// 4. Run tests (should pass)
cargo test
```

### 4. Quality Gates

**Before committing, verify all quality gates pass:**

```bash
# Fast quality checks (<30s)
make quality

# Includes:
# - cargo fmt --check (formatting)
# - cargo clippy (linting)
# - cargo test (unit tests)
```

### 5. Commit Changes

**Commit message format:**

```
<type>: <description>

[optional body]

🤖 Generated with Claude Code
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Add or update tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `style`: Code style (formatting, etc.)

**Example:**

```bash
git add .
git commit -m "feat: add sys_getppid syscall

Implements getting parent process ID following pure functional pattern.
Includes 5 unit tests and 2 property tests.

🤖 Generated with Claude Code"
```

### 6. Push and Create PR

```bash
# Push to main
git push origin main

# Create PR via GitHub UI or gh CLI
gh pr create --title "Add sys_getppid syscall" \
  --body "Implements WOS-022: Parent PID syscall"
```

## Quality Standards

### Code Quality Requirements

All contributions must meet these standards:

#### 1. Test Coverage

- **Minimum**: 85% line coverage
- **Target**: 88%+
- **Verify**: `make coverage`

```bash
# Generate coverage report
make coverage

# Check thresholds
make coverage-check
```

#### 2. Code Complexity

- **Maximum**: 15 cyclomatic complexity per function
- **Average**: <8 across module
- **Tool**: cargo-clippy

```rust
// Good: Low complexity
pub fn simple_function(x: i32) -> i32 {
    x * 2
}

// Bad: High complexity (too many branches)
pub fn complex_function(x: i32) -> i32 {
    if x < 0 {
        if x < -10 {
            // ...
        } else {
            // ...
        }
    } else if x < 10 {
        // ...
    } else {
        // ...
    }
    // ... 10 more branches
}
```

#### 3. Documentation

- **All public APIs must be documented**
- **Include examples**
- **Document errors**

```rust
/// Get the parent process ID of the calling process
///
/// # Arguments
///
/// * `state` - Current kernel state
/// * `calling_pid` - Process ID making the syscall
///
/// # Returns
///
/// Returns parent PID, or 0 if process has no parent (init).
///
/// # Errors
///
/// * `ProcessNotFound` - Process doesn't exist
///
/// # Examples
///
/// ```
/// let (new_state, output) = sys_getppid(state, child_pid)?;
/// match output {
///     SyscallOutput::Pid(ppid) => println!("Parent: {}", ppid),
///     _ => unreachable!(),
/// }
/// ```
pub fn sys_getppid(
    state: KernelState,
    calling_pid: ProcessId,
) -> Result<(KernelState, SyscallOutput), KernelError> {
    // ...
}
```

#### 4. No Technical Debt

- **No TODO comments** - Create issues instead
- **No FIXME comments** - Fix or create issue
- **No HACK comments** - Refactor properly
- **No unwrap()** - Use proper error handling

```rust
// Bad
let process = state.processes.get(&pid).unwrap();  // FIXME: handle error

// Good
let process = state.processes.get(&pid)
    .ok_or(KernelError::ProcessNotFound)?;
```

#### 5. Zero Unsafe Code

- **Forbidden**: `#![forbid(unsafe_code)]` enforced
- **No exceptions** - Pure safe Rust only

### Testing Requirements

#### Minimum Tests per Feature

- **5+ unit tests** - Cover success and error cases
- **2+ property tests** - Verify invariants
- **1+ integration test** - End-to-end workflow

#### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests
    #[test]
    fn test_success_case() {
        // Test happy path
    }

    #[test]
    fn test_error_case() {
        // Test error handling
    }

    #[test]
    fn test_edge_case() {
        // Test boundary conditions
    }

    #[test]
    fn test_immutability() {
        // Verify no state mutation
    }

    #[test]
    fn test_idempotence() {
        // Verify same input → same output
    }

    // Property tests
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_invariant_holds(input in 0u32..1000) {
            // Test invariant across many inputs
        }
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs

#[test]
fn test_full_workflow() {
    // 1. Setup
    let state = KernelState::new();

    // 2. Execute workflow
    let (state, _) = sys_fork(state, 1).unwrap();
    let (state, _) = sys_getppid(state, 2).unwrap();

    // 3. Verify final state
    assert_eq!(state.processes.len(), 2);
}
```

## Documentation

### What to Document

1. **All public APIs** - Functions, structs, enums
2. **Architecture decisions** - Why, not just what
3. **Tutorials** - How to use features
4. **Examples** - Working code samples

### Documentation Locations

```
docs/
├── API.md              # API reference
├── ARCHITECTURE.md     # System design
├── PERFORMANCE.md      # Performance guide
├── CONTRIBUTING.md     # This file
├── tutorials/          # Step-by-step guides
└── specifications/     # Formal specs
```

### Updating Documentation

When adding features:

1. **Update API.md** - Add function documentation
2. **Update ARCHITECTURE.md** - If design changes
3. **Add tutorial** - For major features
4. **Update README.md** - If user-facing

## Pull Request Process

### Before Submitting

- [ ] All tests pass: `make test`
- [ ] Quality gates pass: `make quality`
- [ ] Coverage ≥85%: `make coverage-check`
- [ ] Documentation updated
- [ ] Commit messages follow format

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added (5+)
- [ ] Property tests added (2+)
- [ ] Integration tests added (1+)
- [ ] All tests passing

## Quality
- [ ] make quality passes
- [ ] Coverage ≥85%
- [ ] Documentation updated

## Related Issues
Closes #123
```

### Review Process

1. **Automated checks** - CI runs tests, coverage, quality
2. **Code review** - Maintainer reviews code
3. **Feedback** - Address review comments
4. **Approval** - Maintainer approves
5. **Merge** - Merged to main

### Review Criteria

Reviewers check:

- [ ] Code follows project style
- [ ] Tests are comprehensive
- [ ] Documentation is clear
- [ ] No performance regressions
- [ ] Quality standards met

## Project Structure

### Crate Organization

```
wos/
├── shared/           # Types used by multiple crates
│   ├── src/
│   │   ├── lib.rs
│   │   └── types.rs
│   └── Cargo.toml
│
├── kernel/           # Core kernel implementation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── process.rs
│   │   ├── memory.rs
│   │   ├── syscall.rs
│   │   └── scheduler.rs
│   ├── tests/        # Integration tests
│   └── Cargo.toml
│
├── userspace/        # Userspace programs
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ps.rs
│   │   └── tree.rs
│   └── Cargo.toml
│
└── wos/              # WASM bindings
    ├── src/
    │   ├── lib.rs
    │   └── quality.rs
    └── Cargo.toml
```

### File Naming Conventions

- **Modules**: `snake_case.rs`
- **Tests**: `test_*.rs` or `*_test.rs`
- **Binaries**: `bin/*.rs`
- **Documentation**: `UPPERCASE.md`

## Common Tasks

### Adding a Syscall

See [Tutorial 1: Adding a Syscall](tutorials/01-adding-syscall.md)

### Creating a Program

See [Tutorial 2: Creating a Program](tutorials/02-creating-program.md)

### Updating Quality Metrics

```rust
// wos/src/quality.rs

pub fn new() -> Self {
    QualityMetrics {
        test_count: 270,        // Update after adding tests
        coverage: 88.5,         // From coverage report
        max_complexity: 12,     // From clippy
        avg_complexity: 7.2,    // From clippy
        satd_count: 0,          // Grep for TODO/FIXME
        unsafe_count: 0,        // Should always be 0
        clippy_warnings: 0,     // From clippy output
        mutation_score: 92.0,   // From cargo-mutants
        tdg_score: 0.0,         // Calculated
        tdg_grade: String::new(), // Calculated
    }
}
```

### Running Benchmarks

```bash
# Install criterion
cargo install criterion

# Run benchmarks
cargo bench

# Specific benchmark
cargo bench --bench syscalls

# With baseline
cargo bench -- --save-baseline main
```

### Profiling Performance

```bash
# Install flamegraph
cargo install flamegraph

# Profile tests
cargo flamegraph --test syscall_tests

# Profile benchmarks
cargo flamegraph --bench syscalls

# View flamegraph.svg in browser
```

## Getting Help

### Resources

- **Documentation**: [docs/](.)
- **Issues**: GitHub Issues (check repository)
- **Discussions**: GitHub Discussions (check repository)

### Ask Questions

Don't hesitate to ask if you're unsure:

- Open a discussion for general questions
- Comment on issues for specific tasks
- Tag maintainers for urgent matters

## Recognition

Contributors are recognized in:

- **README.md** - Contributors section
- **Commit history** - Co-authored-by tags
- **Release notes** - Feature credits

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Thank You!

Every contribution, no matter how small, helps improve WOS. Thank you for being part of the project!

---

**Questions?** Open a discussion (check repository) or reach out to maintainers.
