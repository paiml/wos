# Changelog

All notable changes to the WOS project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- System Monitor panel cutoff issue at 1080p resolution - all 4 metrics now display without vertical overflow
- Panel layout optimization: reduced spacing, padding, and margins across all panels for better viewport utilization
- Playwright test suite: updated 3 panel layout tests to match current HTML structure
  - Fixed file actions visibility checks (changed from `.toBeVisible()` to `.count()` for disabled elements)
  - Rewrote file actions integration test to verify correct structure
  - All 6 panel layout optimization tests now passing

### Changed
- CSS spacing optimizations in `dist/wos/style.css`:
  - `.file-manager` gap: 20px → 10px
  - `.file-panel-header` padding: 12px 20px → 8px 15px
  - `.panel-content` padding: 15px → 10px
  - `.system-monitor-grid` gap: 15px → 10px, padding: 15px → 0
  - `.monitor-card` padding: 15px → 10px
  - `.monitor-value` font-size: 28px → 22px
  - `.monitor-label` and `.monitor-value` margins reduced to 5px

## [0.1.0] - 2025-10-18

### Added
- Initial WOS implementation with browser-based terminal interface
- System Monitor panel with CPU, Memory, Processes, and Syscalls metrics
- Filesystem panel with integrated file management
- Process List panel
- Memory Map panel
- System Call Trace panel
- Vim editor modal integration
- Configuration management system
- Comprehensive E2E test suite with Playwright
- Quality gates: formatting, linting, unit tests, complexity analysis
- Pre-commit hooks for code quality
