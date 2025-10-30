//! Thread-safe message queue for cross-thread WebView communication
//!
//! This module provides a message queue system that allows safe communication
//! between the DCC main thread (e.g., Maya) and the WebView background thread.
//!
//! ## Problem
//! WryWebView is not Send/Sync, so we cannot call evaluate_script() from
//! a different thread than the one that created the WebView.
//!
//! ## Solution
//! Use a message queue with crossbeam-channel for high-performance communication:
//! 1. Main thread calls emit() -> pushes message to queue
//! 2. Background thread's event loop polls queue -> executes JavaScript
//!
//! This ensures all WebView operations happen on the correct thread.

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::sync::{Arc, Mutex};

// Import UserEvent from webview event_loop module
use crate::webview::event_loop::UserEvent;
use tao::event_loop::EventLoopProxy;

/// Message types that can be sent to the WebView
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WebViewMessage {
    /// Execute JavaScript code
    EvalJs(String),

    /// Emit an event to JavaScript
    EmitEvent {
        event_name: String,
        data: serde_json::Value,
    },

    /// Load a URL
    LoadUrl(String),

    /// Load HTML content
    LoadHtml(String),
}

/// Configuration for message queue
#[derive(Debug, Clone)]
pub struct MessageQueueConfig {
    /// Maximum number of messages in the queue (backpressure)
    pub capacity: usize,

    /// Whether to block when queue is full (true) or drop messages (false)
    pub block_on_full: bool,
}

impl Default for MessageQueueConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            block_on_full: false,
        }
    }
}

/// Thread-safe message queue for WebView operations
///
/// Uses crossbeam-channel for high-performance lock-free communication.
/// Provides backpressure control to prevent unbounded memory growth.
#[derive(Clone)]
pub struct MessageQueue {
    /// Sender for pushing messages (lock-free)
    tx: Sender<WebViewMessage>,

    /// Receiver for popping messages (lock-free)
    rx: Receiver<WebViewMessage>,

    /// Event loop proxy for immediate wake-up
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,

    /// Configuration
    config: MessageQueueConfig,
}

impl MessageQueue {
    /// Create a new message queue with default configuration
    pub fn new() -> Self {
        Self::with_config(MessageQueueConfig::default())
    }

    /// Create a new message queue with custom configuration
    pub fn with_config(config: MessageQueueConfig) -> Self {
        let (tx, rx) = bounded(config.capacity);
        Self {
            tx,
            rx,
            event_loop_proxy: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Set the event loop proxy for immediate wake-up
    pub fn set_event_loop_proxy(&self, proxy: EventLoopProxy<UserEvent>) {
        if let Ok(mut proxy_guard) = self.event_loop_proxy.lock() {
            *proxy_guard = Some(proxy);
            tracing::info!("Event loop proxy set in message queue");
        }
    }

    /// Push a message to the queue (thread-safe)
    ///
    /// This can be called from any thread, including the DCC main thread.
    /// After pushing the message, it will wake up the event loop immediately.
    ///
    /// # Backpressure
    /// - If `block_on_full` is true, this will block until space is available
    /// - If `block_on_full` is false, this will drop the message and log an error
    pub fn push(&self, message: WebViewMessage) {
        tracing::debug!(
            "🔵 [MessageQueue::push] Pushing message: {:?}",
            match &message {
                WebViewMessage::EvalJs(_) => "EvalJs",
                WebViewMessage::EmitEvent { event_name, .. } => event_name,
                WebViewMessage::LoadUrl(_) => "LoadUrl",
                WebViewMessage::LoadHtml(_) => "LoadHtml",
            }
        );

        // Try to send the message
        match self.tx.try_send(message.clone()) {
            Ok(_) => {
                tracing::debug!(
                    "🔵 [MessageQueue::push] Message sent successfully (queue length: {})",
                    self.len()
                );

                // Wake up the event loop immediately
                self.wake_event_loop();
            }
            Err(TrySendError::Full(_)) => {
                if self.config.block_on_full {
                    // Block until space is available
                    tracing::warn!("⚠️ [MessageQueue::push] Queue full, blocking...");
                    if let Err(e) = self.tx.send(message) {
                        tracing::error!("❌ [MessageQueue::push] Failed to send message: {:?}", e);
                    } else {
                        self.wake_event_loop();
                    }
                } else {
                    // Drop the message
                    tracing::error!("❌ [MessageQueue::push] Queue full, dropping message!");
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                tracing::error!("❌ [MessageQueue::push] Channel disconnected!");
            }
        }
    }

    /// Wake up the event loop
    fn wake_event_loop(&self) {
        if let Ok(proxy_guard) = self.event_loop_proxy.lock() {
            if let Some(proxy) = proxy_guard.as_ref() {
                tracing::debug!("🔵 [MessageQueue] Sending wake-up event...");
                match proxy.send_event(UserEvent::ProcessMessages) {
                    Ok(_) => {
                        tracing::debug!("✅ [MessageQueue] Event loop woken up successfully!");
                    }
                    Err(e) => {
                        tracing::error!("❌ [MessageQueue] Failed to wake up event loop: {:?}", e);
                    }
                }
            } else {
                tracing::debug!(
                    "⚠️ [MessageQueue] Event loop proxy is None - cannot wake up event loop!"
                );
            }
        }
    }

    /// Pop a message from the queue (thread-safe)
    ///
    /// This should be called from the WebView thread only.
    pub fn pop(&self) -> Option<WebViewMessage> {
        self.rx.try_recv().ok()
    }

    /// Check if the queue is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    /// Get the number of pending messages
    pub fn len(&self) -> usize {
        self.rx.len()
    }

    /// Process all pending messages
    ///
    /// This should be called from the WebView thread's event loop.
    /// Returns the number of messages processed.
    pub fn process_all<F>(&self, mut handler: F) -> usize
    where
        F: FnMut(WebViewMessage),
    {
        let mut count = 0;

        while let Some(message) = self.pop() {
            handler(message);
            count += 1;
        }

        if count > 0 {
            tracing::debug!("Processed {} messages from queue", count);
        }

        count
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}
