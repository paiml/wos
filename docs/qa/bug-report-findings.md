# Bug Report Investigation Findings

**Date**: 2025-10-23
**Investigation**: Bug Report cd-terminal-state-vim

## Executive Summary

Investigation reveals that **cd command IS implemented** in Rust shell (userspace/src/shell.rs:102-147), but the JavaScript terminal UI has a **hardcoded prompt** that doesn't reflect the current working directory. The vim implementation also has a limited command set with poor error messages.

## Findings

### Issue 1: cd Command - PARTIALLY WORKING ✓/✗

**Status**: Command logic works, but UI doesn't reflect state

**Evidence**:
- `userspace/src/shell.rs:102-147` - cd builtin fully implemented
- Handles: absolute paths, relative paths, `cd` (home), `cd ..`
- Tests passing: `test_shell_builtin_cd()`, `test_shell_builtin_cd_relative()`, `test_shell_builtin_cd_home()`
- Shell state tracks `cwd` correctly (shell.rs:34-36)

**Root Cause**:
- JavaScript terminal has **hardcoded prompt**: `wos$` (app.js:1635)
- No query to WASM for current process state
- Prompt should be: `user@wos:/current/path$`

**Fix Required**:
1. Add WASM export to get current shell state (PID, CWD, user)
2. Update `printCommand()` to query state dynamically
3. Update prompt format to show CWD

### Issue 2: Terminal State Visibility - NOT IMPLEMENTED ✗

**Status**: Prompt is static, no dynamic state

**Evidence**:
- `app.js:1635` - Hardcoded prompt: `this.printLine(\`wos$ ${cmd}\`, 'command');`
- No mechanism to query kernel for process state
- No user@host prefix
- No CWD display

**Root Cause**:
- Terminal doesn't integrate with kernel state
- No WASM exports for getCurrentWorkingDirectory(), getCurrentUser(), etc.

**Fix Required**:
1. Add WASM exports:
   - `getCurrentWorkingDirectory() -> String`
   - `getCurrentUser() -> String`
   - `getCurrentProcessId() -> ProcessId`
2. Update Terminal class to query state before each prompt
3. Format: `user@wos:/current/path$`

### Issue 3: Vim Command Subset - LIMITED IMPLEMENTATION ✗

**Status**: Only 6 commands implemented, poor error messages

**Evidence**:
- `app.js:1131-1151` - VimEditor.executeCommand()
- **Implemented commands**: `:w`, `:write`, `:q`, `:quit`, `:q!`, `:quit!`, `:wq`, `:x`
- **Missing commands**: `:sp`, `:vs`, `:e`, `:help`, `:set`, etc.
- Generic error: `E492: Not an editor command: ${cmd}` (app.js:1149)

**Root Cause**:
- Intentional minimal implementation (educational OS)
- No documentation of supported commands
- Error message doesn't list available commands

**Fix Required**:
1. Add helpful error message with available commands
2. Consider implementing common commands:
   - `:help` - Show available commands
   - `:e <file>` - Edit file (switch to Monaco editor)
3. Update help to list vim capabilities

## Code Locations

### Shell Implementation (Rust)
- `userspace/src/shell.rs:102-147` - cd builtin
- `userspace/src/shell.rs:34-36` - Shell.cwd field
- `userspace/src/shell.rs:253-280` - cd tests (ALL PASSING ✓)

### Terminal UI (JavaScript)
- `dist/wos/app.js:1635` - printCommand() with hardcoded prompt
- `dist/wos/app.js:1524` - Terminal class
- `dist/wos/app.js:1837` - executeCommand()

### Vim Implementation (JavaScript)
- `dist/wos/app.js:917-1200` - VimEditor class
- `dist/wos/app.js:1131-1151` - executeCommand()
- `dist/wos/app.js:1149` - Error message

## Test Coverage

### Existing Tests (Passing)
- ✓ `test_shell_builtin_cd()` - Absolute path
- ✓ `test_shell_builtin_cd_relative()` - Relative path
- ✓ `test_shell_builtin_cd_home()` - Home directory
- ✓ `test_shell_main_loop_builtin()` - Integration

### Missing Tests (Need to Add)
- ✗ E2E test for prompt showing CWD
- ✗ E2E test for cd updating prompt
- ✗ E2E test for vim error messages
- ✗ Property test for cd path resolution

## Implementation Plan

### Phase 1: Dynamic Prompt (HIGH PRIORITY)
**Ticket**: WOS-400 (new)

1. **RED Tests** (E2E with Playwright):
```javascript
test('prompt shows current directory', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000/');
  await page.waitForSelector('.terminal-line.command');

  const prompt = await page.locator('.terminal-line.command').last();
  await expect(prompt).toContainText('wos@/$ ls');
});

test('cd command updates prompt', async ({ page }) => {
  await executeCommand(page, 'cd /home');
  await executeCommand(page, 'pwd');

  const prompt = await page.locator('.terminal-line.command').last();
  await expect(prompt).toContainText('wos@/home$');
});
```

2. **GREEN Implementation**:
   - Add WASM exports in `wos/src/lib.rs`:
     ```rust
     #[wasm_bindgen]
     pub fn get_current_working_directory(&self) -> String

     #[wasm_bindgen]
     pub fn get_current_user(&self) -> String
     ```

   - Update Terminal.printCommand() in `dist/wos/app.js`:
     ```javascript
     printCommand(cmd) {
       const cwd = this.wos.get_current_working_directory();
       const user = this.wos.get_current_user();
       this.printLine(`${user}@wos:${cwd}$ ${cmd}`, 'command');
     }
     ```

3. **REFACTOR**: Add caching if performance issues

### Phase 2: Vim Error Messages (MEDIUM PRIORITY)
**Ticket**: WOS-401 (new)

1. **RED Tests** (E2E):
```javascript
test('vim shows helpful error for unknown command', async ({ page }) => {
  await openVim(page, 'test.txt');
  await page.keyboard.press(':');
  await page.keyboard.type('sp');
  await page.keyboard.press('Enter');

  const message = await page.locator('.vim-message');
  await expect(message).toContainText('Available commands:');
  await expect(message).toContainText(':w, :q, :wq');
});
```

2. **GREEN Implementation**:
   - Update VimEditor.executeCommand() in `dist/wos/app.js:1131-1151`:
     ```javascript
     const AVAILABLE_COMMANDS = [':w', ':q', ':q!', ':wq', ':x', ':write', ':quit'];

     executeCommand(cmd) {
       // ... existing logic ...
       else {
         this.message = `Command not available. Available commands: ${AVAILABLE_COMMANDS.join(', ')}`;
       }
     }
     ```

3. **REFACTOR**: Add :help command

### Phase 3: Additional Vim Commands (LOW PRIORITY)
**Ticket**: WOS-402 (new)

Consider implementing:
- `:help` - Show available commands
- `:e <file>` - Edit file (delegate to Monaco editor)
- Better error messages with examples

## Quality Requirements

All fixes must meet:
- ✓ E2E tests pass (Playwright)
- ✓ Unit tests pass (if applicable)
- ✓ Property tests pass (if applicable)
- ✓ Mutation tests ≥90% kill rate
- ✓ Zero PMAT violations
- ✓ Code coverage ≥85%

## Success Criteria

- [ ] Terminal prompt shows: `user@wos:/current/path$`
- [ ] `cd` command updates prompt immediately
- [ ] `pwd` command output matches prompt CWD
- [ ] Vim error messages list available commands
- [ ] All E2E tests pass
- [ ] All quality gates pass
- [ ] Customer can navigate directories and see state

## Next Steps

1. **NOW**: Create roadmap tickets (WOS-400, WOS-401, WOS-402)
2. **NOW**: Write RED E2E tests for dynamic prompt
3. **NOW**: Implement WASM exports and UI updates
4. **NOW**: Verify all quality gates pass
5. **NOW**: Deploy to production

## References

- Customer bug report: `docs/qa/bug-report-cd-terminal-state-vim.md`
- Shell implementation: `userspace/src/shell.rs`
- Terminal UI: `dist/wos/app.js`
- E2E tests: `tests/e2e/`
