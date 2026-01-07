//! DOM manipulation utilities
//!
//! Provides type-safe wrappers around web-sys DOM operations.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, Window};

/// DOM manipulation helper
pub struct Dom;

#[cfg(feature = "wasm")]
impl Dom {
    /// Get the window object
    #[must_use]
    pub fn window() -> Option<Window> {
        web_sys::window()
    }

    /// Get the document object
    #[must_use]
    pub fn document() -> Option<Document> {
        Self::window()?.document()
    }

    /// Get an element by ID
    #[must_use]
    pub fn get_element_by_id(id: &str) -> Option<Element> {
        Self::document()?.get_element_by_id(id)
    }

    /// Get an HTML element by ID
    #[must_use]
    pub fn get_html_element_by_id(id: &str) -> Option<HtmlElement> {
        Self::get_element_by_id(id)?.dyn_into::<HtmlElement>().ok()
    }

    /// Get an input element by ID
    #[must_use]
    pub fn get_input_by_id(id: &str) -> Option<HtmlInputElement> {
        Self::get_element_by_id(id)?
            .dyn_into::<HtmlInputElement>()
            .ok()
    }

    /// Create an element
    #[must_use]
    pub fn create_element(tag: &str) -> Option<Element> {
        Self::document()?.create_element(tag).ok()
    }

    /// Create a div element
    #[must_use]
    pub fn create_div() -> Option<HtmlElement> {
        Self::create_element("div")?.dyn_into::<HtmlElement>().ok()
    }

    /// Create a span element
    #[must_use]
    pub fn create_span() -> Option<HtmlElement> {
        Self::create_element("span")?.dyn_into::<HtmlElement>().ok()
    }

    /// Create a text node
    #[must_use]
    pub fn create_text(content: &str) -> Option<web_sys::Text> {
        Self::document()?.create_text_node(content).into()
    }

    /// Query selector
    #[must_use]
    pub fn query_selector(selector: &str) -> Option<Element> {
        Self::document()?.query_selector(selector).ok()?
    }

    /// Query selector all
    #[must_use]
    pub fn query_selector_all(selector: &str) -> Option<web_sys::NodeList> {
        Self::document()?.query_selector_all(selector).ok()
    }

    /// Get localStorage
    #[must_use]
    pub fn local_storage() -> Option<web_sys::Storage> {
        Self::window()?.local_storage().ok()?
    }

    /// Get item from localStorage
    #[must_use]
    pub fn get_storage_item(key: &str) -> Option<String> {
        Self::local_storage()?.get_item(key).ok()?
    }

    /// Set item in localStorage
    pub fn set_storage_item(key: &str, value: &str) -> bool {
        Self::local_storage()
            .map(|s| s.set_item(key, value).is_ok())
            .unwrap_or(false)
    }

    /// Remove item from localStorage
    pub fn remove_storage_item(key: &str) -> bool {
        Self::local_storage()
            .map(|s| s.remove_item(key).is_ok())
            .unwrap_or(false)
    }

    /// Get URL search params
    #[must_use]
    pub fn url_params() -> Option<web_sys::UrlSearchParams> {
        let window = Self::window()?;
        let location = window.location();
        let search = location.search().ok()?;
        web_sys::UrlSearchParams::new_with_str(&search).ok()
    }

    /// Get URL parameter
    #[must_use]
    pub fn get_url_param(key: &str) -> Option<String> {
        Self::url_params()?.get(key)
    }

    /// Focus an element
    pub fn focus(element: &HtmlElement) {
        let _ = element.focus();
    }

    /// Scroll element into view
    pub fn scroll_into_view(element: &Element) {
        element.scroll_into_view();
    }

    /// Set element text content
    pub fn set_text_content(element: &Element, content: &str) {
        element.set_text_content(Some(content));
    }

    /// Set element inner HTML (use carefully)
    pub fn set_inner_html(element: &Element, html: &str) {
        element.set_inner_html(html);
    }

    /// Add class to element
    pub fn add_class(element: &Element, class: &str) {
        let _ = element.class_list().add_1(class);
    }

    /// Remove class from element
    pub fn remove_class(element: &Element, class: &str) {
        let _ = element.class_list().remove_1(class);
    }

    /// Toggle class on element
    pub fn toggle_class(element: &Element, class: &str) -> bool {
        element.class_list().toggle(class).unwrap_or(false)
    }

    /// Check if element has class
    #[must_use]
    pub fn has_class(element: &Element, class: &str) -> bool {
        element.class_list().contains(class)
    }

    /// Set element attribute
    pub fn set_attribute(element: &Element, name: &str, value: &str) {
        let _ = element.set_attribute(name, value);
    }

    /// Get element attribute
    #[must_use]
    pub fn get_attribute(element: &Element, name: &str) -> Option<String> {
        element.get_attribute(name)
    }

    /// Set element style property
    pub fn set_style(element: &HtmlElement, property: &str, value: &str) {
        let _ = element.style().set_property(property, value);
    }

    /// Get current time in milliseconds
    #[must_use]
    pub fn now() -> f64 {
        Self::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }
}

#[cfg(not(feature = "wasm"))]
impl Dom {
    /// Stub implementations for non-WASM testing
    #[must_use]
    pub fn now() -> f64 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_now_non_wasm() {
        // Non-WASM returns 0.0
        assert_eq!(Dom::now(), 0.0);
    }
}
