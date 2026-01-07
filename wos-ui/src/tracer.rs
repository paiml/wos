//! Tracing system for WOS UI
//!
//! Provides structured logging with levels and categories.
//! Configuration loaded from URL parameters or localStorage.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Trace levels for filtering output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum TraceLevel {
    /// No tracing
    None = 0,
    /// Error messages only
    Error = 1,
    /// Warnings and errors
    Warn = 2,
    /// Info, warnings, and errors
    Info = 3,
    /// Debug output
    Debug = 4,
    /// Verbose trace output
    Trace = 5,
}

impl TraceLevel {
    /// Parse trace level from string
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ERROR" => Self::Error,
            "WARN" => Self::Warn,
            "INFO" => Self::Info,
            "DEBUG" => Self::Debug,
            "TRACE" => Self::Trace,
            _ => Self::None,
        }
    }

    /// Get the name of this level
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

/// Trace categories for filtering by component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum TraceCategory {
    /// Application initialization
    Init,
    /// WASM operations
    Wasm,
    /// Configuration
    Config,
    /// Panel management
    Panel,
    /// Terminal operations
    Terminal,
    /// Vim editor
    Vim,
    /// Process management
    Process,
    /// Memory operations
    Memory,
    /// System calls
    Syscall,
    /// File operations
    File,
    /// Event handling
    Event,
    /// UI rendering
    Render,
}

impl TraceCategory {
    /// Get the name of this category
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Init => "INIT",
            Self::Wasm => "WASM",
            Self::Config => "CONFIG",
            Self::Panel => "PANEL",
            Self::Terminal => "TERMINAL",
            Self::Vim => "VIM",
            Self::Process => "PROCESS",
            Self::Memory => "MEMORY",
            Self::Syscall => "SYSCALL",
            Self::File => "FILE",
            Self::Event => "EVENT",
            Self::Render => "RENDER",
        }
    }

    /// Parse category from string
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INIT" => Some(Self::Init),
            "WASM" => Some(Self::Wasm),
            "CONFIG" => Some(Self::Config),
            "PANEL" => Some(Self::Panel),
            "TERMINAL" => Some(Self::Terminal),
            "VIM" => Some(Self::Vim),
            "PROCESS" => Some(Self::Process),
            "MEMORY" => Some(Self::Memory),
            "SYSCALL" => Some(Self::Syscall),
            "FILE" => Some(Self::File),
            "EVENT" => Some(Self::Event),
            "RENDER" => Some(Self::Render),
            _ => None,
        }
    }
}

/// Tracer for structured logging
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Tracer {
    level: TraceLevel,
    enabled_categories: HashSet<String>,
    #[cfg(feature = "wasm")]
    start_time: f64,
    #[cfg(not(feature = "wasm"))]
    start_time: std::time::Instant,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Tracer {
    /// Create a new tracer, loading config from URL params or localStorage
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let performance = window.performance().expect("no performance");
        let start_time = performance.now();

        let mut tracer = Self {
            level: TraceLevel::None,
            enabled_categories: HashSet::new(),
            start_time,
        };

        tracer.load_config();
        tracer
    }

    /// Load configuration from URL parameters or localStorage
    pub fn load_config(&mut self) {
        let window = web_sys::window().expect("no window");
        let location = window.location();

        // Try URL parameters first
        if let Ok(search) = location.search() {
            let params = web_sys::UrlSearchParams::new_with_str(&search).ok();
            if let Some(params) = params {
                // Load level from URL
                if let Some(level_str) = params.get("trace") {
                    self.level = TraceLevel::parse(&level_str);
                }

                // Load categories from URL
                if let Some(cats_str) = params.get("categories") {
                    for cat in cats_str.split(',') {
                        self.enabled_categories.insert(cat.to_uppercase());
                    }
                }
            }
        }

        // Fall back to localStorage if not set from URL
        if self.level == TraceLevel::None {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(level_str)) = storage.get_item("wos-trace-level") {
                    self.level = TraceLevel::parse(&level_str);
                }

                if self.enabled_categories.is_empty() {
                    if let Ok(Some(cats_str)) = storage.get_item("wos-trace-categories") {
                        for cat in cats_str.split(',') {
                            self.enabled_categories.insert(cat.to_uppercase());
                        }
                    }
                }
            }
        }
    }

    /// Check if a message should be traced
    #[must_use]
    pub fn should_trace(&self, level: TraceLevel, category: TraceCategory) -> bool {
        if self.level < level {
            return false;
        }
        if !self.enabled_categories.is_empty() && !self.enabled_categories.contains(category.name())
        {
            return false;
        }
        true
    }

    /// Log a message
    pub fn log(&self, level: TraceLevel, category: TraceCategory, message: &str) {
        if !self.should_trace(level, category) {
            return;
        }

        let window = web_sys::window().expect("no window");
        let performance = window.performance().expect("no performance");
        let elapsed = performance.now() - self.start_time;

        let formatted = format!(
            "[{:.2}ms] [{}] [{}] {}",
            elapsed,
            category.name(),
            level.name(),
            message
        );

        web_sys::console::log_1(&formatted.into());
    }

    /// Log an error
    pub fn error(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Error, category, message);
    }

    /// Log a warning
    pub fn warn(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Warn, category, message);
    }

    /// Log info
    pub fn info(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Info, category, message);
    }

    /// Log debug
    pub fn debug(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Debug, category, message);
    }

    /// Log trace
    pub fn trace_msg(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Trace, category, message);
    }

    /// Set the trace level
    pub fn set_level(&mut self, level: TraceLevel) {
        self.level = level;

        // Save to localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("wos-trace-level", level.name());
            }
        }
    }

    /// Clear all trace configuration
    pub fn clear(&mut self) {
        self.level = TraceLevel::None;
        self.enabled_categories.clear();

        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("wos-trace-level");
                let _ = storage.remove_item("wos-trace-categories");
            }
        }
    }
}

#[cfg(not(feature = "wasm"))]
impl Tracer {
    /// Create a new tracer (non-WASM version for testing)
    #[must_use]
    pub fn new() -> Self {
        Self {
            level: TraceLevel::None,
            enabled_categories: HashSet::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Load configuration (no-op for non-WASM)
    pub fn load_config(&mut self) {}

    /// Check if should trace
    #[must_use]
    pub fn should_trace(&self, level: TraceLevel, category: TraceCategory) -> bool {
        if self.level < level {
            return false;
        }
        if !self.enabled_categories.is_empty() && !self.enabled_categories.contains(category.name())
        {
            return false;
        }
        true
    }

    /// Log a message (prints to stdout for non-WASM)
    pub fn log(&self, level: TraceLevel, category: TraceCategory, message: &str) {
        if !self.should_trace(level, category) {
            return;
        }
        let elapsed = self.start_time.elapsed().as_secs_f64() * 1000.0;
        println!(
            "[{:.2}ms] [{}] [{}] {}",
            elapsed,
            category.name(),
            level.name(),
            message
        );
    }

    /// Log error
    pub fn error(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Error, category, message);
    }

    /// Log warning
    pub fn warn(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Warn, category, message);
    }

    /// Log info
    pub fn info(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Info, category, message);
    }

    /// Log debug
    pub fn debug(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Debug, category, message);
    }

    /// Log trace
    pub fn trace_msg(&self, category: TraceCategory, message: &str) {
        self.log(TraceLevel::Trace, category, message);
    }

    /// Set level
    pub fn set_level(&mut self, level: TraceLevel) {
        self.level = level;
    }

    /// Clear configuration
    pub fn clear(&mut self) {
        self.level = TraceLevel::None;
        self.enabled_categories.clear();
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_level_from_str() {
        assert_eq!(TraceLevel::parse("ERROR"), TraceLevel::Error);
        assert_eq!(TraceLevel::parse("error"), TraceLevel::Error);
        assert_eq!(TraceLevel::parse("INFO"), TraceLevel::Info);
        assert_eq!(TraceLevel::parse("unknown"), TraceLevel::None);
    }

    #[test]
    fn test_trace_category_from_str() {
        assert_eq!(TraceCategory::parse("INIT"), Some(TraceCategory::Init));
        assert_eq!(
            TraceCategory::parse("terminal"),
            Some(TraceCategory::Terminal)
        );
        assert_eq!(TraceCategory::parse("unknown"), None);
    }

    #[test]
    fn test_tracer_default_level() {
        let tracer = Tracer::new();
        assert_eq!(tracer.level, TraceLevel::None);
    }

    #[test]
    fn test_should_trace() {
        let mut tracer = Tracer::new();
        tracer.level = TraceLevel::Info;

        assert!(tracer.should_trace(TraceLevel::Error, TraceCategory::Init));
        assert!(tracer.should_trace(TraceLevel::Info, TraceCategory::Init));
        assert!(!tracer.should_trace(TraceLevel::Debug, TraceCategory::Init));
    }
}
