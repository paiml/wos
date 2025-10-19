# UI Feature Implementation Plan

**Goal**: Implement missing UI features to achieve 100% E2E test coverage (1,055/1,055 tests)
**Current Status**: 830/1,055 passing (78.67%)
**Target**: 1,055/1,055 passing (100%)
**Estimated Effort**: 2-3 weeks
**Impact**: +225 passing tests

## Executive Summary

The WOS project has completed all Rust kernel development and has 100% unit test coverage. The remaining E2E test failures (225 tests) are **not code issues** - they require UI feature implementation that was deferred during MVP development.

### Test Dependencies

All 225 failing tests fall into 4 categories:

1. **Vim Editor Modal** (70 tests) - Core UI missing
2. **Config Management** (50 tests) - Shell commands need implementation
3. **Panel Management** (70 tests) - Partial implementation exists
4. **Shell Scripts UI** (35 tests) - Depends on Vim modal

## Current State Analysis

### ✅ What's Already Implemented

**HTML Structure** (dist/wos/index.html):
- ✅ Vim modal DOM structure (lines 252-272)
- ✅ Panel collapse/expand buttons
- ✅ All panel containers with `data-panel` attributes
- ✅ Config Manager infrastructure

**JavaScript** (dist/wos/app.js):
- ✅ ConfigManager class (theme application logic)
- ✅ PanelManager class (collapse/expand functionality)
- ✅ Theme application (`theme-dark`, `theme-light` classes)
- ✅ localStorage persistence

### ❌ What's Missing

**1. Vim Editor Functionality**
- ❌ VimEditor class (complete modal controller)
- ❌ Keyboard event handlers (i, Escape, :, etc.)
- ❌ Mode state machine (NORMAL → INSERT → COMMAND)
- ❌ Text buffer management
- ❌ Command execution (:w, :q, :wq, :q!)
- ❌ File save/load integration with WOS kernel
- ❌ Terminal integration (`vim filename` command)

**2. Config & Theme Shell Commands**
- ❌ `config` command (display configuration)
- ❌ `theme dark` command
- ❌ `theme light` command
- ❌ `theme auto` command
- ❌ Config persistence to localStorage
- ❌ Help text updates

**3. Panel Management Enhancements**
- ⚠️ Collapse/expand partially implemented
- ❌ Icon rotation on collapse (CSS transform)
- ❌ State persistence across operations

## Implementation Plan

### Phase 1: Vim Editor Modal (Week 1)

**Priority**: HIGHEST (blocks 105 tests - Vim 70 + Shell Scripts 35)

#### 1.1: VimEditor Class Foundation
**File**: `dist/wos/app.js`
**Lines to Add**: ~300

```javascript
class VimEditor {
  constructor(wos, terminal, fileManager) {
    this.wos = wos;
    this.terminal = terminal;
    this.fileManager = fileManager;

    // State
    this.mode = 'NORMAL';          // NORMAL, INSERT, COMMAND
    this.buffer = [];              // Array of lines
    this.cursor = { row: 0, col: 0 };
    this.filename = '';
    this.modified = false;
    this.commandBuffer = '';

    // DOM elements
    this.modal = document.getElementById('vim-modal');
    this.editor = document.getElementById('vim-editor');
    this.modeDisplay = document.getElementById('vim-mode');
    this.filenameDisplay = document.getElementById('vim-filename');
    this.modifiedIndicator = document.getElementById('vim-modified');
    this.positionDisplay = document.getElementById('vim-position');
    this.commandDisplay = document.getElementById('vim-command');
    this.messageDisplay = document.getElementById('vim-message');
    this.closeButton = document.getElementById('vim-close');

    this.setupEventListeners();
  }

  open(filename) {
    this.filename = filename;
    this.loadFile();
    this.showModal();
    this.setMode('NORMAL');
    this.render();
    this.editor.focus();
  }

  loadFile() {
    // Load file from WOS VFS
    const content = this.fileManager.readFile(this.filename);
    if (content) {
      this.buffer = content.split('\n');
      this.modified = false;
    } else {
      this.buffer = [''];
      this.modified = false;
    }
  }

  saveFile() {
    const content = this.buffer.join('\n');
    this.fileManager.writeFile(this.filename, content);
    this.modified = false;
    this.showMessage(`"${this.filename}" written`);
    this.updateModifiedIndicator();
  }

  // ...mode handlers, keyboard handlers, render logic...
}
```

**Test Coverage**: 70 tests in `07-vim-editor.spec.ts`

#### 1.2: Keyboard Event Handling
**Complexity**: HIGH
**Key Behaviors**:
- NORMAL mode: `i` → INSERT, `:` → COMMAND, arrow keys navigation
- INSERT mode: text input, Escape → NORMAL, Backspace
- COMMAND mode: command buffer, Enter executes, Escape cancels

#### 1.3: Terminal Integration
**File**: Terminal class in `app.js`
**Change**: Add `vim` command handler

```javascript
// In Terminal.executeCommand()
if (command === 'vim' && args.length > 0) {
  const filename = args[0];
  this.vimEditor.open(filename);
  return;
}
```

**Test Coverage**: Integration with `13-shell-scripts.spec.ts` (35 tests)

### Phase 2: Config Management Commands (Week 2)

**Priority**: HIGH (blocks 50 tests)

#### 2.1: Config Command
**File**: Terminal class
**Lines to Add**: ~50

```javascript
if (command === 'config') {
  const config = this.configManager.getConfig();
  this.writeOutput('Current Configuration:');
  this.writeOutput(`  Version: ${config.version}`);
  this.writeOutput(`  Environment: ${config.environment}`);
  this.writeOutput('UI Settings:');
  this.writeOutput(`  Theme: ${config.ui.theme}`);
  this.writeOutput(`  Mode: ${config.ui.mode}`);
  // ... more config details
  return;
}
```

#### 2.2: Theme Commands
**File**: Terminal class, ConfigManager
**Lines to Add**: ~100

```javascript
if (command === 'theme' && args.length > 0) {
  const theme = args[0]; // 'dark', 'light', 'auto'

  if (!['dark', 'light', 'auto'].includes(theme)) {
    this.writeOutput(`Error: Invalid theme '${theme}'`);
    this.writeOutput('Valid themes: dark, light, auto');
    return;
  }

  // Update config
  const config = this.configManager.getConfig();
  config.ui.theme = theme;

  // Save to localStorage
  const yamlConfig = JSON.stringify(config);
  this.configManager.saveConfig(yamlConfig);

  // Apply theme
  this.configManager.applyConfig();

  this.writeOutput(`Theme set to: ${theme}`);
  return;
}
```

**Test Coverage**: 50 tests in `08-config-management.spec.ts`

### Phase 3: Panel Management Polish (Week 2-3)

**Priority**: MEDIUM (blocks 70 tests)

#### 3.1: Icon Rotation CSS
**File**: `dist/wos/style.css`
**Lines to Add**: ~20

```css
.btn-collapse svg {
  transition: transform 0.3s ease;
}

.collapsed .btn-collapse svg {
  transform: rotate(180deg);
}
```

#### 3.2: Panel State Verification
**File**: PanelManager class
**Enhancement**: Ensure state persists across operations

**Test Coverage**: 70 tests in `09-panel-management.spec.ts`

### Phase 4: Integration & Testing (Week 3)

#### 4.1: Help Command Updates
Add documentation for all new commands:
- `vim <filename>` - Open file in Vim editor
- `config` - Display current configuration
- `theme <dark|light|auto>` - Set color theme

#### 4.2: E2E Test Runs
Run full test suite across all 5 browsers:
- Chromium
- Firefox
- WebKit
- Mobile Chrome
- Mobile Safari

**Target**: 1,055/1,055 passing (100%)

## Implementation Guidelines

### Code Quality Requirements

1. **No Breaking Changes**: All existing 830 passing tests must continue to pass
2. **Extreme TDD**: Write E2E test, run to verify failure, implement, verify pass
3. **Browser Compatibility**: Test on all 5 Playwright browsers
4. **Documentation**: Update help command and CLAUDE.md

### Testing Strategy

**Incremental Testing**:
```bash
# Test Vim editor only
cd e2e && npx playwright test tests/07-vim-editor.spec.ts --project=chromium

# Test config management only
cd e2e && npx playwright test tests/08-config-management.spec.ts --project=chromium

# Test panel management only
cd e2e && npx playwright test tests/09-panel-management.spec.ts --project=chromium

# Test shell scripts (depends on Vim)
cd e2e && npx playwright test tests/13-shell-scripts.spec.ts --project=chromium

# Full suite
cd e2e && npm test
```

### Risk Mitigation

**Risks**:
1. **Vim modal complexity** - Many edge cases, keyboard event handling
2. **Browser differences** - Event handling varies across browsers
3. **State management** - Vim state, config state, panel state coordination

**Mitigation**:
1. **Start simple**: Implement minimal Vim (NORMAL, INSERT modes first)
2. **Test early**: Run E2E tests after each feature
3. **Incremental commits**: Atomic commits per feature

## Success Criteria

### Definition of Done

- ✅ All 1,055 E2E tests passing across 5 browsers
- ✅ No regressions in existing 830 passing tests
- ✅ Code follows WOS quality standards (no SATD, clean code)
- ✅ Documentation updated (CLAUDE.md, help command)
- ✅ Vim editor functional with all test scenarios
- ✅ Config/theme commands working
- ✅ Panel management fully functional

### Metrics

**Current**:
- E2E Tests: 830/1,055 passing (78.67%)
- Failing Suites: 4 (Vim, Config, Panels, Scripts)

**Target**:
- E2E Tests: 1,055/1,055 passing (100%)
- Failing Suites: 0

## Timeline

| Week | Phase | Deliverables | Tests |
|------|-------|--------------|-------|
| 1 | Vim Editor | VimEditor class, keyboard handling, file I/O | +70 |
| 1-2 | Shell Scripts | Vim integration working for scripts | +35 |
| 2 | Config Management | config, theme commands | +50 |
| 2-3 | Panel Polish | Icon rotation, state persistence | +70 |
| 3 | Testing & Docs | Full E2E coverage, documentation | +0 |

**Total**: +225 tests, 100% coverage

## Next Steps

### Immediate Actions

1. **Read full app.js** to understand Terminal class structure
2. **Implement VimEditor class** (highest priority)
3. **Add vim command** to Terminal.executeCommand()
4. **Test first Vim scenario** (`npx playwright test tests/07-vim-editor.spec.ts --grep "should open"`)
5. **Iterate** until all Vim tests pass

### Decision Points

**Q**: Should we implement full Vim compatibility?
**A**: No - only implement features tested by E2E tests (14 unique test scenarios)

**Q**: What about Vim commands beyond :w, :q, :wq, :q!?
**A**: Out of scope - tests only cover these 4 commands

**Q**: Should we add more Vim features for educational value?
**A**: Defer to future phase - focus on 100% test coverage first

## References

### Test Files
- `e2e/tests/07-vim-editor.spec.ts` - Vim editor tests (14 tests × 5 browsers)
- `e2e/tests/08-config-management.spec.ts` - Config management (10 tests × 5 browsers)
- `e2e/tests/09-panel-management.spec.ts` - Panel management (14 tests × 5 browsers)
- `e2e/tests/13-shell-scripts.spec.ts` - Shell scripts (7 tests × 5 browsers)

### Implementation Files
- `dist/wos/index.html` - HTML structure (already complete)
- `dist/wos/app.js` - JavaScript implementation (needs VimEditor, commands)
- `dist/wos/style.css` - CSS styling (needs icon rotation)

### Documentation
- `docs/PROJECT-STATUS-2025-01-19.md` - Current project status
- `CLAUDE.md` - Development guidelines
- `README.md` - Project overview

## Conclusion

This is a well-scoped, achievable implementation plan that will bring WOS to 100% E2E test coverage. The work is frontend-focused, requires no Rust changes, and builds on existing infrastructure.

**Estimated**: 2-3 weeks
**Impact**: +225 passing tests
**Risk**: Low (no kernel changes, incremental approach)
**Value**: Complete the WOS educational experience with full UI functionality
