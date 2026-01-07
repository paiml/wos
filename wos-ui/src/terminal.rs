//! Terminal implementation for WOS
//!
//! Pure Rust terminal with command history, input handling,
//! and output rendering via web-sys.

use crate::config::ConfigManager;
use crate::dom::Dom;
use crate::tracer::{TraceCategory, Tracer};
use std::collections::VecDeque;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsCast;
#[cfg(feature = "wasm")]
use web_sys::KeyboardEvent;

/// Command history entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The command that was entered
    pub command: String,
    /// Output from the command (used for history replay)
    #[allow(dead_code)]
    pub output: String,
    /// Exit code (0 = success, used for history display)
    #[allow(dead_code)]
    pub exit_code: i32,
}

/// Terminal state
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Terminal {
    /// Command history
    history: VecDeque<HistoryEntry>,
    /// Current history index for navigation
    history_index: Option<usize>,
    /// Maximum history size
    max_history: usize,
    /// Current input buffer
    input_buffer: String,
    /// Saved input when navigating history
    saved_input: String,
    /// Current working directory
    cwd: String,
    /// Terminal prompt
    prompt: String,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Terminal {
    /// Create a new terminal
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        let config = ConfigManager::new();
        Self {
            history: VecDeque::new(),
            history_index: None,
            max_history: config.history_size(),
            input_buffer: String::new(),
            saved_input: String::new(),
            cwd: "/".to_string(),
            prompt: config.prompt(),
        }
    }

    /// Initialize the terminal UI
    pub fn init(&self) {
        let tracer = Tracer::new();
        tracer.info(TraceCategory::Terminal, "Initializing terminal UI");

        // Set up event listeners
        self.setup_input_handler();
        self.render_prompt();

        tracer.info(TraceCategory::Terminal, "Terminal initialized");
    }

    /// Set up keyboard input handler
    fn setup_input_handler(&self) {
        if let Some(input) = Dom::get_input_by_id("terminal-input") {
            let closure = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
                let key = event.key();
                match key.as_str() {
                    "Enter" => {
                        // Handle command execution
                        event.prevent_default();
                    }
                    "ArrowUp" => {
                        // Navigate history up
                        event.prevent_default();
                    }
                    "ArrowDown" => {
                        // Navigate history down
                        event.prevent_default();
                    }
                    "Tab" => {
                        // Tab completion
                        event.prevent_default();
                    }
                    _ => {}
                }
            });

            let _ =
                input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget(); // Keep closure alive
        }
    }

    /// Render the terminal prompt
    fn render_prompt(&self) {
        if let Some(output) = Dom::get_html_element_by_id("terminal-output") {
            let prompt_line = format!("{}{}", self.prompt, self.input_buffer);
            // Add prompt to output
            if let Some(span) = Dom::create_span() {
                span.set_inner_html(&prompt_line);
                Dom::add_class(&span, "prompt");
                let _ = output.append_child(&span);
            }
        }
    }

    /// Write output to the terminal
    pub fn write(&self, text: &str) {
        if let Some(output) = Dom::get_html_element_by_id("terminal-output") {
            if let Some(div) = Dom::create_div() {
                Dom::set_text_content(&div, text);
                Dom::add_class(&div, "output-line");
                let _ = output.append_child(&div);
                Dom::scroll_into_view(&div);
            }
        }
    }

    /// Write error output
    pub fn write_error(&self, text: &str) {
        if let Some(output) = Dom::get_html_element_by_id("terminal-output") {
            if let Some(div) = Dom::create_div() {
                Dom::set_text_content(&div, text);
                Dom::add_class(&div, "error-line");
                let _ = output.append_child(&div);
                Dom::scroll_into_view(&div);
            }
        }
    }

    /// Clear the terminal
    pub fn clear(&self) {
        if let Some(output) = Dom::get_html_element_by_id("terminal-output") {
            Dom::set_inner_html(&output, "");
        }
    }

    /// Add command to history
    pub fn add_to_history(&mut self, command: &str, output: &str, exit_code: i32) {
        if command.is_empty() {
            return;
        }

        let entry = HistoryEntry {
            command: command.to_string(),
            output: output.to_string(),
            exit_code,
        };

        self.history.push_back(entry);

        // Trim history if needed
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        self.history_index = None;
    }

    /// Navigate history up
    pub fn history_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }

        match self.history_index {
            None => {
                // Save current input and go to most recent
                self.saved_input = self.input_buffer.clone();
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => {
                // Already at oldest entry
                return self.history.front().map(|e| e.command.clone());
            }
            Some(idx) => {
                self.history_index = Some(idx - 1);
            }
        }

        self.history_index
            .and_then(|idx| self.history.get(idx))
            .map(|e| e.command.clone())
    }

    /// Navigate history down
    pub fn history_down(&mut self) -> Option<String> {
        match self.history_index {
            None => None,
            Some(idx) => {
                if idx >= self.history.len() - 1 {
                    // Return to saved input
                    self.history_index = None;
                    Some(self.saved_input.clone())
                } else {
                    self.history_index = Some(idx + 1);
                    self.history.get(idx + 1).map(|e| e.command.clone())
                }
            }
        }
    }

    /// Get the current input
    #[must_use]
    pub fn get_input(&self) -> String {
        if let Some(input) = Dom::get_input_by_id("terminal-input") {
            input.value()
        } else {
            String::new()
        }
    }

    /// Set the input value
    pub fn set_input(&self, value: &str) {
        if let Some(input) = Dom::get_input_by_id("terminal-input") {
            input.set_value(value);
        }
    }

    /// Focus the terminal input
    pub fn focus(&self) {
        if let Some(input) = Dom::get_html_element_by_id("terminal-input") {
            Dom::focus(&input);
        }
    }

    /// Get history count
    #[must_use]
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Get current working directory
    #[must_use]
    pub fn cwd(&self) -> String {
        self.cwd.clone()
    }

    /// Set current working directory
    pub fn set_cwd(&mut self, cwd: &str) {
        self.cwd = cwd.to_string();
    }
}

#[cfg(not(feature = "wasm"))]
impl Terminal {
    /// Create new terminal (non-WASM)
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            history_index: None,
            max_history: 100,
            input_buffer: String::new(),
            saved_input: String::new(),
            cwd: "/".to_string(),
            prompt: "wos$ ".to_string(),
        }
    }

    /// Init (no-op for non-WASM)
    pub fn init(&self) {}

    /// Write output (print to stdout)
    pub fn write(&self, text: &str) {
        println!("{text}");
    }

    /// Write error (print to stderr)
    pub fn write_error(&self, text: &str) {
        eprintln!("{text}");
    }

    /// Clear (no-op for non-WASM)
    pub fn clear(&self) {}

    /// Add to history
    pub fn add_to_history(&mut self, command: &str, output: &str, exit_code: i32) {
        if command.is_empty() {
            return;
        }
        self.history.push_back(HistoryEntry {
            command: command.to_string(),
            output: output.to_string(),
            exit_code,
        });
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
        self.history_index = None;
    }

    /// History up
    pub fn history_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        match self.history_index {
            None => {
                self.saved_input = self.input_buffer.clone();
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => {
                return self.history.front().map(|e| e.command.clone());
            }
            Some(idx) => {
                self.history_index = Some(idx - 1);
            }
        }
        self.history_index
            .and_then(|idx| self.history.get(idx))
            .map(|e| e.command.clone())
    }

    /// History down
    pub fn history_down(&mut self) -> Option<String> {
        match self.history_index {
            None => None,
            Some(idx) => {
                if idx >= self.history.len() - 1 {
                    self.history_index = None;
                    Some(self.saved_input.clone())
                } else {
                    self.history_index = Some(idx + 1);
                    self.history.get(idx + 1).map(|e| e.command.clone())
                }
            }
        }
    }

    /// Get input
    #[must_use]
    pub fn get_input(&self) -> String {
        self.input_buffer.clone()
    }

    /// Set input
    pub fn set_input(&mut self, value: &str) {
        self.input_buffer = value.to_string();
    }

    /// Focus (no-op)
    pub fn focus(&self) {}

    /// History count
    #[must_use]
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// CWD
    #[must_use]
    pub fn cwd(&self) -> String {
        self.cwd.clone()
    }

    /// Set CWD
    pub fn set_cwd(&mut self, cwd: &str) {
        self.cwd = cwd.to_string();
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_history() {
        let mut term = Terminal::new();
        assert_eq!(term.history_count(), 0);

        term.add_to_history("ls", "file1\nfile2", 0);
        assert_eq!(term.history_count(), 1);

        term.add_to_history("pwd", "/home", 0);
        assert_eq!(term.history_count(), 2);
    }

    #[test]
    fn test_history_navigation() {
        let mut term = Terminal::new();
        term.add_to_history("cmd1", "", 0);
        term.add_to_history("cmd2", "", 0);
        term.add_to_history("cmd3", "", 0);

        // Navigate up
        assert_eq!(term.history_up(), Some("cmd3".to_string()));
        assert_eq!(term.history_up(), Some("cmd2".to_string()));
        assert_eq!(term.history_up(), Some("cmd1".to_string()));
        assert_eq!(term.history_up(), Some("cmd1".to_string())); // Stay at oldest

        // Navigate down
        assert_eq!(term.history_down(), Some("cmd2".to_string()));
        assert_eq!(term.history_down(), Some("cmd3".to_string()));
    }

    #[test]
    fn test_cwd() {
        let mut term = Terminal::new();
        assert_eq!(term.cwd(), "/");

        term.set_cwd("/home/user");
        assert_eq!(term.cwd(), "/home/user");
    }
}
