# WOS Enhanced Vim Integration Specification
# Based on vim.wasm Architecture

**Document Version:** 1.0
**Status:** DRAFT
**Created:** 2025-10-27
**Author:** WOS Development Team
**Reference:** https://github.com/rhysd/vim.wasm

---

## Executive Summary

This specification outlines the integration of advanced Vim features into WOS (WASM Operating System) inspired by vim.wasm's architecture. The goal is to enhance WOS's current modal Vim editor with full Vim functionality including syntax highlighting, VimScript support, visual mode, registers, macros, and plugin compatibility.

**Current State (WOS v0.2.0):**
- Modal editor with INSERT and NORMAL modes
- Basic commands: `:w`, `:q`, `:q!`, `:help`
- Simple editing: `i`, `a`, `o`, `dd`, `x`, navigation
- File save/load via VFS integration

**Target State:**
- Full Vim 8.2+ feature parity in browser
- Syntax highlighting for 20+ languages
- VimScript interpreter for configuration
- Visual mode (character, line, block)
- Registers, macros, marks, and folds
- Plugin system compatible with Vim ecosystem

---

## 1. Architecture Overview

### 1.1 Current WOS Vim Implementation

**Location:** `wos/src/lib.rs` (Vim modal state + commands)

```rust
// Current architecture (simplified)
pub struct WosWasm {
    state: State,           // VFS, processes, memory
    vim_buffer: String,     // Current file content
    vim_cursor: usize,      // Cursor position
    vim_mode: VimMode,      // INSERT | NORMAL
    vim_command: String,    // Command line buffer
}

impl WosWasm {
    pub fn vim_input(&mut self, key: &str) -> String { /* ... */ }
    pub fn vim_render(&self) -> String { /* ... */ }
}
```

**Strengths:**
- ✅ Rust-native, safe, and deterministic
- ✅ Integrated with VFS (`wos/src/state.rs`)
- ✅ Already in WASM (no additional compilation)

**Limitations:**
- ❌ No syntax highlighting
- ❌ Limited command set (~15 commands)
- ❌ No visual mode or registers
- ❌ No VimScript support

### 1.2 vim.wasm Architecture (Reference Implementation)

**Core Design:**
1. **Full Vim C codebase** compiled to WASM via Emscripten
2. **Worker thread** runs Vim WASM (non-blocking UI)
3. **SharedArrayBuffer** for synchronous I/O
4. **Atomics.wait()** for blocking operations
5. **Custom GUI frontend** (`gui_wasm.c`) for rendering

**Key Innovation:**
```javascript
// vim.wasm event loop
while (running) {
    // Block wait for input in worker thread
    Atomics.wait(sharedBuffer, 0, 0);

    // Process input from main thread
    const key = readFromSharedMemory();
    vim_process_key(key);

    // Send draw commands back to main thread
    postMessage({ type: 'draw', buffer: screen });
}
```

### 1.3 Proposed WOS Integration Strategy

**Hybrid Approach:** Leverage vim.wasm as a library, not a replacement

```
┌─────────────────────────────────────────────────┐
│ WOS Terminal (dist/wos/index.html)             │
│  ├─ Bash Shell                                  │
│  └─ Vim Modal (CURRENT: Rust impl)             │
│      ↓                                          │
│     [ENHANCED] Vim Integration Layer            │
│      ├─ Mode: "lite" (current Rust impl)        │
│      └─ Mode: "full" (vim.wasm via iframe)      │
└─────────────────────────────────────────────────┘
```

**Why Hybrid?**
1. **Backwards compatibility:** Keep existing Rust Vim for quick edits
2. **Progressive enhancement:** Opt-in to full Vim with `:set vim=full`
3. **Size optimization:** vim.wasm bundle (~2MB) loaded on-demand
4. **Performance:** Lite mode faster for simple edits

---

## 2. Feature Roadmap

### 2.1 Phase 1: Foundation (v0.3.0)
**Goal:** Enhanced editing without WASM dependency

**Features:**
- [ ] **Visual Mode** (v, V, Ctrl+V)
  - Character-wise selection
  - Line-wise selection
  - Block-wise selection
- [ ] **Registers** (unnamed, named, clipboard)
  - `"ayy` - yank line to register a
  - `"ap` - paste from register a
  - `"*y` - yank to system clipboard
- [ ] **Marks** (m{a-z}, '{a-z})
  - Local marks per buffer
  - Jump to mark with `'{mark}`
- [ ] **Basic Macros** (q{a-z}, @{a-z})
  - Record keystroke sequence
  - Replay macro N times

**Implementation Location:** `wos/src/vim_enhanced.rs` (new module)

**Estimated Size Impact:** +15KB WASM

### 2.2 Phase 2: Syntax Highlighting (v0.4.0)
**Goal:** Visual language awareness

**Features:**
- [ ] **Syntax Engine**
  - Use `tree-sitter-wasm` for parsing
  - Support: JavaScript, Rust, Python, Markdown, JSON
  - Configurable color schemes
- [ ] **Highlight Groups**
  - Comments, strings, keywords, functions
  - Numbers, operators, identifiers
- [ ] **Theme System**
  - Built-in: `default`, `gruvbox`, `monokai`
  - Load from `~/.vim/colors/`

**Integration:**
```rust
// wos/src/vim_syntax.rs
pub struct SyntaxHighlighter {
    parser: TreeSitterParser,
    language: Language,
    theme: ColorScheme,
}

impl SyntaxHighlighter {
    pub fn highlight(&self, text: &str) -> Vec<HighlightSpan> {
        let tree = self.parser.parse(text);
        self.apply_theme(tree.root_node())
    }
}
```

**Dependencies:**
- `tree-sitter-wasm` (70KB gzipped)
- Language grammars: ~10KB each

**Estimated Size Impact:** +150KB WASM

### 2.3 Phase 3: VimScript Interpreter (v0.5.0)
**Goal:** Configuration and customization

**Features:**
- [ ] **Core VimScript**
  - Variables (`let g:var = value`)
  - Functions (`function! Name() ... endfunction`)
  - Control flow (if/else, for, while)
  - Commands (`:command`, `:autocmd`)
- [ ] **Vim Configuration**
  - Load `~/.vimrc` on startup
  - `:source` command to load scripts
  - `:set` options (expandtab, tabstop, etc.)
- [ ] **Essential Commands**
  - `:map`, `:nmap`, `:imap` (key mappings)
  - `:syntax on/off`
  - `:colorscheme`
  - `:set number`, `:set relativenumber`

**Implementation:**
```rust
// wos/src/vimscript.rs
pub struct VimScriptEngine {
    globals: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    mappings: HashMap<String, Mapping>,
}

impl VimScriptEngine {
    pub fn execute(&mut self, script: &str) -> Result<(), VimError> {
        let ast = self.parse(script)?;
        self.eval(ast)
    }
}
```

**Estimated Size Impact:** +50KB WASM

### 2.4 Phase 4: vim.wasm Integration (v0.6.0)
**Goal:** Full Vim parity via on-demand loading

**Features:**
- [ ] **Dual Mode System**
  - Command: `:set vim=lite` (default, Rust impl)
  - Command: `:set vim=full` (load vim.wasm)
- [ ] **vim.wasm Bridge**
  - Iframe isolation for vim.wasm
  - PostMessage API for file I/O
  - VFS synchronization
- [ ] **Advanced Features** (vim.wasm only)
  - Popup windows
  - Terminal mode (`:terminal`)
  - Plugin support (vim-plug, Vundle)
  - Diff mode
  - Spell checking

**Integration Architecture:**
```javascript
// dist/wos/vim-bridge.js
class VimBridge {
    constructor() {
        this.iframe = null;
        this.vfs = null;
    }

    async loadVimWasm() {
        // Lazy load vim.wasm bundle
        this.iframe = document.createElement('iframe');
        this.iframe.src = 'vim-wasm/index.html';
        this.iframe.style.display = 'none';
        document.body.appendChild(this.iframe);

        // Set up message passing
        this.iframe.contentWindow.addEventListener('message',
            this.handleVimMessage.bind(this));
    }

    handleVimMessage(event) {
        switch (event.data.type) {
            case 'file:read':
                return this.vfs.readFile(event.data.path);
            case 'file:write':
                return this.vfs.writeFile(event.data.path, event.data.content);
            case 'draw':
                return this.renderVimBuffer(event.data.buffer);
        }
    }
}
```

**Bundle Size:**
- vim.wasm core: ~1.5MB (gzipped)
- Runtime: ~200KB
- Total on-demand: ~1.7MB

---

## 3. Technical Implementation

### 3.1 Rust Vim Enhancement (Phase 1-3)

**File Structure:**
```
wos/src/
├── lib.rs                  # Current WosWasm entry point
├── vim/                    # New vim module (PROPOSED)
│   ├── mod.rs              # Vim state machine
│   ├── modes.rs            # NORMAL, INSERT, VISUAL, COMMAND
│   ├── commands.rs         # Ex commands (:w, :q, etc.)
│   ├── motions.rs          # h,j,k,l, w, b, $, 0, gg, G
│   ├── operators.rs        # d, y, c, >, <, =
│   ├── registers.rs        # Named registers + clipboard
│   ├── marks.rs            # Mark management
│   ├── macros.rs           # Macro recording/playback
│   ├── syntax.rs           # Syntax highlighting (tree-sitter)
│   ├── vimscript.rs        # VimScript interpreter
│   └── buffer.rs           # Buffer abstraction (text + metadata)
```

**Example: Visual Mode Implementation**
```rust
// wos/src/vim/modes.rs
#[derive(Debug, Clone, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual(VisualMode),
    Command,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisualMode {
    Character { start: usize, end: usize },
    Line { start_line: usize, end_line: usize },
    Block { start: (usize, usize), end: (usize, usize) },
}

impl VimState {
    pub fn enter_visual_char(&mut self) {
        let cursor = self.cursor;
        self.mode = VimMode::Visual(VisualMode::Character {
            start: cursor,
            end: cursor,
        });
    }

    pub fn visual_delete(&mut self) -> Result<(), VimError> {
        match &self.mode {
            VimMode::Visual(VisualMode::Character { start, end }) => {
                let (s, e) = (*start.min(end), *start.max(end));
                self.buffer.delete_range(s..=e);
                self.mode = VimMode::Normal;
                Ok(())
            }
            _ => Err(VimError::InvalidMode),
        }
    }
}
```

### 3.2 Syntax Highlighting with Tree-Sitter

**Dependencies:**
```toml
# wos/Cargo.toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-python = "0.20"
tree-sitter-markdown = "0.20"
```

**Implementation:**
```rust
// wos/src/vim/syntax.rs
use tree_sitter::{Parser, Language};

pub struct SyntaxHighlighter {
    parser: Parser,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new(language: Language, theme: Theme) -> Self {
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        Self { parser, theme }
    }

    pub fn highlight(&mut self, text: &str) -> Vec<HighlightSpan> {
        let tree = self.parser.parse(text, None).unwrap();
        let mut highlights = Vec::new();

        let mut cursor = tree.walk();
        self.visit_node(&mut cursor, &mut highlights);

        highlights
    }

    fn visit_node(&self, cursor: &mut TreeCursor, highlights: &mut Vec<HighlightSpan>) {
        let node = cursor.node();
        let kind = node.kind();

        if let Some(color) = self.theme.get_color(kind) {
            highlights.push(HighlightSpan {
                range: node.byte_range(),
                color,
                style: self.theme.get_style(kind),
            });
        }

        if cursor.goto_first_child() {
            loop {
                self.visit_node(cursor, highlights);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub range: std::ops::Range<usize>,
    pub color: Color,
    pub style: Style,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum Style {
    Normal,
    Bold,
    Italic,
    Underline,
}
```

**Rendering:**
```javascript
// dist/wos/app.js (existing file)
function renderVimBufferWithHighlights(buffer, highlights) {
    const lines = buffer.split('\n');
    const container = document.getElementById('vim-buffer');
    container.innerHTML = '';

    highlights.forEach(span => {
        const pre = document.createElement('span');
        pre.textContent = buffer.slice(span.start, span.end);
        pre.style.color = `rgb(${span.color.r}, ${span.color.g}, ${span.color.b})`;

        if (span.style === 'Bold') pre.style.fontWeight = 'bold';
        if (span.style === 'Italic') pre.style.fontStyle = 'italic';
        if (span.style === 'Underline') pre.style.textDecoration = 'underline';

        container.appendChild(pre);
    });
}
```

### 3.3 VimScript Interpreter

**Parser:**
```rust
// wos/src/vim/vimscript.rs
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "vim/vimscript.pest"]
pub struct VimScriptParser;

pub fn parse_vimscript(input: &str) -> Result<Vec<Statement>, Error> {
    let pairs = VimScriptParser::parse(Rule::script, input)?;
    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::let_statement => statements.push(parse_let(pair)),
            Rule::function_def => statements.push(parse_function(pair)),
            Rule::if_statement => statements.push(parse_if(pair)),
            Rule::command => statements.push(parse_command(pair)),
            _ => {}
        }
    }

    Ok(statements)
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { var: String, value: Value },
    Function { name: String, params: Vec<String>, body: Vec<Statement> },
    If { condition: Expr, then_block: Vec<Statement>, else_block: Vec<Statement> },
    Command { name: String, args: Vec<String> },
}
```

**Grammar (pest):**
```pest
// wos/src/vim/vimscript.pest
script = { SOI ~ statement* ~ EOI }

statement = {
    let_statement |
    function_def |
    if_statement |
    command
}

let_statement = { "let" ~ variable ~ "=" ~ expression }
function_def = { "function!" ~ identifier ~ "(" ~ param_list? ~ ")" ~ statement* ~ "endfunction" }
if_statement = { "if" ~ expression ~ statement* ~ ("else" ~ statement*)? ~ "endif" }
command = { ":" ~ identifier ~ arg_list? }

variable = { ("g:" | "l:" | "s:")? ~ identifier }
expression = { number | string | variable | function_call }
identifier = { (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
```

### 3.4 vim.wasm Bridge (Phase 4)

**Lazy Loading:**
```javascript
// dist/wos/vim-integration.js
class VimIntegration {
    constructor(wosInstance) {
        this.wos = wosInstance;
        this.mode = 'lite'; // 'lite' | 'full'
        this.vimWasmBridge = null;
    }

    async enableFullVim() {
        if (this.mode === 'full') return;

        // Show loading indicator
        this.wos.showMessage('Loading vim.wasm...');

        // Dynamically import vim.wasm bundle
        const { VimWasm } = await import('./vim-wasm/bundle.js');

        // Initialize vim.wasm in worker
        this.vimWasmBridge = new VimWasm({
            workerScriptPath: './vim-wasm/worker.js',
            canvas: document.getElementById('vim-canvas'),
            onFileRead: (path) => this.wos.vfs.readFile(path),
            onFileWrite: (path, content) => this.wos.vfs.writeFile(path, content),
        });

        await this.vimWasmBridge.init();
        this.mode = 'full';

        this.wos.showMessage('vim.wasm loaded! Full Vim features enabled.');
    }

    handleInput(key) {
        if (this.mode === 'lite') {
            // Use Rust Vim implementation
            return this.wos.vim_input(key);
        } else {
            // Forward to vim.wasm
            return this.vimWasmBridge.input(key);
        }
    }
}
```

**VFS Synchronization:**
```rust
// wos/src/lib.rs (additions)
#[wasm_bindgen]
impl WosWasm {
    /// Called from JavaScript when vim.wasm wants to read a file
    pub fn vim_wasm_read_file(&self, path: String) -> Result<String, JsValue> {
        let path_buf = std::path::PathBuf::from(&path);
        match self.state.vfs.read_file(&path_buf) {
            Ok(bytes) => String::from_utf8(bytes)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Called from JavaScript when vim.wasm wants to write a file
    pub fn vim_wasm_write_file(&mut self, path: String, content: String) -> Result<(), JsValue> {
        let path_buf = std::path::PathBuf::from(&path);
        self.state.vfs.write_file(&path_buf, content.into_bytes())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

---

## 4. Quality & Testing Strategy

### 4.1 Test Coverage Requirements

**Unit Tests (Rust):**
```rust
// wos/tests/vim_tests.rs
#[cfg(test)]
mod vim_tests {
    use super::*;

    #[test]
    fn test_visual_char_selection() {
        let mut vim = VimState::new();
        vim.buffer.set_text("hello world");
        vim.cursor = 0;

        vim.enter_visual_char();
        vim.move_cursor_right(5);

        match &vim.mode {
            VimMode::Visual(VisualMode::Character { start, end }) => {
                assert_eq!(*start, 0);
                assert_eq!(*end, 5);
            }
            _ => panic!("Expected visual mode"),
        }
    }

    #[test]
    fn test_register_yank_paste() {
        let mut vim = VimState::new();
        vim.buffer.set_text("line1\nline2\nline3");

        // Yank line to register 'a'
        vim.cursor = 0;
        vim.execute_command("\"ayy").unwrap();

        // Move to different line
        vim.cursor = vim.buffer.line_start(2);

        // Paste from register 'a'
        vim.execute_command("\"ap").unwrap();

        assert!(vim.buffer.text().contains("line1\nline2\nline1\nline3"));
    }

    #[test]
    fn test_vimscript_let() {
        let mut engine = VimScriptEngine::new();
        engine.execute("let g:myvar = 42").unwrap();

        match engine.get_global("myvar") {
            Some(Value::Number(n)) => assert_eq!(n, 42),
            _ => panic!("Expected number 42"),
        }
    }
}
```

**E2E Tests (Playwright):**
```javascript
// tests/e2e/vim-enhanced-test.spec.js
const { test, expect } = require('@playwright/test');

test.describe('Vim Enhanced Features', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:8000');
        await page.waitForSelector('#terminal-input');

        // Open file in vim
        await page.fill('#terminal-input', 'vim test.txt');
        await page.press('#terminal-input', 'Enter');
        await page.waitForTimeout(500);
    });

    test('visual mode character selection', async ({ page }) => {
        // Enter visual mode
        await page.keyboard.press('v');

        // Select 5 characters
        for (let i = 0; i < 5; i++) {
            await page.keyboard.press('l');
        }

        // Delete selection
        await page.keyboard.press('d');

        // Verify deletion
        const buffer = await page.locator('#vim-buffer').textContent();
        expect(buffer).not.toContain('hello');
    });

    test('register operations', async ({ page }) => {
        // Type some text
        await page.keyboard.press('i');
        await page.keyboard.type('test line');
        await page.keyboard.press('Escape');

        // Yank line to register 'a'
        await page.keyboard.type('"ayy');

        // Move down and paste
        await page.keyboard.press('o');
        await page.keyboard.press('Escape');
        await page.keyboard.type('"ap');

        const buffer = await page.locator('#vim-buffer').textContent();
        expect(buffer).toContain('test line');
        expect(buffer.split('\n')).toHaveLength(3);
    });

    test('syntax highlighting for JavaScript', async ({ page }) => {
        // Enter JavaScript code
        await page.keyboard.press('i');
        await page.keyboard.type('function hello() {');
        await page.keyboard.press('Enter');
        await page.keyboard.type('  console.log("hi");');
        await page.keyboard.press('Enter');
        await page.keyboard.type('}');
        await page.keyboard.press('Escape');

        // Enable syntax highlighting
        await page.keyboard.type(':syntax on');
        await page.keyboard.press('Enter');

        // Verify highlighted spans exist
        const keywords = await page.locator('.vim-hl-keyword').count();
        expect(keywords).toBeGreaterThan(0);

        const strings = await page.locator('.vim-hl-string').count();
        expect(strings).toBeGreaterThan(0);
    });
});
```

### 4.2 Performance Benchmarks

**Acceptance Criteria:**
- Syntax highlighting: <50ms for 1000 lines
- Visual mode selection: <5ms per keystroke
- VimScript execution: <10ms for typical `~/.vimrc`
- vim.wasm load time: <2s on 3G connection

**Measurement:**
```rust
// wos/benches/vim_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_syntax_highlighting(c: &mut Criterion) {
    let mut highlighter = SyntaxHighlighter::new(
        tree_sitter_rust::language(),
        Theme::default(),
    );

    let code = std::fs::read_to_string("benches/fixtures/large_file.rs").unwrap();

    c.bench_function("highlight 1000 lines", |b| {
        b.iter(|| highlighter.highlight(black_box(&code)))
    });
}

criterion_group!(benches, bench_syntax_highlighting);
criterion_main!(benches);
```

### 4.3 Bundle Size Monitoring

**Target Sizes:**
- Phase 1 (Visual mode + registers): +15KB
- Phase 2 (Syntax highlighting): +150KB
- Phase 3 (VimScript): +50KB
- Phase 4 (vim.wasm bridge): +10KB (bridge only, ~1.7MB on-demand)

**Total:** +225KB for enhanced Vim (WOS baseline ~500KB)

**Monitoring:**
```bash
# wos/scripts/check-bundle-size.sh
#!/bin/bash

WASM_FILE="dist/wos/wos_bg.wasm"
CURRENT_SIZE=$(stat -f%z "$WASM_FILE")
BASELINE=512000  # 500KB

if [ $CURRENT_SIZE -gt $((BASELINE + 250000)) ]; then
    echo "ERROR: WASM bundle exceeds size limit"
    echo "Current: ${CURRENT_SIZE} bytes"
    echo "Limit: $((BASELINE + 250000)) bytes"
    exit 1
fi

echo "Bundle size OK: ${CURRENT_SIZE} bytes"
```

---

## 5. User Experience

### 5.1 Discoverability

**Help System:**
```
:help vim-features

WOS Vim Features
================

Current Mode: lite (enhanced Rust implementation)

Available Features:
  ✓ Visual mode (v, V, Ctrl-V)
  ✓ Registers (" + letter)
  ✓ Marks (m + letter, ' + letter)
  ✓ Basic macros (q + letter, @ + letter)
  ✓ Syntax highlighting (:syntax on)
  ✓ VimScript support (limited)

To enable full Vim features:
  :set vim=full

This will load vim.wasm (~1.7MB) and enable:
  ✓ All Vim 8.2 features
  ✓ Plugin support
  ✓ Terminal mode
  ✓ Popup windows
  ✓ Advanced VimScript

See also:
  :help visual-mode
  :help registers
  :help vimscript
```

### 5.2 Progressive Enhancement

**User Journey:**
1. **First Use:** Default lite mode, instant load
2. **Learning:** Discover advanced features via `:help`
3. **Power User:** Opt-in to full mode with `:set vim=full`
4. **Persistence:** Setting saved to localStorage

**Configuration:**
```vim
" ~/.vimrc (loaded by WOS)
set number
set relativenumber
set expandtab
set tabstop=4
set shiftwidth=4
syntax on
colorscheme gruvbox

" Mappings
nnoremap <C-s> :w<CR>
inoremap jk <Esc>

" Enable full vim if available
if exists('vim_wasm_available')
    set vim=full
endif
```

### 5.3 Error Handling

**Graceful Degradation:**
```rust
// wos/src/vim/mod.rs
impl VimState {
    pub fn execute_command(&mut self, cmd: &str) -> Result<String, VimError> {
        match self.parse_command(cmd) {
            Ok(Command::Plugin { .. }) if !self.features.has_plugins() => {
                Err(VimError::FeatureNotAvailable {
                    feature: "plugins",
                    hint: "Use :set vim=full to enable plugins",
                })
            }
            Ok(Command::Terminal { .. }) if !self.features.has_terminal() => {
                Err(VimError::FeatureNotAvailable {
                    feature: "terminal",
                    hint: "Terminal mode requires vim=full",
                })
            }
            Ok(cmd) => self.execute(cmd),
            Err(e) => Err(e),
        }
    }
}
```

---

## 6. Migration & Compatibility

### 6.1 Backward Compatibility

**Guarantee:** All existing WOS Vim commands remain functional

**Before (v0.2.0):**
```javascript
wos.vim_input('i');
wos.vim_input('hello');
wos.vim_input('Escape');
wos.vim_input(':wq');
```

**After (v0.3.0+):**
```javascript
// Still works exactly the same
wos.vim_input('i');
wos.vim_input('hello');
wos.vim_input('Escape');
wos.vim_input(':wq');

// Plus new features
wos.vim_input('v');  // Visual mode
wos.vim_input('5l'); // Select 5 chars
wos.vim_input('"ay'); // Yank to register a
```

### 6.2 vim.wasm Compatibility

**Shared Storage:**
- Both implementations use WOS VFS
- `~/.vimrc` applies to both modes
- Files saved in lite mode work in full mode

**Mode Switching:**
```rust
impl WosWasm {
    pub fn switch_vim_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        match mode {
            "lite" => {
                // Save vim.wasm state
                if let Some(bridge) = &self.vim_wasm_bridge {
                    let state = bridge.get_state()?;
                    self.vim_state.restore_from(state)?;
                }
                self.vim_mode = VimMode::Lite;
            }
            "full" => {
                // Transfer state to vim.wasm
                let state = self.vim_state.serialize();
                self.load_vim_wasm()?;
                self.vim_wasm_bridge.as_ref().unwrap().set_state(state)?;
                self.vim_mode = VimMode::Full;
            }
            _ => return Err(JsValue::from_str("Invalid mode")),
        }
        Ok(())
    }
}
```

---

## 7. Security Considerations

### 7.1 Sandboxing

**vim.wasm Isolation:**
- Runs in dedicated iframe with `sandbox` attribute
- No direct access to parent window
- All file I/O through postMessage API

```html
<!-- dist/wos/index.html -->
<iframe id="vim-wasm-frame"
        sandbox="allow-scripts allow-same-origin"
        src="vim-wasm/index.html"
        style="display: none;">
</iframe>
```

### 7.2 VimScript Safety

**Restricted Operations:**
```rust
// wos/src/vim/vimscript.rs
impl VimScriptEngine {
    fn is_safe_command(&self, cmd: &str) -> bool {
        // Block dangerous operations
        let blocked = [
            "!sh",       // No shell access
            "!bash",
            "!curl",     // No network
            "!wget",
            "py",        // No Python
            "ruby",      // No Ruby
        ];

        !blocked.iter().any(|b| cmd.starts_with(b))
    }

    pub fn execute(&mut self, script: &str) -> Result<(), VimError> {
        if !self.is_safe_command(script) {
            return Err(VimError::PermissionDenied {
                command: script.to_string(),
                reason: "Command blocked for security",
            });
        }

        // Safe execution
        self.eval_script(script)
    }
}
```

### 7.3 Resource Limits

**Prevent DoS:**
```rust
const MAX_MACRO_ITERATIONS: usize = 1000;
const MAX_VIMSCRIPT_RECURSION: usize = 100;
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

impl VimState {
    pub fn execute_macro(&mut self, reg: char, count: usize) -> Result<(), VimError> {
        let iterations = count.min(MAX_MACRO_ITERATIONS);
        if count > MAX_MACRO_ITERATIONS {
            return Err(VimError::LimitExceeded {
                limit: "macro iterations",
                max: MAX_MACRO_ITERATIONS,
            });
        }

        for _ in 0..iterations {
            self.replay_register(reg)?;
        }

        Ok(())
    }
}
```

---

## 8. Deployment Strategy

### 8.1 Phased Rollout

**Phase 1: Beta (v0.3.0-beta)**
- Feature flag: `?vim=enhanced`
- Opt-in via localStorage
- Collect feedback

**Phase 2: General Availability (v0.3.0)**
- Enabled by default for new users
- Migration path for existing users
- `?vim=lite` to revert

**Phase 3: vim.wasm Integration (v0.6.0)**
- On-demand loading
- Progressive enhancement
- Default remains lite mode

### 8.2 Feature Flags

```rust
// wos/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimConfig {
    pub mode: VimMode,              // lite | full
    pub syntax_highlighting: bool,  // Phase 2
    pub vimscript: bool,            // Phase 3
    pub vim_wasm_enabled: bool,     // Phase 4
    pub experimental: bool,         // Unstable features
}

impl Default for VimConfig {
    fn default() -> Self {
        Self {
            mode: VimMode::Lite,
            syntax_highlighting: true,
            vimscript: true,
            vim_wasm_enabled: false, // On-demand
            experimental: false,
        }
    }
}
```

### 8.3 Metrics & Monitoring

**Track Usage:**
```javascript
// dist/wos/analytics.js
class VimMetrics {
    track(event, data) {
        // Privacy-preserving metrics
        const metric = {
            event,
            timestamp: Date.now(),
            vim_mode: data.mode,          // lite | full
            feature: data.feature,         // visual | register | macro
            duration_ms: data.duration,
        };

        // Store locally, aggregate on demand
        this.store(metric);
    }
}

// Usage
metrics.track('vim.visual_mode_used', { mode: 'lite', duration: 1500 });
metrics.track('vim.wasm_loaded', { mode: 'full', bundle_size: 1700000 });
```

---

## 9. Future Enhancements

### 9.1 Plugin Ecosystem

**Vision:** vim-plug / Vundle compatibility

```vim
" ~/.vimrc
call plug#begin('~/.vim/plugged')

" File explorer
Plug 'preservim/nerdtree'

" Fuzzy finder
Plug 'junegunn/fzf.vim'

" Status line
Plug 'vim-airline/vim-airline'

call plug#end()
```

**Challenges:**
- Plugin installation requires network access
- Many plugins rely on external binaries
- Large plugin ecosystem (~10,000 plugins)

**Proposed Solution:**
- Curated plugin whitelist (50-100 plugins)
- Pre-compile popular plugins to WASM
- Plugin CDN hosted on S3

### 9.2 Language Server Protocol (LSP)

**Vision:** Code intelligence (autocomplete, go-to-definition, refactor)

```vim
" ~/.vimrc
" Enable LSP for Rust
let g:lsp_servers = ['rust-analyzer']

" Key mappings
nnoremap gd :LspDefinition<CR>
nnoremap K :LspHover<CR>
nnoremap <leader>r :LspRename<CR>
```

**Implementation:**
- Run LSP servers in Web Workers
- Use WASM-compiled language servers (rust-analyzer.wasm)
- Communication via JSON-RPC over postMessage

### 9.3 Collaborative Editing

**Vision:** Real-time multi-user editing (Google Docs for Vim)

```vim
:VimShareStart
" Generates shareable URL: wos.example.com/?session=abc123

:VimShareJoin abc123
" Joins existing session
```

**Technology:**
- Operational Transformation (OT) or CRDTs
- WebRTC for peer-to-peer communication
- Fallback to WebSocket server

---

## 10. Success Metrics

### 10.1 Adoption

**Phase 1 (v0.3.0):**
- 50% of Vim users try visual mode (1st week)
- 30% of Vim users configure registers (1st month)
- 10% write custom VimScript (3 months)

**Phase 4 (v0.6.0):**
- 20% of users enable full vim.wasm mode
- <5% bundle size increase for non-vim.wasm users
- <2s vim.wasm load time (p95)

### 10.2 Quality

**All Phases:**
- Zero regressions in existing Vim commands
- 85%+ unit test coverage for new features
- <10ms input latency (p95)
- <100ms syntax highlight update (p95)

### 10.3 User Satisfaction

**Qualitative:**
- NPS score >50 for Vim features
- <5% churn due to Vim changes
- Positive mentions in GitHub issues

**Quantitative:**
- Average session duration increases 20%
- File save frequency increases 15%
- Help command usage decreases 30% (better discoverability)

---

## 11. Open Questions

### 11.1 Architecture

- **Q:** Should syntax highlighting run in worker thread?
  - **Pro:** Non-blocking UI, better for large files
  - **Con:** Increased complexity, postMessage overhead
  - **Decision:** TBD based on Phase 2 benchmarks

- **Q:** How to handle vim.wasm version updates?
  - **A:** Cache-bust with version hash in filename
  - **A:** Implement automatic update check

### 11.2 UX

- **Q:** Should visual mode be default-enabled?
  - **Pro:** Standard Vim behavior, expected by users
  - **Con:** Adds complexity to default experience
  - **Decision:** Yes, it's core Vim functionality

- **Q:** How to indicate vim.wasm loading state?
  - **A:** Loading spinner + progress bar
  - **A:** Show hint: "Loading full Vim features..."

### 11.3 Technical

- **Q:** Can tree-sitter run in main thread without blocking?
  - **A:** Needs benchmarking, likely need worker
  - **A:** Incremental parsing may help

- **Q:** How to handle vim.wasm crashes?
  - **A:** Auto-fallback to lite mode
  - **A:** Save buffer state every N seconds
  - **A:** Show error message with recovery options

---

## 12. References

### 12.1 External Resources

- **vim.wasm:** https://github.com/rhysd/vim.wasm
- **Vim documentation:** https://vimhelp.org/
- **tree-sitter:** https://tree-sitter.github.io/tree-sitter/
- **Emscripten:** https://emscripten.org/
- **pest (Rust parser):** https://pest.rs/

### 12.2 WOS Codebase

- **Current Vim impl:** `wos/src/lib.rs` (lines 2100-2500)
- **VFS:** `wos/src/state.rs`
- **WASM exports:** `wos/src/lib.rs` (wasm_bindgen functions)
- **UI rendering:** `dist/wos/app.js`
- **E2E tests:** `tests/e2e/vim-*.spec.js`

### 12.3 Related Specifications

- **WOS Spec v1:** `docs/specifications/wos-spec-v1.md`
- **Tech Review:** `docs/specifications/wos-tech-review.md`
- **UI Roadmap:** `docs/ui-roadmap.yaml`

---

## Appendix A: Command Reference

### A.1 New Commands (Phase 1)

| Command | Mode | Description |
|---------|------|-------------|
| `v` | NORMAL | Enter visual character mode |
| `V` | NORMAL | Enter visual line mode |
| `Ctrl-V` | NORMAL | Enter visual block mode |
| `"ay` | VISUAL | Yank selection to register a |
| `"ap` | NORMAL | Paste from register a |
| `ma` | NORMAL | Set mark 'a' at cursor |
| `'a` | NORMAL | Jump to mark 'a' |
| `qa` | NORMAL | Start recording macro to register a |
| `q` | RECORDING | Stop recording macro |
| `@a` | NORMAL | Replay macro from register a |
| `@@` | NORMAL | Replay last macro |

### A.2 New Commands (Phase 2)

| Command | Mode | Description |
|---------|------|-------------|
| `:syntax on` | COMMAND | Enable syntax highlighting |
| `:syntax off` | COMMAND | Disable syntax highlighting |
| `:colorscheme <name>` | COMMAND | Change color scheme |

### A.3 New Commands (Phase 3)

| Command | Mode | Description |
|---------|------|-------------|
| `:source <file>` | COMMAND | Execute VimScript file |
| `:let <var> = <value>` | COMMAND | Set variable |
| `:function <name>()` | COMMAND | Define function |
| `:map <lhs> <rhs>` | COMMAND | Create key mapping |

### A.4 New Commands (Phase 4)

| Command | Mode | Description |
|---------|------|-------------|
| `:set vim=lite` | COMMAND | Use Rust Vim implementation |
| `:set vim=full` | COMMAND | Load vim.wasm (full features) |
| `:terminal` | COMMAND | Open terminal buffer (vim.wasm only) |
| `:PlugInstall` | COMMAND | Install plugins (vim.wasm only) |

---

## Appendix B: API Reference

### B.1 Rust API (wos/src/vim/mod.rs)

```rust
pub struct VimState {
    pub mode: VimMode,
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub registers: Registers,
    pub marks: Marks,
    pub macros: Macros,
    pub config: VimConfig,
}

impl VimState {
    pub fn new() -> Self { /* ... */ }
    pub fn input(&mut self, key: &str) -> Result<(), VimError> { /* ... */ }
    pub fn execute_command(&mut self, cmd: &str) -> Result<String, VimError> { /* ... */ }
    pub fn render(&self) -> String { /* ... */ }
    pub fn get_highlights(&self) -> Vec<HighlightSpan> { /* ... */ }
}
```

### B.2 JavaScript API (dist/wos/vim-integration.js)

```javascript
class VimIntegration {
    constructor(wosInstance) { /* ... */ }

    // Mode management
    async enableFullVim() { /* ... */ }
    switchToLiteMode() { /* ... */ }

    // Input handling
    handleInput(key) { /* ... */ }

    // State synchronization
    getState() { /* ... */ }
    setState(state) { /* ... */ }

    // File I/O
    readFile(path) { /* ... */ }
    writeFile(path, content) { /* ... */ }
}
```

### B.3 WASM Exports (wos/src/lib.rs)

```rust
#[wasm_bindgen]
impl WosWasm {
    // Vim input
    pub fn vim_input(&mut self, key: &str) -> String;
    pub fn vim_command(&mut self, cmd: &str) -> Result<String, JsValue>;

    // Vim state
    pub fn vim_get_buffer(&self) -> String;
    pub fn vim_get_cursor(&self) -> usize;
    pub fn vim_get_mode(&self) -> String;
    pub fn vim_get_highlights(&self) -> JsValue; // Serialized Vec<HighlightSpan>

    // vim.wasm bridge
    pub fn vim_wasm_read_file(&self, path: String) -> Result<String, JsValue>;
    pub fn vim_wasm_write_file(&mut self, path: String, content: String) -> Result<(), JsValue>;
}
```

---

**Document Status:** DRAFT - Ready for Review
**Next Steps:**
1. Team review and feedback
2. Prototype Phase 1 (visual mode + registers)
3. Benchmark syntax highlighting options
4. Evaluate vim.wasm integration complexity

**Approval Required From:**
- [ ] WOS Core Team
- [ ] UI/UX Lead
- [ ] Performance Engineering
- [ ] Security Team

---

*This specification is a living document. Updates will be tracked in git history.*
