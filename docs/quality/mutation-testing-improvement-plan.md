# Mutation Testing Improvement Plan

**Date:** October 18, 2025
**Current Score:** 83.8% (680 caught / 811 tested)
**Target Score:** 90%+ (≥729 caught / 811 tested)
**Gap:** +6.2 percentage points (+49 mutants to catch)

## Executive Summary

Mutation testing results show **82 missed mutants** preventing us from meeting the 90% quality gate requirement. This document provides a detailed analysis and improvement plan to achieve the target score.

## Current Status

### Test Results
```
811 mutants tested in 22m 41s:
- 680 caught (83.8%)
- 82 missed (10.1%)
- 47 unviable (5.8%)
- 2 timeouts (0.2%)
```

### Quality Gates Status
| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Mutation Score | 83.8% | ≥90% | ⚠️ FAIL |
| Unit Tests | 547/547 | 100% | ✅ PASS |
| E2E Tests (Chromium) | 165/165 | - | ✅ PASS |
| Code Coverage | ~94% | ≥85% | ✅ PASS |

## Missed Mutants by File

| File | Missed Count | % of Total | Priority |
|------|--------------|------------|----------|
| `wos/src/lib.rs` | 29 | 35.4% | **HIGH** |
| `shared/src/pipeline.rs` | 12 | 14.6% | **HIGH** |
| `userspace/src/programs.rs` | 8 | 9.8% | MEDIUM |
| `userspace/src/vim/command.rs` | 6 | 7.3% | MEDIUM |
| `shared/src/context.rs` | 6 | 7.3% | MEDIUM |
| `kernel/src/memory.rs` | 5 | 6.1% | MEDIUM |
| `wos/src/quality.rs` | 4 | 4.9% | LOW |
| `shared/src/parser.rs` | 3 | 3.7% | LOW |
| `kernel/src/trace.rs` | 3 | 3.7% | LOW |
| `userspace/src/init.rs` | 2 | 2.4% | LOW |
| `kernel/src/state.rs` | 2 | 2.4% | LOW |
| `userspace/src/vim/state.rs` | 1 | 1.2% | LOW |
| `userspace/src/vim/buffer.rs` | 1 | 1.2% | LOW |
| **TOTAL** | **82** | **100%** | - |

## Detailed Analysis

### 1. wos/src/lib.rs (29 mutants - 35.4%)

**Functions Affected:**
- `parse_assignment` (~15 mutants) - Variable assignment parsing
- `execute_pipeline` (~6 mutants) - Pipeline execution logic
- `handle_export` (~4 mutants) - Export command handling
- `expand_variables` (~2 mutants) - Variable expansion
- `cmd_state` (~2 mutants) - State command logic

**Common Mutation Types:**
- `replace == with !=` in conditionals
- `replace || with &&` in boolean expressions
- `delete !` (negation removal)
- `replace += with -= or *=` in arithmetic

**Root Cause:**
Tests don't cover all edge cases in parsing logic, particularly:
- Invalid variable assignments
- Edge cases in boolean conditions
- Error paths when conditions fail

**Recommended Actions:**
1. Add tests for invalid `VAR=value` patterns
2. Test all boolean condition combinations
3. Add boundary tests for string parsing
4. Test negated conditions explicitly

### 2. shared/src/pipeline.rs (12 mutants - 14.6%)

**Functions Affected:**
- `extract_redirections` (~5 mutants) - Redirection parsing
- `extract_filename` (~2 mutants) - Filename extraction
- `update_quote_state` (~2 mutants) - Quote state tracking
- `split_by_operators` (~2 mutants) - Operator splitting
- `Pipeline::is_simple` (~1 mutant) - Simple pipeline check

**Common Mutation Types:**
- `replace match guard !in_quotes with true/false`
- `delete !` in quote handling
- `replace && with ||` in complex conditions

**Root Cause:**
Quote handling logic has many edge cases not fully tested:
- Quotes in different contexts
- Nested quotes
- Escape sequences within quotes
- Quote state transitions

**Recommended Actions:**
1. Add comprehensive quote edge case tests
2. Test all quote state transitions
3. Add tests for quotes in operators
4. Test escaped quotes

### 3. userspace/src/programs.rs (8 mutants - 9.8%)

**Functions Affected:**
- `Vim::process_input` (~4 mutants) - Vim input processing
- `Vim::render_screen` (~2 mutants) - Vim screen rendering
- `ps_main_loop` (~1 mutant) - Process list loop
- `ls_main_loop` (~1 mutant) - File list loop

**Common Mutation Types:**
- `replace == with !=` in character matching
- `replace || with &&` in mode checks
- `delete !` in boolean checks

**Root Cause:**
Vim editor logic has incomplete test coverage for:
- Edge cases in key handling
- Mode transitions
- Screen rendering conditions

**Recommended Actions:**
1. Add tests for all Vim key combinations
2. Test all mode transitions
3. Add boundary tests for cursor movement
4. Test screen rendering edge cases

### 4. Remaining Files (<6 mutants each)

**Files:**
- `userspace/src/vim/command.rs` (6) - Cursor arithmetic
- `shared/src/context.rs` (6) - Equality checks
- `kernel/src/memory.rs` (5) - Boundary conditions
- `wos/src/quality.rs` (4) - SARIF generation
- `shared/src/parser.rs` (3) - Escape sequences
- `kernel/src/trace.rs` (3) - Getter return values
- Others (7 total) - Misc edge cases

**Recommended Actions:**
1. Focus on high-value files first (>5 mutants)
2. Add boundary condition tests
3. Test equality methods explicitly
4. Add error path coverage

## Implementation Plan

### Phase 1: High Priority Files (49 mutants - 59.8%)

**Week 1: wos/src/lib.rs (29 mutants)**
- Day 1-2: parse_assignment tests (15 mutants)
- Day 3: execute_pipeline tests (6 mutants)
- Day 4: handle_export tests (4 mutants)
- Day 5: Remaining functions (4 mutants)

**Week 2: shared/src/pipeline.rs (12 mutants)**
- Day 1-2: extract_redirections tests (5 mutants)
- Day 3: Quote handling tests (4 mutants)
- Day 4: Remaining functions (3 mutants)

**Week 3: userspace/src/programs.rs (8 mutants)**
- Day 1-2: Vim::process_input tests (4 mutants)
- Day 3: Vim::render_screen tests (2 mutants)
- Day 4: Loop tests (2 mutants)

**Expected Improvement:** +49 mutants caught → **89.8% score**

### Phase 2: Medium Priority Files (20 mutants - 24.4%)

**Week 4: Remaining Medium Files**
- Day 1: userspace/src/vim/command.rs (6 mutants)
- Day 2: shared/src/context.rs (6 mutants)
- Day 3: kernel/src/memory.rs (5 mutants)
- Day 4: wos/src/quality.rs (4 mutants)

**Expected Improvement:** +20 mutants caught → **93.3% score**

### Phase 3: Low Priority Files (13 mutants - 15.9%)

**Week 5: Cleanup**
- Day 1-2: All remaining files (13 mutants)
- Day 3-4: Re-run mutation testing
- Day 5: Final adjustments

**Expected Improvement:** +13 mutants caught → **95.6% score**

## Testing Strategy

### 1. Edge Case Testing
For each missed mutant, identify the edge case not covered:
```rust
// Example: Mutant changed == to !=
// Original: if var_name == "export"
// Mutant: if var_name != "export"
// Missing test: Case where var_name is NOT "export"

#[test]
fn test_parse_assignment_non_export_variable() {
    let input = "FOO=bar";
    let result = parse_assignment(input);
    // Assert it's treated as regular assignment, not export
}
```

### 2. Boundary Condition Testing
Test all boundary values:
```rust
#[test]
fn test_cursor_at_line_boundary() {
    let mut vim = Vim::new();
    vim.cursor.line = 0;  // First line
    vim.move_up();
    assert_eq!(vim.cursor.line, 0);  // Should stay at 0
}
```

### 3. Boolean Logic Testing
Test all combinations of boolean conditions:
```rust
#[test]
fn test_quote_handling_all_combinations() {
    // in_single_quote=true, in_double_quote=true
    // in_single_quote=true, in_double_quote=false
    // in_single_quote=false, in_double_quote=true
    // in_single_quote=false, in_double_quote=false
}
```

### 4. Negation Testing
Test both positive and negative cases:
```rust
#[test]
fn test_empty_string_handling() {
    assert!(is_empty(""));     // True case
    assert!(!is_empty("x"));   // False case (tests negation)
}
```

## Estimation

### Time Investment
- Phase 1 (High Priority): ~3 weeks (15 work days)
- Phase 2 (Medium Priority): ~1 week (5 work days)
- Phase 3 (Low Priority): ~1 week (5 work days)
- **Total:** ~5 weeks (25 work days)

### Effort Distribution
- Writing tests: 60% (15 days)
- Running mutation testing: 20% (5 days)
- Analysis and iteration: 15% (3.75 days)
- Documentation: 5% (1.25 days)

## Success Metrics

### Primary Goal
- **Mutation Score ≥ 90%** (≥729 caught mutants)

### Secondary Goals
- Zero missed mutants in parse_assignment
- Zero missed mutants in execute_pipeline
- <3 missed mutants per file

### Stretch Goals
- Mutation Score ≥ 95% (≥770 caught mutants)
- Zero missed mutants in critical path code
- Comprehensive edge case documentation

## Risks and Mitigation

### Risk 1: Time Investment
**Risk:** 5 weeks is significant time investment
**Mitigation:** Prioritize high-value files first, stop at 90% if time-constrained

### Risk 2: Test Maintenance
**Risk:** More tests = more maintenance burden
**Mitigation:** Write clear, focused tests with good names

### Risk 3: Unviable Mutants
**Risk:** Some mutants may be semantically equivalent
**Mitigation:** Use `#[mutants::skip]` for known false positives

### Risk 4: Mutation Testing Time
**Risk:** Each run takes ~23 minutes
**Mitigation:** Run incrementally per file using `--file` flag

## Incremental Approach

Instead of full 5-week plan, use iterative approach:

### Sprint 1: Quick Wins (1 week)
- Focus on parse_assignment only (15 mutants)
- Target: 85.6% → 87.6% (+2 percentage points)
- Low risk, high confidence

### Sprint 2: Pipeline Logic (1 week)
- Focus on shared/src/pipeline.rs (12 mutants)
- Target: 87.6% → 89.1% (+1.5 percentage points)
- Moderate risk, medium confidence

### Sprint 3: Final Push (1 week)
- Focus on remaining high-value files (22 mutants)
- Target: 89.1% → 91.8% (+2.7 percentage points)
- High risk, meet target with buffer

**Total Time:** 3 weeks to reach 90%+ (vs 5 weeks for 95%)

## Recommendations

### Immediate Next Steps (This Session)
1. ✅ Complete mutation testing analysis (DONE)
2. ✅ Create this improvement plan document (DONE)
3. Start Sprint 1: parse_assignment tests
4. Write 5-10 targeted tests
5. Re-run mutation testing on wos/src/lib.rs only
6. Iterate until parse_assignment mutants are caught

### Long-Term Strategy
1. Integrate mutation testing into CI/CD
2. Set mutation score requirement for PRs
3. Run incrementally (per-file) to save time
4. Document known unviable mutants
5. Review missed mutants quarterly

## Conclusion

Achieving 90% mutation score is feasible with focused effort on 3 high-priority files containing 49 of 82 missed mutants. The incremental 3-sprint approach provides a balance between quality improvement and time investment.

**Recommended Path:** Start with Sprint 1 (parse_assignment) this session, continue with Sprints 2-3 in future sessions.

---

**Status:** Planning Complete
**Next Action:** Begin Sprint 1 - parse_assignment tests
**Estimated Time to 90%:** 3 weeks (3 sprints)
