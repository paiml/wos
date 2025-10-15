# WOS Canary Tests

**SQLite-Inspired Browser Testing for Critical User Workflows**

---

## Quick Start

```bash
# Run all canary tests (fast - Chromium only, ~2-3 min)
make canary

# Run just terminal tests (fastest - ~1 min)
make canary-fast

# Run with visible browser (for debugging)
make canary-headed
```

---

## What Are Canary Tests?

Canary tests are **critical workflow validation tests** inspired by SQLite's legendary 608:1 test-to-code ratio. They validate that the most important user actions work correctly before every release.

**Philosophy**: Like canaries in coal mines, these tests detect problems early.

**Coverage**: 80%+ of critical user workflows with 59 comprehensive tests.

**Speed**: Optimized for fast feedback (~2-3 minutes for full suite).

---

## Test Suites

### 1. Terminal Interaction (C01-C12) - 12 tests
**What**: Command execution, history, keyboard shortcuts
**File**: `01-terminal-interaction.spec.ts` (340 lines)
**Run**: `make canary-terminal`

**Tests**:
- Basic command input/output
- Arrow key command history navigation
- Ctrl+L terminal clearing
- Rapid command execution
- Special character handling
- Edge cases (long input, empty commands)

### 2. Process Management (C13-C27) - 15 tests
**What**: Process lifecycle, ps command, stability
**File**: `02-process-management.spec.ts` (322 lines)
**Run**: `make canary-process`

**Tests**:
- Process listing (ps command)
- Init process validation (PID 1)
- Shell responsiveness
- Process state transitions
- Version/state commands
- System stability under load

### 3. File Operations (C28-C47) - 20 tests
**What**: VFS, ProcFS, file I/O
**File**: `03-file-operations.spec.ts` (421 lines)
**Run**: `make canary-file`

**Tests**:
- Directory listing (ls command)
- VFS operations
- ProcFS (/proc) access
- Error handling (invalid paths)
- Performance under load
- Integration with other subsystems

### 4. State Management (C48-C59) - 12 tests
**What**: Persistence, recovery, localStorage
**File**: `04-state-management.spec.ts` (397 lines)
**Run**: `make canary-state`

**Tests**:
- State persistence across reloads
- localStorage validation
- Corruption recovery
- Tab isolation
- State size bounds
- Performance benchmarks

---

## Running Tests

### Development Workflow (Fast)

```bash
# Recommended for daily development
make canary              # All tests, Chromium only (~2-3 min)

# Even faster - terminal tests only
make canary-fast         # Terminal tests only (~1 min)

# Category-specific
make canary-terminal     # Terminal interaction tests
make canary-process      # Process management tests
make canary-file         # File operations tests
make canary-state        # State management tests
```

### Pre-Release Workflow (Comprehensive)

```bash
# Full browser compatibility testing
make canary-all          # All browsers (~15-20 min)

# Specific browsers
make canary-chromium     # Chromium only
make canary-firefox      # Firefox only
make canary-webkit       # WebKit only
```

### Debugging & Development

```bash
# See what's happening
make canary-headed       # Run with visible browser

# Interactive debugging
make canary-ui           # Playwright UI mode (pause, step, time-travel)

# Debug mode
make canary-debug        # Run with debugger

# View results
make canary-report       # Open HTML test report
```

---

## Performance Baselines

All tests include performance assertions:

| Operation | Target | Measured In |
|-----------|--------|-------------|
| Terminal command | <100ms | C09B |
| Process creation | <200ms | C22 |
| File listing (ls) | <150ms | C32 |
| State reload | <5s | C53 |
| 20 mixed commands | <3s | C45 |

Tests automatically **fail** if performance degrades beyond these targets.

---

## Test Structure

Each test file follows this pattern:

```typescript
import { test, expect, Page } from '@playwright/test';

// Helper functions
async function executeCommand(page: Page, command: string): Promise<void> {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(50);
}

async function getLastOutput(page: Page): Promise<string> {
  const output = page.locator('#terminal-output');
  const text = await output.textContent();
  return text || '';
}

// Test suite
test.describe('Test Category', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('CXX: Test description', async ({ page }) => {
    // Test implementation
    // Performance assertions
    // Validation
  });
});
```

---

## Adding New Tests

1. **Choose the appropriate file**:
   - Terminal → `01-terminal-interaction.spec.ts`
   - Process → `02-process-management.spec.ts`
   - File ops → `03-file-operations.spec.ts`
   - State → `04-state-management.spec.ts`

2. **Follow naming convention**:
   ```typescript
   test('CXX: Clear description of what is being tested', async ({ page }) => {
     // Implementation
   });
   ```

3. **Use helper functions**:
   - `executeCommand(page, cmd)` - Run a command
   - `getLastOutput(page)` - Get terminal output
   - `clearTerminal(page)` - Clear the terminal

4. **Include performance checks** where relevant:
   ```typescript
   const startTime = Date.now();
   // ... operation
   const duration = Date.now() - startTime;
   expect(duration).toBeLessThan(200);
   console.log(`Operation: ${duration}ms (target: <200ms)`);
   ```

5. **Test the test**:
   ```bash
   make canary-chromium
   ```

---

## Troubleshooting

### Tests Timeout

**Problem**: `Test timeout of 30000ms exceeded`

**Solutions**:
```bash
# Rebuild WASM
make wasm-full

# Reinstall dependencies
make e2e-install

# Run with visible browser to see what's happening
make canary-headed
```

### WASM Not Loading

**Problem**: Status stuck on "Initializing..."

**Solutions**:
```bash
# Check WASM files exist
ls -lah dist/wos/

# Rebuild from scratch
make clean
make wasm-full

# Test manually
python3 -m http.server 8000
# Open http://localhost:8000/dist/wos/
# Check browser console for errors
```

### Flaky Tests

**Problem**: Tests pass sometimes, fail other times

**Solutions**:
- Run multiple times: `for i in {1..5}; do make canary; done`
- Check test report: `make canary-report`
- Run in UI mode to inspect: `make canary-ui`

---

## Documentation

- **Quick Reference**: `docs/CANARY-QUICK-REF.md` - Commands and workflows
- **Testing Guide**: `docs/TESTING-GUIDE.md` - Complete testing reference
- **Roadmap**: `docs/CANARY-TESTING-ROADMAP.md` - Implementation plan
- **Status**: `docs/CANARY-TEST-STATUS.md` - Current status
- **Final Report**: `docs/CANARY-FINAL-STATUS.md` - Complete summary

---

## Test Naming Convention

Tests use **SQLite-inspired numbering**:

- **C01-C12**: Terminal Interaction
- **C13-C27**: Process Management
- **C28-C47**: File Operations
- **C48-C59**: State Management

Example: **C01: User types command and sees output**

Format: `C` + number + `: ` + description

---

## Performance

**Default workflow** (make canary):
- **Runtime**: ~2-3 minutes
- **Browser**: Chromium only
- **Tests**: All 59 tests
- **Parallelization**: Fully parallel

**Comprehensive workflow** (make canary-all):
- **Runtime**: ~15-20 minutes
- **Browsers**: Chromium, Firefox, WebKit, Mobile Chrome, Mobile Safari
- **Tests**: All 59 tests × 5 browsers = 295 test runs
- **Parallelization**: Fully parallel

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Canary Tests

on: [push, pull_request]

jobs:
  canary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Dependencies
        run: make e2e-install

      - name: Build WASM
        run: make wasm-full

      - name: Run Canary Tests
        run: make canary-all

      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: canary-report
          path: e2e/playwright-report/
```

### Pre-commit Hook (Optional)

```bash
#!/bin/sh
# .git/hooks/pre-commit

echo "Running canary tests..."
make canary || exit 1
```

---

## FAQ

**Q: Why "canary" tests?**
A: Like canaries in coal mines that detected danger early, these tests detect problems before they reach production.

**Q: How is this different from regular E2E tests?**
A: Canary tests focus on **critical workflows** (80%+ user actions) with **fast execution** (~2-3 min) for rapid feedback.

**Q: When should I run canary tests?**
A: Run `make canary` before every commit. Run `make canary-all` before every release.

**Q: What if a canary test fails?**
A: Don't commit! The test caught a regression. Fix the issue first.

**Q: Can I run a single test?**
A: Yes: `cd e2e && npx playwright test tests/canary/01-terminal-interaction.spec.ts:51`

**Q: How do I see what the test sees?**
A: Run `make canary-headed` to see the browser, or `make canary-ui` for interactive debugging.

---

## Contributing

When adding new canary tests:

1. ✅ Use helper functions
2. ✅ Follow naming convention (CXX: Description)
3. ✅ Include performance assertions
4. ✅ Test edge cases
5. ✅ Add clear error messages
6. ✅ Update this README if adding new categories

---

## Resources

- **Playwright Docs**: https://playwright.dev
- **SQLite Testing**: https://sqlite.org/testing.html
- **WOS Docs**: `docs/`

---

**Questions?** Check `docs/TESTING-GUIDE.md` or `docs/CANARY-QUICK-REF.md`

**Issues?** Check `docs/CANARY-TEST-STATUS.md` for troubleshooting

---

*SQLite-inspired testing for browser-based operating systems* 🐤
