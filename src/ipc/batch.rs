//! IPC implementation with message batching and reduced GIL contention
//!
//! This module provides performance improvements over the basic IPC handler:
//! 1. Message batching - group multiple messages to reduce overhead
//! 2. Reduced GIL locking - minimize Python GIL acquisition
//! 3. Zero-copy serialization where possible
//! 4. Async message processing

#[cfg(feature = "python-bindings")]
use dashmap::DashMap;
#[cfg(feature = "python-bindings")]
use parking_lot::RwLock;
#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3::types::{PyDict, PyList};
#[cfg(feature = "python-bindings")]
use pyo3::{Py, PyAny};
use serde::{Deserialize, Serialize};
#[cfg(feature = "python-bindings")]
use std::sync::Arc;

/// IPC message with metadata for batching
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchedMessage {
    /// Event name
    pub event: String,

    /// Message payload
    pub data: serde_json::Value,

    /// Message priority (higher = more important)
    #[serde(default)]
    pub priority: u8,

    /// Timestamp (milliseconds since epoch)
    #[serde(default)]
    pub timestamp: u64,

    /// Message ID for tracking
    #[serde(default)]
    pub id: Option<String>,
}

#[allow(dead_code)]
impl BatchedMessage {
    /// Create a new message
    pub fn new(event: String, data: serde_json::Value) -> Self {
        Self {
            event,
            data,
            priority: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            id: None,
        }
    }

    /// Create a high-priority message
    pub fn high_priority(event: String, data: serde_json::Value) -> Self {
        Self {
            event,
            data,
            priority: 10,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            id: None,
        }
    }
}

/// Message batch for efficient processing
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MessageBatch {
    /// Messages in this batch
    pub messages: Vec<BatchedMessage>,

    /// Batch creation time
    pub created_at: std::time::Instant,
}

#[allow(dead_code)]
impl MessageBatch {
    /// Create a new batch
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            created_at: std::time::Instant::now(),
        }
    }

    /// Add a message to the batch
    pub fn add(&mut self, message: BatchedMessage) {
        self.messages.push(message);
    }

    /// Check if batch should be flushed
    pub fn should_flush(&self, max_size: usize, max_age_ms: u64) -> bool {
        self.messages.len() >= max_size
            || self.created_at.elapsed().as_millis() as u64 >= max_age_ms
    }

    /// Sort messages by priority (high to low)
    pub fn sort_by_priority(&mut self) {
        self.messages.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

impl Default for MessageBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Python callback with reduced GIL contention
#[cfg(feature = "python-bindings")]
#[allow(dead_code)]
pub struct BatchedCallback {
    /// Python callable object
    callback: Py<PyAny>,

    /// Whether to batch messages
    batching_enabled: bool,
}

#[cfg(feature = "python-bindings")]
#[allow(dead_code)]
impl BatchedCallback {
    /// Create a new batched callback
    pub fn new(callback: Py<PyAny>, batching_enabled: bool) -> Self {
        Self {
            callback,
            batching_enabled,
        }
    }

    /// Call the callback with a single message
    pub fn call_single(&self, message: &BatchedMessage) -> Result<(), String> {
        Python::attach(|py| {
            // Convert message to Python dict
            let py_dict = PyDict::new(py);
            py_dict
                .set_item("event", &message.event)
                .map_err(|e| format!("Failed to set event: {}", e))?;

            // Convert data to Python object
            let py_data = json_to_python(py, &message.data)
                .map_err(|e| format!("Failed to convert data: {}", e))?;
            py_dict
                .set_item("data", py_data)
                .map_err(|e| format!("Failed to set data: {}", e))?;

            py_dict
                .set_item("priority", message.priority)
                .map_err(|e| format!("Failed to set priority: {}", e))?;
            py_dict
                .set_item("timestamp", message.timestamp)
                .map_err(|e| format!("Failed to set timestamp: {}", e))?;

            // Call Python callback
            self.callback
                .call1(py, (py_dict,))
                .map_err(|e| format!("Python callback error: {}", e))?;

            Ok(())
        })
    }

    /// Call the callback with a batch of messages
    pub fn call_batch(&self, batch: &MessageBatch) -> Result<(), String> {
        if !self.batching_enabled {
            // Fall back to individual calls
            for msg in &batch.messages {
                self.call_single(msg)?;
            }
            return Ok(());
        }

        Python::attach(|py| {
            // Convert batch to Python list (create empty list)
            let py_list = PyList::new(py, Vec::<i32>::new())
                .map_err(|e| format!("Failed to create list: {}", e))?;

            for message in &batch.messages {
                let py_dict = PyDict::new(py);
                py_dict
                    .set_item("event", &message.event)
                    .map_err(|e| format!("Failed to set event: {}", e))?;

                let py_data = json_to_python(py, &message.data)
                    .map_err(|e| format!("Failed to convert data: {}", e))?;
                py_dict
                    .set_item("data", py_data)
                    .map_err(|e| format!("Failed to set data: {}", e))?;

                py_dict
                    .set_item("priority", message.priority)
                    .map_err(|e| format!("Failed to set priority: {}", e))?;
                py_dict
                    .set_item("timestamp", message.timestamp)
                    .map_err(|e| format!("Failed to set timestamp: {}", e))?;

                py_list
                    .append(py_dict)
                    .map_err(|e| format!("Failed to append to list: {}", e))?;
            }

            // Call Python callback with batch
            self.callback
                .call1(py, (py_list,))
                .map_err(|e| format!("Python callback error: {}", e))?;

            Ok(())
        })
    }
}

/// Convert JSON value to Python object
#[cfg(feature = "python-bindings")]
#[allow(dead_code)]
fn json_to_python(py: Python, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => {
            let obj = b.into_pyobject(py)?;
            Ok(obj.as_any().clone().unbind())
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                let obj = i.into_pyobject(py)?;
                Ok(obj.as_any().clone().unbind())
            } else if let Some(f) = n.as_f64() {
                let obj = f.into_pyobject(py)?;
                Ok(obj.as_any().clone().unbind())
            } else {
                let obj = n.to_string().into_pyobject(py)?;
                Ok(obj.as_any().clone().unbind())
            }
        }
        serde_json::Value::String(s) => {
            let obj = s.into_pyobject(py)?;
            Ok(obj.as_any().clone().unbind())
        }
        serde_json::Value::Array(arr) => {
            let py_list = PyList::new(py, arr.iter().map(|_| py.None()))?;
            for (idx, item) in arr.iter().enumerate() {
                let py_item = json_to_python(py, item)?;
                py_list.set_item(idx, py_item)?;
            }
            Ok(py_list.into_any().unbind())
        }
        serde_json::Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                let py_val = json_to_python(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into_any().unbind())
        }
    }
}

/// IPC handler with message batching support
#[cfg(feature = "python-bindings")]
#[allow(dead_code)]
pub struct BatchedHandler {
    /// Registered callbacks
    callbacks: Arc<DashMap<String, Vec<BatchedCallback>>>,

    /// Message queue for batching
    message_queue: Arc<RwLock<MessageBatch>>,

    /// Batch configuration
    max_batch_size: usize,
    max_batch_age_ms: u64,
}

#[cfg(feature = "python-bindings")]
#[allow(dead_code)]
impl BatchedHandler {
    /// Create a new batched IPC handler
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(DashMap::new()),
            message_queue: Arc::new(RwLock::new(MessageBatch::new())),
            max_batch_size: 10,
            max_batch_age_ms: 16, // ~60 FPS
        }
    }

    /// Register a callback for an event
    pub fn on(&self, event: String, callback: Py<PyAny>, batching: bool) {
        let cb = BatchedCallback::new(callback, batching);
        self.callbacks.entry(event).or_default().push(cb);
    }

    /// Emit a message (with batching)
    pub fn emit(&self, message: BatchedMessage) -> Result<(), String> {
        let _event = message.event.clone();

        // Add to batch
        {
            let mut batch = self.message_queue.write();
            batch.add(message);

            // Check if we should flush
            if batch.should_flush(self.max_batch_size, self.max_batch_age_ms) {
                self.flush_batch()?;
            }
        }

        Ok(())
    }

    /// Flush the current batch
    pub fn flush_batch(&self) -> Result<(), String> {
        let batch = {
            let mut queue = self.message_queue.write();
            let mut new_batch = MessageBatch::new();
            std::mem::swap(&mut *queue, &mut new_batch);
            new_batch
        };

        if batch.messages.is_empty() {
            return Ok(());
        }

        // Group messages by event
        let mut event_batches: std::collections::HashMap<String, MessageBatch> =
            std::collections::HashMap::new();

        for message in batch.messages {
            event_batches
                .entry(message.event.clone())
                .or_default()
                .add(message);
        }

        // Process each event's batch
        for (event, mut batch) in event_batches {
            batch.sort_by_priority();

            if let Some(callbacks) = self.callbacks.get(&event) {
                for callback in callbacks.iter() {
                    callback.call_batch(&batch)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "python-bindings")]
impl Default for BatchedHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Integration tests have been moved to tests/ipc_batch_integration_tests.rs
// This includes tests for:
// - Message batch flush conditions (size and age-based)
// - BatchedCallback with Python integration
// - Single and batch message processing

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::thread;
    use std::time::Duration;

    #[fixture]
    fn sample_message() -> BatchedMessage {
        BatchedMessage::new(
            "test_event".to_string(),
            serde_json::json!({"key": "value"}),
        )
    }

    #[rstest]
    fn test_batched_message_new(sample_message: BatchedMessage) {
        assert_eq!(sample_message.event, "test_event");
        assert_eq!(sample_message.data, serde_json::json!({"key": "value"}));
        assert_eq!(sample_message.priority, 0);
        assert!(sample_message.timestamp > 0);
        assert!(sample_message.id.is_none());
    }

    #[test]
    fn test_batched_message_high_priority() {
        let msg = BatchedMessage::high_priority(
            "urgent_event".to_string(),
            serde_json::json!({"urgent": true}),
        );

        assert_eq!(msg.event, "urgent_event");
        assert_eq!(msg.priority, 10);
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_message_batch_new() {
        let batch = MessageBatch::new();
        assert!(batch.messages.is_empty());
    }

    #[test]
    fn test_message_batch_add() {
        let mut batch = MessageBatch::new();
        let message = BatchedMessage::new(
            "test_event".to_string(),
            serde_json::json!({"key": "value"}),
        );
        batch.add(message);

        assert_eq!(batch.messages.len(), 1);
        assert_eq!(batch.messages[0].event, "test_event");
    }

    #[test]
    fn test_message_batch_should_flush_by_size() {
        let mut batch = MessageBatch::new();

        for i in 0..5 {
            batch.add(BatchedMessage::new(
                format!("event_{}", i),
                serde_json::json!({}),
            ));
        }

        assert!(!batch.should_flush(10, 1000)); // Not at max size yet
        assert!(batch.should_flush(5, 1000)); // At max size
        assert!(batch.should_flush(3, 1000)); // Over max size
    }

    #[test]
    fn test_message_batch_should_flush_by_age() {
        let batch = MessageBatch::new();

        // Fresh batch should not flush by age
        assert!(!batch.should_flush(100, 100));

        // Sleep and check again
        thread::sleep(Duration::from_millis(50));
        assert!(batch.should_flush(100, 10)); // 10ms age, but we slept 50ms
    }

    #[test]
    fn test_message_batch_sort_by_priority() {
        let mut batch = MessageBatch::new();

        batch.add(BatchedMessage {
            event: "low".to_string(),
            data: serde_json::json!({}),
            priority: 1,
            timestamp: 0,
            id: None,
        });

        batch.add(BatchedMessage {
            event: "high".to_string(),
            data: serde_json::json!({}),
            priority: 10,
            timestamp: 0,
            id: None,
        });

        batch.add(BatchedMessage {
            event: "medium".to_string(),
            data: serde_json::json!({}),
            priority: 5,
            timestamp: 0,
            id: None,
        });

        batch.sort_by_priority();

        assert_eq!(batch.messages[0].event, "high");
        assert_eq!(batch.messages[0].priority, 10);
        assert_eq!(batch.messages[1].event, "medium");
        assert_eq!(batch.messages[1].priority, 5);
        assert_eq!(batch.messages[2].event, "low");
        assert_eq!(batch.messages[2].priority, 1);
    }

    #[test]
    fn test_message_batch_default() {
        let batch = MessageBatch::default();
        assert!(batch.messages.is_empty());
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_batched_handler_new() {
        let handler = BatchedHandler::new();
        assert_eq!(handler.max_batch_size, 10);
        assert_eq!(handler.max_batch_age_ms, 16);
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_batched_handler_default() {
        let handler = BatchedHandler::default();
        assert_eq!(handler.max_batch_size, 10);
        assert_eq!(handler.max_batch_age_ms, 16);
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_batched_handler_flush_empty_batch() {
        let handler = BatchedHandler::new();
        // Flushing an empty batch should succeed
        let result = handler.flush_batch();
        assert!(result.is_ok());
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_json_to_python_basic_types() {
        Python::attach(|py| {
            // Null
            let null_val = json_to_python(py, &serde_json::Value::Null).unwrap();
            assert!(null_val.is_none(py));

            // Bool
            let bool_val = json_to_python(py, &serde_json::Value::Bool(true)).unwrap();
            let extracted: bool = bool_val.bind(py).extract().unwrap();
            assert!(extracted);

            // Integer
            let int_val = json_to_python(py, &serde_json::json!(42)).unwrap();
            let extracted: i64 = int_val.bind(py).extract().unwrap();
            assert_eq!(extracted, 42);

            // Float
            let float_val = json_to_python(py, &serde_json::json!(1.234)).unwrap();
            let extracted: f64 = float_val.bind(py).extract().unwrap();
            assert!((extracted - 1.234).abs() < 0.001);

            // String
            let str_val = json_to_python(py, &serde_json::json!("hello")).unwrap();
            let extracted: String = str_val.bind(py).extract().unwrap();
            assert_eq!(extracted, "hello");

            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_json_to_python_array() {
        Python::attach(|py| {
            let arr = serde_json::json!([1, 2, 3]);
            let py_val = json_to_python(py, &arr).unwrap();
            let py_list = py_val.bind(py).cast::<PyList>().unwrap();

            assert_eq!(py_list.len(), 3);

            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_json_to_python_object() {
        Python::attach(|py| {
            let obj = serde_json::json!({"name": "test", "value": 123});
            let py_val = json_to_python(py, &obj).unwrap();
            let py_dict = py_val.bind(py).cast::<PyDict>().unwrap();

            let name = py_dict
                .get_item("name")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap();
            assert_eq!(name, "test");

            let value = py_dict
                .get_item("value")
                .unwrap()
                .unwrap()
                .extract::<i64>()
                .unwrap();
            assert_eq!(value, 123);

            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_batched_message_clone() {
        let msg = BatchedMessage::new("test".to_string(), serde_json::json!({"key": "value"}));
        let cloned = msg.clone();

        assert_eq!(msg.event, cloned.event);
        assert_eq!(msg.data, cloned.data);
        assert_eq!(msg.priority, cloned.priority);
        assert_eq!(msg.timestamp, cloned.timestamp);
    }

    #[test]
    fn test_batched_message_debug() {
        let msg = BatchedMessage::new("test".to_string(), serde_json::json!({}));
        let debug_str = format!("{:?}", msg);

        assert!(debug_str.contains("BatchedMessage"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_message_batch_clone() {
        let mut batch = MessageBatch::new();
        batch.add(BatchedMessage::new(
            "event".to_string(),
            serde_json::json!({}),
        ));

        let cloned = batch.clone();
        assert_eq!(batch.messages.len(), cloned.messages.len());
    }
}
