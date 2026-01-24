//! User events for the event loop

/// Custom user events for the desktop event loop
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// Close the window
    CloseWindow,

    /// Show the window
    ShowWindow,

    /// Hide the window
    HideWindow,

    /// Start window drag
    DragWindow,

    /// Plugin event from background thread
    PluginEvent { event: String, data: String },

    /// Evaluate JavaScript
    EvalJs(String),

    /// Wake up the event loop (for message processing)
    WakeUp,
}
