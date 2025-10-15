# WOS E2E Tests

End-to-end tests for WOS using Playwright.

## Overview

This directory contains browser-based E2E tests that verify the complete WOS system running in a real browser environment. Tests cover:

- **Basic Loading** - WASM initialization, UI rendering
- **Command Execution** - All terminal commands
- **Command History** - Arrow key navigation
- **UI Interactions** - Buttons, keyboard shortcuts, auto-scroll
- **State Persistence** - Save/load via localStorage

## Prerequisites

- **Node.js** 18+ (for Playwright)
- **Python 3** (for local HTTP server)
- **WASM binary** built (`make wasm` from project root)

## Setup

```bash
# Install dependencies
npm install

# Install Playwright browsers
npx playwright install
```

## Running Tests

### All Browsers

```bash
# Run all tests in all browsers (headless)
npm test

# Or from project root
make e2e
```

### Single Browser

```bash
# Chromium only
npm run test:chromium

# Firefox only
npm run test:firefox

# WebKit/Safari only
npm run test:webkit
```

### Interactive Mode

```bash
# Run with UI (great for debugging)
npm run test:ui

# Run in headed mode (see browser)
npm run test:headed

# Debug mode (step through tests)
npm run test:debug
```

### From Project Root

```bash
# Run E2E tests (starts server automatically)
make e2e

# Run in headed mode
make e2e-headed

# Run UI mode
make e2e-ui

# Run specific browser
make e2e-chromium
make e2e-firefox
make e2e-webkit
```

## Test Structure

```
e2e/
├── package.json           # Dependencies
├── playwright.config.ts   # Playwright configuration
├── tests/
│   ├── 01-basic-loading.spec.ts      # 5 tests - Page load, WASM init
│   ├── 02-command-execution.spec.ts  # 8 tests - Commands
│   ├── 03-command-history.spec.ts    # 4 tests - History navigation
│   ├── 04-ui-interactions.spec.ts    # 8 tests - UI elements
│   └── 05-state-persistence.spec.ts  # 4 tests - localStorage
└── playwright-report/     # HTML test report (generated)
```

**Total: 29 E2E tests**

## Test Organization

Tests are numbered by category:

1. **01-basic-loading** - Foundation tests that must pass first
2. **02-command-execution** - Core functionality
3. **03-command-history** - Advanced terminal features
4. **04-ui-interactions** - User interface components
5. **05-state-persistence** - Data persistence features

## Configuration

### Browsers Tested

- ✅ **Chromium** (Chrome, Edge)
- ✅ **Firefox**
- ✅ **WebKit** (Safari)
- ✅ **Mobile Chrome** (Pixel 5 emulation)
- ✅ **Mobile Safari** (iPhone 12 emulation)

### Test Settings

- **Parallel Execution**: Yes (except CI)
- **Retries on Failure**: 2 (CI only)
- **Trace on Failure**: Yes
- **Screenshots on Failure**: Yes
- **Video on Failure**: Yes

### Local Server

Tests automatically start `python3 -m http.server 8000` and wait for it to be ready before running tests.

## Writing Tests

### Test Template

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should do something', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute action
    await input.fill('help');
    await input.press('Enter');

    // Verify result
    await expect(output).toContainText('Available commands');
  });
});
```

### Best Practices

1. **Always wait for WASM** - Use `waitForSelector('#status:has-text("Ready")')`
2. **Use locators** - Prefer `page.locator()` over selectors
3. **Descriptive names** - Test names should clearly state what they test
4. **Independent tests** - Each test should be runnable in isolation
5. **Clean state** - Use `beforeEach` to ensure clean starting state

### Common Patterns

**Executing a command:**
```typescript
const input = page.locator('#terminal-input');
await input.fill('echo hello');
await input.press('Enter');
```

**Checking output:**
```typescript
const output = page.locator('#terminal-output');
await expect(output).toContainText('hello');
```

**Clicking buttons:**
```typescript
const btn = page.locator('#btn-clear');
await btn.click();
```

**Checking visibility:**
```typescript
const element = page.locator('#terminal');
await expect(element).toBeVisible();
```

## Debugging

### View Test Report

```bash
# After running tests
npm run report

# Or from project root
make e2e-report
```

Opens HTML report with:
- Test results
- Screenshots of failures
- Videos of failures
- Execution traces

### Debug Failing Test

```bash
# Run specific test in debug mode
npx playwright test tests/02-command-execution.spec.ts --debug

# Or run single test
npx playwright test -g "should execute help command" --debug
```

### Generate Test Code

```bash
# Open codegen tool to record actions
npm run codegen

# Playwright will record your browser actions and generate test code
```

### Inspect Element

```bash
# Run with headed mode and Playwright Inspector
PWDEBUG=1 npm test
```

## CI Integration

### GitHub Actions

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Build WASM
        run: |
          rustup target add wasm32-unknown-unknown
          cargo install wasm-bindgen-cli
          make wasm

      - name: Install Playwright
        working-directory: e2e
        run: |
          npm install
          npx playwright install --with-deps

      - name: Run E2E Tests
        working-directory: e2e
        run: npm test

      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: e2e/playwright-report/
```

## Troubleshooting

### Tests Timing Out

**Problem**: Tests timeout waiting for WASM to load

**Solutions**:
- Increase timeout: `{ timeout: 20000 }`
- Check WASM binary was built: `ls -lh dist/wos/*.wasm`
- Verify server is running: `curl http://localhost:8000/dist/wos/`

### Server Not Starting

**Problem**: "Error: connect ECONNREFUSED 127.0.0.1:8000"

**Solutions**:
- Kill existing Python servers: `pkill -f "python3 -m http.server"`
- Check port 8000 is free: `lsof -i :8000`
- Start server manually: `python3 -m http.server 8000`

### Browser Download Failed

**Problem**: "browserType.launch: Executable doesn't exist"

**Solution**:
```bash
npx playwright install chromium
npx playwright install firefox
npx playwright install webkit
```

### WASM Loading Error

**Problem**: "Failed to initialize WASM"

**Solutions**:
- Rebuild WASM: `make wasm`
- Check console errors in headed mode: `npm run test:headed`
- Verify WASM files exist: `ls dist/wos/*.wasm dist/wos/*.js`

## Performance

### Test Execution Times

- **Single browser**: ~15-30 seconds
- **All browsers**: ~60-90 seconds
- **With video/screenshots**: +20% overhead

### Optimization Tips

1. **Disable video** for faster runs:
   ```typescript
   use: { video: 'off' }
   ```

2. **Run fewer browsers** in development:
   ```bash
   npm run test:chromium  # Fastest browser
   ```

3. **Parallel execution**:
   ```typescript
   workers: 4  // Run 4 tests in parallel
   ```

## Coverage

E2E tests cover:

- ✅ **WASM Loading** - Initialization, error handling
- ✅ **Terminal Commands** - help, ps, echo, version, state, reset
- ✅ **Command History** - Arrow up/down navigation
- ✅ **Keyboard Shortcuts** - Ctrl+L for clear
- ✅ **Button Controls** - Clear, reset, save, load
- ✅ **Quality Dashboard** - Metrics display, exports
- ✅ **State Persistence** - Save/load via localStorage
- ✅ **Auto-scroll** - Terminal scrolls to bottom
- ✅ **Input Focus** - Always returns to input

## Future Tests

Potential additional test coverage:

- [ ] Process creation/termination workflows
- [ ] Memory allocation stress testing
- [ ] File system operations
- [ ] Error recovery scenarios
- [ ] Performance benchmarks
- [ ] Accessibility (a11y) tests
- [ ] Mobile gesture support
- [ ] Offline/PWA functionality

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Test Selectors](https://playwright.dev/docs/selectors)
- [Debugging Guide](https://playwright.dev/docs/debug)
- [CI/CD Integration](https://playwright.dev/docs/ci)

## Contributing

When adding E2E tests:

1. Follow the numbering scheme (`01-`, `02-`, etc.)
2. Group related tests in describe blocks
3. Use descriptive test names
4. Add comments for complex interactions
5. Update this README with new test coverage
6. Ensure tests pass in all browsers

## License

Same as WOS project (MIT).
