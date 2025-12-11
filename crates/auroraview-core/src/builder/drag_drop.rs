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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_type_names() {
        assert_eq!(DragDropEventType::Enter.as_event_name(), "file_drop_hover");
        assert_eq!(DragDropEventType::Over.as_event_name(), "file_drop_over");
        assert_eq!(DragDropEventType::Drop.as_event_name(), "file_drop");
        assert_eq!(
            DragDropEventType::Leave.as_event_name(),
            "file_drop_cancelled"
        );
    }

    #[test]
    fn test_event_type_equality() {
        assert_eq!(DragDropEventType::Enter, DragDropEventType::Enter);
        assert_ne!(DragDropEventType::Enter, DragDropEventType::Leave);
    }

    #[test]
    fn test_event_type_debug() {
        let event = DragDropEventType::Drop;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Drop"));
    }

    #[test]
    fn test_event_type_clone() {
        let original = DragDropEventType::Enter;
        let cloned = original;
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_event_type_copy() {
        let original = DragDropEventType::Over;
        let copied = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn test_event_data_to_json_enter() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Enter,
            paths: vec!["file1.txt".to_string(), "file2.txt".to_string()],
            position: Some((100.0, 200.0)),
            timestamp: None,
        };

        let json = data.to_json();
        assert_eq!(json["hovering"], true);
        assert_eq!(json["paths"].as_array().unwrap().len(), 2);
        assert_eq!(json["position"]["x"], 100.0);
        assert_eq!(json["position"]["y"], 200.0);
    }

    #[test]
    fn test_event_data_to_json_enter_no_position() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Enter,
            paths: vec!["file.txt".to_string()],
            position: None,
            timestamp: None,
        };

        let json = data.to_json();
        assert_eq!(json["hovering"], true);
        assert!(json["position"].is_null());
    }

    #[test]
    fn test_event_data_to_json_over() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Over,
            paths: Vec::new(),
            position: Some((150.0, 250.0)),
            timestamp: None,
        };

        let json = data.to_json();
        assert_eq!(json["position"]["x"], 150.0);
        assert_eq!(json["position"]["y"], 250.0);
    }

    #[test]
    fn test_event_data_to_json_over_no_position() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Over,
            paths: Vec::new(),
            position: None,
            timestamp: None,
        };

        let json = data.to_json();
        assert!(json["position"].is_null());
    }

    #[test]
    fn test_event_data_to_json_drop() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Drop,
            paths: vec!["file.txt".to_string()],
            position: Some((50.0, 75.0)),
            timestamp: Some(1234567890),
        };

        let json = data.to_json();
        assert_eq!(json["paths"].as_array().unwrap().len(), 1);
        assert_eq!(json["paths"][0], "file.txt");
        assert_eq!(json["timestamp"], 1234567890);
        assert_eq!(json["position"]["x"], 50.0);
        assert_eq!(json["position"]["y"], 75.0);
    }

    #[test]
    fn test_event_data_to_json_drop_multiple_files() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Drop,
            paths: vec![
                "/path/to/file1.txt".to_string(),
                "/path/to/file2.png".to_string(),
                "/path/to/file3.pdf".to_string(),
            ],
            position: Some((0.0, 0.0)),
            timestamp: Some(0),
        };

        let json = data.to_json();
        let paths = json["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], "/path/to/file1.txt");
        assert_eq!(paths[1], "/path/to/file2.png");
        assert_eq!(paths[2], "/path/to/file3.pdf");
    }

    #[test]
    fn test_event_data_to_json_drop_no_timestamp() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Drop,
            paths: vec!["file.txt".to_string()],
            position: Some((10.0, 20.0)),
            timestamp: None,
        };

        let json = data.to_json();
        assert!(json["timestamp"].is_null());
    }

    #[test]
    fn test_event_data_to_json_leave() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Leave,
            paths: Vec::new(),
            position: None,
            timestamp: None,
        };

        let json = data.to_json();
        assert_eq!(json["hovering"], false);
        assert_eq!(json["reason"], "left_window");
    }

    #[test]
    fn test_event_data_debug() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Drop,
            paths: vec!["test.txt".to_string()],
            position: Some((10.0, 20.0)),
            timestamp: Some(12345),
        };

        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("DragDropEventData"));
        assert!(debug_str.contains("Drop"));
        assert!(debug_str.contains("test.txt"));
    }

    #[test]
    fn test_event_data_clone() {
        let original = DragDropEventData {
            event_type: DragDropEventType::Enter,
            paths: vec!["file.txt".to_string()],
            position: Some((1.0, 2.0)),
            timestamp: None,
        };

        let cloned = original.clone();
        assert_eq!(original.event_type, cloned.event_type);
        assert_eq!(original.paths, cloned.paths);
        assert_eq!(original.position, cloned.position);
        assert_eq!(original.timestamp, cloned.timestamp);
    }

    #[test]
    fn test_handler_callback() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let handler = DragDropHandler::new(move |_data| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // We can't easily test the actual handler without wry,
        // but we can verify the handler is created correctly
        let _handler_fn = handler.into_handler();

        // The callback should be callable
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_handler_with_data_capture() {
        let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let _handler = DragDropHandler::new(move |data| {
            captured_clone.lock().unwrap().push(data);
        });

        // Verify handler is created
        assert!(captured.lock().unwrap().is_empty());
    }

    #[test]
    fn test_empty_paths() {
        let data = DragDropEventData {
            event_type: DragDropEventType::Enter,
            paths: Vec::new(),
            position: Some((0.0, 0.0)),
            timestamp: None,
        };

        let json = data.to_json();
        assert!(json["paths"].as_array().unwrap().is_empty());
    }
}
