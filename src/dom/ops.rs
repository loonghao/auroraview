//! DOM operation types and batch processing.
//!
//! This module defines all supported DOM operations and provides a high-performance
//! batch processor that generates optimized JavaScript code.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

/// Represents a single DOM operation.
///
/// Each variant maps to a specific DOM manipulation that will be
/// converted to JavaScript code.
#[derive(Debug, Clone, PartialEq)]
pub enum DomOp {
    // === Text & Content ===
    /// Set element's text content
    SetText { selector: String, text: String },
    /// Set element's innerHTML
    SetHtml { selector: String, html: String },

    // === Attributes ===
    /// Set an attribute value
    SetAttribute {
        selector: String,
        name: String,
        value: String,
    },
    /// Remove an attribute
    RemoveAttribute { selector: String, name: String },

    // === Classes ===
    /// Add a CSS class
    AddClass { selector: String, class: String },
    /// Remove a CSS class
    RemoveClass { selector: String, class: String },
    /// Toggle a CSS class
    ToggleClass { selector: String, class: String },

    // === Styles ===
    /// Set a CSS style property
    SetStyle {
        selector: String,
        property: String,
        value: String,
    },
    /// Set multiple CSS styles at once
    SetStyles {
        selector: String,
        styles: Vec<(String, String)>,
    },

    // === Visibility ===
    /// Show element (display: '')
    Show { selector: String },
    /// Hide element (display: none)
    Hide { selector: String },

    // === Forms ===
    /// Set input/textarea value
    SetValue { selector: String, value: String },
    /// Set checkbox/radio checked state
    SetChecked { selector: String, checked: bool },
    /// Set element disabled state
    SetDisabled { selector: String, disabled: bool },
    /// Select an option by value
    SelectOption { selector: String, value: String },

    // === Interactions ===
    /// Click an element
    Click { selector: String },
    /// Double-click an element
    DoubleClick { selector: String },
    /// Focus an element
    Focus { selector: String },
    /// Blur (unfocus) an element
    Blur { selector: String },
    /// Scroll element into view
    ScrollIntoView { selector: String, smooth: bool },

    // === Input ===
    /// Type text into an input (simulates keystrokes)
    TypeText {
        selector: String,
        text: String,
        clear: bool,
    },
    /// Clear input value
    Clear { selector: String },
    /// Submit a form
    Submit { selector: String },

    // === DOM Manipulation ===
    /// Append HTML inside element
    AppendHtml { selector: String, html: String },
    /// Prepend HTML inside element
    PrependHtml { selector: String, html: String },
    /// Remove element from DOM
    Remove { selector: String },
    /// Empty element's content
    Empty { selector: String },

    // === Custom ===
    /// Execute raw JavaScript on element
    Raw { selector: String, script: String },
    /// Execute global JavaScript (no element selection)
    RawGlobal { script: String },
}

/// A batch of DOM operations for high-performance execution.
#[cfg_attr(feature = "python-bindings", pyclass)]
#[derive(Debug, Clone, Default)]
pub struct DomBatch {
    operations: Vec<DomOp>,
}

impl DomBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Create a batch with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            operations: Vec::with_capacity(capacity),
        }
    }

    /// Add an operation to the batch.
    pub fn push(&mut self, op: DomOp) {
        self.operations.push(op);
    }

    /// Get the number of operations in the batch.
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Clear all operations from the batch.
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Escape a CSS selector for use in JavaScript.
    fn escape_selector(selector: &str) -> String {
        selector
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    /// Escape a string value for JavaScript.
    fn escape_string(value: &str) -> String {
        let mut result = String::with_capacity(value.len() + 10);
        for ch in value.chars() {
            match ch {
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\'' => result.push_str("\\'"),
                _ => result.push(ch),
            }
        }
        result
    }

    /// Generate JavaScript code for a single operation.
    fn op_to_js(op: &DomOp) -> String {
        match op {
            DomOp::SetText { selector, text } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.textContent=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(text)
                )
            }
            DomOp::SetHtml { selector, html } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.innerHTML=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(html)
                )
            }
            DomOp::SetAttribute {
                selector,
                name,
                value,
            } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.setAttribute('{}',\"{}\");",
                    Self::escape_selector(selector),
                    Self::escape_string(name),
                    Self::escape_string(value)
                )
            }
            DomOp::RemoveAttribute { selector, name } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.removeAttribute('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(name)
                )
            }
            DomOp::AddClass { selector, class } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.classList.add('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(class)
                )
            }
            DomOp::RemoveClass { selector, class } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.classList.remove('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(class)
                )
            }
            DomOp::ToggleClass { selector, class } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.classList.toggle('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(class)
                )
            }
            DomOp::SetStyle {
                selector,
                property,
                value,
            } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.style['{}']=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(property),
                    Self::escape_string(value)
                )
            }
            DomOp::SetStyles { selector, styles } => {
                let style_assignments: String = styles
                    .iter()
                    .map(|(prop, val)| {
                        format!(
                            "e.style['{}']=\"{}\";",
                            Self::escape_string(prop),
                            Self::escape_string(val)
                        )
                    })
                    .collect();
                format!(
                    "var e=document.querySelector('{}');if(e){{{}}}",
                    Self::escape_selector(selector),
                    style_assignments
                )
            }
            DomOp::Show { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.style.display='';",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Hide { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.style.display='none';",
                    Self::escape_selector(selector)
                )
            }
            DomOp::SetValue { selector, value } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.value=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(value)
                )
            }
            DomOp::SetChecked { selector, checked } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.checked={};",
                    Self::escape_selector(selector),
                    checked
                )
            }
            DomOp::SetDisabled { selector, disabled } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.disabled={};",
                    Self::escape_selector(selector),
                    disabled
                )
            }
            DomOp::SelectOption { selector, value } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.value=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(value)
                )
            }
            DomOp::Click { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.click();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::DoubleClick { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{var ev=new MouseEvent('dblclick',{{bubbles:true}});e.dispatchEvent(ev);}}",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Focus { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.focus();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Blur { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.blur();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::ScrollIntoView { selector, smooth } => {
                let behavior = if *smooth { "smooth" } else { "auto" };
                format!(
                    "var e=document.querySelector('{}');if(e)e.scrollIntoView({{behavior:'{}'}});",
                    Self::escape_selector(selector),
                    behavior
                )
            }
            DomOp::TypeText {
                selector,
                text,
                clear,
            } => {
                let clear_code = if *clear { "e.value='';" } else { "" };
                format!(
                    "var e=document.querySelector('{}');if(e){{{}\"{}\".split('').forEach(function(c){{e.value+=c;e.dispatchEvent(new Event('input',{{bubbles:true}}));}});}}",
                    Self::escape_selector(selector),
                    clear_code,
                    Self::escape_string(text)
                )
            }
            DomOp::Clear { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{e.value='';e.dispatchEvent(new Event('input',{{bubbles:true}}));}}",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Submit { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{var f=e.closest('form');if(f)f.submit();else if(e.tagName==='FORM')e.submit();}}",
                    Self::escape_selector(selector)
                )
            }
            DomOp::AppendHtml { selector, html } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.insertAdjacentHTML('beforeend',\"{}\");",
                    Self::escape_selector(selector),
                    Self::escape_string(html)
                )
            }
            DomOp::PrependHtml { selector, html } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.insertAdjacentHTML('afterbegin',\"{}\");",
                    Self::escape_selector(selector),
                    Self::escape_string(html)
                )
            }
            DomOp::Remove { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.remove();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Empty { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.innerHTML='';",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Raw { selector, script } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{{}}}",
                    Self::escape_selector(selector),
                    script
                )
            }
            DomOp::RawGlobal { script } => script.clone(),
        }
    }

    /// Generate optimized JavaScript for all operations.
    pub fn to_js(&self) -> String {
        if self.operations.is_empty() {
            return String::from("(function(){})()");
        }
        let mut js = String::with_capacity(self.operations.len() * 100);
        js.push_str("(function(){");
        for op in &self.operations {
            js.push_str(&Self::op_to_js(op));
        }
        js.push_str("})()");
        js
    }
}

// === Convenience methods ===
impl DomBatch {
    pub fn set_text(&mut self, selector: &str, text: &str) -> &mut Self {
        self.push(DomOp::SetText {
            selector: selector.to_string(),
            text: text.to_string(),
        });
        self
    }
    pub fn set_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.push(DomOp::SetHtml {
            selector: selector.to_string(),
            html: html.to_string(),
        });
        self
    }
    pub fn set_attribute(&mut self, selector: &str, name: &str, value: &str) -> &mut Self {
        self.push(DomOp::SetAttribute {
            selector: selector.to_string(),
            name: name.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn remove_attribute(&mut self, selector: &str, name: &str) -> &mut Self {
        self.push(DomOp::RemoveAttribute {
            selector: selector.to_string(),
            name: name.to_string(),
        });
        self
    }
    pub fn add_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.push(DomOp::AddClass {
            selector: selector.to_string(),
            class: class.to_string(),
        });
        self
    }
    pub fn remove_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.push(DomOp::RemoveClass {
            selector: selector.to_string(),
            class: class.to_string(),
        });
        self
    }
    pub fn toggle_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.push(DomOp::ToggleClass {
            selector: selector.to_string(),
            class: class.to_string(),
        });
        self
    }
    pub fn set_style(&mut self, selector: &str, property: &str, value: &str) -> &mut Self {
        self.push(DomOp::SetStyle {
            selector: selector.to_string(),
            property: property.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn show(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Show {
            selector: selector.to_string(),
        });
        self
    }
    pub fn hide(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Hide {
            selector: selector.to_string(),
        });
        self
    }
    pub fn set_value(&mut self, selector: &str, value: &str) -> &mut Self {
        self.push(DomOp::SetValue {
            selector: selector.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn set_checked(&mut self, selector: &str, checked: bool) -> &mut Self {
        self.push(DomOp::SetChecked {
            selector: selector.to_string(),
            checked,
        });
        self
    }
    pub fn set_disabled(&mut self, selector: &str, disabled: bool) -> &mut Self {
        self.push(DomOp::SetDisabled {
            selector: selector.to_string(),
            disabled,
        });
        self
    }
    pub fn click(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Click {
            selector: selector.to_string(),
        });
        self
    }
    pub fn double_click(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::DoubleClick {
            selector: selector.to_string(),
        });
        self
    }
    pub fn focus(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Focus {
            selector: selector.to_string(),
        });
        self
    }
    pub fn blur(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Blur {
            selector: selector.to_string(),
        });
        self
    }
    pub fn scroll_into_view(&mut self, selector: &str, smooth: bool) -> &mut Self {
        self.push(DomOp::ScrollIntoView {
            selector: selector.to_string(),
            smooth,
        });
        self
    }
    pub fn type_text(&mut self, selector: &str, text: &str, clear: bool) -> &mut Self {
        self.push(DomOp::TypeText {
            selector: selector.to_string(),
            text: text.to_string(),
            clear,
        });
        self
    }
    pub fn clear_input(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Clear {
            selector: selector.to_string(),
        });
        self
    }
    pub fn submit(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Submit {
            selector: selector.to_string(),
        });
        self
    }
    pub fn append_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.push(DomOp::AppendHtml {
            selector: selector.to_string(),
            html: html.to_string(),
        });
        self
    }
    pub fn prepend_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.push(DomOp::PrependHtml {
            selector: selector.to_string(),
            html: html.to_string(),
        });
        self
    }
    pub fn remove(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Remove {
            selector: selector.to_string(),
        });
        self
    }
    pub fn empty(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Empty {
            selector: selector.to_string(),
        });
        self
    }
    pub fn raw(&mut self, selector: &str, script: &str) -> &mut Self {
        self.push(DomOp::Raw {
            selector: selector.to_string(),
            script: script.to_string(),
        });
        self
    }
    pub fn raw_global(&mut self, script: &str) -> &mut Self {
        self.push(DomOp::RawGlobal {
            script: script.to_string(),
        });
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

    // ============ Basic Batch Tests ============

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

    // ============ Text Operations ============

    #[test]
    fn test_set_text() {
        let mut batch = DomBatch::new();
        batch.set_text("#title", "Hello World");
        let js = batch.to_js();
        assert!(js.contains("querySelector('#title')"));
        assert!(js.contains("textContent"));
        assert!(js.contains("Hello World"));
    }

    #[test]
    fn test_set_html() {
        let mut batch = DomBatch::new();
        batch.set_html("#container", "<div>Content</div>");
        let js = batch.to_js();
        assert!(js.contains("innerHTML"));
        assert!(js.contains("<div>Content</div>"));
    }

    // ============ Attribute Operations ============

    #[test]
    fn test_set_attribute() {
        let mut batch = DomBatch::new();
        batch.set_attribute("#link", "href", "https://example.com");
        let js = batch.to_js();
        assert!(js.contains("setAttribute"));
        assert!(js.contains("href"));
        assert!(js.contains("https://example.com"));
    }

    #[test]
    fn test_remove_attribute() {
        let mut batch = DomBatch::new();
        batch.remove_attribute("#input", "disabled");
        let js = batch.to_js();
        assert!(js.contains("removeAttribute"));
        assert!(js.contains("disabled"));
    }

    // ============ Class Operations ============

    #[test]
    fn test_add_class() {
        let mut batch = DomBatch::new();
        batch.add_class(".item", "active");
        let js = batch.to_js();
        assert!(js.contains("classList.add"));
        assert!(js.contains("active"));
    }

    #[test]
    fn test_remove_class() {
        let mut batch = DomBatch::new();
        batch.remove_class(".item", "hidden");
        let js = batch.to_js();
        assert!(js.contains("classList.remove"));
        assert!(js.contains("hidden"));
    }

    #[test]
    fn test_toggle_class() {
        let mut batch = DomBatch::new();
        batch.toggle_class(".item", "expanded");
        let js = batch.to_js();
        assert!(js.contains("classList.toggle"));
        assert!(js.contains("expanded"));
    }

    // ============ Style Operations ============

    #[test]
    fn test_set_style() {
        let mut batch = DomBatch::new();
        batch.set_style("#box", "background-color", "red");
        let js = batch.to_js();
        assert!(js.contains("style['background-color']"));
        assert!(js.contains("red"));
    }

    #[test]
    fn test_show() {
        let mut batch = DomBatch::new();
        batch.show("#modal");
        let js = batch.to_js();
        assert!(js.contains("style.display=''"));
    }

    #[test]
    fn test_hide() {
        let mut batch = DomBatch::new();
        batch.hide("#modal");
        let js = batch.to_js();
        assert!(js.contains("style.display='none'"));
    }

    // ============ Form Operations ============

    #[test]
    fn test_set_value() {
        let mut batch = DomBatch::new();
        batch.set_value("#email", "test@example.com");
        let js = batch.to_js();
        assert!(js.contains(".value="));
        assert!(js.contains("test@example.com"));
    }

    #[test]
    fn test_set_checked_true() {
        let mut batch = DomBatch::new();
        batch.set_checked("#checkbox", true);
        let js = batch.to_js();
        assert!(js.contains(".checked=true"));
    }

    #[test]
    fn test_set_checked_false() {
        let mut batch = DomBatch::new();
        batch.set_checked("#checkbox", false);
        let js = batch.to_js();
        assert!(js.contains(".checked=false"));
    }

    #[test]
    fn test_set_disabled_true() {
        let mut batch = DomBatch::new();
        batch.set_disabled("#button", true);
        let js = batch.to_js();
        assert!(js.contains(".disabled=true"));
    }

    #[test]
    fn test_set_disabled_false() {
        let mut batch = DomBatch::new();
        batch.set_disabled("#button", false);
        let js = batch.to_js();
        assert!(js.contains(".disabled=false"));
    }

    #[test]
    fn test_clear_input() {
        let mut batch = DomBatch::new();
        batch.clear_input("#search");
        let js = batch.to_js();
        assert!(js.contains("value=''"));
        assert!(js.contains("dispatchEvent"));
    }

    #[test]
    fn test_submit() {
        let mut batch = DomBatch::new();
        batch.submit("#form");
        let js = batch.to_js();
        assert!(js.contains("submit()"));
        assert!(js.contains("closest('form')"));
    }

    // ============ Event Operations ============

    #[test]
    fn test_click() {
        let mut batch = DomBatch::new();
        batch.click("#button");
        let js = batch.to_js();
        assert!(js.contains(".click()"));
    }

    #[test]
    fn test_double_click() {
        let mut batch = DomBatch::new();
        batch.double_click("#item");
        let js = batch.to_js();
        assert!(js.contains("dblclick"));
        assert!(js.contains("dispatchEvent"));
    }

    #[test]
    fn test_focus() {
        let mut batch = DomBatch::new();
        batch.focus("#input");
        let js = batch.to_js();
        assert!(js.contains(".focus()"));
    }

    #[test]
    fn test_blur() {
        let mut batch = DomBatch::new();
        batch.blur("#input");
        let js = batch.to_js();
        assert!(js.contains(".blur()"));
    }

    // ============ Scroll Operations ============

    #[test]
    fn test_scroll_into_view_smooth() {
        let mut batch = DomBatch::new();
        batch.scroll_into_view("#section", true);
        let js = batch.to_js();
        assert!(js.contains("scrollIntoView"));
        assert!(js.contains("behavior:'smooth'"));
    }

    #[test]
    fn test_scroll_into_view_auto() {
        let mut batch = DomBatch::new();
        batch.scroll_into_view("#section", false);
        let js = batch.to_js();
        assert!(js.contains("behavior:'auto'"));
    }

    // ============ Type Text Operations ============

    #[test]
    fn test_type_text_without_clear() {
        let mut batch = DomBatch::new();
        batch.type_text("#input", "hello", false);
        let js = batch.to_js();
        assert!(js.contains("split('')"));
        assert!(js.contains("hello"));
        // Should NOT clear first
        assert!(!js.contains("e.value='';\""));
    }

    #[test]
    fn test_type_text_with_clear() {
        let mut batch = DomBatch::new();
        batch.type_text("#input", "world", true);
        let js = batch.to_js();
        // Should clear first
        assert!(js.contains("e.value='';"));
    }

    // ============ HTML Manipulation ============

    #[test]
    fn test_append_html() {
        let mut batch = DomBatch::new();
        batch.append_html("#list", "<li>New item</li>");
        let js = batch.to_js();
        assert!(js.contains("insertAdjacentHTML"));
        assert!(js.contains("beforeend"));
        assert!(js.contains("<li>New item</li>"));
    }

    #[test]
    fn test_prepend_html() {
        let mut batch = DomBatch::new();
        batch.prepend_html("#list", "<li>First item</li>");
        let js = batch.to_js();
        assert!(js.contains("insertAdjacentHTML"));
        assert!(js.contains("afterbegin"));
    }

    #[test]
    fn test_remove() {
        let mut batch = DomBatch::new();
        batch.remove("#old-element");
        let js = batch.to_js();
        assert!(js.contains(".remove()"));
    }

    #[test]
    fn test_empty() {
        let mut batch = DomBatch::new();
        batch.empty("#container");
        let js = batch.to_js();
        assert!(js.contains("innerHTML=''"));
    }

    // ============ Raw JS Operations ============

    #[test]
    fn test_raw() {
        let mut batch = DomBatch::new();
        batch.raw("#element", "e.dataset.custom = 'value'");
        let js = batch.to_js();
        assert!(js.contains("querySelector('#element')"));
        assert!(js.contains("e.dataset.custom = 'value'"));
    }

    #[test]
    fn test_raw_global() {
        let mut batch = DomBatch::new();
        batch.raw_global("console.log('Hello')");
        let js = batch.to_js();
        assert!(js.contains("console.log('Hello')"));
    }

    // ============ Multiple Operations ============

    #[test]
    fn test_multiple_ops() {
        let mut batch = DomBatch::new();
        batch.set_text("#title", "Hello");
        batch.add_class(".item", "active");
        batch.click("#btn");
        let js = batch.to_js();
        assert!(js.contains("#title"));
        assert!(js.contains(".item"));
        assert!(js.contains("#btn"));
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_chaining() {
        let mut batch = DomBatch::new();
        batch
            .set_text("#a", "1")
            .set_text("#b", "2")
            .set_text("#c", "3");
        assert_eq!(batch.len(), 3);
    }

    // ============ Escaping Tests ============

    #[test]
    fn test_escapes_double_quotes() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "Hello \"World\"");
        let js = batch.to_js();
        assert!(js.contains("\\\""));
    }

    #[test]
    fn test_escapes_backslashes() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "C:\\path\\to\\file");
        let js = batch.to_js();
        assert!(js.contains("\\\\"));
    }

    #[test]
    fn test_escapes_newlines() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "Line1\nLine2");
        let js = batch.to_js();
        assert!(js.contains("\\n"));
    }

    #[test]
    fn test_escapes_carriage_returns() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "Line1\rLine2");
        let js = batch.to_js();
        assert!(js.contains("\\r"));
    }

    #[test]
    fn test_escapes_tabs() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "Col1\tCol2");
        let js = batch.to_js();
        assert!(js.contains("\\t"));
    }

    #[test]
    fn test_escapes_single_quotes_in_selector() {
        let mut batch = DomBatch::new();
        batch.click("[data-name='test']");
        let js = batch.to_js();
        assert!(js.contains("[data-name=\\'test\\']"));
    }

    // ============ Edge Cases ============

    #[test]
    fn test_empty_text() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "");
        let js = batch.to_js();
        assert!(js.contains("textContent=\"\""));
    }

    #[test]
    fn test_empty_value() {
        let mut batch = DomBatch::new();
        batch.set_value("#input", "");
        let js = batch.to_js();
        assert!(js.contains(".value=\"\""));
    }

    #[test]
    fn test_unicode_content() {
        let mut batch = DomBatch::new();
        batch.set_text("#test", "你好世界 🚀");
        let js = batch.to_js();
        assert!(js.contains("你好世界 🚀"));
    }

    #[test]
    fn test_html_entities_in_content() {
        let mut batch = DomBatch::new();
        batch.set_html("#test", "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
        let js = batch.to_js();
        assert!(js.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_complex_selector() {
        let mut batch = DomBatch::new();
        batch.click("div.container > ul.list li:nth-child(2)");
        let js = batch.to_js();
        assert!(js.contains("div.container > ul.list li:nth-child(2)"));
    }

    #[test]
    fn test_attribute_selector() {
        let mut batch = DomBatch::new();
        batch.click("[data-testid=\"submit-btn\"]");
        let js = batch.to_js();
        // Double quotes are preserved in selectors (only single quotes are escaped)
        assert!(js.contains("[data-testid=\"submit-btn\"]"));
    }

    // ============ Default Trait ============

    #[test]
    fn test_default_trait() {
        let batch: DomBatch = Default::default();
        assert!(batch.is_empty());
    }

    // ============ DomOp Enum Tests ============

    #[test]
    fn test_dom_op_set_styles() {
        let mut batch = DomBatch::new();
        batch.push(DomOp::SetStyles {
            selector: "#box".to_string(),
            styles: vec![
                ("color".to_string(), "red".to_string()),
                ("font-size".to_string(), "16px".to_string()),
            ],
        });
        let js = batch.to_js();
        assert!(js.contains("style['color']=\"red\""));
        assert!(js.contains("style['font-size']=\"16px\""));
    }

    #[test]
    fn test_dom_op_select_option() {
        let mut batch = DomBatch::new();
        batch.push(DomOp::SelectOption {
            selector: "#dropdown".to_string(),
            value: "option2".to_string(),
        });
        let js = batch.to_js();
        assert!(js.contains(".value="));
        assert!(js.contains("option2"));
    }

    // ============ Performance/Stress Tests ============

    #[test]
    fn test_large_batch() {
        let mut batch = DomBatch::new();
        for i in 0..100 {
            batch.set_text(&format!("#item-{}", i), &format!("Value {}", i));
        }
        assert_eq!(batch.len(), 100);
        let js = batch.to_js();
        assert!(js.contains("#item-0"));
        assert!(js.contains("#item-99"));
    }

    #[test]
    fn test_very_long_content() {
        let mut batch = DomBatch::new();
        let long_text = "x".repeat(10000);
        batch.set_text("#test", &long_text);
        let js = batch.to_js();
        assert!(js.len() > 10000);
    }
}
