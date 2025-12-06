//! AuroraView Core - Storage and Cookie Management
//!
//! This module contains storage-related methods:
//! - localStorage API
//! - sessionStorage API
//! - Cookie management

use pyo3::prelude::*;

use super::AuroraView;
use crate::ipc::WebViewMessage;

#[pymethods]
impl AuroraView {
    // ========================================
    // localStorage API
    // ========================================

    /// Set a value in localStorage
    ///
    /// Args:
    ///     key (str): Storage key
    ///     value (str): Value to store
    fn set_local_storage(&self, key: &str, value: &str) -> PyResult<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_value = value.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            "localStorage.setItem('{}', '{}')",
            escaped_key, escaped_value
        );
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Get a value from localStorage asynchronously
    ///
    /// Args:
    ///     key (str): Storage key
    ///     callback (callable): Python function(value, error) to call with result
    fn get_local_storage(&self, key: &str, callback: Py<PyAny>) -> PyResult<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!("localStorage.getItem('{}')", escaped_key);
        self.eval_js_async_internal(&script, callback, 5000)
    }

    /// Remove a value from localStorage
    fn remove_local_storage(&self, key: &str) -> PyResult<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!("localStorage.removeItem('{}')", escaped_key);
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Clear all localStorage data
    fn clear_local_storage(&self) -> PyResult<()> {
        self.message_queue
            .push(WebViewMessage::EvalJs("localStorage.clear()".to_string()));
        Ok(())
    }

    // ========================================
    // sessionStorage API
    // ========================================

    /// Set a value in sessionStorage
    fn set_session_storage(&self, key: &str, value: &str) -> PyResult<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_value = value.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            "sessionStorage.setItem('{}', '{}')",
            escaped_key, escaped_value
        );
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Get a value from sessionStorage asynchronously
    fn get_session_storage(&self, key: &str, callback: Py<PyAny>) -> PyResult<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!("sessionStorage.getItem('{}')", escaped_key);
        self.eval_js_async_internal(&script, callback, 5000)
    }

    /// Remove a value from sessionStorage
    fn remove_session_storage(&self, key: &str) -> PyResult<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!("sessionStorage.removeItem('{}')", escaped_key);
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Clear all sessionStorage data
    fn clear_session_storage(&self) -> PyResult<()> {
        self.message_queue
            .push(WebViewMessage::EvalJs("sessionStorage.clear()".to_string()));
        Ok(())
    }

    // ========================================
    // Cookie Management API
    // ========================================

    /// Set a cookie
    ///
    /// Args:
    ///     name (str): Cookie name
    ///     value (str): Cookie value
    ///     expires_days (int, optional): Number of days until expiration
    ///     path (str, optional): Cookie path (default: "/")
    ///     secure (bool, optional): Secure flag (default: False)
    ///     same_site (str, optional): SameSite attribute ("Strict", "Lax", "None")
    #[pyo3(signature = (name, value, expires_days=None, path=None, secure=false, same_site=None))]
    fn set_cookie(
        &self,
        name: &str,
        value: &str,
        expires_days: Option<i32>,
        path: Option<&str>,
        secure: bool,
        same_site: Option<&str>,
    ) -> PyResult<()> {
        let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_value = value.replace('\\', "\\\\").replace('\'', "\\'");

        let mut cookie_parts = vec![format!("{}={}", escaped_name, escaped_value)];

        if let Some(days) = expires_days {
            cookie_parts.push(format!(
                "expires=' + new Date(Date.now() + {} * 864e5).toUTCString() + '",
                days
            ));
        }

        cookie_parts.push(format!("path={}", path.unwrap_or("/")));

        if secure {
            cookie_parts.push("secure".to_string());
        }

        if let Some(ss) = same_site {
            cookie_parts.push(format!("SameSite={}", ss));
        }

        let script = format!("document.cookie = '{}'", cookie_parts.join("; "));
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Get a cookie value asynchronously
    fn get_cookie(&self, name: &str, callback: Py<PyAny>) -> PyResult<()> {
        let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            r#"(function() {{
                var name = '{}=';
                var cookies = document.cookie.split(';');
                for (var i = 0; i < cookies.length; i++) {{
                    var c = cookies[i].trim();
                    if (c.indexOf(name) === 0) {{
                        return c.substring(name.length);
                    }}
                }}
                return null;
            }})()"#,
            escaped_name
        );
        self.eval_js_async_internal(&script, callback, 5000)
    }

    /// Delete a cookie
    #[pyo3(signature = (name, path=None))]
    fn delete_cookie(&self, name: &str, path: Option<&str>) -> PyResult<()> {
        let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
        let cookie_path = path.unwrap_or("/");
        let script = format!(
            "document.cookie = '{}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path={}'",
            escaped_name, cookie_path
        );
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Clear all cookies (for current domain)
    fn clear_cookies(&self) -> PyResult<()> {
        let script = r#"
            document.cookie.split(';').forEach(function(c) {
                document.cookie = c.trim().split('=')[0] + '=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/';
            });
        "#;
        self.message_queue
            .push(WebViewMessage::EvalJs(script.to_string()));
        Ok(())
    }
}
