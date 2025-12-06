//! AuroraView Core - DOM Operation Methods
//!
//! This module provides high-performance DOM operation methods that expose
//! Rust-generated JavaScript to Python, avoiding the overhead of Python-based
//! JavaScript generation and eval_js fallbacks.
//!
//! Performance benefits:
//! - Rust-side JS generation is faster than Python string manipulation
//! - Generated JavaScript is optimized via DomBatch
//! - Reduces Python/JS boundary crossing overhead
//! - Single method call can execute multiple DOM operations

use auroraview_core::dom::{DomBatch as CoreDomBatch, DomOp};
use pyo3::prelude::*;

use super::AuroraView;
use crate::dom::DomBatch;
use crate::ipc::WebViewMessage;

#[pymethods]
impl AuroraView {
    /// Execute a single DOM operation efficiently.
    ///
    /// This method generates optimized JavaScript code using Rust's DomBatch
    /// and queues it for execution. It's faster than the Python/JS fallback
    /// because the JS generation happens in Rust.
    ///
    /// Args:
    ///     selector (str): CSS selector for the target element
    ///     operation (str): Operation name (e.g., "click", "set_text", "add_class")
    ///     value (str, optional): Value for the operation (if applicable)
    ///     extra (str, optional): Extra parameter (for operations like set_attribute)
    ///
    /// Returns:
    ///     bool: True if operation was queued successfully
    ///
    /// Example:
    ///     >>> webview.dom_op("#btn", "click")
    ///     >>> webview.dom_op("#title", "set_text", "Hello World")
    ///     >>> webview.dom_op(".item", "add_class", "active")
    #[pyo3(signature = (selector, operation, value=None, extra=None))]
    fn dom_op(
        &self,
        selector: &str,
        operation: &str,
        value: Option<&str>,
        extra: Option<&str>,
    ) -> PyResult<bool> {
        let op = Self::parse_dom_op(selector, operation, value, extra)?;
        let js = CoreDomBatch::op_to_js(&op);

        tracing::debug!(
            "DOM operation '{}' on '{}' -> {} bytes JS",
            operation,
            selector,
            js.len()
        );

        self.message_queue
            .push(WebViewMessage::EvalJs(format!("(function(){{{}}})()", js)));
        Ok(true)
    }

    /// Execute a DomBatch directly from Python.
    ///
    /// This is more efficient than calling dom_op multiple times because
    /// all operations are batched into a single JavaScript execution.
    ///
    /// Args:
    ///     batch (DomBatch): A batch of DOM operations created in Python
    ///
    /// Returns:
    ///     bool: True if batch was queued successfully
    ///
    /// Example:
    ///     >>> from auroraview import DomBatch
    ///     >>> batch = DomBatch()
    ///     >>> batch.set_text("#title", "Hello")
    ///     >>> batch.add_class(".item", "active")
    ///     >>> batch.click("#submit")
    ///     >>> webview.execute_dom_batch(batch)
    #[pyo3(signature = (batch))]
    fn execute_dom_batch(&self, batch: &DomBatch) -> PyResult<bool> {
        let js = batch.to_js();

        if js == "(function(){})()" {
            tracing::debug!("Skipping empty DomBatch execution");
            return Ok(true);
        }

        tracing::info!("Executing DomBatch with {} bytes of JavaScript", js.len());

        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }

    /// Execute multiple DOM operations in a single call.
    ///
    /// This is a convenience method that creates a batch internally.
    /// Each operation is a tuple of (selector, operation, value, extra).
    ///
    /// Args:
    ///     operations: List of tuples (selector, operation, value?, extra?)
    ///
    /// Returns:
    ///     bool: True if all operations were queued successfully
    ///
    /// Example:
    ///     >>> webview.dom_ops_batch([
    ///     ...     ("#title", "set_text", "Hello", None),
    ///     ...     (".item", "add_class", "active", None),
    ///     ...     ("#submit", "click", None, None),
    ///     ... ])
    #[pyo3(signature = (operations))]
    fn dom_ops_batch(
        &self,
        operations: Vec<(String, String, Option<String>, Option<String>)>,
    ) -> PyResult<bool> {
        if operations.is_empty() {
            return Ok(true);
        }

        let mut batch = CoreDomBatch::with_capacity(operations.len());

        for (selector, operation, value, extra) in &operations {
            let op = Self::parse_dom_op(selector, operation, value.as_deref(), extra.as_deref())?;
            batch.push(op);
        }

        let js = batch.to_js();
        tracing::info!(
            "Executing {} DOM operations in batch ({} bytes JS)",
            operations.len(),
            js.len()
        );

        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }

    /// Simulate a drag operation on an element.
    ///
    /// This generates JavaScript that dispatches mousedown, mousemove,
    /// and mouseup events to simulate a drag gesture.
    ///
    /// Args:
    ///     selector (str): CSS selector for the element to drag
    ///     dx (i32): Horizontal offset in pixels
    ///     dy (i32): Vertical offset in pixels
    ///
    /// Returns:
    ///     bool: True if operation was queued successfully
    #[pyo3(signature = (selector, dx, dy))]
    fn simulate_drag(&self, selector: &str, dx: i32, dy: i32) -> PyResult<bool> {
        let escaped_selector = CoreDomBatch::escape_selector(selector);
        let js = format!(
            r#"(function(){{
                var e=document.querySelector('{}');
                if(!e)return;
                var r=e.getBoundingClientRect();
                var sx=r.left+r.width/2,sy=r.top+r.height/2;
                var ex=sx+{},ey=sy+{};
                e.dispatchEvent(new MouseEvent('mousedown',{{bubbles:true,clientX:sx,clientY:sy}}));
                document.dispatchEvent(new MouseEvent('mousemove',{{bubbles:true,clientX:ex,clientY:ey}}));
                document.dispatchEvent(new MouseEvent('mouseup',{{bubbles:true,clientX:ex,clientY:ey}}));
            }})()"#,
            escaped_selector, dx, dy
        );

        tracing::debug!("Simulating drag on '{}' by ({}, {})", selector, dx, dy);
        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }

    /// Check if an element exists in the DOM (queues check, does not return result).
    ///
    /// This queues a JavaScript check that stores the result in window._auroraview_last_check.
    /// For synchronous existence checking, use eval_js with a callback.
    ///
    /// Args:
    ///     selector (str): CSS selector for the element
    ///
    /// Returns:
    ///     bool: True if check was queued successfully
    #[pyo3(signature = (selector))]
    fn queue_element_exists_check(&self, selector: &str) -> PyResult<bool> {
        let escaped = CoreDomBatch::escape_selector(selector);
        let js = format!(
            "(function(){{window._auroraview_last_check={{selector:'{}',exists:document.querySelector('{}')!==null,timestamp:Date.now()}};}})()",
            escaped, escaped
        );

        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }

    /// Check if an element exists and emit result via IPC event.
    ///
    /// This is useful for testing frameworks that need to check element existence
    /// asynchronously. The result is emitted as a "_element_exists_result" event.
    ///
    /// Args:
    ///     selector (str): CSS selector for the element
    ///
    /// Returns:
    ///     bool: True if check was queued successfully
    #[pyo3(signature = (selector))]
    fn check_element_exists(&self, selector: &str) -> PyResult<bool> {
        let escaped = CoreDomBatch::escape_selector(selector);
        let js = format!(
            r#"(function(){{
                var exists=document.querySelector('{}')!==null;
                if(window.auroraview&&window.auroraview.emit){{
                    window.auroraview.emit('_element_exists_result',{{selector:'{}',exists:exists}});
                }}
            }})()"#,
            escaped, escaped
        );

        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }

    /// Get element text content and emit result via IPC event.
    ///
    /// The result is emitted as a "_element_text_result" event.
    ///
    /// Args:
    ///     selector (str): CSS selector for the element
    ///
    /// Returns:
    ///     bool: True if query was queued successfully
    #[pyo3(signature = (selector))]
    fn query_element_text(&self, selector: &str) -> PyResult<bool> {
        let escaped = CoreDomBatch::escape_selector(selector);
        let js = format!(
            r#"(function(){{
                var e=document.querySelector('{}');
                var text=e?e.textContent:'';
                if(window.auroraview&&window.auroraview.emit){{
                    window.auroraview.emit('_element_text_result',{{selector:'{}',text:text}});
                }}
            }})()"#,
            escaped, escaped
        );

        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }

    /// Get element attribute value and emit result via IPC event.
    ///
    /// The result is emitted as a "_element_attribute_result" event.
    ///
    /// Args:
    ///     selector (str): CSS selector for the element
    ///     attribute (str): Name of the attribute to get
    ///
    /// Returns:
    ///     bool: True if query was queued successfully
    #[pyo3(signature = (selector, attribute))]
    fn query_element_attribute(&self, selector: &str, attribute: &str) -> PyResult<bool> {
        let escaped_sel = CoreDomBatch::escape_selector(selector);
        let escaped_attr = CoreDomBatch::escape_string(attribute);
        let js = format!(
            r#"(function(){{
                var e=document.querySelector('{}');
                var value=e?e.getAttribute('{}'):null;
                if(window.auroraview&&window.auroraview.emit){{
                    window.auroraview.emit('_element_attribute_result',{{selector:'{}',attribute:'{}',value:value}});
                }}
            }})()"#,
            escaped_sel, escaped_attr, escaped_sel, escaped_attr
        );

        self.message_queue.push(WebViewMessage::EvalJs(js));
        Ok(true)
    }
}

// === Helper methods (not exposed to Python) ===
impl AuroraView {
    /// Parse a DOM operation from string parameters.
    fn parse_dom_op(
        selector: &str,
        operation: &str,
        value: Option<&str>,
        extra: Option<&str>,
    ) -> PyResult<DomOp> {
        let sel = selector.to_string();
        let val = value.unwrap_or("").to_string();
        let ext = extra.unwrap_or("").to_string();

        let op = match operation {
            // Text & Content
            "set_text" => DomOp::SetText {
                selector: sel,
                text: val,
            },
            "set_html" => DomOp::SetHtml {
                selector: sel,
                html: val,
            },

            // Attributes
            "set_attribute" => DomOp::SetAttribute {
                selector: sel,
                name: val,
                value: ext,
            },
            "remove_attribute" => DomOp::RemoveAttribute {
                selector: sel,
                name: val,
            },

            // Classes
            "add_class" => DomOp::AddClass {
                selector: sel,
                class: val,
            },
            "remove_class" => DomOp::RemoveClass {
                selector: sel,
                class: val,
            },
            "toggle_class" => DomOp::ToggleClass {
                selector: sel,
                class: val,
            },

            // Styles
            "set_style" => DomOp::SetStyle {
                selector: sel,
                property: val,
                value: ext,
            },

            // Visibility
            "show" => DomOp::Show { selector: sel },
            "hide" => DomOp::Hide { selector: sel },

            // Forms
            "set_value" => DomOp::SetValue {
                selector: sel,
                value: val,
            },
            "set_checked" => DomOp::SetChecked {
                selector: sel,
                checked: val.parse().unwrap_or(false),
            },
            "set_disabled" => DomOp::SetDisabled {
                selector: sel,
                disabled: val.parse().unwrap_or(false),
            },

            // Interactions
            "click" => DomOp::Click { selector: sel },
            "double_click" => DomOp::DoubleClick { selector: sel },
            "focus" => DomOp::Focus { selector: sel },
            "blur" => DomOp::Blur { selector: sel },
            "scroll_into_view" => DomOp::ScrollIntoView {
                selector: sel,
                smooth: val.parse().unwrap_or(true),
            },

            // Input
            "type_text" => DomOp::TypeText {
                selector: sel,
                text: val,
                clear: ext.parse().unwrap_or(false),
            },
            "clear" | "clear_input" => DomOp::Clear { selector: sel },
            "submit" => DomOp::Submit { selector: sel },

            // DOM Manipulation
            "append_html" => DomOp::AppendHtml {
                selector: sel,
                html: val,
            },
            "prepend_html" => DomOp::PrependHtml {
                selector: sel,
                html: val,
            },
            "remove" => DomOp::Remove { selector: sel },
            "empty" => DomOp::Empty { selector: sel },

            // Custom
            "raw" => DomOp::Raw {
                selector: sel,
                script: val,
            },

            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown DOM operation: '{}'. Valid operations: click, set_text, set_html, \
                     add_class, remove_class, toggle_class, set_style, show, hide, set_value, \
                     set_checked, set_disabled, focus, blur, scroll_into_view, type_text, \
                     clear, submit, append_html, prepend_html, remove, empty, set_attribute, \
                     remove_attribute, double_click, raw",
                    operation
                )))
            }
        };

        Ok(op)
    }
}
