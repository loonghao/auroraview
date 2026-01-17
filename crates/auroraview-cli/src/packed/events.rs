//! User events for communication between threads and WebView

/// User event for communication between threads and WebView
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// Python response to be sent to WebView
    PythonResponse(String),
    /// Plugin event to be sent to WebView
    PluginEvent { event: String, data: String },
    /// Python backend is ready, notify frontend
    /// Contains list of registered API handlers (e.g., ["api.get_samples", "api.run_sample"])
    PythonReady { handlers: Vec<String> },
    /// Loading screen is ready (DOM rendered)
    LoadingScreenReady,
    /// Navigate to application (triggered by frontend)
    NavigateToApp,
    /// Page ready event (triggered when new page loads and auroraview bridge is initialized)
    /// This is used to re-register API methods after navigation
    PageReady,
    /// Loading screen update (progress, text, steps)
    LoadingUpdate {
        progress: Option<i32>,
        text: Option<String>,
        step_id: Option<String>,
        step_text: Option<String>,
        step_status: Option<String>,
    },
    /// Backend error from Python stderr
    /// Used to display errors on the loading screen for debugging
    BackendError {
        message: String,
        /// Error source: "stderr", "startup", "crash"
        source: String,
    },
    /// Set HTML content for WebView (dynamic HTML from Python)
    /// Used by Browser component in packed mode to load dynamic HTML
    SetHtml {
        /// The HTML content to load
        html: String,
        /// Optional title to set for the window
        title: Option<String>,
    },
}
