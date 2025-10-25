# WOS-FILE-EDIT-01: Complete File Editing Workflow with Extreme TDD

**Priority**: CRITICAL
**Status**: pending
**Sprint**: Next (post-perfection)
**Complexity**: HIGH
**Time Estimate**: 12-16 hours
**Created**: 2025-10-25
**Reporter**: User feedback - file editing disabled even after `touch foo.txt`

---

## Problem Statement

The file editing workflow is **incomplete and untested**:

1. ❌ **Edit button remains disabled** even after creating files with `touch foo.txt`
2. ❌ **Zero E2E tests** for the file editing workflow
3. ❌ **No integration tests** for Monaco editor
4. ❌ **No tests for Vim editor file operations** (save/load)
5. ⚠️ **Button enable/disable logic broken** - doesn't detect created files

**Current State**:
- `dist/wos/index.html:253` - Edit button starts disabled: `<button id="btn-edit" class="action-btn" disabled>`
- `dist/wos/app.js:833` - Button enabled when file selected: `document.getElementById('btn-edit').disabled = false;`
- `dist/wos/app.js:906` - Button disabled when deselected: `document.getElementById('btn-edit').disabled = true;`

**Bug**: File selection logic doesn't trigger when creating new files via `touch` command.

---

## Root Cause Analysis

### Code Locations

**File Manager** (`dist/wos/app.js:685-908`):
- `selectFile(fileName)` - sets `this.selectedFile` and enables edit button (line 833)
- `deselectFile()` - clears selection and disables edit button (line 906)
- `openVimEditor(fileName)` - opens Vim modal (line 866)

**Button Event Handler** (`dist/wos/app.js:691-695`):
```javascript
document.getElementById('btn-edit').addEventListener('click', () => {
  if (this.selectedFile) {
    this.openVimEditor(this.selectedFile);
  }
});
```

**File List Population** (`dist/wos/app.js:798-839`):
- Reads files from WASM filesystem
- Creates file list items
- Sets up click handlers for selection
- **Missing**: Auto-refresh after `touch` command

**Issue**: When user runs `touch foo.txt`, the file is created in the WASM filesystem but:
1. File manager doesn't refresh file list
2. New file doesn't appear in UI
3. Edit button remains disabled

---

## Acceptance Criteria

### Functional Requirements

1. ✅ **File Creation Workflow**:
   ```bash
   $ touch newfile.txt
   $ # File appears in file list automatically
   $ # File is auto-selected
   $ # Edit button becomes enabled
   ```

2. ✅ **Edit Button Behavior**:
   - Disabled when no file selected
   - Enabled when file selected (existing or new)
   - Opens Vim editor on click

3. ✅ **Vim Editor**:
   - Opens with file content (or empty for new files)
   - Saves changes back to filesystem on `:w` or `:wq`
   - Refreshes file list after save
   - Updates file content display

4. ✅ **Monaco Editor** (bonus):
   - Opens with `edit filename.txt` command
   - Syntax highlighting based on file extension
   - Saves on Ctrl+S or Escape
   - Refreshes file list after save

### Testing Requirements (Extreme TDD)

#### Unit Tests (Rust - shared/kernel crates)
- [ ] Test file creation syscall updates filesystem
- [ ] Test file write syscall updates content
- [ ] Test file read syscall returns correct content
- [ ] Property test: file operations maintain consistency

#### Integration Tests (JavaScript - app.js)
- [ ] Test `FileManager.refreshFileList()` reads from WASM
- [ ] Test `FileManager.selectFile()` enables edit button
- [ ] Test `FileManager.deselectFile()` disables edit button
- [ ] Test `VimEditor.save()` updates filesystem
- [ ] Test file list updates after file creation
- [ ] Test file list updates after file deletion

#### E2E Tests (Playwright)
- [ ] **test-file-creation-and-edit.spec.js**:
  - Create file with `touch foo.txt`
  - Verify file appears in file list
  - Verify edit button enabled
  - Click edit button
  - Verify Vim editor opens
  - Type content in Vim
  - Save with `:wq`
  - Verify file content persists
  - Verify file list shows file size updated

- [ ] **test-vim-editor-full-workflow.spec.js**:
  - Open existing file in Vim
  - Verify file content loads
  - Edit content (insert mode)
  - Save and quit (`:wq`)
  - Reopen file
  - Verify changes persisted
  - Test `:w` (save without quit)
  - Test `:q!` (quit without saving)
  - Verify discard works

- [ ] **test-monaco-editor-workflow.spec.js**:
  - Run `edit newfile.js`
  - Verify Monaco editor opens
  - Type JavaScript code
  - Verify syntax highlighting works
  - Save with Ctrl+S
  - Verify file created
  - Close editor (Escape)
  - Reopen file
  - Verify content persisted

- [ ] **test-file-manager-selection.spec.js**:
  - Create multiple files
  - Click file in list
  - Verify selection highlighting
  - Verify edit button enabled
  - Click another file
  - Verify selection changes
  - Verify edit button stays enabled
  - Click empty space
  - Verify selection clears
  - Verify edit button disabled

---

## Implementation Plan (RED-GREEN-REFACTOR)

### Phase 1: Fix File List Auto-Refresh (4 hours)

**RED** (Write Failing Tests):
```javascript
// tests/e2e/file-creation-refresh-test.spec.js
test('file list auto-refreshes after touch command', async ({ page }) => {
  // Execute touch command
  await page.locator('#terminal-input').fill('touch newfile.txt');
  await page.locator('#terminal-input').press('Enter');

  // Wait for file list refresh
  await page.waitForTimeout(500);

  // Verify file appears in list
  const fileItem = page.locator('.file-item:has-text("newfile.txt")');
  await expect(fileItem).toBeVisible();

  // Verify edit button enabled
  const editBtn = page.locator('#btn-edit');
  await expect(editBtn).toBeEnabled();
});
```

**GREEN** (Implement):
```javascript
// dist/wos/app.js - Add file creation observer
class FileManager {
  constructor(wos) {
    this.wos = wos;
    this.observeFileSystemChanges();
  }

  observeFileSystemChanges() {
    // Poll filesystem every 500ms for changes
    setInterval(() => {
      this.refreshFileList();
    }, 500);
  }

  refreshFileList() {
    const files = this.getFilesFromWasm();
    const currentFiles = this.getCurrentFileList();

    if (!this.arraysEqual(files, currentFiles)) {
      this.updateFileList(files);
    }
  }
}
```

**REFACTOR**: Optimize polling → use event-based updates via WASM callbacks

### Phase 2: Fix Edit Button Enable Logic (2 hours)

**RED** (Write Failing Tests):
```javascript
test('edit button enables after touch command', async ({ page }) => {
  await page.locator('#terminal-input').fill('touch test.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(600);

  const editBtn = page.locator('#btn-edit');
  await expect(editBtn).toBeEnabled();
});
```

**GREEN** (Implement):
```javascript
refreshFileList() {
  // ... existing code ...

  // Auto-select newly created file
  if (newFiles.length > 0) {
    this.selectFile(newFiles[0]);
  }
}
```

**REFACTOR**: Extract auto-selection logic, add configuration option

### Phase 3: Vim Editor Save/Load Integration (4 hours)

**RED** (Write Failing Tests):
```javascript
test('vim editor saves and loads files correctly', async ({ page }) => {
  // Create file
  await page.locator('#terminal-input').fill('touch readme.md');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(600);

  // Open in Vim
  await page.locator('#btn-edit').click();
  await page.waitForSelector('#vim-modal');

  // Enter insert mode
  await page.keyboard.press('i');

  // Type content
  await page.keyboard.type('# Hello World\n\nThis is a test.');

  // Exit insert mode
  await page.keyboard.press('Escape');

  // Save and quit
  await page.keyboard.type(':wq');
  await page.keyboard.press('Enter');

  // Verify modal closed
  await expect(page.locator('#vim-modal')).toBeHidden();

  // Reopen file
  await page.locator('#btn-edit').click();

  // Verify content loaded
  const content = await page.locator('#vim-editor').textContent();
  expect(content).toContain('# Hello World');
  expect(content).toContain('This is a test.');
});
```

**GREEN** (Implement):
```javascript
// VimEditor.save() - ensure WASM integration
save() {
  const content = this.lines.join('\n');

  // Write to WASM filesystem
  this.wos.executeCommand(`echo "${this.escapeContent(content)}" > ${this.fileName}`);

  // Trigger file manager refresh
  window.fileManager.refreshFileList();

  this.modified = false;
}
```

**REFACTOR**: Extract content escaping, add error handling

### Phase 4: Monaco Editor Integration (4-6 hours)

**RED** (Write Failing Tests):
```javascript
test('monaco editor creates and edits files', async ({ page }) => {
  await page.locator('#terminal-input').fill('edit script.js');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(1000); // Monaco load time

  // Verify Monaco container visible
  await expect(page.locator('#monaco-editor-container')).toBeVisible();

  // Type code
  await page.keyboard.type('function hello() {\n  return "world";\n}');

  // Save with Ctrl+S
  await page.keyboard.press('Control+S');
  await page.waitForTimeout(300);

  // Close with Escape
  await page.keyboard.press('Escape');

  // Verify file created
  await page.locator('#terminal-input').fill('cat script.js');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(300);

  const output = await page.locator('#terminal-output').textContent();
  expect(output).toContain('function hello()');
  expect(output).toContain('return "world"');
});
```

**GREEN** (Implement):
```javascript
// Add Ctrl+S save handler
monacoEditor.addCommand(window.monaco.KeyMod.CtrlCmd | window.monaco.KeyCode.KeyS, function() {
  closeMonacoEditor(true); // Save on Ctrl+S
});

// Update closeMonacoEditor to refresh file list
function closeMonacoEditor(save) {
  // ... existing save logic ...

  if (save) {
    window.fileManager.refreshFileList();
  }
}
```

**REFACTOR**: Extract Monaco configuration, add settings persistence

---

## Files to Modify

1. **dist/wos/app.js**:
   - Line 685-908: `FileManager` class - add auto-refresh
   - Line 866-880: `openVimEditor()` - add refresh callback
   - Line 917-1241: `VimEditor` class - fix save integration
   - Line 165-250: Monaco editor functions - add refresh callback

2. **dist/wos/index.html**:
   - Line 253: Remove `disabled` attribute from edit button if file exists
   - Add data attributes for better testing

3. **tests/e2e/** (NEW FILES):
   - `file-creation-refresh-test.spec.js`
   - `vim-editor-full-workflow.spec.js`
   - `monaco-editor-workflow.spec.js`
   - `file-manager-selection.spec.js`

---

## Definition of Done

- [ ] All E2E tests passing (4 new test files, ~20 tests total)
- [ ] All integration tests passing
- [ ] All unit tests passing
- [ ] `touch foo.txt` → file appears in list, edit button enabled
- [ ] Click edit button → Vim opens with file content
- [ ] Save in Vim → content persists, file list refreshes
- [ ] `edit file.js` → Monaco opens with syntax highlighting
- [ ] Save in Monaco → content persists, file list refreshes
- [ ] File selection UI works correctly
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] Coverage ≥85% for new code
- [ ] Mutation score ≥90% for new code
- [ ] Documentation updated (CLAUDE.md, README.md)

---

## Success Metrics

**Before** (Current State):
- Edit button: Always disabled ❌
- File creation: No UI feedback ❌
- Vim save: Untested ❌
- Monaco: Untested ❌
- E2E coverage: 0% ❌

**After** (Target State):
- Edit button: Enabled when file selected ✅
- File creation: Auto-refreshes, auto-selects ✅
- Vim save: Full workflow tested, 100% working ✅
- Monaco: Full workflow tested, 100% working ✅
- E2E coverage: 100% (20+ tests) ✅

---

## Dependencies

- Playwright installed and configured ✅
- Monaco Editor CDN loaded ✅
- Vim editor UI components exist ✅
- WASM filesystem syscalls working ✅

---

## Risks & Mitigation

**Risk 1**: Monaco editor load time affects test reliability
- **Mitigation**: Add proper wait conditions, increase timeouts for Monaco tests

**Risk 2**: File system polling could impact performance
- **Mitigation**: Use debouncing, implement event-based updates in Phase 2

**Risk 3**: Vim escape sequence handling could break
- **Mitigation**: Comprehensive unit tests for all Vim commands, property testing

---

## Related Tickets

- WOS-UX-01: Icon Toolbar (completed) - edit button exists
- WOS-402: Vim help command (completed) - Vim UI exists
- Phase 14 (NEW): File Editing Workflow

---

## Notes

This ticket is **CRITICAL** for MVP completeness. File editing is a core feature of a terminal/OS interface. Current state is unusable - users cannot edit files they create.

**User Impact**: HIGH - blocks basic file operations workflow
**Technical Debt**: HIGH - zero test coverage for critical feature
**Complexity**: MEDIUM-HIGH - requires E2E testing expertise

**Recommendation**: Prioritize this for next sprint. Use Extreme TDD methodology - write all tests first, then implement. Target 100% E2E coverage for file editing workflow.

---

**Created**: 2025-10-25
**Last Updated**: 2025-10-25
**Assignee**: TBD
**Labels**: `critical`, `file-editing`, `extreme-tdd`, `e2e-testing`, `vim`, `monaco`
