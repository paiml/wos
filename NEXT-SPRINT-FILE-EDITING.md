# Next Sprint Priority: File Editing Workflow (CRITICAL)

**Date**: 2025-10-25
**Reporter**: User feedback
**Priority**: CRITICAL
**Status**: Documented, ready for next sprint

---

## Executive Summary

**CRITICAL GAP IDENTIFIED**: File editing workflow is incomplete and untested.

**User Issue**:
```bash
$ touch foo.txt
$ # Edit button remains disabled ❌
$ # File doesn't appear in file list ❌
$ # Cannot edit the file ❌
```

**Impact**: HIGH - Blocks basic file operations, core OS feature unusable

---

## Current State Analysis

### What Works ✅
- Vim editor UI exists and renders
- Monaco editor loads via CDN
- File manager UI component exists
- Edit button exists in UI
- WASM filesystem syscalls work (touch, cat, echo >)

### What's Broken ❌
1. **File list doesn't auto-refresh** after `touch` command
2. **Edit button remains disabled** even when files exist
3. **Zero E2E tests** for file editing workflow
4. **No integration between** file creation → file list → edit button → editor
5. **Vim save/load** untested (no E2E coverage)
6. **Monaco editor** untested (no E2E coverage)

---

## Code Locations

**File Manager**: `dist/wos/app.js:685-908`
- `selectFile()` - enables edit button (line 833)
- `deselectFile()` - disables edit button (line 906)
- `openVimEditor()` - opens Vim modal (line 866)
- **Missing**: `refreshFileList()` auto-refresh after file operations

**Edit Button**: `dist/wos/index.html:253`
```html
<button id="btn-edit" class="action-btn" disabled>
```
Always starts disabled, only enabled when file selected manually.

**Vim Editor**: `dist/wos/app.js:917-1241`
- Full Vim modal implementation exists
- Insert/normal mode switching works
- `:wq`, `:w`, `:q!` commands implemented
- **Missing**: E2E tests verifying save/load cycle

**Monaco Editor**: `dist/wos/app.js:122-250`
- CDN loading works
- `edit <filename>` command exists
- Ctrl+S save, Escape close
- **Missing**: E2E tests verifying full workflow

---

## Root Cause

**File list doesn't update after file creation**:
```javascript
// When user runs: touch foo.txt
// 1. WASM filesystem creates file ✅
// 2. File list UI updates? ❌ (no refresh mechanism)
// 3. Edit button enables? ❌ (file not in UI, can't be selected)
```

**Missing Component**: File system change observer
- No polling mechanism
- No event-based updates
- Manual refresh only (reload page)

---

## Solution: Extreme TDD Approach

### Phase 1: Auto-Refresh File List (4 hours)
**Tests First**:
```javascript
// tests/e2e/file-creation-refresh-test.spec.js
test('file list auto-refreshes after touch command', async ({ page }) => {
  await executeCommand(page, 'touch newfile.txt');
  await page.waitForTimeout(600);

  const fileItem = page.locator('.file-item:has-text("newfile.txt")');
  await expect(fileItem).toBeVisible();

  const editBtn = page.locator('#btn-edit');
  await expect(editBtn).toBeEnabled();
});
```

**Implementation**:
```javascript
class FileManager {
  observeFileSystemChanges() {
    setInterval(() => this.refreshFileList(), 500);
  }

  refreshFileList() {
    const files = this.getFilesFromWasm();
    if (filesChanged(files)) {
      this.updateFileList(files);
      if (newFileCreated) this.selectFile(files[0]);
    }
  }
}
```

### Phase 2: Vim Editor E2E Tests (4 hours)
**Tests First**:
```javascript
test('vim editor saves and loads files correctly', async ({ page }) => {
  await executeCommand(page, 'touch readme.md');
  await page.locator('#btn-edit').click();

  // Enter insert mode, type content
  await page.keyboard.press('i');
  await page.keyboard.type('# Hello World');

  // Save and quit
  await page.keyboard.press('Escape');
  await page.keyboard.type(':wq');
  await page.keyboard.press('Enter');

  // Reopen and verify content persisted
  await page.locator('#btn-edit').click();
  const content = await page.locator('#vim-editor').textContent();
  expect(content).toContain('# Hello World');
});
```

### Phase 3: Monaco Editor E2E Tests (4 hours)
**Tests First**:
```javascript
test('monaco editor creates and edits files', async ({ page }) => {
  await executeCommand(page, 'edit script.js');
  await page.waitForSelector('#monaco-editor-container');

  await page.keyboard.type('function hello() {}');
  await page.keyboard.press('Control+S');
  await page.keyboard.press('Escape');

  await executeCommand(page, 'cat script.js');
  const output = await page.locator('#terminal-output').textContent();
  expect(output).toContain('function hello()');
});
```

### Phase 4: File Selection UI Tests (2 hours)
**Tests First**:
```javascript
test('file selection enables edit button', async ({ page }) => {
  await executeCommand(page, 'touch file1.txt file2.txt');

  await page.locator('.file-item:has-text("file1.txt")').click();
  await expect(page.locator('#btn-edit')).toBeEnabled();

  await page.locator('.file-item:has-text("file2.txt")').click();
  await expect(page.locator('#btn-edit')).toBeEnabled();

  await page.locator('#file-list-container').click(); // Click empty space
  await expect(page.locator('#btn-edit')).toBeDisabled();
});
```

---

## Test Coverage Requirements

### Before (Current State)
- **E2E Tests**: 0 tests for file editing ❌
- **Coverage**: 0% for file editing workflow ❌
- **Mutation Score**: N/A (untested code) ❌

### After (Target State)
- **E2E Tests**: 20+ tests across 4 test files ✅
- **Coverage**: 100% for file editing workflow ✅
- **Mutation Score**: 90%+ for new code ✅

### Test Files to Create
1. `tests/e2e/file-creation-refresh-test.spec.js` - 5 tests
2. `tests/e2e/vim-editor-full-workflow.spec.js` - 8 tests
3. `tests/e2e/monaco-editor-workflow.spec.js` - 5 tests
4. `tests/e2e/file-manager-selection.spec.js` - 4 tests

**Total**: 22 new E2E tests

---

## Time Estimate

- **Phase 1** (Auto-refresh): 4 hours
- **Phase 2** (Vim E2E): 4 hours
- **Phase 3** (Monaco E2E): 4 hours
- **Phase 4** (Selection UI): 2 hours
- **Bug Fixes**: 2 hours
- **Total**: **16 hours** (2 days at 100% focus)

---

## Definition of Done

- [ ] `touch foo.txt` creates file visible in UI within 600ms
- [ ] Edit button enables automatically when file created
- [ ] Click edit button opens Vim with file content
- [ ] Vim `:w` saves content, verified by `cat` command
- [ ] Vim `:wq` saves and closes, content persists on reopen
- [ ] Vim `:q!` discards changes (tested)
- [ ] Monaco `edit file.js` opens editor
- [ ] Monaco Ctrl+S saves, Escape closes
- [ ] Monaco syntax highlighting works (tested)
- [ ] File selection UI highlights correctly
- [ ] All 22 E2E tests passing
- [ ] Coverage ≥85% for new code
- [ ] Mutation score ≥90% for new code
- [ ] Zero clippy warnings
- [ ] Documentation updated

---

## Success Criteria

**User Workflow** (End-to-End):
```bash
$ touch config.json
# File appears in file list ✅
# Edit button becomes enabled ✅

$ # Click edit button
# Vim opens with empty file ✅

$ # Type content in Vim
$ # Save with :wq
# Content persists ✅

$ cat config.json
# Shows the content I typed ✅

$ vim config.json
# Opens with my content ✅
```

**Metrics**:
- User satisfaction: From "unusable" → "works perfectly"
- Test coverage: From 0% → 100%
- Workflow completion time: <10 seconds for full cycle

---

## Next Steps

1. **Review this ticket**: `docs/tickets/WOS-FILE-EDIT-01-complete-file-editing-workflow.md`
2. **Prioritize for next sprint**: Mark as CRITICAL
3. **Assign developer**: Someone with Playwright + JavaScript expertise
4. **RED phase**: Write all 22 E2E tests first (4 hours)
5. **GREEN phase**: Implement until tests pass (8 hours)
6. **REFACTOR phase**: Optimize, clean up (2 hours)
7. **VERIFY**: Run full test suite, deploy to production

---

## References

- **Detailed Ticket**: `docs/tickets/WOS-FILE-EDIT-01-complete-file-editing-workflow.md`
- **File Manager Code**: `dist/wos/app.js:685-908`
- **Vim Editor Code**: `dist/wos/app.js:917-1241`
- **Monaco Editor Code**: `dist/wos/app.js:122-250`
- **Existing Vim Tests**: `tests/e2e/vim-error-messages-test.spec.js`, `vim-help-test.spec.js`

---

**Created**: 2025-10-25
**Reported By**: User feedback during deployment verification
**Priority**: CRITICAL (blocks core functionality)
**Status**: Documented and ready for next sprint
