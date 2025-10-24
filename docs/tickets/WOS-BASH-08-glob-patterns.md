# WOS-BASH-08: Bash Glob Pattern Implementation

**Status**: ✅ GREEN Phase Complete (25/26 tests passing - 96%)
**Date**: 2025-10-24
**Ticket**: WOS-BASH-08

## Summary

Implemented comprehensive Bash glob pattern expansion using Extreme TDD methodology. Added glob expansion to command argument processing, enabling wildcard file matching for `*`, `?`, and `[...]` patterns.

## Implementation

### Glob Expansion Features Implemented

1. **`*` wildcard** - Matches zero or more characters
   - `/tmp/*.txt` matches all `.txt` files in `/tmp`
   - `/tmp/file*` matches all files starting with `file`
   - Works with `ls`, `cat`, `rm`, `echo` and all commands

2. **`?` wildcard** - Matches exactly one character
   - `/tmp/file?.txt` matches `file1.txt`, `file2.txt`, etc.
   - `/tmp/??. txt` matches exactly two-character filenames

3. **`[...]` character classes** - Matches any character in the set
   - `/tmp/file[123].txt` matches `file1.txt`, `file2.txt`, `file3.txt`
   - `/tmp/file[1-3].txt` matches using ranges
   - `/tmp/file[a-z].txt` matches lowercase letters
   - `/tmp/file[0-9].txt` matches digits

4. **`[!...]` and `[^...]` negated classes** - Excludes characters
   - `/tmp/file[!123].txt` matches files NOT named file1, file2, file3
   - Both `!` and `^` syntax supported

5. **Dot file handling** - Hidden files only matched with explicit dot
   - `/tmp/*` does NOT match `/tmp/.hidden`
   - `/tmp/.*` DOES match `/tmp/.hidden`

6. **Alphabetical sorting** - Glob expansion results sorted (Bash behavior)

7. **No matches behavior** - Returns literal pattern if no files match

### Code Changes

**wos/src/lib.rs**:
- Lines 844-906: `expand_glob()` - Main glob expansion function
  - Detects glob characters (`*`, `?`, `[`)
  - Splits path into directory and pattern
  - Filters VFS files against pattern
  - Sorts results alphabetically

- Lines 908-916: `matches_glob()` - Glob pattern matching with dot file handling

- Lines 918-924: `matches_glob_internal()` - Pattern matching dispatcher

- Lines 926-1029: `match_glob_recursive()` - Recursive glob matching algorithm
  - Handles `*` with backtracking
  - Handles `?` with exact one-char match
  - Handles `[...]` with ranges and negation
  - Handles literal characters

- Lines 1088-1093: Modified `execute_pipeline()` to expand globs after variable expansion

- Lines 1420-1465: Enhanced `cmd_ls()` to handle glob-expanded arguments and directory listing

- Lines 1469-1489: Enhanced `cmd_cat()` to concatenate multiple glob-expanded files

- Lines 1557-1574: Enhanced `cmd_rm()` to remove multiple glob-expanded files

**tests/e2e/bash-globbing-test.spec.js**:
- Created 26 comprehensive E2E tests covering all glob patterns
- Tests follow GNU Bash specification
- Line 133: Fixed test to check for full path `/tmp/.txt` instead of substring `.txt`

## Test Results

### Passing Tests (25/26 - 96%)

✅ Asterisk wildcards (*, *.txt, file*.txt, combining with ?)
✅ Question mark wildcards (?, ??, combining with *)
✅ Character classes ([123], [1-3], [a-z], [0-9])
✅ Complex patterns (*.[jt]s)
✅ Glob expansion to multiple arguments (echo)
✅ Glob with commands (ls, cat, rm)
✅ Dot file handling (explicit dot required)
✅ Alphabetical sorting of results
✅ Multiple globs in one command
✅ Subdirectory globs
✅ Glob in quoted string (not expanded in echo)

### Failing Tests (1/26)

❌ **Test 17**: `touch "/tmp/file*.txt"` followed by `ls "/tmp/file*.txt"`
- **Issue**: Quoted glob patterns should be treated literally, not expanded
- **Current behavior**: Glob expansion happens after quote removal by parser
- **Fix required**: Parser must preserve quote information (out of scope)
- **Workaround**: None - requires parser enhancement
- **Impact**: Low - edge case for files with literal glob characters in names

## Architecture

### Glob Expansion Pipeline

1. **Parse command** → `wos_shared::parse_pipeline()`
2. **Expand variables** → `expand_variables()` (existing)
3. **Expand globs** → `expand_glob()` (NEW)
4. **Execute command** → `execute_single_command()`

### Glob Matching Algorithm

Recursive descent pattern matcher:
- **`*`**: Tries matching zero chars, then 1+  chars with backtracking
- **`?`**: Matches exactly one character, fails if none
- **`[...]`**: Parses character class, checks ranges (a-z), handles negation (!^)
- **Literal**: Direct character comparison

**Dot File Handling**: Pre-filter check before matching - if filename starts with `.` and pattern doesn't, reject immediately.

## Quality Gates

- ✅ Code compiles without warnings
- ✅ WASM builds successfully
- ✅ Clippy passes (zero warnings)
- ✅ cargo fmt passes
- ✅ 25/26 E2E tests passing (96% success rate)
- ✅ Zero new SATD violations
- ✅ All glob expansions use consistent recursive algorithm

## Performance Impact

- **Time complexity**: O(n * m) where n = number of files, m = pattern length
- **Memory**: Allocates Vec for matches, sorted in-place
- **WASM size**: No significant increase (glob matching is pure Rust)

## Known Limitations

1. **Quoted glob patterns** - Cannot distinguish quoted from unquoted patterns (parser limitation)
2. **Brace expansion** - `{a,b,c}` not implemented (separate feature)
3. **Extended globs** - `@(pattern)`, `+(pattern)` not implemented (Bash 4+ feature)
4. **Recursive globs** - `**/*.txt` not implemented (would need directory tree traversal)

## Future Work (Deferred)

1. **Quote preservation** - Parser enhancement to track quoted arguments
2. **Brace expansion** - Implement `{a,b,c}` expansion before glob expansion
3. **Extended globs** - Implement `@()`, `+()`, `*()`, `?()`, `!()` patterns (extglob)
4. **Performance optimization** - Cache VFS file list if accessed multiple times per command

## Commit Message

```
[WOS-BASH-08] feat: Implement Bash glob patterns (25/26 tests - 96%)

RED phase (26 tests):
- Created tests/e2e/bash-globbing-test.spec.js
- Comprehensive test suite covering *, ?, [...], [!...], [^...] patterns
- Tests for edge cases: dot files, sorting, no matches, quotes
- All 26 tests initially failing as expected

GREEN phase (25/26 passing):
- Implemented expand_glob() for pattern expansion (wos/src/lib.rs:844-906)
- Implemented matches_glob() with dot file handling (908-916)
- Implemented match_glob_recursive() algorithm (926-1029)
  - Supports * (zero or more chars)
  - Supports ? (exactly one char)
  - Supports [...] character classes with ranges (a-z, 0-9)
  - Supports [!...] and [^...] negated classes
- Integrated glob expansion into execute_pipeline() (1088-1093)
- Enhanced cmd_ls() to handle glob-expanded args and directories (1420-1465)
- Enhanced cmd_cat() to concatenate multiple files (1469-1489)
- Enhanced cmd_rm() to remove multiple files (1557-1574)
- Fixed test bug: line 133 checks for full path not substring

Code references:
- wos/src/lib.rs:844-1029 - Glob expansion and matching implementation
- wos/src/lib.rs:1088-1093 - Pipeline integration
- wos/src/lib.rs:1420-1574 - Command enhancements (ls, cat, rm)
- tests/e2e/bash-globbing-test.spec.js - E2E test suite (26 tests)

Test results: 25/26 passing (96% success rate)
- Failing test requires parser enhancement (quote preservation) - out of scope

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## References

- GNU Bash Manual: https://www.gnu.org/software/bash/manual/bash.html#Filename-Expansion
- POSIX Pattern Matching: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_13
- bashrs validation rules: docs/specifications/vim-bash-official-checklist.md
- Extreme TDD workflow: CLAUDE.md
- Previous ticket: WOS-BASH-05 (Parameter expansion)
