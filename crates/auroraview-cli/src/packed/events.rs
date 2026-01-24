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
    /// Python process crashed or exited unexpectedly
    /// This triggers a full error page display
    PythonCrash {
        /// Exit code (if available)
        exit_code: Option<i32>,
        /// Last captured stderr output
        stderr_output: String,
        /// Whether crash happened during startup
        during_startup: bool,
    },
    /// Set HTML content for WebView (dynamic HTML from Python)
    /// Used by Browser component in packed mode to load dynamic HTML
    SetHtml {
        /// The HTML content to load
        html: String,
        /// Optional title to set for the window
        title: Option<String>,
    },
    /// Close the window and exit the application
    /// This is triggered by window.close() API from JavaScript
    CloseWindow,
    /// Show error page with full diagnostics
    /// Used when critical errors occur that prevent normal operation
    ShowError {
        /// HTTP-style status code (e.g., 500, 503)
        code: u16,
        /// Error title
        title: String,
        /// User-friendly error message
        message: String,
        /// Technical details (stack trace, etc.)
        details: Option<String>,
        /// Error source: "python", "rust", "javascript", "unknown"
        source: String,
    },
}
