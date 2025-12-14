//! Shared drag-drop handler for WebView
//!
//! This module provides a reusable drag-drop handler that can be used
//! in both standalone and DCC embedded modes.

use std::sync::Arc;
use wry::DragDropEvent;

/// Callback type for handling drag-drop events
pub type DragDropCallback = Arc<dyn Fn(DragDropEventData) + Send + Sync>;

/// Drag-drop event data
#[derive(Debug, Clone)]
pub struct DragDropEventData {
    /// Event type
    pub event_type: DragDropEventType,
    /// File paths (for Enter and Drop events)
    pub paths: Vec<String>,
    /// Position (x, y)
    pub position: Option<(f64, f64)>,
    /// Timestamp (for Drop events)
    pub timestamp: Option<u64>,
}

/// Drag-drop event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragDropEventType {
    /// Files entered the window
    Enter,
    /// Files are hovering over the window
    Over,
    /// Files were dropped
    Drop,
    /// Files left the window
    Leave,
}

impl DragDropEventType {
    /// Get the IPC event name for this event type
    pub fn as_event_name(&self) -> &'static str {
        match self {
            Self::Enter => "file_drop_hover",
            Self::Over => "file_drop_over",
            Self::Drop => "file_drop",
            Self::Leave => "file_drop_cancelled",
        }
    }
}

/// Shared drag-drop handler
///
/// This handler processes wry's DragDropEvent and converts it to
/// a format suitable for IPC messaging.
pub struct DragDropHandler {
    callback: DragDropCallback,
}

impl DragDropHandler {
    /// Create a new drag-drop handler with a callback
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(DragDropEventData) + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(callback),
        }
    }

    /// Create a handler function for wry's with_drag_drop_handler
    ///
    /// Returns a closure that can be passed to `WebViewBuilder::with_drag_drop_handler`
    pub fn into_handler(self) -> impl Fn(DragDropEvent) -> bool + 'static {
        let callback = self.callback;

        move |event: DragDropEvent| {
            let data = match event {
                DragDropEvent::Enter { paths, position } => {
                    let paths_str: Vec<String> = paths
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let (x, y) = position;

                    tracing::debug!(
                        "[DragDropHandler] Enter - {} files at ({}, {})",
                        paths_str.len(),
                        x,
                        y
                    );

                    DragDropEventData {
                        event_type: DragDropEventType::Enter,
                        paths: paths_str,
                        position: Some((x as f64, y as f64)),
                        timestamp: None,
                    }
                }
                DragDropEvent::Over { position } => {
                    let (x, y) = position;
                    tracing::trace!("[DragDropHandler] Over at ({}, {})", x, y);

                    DragDropEventData {
                        event_type: DragDropEventType::Over,
                        paths: Vec::new(),
                        position: Some((x as f64, y as f64)),
                        timestamp: None,
                    }
                }
                DragDropEvent::Drop { paths, position } => {
                    let paths_str: Vec<String> = paths
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let (x, y) = position;

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);

                    tracing::info!(
                        "[DragDropHandler] Drop - {} files at ({}, {}): {:?}",
                        paths_str.len(),
                        x,
                        y,
                        paths_str
                    );

                    DragDropEventData {
                        event_type: DragDropEventType::Drop,
                        paths: paths_str,
                        position: Some((x as f64, y as f64)),
                        timestamp: Some(timestamp),
                    }
                }
                DragDropEvent::Leave => {
                    tracing::debug!("[DragDropHandler] Leave");

                    DragDropEventData {
                        event_type: DragDropEventType::Leave,
                        paths: Vec::new(),
                        position: None,
                        timestamp: None,
                    }
                }
                _ => {
                    // Handle future variants (DragDropEvent is non_exhaustive)
                    tracing::debug!("[DragDropHandler] Unknown event variant");
                    return true;
                }
            };

            callback(data);

            // Return true to prevent default browser drag-drop behavior
            true
        }
    }
}

impl DragDropEventData {
    /// Convert to JSON value for IPC messaging
    pub fn to_json(&self) -> serde_json::Value {
        match self.event_type {
            DragDropEventType::Enter => {
                serde_json::json!({
                    "hovering": true,
                    "paths": self.paths,
                    "position": self.position.map(|(x, y)| serde_json::json!({"x": x, "y": y}))
                })
            }
            DragDropEventType::Over => {
                serde_json::json!({
                    "position": self.position.map(|(x, y)| serde_json::json!({"x": x, "y": y}))
                })
            }
            DragDropEventType::Drop => {
                serde_json::json!({
                    "paths": self.paths,
                    "position": self.position.map(|(x, y)| serde_json::json!({"x": x, "y": y})),
                    "timestamp": self.timestamp
                })
            }
            DragDropEventType::Leave => {
                serde_json::json!({
                    "hovering": false,
                    "reason": "left_window"
                })
            }
        }
    }
}
