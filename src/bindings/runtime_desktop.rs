//! Python bindings for auroraview-desktop runtime
//!
//! This module provides Python bindings for the desktop runtime crate,
//! enabling standalone desktop applications with multi-window support.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;
use std::sync::Arc;

/// Python wrapper for DesktopConfig
#[pyclass(name = "DesktopConfig")]
#[derive(Clone)]
pub struct PyDesktopConfig {
    inner: auroraview_desktop::DesktopConfig,
}

#[pymethods]
impl PyDesktopConfig {
    #[new]
    #[pyo3(signature = (
        title = "AuroraView",
        width = 1024,
        height = 768,
        url = None,
        html = None,
        resizable = true,
        decorations = true,
        always_on_top = false,
        transparent = false,
        devtools = true,
        debug_port = 0
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        title: &str,
        width: u32,
        height: u32,
        url: Option<String>,
        html: Option<String>,
        resizable: bool,
        decorations: bool,
        always_on_top: bool,
        transparent: bool,
        devtools: bool,
        debug_port: u16,
    ) -> Self {
        Self {
            inner: auroraview_desktop::DesktopConfig {
                title: title.to_string(),
                width,
                height,
                url,
                html,
                resizable,
                decorations,
                always_on_top,
                transparent,
                devtools,
                debug_port,
                ..Default::default()
            },
        }
    }

    /// Builder-style: set title
    fn title(&mut self, title: &str) -> Self {
        self.inner.title = title.to_string();
        self.clone()
    }

    /// Builder-style: set size
    fn size(&mut self, width: u32, height: u32) -> Self {
        self.inner.width = width;
        self.inner.height = height;
        self.clone()
    }

    /// Builder-style: set URL
    fn url(&mut self, url: &str) -> Self {
        self.inner.url = Some(url.to_string());
        self.clone()
    }

    /// Builder-style: set HTML
    fn html(&mut self, html: &str) -> Self {
        self.inner.html = Some(html.to_string());
        self.clone()
    }

    /// Builder-style: set devtools
    fn devtools(&mut self, enable: bool) -> Self {
        self.inner.devtools = enable;
        self.clone()
    }

    /// Builder-style: set resizable
    fn resizable(&mut self, enable: bool) -> Self {
        self.inner.resizable = enable;
        self.clone()
    }

    /// Builder-style: set decorations
    fn decorations(&mut self, enable: bool) -> Self {
        self.inner.decorations = enable;
        self.clone()
    }

    /// Builder-style: set always_on_top
    fn always_on_top(&mut self, enable: bool) -> Self {
        self.inner.always_on_top = enable;
        self.clone()
    }

    /// Builder-style: set transparent
    fn transparent(&mut self, enable: bool) -> Self {
        self.inner.transparent = enable;
        self.clone()
    }

    /// Builder-style: set data directory
    fn data_dir(&mut self, path: &str) -> Self {
        self.inner.data_dir = Some(PathBuf::from(path));
        self.clone()
    }

    /// Builder-style: set icon
    fn icon(&mut self, path: &str) -> Self {
        self.inner.icon = Some(PathBuf::from(path));
        self.clone()
    }

    /// Builder-style: set debug port
    fn debug_port(&mut self, port: u16) -> Self {
        self.inner.debug_port = port;
        self.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "DesktopConfig(title='{}', size={}x{}, url={:?})",
            self.inner.title, self.inner.width, self.inner.height, self.inner.url
        )
    }
}

impl PyDesktopConfig {
    pub fn into_inner(self) -> auroraview_desktop::DesktopConfig {
        self.inner
    }
}

/// Python wrapper for TrayConfig
#[pyclass(name = "TrayConfig")]
#[derive(Clone)]
pub struct PyTrayConfig {
    inner: auroraview_desktop::TrayConfig,
}

#[pymethods]
impl PyTrayConfig {
    #[new]
    #[pyo3(signature = (tooltip = None, icon = None))]
    fn new(tooltip: Option<String>, icon: Option<String>) -> Self {
        Self {
            inner: auroraview_desktop::TrayConfig {
                tooltip,
                icon: icon.map(PathBuf::from),
                ..Default::default()
            },
        }
    }

    /// Add a menu item
    fn add_item(&mut self, id: &str, label: &str) -> Self {
        self.inner
            .menu
            .push(auroraview_desktop::TrayMenuItem::Item {
                id: id.to_string(),
                label: label.to_string(),
                enabled: true,
            });
        self.clone()
    }

    /// Add a separator
    fn add_separator(&mut self) -> Self {
        self.inner
            .menu
            .push(auroraview_desktop::TrayMenuItem::Separator);
        self.clone()
    }
}

/// Python wrapper for IpcRouter
#[pyclass(name = "DesktopIpcRouter")]
pub struct PyDesktopIpcRouter {
    inner: Arc<auroraview_desktop::IpcRouter>,
}

#[pymethods]
impl PyDesktopIpcRouter {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(auroraview_desktop::IpcRouter::new()),
        }
    }

    /// Register a handler for a method
    ///
    /// The handler receives a JSON value and returns a JSON value.
    fn register(&self, method: &str, handler: Py<PyAny>) {
        let handler = handler;
        self.inner.register(method, move |params| {
            Python::attach(|py| {
                // Convert params to Python
                let py_params = pythonize::pythonize(py, &params)
                    .map(|v| v.unbind())
                    .unwrap_or_else(|_| py.None().into());

                // Call Python handler
                match handler.call1(py, (py_params,)) {
                    Ok(result) => {
                        // Convert result back to JSON
                        pythonize::depythonize(&result.bind(py)).unwrap_or(serde_json::Value::Null)
                    }
                    Err(e) => {
                        tracing::error!("[IpcRouter] Python handler error: {}", e);
                        serde_json::json!({
                            "error": e.to_string()
                        })
                    }
                }
            })
        });
    }

    /// Subscribe to an event
    fn on(&self, event: &str, handler: Py<PyAny>) {
        let handler = handler;
        self.inner.on(event, move |data| {
            Python::attach(|py| {
                let py_data = pythonize::pythonize(py, &data)
                    .map(|v| v.unbind())
                    .unwrap_or_else(|_| py.None().into());
                if let Err(e) = handler.call1(py, (py_data,)) {
                    tracing::error!("[IpcRouter] Event handler error: {}", e);
                }
            });
        });
    }

    /// Check if a handler exists for a method
    fn has_handler(&self, method: &str) -> bool {
        self.inner.has_handler(method)
    }

    /// Unregister a handler
    fn unregister(&self, method: &str) -> bool {
        self.inner.unregister(method)
    }

    /// Get all registered method names
    fn methods(&self) -> Vec<String> {
        self.inner.methods()
    }

    /// Handle a raw IPC message
    fn handle(&self, raw: &str) -> Option<String> {
        self.inner.handle(raw)
    }
}

impl PyDesktopIpcRouter {
    pub fn inner(&self) -> Arc<auroraview_desktop::IpcRouter> {
        Arc::clone(&self.inner)
    }
}

/// Run a desktop application (blocking)
///
/// This function creates and runs a desktop WebView window.
/// It blocks until the window is closed.
///
/// Args:
///     config: DesktopConfig instance
///     router: Optional IpcRouter for handling JS calls
///
/// Example:
///     >>> from auroraview._core import DesktopConfig, run_desktop_app
///     >>> config = DesktopConfig(title="My App", url="https://example.com")
///     >>> run_desktop_app(config)
#[pyfunction]
#[pyo3(signature = (config, router = None))]
fn run_desktop_app(config: PyDesktopConfig, router: Option<&PyDesktopIpcRouter>) -> PyResult<()> {
    let rust_config = config.into_inner();
    let rust_router = router.map(|r| r.inner());

    match rust_router {
        Some(r) => auroraview_desktop::run_with_router(rust_config, Some(r)),
        None => auroraview_desktop::run(rust_config),
    }
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Python wrapper for WindowManager
#[pyclass(name = "DesktopWindowManager")]
pub struct PyDesktopWindowManager {
    inner: Arc<auroraview_desktop::WindowManager>,
}

#[pymethods]
impl PyDesktopWindowManager {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(auroraview_desktop::WindowManager::new()),
        }
    }

    /// Get the shared IPC router
    fn router(&self) -> PyDesktopIpcRouter {
        PyDesktopIpcRouter {
            inner: self.inner.router(),
        }
    }

    /// Get window count
    fn count(&self) -> usize {
        self.inner.count()
    }

    /// Get all window IDs
    fn window_ids(&self) -> Vec<String> {
        self.inner.window_ids()
    }

    /// Check if a window exists
    fn has_window(&self, id: &str) -> bool {
        self.inner.has_window(id)
    }

    /// Get window info
    fn get_info(&self, id: &str) -> Option<Py<PyAny>> {
        Python::attach(|py| {
            self.inner.get_info(id).map(|info| {
                let dict = PyDict::new(py);
                dict.set_item("id", &info.id).ok();
                dict.set_item("title", &info.title).ok();
                dict.set_item("visible", info.visible).ok();
                dict.unbind().into_any()
            })
        })
    }
}

/// Register desktop runtime functions with Python module
pub fn register_desktop_runtime(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDesktopConfig>()?;
    m.add_class::<PyTrayConfig>()?;
    m.add_class::<PyDesktopIpcRouter>()?;
    m.add_class::<PyDesktopWindowManager>()?;
    m.add_function(wrap_pyfunction!(run_desktop_app, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_config_creation() {
        let config = PyDesktopConfig::new(
            "Test Window",
            800,
            600,
            Some("https://example.com".to_string()),
            None,
            true,
            true,
            false,
            false,
            true,
            0,
        );

        assert_eq!(config.inner.title, "Test Window");
        assert_eq!(config.inner.width, 800);
        assert_eq!(config.inner.height, 600);
        assert_eq!(config.inner.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_desktop_config_builder_style() {
        let mut config = PyDesktopConfig::new(
            "Default", 100, 100, None, None, true, true, false, false, true, 0,
        );

        config = config.title("New Title");
        config = config.size(1920, 1080);
        config = config.url("https://google.com");

        assert_eq!(config.inner.title, "New Title");
        assert_eq!(config.inner.width, 1920);
        assert_eq!(config.inner.height, 1080);
        assert_eq!(config.inner.url, Some("https://google.com".to_string()));
    }

    #[test]
    fn test_ipc_router_creation() {
        let router = PyDesktopIpcRouter::new();
        assert!(router.methods().is_empty());
        assert!(!router.has_handler("test"));
    }

    #[test]
    fn test_window_manager_creation() {
        let manager = PyDesktopWindowManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.window_ids().is_empty());
    }
}
