# Bug Report: Terminal State, cd Command, and Vim Subset Issues

**Date**: 2025-10-23
**Reporter**: Customer
**Status**: Open
**Priority**: High
**Components**: Terminal, Shell, Vim Editor

## Summary

Three related issues affecting terminal usability:
1. `cd` command not implemented - directory navigation not working
2. Terminal state not visible enough - hard to see current working directory and system state
3. Vim command subset incomplete - many commands show ":sp is not a command" error

## Bug Details

### 1. Missing `cd` Command
**Description**: The `cd` command is not implemented in the shell.

**Expected Behavior**:
- `cd /path/to/dir` should change current working directory
- `cd ..` should move to parent directory
- `cd ~` should move to home directory
- `cd` (no args) should move to home directory
- Shell prompt should reflect current directory

**Actual Behavior**:
- `cd` command not recognized or not working
- User cannot navigate directory structure

**Impact**: Cannot navigate filesystem, severely limits shell usability

### 2. Terminal State Visibility
**Description**: Terminal does not show enough real-time state information.

**Missing Information**:
- Current working directory (pwd/cwd)
- Current user
- Current process context
- System status indicators

**Expected Behavior**:
- Shell prompt should show: `user@wos:/current/path$ `
- Status bar or panel showing system state
- Visual feedback for state changes

**Actual Behavior**:
- User cannot see what is going on with system state
- No visual indicators of current context

**Impact**: Poor user experience, difficult to track where you are in the system

### 3. Vim Command Subset Incomplete
**Description**: Many standard Vim commands are not implemented, showing generic error `:sp is not a command`.

**Known Missing Commands**:
- `:sp` (split window horizontally)
- `:vs` (split window vertically)
- Likely many more...

**Expected Behavior**:
- Common Vim commands should either:
  - Work correctly, OR
  - Show helpful error: "Command not supported in WOS Vim. Available commands: ..."

**Actual Behavior**:
- Generic error message: ":sp is not a command"
- No guidance on what commands ARE available

**Impact**: User frustration, unclear what subset of Vim is implemented

## Reproduction Steps

### cd Bug
```bash
# In terminal
cd /bin
# Expected: Change to /bin directory
# Actual: Command not recognized or no effect
```

### Terminal State Bug
```bash
# In terminal
cd /tmp
# Expected: Prompt shows "user@wos:/tmp$ "
# Actual: Prompt unchanged, no indication of current directory
```

### Vim Subset Bug
```vim
# In Vim mode
:sp
# Expected: Split window OR helpful error
# Actual: ":sp is not a command"
```

## Root Cause Analysis (Preliminary)

### cd Command
- **Location**: `userspace/src/shell.rs` likely missing `cd` handler
- **Root Cause**: Shell command parser doesn't implement `cd` builtin
- **Fix Required**: Add `cd` command with:
  - Path resolution
  - `..` handling
  - `~` expansion
  - System call to kernel to update process CWD

### Terminal State
- **Location**: `dist/wos/app.js` terminal rendering
- **Root Cause**: Prompt doesn't query/display system state
- **Fix Required**:
  - Query kernel for current process state (PID, CWD, user)
  - Update prompt format to show state
  - Add status bar for system information

### Vim Subset
- **Location**: `dist/wos/app.js` vim mode handler
- **Root Cause**: Limited command set with poor error messages
- **Fix Required**:
  - Document supported Vim commands
  - Improve error messages with available command list
  - Consider implementing more common commands (split, etc.)

## Testing Strategy (Extreme TDD)

### Unit Tests Required
- [ ] `shell_cd_absolute_path()`
- [ ] `shell_cd_relative_path()`
- [ ] `shell_cd_parent_directory()`
- [ ] `shell_cd_home_directory()`
- [ ] `shell_cd_invalid_path_returns_error()`
- [ ] `process_cwd_updates_on_cd()`

### Property Tests Required
- [ ] `cd_followed_by_pwd_shows_new_directory()`
- [ ] `cd_path_resolution_is_deterministic()`
- [ ] `cd_invalid_paths_never_panic()`

### E2E Tests Required (Playwright)
- [ ] `test_cd_command_changes_directory()`
- [ ] `test_prompt_shows_current_directory()`
- [ ] `test_cd_updates_prompt_immediately()`
- [ ] `test_vim_error_messages_helpful()`
- [ ] `test_vim_unknown_command_lists_available_commands()`

### Mutation Tests Required
- [ ] Kill score ≥90% for shell cd handler
- [ ] Kill score ≥90% for prompt rendering

## Fix Plan

### Phase 1: cd Command (WOS-XXX)
1. Write RED tests for cd command
2. Implement cd in shell.rs
3. Add Chdir system call to kernel
4. Update process CWD state
5. Verify all tests pass
6. Run mutation tests

### Phase 2: Terminal State Visibility (WOS-XXX)
1. Write E2E tests for prompt display
2. Add kernel query for process state
3. Update prompt format: `user@wos:/path$ `
4. Add optional status bar
5. Verify real-time updates

### Phase 3: Vim Error Messages (WOS-XXX)
1. Document currently supported Vim commands
2. Write E2E tests for error messages
3. Improve error message format
4. Add command suggestions
5. Consider implementing additional commands

## Success Criteria

- [ ] `cd` command works for absolute/relative/parent/home paths
- [ ] Shell prompt shows `user@wos:/current/path$ ` format
- [ ] Prompt updates immediately on directory change
- [ ] Vim error messages list available commands
- [ ] All unit tests pass (85%+ coverage)
- [ ] All property tests pass (10K inputs)
- [ ] All E2E tests pass
- [ ] Mutation score ≥90%
- [ ] Zero PMAT violations

## References

- POSIX cd specification: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cd.html
- Shell builtins implementation patterns
- Vim command reference for subset selection

## Notes

- This is a HIGH priority bug affecting core usability
- Requires extreme TDD - no shortcuts
- Must pass ALL quality gates before merge
- Consider adding to roadmap as urgent tickets
