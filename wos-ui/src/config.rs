//! Configuration management for WOS UI
//!
//! Handles loading, saving, and applying UI configuration.

use crate::dom::Dom;
use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Whether the panel is visible
    pub visible: bool,
    /// Whether the panel is collapsed
    pub collapsed: bool,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            visible: true,
            collapsed: false,
        }
    }
}

/// Panels configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelsConfig {
    /// Terminal panel
    pub terminal: PanelConfig,
    /// Process list panel
    pub process_list: PanelConfig,
    /// Memory map panel
    pub memory_map: PanelConfig,
    /// Syscall trace panel
    pub syscall_trace: PanelConfig,
    /// Filesystem panel
    pub filesystem: PanelConfig,
    /// Help panel
    pub help: PanelConfig,
}

impl Default for PanelsConfig {
    fn default() -> Self {
        Self {
            terminal: PanelConfig {
                visible: true,
                collapsed: false,
            },
            process_list: PanelConfig {
                visible: true,
                collapsed: true,
            },
            memory_map: PanelConfig {
                visible: true,
                collapsed: true,
            },
            syscall_trace: PanelConfig {
                visible: true,
                collapsed: true,
            },
            filesystem: PanelConfig {
                visible: true,
                collapsed: true,
            },
            help: PanelConfig {
                visible: true,
                collapsed: true,
            },
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme
    pub theme: String,
    /// UI mode
    pub mode: String,
    /// Panel configuration
    pub panels: PanelsConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            mode: "interactive".to_string(),
            panels: PanelsConfig::default(),
        }
    }
}

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Number of history entries to keep
    pub history_size: usize,
    /// Default prompt
    pub prompt: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            history_size: 100,
            prompt: "wos$ ".to_string(),
        }
    }
}

/// Full WOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WosConfig {
    /// Configuration version
    pub version: String,
    /// Environment (browser, native)
    pub environment: String,
    /// UI configuration
    pub ui: UiConfig,
    /// Terminal configuration
    pub terminal: TerminalConfig,
}

impl Default for WosConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            environment: "browser".to_string(),
            ui: UiConfig::default(),
            terminal: TerminalConfig::default(),
        }
    }
}

/// Configuration manager
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ConfigManager {
    config: WosConfig,
}

#[cfg(feature = "wasm")]
const CONFIG_STORAGE_KEY: &str = "wos-config";

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ConfigManager {
    /// Create a new config manager
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            config: WosConfig::default(),
        };
        manager.load_config();
        manager
    }

    /// Load configuration from localStorage
    pub fn load_config(&mut self) {
        if let Some(saved) = Dom::get_storage_item(CONFIG_STORAGE_KEY) {
            match serde_json::from_str::<WosConfig>(&saved) {
                Ok(config) => {
                    self.config = config;
                }
                Err(_) => {
                    // Fall back to default on parse error
                    self.config = WosConfig::default();
                }
            }
        }
    }

    /// Save configuration to localStorage
    pub fn save_config(&self) {
        if let Ok(json) = serde_json::to_string(&self.config) {
            Dom::set_storage_item(CONFIG_STORAGE_KEY, &json);
        }
    }

    /// Get the current theme
    #[must_use]
    pub fn theme(&self) -> String {
        self.config.ui.theme.clone()
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: &str) {
        self.config.ui.theme = theme.to_string();
        self.save_config();
    }

    /// Get terminal prompt
    #[must_use]
    pub fn prompt(&self) -> String {
        self.config.terminal.prompt.clone()
    }

    /// Get history size
    #[must_use]
    pub fn history_size(&self) -> usize {
        self.config.terminal.history_size
    }

    /// Check if a panel is visible
    #[must_use]
    pub fn is_panel_visible(&self, panel: &str) -> bool {
        match panel {
            "terminal" => self.config.ui.panels.terminal.visible,
            "process_list" => self.config.ui.panels.process_list.visible,
            "memory_map" => self.config.ui.panels.memory_map.visible,
            "syscall_trace" => self.config.ui.panels.syscall_trace.visible,
            "filesystem" => self.config.ui.panels.filesystem.visible,
            "help" => self.config.ui.panels.help.visible,
            _ => false,
        }
    }

    /// Check if a panel is collapsed
    #[must_use]
    pub fn is_panel_collapsed(&self, panel: &str) -> bool {
        match panel {
            "terminal" => self.config.ui.panels.terminal.collapsed,
            "process_list" => self.config.ui.panels.process_list.collapsed,
            "memory_map" => self.config.ui.panels.memory_map.collapsed,
            "syscall_trace" => self.config.ui.panels.syscall_trace.collapsed,
            "filesystem" => self.config.ui.panels.filesystem.collapsed,
            "help" => self.config.ui.panels.help.collapsed,
            _ => true,
        }
    }

    /// Set panel collapsed state
    pub fn set_panel_collapsed(&mut self, panel: &str, collapsed: bool) {
        match panel {
            "terminal" => self.config.ui.panels.terminal.collapsed = collapsed,
            "process_list" => self.config.ui.panels.process_list.collapsed = collapsed,
            "memory_map" => self.config.ui.panels.memory_map.collapsed = collapsed,
            "syscall_trace" => self.config.ui.panels.syscall_trace.collapsed = collapsed,
            "filesystem" => self.config.ui.panels.filesystem.collapsed = collapsed,
            "help" => self.config.ui.panels.help.collapsed = collapsed,
            _ => {}
        }
        self.save_config();
    }

    /// Reset to default configuration
    pub fn reset(&mut self) {
        self.config = WosConfig::default();
        self.save_config();
    }

    /// Get config as JSON string
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.config).unwrap_or_default()
    }
}

#[cfg(not(feature = "wasm"))]
impl ConfigManager {
    /// Create a new config manager (non-WASM)
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: WosConfig::default(),
        }
    }

    /// Load config (no-op for non-WASM)
    pub fn load_config(&mut self) {}

    /// Save config (no-op for non-WASM)
    pub fn save_config(&self) {}

    /// Get theme
    #[must_use]
    pub fn theme(&self) -> String {
        self.config.ui.theme.clone()
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: &str) {
        self.config.ui.theme = theme.to_string();
    }

    /// Get prompt
    #[must_use]
    pub fn prompt(&self) -> String {
        self.config.terminal.prompt.clone()
    }

    /// Get history size
    #[must_use]
    pub fn history_size(&self) -> usize {
        self.config.terminal.history_size
    }

    /// Check panel visible
    #[must_use]
    pub fn is_panel_visible(&self, panel: &str) -> bool {
        match panel {
            "terminal" => self.config.ui.panels.terminal.visible,
            _ => false,
        }
    }

    /// Check panel collapsed
    #[must_use]
    pub fn is_panel_collapsed(&self, panel: &str) -> bool {
        match panel {
            "terminal" => self.config.ui.panels.terminal.collapsed,
            _ => true,
        }
    }

    /// Set panel collapsed
    pub fn set_panel_collapsed(&mut self, panel: &str, collapsed: bool) {
        if panel == "terminal" {
            self.config.ui.panels.terminal.collapsed = collapsed;
        }
    }

    /// Reset config
    pub fn reset(&mut self) {
        self.config = WosConfig::default();
    }

    /// To JSON
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.config).unwrap_or_default()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WosConfig::default();
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn test_config_manager() {
        let mut manager = ConfigManager::new();
        assert_eq!(manager.theme(), "dark");

        manager.set_theme("light");
        assert_eq!(manager.theme(), "light");

        manager.reset();
        assert_eq!(manager.theme(), "dark");
    }

    #[test]
    fn test_panel_config() {
        let manager = ConfigManager::new();
        assert!(manager.is_panel_visible("terminal"));
        assert!(!manager.is_panel_collapsed("terminal"));
        assert!(manager.is_panel_collapsed("process_list"));
    }
}
