//! Async IPC Handler using Tokio
//!
//! This module provides an asynchronous IPC handler that processes messages
//! in a background tokio runtime, preventing UI thread blocking.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │  UI Thread      │────▶│  Tokio Runtime   │────▶│  Python/JS      │
//! │  (non-blocking) │     │  (background)    │     │  Callbacks      │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//! ```

use crossbeam_channel::{bounded, Sender};
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Async callback type
pub type AsyncCallback = Arc<dyn Fn(serde_json::Value) + Send + Sync>;

/// Async message for IPC processing
#[derive(Debug, Clone)]
pub struct AsyncIpcMessage {
    /// Event name
    pub event: String,
    /// Event data as JSON
    pub data: serde_json::Value,
    /// Optional response channel for request-response pattern
    pub response_tx: Option<Sender<serde_json::Value>>,
}

/// Configuration for async IPC handler
#[derive(Debug, Clone)]
pub struct AsyncIpcConfig {
    /// Number of worker threads in tokio runtime
    pub worker_threads: usize,
    /// Channel capacity for message queue
    pub channel_capacity: usize,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for AsyncIpcConfig {
    fn default() -> Self {
        Self {
            worker_threads: 2,
            channel_capacity: 1000,
            debug: false,
        }
    }
}

/// Async IPC handler that processes messages in a background tokio runtime
///
/// This handler receives messages from the UI thread and processes them
/// asynchronously, preventing UI blocking during IPC operations.
pub struct AsyncIpcHandler {
    /// Sender for submitting messages (UI thread -> tokio runtime)
    tx: mpsc::Sender<AsyncIpcMessage>,
    /// Flag indicating if the handler is running
    running: Arc<AtomicBool>,
    /// Handle to the background thread
    _thread_handle: Option<thread::JoinHandle<()>>,
    /// Registered callbacks for events
    callbacks: Arc<DashMap<String, Vec<AsyncCallback>>>,
}

impl AsyncIpcHandler {
    /// Create a new async IPC handler with default configuration
    pub fn new() -> Self {
        Self::with_config(AsyncIpcConfig::default())
    }

    /// Create a new async IPC handler with custom configuration
    pub fn with_config(config: AsyncIpcConfig) -> Self {
        let (tx, mut rx) = mpsc::channel::<AsyncIpcMessage>(config.channel_capacity);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let debug = config.debug;
        let callbacks: Arc<DashMap<String, Vec<AsyncCallback>>> = Arc::new(DashMap::new());
        let callbacks_clone = callbacks.clone();

        // Spawn background thread with tokio runtime
        let thread_handle = thread::spawn(move || {
            let rt = match Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("[AsyncIpcHandler] Failed to create tokio runtime: {}", e);
                    return;
                }
            };

            rt.block_on(async move {
                tracing::info!("[AsyncIpcHandler] Background runtime started");

                while running_clone.load(Ordering::SeqCst) {
                    // Use tokio::select! for efficient async waiting
                    tokio::select! {
                        Some(msg) = rx.recv() => {
                            if debug {
                                tracing::debug!(
                                    "[AsyncIpcHandler] Processing event: {}",
                                    msg.event
                                );
                            }
                            // Process message asynchronously
                            Self::process_message_async(msg, &callbacks_clone).await;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                            // Periodic check for shutdown
                        }
                    }
                }

                tracing::info!("[AsyncIpcHandler] Background runtime stopped");
            });
        });

        Self {
            tx,
            running,
            _thread_handle: Some(thread_handle),
            callbacks,
        }
    }

    /// Register a callback for an event
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        self.callbacks
            .entry(event.to_string())
            .or_default()
            .push(Arc::new(callback));
        tracing::debug!("[AsyncIpcHandler] Registered callback for event: {}", event);
    }

    /// Remove all callbacks for an event
    #[allow(dead_code)]
    pub fn off(&self, event: &str) {
        self.callbacks.remove(event);
    }

    /// Submit a message for async processing (non-blocking)
    pub fn submit(&self, event: String, data: serde_json::Value) -> Result<(), String> {
        let msg = AsyncIpcMessage {
            event: event.clone(),
            data,
            response_tx: None,
        };

        self.tx
            .try_send(msg)
            .map_err(|e| format!("Failed to submit message for event {}: {}", event, e))
    }

    /// Submit a message and wait for response (request-response pattern)
    pub fn submit_with_response(
        &self,
        event: String,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let (response_tx, response_rx) = bounded::<serde_json::Value>(1);
        let msg = AsyncIpcMessage {
            event: event.clone(),
            data,
            response_tx: Some(response_tx),
        };

        self.tx
            .try_send(msg)
            .map_err(|e| format!("Failed to submit message for event {}: {}", event, e))?;

        // Wait for response with timeout
        response_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| format!("Timeout waiting for response: {}", e))
    }

    /// Process a message asynchronously
    async fn process_message_async(
        msg: AsyncIpcMessage,
        callbacks: &Arc<DashMap<String, Vec<AsyncCallback>>>,
    ) {
        // Call registered callbacks
        if let Some(event_callbacks) = callbacks.get(&msg.event) {
            for callback in event_callbacks.value() {
                callback(msg.data.clone());
            }
            tracing::debug!(
                "[AsyncIpcHandler] Processed event '{}' with {} callbacks",
                msg.event,
                event_callbacks.value().len()
            );
        } else {
            tracing::debug!(
                "[AsyncIpcHandler] No callbacks registered for event: {}",
                msg.event
            );
        }

        // Send response if requested
        if let Some(response_tx) = msg.response_tx {
            let _ = response_tx.send(serde_json::json!({"status": "ok"}));
        }
    }

    /// Check if the handler is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Stop the async handler
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("[AsyncIpcHandler] Stop requested");
    }
}

impl Default for AsyncIpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AsyncIpcHandler {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_async_handler_creation() {
        let handler = AsyncIpcHandler::new();
        assert!(handler.is_running());
        handler.stop();
    }

    #[test]
    fn test_async_handler_callback_registration() {
        let handler = AsyncIpcHandler::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        handler.on("test_event", move |_data| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Submit a message and wait for response
        let result = handler.submit_with_response(
            "test_event".to_string(),
            serde_json::json!({"test": "data"}),
        );

        assert!(result.is_ok());

        // Give some time for the callback to be processed
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Callback should have been called
        assert!(call_count.load(Ordering::SeqCst) >= 1);

        handler.stop();
    }

    #[test]
    fn test_async_handler_submit() {
        let handler = AsyncIpcHandler::new();

        // Submit should succeed
        let result = handler.submit(
            "test_event".to_string(),
            serde_json::json!({"data": "test"}),
        );
        assert!(result.is_ok());

        handler.stop();
    }
}
