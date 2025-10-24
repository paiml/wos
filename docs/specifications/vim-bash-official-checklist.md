# WOS Bash & Vim Implementation Checklist

**Based on**: GNU Bash Manual, Official Vim Documentation, bashrs 6.1.0 validation
**Purpose**: Comprehensive feature tracking for WOS shell and vim implementations
**Methodology**: Extreme TDD - RED → GREEN → REFACTOR with Playwright E2E tests
**Quality Gate**: bashrs validation + 100% E2E test coverage

---

## Status Legend

- ✅ **IMPLEMENTED** - Feature complete with passing tests
- 🔄 **PARTIAL** - Partially implemented, needs completion
- ❌ **MISSING** - Not implemented
- 🚫 **OUT_OF_SCOPE** - Intentionally excluded (educational OS constraints)

---

# BASH SHELL FEATURES

## 1. Control Operators & Command Chaining

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Pipe (`\|`) | ✅ | - | existing tests | Working: `ls \| grep foo` |
| Logical AND (`&&`) | ✅ | - | existing tests | Working: `cmd1 && cmd2` |
| Logical OR (`\|\|`) | ✅ | - | existing tests | Working: `cmd1 \|\| cmd2` |
| Semicolon (`;`) | ✅ | - | existing tests | Working: `cmd1; cmd2` |
| Background (`&`) | 🚫 | - | - | Out of scope: no job control |
| Pipe with stderr (`\|&`) | ❌ | - | - | Need to implement |

**bashrs Integration**: Validate pipeline safety and error handling

---

## 2. Redirection Operators

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Output redirect (`>`) | ✅ | - | existing tests | Overwrite mode |
| Append redirect (`>>`) | ✅ | - | existing tests | Append mode |
| Input redirect (`<`) | ✅ | - | existing tests | Read from file |
| Redirect stderr (`2>`) | ❌ | - | - | Need to implement |
| Redirect all (`&>`) | ❌ | - | - | Need to implement |
| Here document (`<<`) | ❌ | - | - | Multi-line input |
| Here string (`<<<`) | ❌ | - | - | Single-line input |
| File descriptor duplication | 🚫 | - | - | Advanced, out of scope |

**bashrs Integration**: Validate redirection quoting and safety

---

## 3. Quoting Mechanisms

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Escape char (`\`) | ✅ | - | parser tests | Literal next char |
| Single quotes (`'...'`) | ✅ | - | parser tests | No expansion |
| Double quotes (`"..."`) | ✅ | SC2086 | parser tests | Allow expansion |
| Variable in quotes | ✅ | SC2086 | existing tests | `"$var"` working |
| ANSI-C quoting (`$'...'`) | ❌ | - | - | `\n`, `\t` escapes |
| Locale translation (`$"..."`) | 🚫 | - | - | Out of scope |

**bashrs Rule SC2086**: "Double quote to prevent globbing and word splitting"
- WOS MUST validate all variable expansions are properly quoted
- bashrs can auto-fix unquoted variables

---

## 4. Variables & Parameter Expansion

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Variable assignment (`VAR=value`) | ✅ | - | existing tests | Basic assignment |
| Variable expansion (`$VAR`) | ✅ | SC2086 | existing tests | With quoting |
| Braced expansion (`${VAR}`) | ✅ | SC2086 | existing tests | Explicit form |
| Default value (`${VAR:-default}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |
| Assign default (`${VAR:=default}`) | 🔄 | - | bash-parameter-expansion-test.spec.js | Returns default, doesn't assign (read-only) |
| Error if unset (`${VAR:?error}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |
| Use alternate (`${VAR:+alt}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |
| String length (`${#VAR}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |
| Substring (`${VAR:offset:len}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |
| Pattern removal (`${VAR#pattern}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |
| Pattern removal (`${VAR%pattern}`) | 🔄 | - | bash-parameter-expansion-test.spec.js | Longest works, shortest has regex issue |
| Case modification (`${VAR^}`, `${VAR,,}`) | ✅ | - | bash-parameter-expansion-test.spec.js | WOS-BASH-05 |

**bashrs Integration**: Validate variable quoting, detect unset variables

---

## 5. Special Variables

| Variable | Status | bashrs Rule | Test File | Notes |
|----------|--------|-------------|-----------|-------|
| `$?` | ✅ | - | bash-special-vars-test.spec.js | Exit status (WOS-BASH-04) |
| `$$` | ✅ | - | bash-special-vars-test.spec.js | Process ID (WOS-BASH-04) |
| `$!` | 🚫 | - | - | Background PID (no job control) |
| `$0` | ✅ | - | bash-special-vars-test.spec.js | Shell name (WOS-BASH-04) |
| `$1`, `$2`, ... | 🔄 | - | bash-special-vars-test.spec.js | Positional params (needs WOS-BASH-05) |
| `$#` | 🔄 | - | bash-special-vars-test.spec.js | Argument count (needs WOS-BASH-05) |
| `$*` | 🔄 | - | bash-special-vars-test.spec.js | All args (needs WOS-BASH-05) |
| `$@` | 🔄 | - | bash-special-vars-test.spec.js | All args (needs WOS-BASH-05) |
| `$-` | 🚫 | - | - | Current options |

---

## 6. Command Substitution

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Modern form (`$(cmd)`) | ❌ | SC2046 | - | Preferred syntax |
| Legacy form (`` `cmd` ``) | ❌ | SC2046 | - | Deprecated |
| Nested substitution | ❌ | SC2046 | - | `$(cmd1 $(cmd2))` |

**bashrs Rule SC2046**: "Quote command substitution to prevent word splitting"
**bashrs Rule SC2116**: "Useless echo; just use $result directly"
- WOS should avoid `echo $(...)` patterns

---

## 7. Shell Expansions

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Brace expansion (`{a,b,c}`) | ❌ | - | - | Generate alternatives |
| Brace range (`{1..10}`) | ❌ | - | - | Numeric/alpha ranges |
| Tilde expansion (`~`) | ❌ | - | - | Home directory |
| Tilde user (`~user`) | 🚫 | - | - | Multi-user (out of scope) |
| Arithmetic expansion (`$((expr))`) | ❌ | - | - | Math evaluation |
| Glob patterns (`*.txt`) | ✅ | - | bash-globbing-test.spec.js | Filename matching (WOS-BASH-08) |
| Glob char class (`[abc]`) | ✅ | - | bash-globbing-test.spec.js | Character ranges (WOS-BASH-08) |
| Word splitting (IFS) | 🔄 | SC2086 | - | Partial: needs validation |

**bashrs Integration**: Validate globbing safety, prevent accidental expansion

---

## 8. Built-in Commands (Core)

| Command | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| `cd` | ✅ | - | prompt-test.spec.js | With path normalization |
| `pwd` | ✅ | - | prompt-test.spec.js | Current directory |
| `echo` | ✅ | SC2116 | existing tests | Output text |
| `export` | ✅ | - | existing tests | Environment vars |
| `set` | ❌ | - | - | Shell options |
| `unset` | ❌ | - | - | Remove variable |
| `readonly` | ❌ | - | - | Immutable variables |
| `local` | ❌ | - | - | Function-local vars |
| `declare/typeset` | ❌ | - | - | Variable attributes |
| `alias` | ❌ | - | - | Command shortcuts |
| `source`/`.` | ✅ | - | existing tests | Execute script |
| `eval` | 🚫 | - | - | Security risk |
| `exec` | 🚫 | - | - | Process replacement |
| `read` | ❌ | - | - | Read user input |
| `printf` | ❌ | - | - | Formatted output |
| `test`/`[` | ❌ | - | - | Conditional test |

**bashrs Safety Transforms**:
- `mkdir` → `mkdir -p` (idempotent)
- `rm` → `rm -f` (safe removal)

---

## 9. Built-in Commands (Advanced)

| Command | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| `return` | ❌ | - | - | Exit function |
| `break` | ❌ | - | - | Exit loop |
| `continue` | ❌ | - | - | Skip iteration |
| `shift` | ❌ | - | - | Rotate parameters |
| `getopts` | 🚫 | - | - | Option parsing |
| `trap` | 🚫 | - | - | Signal handling |
| `jobs` | 🚫 | - | - | Job control |
| `bg`/`fg` | 🚫 | - | - | Job control |
| `wait` | 🚫 | - | - | Process waiting |
| `kill` | ✅ | - | existing tests | Send signals |
| `times` | 🚫 | - | - | Process times |
| `ulimit` | 🚫 | - | - | Resource limits |

---

## 10. Control Structures

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| `if/then/elif/else/fi` | ❌ | - | - | Conditional execution |
| `case/esac` | ❌ | - | - | Pattern matching |
| `for var in list` | ❌ | - | - | Iterate over items |
| `for ((;;))` | ❌ | - | - | C-style loop |
| `while condition` | ❌ | - | - | Loop while true |
| `until condition` | ❌ | - | - | Loop until true |
| `[[ ]]` | ❌ | - | - | Modern test |
| `(( ))` | ❌ | - | - | Arithmetic test |
| Subshell `(...)` | ❌ | - | - | Isolated execution |
| Group `{...}` | ❌ | - | - | Current shell |

---

## 11. Functions

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Function definition | ❌ | - | - | `name() { ... }` |
| `function` keyword | ❌ | - | - | `function name { ... }` |
| Function parameters | ❌ | - | - | `$1`, `$2` in function |
| Local variables | ❌ | - | - | `local var=value` |
| Return status | ❌ | - | - | `return N` |
| Export functions | 🚫 | - | - | Advanced feature |
| Recursion | 🚫 | - | - | Stack limits |

---

## 12. Arrays

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| Indexed arrays | ❌ | - | - | `arr=(a b c)` |
| Array assignment | ❌ | - | - | `arr[0]=value` |
| Array expansion | ❌ | - | - | `${arr[@]}` |
| Array length | ❌ | - | - | `${#arr[@]}` |
| Associative arrays | 🚫 | - | - | Advanced feature |

---

## 13. Pattern Matching & Globbing

| Feature | Status | bashrs Rule | Test File | Notes |
|---------|--------|-------------|-----------|-------|
| `*` wildcard | ❌ | - | - | Match any chars |
| `?` wildcard | ❌ | - | - | Match single char |
| `[...]` char class | ❌ | - | - | Match from set |
| `[^...]` negation | ❌ | - | - | Not in set |
| Extended globbing | 🚫 | - | - | `?(pattern)`, etc. |

---

## 14. bashrs-Specific Quality Checks

| Check | Status | Implementation | Test File |
|-------|--------|----------------|-----------|
| SC2086: Unquoted variables | 🔄 | Needs validation | - |
| SC2046: Unquoted cmd substitution | ❌ | Not implemented | - |
| SC2116: Useless echo | 🔄 | Needs detection | - |
| Idempotent mkdir | ❌ | Need `mkdir -p` | - |
| Safe rm | ❌ | Need `rm -f` | - |
| Deterministic IDs | ✅ | No $RANDOM used | - |
| Auto-quoting | ❌ | Need validation | - |

**Integration Plan**: Run bashrs on all shell scripts in WOS, enforce rules in pre-commit hooks

---

# VIM EDITOR FEATURES

## 15. Vim Modes

| Mode | Status | Test File | Notes |
|------|--------|-----------|-------|
| Normal mode | ✅ | vim tests | Default mode |
| Insert mode (`i`, `a`, `o`, `O`) | ✅ | vim tests | Text insertion |
| Visual mode (`v`, `V`, `Ctrl-v`) | ❌ | - | Text selection |
| Command-line mode (`:`) | ✅ | vim tests | Ex commands |
| Replace mode (`R`) | ❌ | - | Overwrite text |

---

## 16. Movement Commands (Normal Mode)

| Command | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `h`, `j`, `k`, `l` | ✅ | vim tests | Char/line movement |
| `w`, `b`, `e` | ❌ | - | Word movement |
| `0`, `^`, `$` | ❌ | - | Line start/end |
| `gg`, `G` | ❌ | - | File start/end |
| `:[number]` | ❌ | - | Go to line |
| `%` | ❌ | - | Matching bracket |
| `f{char}`, `t{char}` | ❌ | - | Find character |
| `F{char}`, `T{char}` | ❌ | - | Find backward |
| `;`, `,` | ❌ | - | Repeat find |
| `{`, `}` | ❌ | - | Paragraph movement |
| `(`, `)` | ❌ | - | Sentence movement |
| `Ctrl-d`, `Ctrl-u` | ❌ | - | Half page scroll |
| `Ctrl-f`, `Ctrl-b` | ❌ | - | Full page scroll |

---

## 17. Editing Commands (Normal Mode)

| Command | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `i` | ✅ | vim tests | Insert before cursor |
| `a` | ❌ | - | Insert after cursor |
| `o` | ❌ | - | Open line below |
| `O` | ❌ | - | Open line above |
| `x` | ❌ | - | Delete character |
| `dd` | ❌ | - | Delete line |
| `D` | ❌ | - | Delete to end of line |
| `d{motion}` | ❌ | - | Delete with motion |
| `cc` | ❌ | - | Change line |
| `C` | ❌ | - | Change to end of line |
| `c{motion}` | ❌ | - | Change with motion |
| `yy` | ❌ | - | Yank (copy) line |
| `y{motion}` | ❌ | - | Yank with motion |
| `p` | ❌ | - | Paste after |
| `P` | ❌ | - | Paste before |
| `.` | ❌ | - | Repeat last change |
| `u` | ❌ | - | Undo |
| `Ctrl-r` | ❌ | - | Redo |
| `~` | ❌ | - | Toggle case |
| `r{char}` | ❌ | - | Replace character |
| `J` | ❌ | - | Join lines |

---

## 18. Ex Commands (Command-Line Mode)

| Command | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `:w` | ✅ | vim-error/help tests | Write file |
| `:write` | ✅ | vim-error/help tests | Write (long form) |
| `:q` | ✅ | vim-error/help tests | Quit |
| `:quit` | ✅ | vim-error/help tests | Quit (long form) |
| `:q!` | ✅ | vim-error/help tests | Quit without saving |
| `:quit!` | ✅ | vim-error/help tests | Quit without saving |
| `:wq` | ✅ | vim-error/help tests | Write and quit |
| `:x` | ✅ | vim-error/help tests | Write and quit |
| `:help` | ✅ | vim-help-test.spec.js | Show help |
| `:e {file}` | ❌ | - | Edit file |
| `:sp {file}` | ❌ | - | Horizontal split |
| `:vs {file}` | ❌ | - | Vertical split |
| `:set {option}` | ❌ | - | Set option |
| `:set no{option}` | ❌ | - | Unset option |
| `:set {opt}?` | ❌ | - | Query option |
| `:s/pattern/repl/` | ❌ | - | Substitute |
| `:%s/pattern/repl/g` | ❌ | - | Global substitute |
| `:d` | ❌ | - | Delete lines |
| `:y` | ❌ | - | Yank lines |
| `:p` | ❌ | - | Print lines |
| `:r {file}` | ❌ | - | Read file |
| `:!{cmd}` | ❌ | - | Execute shell |
| `:earlier`, `:later` | ❌ | - | Time travel |

---

## 19. Search and Replace

| Feature | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `/pattern` | ❌ | - | Search forward |
| `?pattern` | ❌ | - | Search backward |
| `n` | ❌ | - | Next match |
| `N` | ❌ | - | Previous match |
| `*` | ❌ | - | Search word under cursor |
| `#` | ❌ | - | Search backward |
| `:s/old/new/` | ❌ | - | Substitute in line |
| `:s/old/new/g` | ❌ | - | Substitute all in line |
| `:%s/old/new/g` | ❌ | - | Substitute all in file |
| `:%s/old/new/gc` | ❌ | - | Substitute with confirm |
| Case insensitive search | ❌ | - | `/pattern\c` |
| Regex patterns | ❌ | - | `.`, `*`, `\+`, etc. |

---

## 20. Visual Mode

| Command | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `v` | ❌ | - | Character-wise visual |
| `V` | ❌ | - | Line-wise visual |
| `Ctrl-v` | ❌ | - | Block-wise visual |
| `d` | ❌ | - | Delete selection |
| `c` | ❌ | - | Change selection |
| `y` | ❌ | - | Yank selection |
| `>` | ❌ | - | Indent |
| `<` | ❌ | - | Unindent |
| `gv` | ❌ | - | Reselect |

---

## 21. Registers and Marks

| Feature | Status | Test File | Notes |
|---------|--------|-----------|-------|
| Named registers (`"a-"z`) | ❌ | - | Store in register |
| Unnamed register (`""`) | ❌ | - | Default register |
| System clipboard (`"+`) | ❌ | - | OS clipboard |
| Marks (`ma-mz`) | ❌ | - | Set mark |
| Jump to mark (`'a-'z`) | ❌ | - | Go to mark |
| Jump to line (`` `a-`z ``) | ❌ | - | Go to exact position |

---

## 22. Macros

| Feature | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `q{reg}` | ❌ | - | Start recording |
| `q` | ❌ | - | Stop recording |
| `@{reg}` | ❌ | - | Play macro |
| `@@` | ❌ | - | Repeat last macro |

---

## 23. Windows and Buffers

| Feature | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `:split` | ❌ | - | Horizontal split |
| `:vsplit` | ❌ | - | Vertical split |
| `Ctrl-w w` | ❌ | - | Switch windows |
| `Ctrl-w hjkl` | ❌ | - | Navigate windows |
| `Ctrl-w q` | ❌ | - | Close window |
| `:bnext`, `:bprev` | ❌ | - | Buffer navigation |
| `:bdelete` | ❌ | - | Delete buffer |
| `:ls` | ❌ | - | List buffers |

---

## 24. Configuration

| Feature | Status | Test File | Notes |
|---------|--------|-----------|-------|
| `:set number` | ❌ | - | Show line numbers |
| `:set relativenumber` | ❌ | - | Relative line numbers |
| `:set tabstop` | ❌ | - | Tab width |
| `:set expandtab` | ❌ | - | Spaces for tabs |
| `:set autoindent` | ❌ | - | Auto-indent |
| `:set hlsearch` | ❌ | - | Highlight search |
| `:set incsearch` | ❌ | - | Incremental search |
| `:syntax on` | ❌ | - | Syntax highlighting |

---

# IMPLEMENTATION PRIORITY

## Phase 1: bashrs Integration & Validation (IMMEDIATE)

**Goal**: Integrate bashrs validation into WOS development workflow

**Tickets**:
1. **WOS-BASH-01**: bashrs pre-commit hook integration
   - Add bashrs to quality gates
   - Validate all shell scripts in codebase
   - Auto-fix common issues
   - **Tests**: Pre-commit hook tests

2. **WOS-BASH-02**: Variable quoting validation (SC2086)
   - Enforce quoted variable expansion
   - Add E2E tests for unquoted vars
   - **Tests**: `tests/e2e/bash-quoting-test.spec.js`

3. **WOS-BASH-03**: Command substitution safety (SC2046)
   - Implement `$(...)` syntax
   - Validate quoting
   - **Tests**: `tests/e2e/bash-cmd-subst-test.spec.js`

---

## Phase 2: Core Bash Features (HIGH PRIORITY)

**Goal**: Implement most-used bash features

**Tickets**:
1. **WOS-BASH-04**: Special variables (`$?`, `$$`, `$0`, `$1-$9`)
   - **Tests**: `tests/e2e/bash-special-vars-test.spec.js`

2. **WOS-BASH-05**: Parameter expansion (default values, substring, etc.)
   - **Tests**: `tests/e2e/bash-param-expansion-test.spec.js`

3. **WOS-BASH-06**: Control structures (`if`, `while`, `for`, `case`)
   - **Tests**: `tests/e2e/bash-control-structures-test.spec.js`

4. **WOS-BASH-07**: Functions and local variables
   - **Tests**: `tests/e2e/bash-functions-test.spec.js`

5. **WOS-BASH-08**: Glob patterns and filename expansion
   - **Tests**: `tests/e2e/bash-globbing-test.spec.js`

---

## Phase 3: Essential Vim Features (HIGH PRIORITY)

**Goal**: Implement most-used vim editing commands

**Tickets**:
1. **WOS-VIM-01**: Word movement (`w`, `b`, `e`)
   - **Tests**: `tests/e2e/vim-word-movement-test.spec.js`

2. **WOS-VIM-02**: Line navigation (`0`, `$`, `^`, `gg`, `G`)
   - **Tests**: `tests/e2e/vim-line-nav-test.spec.js`

3. **WOS-VIM-03**: Delete operations (`x`, `dd`, `D`, `d{motion}`)
   - **Tests**: `tests/e2e/vim-delete-test.spec.js`

4. **WOS-VIM-04**: Yank and paste (`yy`, `y{motion}`, `p`, `P`)
   - **Tests**: `tests/e2e/vim-yank-paste-test.spec.js`

5. **WOS-VIM-05**: Undo/redo (`u`, `Ctrl-r`)
   - **Tests**: `tests/e2e/vim-undo-redo-test.spec.js`

6. **WOS-VIM-06**: Visual mode (`v`, `V`, basic operations)
   - **Tests**: `tests/e2e/vim-visual-mode-test.spec.js`

7. **WOS-VIM-07**: Search (`/pattern`, `n`, `N`)
   - **Tests**: `tests/e2e/vim-search-test.spec.js`

8. **WOS-VIM-08**: Substitute (`:s/old/new/`, `:%s/old/new/g`)
   - **Tests**: `tests/e2e/vim-substitute-test.spec.js`

---

## Phase 4: Advanced Bash Features (MEDIUM PRIORITY)

**Tickets**:
1. **WOS-BASH-09**: Arrays (indexed arrays)
2. **WOS-BASH-10**: Arithmetic expansion (`$((expr))`)
3. **WOS-BASH-11**: Brace expansion (`{a,b,c}`, `{1..10}`)
4. **WOS-BASH-12**: Here documents (`<<`)
5. **WOS-BASH-13**: Advanced redirection (`2>`, `&>`)

---

## Phase 5: Advanced Vim Features (MEDIUM PRIORITY)

**Tickets**:
1. **WOS-VIM-09**: Macros (`q{reg}`, `@{reg}`)
2. **WOS-VIM-10**: Marks (`m{a-z}`, `'{a-z}`)
3. **WOS-VIM-11**: Registers (`"{a-z}`)
4. **WOS-VIM-12**: Advanced movement (`f`, `t`, `%`, `{`, `}`)
5. **WOS-VIM-13**: Text objects (`iw`, `aw`, `i"`, `a"`)
6. **WOS-VIM-14**: Configuration (`:set` commands)

---

# EXTREME TDD WORKFLOW

For EVERY feature:

## 1. RED Phase (Write Failing Tests)

```bash
# Example: Implementing $? special variable
# File: tests/e2e/bash-special-vars-test.spec.js

test('$? returns exit status of last command', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');

  // Success case
  await executeCommand(page, 'echo hello');
  await executeCommand(page, 'echo $?');
  const output1 = await getLastOutput(page);
  expect(output1).toBe('0');

  // Failure case
  await executeCommand(page, 'ls /nonexistent');
  await executeCommand(page, 'echo $?');
  const output2 = await getLastOutput(page);
  expect(output2).not.toBe('0');
});
```

**Run tests**: `npx playwright test tests/e2e/bash-special-vars-test.spec.js`
**Expected**: FAIL (not implemented yet)

---

## 2. GREEN Phase (Minimal Implementation)

```rust
// userspace/src/shell.rs

pub fn expand_variable(&self, var: &str) -> String {
    match var {
        "?" => self.last_exit_code.to_string(),
        _ => self.env.get(var).cloned().unwrap_or_default(),
    }
}
```

```javascript
// dist/wos/app.js

expandVariable(varName) {
  if (varName === '?') {
    return this.wos.getLastExitCode().toString();
  }
  return this.wos.getEnvVar(varName) || '';
}
```

**Run tests**: `npx playwright test tests/e2e/bash-special-vars-test.spec.js`
**Expected**: PASS

---

## 3. REFACTOR Phase (Improve Quality)

- Add property tests (proptest)
- Add unit tests
- Run bashrs validation
- Check complexity (≤10)
- Verify coverage (≥85%)
- Run mutation tests

---

## 4. COMMIT

```bash
git add .
git commit -m "[WOS-BASH-04] feat: Implement special variable $? (exit status)

- Added $? expansion in shell variable expansion
- Returns exit code of last executed command
- E2E tests: 5/5 passing
- bashrs validation: PASS
- Complexity: 4 (under limit)

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

# bashrs INTEGRATION CHECKLIST

## Pre-commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
set -euo pipefail

echo "Running bashrs validation..."
bashrs lint userspace/src/*.rs --format json > bashrs-report.json

if [ "$(jq '.violations | length' bashrs-report.json)" -gt 0 ]; then
  echo "❌ bashrs violations found:"
  jq '.violations' bashrs-report.json
  exit 1
fi

echo "✅ bashrs validation passed"
```

## Quality Gates

Add to `Makefile`:

```makefile
.PHONY: bashrs-check
bashrs-check:
	@echo "🔍 Running bashrs validation..."
	@bashrs lint userspace/src/*.rs --format json > bashrs-report.json
	@if [ "$$(jq '.violations | length' bashrs-report.json)" -gt 0 ]; then \
		echo "❌ bashrs violations found"; \
		jq '.violations' bashrs-report.json; \
		exit 1; \
	fi
	@echo "✅ bashrs validation passed"

.PHONY: bashrs-fix
bashrs-fix:
	@echo "🔧 Auto-fixing bashrs violations..."
	@bashrs fix userspace/src/*.rs --backup
	@echo "✅ Fixes applied (backups created)"
```

Add to quality gate pipeline:

```bash
make bashrs-check  # Run in CI/CD
```

---

# COVERAGE TARGETS

- **Bash Core Features**: 90% E2E coverage
- **Vim Core Features**: 90% E2E coverage
- **bashrs Rules**: 100% compliance
- **Mutation Score**: ≥90% for new code
- **Complexity**: All functions ≤10

---

# OUT OF SCOPE

Features intentionally excluded:

- Job control (`&`, `bg`, `fg`, `jobs`)
- Signal handling (`trap`)
- Process substitution (`<(...)`)
- Coprocesses (`coproc`)
- Advanced regex (`[[ =~ ]]`)
- Vim plugins and scripting
- Vim encryption
- Vim network files
- Multi-user features

---

# REFERENCES

- GNU Bash Manual: https://www.gnu.org/software/bash/manual/bash.html
- Vim User Manual: https://vimhelp.org/usr_toc.txt.html
- bashrs Repository: https://github.com/paiml/bashrs
- WOS Specification: docs/specifications/wos-spec-v1.md

---

**Last Updated**: 2025-10-23
**Version**: 1.0.0
**Status**: Living document - update as features implemented
