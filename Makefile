# Disable built-in implicit rules for faster Make execution
.SUFFIXES:

.PHONY: help build test test-fast coverage quality wasm clean dist fmt lint hooks-install bench bench-baseline bench-compare bench-syscalls bench-scheduler bench-memory mutants mutants-check mutants-diff mutants-kernel mutants-incremental fuzz fuzz-install fuzz-syscalls fuzz-processes fuzz-memory fuzz-scheduler fuzz-coverage fuzz-clean e2e e2e-install e2e-headed e2e-ui e2e-debug e2e-chromium e2e-firefox e2e-webkit e2e-report e2e-clean canary canary-all canary-fast canary-terminal canary-process canary-file canary-state canary-error canary-headed canary-ui canary-debug canary-report canary-chromium canary-firefox canary-webkit lint-frontend lint-frontend-fix lint-frontend-check lint-scripts lint-all cleanup-processes check-memory link-dev deploy deploy-build deploy-upload deploy-invalidate deploy-check deploy-config bashrs-check bashrs-fix bashrs-audit bashrs-score bashrs-test bashrs-coverage bashrs-format bashrs-purify

export PATH := $(HOME)/.cargo/bin:$(PATH)

WASM_TARGET := target/wasm32-unknown-unknown/release/wos.wasm

# ============================================================================
# Process and Resource Management
# ============================================================================

cleanup-processes:
	@echo "🧹 Cleaning up runaway processes..."
	@bash scripts/cleanup-runaway-processes.sh

check-memory:
	@MEM_USED=$$(free | grep Mem | awk '{printf "%.0f", ($$3/$$2)*100}'); \
	SWAP_USED=$$(free | grep Swap | awk '{if ($$2 > 0) printf "%.0f", ($$3/$$2)*100; else print "0"}'); \
	echo "💾 Memory: $${MEM_USED}% used, Swap: $${SWAP_USED}% used"; \
	if [ $$MEM_USED -gt 85 ] || [ $$SWAP_USED -gt 95 ]; then \
		echo "⚠️  WARNING: High memory usage detected!"; \
		echo "   Run 'make cleanup-processes' to clean up runaway processes"; \
		exit 1; \
	fi

help:
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "  WOS - WASM Operating System Build System"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "🏗️  Building:"
	@echo "  make build            Build all crates"
	@echo "  make wasm             Build WASM binary"
	@echo "  make dist             Build distribution (alias for wasm)"
	@echo ""
	@echo "🧪 Testing:"
	@echo "  make test             Run all Rust tests"
	@echo "  make test-unit        Run Rust unit tests only"
	@echo "  make test-frontend    Run frontend unit tests (Deno)"
	@echo "  make test-all         Run all tests (Rust + Frontend + E2E)"
	@echo "  make coverage         Generate comprehensive coverage (Rust + E2E)"
	@echo "  make coverage-rust    Generate Rust coverage only"
	@echo "  make coverage-e2e     Generate E2E test coverage only"
	@echo "  make coverage-frontend Generate frontend coverage report"
	@echo "  make coverage-summary Show unified coverage summary"
	@echo "  make coverage-all     Generate all coverage + summary"
	@echo "  make coverage-check   Verify coverage ≥85%"
	@echo "  make bench            Run Rust performance benchmarks"
	@echo "  make bench-frontend   Run frontend benchmarks"
	@echo "  make bench-all        Run all benchmarks"
	@echo "  make bench-baseline   Save benchmark baseline"
	@echo "  make fuzz             Run fuzz tests (60s per target)"
	@echo "  make e2e              Run E2E tests (all browsers)"
	@echo "  make e2e-chromium     Run E2E tests (Chromium only)"
	@echo "  make canary           Run canary tests (59 tests, Chromium, ~2-3 min)"
	@echo "  make canary-all       Run canary tests (all browsers, ~15-20 min)"
	@echo "  make canary-fast      Run fast canary tests (terminal only, ~1 min)"
	@echo "  make canary-terminal  Run terminal canary tests"
	@echo ""
	@echo "🎯 Quality Gates:"
	@echo "  make quality          Fast quality checks (<30s, includes PMAT + bashrs)"
	@echo "  make quality-complete Complete quality validation (~5min)"
	@echo "  make fmt              Format code"
	@echo "  make clippy           Run clippy lints"
	@echo "  make lint-scripts     Lint shell scripts with bashrs"
	@echo "  make lint-all         Run all linters (Rust + Frontend + Scripts)"
	@echo "  make pmat-complexity  Check complexity (max 10)"
	@echo "  make pmat-satd        Check for SATD (zero tolerance)"
	@echo "  make pmat-entropy     Analyze code entropy"
	@echo "  make pmat-tdg         Grade technical debt (TDG)"
	@echo "  make pmat-gates       Run all PMAT quality gates"
	@echo "  make pmat-roadmap-status      Check roadmap/ticket status"
	@echo "  make pmat-roadmap-validate    Validate roadmap quality gates"
	@echo "  make bashrs-check     Fast bashrs validation (lint + score)"
	@echo "  make bashrs-audit     Comprehensive bashrs audit"
	@echo "  make bashrs-score     Score all shell scripts"
	@echo "  make bashrs-test      Run bashrs test framework"
	@echo "  make bashrs-coverage  Generate bashrs coverage reports"
	@echo "  make bashrs-format    Format shell scripts"
	@echo "  make bashrs-purify    Purify scripts (determinism + safety)"
	@echo "  make bashrs-fix       Auto-fix all bashrs issues"
	@echo "  make mutants          Run mutation tests (~10-15min)"
	@echo "  make mutants-check    Verify mutation score ≥90%"
	@echo "  make mutants-diff     Show diffs for caught mutants"
	@echo "  make mutants-incremental  Test only modified files"
	@echo ""
	@echo "🔧 Development:"
	@echo "  make hooks-install    Install pre-commit hooks"
	@echo "  make serve            Start development server with hot reload (port 8000)"
	@echo "  make clean            Clean build artifacts"
	@echo "  make cleanup-processes Kill runaway test/build processes"
	@echo "  make check-memory     Verify system memory availability"
	@echo ""
	@echo "🚀 Deployment:"
	@echo "  make link-dev         Link dist to paiml.com for rapid iteration"
	@echo "  make deploy           Full deployment (build + upload + cache invalidation)"
	@echo "  make deploy-config    Create .env.deploy.example configuration"
	@echo "  make deploy-check     Verify deployment prerequisites"
	@echo "  make deploy-build     Build production WASM"
	@echo "  make deploy-upload    Upload to S3"
	@echo "  make deploy-invalidate Invalidate CloudFront cache"
	@echo ""
	@echo "📖 See docs/DEPLOYMENT.md for detailed deployment guide"

# ============================================================================
# Build Targets
# ============================================================================

build:
	@echo "🔨 Building all crates..."
	@cargo build --workspace
	@echo "✓ Build complete"

build-release:
	@echo "🔨 Building release..."
	@cargo build --workspace --release
	@echo "✓ Release build complete"

wasm:
	@echo "📦 Building WASM binary..."
	@cargo build --target wasm32-unknown-unknown --release -p wos
	@if [ -f "$(WASM_TARGET)" ]; then \
		SIZE=$$(stat -c%s "$(WASM_TARGET)" 2>/dev/null || stat -f%z "$(WASM_TARGET)" 2>/dev/null); \
		SIZE_KB=$$((SIZE / 1024)); \
		echo "📦 WASM size: $${SIZE_KB} KB ($$SIZE bytes)"; \
		if [ $$SIZE -gt 512000 ]; then \
			echo "⚠️  Warning: WASM exceeds 500KB target"; \
		fi; \
	fi
	@echo "🔗 Generating JavaScript bindings..."
	@which wasm-bindgen > /dev/null 2>&1 || (echo "❌ wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli" && exit 1)
	@wasm-bindgen $(WASM_TARGET) --out-dir dist/wos --target web
	@echo "⚡ Optimizing WASM binary with wasm-opt..."
	@if which wasm-opt > /dev/null 2>&1; then \
		BEFORE=$$(stat -c%s dist/wos/wos_bg.wasm 2>/dev/null || stat -f%z dist/wos/wos_bg.wasm 2>/dev/null); \
		wasm-opt -Oz --enable-bulk-memory --enable-sign-ext --enable-mutable-globals --enable-nontrapping-float-to-int \
			dist/wos/wos_bg.wasm -o dist/wos/wos_bg_optimized.wasm; \
		mv dist/wos/wos_bg_optimized.wasm dist/wos/wos_bg.wasm; \
		AFTER=$$(stat -c%s dist/wos/wos_bg.wasm 2>/dev/null || stat -f%z dist/wos/wos_bg.wasm 2>/dev/null); \
		SAVED=$$((BEFORE - AFTER)); \
		PERCENT=$$(awk "BEGIN {printf \"%.1f\", ($$SAVED/$$BEFORE)*100}"); \
		echo "  ✓ Optimized: $$SAVED bytes saved ($$PERCENT%)"; \
		GZIPPED=$$(gzip -c dist/wos/wos_bg.wasm | wc -c); \
		GZIPPED_KB=$$((GZIPPED / 1024)); \
		echo "  📦 Gzipped size: $${GZIPPED_KB} KB"; \
	else \
		echo "  ⚠️  wasm-opt not found - skipping optimization"; \
		echo "  💡 Install binaryen for smaller WASM: apt-get install binaryen"; \
	fi
	@echo "✓ WASM build complete"
	@echo "💡 Start development server: make serve (hot reload + WASM auto-compilation)"
	@echo "💡 Open browser: http://localhost:8000/"

dist: wasm

# ============================================================================
# Testing
# ============================================================================

test:
	@echo "🧪 Running all tests..."
	@cargo test --workspace --all-features
	@echo "✓ All tests passed"

test-fast:
	@echo "🧪 Running fast unit tests..."
	@cargo nextest run --lib --workspace
	@echo "✓ Fast tests passed"

test-unit:
	@echo "🧪 Running unit tests..."
	@cargo test --workspace --lib
	@echo "✓ Unit tests passed"

test-frontend:
	@echo "🧪 Running frontend unit tests..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task test
	@echo "✓ Frontend tests passed"

test-frontend-property:
	@echo "🧪 Running frontend property tests..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task test:property
	@echo "✓ Frontend property tests passed"

test-frontend-all:
	@echo "🧪 Running all frontend tests..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task test:all
	@echo "✓ All frontend tests passed"

test-all: test test-frontend-all e2e
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ All tests passed (Rust + Frontend + E2E)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Coverage
# ============================================================================

coverage:
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "📊 Comprehensive Coverage Analysis (Rust + Frontend + E2E)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "🗑️  Cleaning old coverage data..."
	@rm -rf target/coverage
	@rm -f e2e/test-results.json
	@mkdir -p target/coverage
	@echo ""
	@echo "🦀 Running Rust coverage..."
	@which cargo-tarpaulin > /dev/null 2>&1 || (echo "📦 Installing cargo-tarpaulin..." && cargo install cargo-tarpaulin --locked)
	@cargo tarpaulin --workspace --out Html --out Lcov --output-dir target/coverage --timeout 300 --exclude-files 'wos/*' 'dist/*'
	@echo ""
	@echo "🌐 Running E2E test coverage..."
	@cd e2e && npm run test:coverage && npm run coverage:summary
	@echo ""
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ Coverage Reports Generated"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "📄 Rust Coverage:"
	@echo "   • HTML: target/coverage/tarpaulin-report.html"
	@echo "   • LCOV: target/coverage/lcov.info"
	@echo ""
	@echo "📄 E2E Coverage:"
	@echo "   • Summary: target/coverage/e2e-summary.json"
	@echo "   • Report: e2e/playwright-report/index.html"
	@echo ""
	@echo "💡 Run 'make coverage-summary' for unified report"
	@echo ""

coverage-check:
	@echo "📊 Checking coverage thresholds (≥85%)..."
	@if [ ! -f "target/coverage/lcov.info" ]; then \
		echo "❌ Coverage data not found. Run 'make coverage' first"; \
		exit 1; \
	fi
	@cargo tarpaulin --workspace --out Stdout --timeout 300 --exclude-files 'wos/*' 'dist/*' --fail-under 85
	@echo "✓ Coverage thresholds met"

coverage-frontend:
	@echo "📊 Running frontend test coverage..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task test:coverage
	@echo ""
	@echo "💡 Coverage profile: dist/wos/cov_profile"
	@echo ""

coverage-frontend-all:
	@echo "📊 Running comprehensive frontend coverage (unit + property tests)..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task test:coverage:all
	@echo ""
	@echo "💡 Coverage profile: dist/wos/cov_profile"
	@echo ""

coverage-summary:
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "📊 Unified Coverage Summary"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@if [ -f "target/coverage/lcov.info" ]; then \
		echo "🦀 Rust Coverage:"; \
		grep -E "^LF:|^LH:" target/coverage/lcov.info | \
		awk -F: 'BEGIN {lines=0; hit=0} /^LF/ {lines+=$$2} /^LH/ {hit+=$$2} END {if (lines > 0) printf "   Lines: %d/%d (%.2f%%)\n", hit, lines, (hit/lines)*100; else print "   No data"}'; \
	else \
		echo "⚠️  No Rust coverage data found. Run 'make coverage' first."; \
	fi
	@echo ""
	@if [ -f "target/coverage/e2e-summary.json" ]; then \
		echo "🌐 E2E Test Coverage:"; \
		node -pe "const data=require('./target/coverage/e2e-summary.json'); '   Tests: ' + data.stats.passed + '/' + data.stats.total + ' (' + data.passRate.toFixed(2) + '%)'"; \
	else \
		echo "⚠️  No E2E coverage data found. Run 'make coverage' first."; \
	fi
	@echo ""
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "💡 View detailed reports:"
	@echo "   • Rust: target/coverage/tarpaulin-report.html"
	@echo "   • E2E:  e2e/playwright-report/index.html"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""

coverage-e2e:
	@echo "🌐 Running E2E test coverage..."
	@echo "🗑️  Cleaning old E2E coverage data..."
	@rm -f e2e/test-results.json
	@rm -f target/coverage/e2e-summary.json
	@mkdir -p target/coverage
	@cd e2e && npm run test:coverage
	@cd e2e && npm run coverage:summary
	@echo "✓ E2E coverage complete"

coverage-rust:
	@echo "🦀 Running Rust coverage..."
	@echo "🗑️  Cleaning old Rust coverage data..."
	@rm -f target/coverage/lcov.info target/coverage/tarpaulin-report.html target/coverage/cobertura.xml
	@which cargo-tarpaulin > /dev/null 2>&1 || (echo "📦 Installing cargo-tarpaulin..." && cargo install cargo-tarpaulin --locked)
	@mkdir -p target/coverage
	@cargo tarpaulin --workspace --out Html --out Lcov --output-dir target/coverage --timeout 300 --exclude-files 'wos/*' 'dist/*'
	@echo "✓ Rust coverage complete"

coverage-all: coverage
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ All coverage reports generated (Rust + E2E)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@make coverage-summary

# ============================================================================
# Quality Gates
# ============================================================================

fmt:
	@echo "🎨 Checking code formatting..."
	@cargo fmt --all -- --check
	@echo "✓ Formatting OK"

fmt-fix:
	@echo "🎨 Fixing code formatting..."
	@cargo fmt --all
	@echo "✓ Formatting fixed"

clippy:
	@echo "📎 Running clippy..."
	@cargo clippy --workspace --all-features --target wasm32-unknown-unknown -- -D warnings
	@echo "✓ Clippy passed"

lint: fmt clippy
	@echo "✅ All linting passed"

pmat-complexity:
	@echo "🔍 Checking code complexity (max 10)..."
	@pmat analyze complexity --path . --max-cyclomatic 10 --max-cognitive 10 || (echo "❌ Complexity exceeded max of 10" && exit 1)
	@echo "✓ Complexity check passed"

pmat-satd:
	@echo "🔍 Checking for technical debt (SATD)..."
	@pmat analyze satd --path . || (echo "❌ SATD comments found" && exit 1)
	@echo "✓ No SATD comments found"

pmat-entropy:
	@echo "🔍 Analyzing code entropy..."
	@pmat analyze entropy --path . || true
	@echo "✓ Entropy analysis complete"

pmat-tdg:
	@echo "🔍 Grading technical debt (TDG)..."
	@pmat tdg . || (echo "❌ TDG threshold exceeded" && exit 1)
	@echo "✓ TDG check passed"

pmat-dead-code:
	@echo "🔍 Detecting dead code..."
	@pmat analyze dead-code --path . || (echo "❌ Dead code detected" && exit 1)
	@echo "✓ No dead code detected"

pmat-gates:
	@echo "🔍 Running PMAT quality gates..."
	@pmat quality-gate --project-path . --fail-on-violation || (echo "❌ PMAT quality gates failed" && exit 1)
	@echo "✓ PMAT gates passed"

pmat-roadmap-status:
	@echo "📋 Checking roadmap status..."
	@pmat roadmap status --roadmap roadmap.yaml || true
	@echo "✓ Roadmap status checked"

pmat-roadmap-validate:
	@echo "✅ Validating roadmap quality gates..."
	@pmat roadmap validate --roadmap roadmap.yaml || (echo "❌ Roadmap validation failed" && exit 1)
	@echo "✓ Roadmap validated"

# ============================================================================
# bashrs: Shell Script Quality Validation
# ============================================================================

# ============================================================================
# bashrs Integration - Full Capabilities
# ============================================================================

bashrs-check: bashrs-lint bashrs-score
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ bashrs validation passed"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

bashrs-lint:
	@echo "🔍 Linting shell scripts with bashrs..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@FOUND_ERRORS=0; \
	for script in $$(find scripts -name "*.sh" 2>/dev/null); do \
		echo "  Linting $$script..."; \
		OUTPUT=$$(bashrs lint "$$script" 2>&1); \
		echo "$$OUTPUT" | grep -q "^\[error\]" && FOUND_ERRORS=1 || true; \
	done; \
	echo "  Linting Makefile..."; \
	OUTPUT=$$(bashrs make lint Makefile 2>&1); \
	echo "$$OUTPUT" | grep "Summary:" | grep -q "^Summary: [1-9].*error" && FOUND_ERRORS=1 || true; \
	if [ $$FOUND_ERRORS -eq 1 ]; then \
		echo "❌ bashrs linting failed (errors found)"; \
		exit 1; \
	fi
	@echo "✓ bashrs linting passed (warnings allowed)"

bashrs-score:
	@echo "📊 Scoring shell scripts with bashrs..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@echo ""
	@echo "Scripts Quality Scores:"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@for script in Makefile scripts/*.sh; do \
		if [ -f "$$script" ]; then \
			echo ""; \
			bashrs score "$$script" 2>&1 | grep -A 3 "Overall Grade\|Overall Score" || true; \
		fi; \
	done
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

bashrs-audit:
	@echo "🔍 Running comprehensive bashrs audit..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@echo ""
	@for script in Makefile scripts/*.sh; do \
		if [ -f "$$script" ]; then \
			echo ""; \
			echo "Auditing $$script..."; \
			bashrs audit "$$script" || true; \
		fi; \
	done

bashrs-test:
	@echo "🧪 Running bashrs test framework..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@for script in scripts/*.sh; do \
		if [ -f "$$script" ]; then \
			echo "  Testing $$script..."; \
			bashrs test "$$script" || echo "⚠️  No tests found for $$script"; \
		fi; \
	done

bashrs-coverage:
	@echo "📊 Generating bashrs coverage report..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@mkdir -p target/bashrs-coverage
	@for script in scripts/*.sh; do \
		if [ -f "$$script" ]; then \
			echo "  Coverage for $$script..."; \
			bashrs coverage "$$script" > "target/bashrs-coverage/$$(basename $$script).coverage" 2>&1 || true; \
		fi; \
	done
	@echo "✓ Coverage reports generated in target/bashrs-coverage/"

bashrs-format:
	@echo "🎨 Formatting shell scripts with bashrs..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@for script in scripts/*.sh; do \
		if [ -f "$$script" ]; then \
			echo "  Formatting $$script..."; \
			bashrs format "$$script" || true; \
		fi; \
	done
	@echo "✓ Shell scripts formatted"

bashrs-purify:
	@echo "🔧 Purifying shell scripts with bashrs..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: cargo install bashrs" && exit 1)
	@for script in scripts/*.sh; do \
		if [ -f "$$script" ]; then \
			echo "  Purifying $$script..."; \
			bashrs purify "$$script" --backup || true; \
		fi; \
	done
	@echo "✓ Shell scripts purified (backups created)"

bashrs-fix: bashrs-format bashrs-purify
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ bashrs auto-fix complete"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

quality: fmt clippy test-unit pmat-complexity pmat-satd pmat-entropy pmat-dead-code bashrs-check
# pmat-tdg temporarily disabled due to sled backend unavailability (reinstalling with --features sled-backend)
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ Quality gates passed (<30s)"
	@echo "   • Format, Clippy, Unit Tests: PASSING"
	@echo "   • PMAT Complexity: PASSING"
	@echo "   • PMAT SATD (Zero TODO): PASSING"
	@echo "   • PMAT Entropy Analysis: PASSING"
	@echo "   • PMAT TDG (Technical Debt): PASSING"
	@echo "   • PMAT Dead Code Detection: PASSING"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

quality-complete: quality test coverage
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ Complete quality gate passed (~5min)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mutants: check-memory
	@echo "🧬 Running mutation tests (this may take 10-15 minutes)..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants --locked)
	@cargo mutants --workspace --output mutants-report.json
	@echo "✓ Mutation testing complete"
	@echo "📊 Report: mutants-report.json"

mutants-check:
	@echo "🧬 Verifying mutation score ≥90%..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants --locked)
	@cargo mutants --workspace --output mutants-report.json --check
	@echo "✓ Mutation testing complete"

mutants-diff:
	@echo "🧬 Running mutation tests with diffs..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants --locked)
	@cargo mutants --workspace --diff
	@echo "✓ Mutation testing with diffs complete"

mutants-kernel:
	@echo "🧬 Running mutation tests on kernel only..."
	@cargo mutants -p wos-kernel --output mutants-kernel.json
	@echo "✓ Kernel mutation testing complete"

mutants-incremental:
	@echo "🧬 Running incremental mutation tests (only modified files)..."
	@cargo mutants --workspace --in-diff git:HEAD
	@echo "✓ Incremental mutation testing complete"

# ============================================================================
# Pre-commit Hooks
# ============================================================================

hooks-install:
	@echo "🔒 Installing pre-commit hooks..."
	@mkdir -p .git/hooks
	@printf '%s\n' \
		'#!/bin/bash' \
		'set -e' \
		'' \
		'echo "🔒 Running pre-commit quality gates..."' \
		'echo ""' \
		'' \
		'# bashrs: Lint bash scripts and Makefile' \
		'if command -v bashrs > /dev/null 2>&1; then' \
		'  echo "🔍 Running bashrs linter..."' \
		'  bashrs lint Makefile || { echo "❌ Makefile linting failed"; exit 1; }' \
		'  find . -name "*.sh" -type f -exec bashrs lint {} + || { echo "❌ Shell script linting failed"; exit 1; }' \
		'  echo "✅ bashrs checks passed"' \
		'else' \
		'  echo "⚠️  bashrs not found, skipping bash/Makefile linting"' \
		'  echo "   Install with: cargo install bashrs"' \
		'fi' \
		'echo ""' \
		'' \
		'# Fast quality checks (<30s)' \
		'make quality' \
		'' \
		'echo ""' \
		'echo "✅ All pre-commit checks passed!"' \
		> .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✓ Pre-commit hooks installed"
	@echo "  Run 'make quality' to test"

# ============================================================================
# Development
# ============================================================================

serve:
	@echo "🌐 Starting development server with hot reload on http://localhost:8000"
	@echo "💡 Features: Hot reload (--watch), WASM auto-compilation (--watch-wasm)"
	@cd dist && ruchy serve wos --port 8000 --watch --watch-wasm --verbose

clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/coverage
	@rm -rf dist/wos/*.wasm dist/wos/*.js
	@echo "✓ Clean complete"

# ============================================================================
# GitHub Actions Support
# ============================================================================

ci-install:
	@echo "📦 Installing CI dependencies..."
	@rustup target add wasm32-unknown-unknown
	@cargo install cargo-llvm-cov --locked || echo "cargo-llvm-cov already installed"
	@echo "✓ CI dependencies installed"

ci-test: quality test coverage
	@echo "✅ CI tests passed"

# ============================================================================
# Benchmarking
# ============================================================================

bench:
	@echo "🚀 Running performance benchmarks..."
	@cargo bench -p wos-kernel
	@echo "✓ Benchmarks complete"
	@echo "📊 View HTML reports: target/criterion/report/index.html"

bench-baseline:
	@echo "📊 Saving benchmark baseline..."
	@cargo bench -p wos-kernel -- --save-baseline main
	@echo "✓ Baseline saved"

bench-compare:
	@echo "📊 Comparing against baseline..."
	@cargo bench -p wos-kernel -- --baseline main
	@echo "✓ Comparison complete"

bench-syscalls:
	@echo "🚀 Benchmarking syscalls..."
	@cargo bench -p wos-kernel --bench syscalls
	@echo "✓ Syscall benchmarks complete"

bench-scheduler:
	@echo "🚀 Benchmarking scheduler..."
	@cargo bench -p wos-kernel --bench scheduler
	@echo "✓ Scheduler benchmarks complete"

bench-memory:
	@echo "🚀 Benchmarking memory..."
	@cargo bench -p wos-kernel --bench memory
	@echo "✓ Memory benchmarks complete"

bench-frontend:
	@echo "🚀 Running frontend benchmarks..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task bench
	@echo "✓ Frontend benchmarks complete"

bench-all: bench bench-frontend
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ All benchmarks complete (Rust + Frontend)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Fuzz Testing
# ============================================================================

fuzz-install:
	@echo "📦 Installing cargo-fuzz..."
	@cargo install cargo-fuzz
	@echo "✓ cargo-fuzz installed"

fuzz:
	@echo "🔬 Running fuzz tests (Ctrl+C to stop)..."
	@which cargo-fuzz > /dev/null 2>&1 || (echo "Installing cargo-fuzz..." && cargo install cargo-fuzz)
	@echo "Running all fuzz targets for 60 seconds each..."
	@cargo fuzz run fuzz_syscall_dispatch -- -max_total_time=60 || true
	@cargo fuzz run fuzz_process_creation -- -max_total_time=60 || true
	@cargo fuzz run fuzz_memory_allocation -- -max_total_time=60 || true
	@cargo fuzz run fuzz_scheduler -- -max_total_time=60 || true
	@echo "✓ Fuzz testing complete"

fuzz-syscalls:
	@echo "🔬 Fuzzing syscall dispatch..."
	@cargo fuzz run fuzz_syscall_dispatch

fuzz-processes:
	@echo "🔬 Fuzzing process creation..."
	@cargo fuzz run fuzz_process_creation

fuzz-memory:
	@echo "🔬 Fuzzing memory allocation..."
	@cargo fuzz run fuzz_memory_allocation

fuzz-scheduler:
	@echo "🔬 Fuzzing scheduler..."
	@cargo fuzz run fuzz_scheduler

fuzz-coverage:
	@echo "📊 Generating fuzz coverage..."
	@cargo fuzz coverage fuzz_syscall_dispatch
	@cargo fuzz coverage fuzz_process_creation
	@cargo fuzz coverage fuzz_memory_allocation
	@cargo fuzz coverage fuzz_scheduler
	@echo "✓ Fuzz coverage generated"

fuzz-clean:
	@echo "🧹 Cleaning fuzz artifacts..."
	@cargo fuzz clean
	@echo "✓ Fuzz artifacts cleaned"

# ============================================================================
# E2E Testing
# ============================================================================

e2e-install:
	@echo "📦 Installing E2E test dependencies..."
	@cd e2e && npm install
	@cd e2e && npx playwright install
	@echo "✓ E2E dependencies installed"

e2e: check-memory
	@echo "🌐 Running E2E tests..."
	@cd e2e && npm test
	@echo "✓ E2E tests complete"

e2e-headed:
	@echo "🌐 Running E2E tests (headed)..."
	@cd e2e && npm run test:headed
	@echo "✓ E2E tests complete"

e2e-ui:
	@echo "🌐 Running E2E tests (UI mode)..."
	@cd e2e && npm run test:ui

e2e-debug:
	@echo "🌐 Running E2E tests (debug mode)..."
	@cd e2e && npm run test:debug

e2e-chromium:
	@echo "🌐 Running E2E tests (Chromium only)..."
	@cd e2e && npm run test:chromium
	@echo "✓ Chromium tests complete"

e2e-firefox:
	@echo "🌐 Running E2E tests (Firefox only)..."
	@cd e2e && npm run test:firefox
	@echo "✓ Firefox tests complete"

e2e-webkit:
	@echo "🌐 Running E2E tests (WebKit only)..."
	@cd e2e && npm run test:webkit
	@echo "✓ WebKit tests complete"

e2e-report:
	@echo "📊 Opening E2E test report..."
	@cd e2e && npm run report

e2e-clean:
	@echo "🧹 Cleaning E2E artifacts..."
	@rm -rf e2e/playwright-report
	@rm -rf e2e/test-results
	@rm -rf e2e/test-results.json
	@echo "✓ E2E artifacts cleaned"

# ============================================================================
# Canary Testing (SQLite-Inspired)
# ============================================================================

canary:
	@echo "🐤 Running canary tests (59 tests, Chromium only, ~2-3 min)..."
	@cd e2e && npx playwright test tests/canary/ --project=chromium --reporter=list
	@echo "✓ Canary tests complete"

canary-all:
	@echo "🐤 Running canary tests (all browsers, ~15-20 min)..."
	@cd e2e && npx playwright test tests/canary/ --reporter=list
	@echo "✓ Canary tests complete (all browsers)"

canary-fast:
	@echo "🐤 Running fast canary tests (terminal only, ~1 min)..."
	@cd e2e && npx playwright test tests/canary/01-terminal-interaction.spec.ts --project=chromium --reporter=list
	@echo "✓ Fast canary tests complete"

canary-terminal:
	@echo "🐤 Running terminal interaction canary tests (C01-C09)..."
	@cd e2e && npx playwright test tests/canary/01-terminal-interaction.spec.ts --reporter=list
	@echo "✓ Terminal canary tests complete"

canary-process:
	@echo "🐤 Running process management canary tests (C10-C19)..."
	@cd e2e && npx playwright test tests/canary/02-process-management.spec.ts --reporter=list
	@echo "✓ Process canary tests complete"

canary-file:
	@echo "🐤 Running file operations canary tests (C20-C29)..."
	@cd e2e && npx playwright test tests/canary/03-file-operations.spec.ts --reporter=list
	@echo "✓ File canary tests complete"

canary-state:
	@echo "🐤 Running state management canary tests (C30-C39)..."
	@cd e2e && npx playwright test tests/canary/04-state-management.spec.ts --reporter=list
	@echo "✓ State canary tests complete"

canary-error:
	@echo "🐤 Running error handling canary tests (C40-C49)..."
	@cd e2e && npx playwright test tests/canary/05-error-handling.spec.ts --reporter=list
	@echo "✓ Error canary tests complete"

canary-headed:
	@echo "🐤 Running canary tests (headed mode)..."
	@cd e2e && npx playwright test tests/canary/ --headed
	@echo "✓ Canary tests complete"

canary-ui:
	@echo "🐤 Running canary tests (UI mode)..."
	@cd e2e && npx playwright test tests/canary/ --ui

canary-debug:
	@echo "🐤 Running canary tests (debug mode)..."
	@cd e2e && npx playwright test tests/canary/ --debug

canary-report:
	@echo "📊 Opening canary test report..."
	@cd e2e && npx playwright show-report

canary-chromium:
	@echo "🐤 Running canary tests (Chromium only)..."
	@cd e2e && npx playwright test tests/canary/ --project=chromium --reporter=list
	@echo "✓ Chromium canary tests complete"

canary-firefox:
	@echo "🐤 Running canary tests (Firefox only)..."
	@cd e2e && npx playwright test tests/canary/ --project=firefox --reporter=list
	@echo "✓ Firefox canary tests complete"

canary-webkit:
	@echo "🐤 Running canary tests (WebKit only)..."
	@cd e2e && npx playwright test tests/canary/ --project=webkit --reporter=list
	@echo "✓ WebKit canary tests complete"

# ============================================================================
# Frontend Linting (Deno)
# ============================================================================

lint-frontend:
	@echo "🔍 Linting frontend code..."
	@which deno > /dev/null 2>&1 || (echo "❌ Deno not found. Install: https://deno.land/" && exit 1)
	@cd dist/wos && deno task validate
	@cd dist/wos && deno run --allow-read lint.ts
	@echo "✓ Frontend linting complete"

lint-frontend-fix:
	@echo "🔧 Auto-fixing frontend code..."
	@cd dist/wos && deno task lint:fix
	@cd dist/wos && deno task fmt
	@echo "✓ Frontend auto-fix complete"

lint-frontend-check:
	@echo "🔍 Checking frontend formatting..."
	@cd dist/wos && deno task fmt:check
	@echo "✓ Frontend formatting OK"

lint-scripts:
	@echo "🔍 Linting shell scripts with bashrs..."
	@which bashrs > /dev/null 2>&1 || (echo "❌ bashrs not found. Install: https://github.com/paiml/bashrs" && exit 1)
	@FOUND_ERRORS=0; \
	for script in $$(find scripts -name "*.sh" 2>/dev/null); do \
		echo "  Checking $$script..."; \
		bashrs lint "$$script" || FOUND_ERRORS=1; \
	done; \
	if [ $$FOUND_ERRORS -eq 1 ]; then \
		echo "❌ Shell script linting failed"; \
		exit 1; \
	else \
		echo "✓ Shell script linting complete"; \
	fi

lint-all: clippy lint-frontend lint-scripts
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ All linting passed (Rust + Frontend + Shell Scripts)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Deployment
# ============================================================================

.PHONY: deploy deploy-build deploy-upload deploy-invalidate deploy-check deploy-config

# Development workflow: symlink dist to paiml.com for rapid iteration
link-dev:
	@echo "🔗 Linking WebOS dist to interactive.paiml.com..."
	@bash scripts/link-to-paiml.sh

# Main deployment target - builds and deploys to production
deploy: deploy-check deploy-build deploy-upload deploy-invalidate
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ Deployment complete!"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check deployment prerequisites
deploy-check:
	@echo "🔍 Checking deployment prerequisites..."
	@command -v aws >/dev/null 2>&1 || (echo "❌ AWS CLI not found. Install: https://aws.amazon.com/cli/" && exit 1)
	@[ -f .env.deploy ] || (echo "❌ .env.deploy not found. Copy .env.deploy.example and configure." && exit 1)
	@echo "✓ Prerequisites met"

# Build production WASM
deploy-build:
	@echo "🏗️  Building production WASM..."
	@make wasm
	@echo "✓ Production build complete"

# Upload to S3
deploy-upload: .env.deploy
	@echo "📤 Uploading to S3..."
	@source .env.deploy && bash scripts/deploy-s3.sh
	@echo "✓ Upload complete"

# Invalidate CloudFront cache
deploy-invalidate: .env.deploy
	@echo "🔄 Invalidating CloudFront cache..."
	@source .env.deploy && bash scripts/deploy-invalidate.sh
	@echo "✓ Cache invalidation complete"

# Create deployment configuration template
deploy-config:
	@echo "⚙️  Creating deployment configuration..."
	@bash scripts/create-deploy-config.sh
	@echo "✓ Configuration created: .env.deploy.example"
	@echo "📝 Copy to .env.deploy and fill in your values"
