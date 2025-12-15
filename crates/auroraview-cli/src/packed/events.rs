//! User events for communication between threads and WebView

/// User event for communication between threads and WebView
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// Python response to be sent to WebView
    PythonResponse(String),
    /// Plugin event to be sent to WebView
    PluginEvent { event: String, data: String },
    /// Python backend is ready, notify frontend
    PythonReady,
    /// Loading screen is ready (DOM rendered)
    LoadingScreenReady,
    /// Navigate to application (triggered by frontend)
    NavigateToApp,
    /// Loading screen update (progress, text, steps)
    LoadingUpdate {
        progress: Option<i32>,
        text: Option<String>,
        step_id: Option<String>,
        step_text: Option<String>,
        step_status: Option<String>,
    },
}
