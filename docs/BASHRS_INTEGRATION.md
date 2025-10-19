# Bashrs Integration for Shell Script Quality

## Overview

WOS integrates [bashrs](https://github.com/paiml/bashrs) for static analysis and linting of shell scripts. While WOS has its own lightweight bash interpreter compiled to WASM for runtime execution, we use bashrs CLI tools for **static analysis, linting, and quality gates**.

## Why Not Compile Bashrs to WASM?

Bashrs cannot be compiled to WASM because it:
- Uses `tokio` (async runtime requiring OS threads)
- Performs file I/O operations
- Depends on native system calls

Instead, we use bashrs as a **development-time tool** for validating shell scripts before they're executed in WOS.

## Integration Points

### 1. Pre-Commit Hook (Future)
```bash
# .git/hooks/pre-commit
#!/bin/bash
for script in $(git diff --cached --name-only --diff-filter=ACM | grep '.sh$'); do
    bashrs lint "$script" || exit 1
done
```

### 2. Makefile Quality Gates
```makefile
.PHONY: lint-scripts

lint-scripts:
	@echo "🔍 Linting shell scripts with bashrs..."
	@find scripts -name "*.sh" -exec bashrs lint {} \;
```

### 3. CI/CD Pipeline
Run `bashrs lint` on all shell scripts in CI to catch issues before merge.

## Usage

### Lint a Single Script
```bash
bashrs lint scripts/cleanup-runaway-processes.sh
```

### Lint with Auto-Fix Suggestions
```bash
bashrs lint --fix scripts/cleanup-runaway-processes.sh
```

### Output Formats
```bash
# Human-readable (default)
bashrs lint script.sh

# JSON for tooling
bashrs lint --format json script.sh

# SARIF for IDE integration
bashrs lint --format sarif script.sh
```

## Quality Rules Enforced

Bashrs enforces:
- **Security**: No unquoted variables (prevents injection)
- **Determinism**: No `$RANDOM`, timestamps, or process IDs
- **Idempotency**: Prefer `mkdir -p`, `rm -f`, `ln -sf`
- **POSIX Compliance**: Shellcheck-compatible rules
- **Safety**: Proper error handling with `set -euo pipefail`

## Example Output

```
Issues found in cleanup-runaway-processes.sh:

⚠ 12:17-31 [warning] SC2086: Double quote to prevent globbing
  Fix: "$MUTANTS_COUNT"

Summary: 0 error(s), 8 warning(s), 0 info(s)
```

## Current Status

- ✅ Bashrs CLI installed and available
- ✅ Integration documented
- ✅ Makefile target implemented (`make lint-scripts`)
- ⏸️ Pre-commit hook pending
- ⏸️ CI/CD integration pending

## Next Steps

1. Fix existing warnings in `scripts/cleanup-runaway-processes.sh`
2. Add bashrs lint to pre-commit hooks
3. Integrate into CI/CD pipeline
4. Extend to lint E2E test scripts (generated in tests)

## References

- Bashrs Documentation: https://github.com/paiml/bashrs
- Bashrs Sprint Status: `/home/noah/src/bashrs/CURRENT-STATUS.md`
- WOS Script Executor: `/home/noah/src/wos/wos/src/script_executor.rs`
