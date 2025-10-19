# Resource Management

This document describes the resource management safeguards implemented to prevent runaway processes from consuming all system resources.

## Problem

During development and testing, especially with mutation testing and E2E tests across multiple browsers, processes can accumulate and consume excessive system resources:

- **Cargo mutants**: Spawns rustc processes that can consume 1-3GB each
- **Playwright tests**: Spawns browser instances and node processes
- **Memory exhaustion**: Can lead to 120GB RAM + 85GB swap consumption
- **System instability**: OOM killer, frozen desktop, lost work

## Solution

### 1. Cleanup Script (`scripts/cleanup-runaway-processes.sh`)

Automatically kills runaway processes:

```bash
./scripts/cleanup-runaway-processes.sh
```

Or via Make:

```bash
make cleanup-processes
```

Kills:
- All `cargo mutants` processes
- All `rustc` processes spawned by mutants
- All `playwright test` processes
- All `node` processes running playwright

### 2. Memory Checks (Makefile)

Before running expensive operations (mutants, e2e tests), the Makefile checks memory availability:

```bash
make check-memory
```

Fails if:
- Memory usage > 80%
- Swap usage > 50%

This prevents starting expensive operations when resources are already constrained.

### 3. Automatic Integration

The following Make targets automatically check memory before running:

- `make mutants` - Mutation testing
- `make e2e` - E2E tests across all browsers

If memory check fails, you'll see:

```
⚠️  WARNING: High memory usage detected!
   Run 'make cleanup-processes' to clean up runaway processes
```

### 4. Manual Intervention

If swap is heavily used (>80%), the cleanup script will suggest:

```bash
sudo swapoff -a && sudo swapon -a
```

This clears swap and can free significant memory. **CAUTION**: Only run this when you're sure no critical processes are using swap.

## Best Practices

### Before Starting Work

1. Check memory availability:
   ```bash
   make check-memory
   ```

2. If memory is high, clean up first:
   ```bash
   make cleanup-processes
   ```

### During Development

1. **Avoid running multiple mutation tests in parallel** - Each spawns hundreds of rustc processes
2. **Run E2E tests on single browser first** - Use `make e2e-chromium` instead of `make e2e`
3. **Monitor resource usage** - Keep `htop` or `free -h` running in a terminal
4. **Kill stuck tests immediately** - Don't let them accumulate

### After Testing

1. Always run cleanup after mutation testing:
   ```bash
   make mutants
   make cleanup-processes
   ```

2. Check for lingering processes:
   ```bash
   ps aux | grep -E "(playwright|cargo mutants|rustc.*mutants)" | grep -v grep
   ```

## Monitoring

### Check Memory Usage

```bash
free -h
```

Look for:
- **Mem used**: Should be <80%
- **Swap used**: Should be <50%

### Check Running Processes

```bash
# Top memory consumers
ps aux --sort=-%mem | head -20

# Count playwright processes
ps aux | grep playwright | grep -v grep | wc -l

# Count cargo mutants processes
ps aux | grep -i mutants | grep -v grep | wc -l
```

## Troubleshooting

### Memory Check Fails

**Symptoms**: `make e2e` or `make mutants` fails with memory warning

**Solution**:
```bash
make cleanup-processes
# Wait a few seconds for memory to be freed
make check-memory
# If still high, check what's using memory:
ps aux --sort=-%mem | head -20
```

### Swap is 100% Full

**Symptoms**: System is slow, cleanup script warns about swap usage

**Immediate action**:
```bash
make cleanup-processes
```

**If swap still full after cleanup**:
```bash
# CAUTION: Only do this if you're sure!
sudo swapoff -a && sudo swapon -a
```

### Can't Kill Processes

**Symptoms**: Cleanup script runs but processes remain

**Solution**:
```bash
# Manually kill with root privileges
sudo pkill -9 -f "cargo.mutants"
sudo pkill -9 -f "playwright"
sudo pkill -9 -f "rustc.*mutants"
```

## Future Improvements

Potential enhancements to consider:

1. **Process limits**: Use `ulimit` to cap memory per process
2. **Automatic cleanup**: Pre-commit hook that checks for runaway processes
3. **Resource monitoring**: Log resource usage during test runs
4. **Test isolation**: Use containers or VMs for mutation testing
5. **Incremental testing**: Only run tests on changed code
6. **Parallel test limits**: Cap number of parallel E2E browser instances

## Integration with CI/CD

The memory checks are designed for local development. In CI/CD environments:

- **GitHub Actions**: Runners have fixed memory limits, OOM kills jobs automatically
- **Local CI**: Ensure cleanup runs after each job
- **Docker**: Use memory limits (`--memory`, `--memory-swap`)

Example GitHub Actions workflow:

```yaml
- name: Cleanup before tests
  run: make cleanup-processes || true

- name: Run mutation tests
  run: make mutants

- name: Cleanup after tests
  if: always()
  run: make cleanup-processes || true
```

## Summary

The resource management system provides:

1. ✅ **Automatic memory checks** before expensive operations
2. ✅ **One-command cleanup** for runaway processes
3. ✅ **Clear warnings** when resources are constrained
4. ✅ **Documentation** for troubleshooting and best practices

Use `make cleanup-processes` regularly during development to prevent resource exhaustion.
