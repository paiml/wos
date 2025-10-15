# WOS Testing Guide

**Last Updated**: 2025-10-15
**Status**: Active
**Related**: `docs/CANARY-TESTING-ROADMAP.md`, `docs/specifications/wasm-canary-testing-spec.md`

---

## Quick Start

```bash
# Install dependencies (first time only)
make e2e-install

# Build WASM (required before running tests)
make wasm-full

# Run canary tests (single browser - fastest)
make canary-chromium

# Run all canary tests (all browsers - comprehensive)
make canary
```

---

## Test Suites

### 1. Canary Tests (Browser-Based E2E)

**Purpose**: Validate critical user workflows and catch regressions before deployment

**Location**: `e2e/tests/canary/`

**Test Files** (59 tests total):
1. `01-terminal-interaction.spec.ts` - 12 tests (C01-C12)
2. `02-process-management.spec.ts` - 15 tests (C13-C27)
3. `03-file-operations.spec.ts` - 20 tests (C28-C47)
4. `04-state-management.spec.ts` - 12 tests (C48-C59)

**Quick Commands**:
```bash
# Run specific test suite
make canary-terminal    # Terminal interaction tests only
make canary-process     # Process management tests only
make canary-file        # File operations tests only
make canary-state       # State management tests only

# Run on specific browser
make canary-chromium    # Chromium only (fastest, ~2-3 min)
make canary-firefox     # Firefox only
make canary-webkit      # WebKit only

# Development modes
make canary-headed      # Run with visible browser
make canary-ui          # Interactive UI mode
make canary-debug       # Debug mode with breakpoints

# View results
make canary-report      # Open HTML test report
```

**Expected Execution Time**:
- Single browser (chromium): ~2-3 minutes
- All browsers (6 configs): ~15-20 minutes
- Fast subset: ~1 minute

**Performance Baselines**:
- Terminal commands: <100ms
- Process operations: <200ms
- File operations: <150ms
- State reload: <5 seconds

---

## Current Test Status

### Unit Tests (Rust)

**Status**: ✅ All Passing
**Location**: Inline in `*/src/**/*.rs` files
**Count**: 277 tests total
- `wos/src/lib.rs`: 56 tests
- `kernel/src/lib.rs`: 159 tests (includes 68 property tests)
- `shared/src/lib.rs`: 17 tests
- `userspace/src/lib.rs`: 45 tests

**Run Commands**:
```bash
make test              # Run all unit tests
make test-kernel       # Kernel tests only
make test-userspace    # Userspace tests only
make test-shared       # Shared library tests only
```

**Performance**: ~0.8s total execution time

---

### E2E Tests (Existing)

**Status**: ✅ All Passing
**Location**: `e2e/tests/`
**Count**: 29 tests in 5 files

**Test Files**:
1. `01-wasm-initialization.spec.ts` - 3 tests
2. `02-command-execution.spec.ts` - 8 tests
3. `03-process-management.spec.ts` - 6 tests
4. `04-ui-interaction.spec.ts` - 8 tests
5. `05-state-persistence.spec.ts` - 4 tests

**Run Commands**:
```bash
make e2e               # Run all E2E tests
make e2e-chromium      # Chromium only
make e2e-headed        # With visible browser
```

---

### Canary Tests (New)

**Status**: ⏳ Needs First Validation Run
**Location**: `e2e/tests/canary/`
**Count**: 59 tests in 4 files

**Implementation**: ✅ Complete (100%)
**Documentation**: ✅ Complete
**First Run**: ⏳ Pending

**Known Issues**:
1. Default Playwright config runs on 6 browser configurations
2. Causes ~15-20 minute test execution time
3. Recommendation: Use single-browser commands for development

**Recommended Workflow**:
```bash
# Development workflow (fast feedback)
make canary-chromium

# Pre-commit workflow (comprehensive)
make canary

# CI/CD workflow (all browsers + mobile)
cd e2e && npx playwright test tests/canary/
```

---

## Test Configuration

### Playwright Configuration

**Location**: `e2e/playwright.config.ts`

**Browser Projects** (6 total):
1. Chromium (Desktop)
2. Firefox (Desktop)
3. WebKit (Desktop)
4. Mobile Chrome (Android)
5. Mobile Safari (iOS)
6. WebKit (Additional)

**Timeouts**:
- Test timeout: 30 seconds (default)
- Expect timeout: 5 seconds
- Navigation timeout: 30 seconds

**Modification Needed**: Consider reducing browser projects for faster development feedback

---

## Test Development

### Adding New Canary Tests

1. **Choose the appropriate test file**:
   - Terminal interaction → `01-terminal-interaction.spec.ts`
   - Process management → `02-process-management.spec.ts`
   - File operations → `03-file-operations.spec.ts`
   - State management → `04-state-management.spec.ts`

2. **Follow the naming convention**:
   ```typescript
   test('CXX: Test description', async ({ page }) => {
     // Test implementation
   });
   ```

3. **Use helper functions**:
   ```typescript
   async function executeCommand(page: Page, command: string): Promise<void>
   async function getLastOutput(page: Page): Promise<string>
   async function clearTerminal(page: Page): Promise<void>
   ```

4. **Include performance assertions**:
   ```typescript
   const startTime = Date.now();
   // ... operation
   const duration = Date.now() - startTime;
   expect(duration).toBeLessThan(200);
   console.log(`Operation time: ${duration}ms`);
   ```

5. **Run your new tests**:
   ```bash
   make canary-chromium
   ```

---

## Troubleshooting

### Tests Timing Out

**Symptom**: Tests fail with "Timeout exceeded" errors

**Common Causes**:
1. WASM not built → Run `make wasm-full`
2. Dependencies not installed → Run `make e2e-install`
3. Status element not updating → Check browser console
4. Running on too many browsers → Use `make canary-chromium`

**Solutions**:
```bash
# Rebuild everything
make clean
make wasm-full
make e2e-install

# Run with visible browser to debug
make canary-headed

# Run in debug mode
make canary-debug
```

### WASM Initialization Failures

**Symptom**: Status stuck on "Loading WASM..." or "Initializing..."

**Check**:
1. WASM files exist in `dist/wos/`
2. Browser console for errors
3. Network tab shows successful loading

**Solutions**:
```bash
# Rebuild WASM
make wasm-full

# Check build output
ls -lah dist/wos/

# Test manually
python3 -m http.server 8000
# Open http://localhost:8000 in browser
```

### Flaky Tests

**Symptom**: Tests pass sometimes, fail other times

**Common Causes**:
1. Race conditions in async operations
2. Insufficient wait times
3. Browser-specific timing issues

**Solutions**:
- Add `await page.waitForTimeout(50)` after actions
- Use Playwright's auto-waiting features
- Increase timeouts for specific operations

---

## CI/CD Integration

### Pre-Commit Hooks

**Current**: Fast quality gate only (format, clippy, unit tests)
**Planned**: Add `make canary-chromium` to pre-commit

**Configuration**: `.git/hooks/pre-commit`

### GitHub Actions (Future)

**Recommended Workflow**:
```yaml
- name: Run Unit Tests
  run: make test

- name: Build WASM
  run: make wasm-full

- name: Install E2E Dependencies
  run: make e2e-install

- name: Run Canary Tests
  run: make canary

- name: Upload Test Report
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: e2e/playwright-report/
```

---

## Performance Monitoring

### Baseline Metrics

**Captured During Test Execution**:
- Command response times logged to console
- Performance assertions in tests
- HTML report includes timing data

**Viewing Performance Data**:
```bash
make canary-chromium | grep "time:"
# Output shows: "ls command time: 45ms (target: <150ms)"
```

**Performance Targets** (from tests):
| Operation | Target | Test |
|-----------|--------|------|
| Terminal command | <100ms | C09B |
| Process creation | <200ms | C22 |
| File listing (ls) | <150ms | C32 |
| State reload | <5000ms | C53 |
| 20 mixed commands | <3000ms | C45 |

---

## Test Reports

### HTML Report

**Generate**: Automatically created after test run
**View**: `make canary-report` or open `e2e/playwright-report/index.html`

**Contents**:
- Test pass/fail status
- Execution times
- Screenshots on failures
- Browser traces
- Error messages and stack traces

### Console Output

**Format**: List reporter (default)
**Information**:
- Test names and status (✓/✗)
- Execution times
- Performance metrics (via console.log)
- Error details

---

## Next Steps

### Immediate (WOS-025 Completion)

1. ✅ Canary tests implemented (59 tests)
2. ✅ Makefile commands added (14 targets)
3. ⏳ First validation run
4. ⏳ Performance baseline documentation
5. ⏳ Pre-commit hook integration (optional)

### Phase 2 (WOS-026 - CVS)

1. Implement Core Validation Suite (1,000+ systematic tests)
2. Syscall coverage matrix
3. Error path validation
4. Integration with property tests

### Phase 3+ (Future)

1. Anomaly testing (chaos engineering)
2. Differential testing (10K sequences)
3. Soak testing (24-hour stability)
4. Performance regression detection

---

## Resources

- **Canary Testing Roadmap**: `docs/CANARY-TESTING-ROADMAP.md`
- **Canary Testing Spec**: `docs/specifications/wasm-canary-testing-spec.md`
- **Playwright Docs**: https://playwright.dev
- **SQLite Testing Methodology**: https://sqlite.org/testing.html

---

## Questions?

For questions about testing:
1. Check this guide first
2. Review test files in `e2e/tests/canary/`
3. Check Playwright documentation
4. Review commit history for test implementation examples
