//! Help panel for WOS
//!
//! Displays command help and keyboard shortcuts.

use crate::dom::Dom;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Help panel
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct HelpPanel {
    visible: bool,
}

/// Help entry for a command
pub struct HelpEntry {
    /// Command name
    pub command: &'static str,
    /// Command description
    pub description: &'static str,
    /// Usage example
    pub usage: &'static str,
}

/// Built-in help data
pub const HELP_DATA: &[HelpEntry] = &[
    HelpEntry {
        command: "help",
        description: "Show available commands",
        usage: "help [command]",
    },
    HelpEntry {
        command: "ls",
        description: "List directory contents",
        usage: "ls [path]",
    },
    HelpEntry {
        command: "cd",
        description: "Change directory",
        usage: "cd <path>",
    },
    HelpEntry {
        command: "pwd",
        description: "Print working directory",
        usage: "pwd",
    },
    HelpEntry {
        command: "cat",
        description: "Display file contents",
        usage: "cat <file>",
    },
    HelpEntry {
        command: "echo",
        description: "Display text",
        usage: "echo <text>",
    },
    HelpEntry {
        command: "touch",
        description: "Create empty file",
        usage: "touch <file>",
    },
    HelpEntry {
        command: "mkdir",
        description: "Create directory",
        usage: "mkdir <dir>",
    },
    HelpEntry {
        command: "rm",
        description: "Remove file",
        usage: "rm <file>",
    },
    HelpEntry {
        command: "ps",
        description: "List processes",
        usage: "ps",
    },
    HelpEntry {
        command: "kill",
        description: "Terminate process",
        usage: "kill <pid>",
    },
    HelpEntry {
        command: "vim",
        description: "Open text editor",
        usage: "vim [file]",
    },
    HelpEntry {
        command: "grep",
        description: "Search for pattern",
        usage: "grep <pattern> [file]",
    },
    HelpEntry {
        command: "wc",
        description: "Count lines/words/bytes",
        usage: "wc [file]",
    },
    HelpEntry {
        command: "clear",
        description: "Clear terminal",
        usage: "clear",
    },
    HelpEntry {
        command: "reset",
        description: "Reset system",
        usage: "reset",
    },
    HelpEntry {
        command: "version",
        description: "Show WOS version",
        usage: "version",
    },
    HelpEntry {
        command: "state",
        description: "Show kernel state",
        usage: "state",
    },
];

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl HelpPanel {
    /// Create a new help panel
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self { visible: false }
    }

    /// Show the help panel
    pub fn show(&mut self) {
        self.visible = true;
        if let Some(panel) = Dom::get_html_element_by_id("help-panel") {
            Dom::remove_class(&panel, "collapsed");
        }
        self.render();
    }

    /// Hide the help panel
    pub fn hide(&mut self) {
        self.visible = false;
        if let Some(panel) = Dom::get_html_element_by_id("help-panel") {
            Dom::add_class(&panel, "collapsed");
        }
    }

    /// Toggle the help panel
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Check if visible
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get help text for a specific command
    #[must_use]
    pub fn get_command_help(&self, command: &str) -> String {
        for entry in HELP_DATA {
            if entry.command == command {
                return format!(
                    "{}\n\nDescription: {}\nUsage: {}",
                    entry.command, entry.description, entry.usage
                );
            }
        }
        format!("Unknown command: {}", command)
    }

    /// Get all commands help
    #[must_use]
    pub fn get_all_help(&self) -> String {
        let mut help = String::from("Available commands:\n\n");
        for entry in HELP_DATA {
            help.push_str(&format!("  {:12} - {}\n", entry.command, entry.description));
        }
        help.push_str("\nUse 'help <command>' for detailed help.");
        help
    }

    fn render(&self) {
        if let Some(content) = Dom::get_element_by_id("help-content") {
            Dom::set_text_content(&content, &self.get_all_help());
        }
    }
}

#[cfg(not(feature = "wasm"))]
impl HelpPanel {
    /// Create new help panel (non-WASM)
    #[must_use]
    pub fn new() -> Self {
        Self { visible: false }
    }

    /// Show
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Is visible
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get command help
    #[must_use]
    pub fn get_command_help(&self, command: &str) -> String {
        for entry in HELP_DATA {
            if entry.command == command {
                return format!(
                    "{}\n\nDescription: {}\nUsage: {}",
                    entry.command, entry.description, entry.usage
                );
            }
        }
        format!("Unknown command: {}", command)
    }

    /// Get all help
    #[must_use]
    pub fn get_all_help(&self) -> String {
        let mut help = String::from("Available commands:\n\n");
        for entry in HELP_DATA {
            help.push_str(&format!("  {:12} - {}\n", entry.command, entry.description));
        }
        help.push_str("\nUse 'help <command>' for detailed help.");
        help
    }
}

impl Default for HelpPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_panel() {
        let mut panel = HelpPanel::new();
        assert!(!panel.is_visible());

        panel.show();
        assert!(panel.is_visible());

        panel.hide();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_command_help() {
        let panel = HelpPanel::new();
        let help = panel.get_command_help("ls");
        assert!(help.contains("List directory"));
    }

    #[test]
    fn test_unknown_command() {
        let panel = HelpPanel::new();
        let help = panel.get_command_help("unknown");
        assert!(help.contains("Unknown command"));
    }

    #[test]
    fn test_all_help() {
        let panel = HelpPanel::new();
        let help = panel.get_all_help();
        assert!(help.contains("Available commands"));
        assert!(help.contains("ls"));
        assert!(help.contains("cat"));
    }
}
