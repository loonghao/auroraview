//! User Events Module
//!
//! This module provides platform-agnostic user event types for WebView
//! communication across different modes (standalone, CLI, DCC embedded).
//!
//! ## Event Categories
//!
//! - **Core Events**: Common events used by all implementations
//! - **Window Events**: Window lifecycle and state changes
//! - **Plugin Events**: Plugin-related notifications

/// Core user event types (platform-agnostic)
///
/// These events are used for communication between threads and the WebView
/// event loop. They are the common denominator across all implementations.
#[derive(Debug, Clone)]
pub enum CoreUserEvent {
    /// Wake up the event loop to process pending messages
    ProcessMessages,

    /// Request to close the window
    CloseWindow,

    /// Plugin event to be forwarded to WebView
    PluginEvent {
        /// Event name
        event: String,
        /// JSON-encoded event data
        data: String,
    },

    /// Request to start native window drag
    DragWindow,
}

/// Extended user events for standalone/CLI mode
///
/// These events are specific to standalone applications and CLI packed mode,
/// not used in DCC embedded scenarios.
#[derive(Debug, Clone)]
pub enum ExtendedUserEvent {
    /// Python backend is ready, notify frontend
    /// Contains list of registered API handlers
    PythonReady { handlers: Vec<String> },

    /// Python response to be sent to WebView
    PythonResponse(String),

    /// Loading screen is ready (DOM rendered)
    LoadingScreenReady,

    /// Navigate to application (triggered by frontend)
    NavigateToApp,

    /// Page ready event (triggered when new page loads)
    PageReady,

    /// Loading screen update
    LoadingUpdate {
        progress: Option<i32>,
        text: Option<String>,
        step_id: Option<String>,
        step_text: Option<String>,
        step_status: Option<String>,
    },

    /// Backend error from Python stderr
    BackendError {
        message: String,
        /// Error source: "stderr", "startup", "crash"
        source: String,
    },

    /// Set HTML content for WebView
    SetHtml { html: String, title: Option<String> },

    /// Show error page with full diagnostics
    ShowError {
        code: u16,
        title: String,
        message: String,
        details: Option<String>,
        source: String,
    },

    /// Tray menu item clicked
    TrayMenuClick(String),

    /// Tray icon clicked
    TrayIconClick,

    /// Tray icon double-clicked
    TrayIconDoubleClick,

    /// Request to create a new child WebView window
    CreateChildWindow {
        url: String,
        width: u32,
        height: u32,
    },
}
