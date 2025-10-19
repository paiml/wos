# Panel Management Test Failures - Investigation Notes

**Date**: October 19, 2025
**Status**: Requires investigation and fix
**Failing Tests**: 60 tests (12 tests × 5 browsers)

## Summary

Panel management E2E tests are failing across all browsers with "element is not visible" errors for collapse buttons.

## Test Suite

**File**: `/home/noah/src/wos/e2e/tests/09-panel-management.spec.ts`

**Failing Tests (12 tests)**:
1. should display all panels on startup
2. should collapse panel when collapse button is clicked
3. should expand panel when collapse button is clicked again
4. should rotate collapse icon when collapsing
5. should handle multiple panel collapses independently
6. should display process table in process list panel
7. should display memory information in memory map panel
8. should display system call trace in syscall trace panel
9. should have clear trace button in syscall trace panel
10. should have refresh button in process list panel
11. should maintain panel state across command executions
12. (1 additional vim test unrelated to panels)

**Browsers Affected**: All 5 browsers
- Chromium
- Firefox
- WebKit
- Mobile Chrome
- Mobile Safari

## Error Pattern

```
Error: element is not visible
- waiting for locator('[data-panel="process_list"]').locator('.btn-collapse')
  - locator resolved to <button class="btn-icon btn-collapse" title="Collapse/Expand panel">…</button>
  - element is not visible
```

## Investigation Findings

### 1. HTML Structure (dist/wos/index.html)

Panels are properly defined with collapse buttons:

```html
<div id="panel-process-list" class="file-panel process-panel" data-panel="process_list">
  <div class="file-panel-header">
    <h3>Process List</h3>
    <div class="file-controls">
      <button class="btn-icon btn-collapse" title="Collapse/Expand panel">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M7 14l5-5 5 5z"/>
        </svg>
      </button>
      ...
    </div>
  </div>
  <div class="panel-content">
    ...
  </div>
</div>
```

**Panels with collapse buttons**:
- `process_list` ✓
- `memory_map` ✓
- `syscall_trace` ✓
- `system_monitor_detailed` ✓
- `filesystem` ✗ (no collapse button, has other controls)
- `system_monitor` ✗ (no collapse button, simple info panel)

### 2. Panel Manager (dist/wos/app.js)

**Initialization** (lines 66-101):
- Discovers all panels via `[data-panel]` selector
- Applies config-based visibility
- **Applies initial collapsed state from config** (lines 96-97)

```javascript
// Apply initial collapsed state
if (panelConfig.collapsed === true) {
  this.collapsePanel(panelName);
}
```

**Collapse Logic** (lines 128-145):
```javascript
collapsePanel(panelName) {
  const panel = this.panels[panelName];
  if (!panel) return;

  panel.element.classList.add('collapsed');
  const content = panel.element.querySelector('.panel-content, .file-browser, .file-actions, .file-info, .system-info, .quality-metrics');
  if (content) {
    content.style.display = 'none';
  }

  // Rotate collapse icon
  const collapseBtn = panel.element.querySelector('.btn-collapse svg');
  if (collapseBtn) {
    collapseBtn.style.transform = 'rotate(180deg)';
  }

  this.saveState();
}
```

### 3. Default Config (wos/src/config.rs)

**PanelConfig defaults** (lines 152-160):
```rust
impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            visible: true,
            collapsed: false,  // Panels start EXPANDED
            position: 0,
        }
    }
}
```

### 4. Test Setup (e2e/tests/09-panel-management.spec.ts)

**beforeEach** (lines 4-10):
```typescript
test.beforeEach(async ({ page }) => {
  // Clear localStorage before each test
  await page.goto('index.html');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
});
```

This should give clean default config with `collapsed: false`.

### 5. CSS (dist/wos/style.css)

**Collapsed state** (lines 737-743):
```css
.file-panel.collapsed {
  max-height: 50px;
}

.file-panel.collapsed .panel-content {
  display: none;
}
```

No CSS rules hide `.btn-collapse` buttons.

## Hypothesis

The "element is not visible" error suggests one of:

1. **Panel is off-screen** - Panel positioning/layout pushes buttons outside viewport
2. **Parent element hidden** - Some parent container has `display: none` or `visibility: hidden`
3. **Z-index stacking** - Button is behind another element
4. **Timing issue** - Button not yet rendered when test tries to click it
5. **Config loading race** - Config being loaded/applied after panel initialization

## Debugging Steps Required

1. **Visual inspection**: Run test with `headless: false` to see actual rendering
2. **Screenshot analysis**: Check test failure screenshots in `test-results/`
3. **Element inspection**: Log computed styles of buttons and parent elements
4. **Timing analysis**: Add explicit waits for panel initialization
5. **Config verification**: Log actual config being applied to panels

## Recommended Fix Approach

### Option 1: Fix Root Cause (Preferred)
1. Identify why buttons are not visible
2. Fix CSS/layout/timing issue
3. Verify all 60 tests pass

### Option 2: Test Adjustment
1. Add explicit visibility waits
2. Use `force: true` on clicks if elements are functionally available
3. Update selectors if needed

### Option 3: Panel Initialization Fix
1. Ensure panels are fully initialized before tests start
2. Add data attribute when panel is ready
3. Wait for ready state in tests

## Priority

**Medium-High**: 60 failing tests (5.6% of total suite)

Current E2E status:
- **Passing**: 1015/1070 (94.9%)
- **Failing**: 55/1070 (panel tests)

## Next Steps

1. Run single panel test with browser visible
2. Inspect actual rendering
3. Identify root cause
4. Implement fix
5. Verify all 60 tests pass
6. Commit with comprehensive documentation

## Related Files

- **Test**: `e2e/tests/09-panel-management.spec.ts`
- **HTML**: `dist/wos/index.html`
- **JS**: `dist/wos/app.js` (PanelManager class)
- **CSS**: `dist/wos/style.css`
- **Config**: `wos/src/config.rs`

## Notes

- Default config has `collapsed: false`, so panels should be expanded
- Tests clear localStorage to get clean state
- Collapse buttons are present in HTML
- No CSS rules hide buttons
- Issue affects all browsers uniformly (not browser-specific)
