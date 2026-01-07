//! Pure WASM DOM Manipulation Module
//!
//! This module provides zero-JavaScript DOM manipulation for WOS.
//! All DOM interactions are performed through web-sys bindings.
//!
//! # Architecture
//!
//! Following the presentar pattern, this module provides:
//! - Element creation and manipulation
//! - Event handling via closures
//! - Animation frame scheduling
//! - Storage access
//!
//! # Zero JavaScript Mandate
//!
//! This module MUST NOT rely on any JavaScript code. All DOM operations
//! are performed through direct web-sys API calls.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Result type for DOM operations
pub type DomResult<T> = Result<T, DomError>;

/// DOM operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum DomError {
    /// Window object not available
    NoWindow,
    /// Document object not available
    NoDocument,
    /// Element not found
    ElementNotFound(String),
    /// Invalid element type
    InvalidElementType(String),
    /// Event handler registration failed
    EventHandlerFailed(String),
    /// Storage not available
    StorageNotAvailable,
    /// Storage operation failed
    StorageError(String),
    /// Animation frame request failed
    AnimationFrameFailed,
    /// Generic DOM operation failed
    OperationFailed(String),
}

impl std::fmt::Display for DomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoWindow => write!(f, "Window object not available"),
            Self::NoDocument => write!(f, "Document object not available"),
            Self::ElementNotFound(id) => write!(f, "Element not found: {}", id),
            Self::InvalidElementType(t) => write!(f, "Invalid element type: {}", t),
            Self::EventHandlerFailed(e) => write!(f, "Event handler failed: {}", e),
            Self::StorageNotAvailable => write!(f, "Storage not available"),
            Self::StorageError(e) => write!(f, "Storage error: {}", e),
            Self::AnimationFrameFailed => write!(f, "Animation frame request failed"),
            Self::OperationFailed(e) => write!(f, "DOM operation failed: {}", e),
        }
    }
}

impl std::error::Error for DomError {}

/// Get the global window object
pub fn window() -> DomResult<web_sys::Window> {
    web_sys::window().ok_or(DomError::NoWindow)
}

/// Get the document object
pub fn document() -> DomResult<web_sys::Document> {
    window()?.document().ok_or(DomError::NoDocument)
}

/// Get an element by ID
pub fn get_element_by_id(id: &str) -> DomResult<web_sys::Element> {
    document()?
        .get_element_by_id(id)
        .ok_or_else(|| DomError::ElementNotFound(id.to_string()))
}

/// Get an HTML element by ID with type casting
pub fn get_html_element_by_id<T: JsCast>(id: &str) -> DomResult<T> {
    get_element_by_id(id)?
        .dyn_into::<T>()
        .map_err(|_| DomError::InvalidElementType(id.to_string()))
}

/// Create a new element
pub fn create_element(tag: &str) -> DomResult<web_sys::Element> {
    document()?
        .create_element(tag)
        .map_err(|e| DomError::OperationFailed(format!("Failed to create element: {:?}", e)))
}

/// Create a new HTML element with type
pub fn create_html_element<T: JsCast>(tag: &str) -> DomResult<T> {
    create_element(tag)?
        .dyn_into::<T>()
        .map_err(|_| DomError::InvalidElementType(tag.to_string()))
}

/// Create a text node
pub fn create_text_node(text: &str) -> DomResult<web_sys::Text> {
    Ok(document()?.create_text_node(text))
}

/// Query selector on document
pub fn query_selector(selector: &str) -> DomResult<Option<web_sys::Element>> {
    document()?
        .query_selector(selector)
        .map_err(|e| DomError::OperationFailed(format!("Query selector failed: {:?}", e)))
}

/// Query selector all on document
pub fn query_selector_all(selector: &str) -> DomResult<web_sys::NodeList> {
    document()?
        .query_selector_all(selector)
        .map_err(|e| DomError::OperationFailed(format!("Query selector all failed: {:?}", e)))
}

/// Set element text content
pub fn set_text_content(element: &web_sys::Element, text: &str) {
    element.set_text_content(Some(text));
}

/// Set element inner HTML (use carefully - sanitize input!)
pub fn set_inner_html(element: &web_sys::Element, html: &str) {
    element.set_inner_html(html);
}

/// Add class to element
pub fn add_class(element: &web_sys::Element, class: &str) -> DomResult<()> {
    element
        .class_list()
        .add_1(class)
        .map_err(|e| DomError::OperationFailed(format!("Add class failed: {:?}", e)))
}

/// Remove class from element
pub fn remove_class(element: &web_sys::Element, class: &str) -> DomResult<()> {
    element
        .class_list()
        .remove_1(class)
        .map_err(|e| DomError::OperationFailed(format!("Remove class failed: {:?}", e)))
}

/// Toggle class on element
pub fn toggle_class(element: &web_sys::Element, class: &str) -> DomResult<bool> {
    element
        .class_list()
        .toggle(class)
        .map_err(|e| DomError::OperationFailed(format!("Toggle class failed: {:?}", e)))
}

/// Check if element has class
pub fn has_class(element: &web_sys::Element, class: &str) -> bool {
    element.class_list().contains(class)
}

/// Set element attribute
pub fn set_attribute(element: &web_sys::Element, name: &str, value: &str) -> DomResult<()> {
    element
        .set_attribute(name, value)
        .map_err(|e| DomError::OperationFailed(format!("Set attribute failed: {:?}", e)))
}

/// Get element attribute
pub fn get_attribute(element: &web_sys::Element, name: &str) -> Option<String> {
    element.get_attribute(name)
}

/// Remove element attribute
pub fn remove_attribute(element: &web_sys::Element, name: &str) -> DomResult<()> {
    element
        .remove_attribute(name)
        .map_err(|e| DomError::OperationFailed(format!("Remove attribute failed: {:?}", e)))
}

/// Set element style property
pub fn set_style(element: &web_sys::HtmlElement, property: &str, value: &str) -> DomResult<()> {
    element
        .style()
        .set_property(property, value)
        .map_err(|e| DomError::OperationFailed(format!("Set style failed: {:?}", e)))
}

/// Get element style property
pub fn get_style(element: &web_sys::HtmlElement, property: &str) -> DomResult<String> {
    element
        .style()
        .get_property_value(property)
        .map_err(|e| DomError::OperationFailed(format!("Get style failed: {:?}", e)))
}

/// Append child to parent element
pub fn append_child(parent: &web_sys::Element, child: &web_sys::Node) -> DomResult<()> {
    parent
        .append_child(child)
        .map(|_| ())
        .map_err(|e| DomError::OperationFailed(format!("Append child failed: {:?}", e)))
}

/// Remove child from parent
pub fn remove_child(parent: &web_sys::Element, child: &web_sys::Node) -> DomResult<()> {
    parent
        .remove_child(child)
        .map(|_| ())
        .map_err(|e| DomError::OperationFailed(format!("Remove child failed: {:?}", e)))
}

/// Remove element from DOM
pub fn remove_element(element: &web_sys::Element) {
    element.remove();
}

/// Focus an element
pub fn focus(element: &web_sys::HtmlElement) -> DomResult<()> {
    element
        .focus()
        .map_err(|e| DomError::OperationFailed(format!("Focus failed: {:?}", e)))
}

/// Blur an element
pub fn blur(element: &web_sys::HtmlElement) -> DomResult<()> {
    element
        .blur()
        .map_err(|e| DomError::OperationFailed(format!("Blur failed: {:?}", e)))
}

/// Scroll element into view
pub fn scroll_into_view(element: &web_sys::Element) {
    element.scroll_into_view();
}

// ============================================================================
// Event Handling
// ============================================================================

/// Type alias for event callbacks
pub type EventCallback = Closure<dyn FnMut(web_sys::Event)>;

/// Add event listener to element
pub fn add_event_listener(
    target: &web_sys::EventTarget,
    event_type: &str,
    callback: &EventCallback,
) -> DomResult<()> {
    target
        .add_event_listener_with_callback(event_type, callback.as_ref().unchecked_ref())
        .map_err(|e| DomError::EventHandlerFailed(format!("{:?}", e)))
}

/// Remove event listener from element
pub fn remove_event_listener(
    target: &web_sys::EventTarget,
    event_type: &str,
    callback: &EventCallback,
) -> DomResult<()> {
    target
        .remove_event_listener_with_callback(event_type, callback.as_ref().unchecked_ref())
        .map_err(|e| DomError::EventHandlerFailed(format!("{:?}", e)))
}

/// Create a keyboard event callback
pub fn on_keydown<F>(mut handler: F) -> Closure<dyn FnMut(web_sys::Event)>
where
    F: FnMut(web_sys::KeyboardEvent) + 'static,
{
    Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(keyboard_event) = event.dyn_into::<web_sys::KeyboardEvent>() {
            handler(keyboard_event);
        }
    }) as Box<dyn FnMut(web_sys::Event)>)
}

/// Create a mouse event callback
pub fn on_click<F>(mut handler: F) -> Closure<dyn FnMut(web_sys::Event)>
where
    F: FnMut(web_sys::MouseEvent) + 'static,
{
    Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(mouse_event) = event.dyn_into::<web_sys::MouseEvent>() {
            handler(mouse_event);
        }
    }) as Box<dyn FnMut(web_sys::Event)>)
}

/// Create an input event callback
pub fn on_input<F>(mut handler: F) -> Closure<dyn FnMut(web_sys::Event)>
where
    F: FnMut(web_sys::InputEvent) + 'static,
{
    Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(input_event) = event.dyn_into::<web_sys::InputEvent>() {
            handler(input_event);
        }
    }) as Box<dyn FnMut(web_sys::Event)>)
}

// ============================================================================
// Storage Operations
// ============================================================================

/// Get local storage
pub fn local_storage() -> DomResult<web_sys::Storage> {
    window()?
        .local_storage()
        .map_err(|e| DomError::StorageError(format!("{:?}", e)))?
        .ok_or(DomError::StorageNotAvailable)
}

/// Get session storage
pub fn session_storage() -> DomResult<web_sys::Storage> {
    window()?
        .session_storage()
        .map_err(|e| DomError::StorageError(format!("{:?}", e)))?
        .ok_or(DomError::StorageNotAvailable)
}

/// Set item in storage
pub fn storage_set(storage: &web_sys::Storage, key: &str, value: &str) -> DomResult<()> {
    storage
        .set_item(key, value)
        .map_err(|e| DomError::StorageError(format!("{:?}", e)))
}

/// Get item from storage
pub fn storage_get(storage: &web_sys::Storage, key: &str) -> DomResult<Option<String>> {
    storage
        .get_item(key)
        .map_err(|e| DomError::StorageError(format!("{:?}", e)))
}

/// Remove item from storage
pub fn storage_remove(storage: &web_sys::Storage, key: &str) -> DomResult<()> {
    storage
        .remove_item(key)
        .map_err(|e| DomError::StorageError(format!("{:?}", e)))
}

/// Clear all storage
pub fn storage_clear(storage: &web_sys::Storage) -> DomResult<()> {
    storage
        .clear()
        .map_err(|e| DomError::StorageError(format!("{:?}", e)))
}

// ============================================================================
// Animation Frame
// ============================================================================

/// Request animation frame
pub fn request_animation_frame(callback: &Closure<dyn FnMut()>) -> DomResult<i32> {
    window()?
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|_| DomError::AnimationFrameFailed)
}

/// Cancel animation frame
pub fn cancel_animation_frame(handle: i32) -> DomResult<()> {
    window()?
        .cancel_animation_frame(handle)
        .map_err(|e| DomError::OperationFailed(format!("Cancel animation frame failed: {:?}", e)))
}

// ============================================================================
// Performance
// ============================================================================

/// Get performance.now() timestamp
pub fn performance_now() -> DomResult<f64> {
    Ok(window()?.performance().ok_or(DomError::NoWindow)?.now())
}

// ============================================================================
// URL and History
// ============================================================================

/// Get current URL search params
pub fn get_url_search_params() -> DomResult<web_sys::UrlSearchParams> {
    let location = window()?.location();
    let search = location
        .search()
        .map_err(|e| DomError::OperationFailed(format!("Failed to get search params: {:?}", e)))?;
    web_sys::UrlSearchParams::new_with_str(&search)
        .map_err(|e| DomError::OperationFailed(format!("Failed to parse search params: {:?}", e)))
}

/// Get URL parameter
pub fn get_url_param(name: &str) -> DomResult<Option<String>> {
    Ok(get_url_search_params()?.get(name))
}

// ============================================================================
// Console Logging (for debugging)
// ============================================================================

/// Log to browser console
pub fn console_log(message: &str) {
    web_sys::console::log_1(&message.into());
}

/// Log error to browser console
pub fn console_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

/// Log warning to browser console
pub fn console_warn(message: &str) {
    web_sys::console::warn_1(&message.into());
}

/// Log info to browser console
pub fn console_info(message: &str) {
    web_sys::console::info_1(&message.into());
}

// ============================================================================
// Terminal Component
// ============================================================================

/// Terminal component state
pub struct Terminal {
    /// Terminal output element
    output_element: web_sys::Element,
    /// Terminal input element
    input_element: web_sys::HtmlInputElement,
    /// Command history
    history: Vec<String>,
    /// Current history position
    history_position: usize,
}

impl Terminal {
    /// Create a new terminal component
    pub fn new(output_id: &str, input_id: &str) -> DomResult<Self> {
        let output_element = get_element_by_id(output_id)?;
        let input_element: web_sys::HtmlInputElement = get_html_element_by_id(input_id)?;

        Ok(Self {
            output_element,
            input_element,
            history: Vec::new(),
            history_position: 0,
        })
    }

    /// Append output line to terminal
    pub fn append_output(&self, text: &str) -> DomResult<()> {
        let line = create_element("div")?;
        set_text_content(&line, text);
        add_class(&line, "terminal-line")?;
        append_child(&self.output_element, &line)?;
        scroll_into_view(&line);
        Ok(())
    }

    /// Append command echo to terminal
    pub fn append_command(&self, command: &str) -> DomResult<()> {
        let line = create_element("div")?;
        set_text_content(&line, &format!("wos$ {}", command));
        add_class(&line, "terminal-line")?;
        add_class(&line, "command-line")?;
        append_child(&self.output_element, &line)?;
        Ok(())
    }

    /// Append error output
    pub fn append_error(&self, text: &str) -> DomResult<()> {
        let line = create_element("div")?;
        set_text_content(&line, text);
        add_class(&line, "terminal-line")?;
        add_class(&line, "error-line")?;
        append_child(&self.output_element, &line)?;
        scroll_into_view(&line);
        Ok(())
    }

    /// Clear terminal output
    pub fn clear(&self) {
        set_inner_html(&self.output_element, "");
    }

    /// Get current input value
    pub fn get_input(&self) -> String {
        self.input_element.value()
    }

    /// Set input value
    pub fn set_input(&self, value: &str) {
        self.input_element.set_value(value);
    }

    /// Clear input
    pub fn clear_input(&self) {
        self.input_element.set_value("");
    }

    /// Focus input
    pub fn focus_input(&self) -> DomResult<()> {
        focus(&self.input_element)
    }

    /// Add command to history
    pub fn add_to_history(&mut self, command: &str) {
        if !command.is_empty() {
            self.history.push(command.to_string());
            self.history_position = self.history.len();
        }
    }

    /// Navigate history up
    pub fn history_up(&mut self) -> Option<&str> {
        if self.history_position > 0 {
            self.history_position -= 1;
            self.history.get(self.history_position).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Navigate history down
    pub fn history_down(&mut self) -> Option<&str> {
        if self.history_position < self.history.len() {
            self.history_position += 1;
            if self.history_position < self.history.len() {
                self.history.get(self.history_position).map(|s| s.as_str())
            } else {
                Some("")
            }
        } else {
            None
        }
    }

    /// Get history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

// ============================================================================
// Application State
// ============================================================================

/// Application state shared across components
pub struct AppState {
    /// Terminal component
    pub terminal: Terminal,
    /// Event listeners (must be kept alive)
    listeners: Vec<EventCallback>,
}

impl AppState {
    /// Create new application state
    pub fn new(terminal: Terminal) -> Self {
        Self {
            terminal,
            listeners: Vec::new(),
        }
    }

    /// Register an event listener (keeps closure alive)
    pub fn register_listener(&mut self, listener: EventCallback) {
        self.listeners.push(listener);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_error_display() {
        assert_eq!(
            DomError::NoWindow.to_string(),
            "Window object not available"
        );
        assert_eq!(
            DomError::NoDocument.to_string(),
            "Document object not available"
        );
        assert_eq!(
            DomError::ElementNotFound("test".to_string()).to_string(),
            "Element not found: test"
        );
    }

    #[test]
    fn test_dom_error_equality() {
        assert_eq!(DomError::NoWindow, DomError::NoWindow);
        assert_ne!(DomError::NoWindow, DomError::NoDocument);
        assert_eq!(
            DomError::ElementNotFound("a".to_string()),
            DomError::ElementNotFound("a".to_string())
        );
    }
}
