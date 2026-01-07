//! Vim editor implementation for WOS
//!
//! Modal text editor with basic vim keybindings.

use crate::dom::Dom;
use crate::tracer::{TraceCategory, Tracer};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Vim editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal mode (navigation)
    Normal,
    /// Insert mode (typing)
    Insert,
    /// Visual mode (selection)
    Visual,
    /// Command mode (:commands)
    Command,
}

impl VimMode {
    /// Get display name
    #[must_use]
    pub const fn display(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::Command => "COMMAND",
        }
    }
}

/// Vim editor
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VimEditor {
    /// Current mode
    mode: VimMode,
    /// Current file being edited
    filename: Option<String>,
    /// Buffer contents (lines)
    buffer: Vec<String>,
    /// Cursor row (0-indexed)
    cursor_row: usize,
    /// Cursor column (0-indexed)
    cursor_col: usize,
    /// Command buffer for : commands
    command_buffer: String,
    /// Modified flag
    modified: bool,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VimEditor {
    /// Create a new vim editor
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            filename: None,
            buffer: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            command_buffer: String::new(),
            modified: false,
        }
    }

    /// Open a file for editing
    pub fn open(&mut self, filename: &str, content: &str) {
        let tracer = Tracer::new();
        tracer.info(TraceCategory::Vim, &format!("Opening file: {}", filename));

        self.filename = Some(filename.to_string());
        self.buffer = content.lines().map(String::from).collect();
        if self.buffer.is_empty() {
            self.buffer.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.mode = VimMode::Normal;
        self.modified = false;

        self.render();
    }

    /// Get current mode as string
    #[must_use]
    pub fn mode_str(&self) -> String {
        self.mode.display().to_string()
    }

    /// Get current filename
    #[must_use]
    pub fn filename(&self) -> Option<String> {
        self.filename.clone()
    }

    /// Get buffer contents
    #[must_use]
    pub fn content(&self) -> String {
        self.buffer.join("\n")
    }

    /// Check if modified
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: &str, ctrl: bool, _shift: bool) {
        match self.mode {
            VimMode::Normal => self.handle_normal_key(key, ctrl),
            VimMode::Insert => self.handle_insert_key(key, ctrl),
            VimMode::Visual => self.handle_visual_key(key, ctrl),
            VimMode::Command => self.handle_command_key(key),
        }
        self.render();
    }

    fn handle_normal_key(&mut self, key: &str, _ctrl: bool) {
        match key {
            "i" => self.mode = VimMode::Insert,
            "a" => {
                self.cursor_col = self
                    .cursor_col
                    .saturating_add(1)
                    .min(self.current_line_len());
                self.mode = VimMode::Insert;
            }
            "o" => {
                self.buffer.insert(self.cursor_row + 1, String::new());
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.mode = VimMode::Insert;
                self.modified = true;
            }
            "h" | "ArrowLeft" => self.cursor_col = self.cursor_col.saturating_sub(1),
            "j" | "ArrowDown" => {
                if self.cursor_row < self.buffer.len() - 1 {
                    self.cursor_row += 1;
                    self.clamp_cursor_col();
                }
            }
            "k" | "ArrowUp" => {
                self.cursor_row = self.cursor_row.saturating_sub(1);
                self.clamp_cursor_col();
            }
            "l" | "ArrowRight" => {
                if self.cursor_col < self.current_line_len() {
                    self.cursor_col += 1;
                }
            }
            "0" | "Home" => self.cursor_col = 0,
            "$" | "End" => self.cursor_col = self.current_line_len(),
            "g" => self.cursor_row = 0,
            "G" => self.cursor_row = self.buffer.len().saturating_sub(1),
            ":" => {
                self.mode = VimMode::Command;
                self.command_buffer.clear();
            }
            "x" => {
                if let Some(line) = self.buffer.get_mut(self.cursor_row) {
                    if self.cursor_col < line.len() {
                        line.remove(self.cursor_col);
                        self.modified = true;
                    }
                }
            }
            "d" => {
                // dd deletes line (simplified)
                if self.buffer.len() > 1 {
                    self.buffer.remove(self.cursor_row);
                    if self.cursor_row >= self.buffer.len() {
                        self.cursor_row = self.buffer.len() - 1;
                    }
                    self.modified = true;
                }
            }
            "v" => self.mode = VimMode::Visual,
            _ => {}
        }
    }

    fn handle_insert_key(&mut self, key: &str, _ctrl: bool) {
        match key {
            "Escape" => self.mode = VimMode::Normal,
            "Enter" => {
                if let Some(line) = self.buffer.get_mut(self.cursor_row) {
                    let rest = line.split_off(self.cursor_col);
                    self.buffer.insert(self.cursor_row + 1, rest);
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                    self.modified = true;
                }
            }
            "Backspace" => {
                if self.cursor_col > 0 {
                    if let Some(line) = self.buffer.get_mut(self.cursor_row) {
                        line.remove(self.cursor_col - 1);
                        self.cursor_col -= 1;
                        self.modified = true;
                    }
                } else if self.cursor_row > 0 {
                    let current_line = self.buffer.remove(self.cursor_row);
                    self.cursor_row -= 1;
                    self.cursor_col = self.buffer[self.cursor_row].len();
                    self.buffer[self.cursor_row].push_str(&current_line);
                    self.modified = true;
                }
            }
            _ if key.len() == 1 => {
                if let Some(line) = self.buffer.get_mut(self.cursor_row) {
                    line.insert_str(self.cursor_col, key);
                    self.cursor_col += 1;
                    self.modified = true;
                }
            }
            _ => {}
        }
    }

    fn handle_visual_key(&mut self, key: &str, _ctrl: bool) {
        if key == "Escape" {
            self.mode = VimMode::Normal;
        }
    }

    fn handle_command_key(&mut self, key: &str) {
        match key {
            "Escape" => {
                self.mode = VimMode::Normal;
                self.command_buffer.clear();
            }
            "Enter" => {
                self.execute_command();
                self.mode = VimMode::Normal;
            }
            "Backspace" => {
                self.command_buffer.pop();
            }
            _ if key.len() == 1 => {
                self.command_buffer.push_str(key);
            }
            _ => {}
        }
    }

    fn execute_command(&mut self) {
        let tracer = Tracer::new();
        tracer.debug(
            TraceCategory::Vim,
            &format!("Executing: :{}", self.command_buffer),
        );

        match self.command_buffer.as_str() {
            "q" => {
                if !self.modified {
                    self.close();
                }
            }
            "q!" => self.close(),
            "w" => self.save(),
            "wq" | "x" => {
                self.save();
                self.close();
            }
            _ => {}
        }
        self.command_buffer.clear();
    }

    fn save(&mut self) {
        let tracer = Tracer::new();
        if let Some(filename) = &self.filename {
            tracer.info(TraceCategory::Vim, &format!("Saving: {}", filename));
            self.modified = false;
            // Actual save would be done via WOS kernel
        }
    }

    fn close(&self) {
        let tracer = Tracer::new();
        tracer.info(TraceCategory::Vim, "Closing editor");
        // Hide editor panel
        if let Some(editor) = Dom::get_html_element_by_id("vim-editor") {
            Dom::set_style(&editor, "display", "none");
        }
    }

    fn render(&self) {
        // Render editor state to DOM
        if let Some(content_elem) = Dom::get_element_by_id("vim-content") {
            Dom::set_text_content(&content_elem, &self.content());
        }
        if let Some(mode_elem) = Dom::get_element_by_id("vim-mode") {
            Dom::set_text_content(&mode_elem, self.mode.display());
        }
        if let Some(pos_elem) = Dom::get_element_by_id("vim-position") {
            let pos = format!("{}:{}", self.cursor_row + 1, self.cursor_col + 1);
            Dom::set_text_content(&pos_elem, &pos);
        }
    }

    fn current_line_len(&self) -> usize {
        self.buffer.get(self.cursor_row).map_or(0, String::len)
    }

    fn clamp_cursor_col(&mut self) {
        let max_col = self.current_line_len();
        if self.cursor_col > max_col {
            self.cursor_col = max_col;
        }
    }
}

#[cfg(not(feature = "wasm"))]
impl VimEditor {
    /// Create new editor (non-WASM)
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            filename: None,
            buffer: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            command_buffer: String::new(),
            modified: false,
        }
    }

    /// Open file
    pub fn open(&mut self, filename: &str, content: &str) {
        self.filename = Some(filename.to_string());
        self.buffer = content.lines().map(String::from).collect();
        if self.buffer.is_empty() {
            self.buffer.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.mode = VimMode::Normal;
        self.modified = false;
    }

    /// Mode string
    #[must_use]
    pub fn mode_str(&self) -> String {
        self.mode.display().to_string()
    }

    /// Filename
    #[must_use]
    pub fn filename(&self) -> Option<String> {
        self.filename.clone()
    }

    /// Content
    #[must_use]
    pub fn content(&self) -> String {
        self.buffer.join("\n")
    }

    /// Is modified
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Handle key (simplified for testing)
    pub fn handle_key(&mut self, key: &str, _ctrl: bool, _shift: bool) {
        match key {
            "i" if self.mode == VimMode::Normal => self.mode = VimMode::Insert,
            "Escape" => self.mode = VimMode::Normal,
            _ => {}
        }
    }
}

impl Default for VimEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_mode_display() {
        assert_eq!(VimMode::Normal.display(), "NORMAL");
        assert_eq!(VimMode::Insert.display(), "INSERT");
    }

    #[test]
    fn test_vim_editor_open() {
        let mut editor = VimEditor::new();
        editor.open("test.txt", "Hello\nWorld");
        assert_eq!(editor.filename(), Some("test.txt".to_string()));
        assert_eq!(editor.content(), "Hello\nWorld");
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_vim_mode_switch() {
        let mut editor = VimEditor::new();
        assert_eq!(editor.mode, VimMode::Normal);

        editor.handle_key("i", false, false);
        assert_eq!(editor.mode, VimMode::Insert);

        editor.handle_key("Escape", false, false);
        assert_eq!(editor.mode, VimMode::Normal);
    }
}
