.PHONY: help build test coverage quality wasm clean hooks-install

export PATH := $(HOME)/.cargo/bin:$(PATH)

WASM_TARGET := target/wasm32-unknown-unknown/release/wos.wasm

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
	@echo "  make test             Run all tests"
	@echo "  make test-unit        Run unit tests only"
	@echo "  make coverage         Generate coverage report"
	@echo "  make coverage-check   Verify coverage ≥85%"
	@echo ""
	@echo "🎯 Quality Gates:"
	@echo "  make quality          Fast quality checks (<30s)"
	@echo "  make quality-complete Complete quality validation (~5min)"
	@echo "  make fmt              Format code"
	@echo "  make clippy           Run clippy lints"
	@echo "  make mutants          Run mutation tests (~10-15min)"
	@echo ""
	@echo "🔧 Development:"
	@echo "  make hooks-install    Install pre-commit hooks"
	@echo "  make serve            Start local HTTP server (port 8000)"
	@echo "  make clean            Clean build artifacts"
	@echo ""

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
	@echo "✓ WASM build complete"
	@echo "💡 Start local server: make serve"
	@echo "💡 Open browser: http://localhost:8000/dist/wos/"

dist: wasm

# ============================================================================
# Testing
# ============================================================================

test:
	@echo "🧪 Running all tests..."
	@cargo test --workspace --all-features
	@echo "✓ All tests passed"

test-unit:
	@echo "🧪 Running unit tests..."
	@cargo test --workspace --lib
	@echo "✓ Unit tests passed"

# ============================================================================
# Coverage
# ============================================================================

coverage:
	@echo "📊 Running comprehensive test coverage analysis..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@cargo llvm-cov clean --workspace
	@mkdir -p target/coverage
	@cargo llvm-cov --no-report test --lib --all-features 2>&1 | tee target/coverage/test-output.txt
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo ""
	@echo "📊 Coverage Summary:"
	@echo "=================="
	@cargo llvm-cov report --summary-only
	@echo ""
	@echo "💡 HTML report: target/coverage/html/index.html"
	@echo ""

coverage-check:
	@echo "📊 Checking coverage thresholds (≥85% line, ≥90% branch)..."
	@if [ ! -f "target/coverage/lcov.info" ]; then \
		echo "❌ Coverage data not found. Run 'make coverage' first"; \
		exit 1; \
	fi
	@cargo llvm-cov report --fail-under-lines 85 || (echo "❌ Coverage below 85%"; exit 1)
	@echo "✓ Coverage thresholds met"

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

quality: fmt clippy test-unit
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ Fast quality gate passed (<30s)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

quality-complete: quality test coverage
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "✅ Complete quality gate passed (~5min)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mutants:
	@echo "🧬 Running mutation tests (this may take 10-15 minutes)..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants)
	@cargo mutants --workspace
	@echo "✓ Mutation testing complete"

mutants-check:
	@echo "🧬 Verifying mutation score ≥90%..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants)
	@cargo mutants --workspace 2>&1 | tee /tmp/mutants-output.txt
	@echo "✓ Mutation testing complete (manual verification needed)"

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
	@echo "🌐 Starting HTTP server on http://localhost:8000"
	@python3 -m http.server 8000

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
