//! AuroraView Core - API Registration Methods
//!
//! This module provides high-performance API registration methods that expose
//! Rust-generated JavaScript to Python, avoiding the overhead of Python-based
//! JavaScript generation and eval_js fallbacks.
//!
//! Performance benefits:
//! - Rust-side template rendering is faster than Python string manipulation
//! - Generated JavaScript is optimized and type-safe via Askama templates
//! - Reduces Python/JS boundary crossing overhead

use askama::Template;
use auroraview_core::templates::{ApiMethodEntry, ApiRegistrationTemplate};
use pyo3::prelude::*;

use super::AuroraView;
use crate::ipc::WebViewMessage;

#[pymethods]
impl AuroraView {
    /// Register API methods in Rust for high-performance JavaScript injection.
    ///
    /// This method generates optimized JavaScript code using Rust's Askama template
    /// engine and queues it for execution. It's faster than the Python/JS fallback
    /// because:
    /// 1. Template rendering happens in Rust (compiled, no GIL)
    /// 2. No Python string manipulation overhead
    /// 3. Generated code is validated at compile time
    ///
    /// Args:
    ///     namespace (str): API namespace (e.g., "api", "my_service")
    ///     methods (list[str]): List of method names to register
    ///
    /// Example:
    ///     >>> webview.register_api_methods("api", ["echo", "get_data", "set_config"])
    ///     # This generates and executes:
    ///     # window.auroraview._registerApiMethods('api', ['echo', 'get_data', 'set_config']);
    ///
    /// Returns:
    ///     bool: True if registration was queued successfully
    #[pyo3(signature = (namespace, methods))]
    fn register_api_methods(&self, namespace: &str, methods: Vec<String>) -> PyResult<bool> {
        if methods.is_empty() {
            tracing::debug!(
                "register_api_methods called with empty methods list for namespace '{}'",
                namespace
            );
            return Ok(true);
        }

        tracing::info!(
            "Registering {} API methods for namespace '{}' via Rust template",
            methods.len(),
            namespace
        );

        // Use auroraview-core's ApiRegistrationTemplate for type-safe JS generation
        let entry = ApiMethodEntry {
            namespace: namespace.to_string(),
            methods,
        };

        let template = ApiRegistrationTemplate {
            api_methods: vec![entry],
        };

        match template.render() {
            Ok(script) => {
                tracing::debug!("Generated API registration script ({} bytes)", script.len());
                self.message_queue.push(WebViewMessage::EvalJs(script));
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Failed to render API registration template: {}", e);
                Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to generate API registration script: {}",
                    e
                )))
            }
        }
    }

    /// Register multiple API namespaces at once (batch registration).
    ///
    /// More efficient than calling register_api_methods multiple times when
    /// registering APIs from multiple objects/namespaces.
    ///
    /// Args:
    ///     registrations (list[tuple[str, list[str]]]): List of (namespace, methods) tuples
    ///
    /// Example:
    ///     >>> webview.register_api_methods_batch([
    ///     ...     ("api", ["echo", "get_data"]),
    ///     ...     ("service", ["connect", "disconnect"]),
    ///     ... ])
    #[pyo3(signature = (registrations))]
    fn register_api_methods_batch(
        &self,
        registrations: Vec<(String, Vec<String>)>,
    ) -> PyResult<bool> {
        if registrations.is_empty() {
            return Ok(true);
        }

        let entries: Vec<ApiMethodEntry> = registrations
            .into_iter()
            .filter(|(_, methods)| !methods.is_empty())
            .map(|(namespace, methods)| ApiMethodEntry { namespace, methods })
            .collect();

        if entries.is_empty() {
            return Ok(true);
        }

        let total_methods: usize = entries.iter().map(|e| e.methods.len()).sum();
        tracing::info!(
            "Batch registering {} methods across {} namespaces via Rust template",
            total_methods,
            entries.len()
        );

        let template = ApiRegistrationTemplate {
            api_methods: entries,
        };

        match template.render() {
            Ok(script) => {
                tracing::debug!(
                    "Generated batch API registration script ({} bytes)",
                    script.len()
                );
                self.message_queue.push(WebViewMessage::EvalJs(script));
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Failed to render batch API registration template: {}", e);
                Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to generate batch API registration script: {}",
                    e
                )))
            }
        }
    }
}
