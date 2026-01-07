//! Pure WASM Initialization Module
//!
//! This module provides the zero-JavaScript entry point for WOS.
//! When the WASM module loads, the `init` function is called automatically
//! via `#[wasm_bindgen(start)]`, setting up all DOM event handlers and
//! initializing the operating system.
//!
//! # Architecture
//!
//! The initialization follows this sequence:
//! 1. Set panic hook for browser console error reporting
//! 2. Initialize WOS kernel state
//! 3. Set up terminal input handler
//! 4. Set up panel toolbar buttons
//! 5. Update UI with initial state
//!
//! # Zero JavaScript Mandate
//!
//! This module MUST NOT rely on any JavaScript code. All DOM operations
//! are performed through direct web-sys API calls via the `dom` module.

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::dom::{
    self, add_class, add_event_listener, append_child, console_error, console_log, create_element,
    focus, get_element_by_id, get_html_element_by_id, on_click, on_keydown, remove_class,
    set_attribute, set_text_content, DomResult,
};
use crate::WosWasm;

/// Application state wrapper for use in closures
struct App {
    wos: WosWasm,
    history: Vec<String>,
    history_pos: usize,
    active_panel: String,
}

impl App {
    fn new() -> Self {
        Self {
            wos: WosWasm::new(),
            history: Vec::new(),
            history_pos: 0,
            active_panel: "system_monitor".to_string(),
        }
    }
}

/// Initialize WOS pure WASM mode
///
/// This is the main entry point - automatically called when the WASM module loads.
/// Zero JavaScript: all DOM manipulation is done through web-sys bindings.
#[wasm_bindgen(start)]
pub fn init_pure_wasm() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    console_log("WOS v2.0 Pure WASM initializing...");

    // Create application state wrapped in Rc<RefCell> for closure sharing
    let app = Rc::new(RefCell::new(App::new()));

    // Setup terminal input handler
    if let Err(e) = setup_terminal_input(app.clone()) {
        console_error(&format!("Failed to setup terminal input: {:?}", e));
    }

    // Setup panel toolbar
    if let Err(e) = setup_panel_toolbar(app.clone()) {
        console_error(&format!("Failed to setup panel toolbar: {:?}", e));
    }

    // Setup terminal controls (clear button)
    if let Err(e) = setup_terminal_controls(app.clone()) {
        console_error(&format!("Failed to setup terminal controls: {:?}", e));
    }

    // Display welcome message
    if let Err(e) = display_welcome() {
        console_error(&format!("Failed to display welcome: {:?}", e));
    }

    // Update initial system state
    if let Err(e) = update_system_info(&app.borrow().wos) {
        console_error(&format!("Failed to update system info: {:?}", e));
    }

    // Enable terminal input
    if let Ok(input) = get_html_element_by_id::<web_sys::HtmlInputElement>("terminal-input") {
        input.set_disabled(false);
        let _ = focus(&input);
    }

    console_log("WOS v2.0 Pure WASM ready!");

    Ok(())
}

/// Setup terminal input event handler
fn setup_terminal_input(app: Rc<RefCell<App>>) -> DomResult<()> {
    let input: web_sys::HtmlInputElement = get_html_element_by_id("terminal-input")?;
    let input_target: web_sys::EventTarget = input.clone().dyn_into().unwrap();

    let app_clone = app.clone();
    let input_clone = input.clone();

    let keydown_handler = on_keydown(move |event: web_sys::KeyboardEvent| {
        let key = event.key();

        match key.as_str() {
            "Enter" => {
                let command = input_clone.value();
                if !command.is_empty() {
                    // Add to history
                    {
                        let mut app = app_clone.borrow_mut();
                        app.history.push(command.clone());
                        app.history_pos = app.history.len();
                    }

                    // Echo command
                    let _ =
                        append_terminal_line(&format!("wos$ {}", command), Some("command-line"));

                    // Execute command
                    let output = {
                        let mut app = app_clone.borrow_mut();
                        app.wos.execute_command(&command)
                    };

                    // Display output
                    if !output.is_empty() {
                        for line in output.lines() {
                            let _ = append_terminal_line(line, None);
                        }
                    }

                    // Update system info
                    {
                        let mut app = app_clone.borrow_mut();
                        let _ = update_system_info(&app.wos);
                        let _ = update_process_list(&mut app.wos);
                    }

                    // Clear input
                    input_clone.set_value("");
                }
            }
            "ArrowUp" => {
                event.prevent_default();
                let mut app = app_clone.borrow_mut();
                if app.history_pos > 0 {
                    app.history_pos -= 1;
                    if let Some(cmd) = app.history.get(app.history_pos) {
                        input_clone.set_value(cmd);
                    }
                }
            }
            "ArrowDown" => {
                event.prevent_default();
                let mut app = app_clone.borrow_mut();
                if app.history_pos < app.history.len() {
                    app.history_pos += 1;
                    if app.history_pos < app.history.len() {
                        if let Some(cmd) = app.history.get(app.history_pos) {
                            input_clone.set_value(cmd);
                        }
                    } else {
                        input_clone.set_value("");
                    }
                }
            }
            "l" if event.ctrl_key() => {
                // Ctrl+L clears terminal
                event.prevent_default();
                let _ = clear_terminal();
            }
            _ => {}
        }
    });

    add_event_listener(&input_target, "keydown", &keydown_handler)?;

    // Leak the closure to keep it alive (it's needed for the lifetime of the app)
    keydown_handler.forget();

    Ok(())
}

/// Setup panel toolbar buttons
fn setup_panel_toolbar(app: Rc<RefCell<App>>) -> DomResult<()> {
    let toolbar = get_element_by_id("panel-toolbar")?;

    // Define panels
    let panels = vec![
        ("system_monitor", "System"),
        ("process_list", "Processes"),
        ("memory_map", "Memory"),
        ("apr", "APR"),
        ("vm", "VM"),
    ];

    for (panel_id, label) in panels {
        let button = create_element("button")?;
        set_text_content(&button, label);
        set_attribute(&button, "data-panel", panel_id)?;
        set_attribute(&button, "type", "button")?;
        set_attribute(&button, "aria-pressed", "false")?;
        add_class(&button, "panel-btn")?;

        // Set initial active state
        if panel_id == "system_monitor" {
            add_class(&button, "active")?;
            set_attribute(&button, "aria-pressed", "true")?;
        }

        // Add click handler
        let button_target: web_sys::EventTarget = button.clone().dyn_into().unwrap();
        let app_clone = app.clone();
        let panel_id_owned = panel_id.to_string();

        let click_handler = on_click(move |_event: web_sys::MouseEvent| {
            let mut app = app_clone.borrow_mut();
            app.active_panel = panel_id_owned.clone();
            let _ = switch_panel(&panel_id_owned);
        });

        add_event_listener(&button_target, "click", &click_handler)?;
        click_handler.forget();

        append_child(&toolbar, &button)?;
    }

    Ok(())
}

/// Setup terminal control buttons
fn setup_terminal_controls(_app: Rc<RefCell<App>>) -> DomResult<()> {
    let controls = get_element_by_id("terminal-controls")?;

    // Clear button
    let clear_btn = create_element("button")?;
    set_text_content(&clear_btn, "Clear");
    set_attribute(&clear_btn, "type", "button")?;
    set_attribute(&clear_btn, "aria-label", "Clear terminal")?;
    add_class(&clear_btn, "terminal-btn")?;

    let clear_target: web_sys::EventTarget = clear_btn.clone().dyn_into().unwrap();
    let clear_handler = on_click(move |_| {
        let _ = clear_terminal();
    });

    add_event_listener(&clear_target, "click", &clear_handler)?;
    clear_handler.forget();

    append_child(&controls, &clear_btn)?;

    Ok(())
}

/// Switch active panel
fn switch_panel(panel_id: &str) -> DomResult<()> {
    // Update button states
    if let Ok(buttons) = dom::query_selector_all(".panel-btn") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.get(i) {
                if let Ok(btn) = node.dyn_into::<web_sys::Element>() {
                    let _ = remove_class(&btn, "active");
                    let _ = set_attribute(&btn, "aria-pressed", "false");

                    if let Some(data_panel) = btn.get_attribute("data-panel") {
                        if data_panel == panel_id {
                            let _ = add_class(&btn, "active");
                            let _ = set_attribute(&btn, "aria-pressed", "true");
                        }
                    }
                }
            }
        }
    }

    // Update panel visibility
    if let Ok(panels) = dom::query_selector_all(".file-panel") {
        for i in 0..panels.length() {
            if let Some(node) = panels.get(i) {
                if let Ok(panel) = node.dyn_into::<web_sys::HtmlElement>() {
                    let _ = panel.style().set_property("display", "none");

                    if let Some(data_panel) = panel.get_attribute("data-panel") {
                        if data_panel == panel_id {
                            let _ = panel.style().set_property("display", "block");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Display welcome message
fn display_welcome() -> DomResult<()> {
    let output = get_element_by_id("terminal-output")?;

    // Clear default loading messages
    output.set_inner_html("");

    // Welcome banner
    let lines = vec![
        "WOS v2.0 - Pure WASM Operating System",
        "Architecture: Zero JavaScript, 100% Rust + WebAssembly",
        "",
        "Type 'help' for available commands.",
        "",
    ];

    for line in lines {
        append_terminal_line(line, None)?;
    }

    Ok(())
}

/// Append a line to terminal output
fn append_terminal_line(text: &str, extra_class: Option<&str>) -> DomResult<()> {
    let output = get_element_by_id("terminal-output")?;
    let line = create_element("div")?;
    set_text_content(&line, text);
    add_class(&line, "terminal-line")?;

    if let Some(class) = extra_class {
        add_class(&line, class)?;
    }

    append_child(&output, &line)?;

    // Scroll to bottom
    if let Ok(output_el) = output.dyn_into::<web_sys::HtmlElement>() {
        output_el.set_scroll_top(output_el.scroll_height());
    }

    Ok(())
}

/// Clear terminal output
fn clear_terminal() -> DomResult<()> {
    let output = get_element_by_id("terminal-output")?;
    output.set_inner_html("");
    Ok(())
}

/// Update system info panel
fn update_system_info(wos: &WosWasm) -> DomResult<()> {
    // Update status
    if let Ok(status) = get_element_by_id("status") {
        set_text_content(&status, "Running (Pure WASM)");
    }

    // Update process count
    if let Ok(count) = get_element_by_id("process-count") {
        let num_processes = wos.process_count();
        set_text_content(&count, &num_processes.to_string());
    }

    Ok(())
}

/// Update process list panel
fn update_process_list(wos: &mut WosWasm) -> DomResult<()> {
    let tbody = get_element_by_id("process-table-body")?;
    tbody.set_inner_html("");

    // Execute ps command to get process list
    let output = wos.execute_command("ps");
    let lines: Vec<&str> = output.lines().collect();

    // Skip header line
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let row = create_element("tr")?;

            // PID
            let td_pid = create_element("td")?;
            set_text_content(&td_pid, parts[0]);
            append_child(&row, &td_pid)?;

            // State
            let td_state = create_element("td")?;
            set_text_content(&td_state, parts[1]);
            append_child(&row, &td_state)?;

            // Parent
            let td_parent = create_element("td")?;
            set_text_content(&td_parent, parts[2]);
            append_child(&row, &td_parent)?;

            // Command (rest of the line)
            let td_cmd = create_element("td")?;
            let cmd = parts[3..].join(" ");
            set_text_content(&td_cmd, &cmd);
            append_child(&row, &td_cmd)?;

            append_child(&tbody, &row)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: These tests require a browser environment and cannot run in cargo test
    // Use wasm-bindgen-test for browser-based testing
}
