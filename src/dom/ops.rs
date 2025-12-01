//! DOM operation types and batch processing - Python bindings.
//!
//! This module provides Python bindings for the DOM operations defined
//! in `auroraview-core`. The core logic (DomOp, DomBatch) lives in the
//! core crate, and this module adds PyO3 bindings.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

// Re-export core types for Rust users
pub use auroraview_core::dom::DomOp;

// Import core batch for internal use
use auroraview_core::dom::DomBatch as CoreDomBatch;

/// A batch of DOM operations for high-performance execution.
///
/// This is a wrapper around `auroraview_core::dom::DomBatch` that adds
/// Python bindings via PyO3.
#[cfg_attr(feature = "python-bindings", pyclass)]
#[derive(Debug, Clone, Default)]
pub struct DomBatch {
    inner: CoreDomBatch,
}

impl DomBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self {
            inner: CoreDomBatch::new(),
        }
    }

    /// Create a batch with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: CoreDomBatch::with_capacity(capacity),
        }
    }

    /// Add an operation to the batch.
    pub fn push(&mut self, op: DomOp) {
        self.inner.push(op);
    }

    /// Get the number of operations in the batch.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all operations from the batch.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Generate optimized JavaScript for all operations.
    pub fn to_js(&self) -> String {
        self.inner.to_js()
    }
}

// === Convenience methods - delegate to inner ===
impl DomBatch {
    pub fn set_text(&mut self, selector: &str, text: &str) -> &mut Self {
        self.inner.set_text(selector, text);
        self
    }
    pub fn set_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.inner.set_html(selector, html);
        self
    }
    pub fn set_attribute(&mut self, selector: &str, name: &str, value: &str) -> &mut Self {
        self.inner.set_attribute(selector, name, value);
        self
    }
    pub fn remove_attribute(&mut self, selector: &str, name: &str) -> &mut Self {
        self.inner.remove_attribute(selector, name);
        self
    }
    pub fn add_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.inner.add_class(selector, class);
        self
    }
    pub fn remove_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.inner.remove_class(selector, class);
        self
    }
    pub fn toggle_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.inner.toggle_class(selector, class);
        self
    }
    pub fn set_style(&mut self, selector: &str, property: &str, value: &str) -> &mut Self {
        self.inner.set_style(selector, property, value);
        self
    }
    pub fn show(&mut self, selector: &str) -> &mut Self {
        self.inner.show(selector);
        self
    }
    pub fn hide(&mut self, selector: &str) -> &mut Self {
        self.inner.hide(selector);
        self
    }
    pub fn set_value(&mut self, selector: &str, value: &str) -> &mut Self {
        self.inner.set_value(selector, value);
        self
    }
    pub fn set_checked(&mut self, selector: &str, checked: bool) -> &mut Self {
        self.inner.set_checked(selector, checked);
        self
    }
    pub fn set_disabled(&mut self, selector: &str, disabled: bool) -> &mut Self {
        self.inner.set_disabled(selector, disabled);
        self
    }
    pub fn click(&mut self, selector: &str) -> &mut Self {
        self.inner.click(selector);
        self
    }
    pub fn double_click(&mut self, selector: &str) -> &mut Self {
        self.inner.double_click(selector);
        self
    }
    pub fn focus(&mut self, selector: &str) -> &mut Self {
        self.inner.focus(selector);
        self
    }
    pub fn blur(&mut self, selector: &str) -> &mut Self {
        self.inner.blur(selector);
        self
    }
    pub fn scroll_into_view(&mut self, selector: &str, smooth: bool) -> &mut Self {
        self.inner.scroll_into_view(selector, smooth);
        self
    }
    pub fn type_text(&mut self, selector: &str, text: &str, clear: bool) -> &mut Self {
        self.inner.type_text(selector, text, clear);
        self
    }
    pub fn clear_input(&mut self, selector: &str) -> &mut Self {
        self.inner.clear_input(selector);
        self
    }
    pub fn submit(&mut self, selector: &str) -> &mut Self {
        self.inner.submit(selector);
        self
    }
    pub fn append_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.inner.append_html(selector, html);
        self
    }
    pub fn prepend_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.inner.prepend_html(selector, html);
        self
    }
    pub fn remove(&mut self, selector: &str) -> &mut Self {
        self.inner.remove(selector);
        self
    }
    pub fn empty(&mut self, selector: &str) -> &mut Self {
        self.inner.empty(selector);
        self
    }
    pub fn raw(&mut self, selector: &str, script: &str) -> &mut Self {
        self.inner.raw(selector, script);
        self
    }
    pub fn raw_global(&mut self, script: &str) -> &mut Self {
        self.inner.raw_global(script);
        self
    }
}

// === Python bindings ===
#[cfg(feature = "python-bindings")]
#[pymethods]
impl DomBatch {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    #[getter]
    pub fn count(&self) -> usize {
        self.len()
    }

    #[getter]
    pub fn is_empty_prop(&self) -> bool {
        self.is_empty()
    }

    #[pyo3(name = "clear")]
    pub fn py_clear(&mut self) {
        self.clear();
    }

    #[pyo3(name = "to_js")]
    pub fn py_to_js(&self) -> String {
        self.to_js()
    }

    #[pyo3(name = "set_text")]
    pub fn py_set_text(&mut self, selector: &str, text: &str) {
        self.set_text(selector, text);
    }
    #[pyo3(name = "set_html")]
    pub fn py_set_html(&mut self, selector: &str, html: &str) {
        self.set_html(selector, html);
    }
    #[pyo3(name = "set_attribute")]
    pub fn py_set_attribute(&mut self, selector: &str, name: &str, value: &str) {
        self.set_attribute(selector, name, value);
    }
    #[pyo3(name = "remove_attribute")]
    pub fn py_remove_attribute(&mut self, selector: &str, name: &str) {
        self.remove_attribute(selector, name);
    }
    #[pyo3(name = "add_class")]
    pub fn py_add_class(&mut self, selector: &str, class_name: &str) {
        self.add_class(selector, class_name);
    }
    #[pyo3(name = "remove_class")]
    pub fn py_remove_class(&mut self, selector: &str, class_name: &str) {
        self.remove_class(selector, class_name);
    }
    #[pyo3(name = "toggle_class")]
    pub fn py_toggle_class(&mut self, selector: &str, class_name: &str) {
        self.toggle_class(selector, class_name);
    }
    #[pyo3(name = "set_style")]
    pub fn py_set_style(&mut self, selector: &str, property: &str, value: &str) {
        self.set_style(selector, property, value);
    }
    #[pyo3(name = "show")]
    pub fn py_show(&mut self, selector: &str) {
        self.show(selector);
    }
    #[pyo3(name = "hide")]
    pub fn py_hide(&mut self, selector: &str) {
        self.hide(selector);
    }
    #[pyo3(name = "set_value")]
    pub fn py_set_value(&mut self, selector: &str, value: &str) {
        self.set_value(selector, value);
    }
    #[pyo3(name = "set_checked")]
    pub fn py_set_checked(&mut self, selector: &str, checked: bool) {
        self.set_checked(selector, checked);
    }
    #[pyo3(name = "set_disabled")]
    pub fn py_set_disabled(&mut self, selector: &str, disabled: bool) {
        self.set_disabled(selector, disabled);
    }
    #[pyo3(name = "click")]
    pub fn py_click(&mut self, selector: &str) {
        self.click(selector);
    }
    #[pyo3(name = "double_click")]
    pub fn py_double_click(&mut self, selector: &str) {
        self.double_click(selector);
    }
    #[pyo3(name = "focus")]
    pub fn py_focus(&mut self, selector: &str) {
        self.focus(selector);
    }
    #[pyo3(name = "blur")]
    pub fn py_blur(&mut self, selector: &str) {
        self.blur(selector);
    }
    #[pyo3(name = "scroll_into_view")]
    #[pyo3(signature = (selector, smooth=true))]
    pub fn py_scroll_into_view(&mut self, selector: &str, smooth: bool) {
        self.scroll_into_view(selector, smooth);
    }
    #[pyo3(name = "type_text")]
    #[pyo3(signature = (selector, text, clear=false))]
    pub fn py_type_text(&mut self, selector: &str, text: &str, clear: bool) {
        self.type_text(selector, text, clear);
    }
    #[pyo3(name = "clear_input")]
    pub fn py_clear_input(&mut self, selector: &str) {
        self.clear_input(selector);
    }
    #[pyo3(name = "submit")]
    pub fn py_submit(&mut self, selector: &str) {
        self.submit(selector);
    }
    #[pyo3(name = "append_html")]
    pub fn py_append_html(&mut self, selector: &str, html: &str) {
        self.append_html(selector, html);
    }
    #[pyo3(name = "prepend_html")]
    pub fn py_prepend_html(&mut self, selector: &str, html: &str) {
        self.prepend_html(selector, html);
    }
    #[pyo3(name = "remove")]
    pub fn py_remove(&mut self, selector: &str) {
        self.remove(selector);
    }
    #[pyo3(name = "empty")]
    pub fn py_empty(&mut self, selector: &str) {
        self.empty(selector);
    }
    #[pyo3(name = "raw")]
    pub fn py_raw(&mut self, selector: &str, script: &str) {
        self.raw(selector, script);
    }
    #[pyo3(name = "raw_global")]
    pub fn py_raw_global(&mut self, script: &str) {
        self.raw_global(script);
    }

    fn __repr__(&self) -> String {
        format!("DomBatch(operations={})", self.len())
    }
    fn __str__(&self) -> String {
        self.__repr__()
    }
    fn __len__(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_batch_new() {
        let batch = DomBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_dom_batch_with_capacity() {
        let batch = DomBatch::with_capacity(10);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_dom_batch_push() {
        let mut batch = DomBatch::new();
        batch.push(DomOp::Click {
            selector: "#btn".to_string(),
        });
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_dom_batch_count() {
        let mut batch = DomBatch::new();
        assert_eq!(batch.len(), 0);
        batch.set_text("#a", "1");
        batch.set_text("#b", "2");
        assert_eq!(batch.len(), 2);
        batch.clear();
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_dom_batch_empty() {
        let batch = DomBatch::new();
        let js = batch.to_js();
        assert_eq!(js, "(function(){})()");
    }

    #[test]
    fn test_dom_batch_is_wrapped_in_iife() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "value");
        let js = batch.to_js();
        assert!(js.starts_with("(function(){"));
        assert!(js.ends_with("})()"));
    }

    #[test]
    fn test_set_text() {
        let mut batch = DomBatch::new();
        batch.set_text("#title", "Hello");
        let js = batch.to_js();
        assert!(js.contains("textContent"));
        assert!(js.contains("Hello"));
    }

    #[test]
    fn test_add_class() {
        let mut batch = DomBatch::new();
        batch.add_class(".item", "active");
        let js = batch.to_js();
        assert!(js.contains("classList.add"));
        assert!(js.contains("active"));
    }

    #[test]
    fn test_click() {
        let mut batch = DomBatch::new();
        batch.click("#button");
        let js = batch.to_js();
        assert!(js.contains(".click()"));
    }

    #[test]
    fn test_chained_operations() {
        let mut batch = DomBatch::new();
        batch
            .set_text("#title", "Hello")
            .add_class(".item", "active")
            .click("#submit");
        assert_eq!(batch.len(), 3);
    }
}
