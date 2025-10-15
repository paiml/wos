# Canary Testing Quick Reference

**TL;DR**: Run `make canary` for fast feedback (~2-3 min), `make canary-all` for comprehensive validation (~15-20 min)

---

## Quick Commands

### Development Workflow (Fast)

```bash
# Fastest - Terminal tests only (~1 min)
make canary-fast

# Recommended - All canary tests, single browser (~2-3 min)
make canary

# Category-specific tests
make canary-terminal  # Terminal interaction (12 tests)
make canary-process   # Process management (15 tests)
make canary-file      # File operations (20 tests)
make canary-state     # State management (12 tests)
```

### Pre-Release Workflow (Comprehensive)

```bash
# All browsers - Comprehensive validation (~15-20 min)
make canary-all

# Specific browsers
make canary-chromium  # Chromium only
make canary-firefox   # Firefox only
make canary-webkit    # WebKit only
```

### Development & Debugging

```bash
# Visual browser (see what's happening)
make canary-headed

# Interactive UI (pause, step through, time-travel)
make canary-ui

# Debug mode (breakpoints, inspector)
make canary-debug

# View test report
make canary-report
```

---

## Test Coverage

| Suite | Tests | Coverage | Time (Chromium) |
|-------|-------|----------|-----------------|
| Terminal | 12 | Command execution, history, state | ~30s |
| Process | 15 | Process lifecycle, performance | ~45s |
| File | 20 | VFS, ProcFS, errors | ~1m |
| State | 12 | Persistence, recovery | ~30s |
| **Total** | **59** | **80%+ user actions** | **~2-3m** |

---

## Performance Baselines

Tests automatically validate performance:

- ✅ Terminal commands: <100ms
- ✅ Process operations: <200ms
- ✅ File operations: <150ms
- ✅ State reload: <5 seconds
- ✅ Bulk ops (20 cmds): <3 seconds

---

## When to Run What

### During Development
```bash
make canary-fast      # After every few changes
make canary           # Before committing
```

### Before Committing
```bash
make canary           # Validate all critical workflows
```

### Before Releases
```bash
make canary-all       # Full browser compatibility
```

### CI/CD Pipeline
```bash
make canary-all       # Catch browser-specific issues
```

---

## Common Workflows

### 1. Fix a Bug in Terminal

```bash
# Edit code
vim dist/wos/app.js

# Run just terminal tests
make canary-terminal

# If passes, run full suite
make canary
```

### 2. Add New Feature

```bash
# Implement feature
# ...

# Run relevant canary suite
make canary-process  # or canary-file, etc.

# Run full suite before commit
make canary
```

### 3. Debug Failing Test

```bash
# Run in headed mode to see what's happening
make canary-headed

# Or use UI mode for step-by-step debugging
make canary-ui
```

### 4. Performance Regression Check

```bash
# Run canary tests (they include performance assertions)
make canary

# Look for timing logs in output:
# "ls command time: 145ms (target: <150ms)"
```

---

## Troubleshooting

### Tests Timeout

**Problem**: Tests fail with "Timeout exceeded"

**Solutions**:
```bash
# Rebuild WASM
make wasm-full

# Reinstall dependencies
make e2e-install

# Run with visible browser to debug
make canary-headed
```

### WASM Not Loading

**Problem**: Status stuck on "Loading WASM..."

**Solutions**:
```bash
# Check WASM files exist
ls -lah dist/wos/

# Rebuild
make clean
make wasm-full

# Test manually
python3 -m http.server 8000
# Open http://localhost:8000/dist/wos/
```

### Flaky Tests

**Problem**: Tests pass sometimes, fail other times

**Solutions**:
- Run multiple times: `for i in {1..5}; do make canary; done`
- Check browser console for errors: `make canary-headed`
- Review test report: `make canary-report`

---

## Test Naming Convention

Tests follow SQLite-inspired naming:

- **C01-C12**: Terminal Interaction
- **C13-C27**: Process Management
- **C28-C47**: File Operations
- **C48-C59**: State Management

Example: "C01: User types command and sees output"

---

## CI/CD Integration

### GitHub Actions (Recommended)

```yaml
- name: Run Canary Tests
  run: make canary-all

- name: Upload Test Report
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: canary-report
    path: e2e/playwright-report/
```

### Pre-commit Hook (Optional)

```bash
# Add to .git/hooks/pre-commit
make canary || exit 1
```

---

## File Locations

- **Test Files**: `e2e/tests/canary/*.spec.ts`
- **Config**: `e2e/playwright.config.ts`
- **Reports**: `e2e/playwright-report/index.html`
- **Screenshots**: `e2e/test-results/*/screenshots/`
- **Videos**: `e2e/test-results/*/videos/`

---

## Further Reading

- **Full Guide**: `docs/TESTING-GUIDE.md`
- **Roadmap**: `docs/CANARY-TESTING-ROADMAP.md`
- **Specification**: `docs/specifications/wasm-canary-testing-spec.md`
- **Playwright Docs**: https://playwright.dev

---

## Quick Tips

💡 **Use `make canary` for daily development** - Fast feedback loop

💡 **Use `make canary-all` before releases** - Comprehensive validation

💡 **Use `make canary-ui` for debugging** - Visual step-through

💡 **Check `make canary-report` after failures** - Detailed diagnostics

💡 **Performance metrics logged to console** - Watch for regression

---

**Questions?** Check `docs/TESTING-GUIDE.md` or review test files in `e2e/tests/canary/`
