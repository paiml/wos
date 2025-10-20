# Five-Whys Investigation: E2E Test Failures
**Date**: 2025-10-20
**Session**: Continuation from UI Implementation

## Executive Summary

Performed deep five-whys investigation into E2E test timeout failures. Initially concluded tests were working, but fresh full test run reveals **ALL tests timing out** waiting for `#status:has-text("Ready")` in beforeEach hooks.

## Investigation Timeline

### Phase 1: Initial Diagnosis (INCORRECT)
- Observed single test timeout
- Analyzed screenshot showing WOS loaded successfully
- Concluded: timing race condition, tests actually work
- Evidence: Previous test run showed 979/1190 passing

### Phase 2: Configuration Fix
- Fixed `e2e/playwright.config.ts` line 78
- Changed webServer.url from `http://localhost:8001` → `http://localhost:8001/wos/index.html`
- Committed (09d786d) and pushed successfully

### Phase 3: Fresh Test Run (REALITY CHECK)
- Launched full E2E suite with fixed config
- Result: **ALL tests timing out after 1 minute**
- Pattern: Every test fails at beforeEach hook waiting for #status

## Root Cause Analysis (UPDATED)

### Actual Problem
The #status element is NOT being set to "Ready" consistently. Tests worked in previous sessions but something changed.

### Evidence
```
✘ [chromium] › tests/07-vim-editor.spec.ts:33:7 › should enter insert mode with "i" key (1.0m)
✘ [chromium] › tests/09-panel-management.spec.ts:12:7 › should display all panels (10.4s)
✘ [chromium] › tests/10-system-monitor.spec.ts:21:7 › should display System Monitor (1.0m)
```

All share same failure: timeout waiting for status Ready

### What Changed?
1. **playwright.config.ts URL fix** - This was correct but insufficient
2. **WASM Loading** - May be failing silently
3. **init() function** - May not be completing
4. **Status update timing** - Race condition in initialization

## Technical Details

### Application Initialization Flow
```javascript
// app.js line 1433-1456
async function initApp() {
  statusElement.innerHTML = '<span class="loading"></span> Loading WASM...';
  
  await init();  // ← This may be failing
  
  const configManager = new ConfigManager();
  const panelManager = new PanelManager(configManager);
  const terminal = new Terminal(configManager);
  
  window.terminalInstance = terminal;
  
  const wos = new WosWasm();
  terminal.setWOS(wos);
  
  statusElement.textContent = 'Ready';  // ← Never reached
}
```

### Test BeforeEach Hook
```typescript
// All tests use this pattern
test.beforeEach(async ({ page }) => {
  await page.goto('index.html');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  // ↑ This times out every time
});
```

## Hypotheses (Ranked by Likelihood)

### 1. WASM Binary Loading Failure (HIGH)
- **Theory**: WASM binary not loading in Playwright browser context
- **Evidence**: All tests timeout at same point
- **Test**: Check browser console logs, verify WASM path
- **Fix**: Ensure wos_bg.wasm is accessible at correct path

### 2. init() Function Hanging (MEDIUM)  
- **Theory**: wasm-bindgen init() never resolves
- **Evidence**: Status never changes from "Loading WASM..."
- **Test**: Add timeout/error handling to init()
- **Fix**: Catch init() errors, show error state

### 3. JavaScript Error Breaking Initialization (MEDIUM)
- **Theory**: Uncaught exception prevents status update
- **Evidence**: All tests fail consistently
- **Test**: Check browser console for errors
- **Fix**: Add try/catch, show error messages

### 4. DOM Ready Race Condition (LOW)
- **Theory**: initApp() called before DOM ready
- **Evidence**: Code has proper readyState check
- **Code**:
```javascript
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}
```

## Next Steps

1. **Capture Browser Console Logs** - Run test with console output
2. **Check WASM Binary Path** - Verify wos_bg.wasm served correctly  
3. **Add Error Handling** - Wrap init() in try/catch with error display
4. **Test Manual Load** - Open http://localhost:8001/wos/index.html manually
5. **Review Git Changes** - Check if dist/wos/app.js changed unexpectedly

## Files Investigated

- `/home/noah/src/wos/e2e/playwright.config.ts` - Fixed webServer.url
- `/home/noah/src/wos/dist/wos/app.js` - Initialization code (lines 1433-1469)
- `/home/noah/src/wos/dist/wos/index.html` - Status element (line 189)
- `/home/noah/src/wos/e2e/tests/09-panel-management.spec.ts` - BeforeEach hook

## Lessons Learned

1. **Don't Trust Old Test Runs** - Environment may have changed
2. **Fresh Test Required** - Always verify fix with full test run
3. **Screenshot Timing** - Screenshot showed loaded page but after timeout
4. **Pattern Recognition** - ALL tests failing same way = systemic issue

## Status: INVESTIGATION ONGOING

Current full test run shows 100% failure rate. Problem is NOT solved.
Previous conclusion (tests working) was **INCORRECT**.

Waiting for test completion to analyze browser logs and error details.
