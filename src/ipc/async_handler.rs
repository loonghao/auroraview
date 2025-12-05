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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

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
                            Self::process_message_async(msg).await;
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
        }
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
    async fn process_message_async(msg: AsyncIpcMessage) {
        // TODO: Call registered callbacks here
        // For now, just log the message
        tracing::debug!(
            "[AsyncIpcHandler] Processed event: {} (data: {})",
            msg.event,
            msg.data
        );

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
