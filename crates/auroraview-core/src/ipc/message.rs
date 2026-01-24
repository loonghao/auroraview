//! IPC Message Types
//!
//! Core message structures for IPC communication, independent of any
//! specific language bindings.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// IPC message structure
///
/// This is the fundamental message type used for all IPC communication.
/// It is serializable and can be sent between threads or processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Event name (e.g., "click", "state_changed", "invoke")
    pub event: String,

    /// Message data as JSON value
    pub data: Value,

    /// Optional message ID for request-response pattern
    pub id: Option<String>,
}

impl IpcMessage {
    /// Create a new IPC message
    pub fn new(event: impl Into<String>, data: Value) -> Self {
        Self {
            event: event.into(),
            data,
            id: None,
        }
    }

    /// Create a new IPC message with an ID
    pub fn with_id(event: impl Into<String>, data: Value, id: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            data,
            id: Some(id.into()),
        }
    }
}

/// IPC mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IpcMode {
    /// Thread-based communication (default for embedded mode)
    #[default]
    Threaded,

    /// Process-based communication (for standalone mode)
    Process,
}

/// Message types that can be sent to the WebView
///
/// This is the unified message type used for WebView operations across
/// all modes (standalone, CLI packed, DCC embedded).
#[derive(Debug, Clone)]
pub enum WebViewMessage {
    /// Execute JavaScript code
    EvalJs(String),

    /// Execute JavaScript code with async callback
    /// Returns result via the provided callback
    EvalJsAsync {
        script: String,
        callback_id: u64,
    },

    /// Emit an event to JavaScript
    EmitEvent {
        event_name: String,
        data: serde_json::Value,
    },

    /// Load a URL
    LoadUrl(String),

    /// Load HTML content
    LoadHtml(String),

    /// Set window visibility
    SetVisible(bool),

    /// Reload the current page
    Reload,

    /// Stop loading the current page
    StopLoading,

    /// Window event notification (from Rust to callbacks)
    WindowEvent {
        event_type: WindowEventType,
        data: serde_json::Value,
    },

    /// Close the WebView window
    Close,
}

/// Window event types for lifecycle tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEventType {
    /// Window has been shown/visible
    Shown,
    /// Window has been hidden
    Hidden,
    /// Window is about to close (can be cancelled)
    Closing,
    /// Window has been closed
    Closed,
    /// Window gained focus
    Focused,
    /// Window lost focus
    Blurred,
    /// Window was minimized
    Minimized,
    /// Window was maximized
    Maximized,
    /// Window was restored from minimized/maximized
    Restored,
    /// Window was resized (data includes width, height)
    Resized,
    /// Window was moved (data includes x, y)
    Moved,
    /// Page started loading
    LoadStarted,
    /// Page finished loading
    LoadFinished,
    /// Navigation started (data includes url)
    NavigationStarted,
    /// Navigation finished (data includes url)
    NavigationFinished,
    /// WebView2 native window has been created (data includes hwnd)
    /// This is emitted after the WebView2 controller is ready and HWND is available
    WebView2Created,
}

impl WindowEventType {
    /// Convert to event name string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shown => "shown",
            Self::Hidden => "hidden",
            Self::Closing => "closing",
            Self::Closed => "closed",
            Self::Focused => "focused",
            Self::Blurred => "blurred",
            Self::Minimized => "minimized",
            Self::Maximized => "maximized",
            Self::Restored => "restored",
            Self::Resized => "resized",
            Self::Moved => "moved",
            Self::LoadStarted => "load_started",
            Self::LoadFinished => "load_finished",
            Self::NavigationStarted => "navigation_started",
            Self::NavigationFinished => "navigation_finished",
            Self::WebView2Created => "webview2_created",
        }
    }
}

impl std::fmt::Display for WindowEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
