//! Panel management for WOS UI
//!
//! Handles collapsible panels for process list, memory map, etc.

use crate::config::ConfigManager;
use crate::dom::Dom;
use crate::tracer::{TraceCategory, Tracer};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Panel identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    /// Terminal panel
    Terminal,
    /// Process list panel
    ProcessList,
    /// Memory map panel
    MemoryMap,
    /// System call trace panel
    SyscallTrace,
    /// Filesystem panel
    Filesystem,
    /// Help panel
    Help,
}

impl PanelId {
    /// Get the DOM element ID for this panel
    #[must_use]
    pub const fn element_id(&self) -> &'static str {
        match self {
            Self::Terminal => "terminal-panel",
            Self::ProcessList => "process-list-panel",
            Self::MemoryMap => "memory-map-panel",
            Self::SyscallTrace => "syscall-trace-panel",
            Self::Filesystem => "filesystem-panel",
            Self::Help => "help-panel",
        }
    }

    /// Get the config key for this panel
    #[must_use]
    pub const fn config_key(&self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::ProcessList => "process_list",
            Self::MemoryMap => "memory_map",
            Self::SyscallTrace => "syscall_trace",
            Self::Filesystem => "filesystem",
            Self::Help => "help",
        }
    }
}

/// Panel manager
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct PanelManager {
    config: ConfigManager,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl PanelManager {
    /// Create a new panel manager
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ConfigManager::new(),
        }
    }

    /// Initialize all panels
    pub fn init(&self) {
        let tracer = Tracer::new();
        tracer.info(TraceCategory::Panel, "Initializing panels");

        // Apply initial panel states from config
        self.apply_panel_states();

        tracer.info(TraceCategory::Panel, "Panels initialized");
    }

    /// Apply panel states from configuration
    fn apply_panel_states(&self) {
        let panels = [
            PanelId::Terminal,
            PanelId::ProcessList,
            PanelId::MemoryMap,
            PanelId::SyscallTrace,
            PanelId::Filesystem,
            PanelId::Help,
        ];

        for panel in panels {
            let collapsed = self.config.is_panel_collapsed(panel.config_key());
            self.set_collapsed_internal(panel, collapsed);
        }
    }

    /// Toggle a panel's collapsed state
    pub fn toggle(&mut self, panel_id: &str) {
        let panel = match panel_id {
            "terminal" => PanelId::Terminal,
            "process_list" => PanelId::ProcessList,
            "memory_map" => PanelId::MemoryMap,
            "syscall_trace" => PanelId::SyscallTrace,
            "filesystem" => PanelId::Filesystem,
            "help" => PanelId::Help,
            _ => return,
        };

        let currently_collapsed = self.config.is_panel_collapsed(panel.config_key());
        let new_state = !currently_collapsed;

        self.config
            .set_panel_collapsed(panel.config_key(), new_state);
        self.set_collapsed_internal(panel, new_state);
    }

    /// Set a panel's collapsed state
    fn set_collapsed_internal(&self, panel: PanelId, collapsed: bool) {
        if let Some(element) = Dom::get_element_by_id(panel.element_id()) {
            if collapsed {
                Dom::add_class(&element, "collapsed");
            } else {
                Dom::remove_class(&element, "collapsed");
            }
        }

        // Update toggle button state
        let button_id = format!("{}-toggle", panel.element_id());
        if let Some(button) = Dom::get_element_by_id(&button_id) {
            if collapsed {
                Dom::set_text_content(&button, "▶");
            } else {
                Dom::set_text_content(&button, "▼");
            }
        }
    }

    /// Collapse all panels except terminal
    pub fn collapse_all(&mut self) {
        let panels = [
            PanelId::ProcessList,
            PanelId::MemoryMap,
            PanelId::SyscallTrace,
            PanelId::Filesystem,
            PanelId::Help,
        ];

        for panel in panels {
            self.config.set_panel_collapsed(panel.config_key(), true);
            self.set_collapsed_internal(panel, true);
        }
    }

    /// Expand all panels
    pub fn expand_all(&mut self) {
        let panels = [
            PanelId::Terminal,
            PanelId::ProcessList,
            PanelId::MemoryMap,
            PanelId::SyscallTrace,
            PanelId::Filesystem,
            PanelId::Help,
        ];

        for panel in panels {
            self.config.set_panel_collapsed(panel.config_key(), false);
            self.set_collapsed_internal(panel, false);
        }
    }

    /// Check if a panel is collapsed
    #[must_use]
    pub fn is_collapsed(&self, panel_id: &str) -> bool {
        self.config.is_panel_collapsed(panel_id)
    }
}

#[cfg(not(feature = "wasm"))]
impl PanelManager {
    /// Create new panel manager (non-WASM)
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ConfigManager::new(),
        }
    }

    /// Init (no-op)
    pub fn init(&self) {}

    /// Toggle panel
    pub fn toggle(&mut self, panel_id: &str) {
        let collapsed = self.config.is_panel_collapsed(panel_id);
        self.config.set_panel_collapsed(panel_id, !collapsed);
    }

    /// Collapse all
    pub fn collapse_all(&mut self) {}

    /// Expand all
    pub fn expand_all(&mut self) {}

    /// Check collapsed
    #[must_use]
    pub fn is_collapsed(&self, panel_id: &str) -> bool {
        self.config.is_panel_collapsed(panel_id)
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_id_element_id() {
        assert_eq!(PanelId::Terminal.element_id(), "terminal-panel");
        assert_eq!(PanelId::ProcessList.element_id(), "process-list-panel");
    }

    #[test]
    fn test_panel_manager() {
        let manager = PanelManager::new();
        assert!(manager.is_collapsed("process_list"));
        assert!(!manager.is_collapsed("terminal"));
    }
}
