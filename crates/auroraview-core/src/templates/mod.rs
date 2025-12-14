//! JavaScript template structures for Askama
//!
//! This module defines type-safe templates for generating JavaScript code.
//! Templates are compiled at build time by Askama, providing:
//! - Compile-time template validation
//! - Type-safe variable binding
//! - Automatic HTML/JS escaping (disabled for JS output)

use askama::Template;

/// Template for emitting events to JavaScript
///
/// Generates JavaScript code that uses `window.auroraview.trigger()` to dispatch
/// events from Rust/Python to JavaScript listeners.
///
/// # Example
///
/// ```rust,ignore
/// use auroraview_core::templates::EmitEventTemplate;
/// use askama::Template;
///
/// let template = EmitEventTemplate {
///     event_name: "my_event",
///     event_data: r#"{"message": "hello"}"#,
/// };
/// let script = template.render().unwrap();
/// ```
#[derive(Template)]
#[template(path = "emit_event.js", escape = "none")]
pub struct EmitEventTemplate<'a> {
    /// Name of the event to trigger
    pub event_name: &'a str,
    /// JSON string of event data (must be properly escaped for JS)
    pub event_data: &'a str,
}

/// Template for loading a URL
///
/// Generates JavaScript code that navigates the WebView to a new URL.
///
/// # Example
///
/// ```rust,ignore
/// use auroraview_core::templates::LoadUrlTemplate;
/// use askama::Template;
///
/// let template = LoadUrlTemplate {
///     url: "https://example.com",
/// };
/// let script = template.render().unwrap();
/// ```
#[derive(Template)]
#[template(path = "load_url.js", escape = "none")]
pub struct LoadUrlTemplate<'a> {
    /// Target URL to navigate to
    pub url: &'a str,
}

/// Entry for API method registration
///
/// Used by `ApiRegistrationTemplate` to represent a namespace and its methods.
pub struct ApiMethodEntry {
    /// Namespace name (e.g., "test", "my_api")
    pub namespace: String,
    /// List of method names in this namespace
    pub methods: Vec<String>,
}

/// Template for API method registration
///
/// Generates JavaScript code that registers API methods using the
/// `window.auroraview._registerApiMethods()` helper function.
///
/// # Example
///
/// ```rust,ignore
/// use auroraview_core::templates::{ApiRegistrationTemplate, ApiMethodEntry};
/// use askama::Template;
///
/// let entries = vec![
///     ApiMethodEntry {
///         namespace: "test".to_string(),
///         methods: vec!["method1".to_string(), "method2".to_string()],
///     },
/// ];
/// let template = ApiRegistrationTemplate { api_methods: entries };
/// let script = template.render().unwrap();
/// ```
#[derive(Template)]
#[template(path = "api_registration.js", escape = "none")]
pub struct ApiRegistrationTemplate {
    /// List of namespace -> methods mappings
    pub api_methods: Vec<ApiMethodEntry>,
}
