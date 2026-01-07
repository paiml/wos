//! WOS UI - Pure Rust/WASM User Interface
//!
//! Zero JavaScript implementation of the WOS terminal interface.
//! All DOM manipulation done via web-sys bindings.
//!
//! # Probar Compliance
//!
//! This crate is part of WOS-PROBAR-001 to achieve zero JavaScript compliance.
//! See `docs/tickets/WOS-PROBAR-001-zero-javascript-compliance.yaml`

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod config;
mod dom;
mod editor;
mod help;
mod panel;
mod terminal;
mod tracer;

pub use config::ConfigManager;
pub use dom::Dom;
pub use editor::VimEditor;
pub use help::HelpPanel;
pub use panel::PanelManager;
pub use terminal::Terminal;
pub use tracer::{TraceCategory, TraceLevel, Tracer};

/// Initialize the WOS UI
///
/// This is the main entry point called from the HTML page.
/// It sets up panic hooks, initializes the tracer, and starts the terminal.
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init_ui() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize tracer
    let tracer = Tracer::new();
    tracer.info(TraceCategory::Init, "WOS UI initializing (pure Rust/WASM)");

    // Log successful initialization
    tracer.info(TraceCategory::Init, "WOS UI initialized successfully");
}

/// Get the current WOS UI version
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn ui_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
