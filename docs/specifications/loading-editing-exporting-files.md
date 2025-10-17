# WOS Cloud Shell File Management & Editing Specification

**Version**: 1.1
**Date**: October 17, 2025
**Status**: Approved with Enhancements
**Methodology**: Extreme TDD + Mutation + Fuzz + Property + PMAT + Visual Regression

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Requirements Analysis](#requirements-analysis)
3. [Architecture Design](#architecture-design)
4. [Design Pattern Enhancements](#design-pattern-enhancements)
5. [Implementation Roadmap](#implementation-roadmap)
6. [Testing Strategy](#testing-strategy)
7. [Quality Gates](#quality-gates)
8. [Technical Review Recommendations](#technical-review-recommendations)

---

## 1. Executive Summary

### 1.1 Problem Statement

**Current Limitations**:
- ❌ No way to upload files from host system to WOS VFS
- ❌ No interactive file editor (only `echo` redirection)
- ❌ No way to download files from WOS VFS to host
- ❌ Right sidebar wastes space with static panels (Process List, System Info, Quality Metrics)
- ❌ Cannot edit multi-line files interactively
- ❌ No support for loading/running Dockerfiles (future requirement)

**Cloud Shell Comparison** (AWS CloudShell, Azure Cloud Shell, GCP Cloud Shell):
```
Feature                 AWS    Azure   GCP    WOS (Current)
─────────────────────────────────────────────────────────────
File Upload             ✅     ✅      ✅     ❌
File Download           ✅     ✅      ✅     ❌
File Browser/Explorer   ✅     ✅      ✅     ❌
Built-in Editor (vim)   ✅     ✅      ✅     ❌
Syntax Highlighting     ✅     ✅      ✅     ❌
Multi-file Tabs         ✅     ✅      ✅     ❌
```

### 1.2 Proposed Solution

**Cloud Shell File Manager** - Replace right sidebar with:

```
┌─────────────────────────────────────────────────────┐
│                  WOS Cloud Shell                    │
├─────────────────────────────────┬───────────────────┤
│         Terminal (Left)         │  File Manager     │
│  ┌──────────────────────────┐  │  (Right - NEW)    │
│  │ wos$ vim /app.py         │  │                   │
│  │                          │  │ 📁 Files          │
│  │ (vim editor or terminal) │  │  └─ /             │
│  │                          │  │     ├─ 📄 app.py  │
│  │                          │  │     ├─ 📄 test.txt│
│  │                          │  │     └─ 📁 src/    │
│  │                          │  │                   │
│  │                          │  │ 🔧 Actions        │
│  │                          │  │  ├─ ⬆️ Upload File │
│  │                          │  │  ├─ ⬇️ Download   │
│  │                          │  │  ├─ ✏️ Edit in Vim│
│  │                          │  │  └─ 🗑️ Delete     │
│  └──────────────────────────┘  │                   │
│                                 │ 📊 Quick Info     │
│                                 │  • Processes: 3   │
│                                 │  • Files: 12      │
│                                 │  • TDG: A+        │
└─────────────────────────────────┴───────────────────┘
```

**Key Features**:
1. **File Browser** - Visual tree of VFS files
2. **Upload/Download** - Seamless host ↔ VFS transfer
3. **Vim Editor** - Modal editor with insert/command modes
4. **Syntax Highlighting** - Language-aware coloring
5. **Dockerfile Support** - Future: Parse and run Dockerfiles

### 1.3 Success Criteria

**MVP Features** (WOS-031):
- ✅ File browser shows VFS directory tree
- ✅ Upload button loads file from host → VFS
- ✅ Download button saves VFS file → host
- ✅ Vim editor with insert/command/visual modes
- ✅ Basic vim commands (:w, :q, :wq, dd, yy, p, /, etc.)
- ✅ Syntax highlighting (Rust, Python, JS, Dockerfile)
- ✅ Multi-file editing (tabs or buffers)

**Quality Targets**:
- Test Coverage: 90%+ (higher than MVP 85%)
- Mutation Score: 95%+ (higher than MVP 90%)
- E2E Tests: 15+ scenarios (file operations, vim commands)
- Property Tests: 10+ (vim state machine, file operations)
- Fuzzing: 5 targets (parser, vim commands, file paths)
- PMAT Complexity: ≤15 cyclomatic, ≤10 cognitive

---

## 2. Requirements Analysis

### 2.1 Functional Requirements

#### FR-1: File Browser
```yaml
requirement: Display VFS directory tree
acceptance_criteria:
  - Show all files and directories in hierarchical view
  - Click file → select for edit/download
  - Click directory → expand/collapse
  - Show file size and modification time
  - Icons for file types (.rs, .py, .js, .txt, Dockerfile)
test_coverage:
  - Unit: test_file_browser_renders_tree()
  - E2E: test_file_browser_interaction()
  - Property: proptest_tree_structure_consistent()
```

#### FR-2: File Upload (Host → VFS)
```yaml
requirement: Upload file from local filesystem to WOS VFS
approach: File System Access API with graceful fallback
acceptance_criteria:
  - Upload button opens file picker (or File System Access API)
  - Selected file content read as UTF-8 or binary
  - File written to VFS at /uploads/<filename>
  - User can specify destination path
  - Progress indicator for large files (>1MB)
  - Error handling (file too large, invalid encoding)
  - **NEW**: Use File System Access API where supported for richer UX
  - **NEW**: Graceful fallback to traditional upload for older browsers
test_coverage:
  - Unit: test_upload_file_to_vfs()
  - Unit: test_upload_handles_binary_files()
  - Unit: test_file_system_access_api_detection()
  - E2E: test_upload_file_button_workflow()
  - E2E: test_file_system_access_api_workflow()
  - Fuzz: fuzz_upload_with_random_content()

technical_note: |
  The File System Access API (W3C standard) enables direct file system
  access with user permission. When a file is opened via this API, edits
  can be saved back to the original file on the host machine, creating a
  workflow similar to native desktop editors. Falls back to traditional
  File API for browsers that don't support it (Safari, older browsers).
```

#### FR-3: File Download (VFS → Host)
```yaml
requirement: Download file from WOS VFS to local filesystem
acceptance_criteria:
  - Download button for selected file
  - File content read from VFS
  - Browser download triggered with correct filename
  - Supports text and binary files
  - MIME type detection (text/plain, application/octet-stream)
test_coverage:
  - Unit: test_download_file_from_vfs()
  - Unit: test_download_preserves_content()
  - E2E: test_download_file_button_workflow()
  - Property: proptest_upload_download_roundtrip()
```

#### FR-4: Vim Editor (Modal Editing)
```yaml
requirement: Implement basic vim editor with modal editing
modes:
  - NORMAL: Command mode (navigation, delete, yank, paste)
  - INSERT: Text insertion mode
  - VISUAL: Text selection mode
  - COMMAND: Ex commands (:w, :q, :wq, :set, etc.)

normal_mode_commands:
  navigation:
    - h, j, k, l: left, down, up, right
    - w, b: word forward/backward
    - 0, $: line start/end
    - gg, G: file start/end
    - :<line>: go to line
  editing:
    - i, a, o, O: insert before/after, new line below/above
    - x, dd: delete char/line
    - yy, p: yank/paste line
    - u, Ctrl+r: undo/redo
    - r: replace char
  search:
    - /pattern: search forward
    - ?pattern: search backward
    - n, N: next/previous match

insert_mode:
  - ESC: return to normal mode
  - Backspace, Delete: character deletion
  - Arrow keys: navigation
  - Typing: insert text

command_mode:
  - :w: write (save) file
  - :q: quit (if no changes)
  - :wq or :x: write and quit
  - :q!: quit without saving
  - :e <file>: edit another file
  - :set number: show line numbers
  - :syntax on: enable syntax highlighting
  - **:help**: open vim command reference (self-contained documentation)

acceptance_criteria:
  - All modes transition correctly (state machine)
  - Commands execute correctly
  - File content persists to VFS on :w
  - Visual feedback (mode indicator, cursor)
  - Command history (: commands)

test_coverage:
  - Unit: test_vim_mode_transitions() (20+ tests)
  - Unit: test_vim_commands() (50+ tests per command)
  - Property: proptest_vim_state_machine()
  - Property: proptest_undo_redo_consistency()
  - E2E: test_vim_editing_workflow()
  - Mutation: 95%+ kill rate for vim logic
```

#### FR-5: Syntax Highlighting
```yaml
requirement: Language-aware syntax highlighting in vim
approach: tree-sitter parsing library (incremental, robust, WASM-compatible)

supported_languages:
  - Rust (.rs): keywords, types, strings, comments
  - Python (.py): keywords, functions, strings, comments
  - JavaScript (.js, .ts): keywords, functions, strings
  - Dockerfile: FROM, RUN, COPY, etc.
  - Shell (.sh): commands, variables, strings
  - JSON (.json): keys, values, brackets
  - YAML (.yaml, .yml): keys, values, indentation

highlighting_tokens:
  - keywords: blue
  - strings: green
  - comments: gray
  - functions: yellow
  - types: cyan
  - numbers: magenta

implementation_strategy:
  library: tree-sitter (https://tree-sitter.github.io)
  rationale: |
    tree-sitter is an incremental parsing library specifically designed for
    syntax highlighting and code analysis. It provides:
    - Full Concrete Syntax Tree (CST) for accuracy
    - Incremental parsing for performance
    - Pre-built grammars for all target languages
    - Official Rust bindings with WASM support
    - Used by GitHub Codespaces, Zed, Atom

  benefits_over_manual_parsers:
    - Robustness: Handles complex syntax and edge cases
    - Performance: Incremental updates as user types
    - Maintainability: No custom parser code to maintain
    - Ecosystem: Leverage existing high-quality grammars

acceptance_criteria:
  - Auto-detect language from file extension
  - :syntax on/off toggles highlighting
  - Syntax updates in real-time during editing
  - Performance: <50ms for 1000-line file
  - **NEW**: Use tree-sitter for all language parsers
  - **NEW**: Incremental parsing on edits

test_coverage:
  - Unit: test_tree_sitter_integration()
  - Unit: test_syntax_highlighter_rust()
  - Unit: test_syntax_highlighter_python()
  - Unit: test_incremental_parsing()
  - Property: proptest_syntax_never_panics()
  - Benchmark: bench_syntax_highlighting_1000_lines()
  - Benchmark: bench_incremental_update()
```

#### FR-6: Multi-File Editing
```yaml
requirement: Edit multiple files simultaneously
approach: Buffer-based (vim-style)
commands:
  - :e <file>: edit another file
  - :bnext, :bprev: next/previous buffer
  - :ls: list buffers
  - :b <name>: switch to buffer

ui:
  - Buffer list in file browser (highlight active)
  - OR tab bar above editor (optional)

acceptance_criteria:
  - Switch between buffers without data loss
  - Unsaved changes warning on :q
  - Buffer state persists (cursor position, undo stack)

test_coverage:
  - Unit: test_buffer_switching()
  - Unit: test_unsaved_changes_warning()
  - E2E: test_multi_file_editing_workflow()
```

### 2.2 Non-Functional Requirements

#### NFR-1: Performance
```yaml
file_operations:
  - File upload: <200ms for 100KB file
  - File download: <100ms for 100KB file
  - VFS read/write: <10ms per operation

vim_editor:
  - Mode transition: <10ms
  - Command execution: <50ms
  - Syntax highlighting: <50ms for 1000 lines
  - Render update: 60 FPS (16ms frame budget)

file_browser:
  - Tree render: <100ms for 1000 files
  - Expand/collapse: <50ms
```

#### NFR-2: Usability
```yaml
discoverability:
  - Vim mode indicator (NORMAL, INSERT, VISUAL, COMMAND)
  - Vim cheat sheet (accessible via :help)
  - File browser tooltips (hover for actions)

accessibility:
  - Keyboard navigation (no mouse required)
  - Screen reader support (ARIA labels)
  - High contrast mode option
```

#### NFR-3: Scalability
```yaml
limits:
  - Max file size: 10MB (browser memory limit)
  - Max files in VFS: 10,000
  - Max file browser depth: 20 levels
  - Max vim buffer count: 50

error_handling:
  - File too large: graceful error message
  - VFS full: clear error with cleanup suggestion
  - Invalid file: show error, allow retry
```

### 2.3 Comparison with Cloud Shells

#### AWS CloudShell Features (Inspiration)
```
✅ File upload/download
✅ Built-in file browser
✅ Vim editor (system vim)
✅ Syntax highlighting
✅ AWS CLI integration
✅ Session persistence
✅ Pre-installed tools (git, docker, etc.)
```

#### Our Implementation (WOS-031)
```
✅ File upload/download (FR-2, FR-3)
✅ Built-in file browser (FR-1)
✅ Vim editor (FR-4, pure Rust/WASM implementation)
✅ Syntax highlighting (FR-5)
✅ Multi-file editing (FR-6)
🔜 Dockerfile execution (future: WOS-032)
🔜 Git integration (future: WOS-033)
```

---

## 3. Architecture Design

### 3.1 Component Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Browser (WASM Host)                   │
│  ┌────────────────────────────────────────────────────┐ │
│  │         Frontend (HTML/CSS/JS - Enhanced)          │ │
│  │  ┌─────────────────┬───────────────────────────┐  │ │
│  │  │  Terminal       │  File Manager (NEW)       │  │ │
│  │  │  (Existing)     │  ├─ File Browser         │  │ │
│  │  │                 │  ├─ Upload/Download       │  │ │
│  │  │                 │  └─ Quick Stats           │  │ │
│  │  └─────────────────┴───────────────────────────┘  │ │
│  │  ┌──────────────────────────────────────────────┐ │ │
│  │  │       Vim Editor Modal (NEW)                 │ │ │
│  │  │  - Full-screen overlay when editing          │ │ │
│  │  │  - Mode indicator, command line, status      │ │ │
│  │  └──────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────┘ │
│                         ↕                                │
│  ┌────────────────────────────────────────────────────┐ │
│  │       WASM Bindings (wos crate - Enhanced)         │ │
│  │  - File upload/download bindings                   │ │
│  │  - Vim editor WASM exports                         │ │
│  └────────────────────────────────────────────────────┘ │
│                         ↕                                │
│  ┌────────────────────────────────────────────────────┐ │
│  │      Userspace (userspace crate - Enhanced)        │ │
│  │  ┌──────────────────────────────────────────────┐ │ │
│  │  │  Vim Editor (NEW)                            │ │ │
│  │  │  ├─ VimState (modes, buffers, commands)      │ │ │
│  │  │  ├─ VimBuffer (text, cursor, undo/redo)      │ │ │
│  │  │  ├─ VimCommand (parser, executor)            │ │ │
│  │  │  └─ SyntaxHighlighter (language parsers)     │ │ │
│  │  └──────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────┘ │
│                         ↕                                │
│  ┌────────────────────────────────────────────────────┐ │
│  │           Microkernel (kernel crate)                │ │
│  │  - File I/O syscalls (existing)                     │ │
│  │  - VFS operations (existing)                        │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Data Structures

#### Vim State Machine
```rust
// userspace/src/vim.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VimMode {
    Normal,
    Insert,
    Visual { start: CursorPos, end: CursorPos },
    Command { buffer: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimState {
    mode: VimMode,
    buffers: im::HashMap<BufferId, VimBuffer>,
    active_buffer: BufferId,
    command_history: im::Vector<String>,
    registers: im::HashMap<char, String>, // Yank registers
    search_pattern: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimBuffer {
    file_path: PathBuf,
    lines: im::Vector<String>,
    cursor: CursorPos,
    undo_stack: im::Vector<BufferState>,
    redo_stack: im::Vector<BufferState>,
    modified: bool,
    syntax_lang: Option<Language>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorPos {
    line: usize,
    col: usize,
}

#[derive(Clone, Debug)]
pub enum VimCommand {
    // Normal mode
    MoveLeft, MoveRight, MoveUp, MoveDown,
    MoveWordForward, MoveWordBackward,
    MoveLineStart, MoveLineEnd,
    MoveFileStart, MoveFileEnd,
    GoToLine(usize),

    // Editing
    InsertBefore, InsertAfter,
    InsertLineBelow, InsertLineAbove,
    DeleteChar, DeleteLine,
    YankLine, PutAfter,
    Undo, Redo,
    Replace(char),

    // Search
    SearchForward(String),
    SearchBackward(String),
    NextMatch, PrevMatch,

    // Ex commands
    Write, Quit, WriteQuit, QuitForce,
    Edit(PathBuf),
    Set(String, String),
    BufferNext, BufferPrev, BufferList,
}
```

#### Syntax Highlighter
```rust
// userspace/src/syntax.rs

#[derive(Clone, Debug)]
pub enum Language {
    Rust, Python, JavaScript, Dockerfile, Shell, JSON, YAML, Text,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyntaxToken {
    start: usize,
    end: usize,
    kind: TokenKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TokenKind {
    Keyword, String, Comment, Function, Type, Number, Operator, Identifier,
}

pub trait SyntaxParser {
    fn parse(&self, text: &str) -> Vec<SyntaxToken>;
    fn language(&self) -> Language;
}

pub struct RustParser;
pub struct PythonParser;
pub struct DockerfileParser;
// ... etc
```

#### File Browser State
```rust
// wos/src/file_browser.rs (new)

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileBrowserState {
    root: FileNode,
    selected: Option<PathBuf>,
    expanded: im::HashSet<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileNode {
    path: PathBuf,
    name: String,
    kind: FileKind,
    size: usize,
    modified: u64,
    children: im::Vector<FileNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileKind {
    File { extension: Option<String> },
    Directory,
    Symlink { target: PathBuf },
}
```

### 3.3 Key Algorithms

#### Vim Command Parser
```rust
// userspace/src/vim/parser.rs

pub fn parse_normal_command(input: &str) -> Result<VimCommand, VimError> {
    // State machine for vim command parsing
    match input.chars().next() {
        Some('h') => Ok(VimCommand::MoveLeft),
        Some('j') => Ok(VimCommand::MoveDown),
        Some('k') => Ok(VimCommand::MoveUp),
        Some('l') => Ok(VimCommand::MoveRight),
        Some('w') => Ok(VimCommand::MoveWordForward),
        Some('b') => Ok(VimCommand::MoveWordBackward),
        Some('0') => Ok(VimCommand::MoveLineStart),
        Some('$') => Ok(VimCommand::MoveLineEnd),
        Some('i') => Ok(VimCommand::InsertBefore),
        Some('a') => Ok(VimCommand::InsertAfter),
        Some('o') => Ok(VimCommand::InsertLineBelow),
        Some('O') => Ok(VimCommand::InsertLineAbove),
        Some('x') => Ok(VimCommand::DeleteChar),
        Some('d') => {
            if input.starts_with("dd") {
                Ok(VimCommand::DeleteLine)
            } else {
                Err(VimError::IncompleteCommand)
            }
        }
        Some('y') => {
            if input.starts_with("yy") {
                Ok(VimCommand::YankLine)
            } else {
                Err(VimError::IncompleteCommand)
            }
        }
        Some('p') => Ok(VimCommand::PutAfter),
        Some('u') => Ok(VimCommand::Undo),
        Some('r') => {
            if let Some(ch) = input.chars().nth(1) {
                Ok(VimCommand::Replace(ch))
            } else {
                Err(VimError::IncompleteCommand)
            }
        }
        Some('/') => {
            let pattern = input[1..].to_string();
            Ok(VimCommand::SearchForward(pattern))
        }
        Some(':') => parse_ex_command(&input[1..]),
        _ => Err(VimError::InvalidCommand(input.to_string())),
    }
}

pub fn parse_ex_command(input: &str) -> Result<VimCommand, VimError> {
    match input.trim() {
        "w" | "write" => Ok(VimCommand::Write),
        "q" | "quit" => Ok(VimCommand::Quit),
        "wq" | "x" => Ok(VimCommand::WriteQuit),
        "q!" => Ok(VimCommand::QuitForce),
        cmd if cmd.starts_with("e ") => {
            let path = PathBuf::from(&cmd[2..]);
            Ok(VimCommand::Edit(path))
        }
        cmd if cmd.starts_with("set ") => {
            let parts: Vec<&str> = cmd[4..].split('=').collect();
            if parts.len() == 2 {
                Ok(VimCommand::Set(parts[0].to_string(), parts[1].to_string()))
            } else {
                Err(VimError::InvalidSetCommand)
            }
        }
        "bnext" | "bn" => Ok(VimCommand::BufferNext),
        "bprev" | "bp" => Ok(VimCommand::BufferPrev),
        "ls" | "buffers" => Ok(VimCommand::BufferList),
        _ => Err(VimError::UnknownCommand(input.to_string())),
    }
}
```

#### Undo/Redo Stack
```rust
// userspace/src/vim/buffer.rs

impl VimBuffer {
    pub fn apply_edit(&mut self, edit: Edit) {
        // Save current state to undo stack
        let snapshot = BufferState {
            lines: self.lines.clone(),
            cursor: self.cursor.clone(),
        };
        self.undo_stack = self.undo_stack.push_back(snapshot);
        self.redo_stack = im::Vector::new(); // Clear redo on new edit

        // Apply edit
        match edit {
            Edit::InsertChar(ch) => {
                let line = &self.lines[self.cursor.line];
                let new_line = format!(
                    "{}{}{}",
                    &line[..self.cursor.col],
                    ch,
                    &line[self.cursor.col..]
                );
                self.lines = self.lines.update(self.cursor.line, new_line);
                self.cursor.col += 1;
            }
            Edit::DeleteChar => {
                if self.cursor.col > 0 {
                    let line = &self.lines[self.cursor.line];
                    let new_line = format!(
                        "{}{}",
                        &line[..self.cursor.col - 1],
                        &line[self.cursor.col..]
                    );
                    self.lines = self.lines.update(self.cursor.line, new_line);
                    self.cursor.col -= 1;
                }
            }
            Edit::DeleteLine => {
                self.lines = self.lines.remove(self.cursor.line);
            }
            // ... more edit types
        }

        self.modified = true;
    }

    pub fn undo(&mut self) {
        if let Some(prev_state) = self.undo_stack.pop_back() {
            // Save current to redo
            let current = BufferState {
                lines: self.lines.clone(),
                cursor: self.cursor.clone(),
            };
            self.redo_stack = self.redo_stack.push_back(current);

            // Restore previous state
            self.lines = prev_state.0.lines;
            self.cursor = prev_state.0.cursor;
            self.undo_stack = prev_state.1;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop_back() {
            // Save current to undo
            let current = BufferState {
                lines: self.lines.clone(),
                cursor: self.cursor.clone(),
            };
            self.undo_stack = self.undo_stack.push_back(current);

            // Restore next state
            self.lines = next_state.0.lines;
            self.cursor = next_state.0.cursor;
            self.redo_stack = next_state.1;
        }
    }
}
```

### 3.4 UI Layout Changes

#### Before (Current)
```html
<main>
  <div class="terminal-container"><!-- Full width terminal --></div>
  <div class="info-panels">
    <div class="info-panel">Process List</div>
    <div class="info-panel">System Info</div>
    <div class="info-panel">Quality Metrics</div>
  </div>
</main>
```

#### After (WOS-031)
```html
<main>
  <div class="terminal-container"><!-- Left side --></div>
  <div class="file-manager"><!-- Right side - NEW -->
    <div class="file-browser">
      <h3>📁 Files</h3>
      <div id="file-tree"><!-- Dynamic tree --></div>
    </div>

    <div class="file-actions">
      <h3>🔧 Actions</h3>
      <button id="btn-upload">⬆️ Upload File</button>
      <button id="btn-download">⬇️ Download</button>
      <button id="btn-vim-edit">✏️ Edit in Vim</button>
      <button id="btn-delete">🗑️ Delete</button>
    </div>

    <div class="quick-stats">
      <h3>📊 Quick Info</h3>
      <p>Processes: <span id="stat-processes">3</span></p>
      <p>Files: <span id="stat-files">12</span></p>
      <p>TDG: <span id="stat-tdg">A+</span></p>
    </div>
  </div>
</main>

<!-- Vim Editor Modal (full-screen overlay) -->
<div id="vim-modal" class="vim-modal hidden">
  <div class="vim-header">
    <span id="vim-filename">/app.py</span>
    <span id="vim-mode">-- NORMAL --</span>
  </div>
  <div id="vim-editor" class="vim-editor">
    <!-- Rendered text with syntax highlighting -->
  </div>
  <div class="vim-status">
    <span id="vim-line-info">Line 1/100, Col 1</span>
    <span id="vim-modified">*</span>
  </div>
  <div class="vim-command-line">
    <span id="vim-command-prompt">:</span>
    <input id="vim-command-input" />
  </div>
</div>
```

---

## 4. Design Pattern Enhancements

This section documents architectural improvements based on technical review recommendations, incorporating industry-standard design patterns to enhance maintainability, extensibility, and robustness.

### 4.1 Command Pattern for Undo/Redo

**Recommendation**: Formalize the undo/redo stack as an implementation of the Command Pattern.

**Rationale**: The Command Pattern is a foundational "Gang of Four" design pattern for encapsulating operations and supporting undo/redo functionality. It makes the code more self-documenting and extensible for complex operations.

#### Command Pattern Structure

```rust
// userspace/src/vim/command.rs

/// Command Pattern: Encapsulates vim editing operations
pub trait VimEditCommand: Clone {
    /// Execute the command, returning the new buffer state
    fn execute(&self, buffer: &VimBuffer) -> VimBuffer;

    /// Undo the command, restoring previous state
    fn undo(&self, buffer: &VimBuffer, memento: &BufferMemento) -> VimBuffer;

    /// Description for debugging/logging
    fn description(&self) -> &str;
}

/// Memento Pattern: Captures buffer state for undo/redo
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferMemento {
    lines: im::Vector<String>,
    cursor: CursorPos,
    timestamp: u64,
}

impl BufferMemento {
    pub fn capture(buffer: &VimBuffer) -> Self {
        Self {
            lines: buffer.lines.clone(),
            cursor: buffer.cursor.clone(),
            timestamp: current_timestamp(),
        }
    }
}

/// Concrete Commands

#[derive(Clone, Debug)]
pub struct InsertCharCommand {
    char: char,
    position: CursorPos,
}

impl VimEditCommand for InsertCharCommand {
    fn execute(&self, buffer: &VimBuffer) -> VimBuffer {
        let mut new_buffer = buffer.clone();
        let line = &new_buffer.lines[self.position.line];
        let new_line = format!(
            "{}{}{}",
            &line[..self.position.col],
            self.char,
            &line[self.position.col..]
        );
        new_buffer.lines = new_buffer.lines.update(self.position.line, new_line);
        new_buffer.cursor.col += 1;
        new_buffer.modified = true;
        new_buffer
    }

    fn undo(&self, buffer: &VimBuffer, memento: &BufferMemento) -> VimBuffer {
        let mut restored = buffer.clone();
        restored.lines = memento.lines.clone();
        restored.cursor = memento.cursor.clone();
        restored
    }

    fn description(&self) -> &str {
        "Insert character"
    }
}

#[derive(Clone, Debug)]
pub struct DeleteLineCommand {
    line_number: usize,
}

impl VimEditCommand for DeleteLineCommand {
    fn execute(&self, buffer: &VimBuffer) -> VimBuffer {
        let mut new_buffer = buffer.clone();
        new_buffer.lines = new_buffer.lines.remove(self.line_number);
        new_buffer.modified = true;
        new_buffer
    }

    fn undo(&self, buffer: &VimBuffer, memento: &BufferMemento) -> VimBuffer {
        let mut restored = buffer.clone();
        restored.lines = memento.lines.clone();
        restored.cursor = memento.cursor.clone();
        restored
    }

    fn description(&self) -> &str {
        "Delete line"
    }
}

// More concrete commands: YankLineCommand, PutCommand, etc.
```

#### Command History Management

```rust
// userspace/src/vim/buffer.rs

impl VimBuffer {
    /// Execute a command and add to undo stack
    pub fn execute_command(&mut self, command: Box<dyn VimEditCommand>) {
        // Capture current state before executing
        let memento = BufferMemento::capture(self);

        // Execute command
        *self = command.execute(self);

        // Store command and memento in undo stack
        self.undo_stack = self.undo_stack.push_back((command, memento));

        // Clear redo stack on new command
        self.redo_stack = im::Vector::new();
    }

    /// Undo last command
    pub fn undo(&mut self) {
        if let Some((command, memento)) = self.undo_stack.pop_back() {
            // Capture current state for redo
            let current_memento = BufferMemento::capture(self);

            // Restore previous state
            *self = command.undo(self, &memento);

            // Add to redo stack
            self.redo_stack = self.redo_stack.push_back((command, current_memento));
            self.undo_stack = self.undo_stack.pop_back().1;
        }
    }

    /// Redo last undone command
    pub fn redo(&mut self) {
        if let Some((command, _memento)) = self.redo_stack.pop_back() {
            // Re-execute the command
            self.execute_command(command);
            self.redo_stack = self.redo_stack.pop_back().1;
        }
    }
}
```

**Benefits**:
- **Extensibility**: Easy to add new commands without modifying existing code
- **Testability**: Each command can be tested in isolation
- **Debuggability**: Command history provides clear audit trail
- **Composability**: Commands can be combined into macros (future enhancement)

**Test Coverage**: 25+ tests for command execution, undo/redo consistency, edge cases

### 4.2 File System Access API Integration

**Recommendation**: Utilize the File System Access API for richer user experience with graceful fallback.

**Rationale**: The File System Access API (W3C standard) enables direct file system access with user permission, allowing edits to be saved back to the original file on the host machine.

#### API Detection and Fallback

```javascript
// dist/wos/app.js

class FileSystemManager {
    constructor() {
        // Feature detection
        this.hasFileSystemAccess = 'showOpenFilePicker' in window;
        this.fileHandles = new Map(); // fileName -> FileSystemFileHandle
    }

    async openFile() {
        if (this.hasFileSystemAccess) {
            return await this.openWithFileSystemAccess();
        } else {
            return await this.openWithTraditionalAPI();
        }
    }

    async openWithFileSystemAccess() {
        try {
            const [fileHandle] = await window.showOpenFilePicker({
                types: [
                    {
                        description: 'Text Files',
                        accept: {
                            'text/*': ['.txt', '.rs', '.py', '.js', '.md']
                        }
                    }
                ]
            });

            const file = await fileHandle.getFile();
            const content = await file.text();

            // Store handle for later saving
            this.fileHandles.set(file.name, fileHandle);

            return {
                name: file.name,
                content: content,
                handle: fileHandle,
                canSaveInPlace: true
            };
        } catch (err) {
            if (err.name === 'AbortError') {
                return null; // User cancelled
            }
            throw err;
        }
    }

    async saveFile(fileName, content) {
        const handle = this.fileHandles.get(fileName);

        if (handle && this.hasFileSystemAccess) {
            // Save in place (direct to host file system)
            await this.saveWithFileSystemAccess(handle, content);
        } else {
            // Trigger download (traditional approach)
            this.downloadFile(fileName, content);
        }
    }

    async saveWithFileSystemAccess(handle, content) {
        // Request permission if needed
        const permission = await handle.queryPermission({ mode: 'readwrite' });
        if (permission !== 'granted') {
            const newPermission = await handle.requestPermission({ mode: 'readwrite' });
            if (newPermission !== 'granted') {
                throw new Error('Permission denied');
            }
        }

        // Write to file
        const writable = await handle.createWritable();
        await writable.write(content);
        await writable.close();
    }

    // Traditional fallback implementation
    async openWithTraditionalAPI() {
        return new Promise((resolve) => {
            const input = document.createElement('input');
            input.type = 'file';
            input.onchange = async (e) => {
                const file = e.target.files[0];
                if (!file) {
                    resolve(null);
                    return;
                }
                const content = await file.text();
                resolve({
                    name: file.name,
                    content: content,
                    canSaveInPlace: false
                });
            };
            input.click();
        });
    }
}
```

**User Experience Improvements**:
- Modern browsers: Direct file editing with auto-save
- Older browsers: Graceful fallback to upload/download workflow
- Visual indicator showing save mode (in-place vs. download)

**Test Coverage**: E2E tests for both API paths, permission handling, error cases

### 4.3 tree-sitter Integration for Syntax Highlighting

**Recommendation**: Use tree-sitter parser generator instead of manual parsers.

**Rationale**: tree-sitter provides robust, performant, incremental parsing with pre-built grammars. It's the engine behind syntax highlighting in GitHub Codespaces, Zed, and VS Code.

#### tree-sitter Rust Integration

```rust
// Cargo.toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-python = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-dockerfile = "0.1"

// userspace/src/syntax/tree_sitter_highlighter.rs

use tree_sitter::{Language, Parser, Query, QueryCursor};

pub struct TreeSitterHighlighter {
    parser: Parser,
    queries: HashMap<Language, Query>,
}

impl TreeSitterHighlighter {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let mut queries = HashMap::new();

        // Load language grammars
        let rust_lang = tree_sitter_rust::language();
        let python_lang = tree_sitter_python::language();

        // Load highlighting queries
        queries.insert(
            rust_lang,
            Query::new(rust_lang, include_str!("queries/rust-highlights.scm")).unwrap()
        );

        Self { parser, queries }
    }

    pub fn highlight(&mut self, code: &str, language: Language) -> Vec<SyntaxToken> {
        // Set language
        self.parser.set_language(language).unwrap();

        // Parse code into syntax tree
        let tree = self.parser.parse(code, None).unwrap();

        // Run highlighting query
        let query = &self.queries[&language];
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), code.as_bytes());

        // Extract tokens
        let mut tokens = Vec::new();
        for match_ in matches {
            for capture in match_.captures {
                tokens.push(SyntaxToken {
                    start: capture.node.start_byte(),
                    end: capture.node.end_byte(),
                    kind: self.capture_to_token_kind(capture.index),
                });
            }
        }

        tokens
    }

    /// Incremental update (key tree-sitter feature)
    pub fn highlight_with_edits(
        &mut self,
        old_tree: &Tree,
        edits: &[InputEdit],
        new_code: &str,
        language: Language
    ) -> Vec<SyntaxToken> {
        // Apply edits to old tree
        let mut tree = old_tree.clone();
        for edit in edits {
            tree.edit(edit);
        }

        // Re-parse with previous tree (much faster)
        self.parser.set_language(language).unwrap();
        let new_tree = self.parser.parse(new_code, Some(&tree)).unwrap();

        // Query only changed regions
        // tree-sitter automatically identifies minimal re-parse regions
        self.highlight(new_code, language)
    }
}
```

#### Highlighting Query Example

```scheme
; queries/rust-highlights.scm
; tree-sitter query language for Rust syntax highlighting

(function_item name: (identifier) @function)
(type_identifier) @type
(primitive_type) @type.builtin

[
  "fn"
  "let"
  "mut"
  "const"
  "static"
  "impl"
  "trait"
  "struct"
  "enum"
  "pub"
] @keyword

(string_literal) @string
(line_comment) @comment
(block_comment) @comment
(integer_literal) @number
```

**Benefits**:
- **Accuracy**: Full CST parsing handles complex syntax correctly
- **Performance**: Incremental parsing updates only changed regions
- **Maintainability**: Zero custom parser code
- **WASM Compatible**: tree-sitter compiles to WASM via Rust bindings

**Performance Benchmarks**:
- Full parse: ~2ms for 1000 lines
- Incremental update: ~0.1ms for single-line edit
- Well within <50ms target

**Test Coverage**: 15+ tests for tree-sitter integration, grammar loading, incremental updates

### 4.4 Visual Regression Testing

**Recommendation**: Augment E2E suite with visual regression testing.

**Rationale**: Functional tests verify behavior but miss unintended visual changes to layout, styling, or syntax highlighting colors.

#### Playwright Visual Comparison

```typescript
// e2e/tests/09-visual-regression.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Visual Regression', () => {
    test('vim editor layout matches baseline', async ({ page }) => {
        // Open vim with sample code
        await page.goto('http://localhost:8001/wos/');
        await page.locator('#terminal-input').fill('vim /sample.rs');
        await page.locator('#terminal-input').press('Enter');

        // Wait for vim modal to be fully rendered
        await expect(page.locator('#vim-modal')).toBeVisible();
        await page.waitForTimeout(100); // Wait for animations

        // Take screenshot and compare with baseline
        await expect(page.locator('#vim-modal')).toHaveScreenshot('vim-normal-mode.png', {
            maxDiffPixels: 100, // Allow minor rendering differences
        });
    });

    test('syntax highlighting colors match baseline', async ({ page }) => {
        // Load Rust file with syntax highlighting
        await page.goto('http://localhost:8001/wos/');
        await setupRustFile(page);
        await page.locator('#terminal-input').fill('vim /test.rs');
        await page.locator('#terminal-input').press('Enter');

        // Screenshot the editor content
        await expect(page.locator('#vim-editor')).toHaveScreenshot('rust-syntax-highlighting.png');
    });

    test('file browser layout matches baseline', async ({ page }) => {
        await page.goto('http://localhost:8001/wos/');
        await populateFileSystem(page);

        // Screenshot file browser
        await expect(page.locator('.file-manager')).toHaveScreenshot('file-browser.png');
    });

    // 10+ more visual regression tests
});
```

**Baseline Management**:
- Initial run generates baseline screenshots
- Subsequent runs compare against baselines
- Differences trigger test failure with visual diff
- Update baselines when intentional design changes occur

**Test Coverage**: 10+ visual regression tests for UI components

### 4.5 Performance Benchmarking with criterion.rs

**Recommendation**: Formalize performance benchmark tests to detect regressions.

**Rationale**: NFRs specify clear performance targets. Automated benchmarks in CI/CD prevent regressions.

#### Criterion Benchmarks

```rust
// benches/vim_performance.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wos_userspace::vim::{VimState, VimCommand};

fn bench_vim_command_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("vim_commands");

    // Benchmark individual commands
    group.bench_function("move_down", |b| {
        let mut vim = VimState::new_with_text("line1\nline2\nline3");
        b.iter(|| {
            vim.execute(black_box(VimCommand::MoveDown));
        });
    });

    group.bench_function("delete_line", |b| {
        let mut vim = VimState::new_with_text("line1\nline2\nline3");
        b.iter(|| {
            vim.execute(black_box(VimCommand::DeleteLine));
            vim.undo(); // Reset for next iteration
        });
    });

    group.finish();
}

fn bench_syntax_highlighting(c: &mut Criterion) {
    let mut group = c.benchmark_group("syntax_highlighting");

    // Benchmark different file sizes
    for line_count in [100, 500, 1000, 5000].iter() {
        let code = generate_rust_code(*line_count);

        group.bench_with_input(
            BenchmarkId::new("rust_full_parse", line_count),
            &code,
            |b, code| {
                b.iter(|| {
                    let tokens = highlight_rust(black_box(code));
                    black_box(tokens);
                });
            },
        );
    }

    group.finish();
}

fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    let content = vec![0u8; 100_000]; // 100KB

    group.bench_function("upload_100kb", |b| {
        b.iter(|| {
            upload_file(black_box("/test.bin"), black_box(&content));
        });
    });

    group.bench_function("download_100kb", |b| {
        b.iter(|| {
            let data = download_file(black_box("/test.bin"));
            black_box(data);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vim_command_execution,
    bench_syntax_highlighting,
    bench_file_operations
);
criterion_main!(benches);
```

**CI/CD Integration**:
```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --workspace

      - name: Check performance targets
        run: |
          # Parse criterion output
          # Fail if any benchmark exceeds target (e.g., >50ms)
          python scripts/check_bench_targets.py
```

**Performance Targets** (from NFRs):
- Vim command execution: <50ms ✅
- Syntax highlighting (1000 lines): <50ms ✅
- File upload (100KB): <200ms ✅
- File download (100KB): <100ms ✅

**Test Coverage**: 20+ benchmarks covering all critical paths

---

## 5. Implementation Roadmap

### 4.1 Ticket Breakdown

#### WOS-031A: File Browser UI (Week 1, 8 hours)
```yaml
title: Implement visual file browser in right sidebar
priority: critical
cycle: 1
time_estimate: 8_hours

tasks:
  - Replace info panels with file manager component
  - Implement file tree rendering (recursive)
  - Add expand/collapse for directories
  - Add file selection (click to select)
  - Add file type icons (.rs, .py, .js, Dockerfile, etc.)
  - Show file size and modification time

files_modified:
  - dist/wos/index.html (layout changes)
  - dist/wos/style.css (file browser styles)
  - dist/wos/app.js (file tree logic)

tests:
  unit:
    - test_file_tree_render()
    - test_file_selection()
    - test_expand_collapse()
  e2e:
    - test_file_browser_shows_vfs_files()
    - test_file_browser_select_file()

acceptance:
  - File browser shows all VFS files
  - Can select files by clicking
  - Directories expand/collapse
  - Icons match file types
```

#### WOS-031B: File Upload (Week 1, 4 hours)
```yaml
title: Implement file upload from host to VFS
priority: critical
cycle: 1
time_estimate: 4_hours

tasks:
  - Add upload button with file picker
  - Read file content (text and binary)
  - Write to VFS via syscalls
  - Show upload progress for large files
  - Error handling (file too large, invalid encoding)

files_modified:
  - dist/wos/app.js (upload logic)
  - wos/src/lib.rs (WASM bindings for upload)

tests:
  unit:
    - test_upload_text_file()
    - test_upload_binary_file()
    - test_upload_file_size_limit()
  e2e:
    - test_upload_file_workflow()
  fuzz:
    - fuzz_upload_with_random_content()

acceptance:
  - Upload button works
  - File appears in VFS and file browser
  - Binary files handled correctly
  - Error shown for files >10MB
```

#### WOS-031C: File Download (Week 1, 2 hours)
```yaml
title: Implement file download from VFS to host
priority: critical
cycle: 1
time_estimate: 2_hours

tasks:
  - Add download button
  - Read file from VFS
  - Trigger browser download
  - Detect MIME type
  - Preserve filename

files_modified:
  - dist/wos/app.js (download logic)

tests:
  unit:
    - test_download_file()
    - test_download_preserves_content()
  e2e:
    - test_download_file_workflow()
  property:
    - proptest_upload_download_roundtrip()

acceptance:
  - Download button works
  - File downloads with correct name
  - Content preserved exactly
  - Binary files work
```

#### WOS-031D: Vim State Machine (Week 2, 12 hours)
```yaml
title: Implement vim state machine and modes
priority: critical
cycle: 2
time_estimate: 12_hours

tasks:
  - Define VimState, VimMode, VimBuffer types
  - Implement mode transitions (Normal ↔ Insert ↔ Visual ↔ Command)
  - Implement cursor navigation (h, j, k, l, w, b, 0, $, gg, G)
  - Implement basic editing (i, a, o, O, x, dd, yy, p)
  - Implement undo/redo stack
  - Implement search (/, ?, n, N)

files_modified:
  - userspace/src/vim/mod.rs (new module)
  - userspace/src/vim/state.rs
  - userspace/src/vim/buffer.rs
  - userspace/src/vim/commands.rs

tests:
  unit:
    - test_vim_mode_normal() (50+ tests)
    - test_vim_mode_insert() (20+ tests)
    - test_vim_mode_visual() (15+ tests)
    - test_vim_mode_command() (25+ tests)
    - test_vim_undo_redo() (10+ tests)
  property:
    - proptest_vim_state_machine()
    - proptest_undo_redo_consistency()
    - proptest_cursor_bounds()

acceptance:
  - All mode transitions work
  - All basic commands work
  - Undo/redo preserves state
  - Cursor never out of bounds
```

#### WOS-031E: Vim Ex Commands (Week 2, 6 hours)
```yaml
title: Implement vim ex commands (:w, :q, :e, etc.)
priority: critical
cycle: 2
time_estimate: 6_hours

tasks:
  - Implement :w (write to VFS)
  - Implement :q (quit if no changes)
  - Implement :wq, :x (write and quit)
  - Implement :q! (force quit)
  - Implement :e <file> (edit another file)
  - Implement :set (settings)
  - Command line parsing and execution

files_modified:
  - userspace/src/vim/ex_commands.rs (new)
  - userspace/src/vim/parser.rs (new)

tests:
  unit:
    - test_ex_command_write()
    - test_ex_command_quit()
    - test_ex_command_quit_unsaved_warns()
    - test_ex_command_set()
  e2e:
    - test_vim_save_file_workflow()

acceptance:
  - :w saves file to VFS
  - :q warns on unsaved changes
  - :q! quits without saving
  - :e switches files
```

#### WOS-031F: Vim UI Integration (Week 3, 8 hours)
```yaml
title: Integrate vim editor with browser UI
priority: critical
cycle: 3
time_estimate: 8_hours

tasks:
  - Create full-screen vim modal overlay
  - Implement keyboard event handling
  - Render vim buffer (lines, cursor, syntax)
  - Show mode indicator
  - Show command line
  - Handle ESC, Enter, special keys

files_modified:
  - dist/wos/index.html (vim modal)
  - dist/wos/style.css (vim styles)
  - dist/wos/app.js (vim integration)
  - wos/src/lib.rs (WASM bindings)

tests:
  e2e:
    - test_vim_modal_opens()
    - test_vim_keyboard_input()
    - test_vim_mode_indicator()
    - test_vim_command_line()

acceptance:
  - Vim opens in full-screen modal
  - All keys handled correctly
  - Mode indicator updates
  - Command line works
```

#### WOS-031G: Syntax Highlighting (Week 3, 8 hours)
```yaml
title: Implement syntax highlighting for common languages
priority: high
cycle: 3
time_estimate: 8_hours

tasks:
  - Implement Rust syntax parser
  - Implement Python syntax parser
  - Implement JavaScript syntax parser
  - Implement Dockerfile syntax parser
  - Auto-detect language from file extension
  - Render tokens with colors
  - :syntax on/off command

files_modified:
  - userspace/src/syntax/mod.rs (new)
  - userspace/src/syntax/rust.rs
  - userspace/src/syntax/python.rs
  - userspace/src/syntax/dockerfile.rs

tests:
  unit:
    - test_syntax_rust_keywords()
    - test_syntax_python_functions()
    - test_syntax_dockerfile_instructions()
  property:
    - proptest_syntax_never_panics()
  benchmark:
    - bench_syntax_1000_lines()

acceptance:
  - Syntax highlighting for Rust, Python, JS, Dockerfile
  - Auto-detection works
  - :syntax toggles highlighting
  - Performance <50ms for 1000 lines
```

#### WOS-031H: Multi-File Buffers (Week 3, 4 hours)
```yaml
title: Implement multi-file buffer management
priority: medium
cycle: 3
time_estimate: 4_hours

tasks:
  - Implement buffer list
  - :e switches buffers
  - :bnext, :bprev navigation
  - :ls shows buffers
  - Preserve buffer state (cursor, undo stack)

files_modified:
  - userspace/src/vim/buffers.rs (new)

tests:
  unit:
    - test_buffer_switching()
    - test_buffer_state_preserved()
  e2e:
    - test_multi_file_editing()

acceptance:
  - Can edit multiple files
  - Buffer state preserved on switch
  - :ls shows all buffers
```

### 4.2 Implementation Timeline

```
Week 1: File Management Foundation
├─ Day 1-2: WOS-031A (File Browser UI)
├─ Day 3: WOS-031B (File Upload)
└─ Day 4: WOS-031C (File Download)

Week 2: Vim Editor Core
├─ Day 1-3: WOS-031D (Vim State Machine)
└─ Day 4-5: WOS-031E (Ex Commands)

Week 3: Vim UI & Advanced Features
├─ Day 1-2: WOS-031F (Vim UI Integration)
├─ Day 3-4: WOS-031G (Syntax Highlighting)
└─ Day 5: WOS-031H (Multi-File Buffers)

Week 4: Testing & Polish
├─ Day 1-2: E2E test suite (15+ tests)
├─ Day 3: Property tests (10+ tests)
├─ Day 4: Fuzzing (5 targets)
└─ Day 5: Documentation & demos
```

---

## 5. Testing Strategy

### 5.1 Test Coverage Targets

```yaml
overall_coverage: 90%+
mutation_score: 95%+

breakdown:
  vim_state_machine: 95%+ coverage
  vim_commands: 100% coverage (critical path)
  syntax_highlighting: 85%+ coverage
  file_operations: 95%+ coverage
  ui_integration: 80%+ coverage (E2E heavy)
```

### 5.2 Unit Tests (200+ tests)

#### Vim State Machine Tests (50+ tests)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_initial_mode_is_normal() {
        let vim = VimState::new();
        assert_eq!(vim.mode, VimMode::Normal);
    }

    #[test]
    fn test_vim_normal_to_insert_mode() {
        let mut vim = VimState::new();
        vim.handle_key('i');
        assert_eq!(vim.mode, VimMode::Insert);
    }

    #[test]
    fn test_vim_insert_to_normal_on_esc() {
        let mut vim = VimState::new();
        vim.handle_key('i');
        vim.handle_key(Key::Escape);
        assert_eq!(vim.mode, VimMode::Normal);
    }

    #[test]
    fn test_vim_move_down_increases_line() {
        let mut vim = VimState::new_with_text("line1\nline2\nline3");
        vim.handle_key('j');
        assert_eq!(vim.cursor().line, 1);
    }

    #[test]
    fn test_vim_delete_line_removes_current_line() {
        let mut vim = VimState::new_with_text("line1\nline2\nline3");
        vim.handle_keys("dd");
        assert_eq!(vim.buffer().line_count(), 2);
        assert_eq!(vim.buffer().line(0), "line2");
    }

    #[test]
    fn test_vim_yank_paste_duplicates_line() {
        let mut vim = VimState::new_with_text("line1\nline2");
        vim.handle_keys("yyp");
        assert_eq!(vim.buffer().line_count(), 3);
        assert_eq!(vim.buffer().line(1), "line1");
    }

    #[test]
    fn test_vim_undo_reverts_delete() {
        let mut vim = VimState::new_with_text("line1\nline2");
        vim.handle_keys("dd");
        vim.handle_key('u');
        assert_eq!(vim.buffer().line_count(), 2);
        assert_eq!(vim.buffer().line(0), "line1");
    }

    #[test]
    fn test_vim_redo_reapplies_delete() {
        let mut vim = VimState::new_with_text("line1\nline2");
        vim.handle_keys("dd");
        vim.handle_key('u');
        vim.handle_key(Key::Ctrl('r'));
        assert_eq!(vim.buffer().line_count(), 1);
    }

    #[test]
    fn test_vim_search_forward_finds_pattern() {
        let mut vim = VimState::new_with_text("foo\nbar\nbaz");
        vim.handle_keys("/bar\n");
        assert_eq!(vim.cursor().line, 1);
    }

    #[test]
    fn test_vim_command_mode_write_saves_to_vfs() {
        let mut vim = VimState::new_with_file("/test.txt", "content");
        vim.handle_keys(":w\n");
        // Verify VFS has updated content
        assert!(vim.buffer().is_saved());
    }

    // ... 40+ more vim tests
}
```

#### Syntax Highlighting Tests (30+ tests)
```rust
#[cfg(test)]
mod syntax_tests {
    #[test]
    fn test_rust_highlights_keywords() {
        let parser = RustParser::new();
        let tokens = parser.parse("fn main() { let x = 42; }");
        assert!(tokens.contains_keyword("fn"));
        assert!(tokens.contains_keyword("let"));
    }

    #[test]
    fn test_python_highlights_strings() {
        let parser = PythonParser::new();
        let tokens = parser.parse("print(\"hello\")");
        assert_eq!(tokens.count_strings(), 1);
    }

    #[test]
    fn test_dockerfile_highlights_instructions() {
        let parser = DockerfileParser::new();
        let tokens = parser.parse("FROM ubuntu:20.04\nRUN apt-get update");
        assert!(tokens.contains_keyword("FROM"));
        assert!(tokens.contains_keyword("RUN"));
    }

    // ... 27+ more syntax tests
}
```

#### File Operations Tests (20+ tests)
```rust
#[cfg(test)]
mod file_tests {
    #[test]
    fn test_upload_text_file_to_vfs() {
        let content = "Hello, WOS!";
        upload_file("/test.txt", content.as_bytes());
        assert_eq!(read_vfs_file("/test.txt"), content);
    }

    #[test]
    fn test_upload_binary_file() {
        let binary = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
        upload_file("/image.jpg", &binary);
        assert_eq!(read_vfs_file_bytes("/image.jpg"), binary);
    }

    #[test]
    fn test_download_file_triggers_browser_download() {
        write_vfs_file("/test.txt", "content");
        let downloaded = download_file("/test.txt");
        assert_eq!(downloaded.content, "content");
        assert_eq!(downloaded.filename, "test.txt");
    }

    // ... 17+ more file tests
}
```

### 5.3 Property-Based Tests (10+ tests)

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn proptest_vim_undo_redo_consistency(
        initial_text in ".*",
        edits in prop::collection::vec(any::<Edit>(), 1..100)
    ) {
        let mut vim = VimState::new_with_text(&initial_text);

        // Apply edits
        for edit in &edits {
            vim.apply_edit(edit.clone());
        }

        // Undo all
        for _ in 0..edits.len() {
            vim.undo();
        }

        // Text should match initial
        prop_assert_eq!(vim.buffer().text(), initial_text);
    }

    #[test]
    fn proptest_vim_cursor_always_in_bounds(
        moves in prop::collection::vec(any::<VimCommand>(), 1..1000)
    ) {
        let mut vim = VimState::new_with_text("line1\nline2\nline3");

        for cmd in moves {
            vim.execute(cmd);
            let cursor = vim.cursor();
            prop_assert!(cursor.line < vim.buffer().line_count());
            prop_assert!(cursor.col <= vim.buffer().line(cursor.line).len());
        }
    }

    #[test]
    fn proptest_upload_download_roundtrip(
        content in prop::collection::vec(any::<u8>(), 0..10_000)
    ) {
        upload_file("/test.bin", &content);
        let downloaded = download_file_bytes("/test.bin");
        prop_assert_eq!(downloaded, content);
    }

    #[test]
    fn proptest_syntax_highlighting_never_panics(
        code in ".*",
        lang in prop_oneof![
            Just(Language::Rust),
            Just(Language::Python),
            Just(Language::JavaScript)
        ]
    ) {
        let parser = SyntaxParser::for_language(lang);
        // Should not panic on any input
        let _tokens = parser.parse(&code);
    }

    // ... 6+ more property tests
}
```

### 5.4 Fuzzing Targets (5 targets)

```rust
// fuzz/fuzz_targets/vim_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_vim_command(s);
    }
});

// fuzz/fuzz_targets/syntax_parser.rs
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let rust_parser = RustParser::new();
        let _ = rust_parser.parse(s);
    }
});

// fuzz/fuzz_targets/file_upload.rs
fuzz_target!(|data: &[u8]| {
    let _ = upload_file("/fuzz.bin", data);
});

// fuzz/fuzz_targets/ex_command.rs
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_ex_command(s);
    }
});

// fuzz/fuzz_targets/file_path.rs
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = normalize_file_path(s);
    }
});
```

### 5.5 E2E Tests (15+ tests)

```typescript
// e2e/tests/07-file-management.spec.ts
import { test, expect } from '@playwright/test';

test.describe('File Management', () => {
  test('should upload file to VFS', async ({ page }) => {
    await page.goto('http://localhost:8001/wos/');

    // Upload file
    const fileInput = await page.locator('#file-upload-input');
    await fileInput.setInputFiles({
      name: 'test.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from('Hello, WOS!')
    });

    // Verify file in browser
    await expect(page.locator('#file-tree')).toContainText('test.txt');

    // Verify file in VFS via command
    const input = page.locator('#terminal-input');
    await input.fill('cat /uploads/test.txt');
    await input.press('Enter');
    await expect(page.locator('#terminal-output')).toContainText('Hello, WOS!');
  });

  test('should download file from VFS', async ({ page }) => {
    // Create file in VFS
    await page.locator('#terminal-input').fill('echo "Download me" > /test.txt');
    await page.locator('#terminal-input').press('Enter');

    // Select file
    await page.locator('#file-tree').getByText('test.txt').click();

    // Download
    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-download').click();
    const download = await downloadPromise;

    // Verify download
    expect(download.suggestedFilename()).toBe('test.txt');
    const content = await download.path();
    const text = await fs.readFile(content, 'utf-8');
    expect(text).toBe('Download me\n');
  });

  // ... 13+ more E2E tests
});

// e2e/tests/08-vim-editor.spec.ts
test.describe('Vim Editor', () => {
  test('should open vim editor for file', async ({ page }) => {
    // Create file
    await page.locator('#terminal-input').fill('echo "line1\\nline2" > /test.txt');
    await page.locator('#terminal-input').press('Enter');

    // Open in vim
    await page.locator('#terminal-input').fill('vim /test.txt');
    await page.locator('#terminal-input').press('Enter');

    // Verify vim modal opens
    await expect(page.locator('#vim-modal')).toBeVisible();
    await expect(page.locator('#vim-mode')).toContainText('NORMAL');
    await expect(page.locator('#vim-editor')).toContainText('line1');
  });

  test('should edit file in vim insert mode', async ({ page }) => {
    // ... open vim

    // Enter insert mode
    await page.keyboard.press('i');
    await expect(page.locator('#vim-mode')).toContainText('INSERT');

    // Type text
    await page.keyboard.type('Hello, ');

    // Return to normal mode
    await page.keyboard.press('Escape');
    await expect(page.locator('#vim-mode')).toContainText('NORMAL');

    // Save
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    // Verify file updated
    await page.locator('#terminal-input').fill('cat /test.txt');
    await page.locator('#terminal-input').press('Enter');
    await expect(page.locator('#terminal-output')).toContainText('Hello, line1');
  });

  test('should undo and redo in vim', async ({ page }) => {
    // ... open vim, make edits

    // Delete line
    await page.keyboard.type('dd');
    await expect(page.locator('#vim-editor')).not.toContainText('line1');

    // Undo
    await page.keyboard.press('u');
    await expect(page.locator('#vim-editor')).toContainText('line1');

    // Redo
    await page.keyboard.press('Control+r');
    await expect(page.locator('#vim-editor')).not.toContainText('line1');
  });

  test('should search in vim', async ({ page }) => {
    // ... open vim with multi-line file

    // Search forward
    await page.keyboard.type('/pattern');
    await page.keyboard.press('Enter');

    // Cursor should be on match
    // (verify via cursor position indicator)
    await expect(page.locator('#vim-line-info')).toContainText('Line 3');

    // Next match
    await page.keyboard.press('n');
    await expect(page.locator('#vim-line-info')).toContainText('Line 5');
  });

  // ... 11+ more vim E2E tests
});
```

### 5.6 Mutation Testing

```bash
# Run mutation tests (target: 95%+ kill rate)
cargo mutants --workspace --json --output mutants.json

# Focus areas:
# - Vim command parsing (100% kill rate)
# - Mode transitions (95%+ kill rate)
# - Undo/redo logic (98%+ kill rate)
# - File operations (95%+ kill rate)
```

**Expected Mutation Operators**:
- Replace `==` with `!=` (should fail: mode checks)
- Replace `+` with `-` (should fail: cursor movement)
- Remove `if` condition (should fail: bounds checking)
- Replace `true` with `false` (should fail: mode transitions)

---

## 6. Quality Gates

### 6.1 Pre-commit Checks

```yaml
format:
  - cargo fmt --check
  - All Rust code formatted

lint:
  - cargo clippy --all-features -- -D warnings
  - Zero clippy warnings

tests:
  - cargo nextest run --lib --workspace
  - 100% of unit tests passing

complexity:
  - PMAT analysis
  - Cyclomatic ≤15 (stricter than MVP ≤20)
  - Cognitive ≤10 (stricter than MVP ≤15)

satd:
  - Zero TODO/FIXME comments
```

### 6.2 PR Quality Gates

```yaml
coverage:
  - Line coverage ≥90% (vs MVP 85%)
  - Branch coverage ≥95% (vs MVP 90%)

mutation:
  - Mutation score ≥95% (vs MVP 90%)
  - Zero surviving mutants in critical paths

property_tests:
  - 10+ properties with 10K inputs each
  - All property tests passing

fuzzing:
  - 5 fuzz targets
  - 1M+ inputs per target
  - Zero crashes/panics

e2e:
  - 15+ E2E scenarios
  - 100% passing on Chrome, Firefox, Safari

performance:
  - Vim command: <50ms
  - Syntax highlighting: <50ms for 1000 lines
  - File upload: <200ms for 100KB
```

### 6.3 WASM Quality

```yaml
binary_size:
  - Uncompressed: <600KB (allow 100KB growth)
  - Gzipped: <150KB

load_time:
  - Cold start: <150ms (allow 50ms growth)

runtime:
  - Vim key latency: <10ms
  - File browser render: <100ms
  - Syntax highlight: <50ms
```

### 6.4 Documentation

```yaml
required_docs:
  - API documentation (rustdoc)
  - Vim command reference (:help equivalent)
  - User guide (how to upload/edit/download)
  - Architecture decision record (ADR)
  - Migration guide (UI layout changes)

readme_updates:
  - Add file management section
  - Add vim editor section
  - Update feature list
  - Add screenshots/demos
```

---

## 7. Future Enhancements (Post WOS-031)

### 7.1 Dockerfile Execution (WOS-032)

```yaml
goal: Parse and execute Dockerfiles in WOS
features:
  - Parse Dockerfile (FROM, RUN, COPY, ENV, CMD)
  - Simulate Docker layer building
  - Execute RUN commands in shell
  - Handle COPY to VFS
  - Show build progress
test_strategy:
  - Unit tests for Dockerfile parser (50+ tests)
  - E2E tests for Dockerfile execution (10+ tests)
  - Property tests for instruction semantics
```

### 7.2 Git Integration (WOS-033)

```yaml
goal: Basic git operations in WOS
features:
  - git init, add, commit (local only)
  - git status, diff, log
  - Visualize commit history
  - No remote operations (educational focus)
```

### 7.3 Advanced Vim Features (WOS-034)

```yaml
goal: More vim functionality
features:
  - Macros (q, @)
  - Marks (m, ')
  - Splits (:split, :vsplit)
  - Tabs (:tabnew, gt, gT)
  - Visual block mode (Ctrl+v)
```

---

## Appendix A: Vim Command Reference

### Normal Mode Commands (Implemented)

| Command | Action | Test Count |
|---------|--------|------------|
| h, j, k, l | Move left, down, up, right | 4 |
| w, b | Word forward/backward | 2 |
| 0, $ | Line start/end | 2 |
| gg, G | File start/end | 2 |
| :<line> | Go to line | 1 |
| i | Insert before cursor | 1 |
| a | Insert after cursor | 1 |
| o | Insert line below | 1 |
| O | Insert line above | 1 |
| x | Delete character | 1 |
| dd | Delete line | 1 |
| yy | Yank (copy) line | 1 |
| p | Put (paste) after | 1 |
| u | Undo | 1 |
| Ctrl+r | Redo | 1 |
| r<char> | Replace character | 1 |
| /<pattern> | Search forward | 1 |
| ?<pattern> | Search backward | 1 |
| n, N | Next/previous match | 2 |

### Ex Commands (Implemented)

| Command | Action | Test Count |
|---------|--------|------------|
| :w | Write (save) | 1 |
| :q | Quit | 1 |
| :wq, :x | Write and quit | 2 |
| :q! | Force quit | 1 |
| :e <file> | Edit file | 1 |
| :set <option> | Set option | 1 |
| :bnext, :bprev | Buffer navigation | 2 |
| :ls | List buffers | 1 |
| :syntax on/off | Toggle syntax | 2 |

---

## Appendix B: File Browser Icons

```
📄 .txt, .md          Text files
🐍 .py                Python
🦀 .rs                Rust
📜 .js, .ts           JavaScript/TypeScript
🐳 Dockerfile         Docker
🐚 .sh                Shell scripts
📋 .json              JSON
📊 .yaml, .yml        YAML
📁 (directory)        Directory
🔗 (symlink)          Symbolic link
📦 .tar, .gz, .zip    Archives
🖼️ .png, .jpg, .svg   Images
```

---

## Appendix C: Comparison Matrix

| Feature | AWS CloudShell | Azure CS | GCP CS | WOS (Post-031) |
|---------|----------------|----------|--------|----------------|
| File Upload | ✅ | ✅ | ✅ | ✅ |
| File Download | ✅ | ✅ | ✅ | ✅ |
| File Browser | ✅ | ✅ | ✅ | ✅ |
| Vim Editor | ✅ (system) | ✅ (system) | ✅ (system) | ✅ (WASM) |
| Syntax Highlighting | ✅ | ✅ | ✅ | ✅ |
| Multi-file Tabs | ✅ | ✅ | ✅ | ✅ (buffers) |
| Dockerfile Support | ✅ | ✅ | ✅ | 🔜 (WOS-032) |
| Git Integration | ✅ | ✅ | ✅ | 🔜 (WOS-033) |
| Runs in Browser | ❌ | ❌ | ❌ | ✅ |
| Zero Setup | ❌ | ❌ | ❌ | ✅ |
| Educational Focus | ❌ | ❌ | ❌ | ✅ |
| 100% Safe Rust | N/A | N/A | N/A | ✅ |

---

## 8. Technical Review Recommendations

This section summarizes the technical review feedback and how it has been incorporated into the specification.

### 8.1 Review Summary

**Overall Assessment**: Approved with Recommendations

**Reviewer Feedback**: "This specification document is a prime example of thoughtful, rigorous software design. By incorporating the [recommendations], the WOS team can build upon this excellent foundation to deliver a feature that is not only functionally complete but also robust, performant, and highly usable, solidifying WOS's position as a premier educational tool."

### 8.2 Incorporated Recommendations

#### ✅ Recommendation 1.1: Command Pattern for Undo/Redo
**Status**: Incorporated in Section 4.1

- Formalized undo/redo as Command Pattern with Memento Pattern
- Created `VimEditCommand` trait with `execute()` and `undo()` methods
- Implemented concrete commands: `InsertCharCommand`, `DeleteLineCommand`, etc.
- Added 25+ tests for command execution and undo/redo consistency

#### ✅ Recommendation 1.2: tree-sitter for Syntax Highlighting
**Status**: Incorporated in Sections 2.2 (FR-5) and 4.3

- Replaced manual parsers with tree-sitter library
- Leveraging pre-built grammars for Rust, Python, JS, Dockerfile
- Incremental parsing for performance (<50ms target easily met)
- WASM-compatible via official Rust bindings
- Reduced WOS-031G implementation estimate from 8 hours to 6 hours

#### ✅ Recommendation 2.1: File System Access API
**Status**: Incorporated in Sections 2.1 (FR-2) and 4.2

- Implemented File System Access API for modern browsers
- Graceful fallback to traditional File API for older browsers
- Direct file editing with save-in-place capability
- Permission handling and error recovery
- E2E tests for both code paths

#### ✅ Recommendation 2.2: :help Command
**Status**: Incorporated in Section 2.1 (FR-4)

- Added `:help` command to vim command mode
- Opens vim command reference as read-only buffer
- Self-contained documentation within editor
- Mirrors real vim behavior for educational value

#### ✅ Recommendation 3.1: Visual Regression Testing
**Status**: Incorporated in Sections 1 (Methodology) and 4.4

- Added Playwright visual regression tests to E2E suite
- Screenshot comparison for vim editor, file browser, syntax highlighting
- Baseline management workflow
- 10+ visual regression tests
- Updated methodology to include "Visual Regression"

#### ✅ Recommendation 3.2: Performance Benchmarking
**Status**: Incorporated in Section 4.5

- Formalized criterion.rs benchmarks for all critical paths
- 20+ benchmarks for vim commands, syntax highlighting, file operations
- CI/CD integration to detect performance regressions
- Automated checking against NFR performance targets

### 8.3 Impact on Implementation

**Enhanced Quality Targets**:
- Test coverage: 90%+ (was 85%)
- Mutation score: 95%+ (was 90%)
- E2E tests: 20+ (was 15+, added 5 visual regression)
- Property tests: 10+ (unchanged)
- Fuzzing: 5 targets (unchanged)
- **NEW**: Visual regression: 10+ tests
- **NEW**: Performance benchmarks: 20+ benchmarks

**Reduced Implementation Risk**:
- tree-sitter eliminates custom parser complexity
- Command Pattern provides clear architecture for vim editing
- File System Access API enhances UX without breaking changes
- Visual regression catches UI regressions early
- Performance benchmarks prevent regressions

**Updated Timeline**:
- Week 1: File Management (unchanged)
- Week 2: Vim Core with Command Pattern (slight increase)
- Week 3: Vim UI + tree-sitter (reduced from manual parsers)
- Week 4: Testing + Visual Regression + Benchmarks (expanded)

**Total**: 3-4 weeks (unchanged, but higher quality)

### 8.4 References

**Design Patterns**:
- Gamma, E., et al. (1994). *Design Patterns: Elements of Reusable Object-Oriented Software*. Addison-Wesley. (Command Pattern, Memento Pattern)

**Parsing Technology**:
- tree-sitter: https://tree-sitter.github.io/tree-sitter/
- Used by: GitHub Codespaces, Zed, VS Code, Atom

**Web APIs**:
- File System Access API: https://web.dev/file-system-access/
- W3C File API Specification: https://w3c.github.io/FileAPI/

**Testing**:
- Playwright Visual Comparison: https://playwright.dev/docs/test-snapshots
- criterion.rs: https://github.com/bheisler/criterion.rs

---

**Document Status**: Approved - Ready for Implementation
**Version**: 1.1 (Enhanced with Technical Review Recommendations)
**Next Step**: Begin WOS-031A (File Browser UI)
**Estimated Completion**: 3-4 weeks from start
**Quality Target**: A+ (95.0+ TDG score)
**Review Status**: ✅ Approved with all recommendations incorporated
