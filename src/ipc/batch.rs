//! IPC implementation with message batching and reduced GIL contention
//!
//! This module provides performance improvements over the basic IPC handler:
//! 1. Message batching - group multiple messages to reduce overhead
//! 2. Reduced GIL locking - minimize Python GIL acquisition
//! 3. Zero-copy serialization where possible
//! 4. Async message processing

use dashmap::DashMap;
use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::{Deserialize, Serialize};
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
#[allow(dead_code)]
pub struct BatchedCallback {
    /// Python callable object
    callback: PyObject,

    /// Whether to batch messages
    batching_enabled: bool,
}

#[allow(dead_code)]
impl BatchedCallback {
    /// Create a new batched callback
    pub fn new(callback: PyObject, batching_enabled: bool) -> Self {
        Self {
            callback,
            batching_enabled,
        }
    }

    /// Call the callback with a single message
    pub fn call_single(&self, message: &BatchedMessage) -> Result<(), String> {
        Python::with_gil(|py| {
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

        Python::with_gil(|py| {
            // Convert batch to Python list
            let py_list = PyList::empty(py);

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
#[allow(dead_code)]
#[allow(deprecated)]
fn json_to_python(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_py(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(n.to_string().into_py(py))
            }
        }
        serde_json::Value::String(s) => Ok(s.into_py(py)),
        serde_json::Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = json_to_python(py, item)?;
                py_list.append(py_item.bind(py))?;
            }
            Ok(py_list.into_py(py))
        }
        serde_json::Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                let py_val = json_to_python(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into_py(py))
        }
    }
}

/// IPC handler with message batching support
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
    pub fn on(&self, event: String, callback: PyObject, batching: bool) {
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

impl Default for BatchedHandler {
    fn default() -> Self {
        Self::new()
    }
}
