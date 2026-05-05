// AuroraView Unreal Engine integration.
//
// Supports UE4/UE5 Python scripting, GameThread adapter,
// Slate UI integration, and GC-safe WebView embedding.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

use crossbeam_channel::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};

// --- Core Types ---

/// `GameThread` ID wrapper.
/// UE requires `Slate` UI operations to run on the `GameThread`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameThreadId(ThreadId);

impl std::fmt::Display for GameThreadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameThreadId({:?})", self.0)
    }
}

impl GameThreadId {
    /// Create from current thread.
    #[must_use]
    pub fn current() -> Self {
        Self(thread::current().id())
    }

    /// Check if the given thread is the `GameThread`.
    #[must_use]
    pub fn is_current(&self) -> bool {
        thread::current().id() == self.0
    }
}

/// `GameThread` task for deferred execution.
type GameThreadTask = Box<dyn FnOnce() + Send>;

/// Executor that ensures operations are run on UE's `GameThread`.
///
/// UE requires certain operations (like `Slate` UI updates) to be on the `GameThread`.
/// This executor provides a channel-based dispatch mechanism.
pub struct UeGameThreadExecutor {
    task_tx: Sender<GameThreadTask>,
    game_thread_id: GameThreadId,
}

impl UeGameThreadExecutor {
    /// Create a new executor with a channel to the `GameThread`.
    #[must_use]
    pub fn new() -> (Self, Receiver<GameThreadTask>) {
        let (task_tx, task_rx) = crossbeam_channel::unbounded();
        let executor = Self {
            task_tx,
            game_thread_id: GameThreadId::current(),
        };
        (executor, task_rx)
    }

    /// Check if the current thread is the `GameThread`.
    #[must_use]
    pub fn is_game_thread(&self) -> bool {
        self.game_thread_id.is_current()
    }

    /// Execute a closure on the `GameThread`.
    ///
    /// If already on `GameThread`, executes immediately.
    /// Otherwise, sends the task to the `GameThread` via channel (fire-and-forget).
    /// Use `execute_with_callback` if you need a result.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if self.is_game_thread() {
            f();
        } else {
            let task = Box::new(f);
            let _ = self.task_tx.send(task);
        }
    }

    /// Execute a closure on the `GameThread` and receive result via callback.
    ///
    /// The callback will be invoked on the `GameThread` after the task completes.
    pub fn execute_with_callback<F, C>(&self, task: F, callback: C)
    where
        F: FnOnce() -> Result<(), UeError> + Send + 'static,
        C: FnOnce(Result<(), UeError>) + Send + 'static,
    {
        if self.is_game_thread() {
            let result = task();
            callback(result);
        } else {
            let wrapped = Box::new(move || {
                let result = task();
                callback(result);
            });
            let _ = self.task_tx.send(wrapped);
        }
    }
}

/// `Slate` UI widget handle (`FFI` placeholder).
///
/// In real implementation, this would be a pointer to a `Slate` widget.
/// For now, we use an opaque handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlateWidgetHandle(pub u64);

impl std::fmt::Display for SlateWidgetHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            write!(f, "SlateWidgetHandle(null)")
        } else {
            write!(f, "SlateWidgetHandle({})", self.0)
        }
    }
}

impl SlateWidgetHandle {
    /// Create a null handle.
    #[must_use]
    pub fn null() -> Self {
        Self(0)
    }

    /// Check if this is a null handle.
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// `WebView` embedding mode within UE `Slate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UeEmbedMode {
    /// Embed as a `Slate` widget (`SWindow`/`SWidget`).
    SlateWidget,
    /// Embed as a child window (Win32 `HWND` parenting).
    NativeChildWindow,
    /// Floating tool window (for non-`Slate` DCCs).
    FloatingWindow,
}

impl std::fmt::Display for UeEmbedMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::SlateWidget => "SlateWidget",
            Self::NativeChildWindow => "NativeChildWindow",
            Self::FloatingWindow => "FloatingWindow",
        };
        write!(f, "{name}")
    }
}

/// Configuration for UE `WebView` integration.
#[derive(Debug, Clone)]
pub struct UeWebViewConfig {
    /// Initial size (width, height).
    pub initial_size: (u32, u32),
    /// Embed mode.
    pub embed_mode: UeEmbedMode,
    /// Enable developer tools.
    pub dev_tools: bool,
    /// JavaScript to execute on load.
    pub init_script: Option<String>,
}

impl std::fmt::Display for UeWebViewConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UeWebViewConfig {{ {}x{}, mode: {}, dev_tools: {} }}",
            self.initial_size.0, self.initial_size.1, self.embed_mode, self.dev_tools
        )
    }
}

impl Default for UeWebViewConfig {
    fn default() -> Self {
        Self {
            initial_size: (800, 600),
            embed_mode: UeEmbedMode::SlateWidget,
            dev_tools: false,
            init_script: None,
        }
    }
}

/// UE integration manager.
///
/// Manages `WebView` embedding within Unreal Engine's `Slate` UI system.
/// Handles `GameThread` synchronization and GC-safe object references.
#[allow(dead_code)]
pub struct UeIntegration {
    executor: Arc<UeGameThreadExecutor>,
    task_rx: Arc<Mutex<Option<Receiver<GameThreadTask>>>>,
    config: UeWebViewConfig,
    slate_parent: Option<SlateWidgetHandle>,
}

impl UeIntegration {
    /// Create a new `UeIntegration` instance.
    #[must_use]
    pub fn new(config: UeWebViewConfig) -> Self {
        let (executor, task_rx) = UeGameThreadExecutor::new();
        Self {
            executor: Arc::new(executor),
            task_rx: Arc::new(Mutex::new(Some(task_rx))),
            config,
            slate_parent: None,
        }
    }

    /// Set the parent Slate widget for embedding.
    pub fn set_parent_widget(&mut self, handle: SlateWidgetHandle) {
        self.slate_parent = Some(handle);
    }

    /// Get the `GameThread` executor.
    #[must_use]
    pub fn executor(&self) -> &UeGameThreadExecutor {
        &self.executor
    }

    /// Process pending `GameThread` tasks (should be called from `GameThread` each frame).
    pub fn process_tasks(&self) {
        if let Ok(guard) = self.task_rx.lock() {
            if let Some(rx) = guard.as_ref() {
                while let Ok(task) = rx.try_recv() {
                    task();
                }
            }
        }
    }

    /// Create a `WebView` embedded in UE Slate.
    ///
    /// This must be called from the `GameThread`.
    ///
    /// # Errors
    ///
    /// Returns `UeError::NotOnGameThread` if not called from the `GameThread`.
    pub fn create_webview(&self, url: &str) -> Result<SlateWidgetHandle, UeError> {
        if !self.executor.is_game_thread() {
            return Err(UeError::NotOnGameThread);
        }

        // TODO: Actual implementation would:
        // 1. Create a Slate window/widget
        // 2. Get its native handle (HWND on Windows)
        // 3. Pass to WebView backend
        // 4. Return handle for later reference

        tracing::info!("Creating WebView in UE Slate with URL: {url}");
        Ok(SlateWidgetHandle::null()) // Placeholder
    }
}

/// Errors that can occur in UE integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UeError {
    /// Operation must be performed on the `GameThread`.
    NotOnGameThread,
    /// Invalid Slate widget handle.
    InvalidHandle,
    /// `WebView` creation failed.
    WebViewCreationFailed(String),
    /// GC object was collected.
    ObjectCollected,
}

impl std::fmt::Display for UeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOnGameThread => write!(f, "operation must be on GameThread"),
            Self::InvalidHandle => write!(f, "invalid Slate widget handle"),
            Self::WebViewCreationFailed(msg) => {
                write!(f, "WebView creation failed: {msg}")
            }
            Self::ObjectCollected => write!(f, "UE object was garbage collected"),
        }
    }
}

impl std::error::Error for UeError {}

// --- Python Bindings ---

#[cfg(feature = "python-bindings")]
mod python {
    use super::*;
    use pyo3::prelude::*;

    #[pyclass(name = "UeIntegration")]
    #[pyo3(crate = "pyo3")]
    pub struct PyUeIntegration {
        inner: UeIntegration,
    }

    #[pymethods]
    impl PyUeIntegration {
        #[new]
        #[pyo3(signature = (width = 800, height = 600, dev_tools = false))]
        fn new(width: u32, height: u32, dev_tools: bool) -> Self {
            let config = UeWebViewConfig {
                initial_size: (width, height),
                embed_mode: UeEmbedMode::SlateWidget,
                dev_tools,
                init_script: None,
            };
            Self {
                inner: UeIntegration::new(config),
            }
        }

        /// Process pending GameThread tasks.
        #[pyo3(crate = "pyo3")]
        fn process_tasks(&self) {
            self.inner.process_tasks();
        }

        /// Create a WebView (must be called from GameThread).
        #[pyo3(crate = "pyo3")]
        fn create_webview(&self, url: &str) -> PyResult<u64> {
            match self.inner.create_webview(url) {
                Ok(handle) => Ok(handle.0),
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to create WebView: {e}"
                ))),
            }
        }
    }

    /// Register the module.
    #[pymodule]
    #[pyo3(crate = "pyo3")]
    fn auroraview_ue(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyUeIntegration>()?;
        Ok(())
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn gamethread_id_detection() {
        let id = GameThreadId::current();
        assert!(id.is_current());
    }

    #[test]
    fn gamethread_executor_immediate() {
        let (executor, _rx) = UeGameThreadExecutor::new();
        // On same thread, should execute immediately
        let result = Arc::new(Mutex::new(None));
        let result_clone = Arc::clone(&result);
        executor.execute(move || {
            let mut r = result_clone.lock().unwrap();
            *r = Some(42);
        });
        let r = result.lock().unwrap();
        assert_eq!(*r, Some(42));
    }

    #[test]
    fn gamethread_executor_with_callback() {
        let (executor, _rx) = UeGameThreadExecutor::new();
        let output = Arc::new(Mutex::new(None));
        let output_clone = Arc::clone(&output);
        executor.execute_with_callback(
            || Ok(()),
            move |result| {
                let mut o = output_clone.lock().unwrap();
                *o = Some(result);
            },
        );
        let o = output.lock().unwrap();
        assert_eq!(*o, Some(Ok(())));
    }

    #[test]
    fn slate_widget_handle() {
        let handle = SlateWidgetHandle::null();
        assert!(handle.is_null());

        let handle = SlateWidgetHandle(123);
        assert!(!handle.is_null());
    }

    #[test]
    fn ue_integration_creation() {
        let config = UeWebViewConfig::default();
        let ue = UeIntegration::new(config);
        // Should not panic
        ue.process_tasks();
    }

    #[test]
    fn ue_integration_create_webview_on_game_thread() {
        let config = UeWebViewConfig::default();
        let ue = UeIntegration::new(config);
        // This should work since we're on the GameThread (in test)
        let result = ue.create_webview("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn ue_error_display() {
        let err = UeError::NotOnGameThread;
        assert_eq!(err.to_string(), "operation must be on GameThread");

        let err = UeError::WebViewCreationFailed("test".into());
        assert!(err.to_string().contains("test"));
    }

    // Additional tests for Display impls

    #[test]
    fn ue_embed_mode_display() {
        assert_eq!(format!("{}", UeEmbedMode::SlateWidget), "SlateWidget");
        assert_eq!(
            format!("{}", UeEmbedMode::NativeChildWindow),
            "NativeChildWindow"
        );
        assert_eq!(format!("{}", UeEmbedMode::FloatingWindow), "FloatingWindow");
    }

    #[test]
    fn ue_webview_config_display() {
        let config = UeWebViewConfig::default();
        let display = format!("{}", config);
        assert!(display.contains("800"));
        assert!(display.contains("600"));
        assert!(display.contains("SlateWidget"));
    }
}

// ---------------------------------------------------------------------------
// UE `Blueprint` Node Support (Placeholder Implementation)
// ---------------------------------------------------------------------------

/// UE `Blueprint` node wrapper.
///
/// Represents a node in UE's `Blueprint` visual scripting system.
/// This is a placeholder implementation — real implementation would
/// interface with UE's `FKismetCompilerContext` or Python API.
#[derive(Debug, Clone)]
pub struct UeBlueprintNode {
    /// Node identifier (matches UE's internal node ID).
    pub id: String,
    /// Human-readable node title.
    pub title: String,
    /// Input pins (name → type).
    pub inputs: Vec<(String, String)>,
    /// Output pins (name → type).
    pub outputs: Vec<(String, String)>,
    /// Connections to other nodes.
    pub connections: Vec<String>,
}

impl UeBlueprintNode {
    /// Create a new Blueprint node.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            connections: Vec::new(),
        }
    }

    /// Add an input pin.
    pub fn add_input(&mut self, name: impl Into<String>, type_name: impl Into<String>) {
        self.inputs.push((name.into(), type_name.into()));
    }

    /// Add an output pin.
    pub fn add_output(&mut self, name: impl Into<String>, type_name: impl Into<String>) {
        self.outputs.push((name.into(), type_name.into()));
    }

    /// Connect this node to another node.
    pub fn connect_to(&mut self, node_id: impl Into<String>) {
        self.connections.push(node_id.into());
    }

    /// Get the node as JSON (for frontend/serialization).
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "title": self.title,
            "inputs": self.inputs,
            "outputs": self.outputs,
            "connections": self.connections,
        })
    }

    /// Remove an input pin by name.
    pub fn remove_input(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.inputs.retain(|(n, _)| n != &name);
    }

    /// Remove an output pin by name.
    pub fn remove_output(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.outputs.retain(|(n, _)| n != &name);
    }

    /// Remove a connection to another node.
    pub fn remove_connection(&mut self, node_id: impl Into<String>) {
        let node_id = node_id.into();
        self.connections.retain(|c| c != &node_id);
    }

    /// Clear all input pins.
    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    /// Clear all output pins.
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    /// Clear all connections.
    pub fn clear_connections(&mut self) {
        self.connections.clear();
    }
}

/// Errors that can occur in Blueprint node operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UeBlueprintError {
    /// Node with given ID not found.
    NodeNotFound(String),
    /// Invalid pin type.
    InvalidPinType(String),
    /// Compilation failed.
    CompilationFailed(String),
}

impl std::fmt::Display for UeBlueprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Blueprint node not found: {id}"),
            Self::InvalidPinType(t) => write!(f, "invalid pin type: {t}"),
            Self::CompilationFailed(msg) => write!(f, "compilation failed: {msg}"),
        }
    }
}

impl std::error::Error for UeBlueprintError {}
