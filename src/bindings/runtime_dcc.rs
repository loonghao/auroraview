//! Python bindings for auroraview-dcc runtime
//!
//! This module provides Python bindings for the DCC runtime crate,
//! enabling WebView integration in Maya, Houdini, Nuke, Blender, etc.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

/// Python wrapper for DccType enum
#[pyclass(name = "DccType")]
#[derive(Clone, Copy)]
pub struct PyDccType {
    inner: auroraview_dcc::DccType,
}

#[pymethods]
impl PyDccType {
    #[staticmethod]
    fn maya() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Maya,
        }
    }

    #[staticmethod]
    fn houdini() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Houdini,
        }
    }

    #[staticmethod]
    fn nuke() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Nuke,
        }
    }

    #[staticmethod]
    fn blender() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Blender,
        }
    }

    #[staticmethod]
    fn max3ds() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Max3ds,
        }
    }

    #[staticmethod]
    fn unreal() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Unreal,
        }
    }

    #[staticmethod]
    fn unknown() -> Self {
        Self {
            inner: auroraview_dcc::DccType::Unknown,
        }
    }

    /// Auto-detect DCC type from environment
    #[staticmethod]
    fn detect() -> Self {
        Self {
            inner: auroraview_dcc::DccType::detect(),
        }
    }

    /// Get DCC name
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("DccType.{}", self.inner.name().to_lowercase())
    }

    fn __str__(&self) -> &'static str {
        self.inner.name()
    }
}

/// Python wrapper for DccConfig
#[pyclass(name = "DccConfig")]
#[derive(Clone)]
pub struct PyDccConfig {
    inner: auroraview_dcc::DccConfig,
}

#[pymethods]
impl PyDccConfig {
    #[new]
    #[pyo3(signature = (
        title = "AuroraView",
        width = 400,
        height = 600,
        url = None,
        html = None,
        parent_hwnd = None,
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
        parent_hwnd: Option<isize>,
        devtools: bool,
        debug_port: u16,
    ) -> Self {
        Self {
            inner: auroraview_dcc::DccConfig {
                title: title.to_string(),
                width,
                height,
                url,
                html,
                parent_hwnd,
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

    /// Builder-style: set parent HWND (from Qt widget.winId())
    fn parent_hwnd(&mut self, hwnd: isize) -> Self {
        self.inner.parent_hwnd = Some(hwnd);
        self.clone()
    }

    /// Builder-style: set DCC type
    fn dcc_type(&mut self, dcc: &PyDccType) -> Self {
        self.inner.dcc_type = dcc.inner;
        self.clone()
    }

    /// Builder-style: set panel name for dock registration
    fn panel_name(&mut self, name: &str) -> Self {
        self.inner.panel_name = Some(name.to_string());
        self.clone()
    }

    /// Builder-style: set devtools
    fn devtools(&mut self, enable: bool) -> Self {
        self.inner.devtools = enable;
        self.clone()
    }

    /// Builder-style: set debug port
    fn debug_port(&mut self, port: u16) -> Self {
        self.inner.debug_port = port;
        self.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "DccConfig(title='{}', size={}x{}, url={:?}, parent_hwnd={:?})",
            self.inner.title,
            self.inner.width,
            self.inner.height,
            self.inner.url,
            self.inner.parent_hwnd
        )
    }
}

impl PyDccConfig {
    #[allow(dead_code)]
    pub fn into_inner(self) -> auroraview_dcc::DccConfig {
        self.inner
    }
}

/// Python wrapper for IpcRouter (DCC version)
#[pyclass(name = "DccIpcRouter")]
pub struct PyDccIpcRouter {
    inner: Arc<auroraview_dcc::IpcRouter>,
}

#[pymethods]
impl PyDccIpcRouter {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(auroraview_dcc::IpcRouter::new()),
        }
    }

    /// Register a handler for a method
    fn register(&self, method: &str, handler: Py<PyAny>) {
        let handler = handler;
        self.inner.register(method, move |params| {
            Python::attach(|py| {
                let py_params = pythonize::pythonize(py, &params)
                    .map(|v| v.unbind())
                    .unwrap_or_else(|_| py.None().into());

                match handler.call1(py, (py_params,)) {
                    Ok(result) => pythonize::depythonize(&result.bind(py))
                        .unwrap_or(serde_json::Value::Null),
                    Err(e) => {
                        tracing::error!("[DccIpcRouter] Python handler error: {}", e);
                        serde_json::json!({ "error": e.to_string() })
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
                    tracing::error!("[DccIpcRouter] Event handler error: {}", e);
                }
            });
        });
    }

    /// Check if a handler exists
    fn has_handler(&self, method: &str) -> bool {
        self.inner.has_handler(method)
    }

    /// Unregister a handler
    fn unregister(&self, method: &str) -> bool {
        self.inner.unregister(method)
    }

    /// Get all registered methods
    fn methods(&self) -> Vec<String> {
        self.inner.methods()
    }

    /// Handle a raw IPC message
    fn handle(&self, raw: &str) -> Option<String> {
        self.inner.handle(raw)
    }
}

impl PyDccIpcRouter {
    #[allow(dead_code)]
    pub fn inner(&self) -> Arc<auroraview_dcc::IpcRouter> {
        Arc::clone(&self.inner)
    }
}

/// Python wrapper for DCC WindowManager
#[pyclass(name = "DccWindowManager")]
pub struct PyDccWindowManager {
    inner: Arc<auroraview_dcc::WindowManager>,
}

#[pymethods]
impl PyDccWindowManager {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(auroraview_dcc::WindowManager::new()),
        }
    }

    /// Get the shared IPC router
    fn router(&self) -> PyDccIpcRouter {
        PyDccIpcRouter {
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

    /// Process events for all windows (call from Qt timer)
    ///
    /// This should be called periodically from the DCC's event loop,
    /// typically via a QTimer with 16ms interval.
    fn process_events(&self) {
        self.inner.process_events();
    }
}

/// Register DCC runtime functions with Python module
pub fn register_dcc_runtime(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDccType>()?;
    m.add_class::<PyDccConfig>()?;
    m.add_class::<PyDccIpcRouter>()?;
    m.add_class::<PyDccWindowManager>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcc_type_detection() {
        let dcc = PyDccType::detect();
        // Without DCC environment, should be Unknown
        assert_eq!(dcc.inner.name(), "Unknown");
    }

    #[test]
    fn test_dcc_type_constructors() {
        assert_eq!(PyDccType::maya().inner.name(), "Maya");
        assert_eq!(PyDccType::houdini().inner.name(), "Houdini");
        assert_eq!(PyDccType::nuke().inner.name(), "Nuke");
        assert_eq!(PyDccType::blender().inner.name(), "Blender");
        assert_eq!(PyDccType::max3ds().inner.name(), "3ds Max");
        assert_eq!(PyDccType::unreal().inner.name(), "Unreal Engine");
    }

    #[test]
    fn test_dcc_config_creation() {
        let config = PyDccConfig::new(
            "Test Panel",
            400,
            600,
            Some("https://example.com".to_string()),
            None,
            Some(12345),
            true,
            0,
        );

        assert_eq!(config.inner.title, "Test Panel");
        assert_eq!(config.inner.width, 400);
        assert_eq!(config.inner.height, 600);
        assert_eq!(config.inner.parent_hwnd, Some(12345));
    }

    #[test]
    fn test_dcc_config_builder_style() {
        let mut config = PyDccConfig::new("Default", 100, 100, None, None, None, true, 0);

        config = config.title("New Panel");
        config = config.size(800, 600);
        config = config.parent_hwnd(67890);
        config = config.panel_name("my_tool_panel");

        assert_eq!(config.inner.title, "New Panel");
        assert_eq!(config.inner.width, 800);
        assert_eq!(config.inner.height, 600);
        assert_eq!(config.inner.parent_hwnd, Some(67890));
        assert_eq!(config.inner.panel_name, Some("my_tool_panel".to_string()));
    }

    #[test]
    fn test_dcc_ipc_router_creation() {
        let router = PyDccIpcRouter::new();
        assert!(router.methods().is_empty());
        assert!(!router.has_handler("test"));
    }

    #[test]
    fn test_dcc_window_manager_creation() {
        let manager = PyDccWindowManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.window_ids().is_empty());
    }
}
