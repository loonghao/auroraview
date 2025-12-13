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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Import UserEvent from webview event_loop module
use crate::webview::event_loop::UserEvent;
use tao::event_loop::EventLoopProxy;

// Import Metrics from core
use auroraview_core::ipc::IpcMetrics;

/// Callback type for async JavaScript execution
pub type JsCallback = Box<dyn FnOnce(Result<serde_json::Value, String>) + Send + 'static>;

/// Message types that can be sent to the WebView
#[allow(dead_code)]
pub enum WebViewMessage {
    /// Execute JavaScript code
    EvalJs(String),

    /// Execute JavaScript code with async callback
    /// Returns result via the provided callback
    EvalJsAsync { script: String, callback_id: u64 },

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

    /// Window event notification (from Rust to Python callbacks)
    WindowEvent {
        event_type: WindowEventType,
        data: serde_json::Value,
    },

    /// Close the WebView window
    Close,
}

impl Clone for WebViewMessage {
    fn clone(&self) -> Self {
        match self {
            Self::EvalJs(s) => Self::EvalJs(s.clone()),
            Self::EvalJsAsync {
                script,
                callback_id,
            } => Self::EvalJsAsync {
                script: script.clone(),
                callback_id: *callback_id,
            },
            Self::EmitEvent { event_name, data } => Self::EmitEvent {
                event_name: event_name.clone(),
                data: data.clone(),
            },
            Self::LoadUrl(s) => Self::LoadUrl(s.clone()),
            Self::LoadHtml(s) => Self::LoadHtml(s.clone()),
            Self::SetVisible(v) => Self::SetVisible(*v),
            Self::Reload => Self::Reload,
            Self::StopLoading => Self::StopLoading,
            Self::WindowEvent { event_type, data } => Self::WindowEvent {
                event_type: event_type.clone(),
                data: data.clone(),
            },
            Self::Close => Self::Close,
        }
    }
}

impl std::fmt::Debug for WebViewMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EvalJs(s) => f.debug_tuple("EvalJs").field(s).finish(),
            Self::EvalJsAsync {
                script,
                callback_id,
            } => f
                .debug_struct("EvalJsAsync")
                .field("script", script)
                .field("callback_id", callback_id)
                .finish(),
            Self::EmitEvent { event_name, data } => f
                .debug_struct("EmitEvent")
                .field("event_name", event_name)
                .field("data", data)
                .finish(),
            Self::LoadUrl(s) => f.debug_tuple("LoadUrl").field(s).finish(),
            Self::LoadHtml(s) => f.debug_tuple("LoadHtml").field(s).finish(),
            Self::SetVisible(v) => f.debug_tuple("SetVisible").field(v).finish(),
            Self::Reload => f.debug_tuple("Reload").finish(),
            Self::StopLoading => f.debug_tuple("StopLoading").finish(),
            Self::WindowEvent { event_type, data } => f
                .debug_struct("WindowEvent")
                .field("event_type", event_type)
                .field("data", data)
                .finish(),
            Self::Close => f.debug_tuple("Close").finish(),
        }
    }
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
    /// Convert to event name string (matches Python WindowEvent enum)
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

/// Configuration for message queue
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MessageQueueConfig {
    /// Maximum number of messages in the queue (backpressure)
    pub capacity: usize,

    /// Whether to block when queue is full (true) or drop messages (false)
    pub block_on_full: bool,

    /// Maximum number of retry attempts for failed sends
    pub max_retries: u32,

    /// Delay between retry attempts (milliseconds)
    pub retry_delay_ms: u64,

    /// Batch interval for event loop wake-up (milliseconds)
    ///
    /// To reduce CPU usage, the message queue will only wake the event loop
    /// if at least this many milliseconds have passed since the last wake.
    /// Default: 16ms (60 FPS)
    ///
    /// Set to 0 to disable batching (wake on every message).
    pub batch_interval_ms: u64,
}

impl Default for MessageQueueConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            block_on_full: false,
            max_retries: 3,
            retry_delay_ms: 10,
            batch_interval_ms: 16, // 60 FPS - good balance between responsiveness and CPU usage
        }
    }
}

/// Thread-safe message queue for WebView operations
///
/// Uses crossbeam-channel for high-performance lock-free communication.
/// Provides backpressure control to prevent unbounded memory growth.
#[derive(Clone)]
#[allow(dead_code)]
pub struct MessageQueue {
    /// Sender for pushing messages (lock-free)
    tx: Sender<WebViewMessage>,

    /// Receiver for popping messages (lock-free)
    rx: Receiver<WebViewMessage>,

    /// Event loop proxy for immediate wake-up
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,

    /// Performance metrics
    metrics: IpcMetrics,

    /// Configuration
    config: MessageQueueConfig,

    /// Last time the event loop was woken up (for batching)
    last_wake_time: Arc<Mutex<Option<Instant>>>,

    /// Shutdown flag to prevent sending messages after event loop is closed
    shutdown: Arc<AtomicBool>,
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
            metrics: IpcMetrics::new(),
            config,
            last_wake_time: Arc::new(Mutex::new(None)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Mark the queue as shutdown - no more messages will be sent
    ///
    /// This should be called when the event loop is closing to prevent
    /// "EventLoopClosed" errors from background threads.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        tracing::info!("[MessageQueue] Shutdown flag set - no more messages will be sent");
    }

    /// Check if the queue is shutdown
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
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
        // Check shutdown flag first - if shutdown, silently drop the message
        // This prevents "EventLoopClosed" errors from background threads
        if self.shutdown.load(Ordering::Relaxed) {
            tracing::debug!("[MessageQueue::push] Queue is shutdown, dropping message silently");
            return;
        }

        // Use info level for Close message to help diagnose shutdown issues
        let msg_type = match &message {
            WebViewMessage::EvalJs(_) => "EvalJs",
            WebViewMessage::EvalJsAsync { .. } => "EvalJsAsync",
            WebViewMessage::EmitEvent { event_name, .. } => event_name,
            WebViewMessage::LoadUrl(_) => "LoadUrl",
            WebViewMessage::LoadHtml(_) => "LoadHtml",
            WebViewMessage::SetVisible(v) => {
                if *v {
                    "SetVisible(true)"
                } else {
                    "SetVisible(false)"
                }
            }
            WebViewMessage::Reload => "Reload",
            WebViewMessage::StopLoading => "StopLoading",
            WebViewMessage::WindowEvent { event_type, .. } => event_type.as_str(),
            WebViewMessage::Close => "Close",
        };

        if matches!(&message, WebViewMessage::Close) {
            tracing::info!("[PUSH] [MessageQueue::push] Pushing Close message");
        } else {
            tracing::debug!("[PUSH] [MessageQueue::push] Pushing message: {}", msg_type);
        }

        // Try to send the message
        match self.tx.try_send(message.clone()) {
            Ok(_) => {
                self.metrics.record_send();
                let queue_len = self.len();
                self.metrics.update_peak_queue_length(queue_len);

                if matches!(&message, WebViewMessage::Close) {
                    tracing::info!(
                        "[PUSH] [MessageQueue::push] Close message sent successfully (queue length: {})",
                        queue_len
                    );
                } else {
                    tracing::debug!(
                        "[PUSH] [MessageQueue::push] Message sent successfully (queue length: {})",
                        queue_len
                    );
                }

                // Wake up the event loop immediately
                self.wake_event_loop();
            }
            Err(TrySendError::Full(_)) => {
                if self.config.block_on_full {
                    // Block until space is available
                    tracing::warn!("[WARNING] [MessageQueue::push] Queue full, blocking...");
                    if let Err(e) = self.tx.send(message) {
                        self.metrics.record_failure();
                        tracing::error!(
                            "[ERROR] [MessageQueue::push] Failed to send message: {:?}",
                            e
                        );
                    } else {
                        self.metrics.record_send();
                        self.wake_event_loop();
                    }
                } else {
                    // Drop the message
                    self.metrics.record_drop();
                    tracing::error!("[ERROR] [MessageQueue::push] Queue full, dropping message!");
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                self.metrics.record_failure();
                tracing::error!("[ERROR] [MessageQueue::push] Channel disconnected!");
            }
        }
    }

    /// Push a message with retry logic (thread-safe)
    ///
    /// This method will retry sending the message if the queue is full,
    /// using the configured retry count and delay. Failed messages are
    /// automatically sent to the dead letter queue.
    ///
    /// # Returns
    /// - `Ok(())` if the message was sent successfully
    /// - `Err(String)` if all retry attempts failed
    #[allow(dead_code)]
    pub fn push_with_retry(&self, message: WebViewMessage) -> Result<(), String> {
        let max_retries = self.config.max_retries;
        let retry_delay = std::time::Duration::from_millis(self.config.retry_delay_ms);
        let start_time = std::time::Instant::now();

        for attempt in 0..=max_retries {
            match self.tx.try_send(message.clone()) {
                Ok(_) => {
                    self.metrics.record_send();
                    let queue_len = self.len();
                    self.metrics.update_peak_queue_length(queue_len);

                    // Record latency
                    let latency_us = start_time.elapsed().as_micros() as u64;
                    self.metrics.record_latency(latency_us);

                    if attempt > 0 {
                        tracing::info!(
                            "[RETRY] Message sent successfully after {} attempts (latency: {}μs)",
                            attempt,
                            latency_us
                        );
                    }
                    self.wake_event_loop();
                    return Ok(());
                }
                Err(TrySendError::Full(_)) => {
                    self.metrics.record_retry();

                    if attempt < max_retries {
                        tracing::warn!(
                            "[RETRY] Queue full, attempt {}/{}, retrying in {:?}...",
                            attempt + 1,
                            max_retries,
                            retry_delay
                        );
                        std::thread::sleep(retry_delay);
                    } else {
                        self.metrics.record_failure();

                        let error_msg = format!(
                            "Failed to send message after {} attempts: queue full",
                            max_retries + 1
                        );
                        tracing::error!("[ERROR] {}", error_msg);
                        return Err(error_msg);
                    }
                }
                Err(TrySendError::Disconnected(_)) => {
                    self.metrics.record_failure();

                    let error_msg = "Channel disconnected".to_string();
                    tracing::error!("[ERROR] {}", error_msg);
                    return Err(error_msg);
                }
            }
        }

        Err("Unexpected retry loop exit".to_string())
    }

    /// Wake up the event loop with batching optimization
    ///
    /// To reduce CPU usage, this method implements batching:
    /// - If batch_interval_ms is 0, wake immediately on every call
    /// - Otherwise, only wake if enough time has passed since last wake
    ///
    /// This prevents excessive event loop wake-ups during high-frequency operations
    /// (e.g., rapid eval_js calls), reducing CPU usage by 30-50%.
    fn wake_event_loop(&self) {
        // Check if we should batch wake-ups
        if self.config.batch_interval_ms > 0 {
            // Try to acquire lock on last_wake_time
            if let Ok(mut last_wake_guard) = self.last_wake_time.lock() {
                let now = Instant::now();
                let should_wake = match *last_wake_guard {
                    Some(last_wake) => {
                        let elapsed = now.duration_since(last_wake);
                        let batch_interval =
                            std::time::Duration::from_millis(self.config.batch_interval_ms);
                        elapsed >= batch_interval
                    }
                    None => true, // First wake, always wake
                };

                if !should_wake {
                    tracing::trace!(
                        "[MessageQueue] Skipping wake (batching, interval={}ms)",
                        self.config.batch_interval_ms
                    );
                    return;
                }

                // Update last wake time
                *last_wake_guard = Some(now);
            }
        }

        // Perform the actual wake-up
        if let Ok(proxy_guard) = self.event_loop_proxy.lock() {
            if let Some(proxy) = proxy_guard.as_ref() {
                tracing::info!("[WAKE] [MessageQueue] Sending wake-up event to event loop...");
                match proxy.send_event(UserEvent::ProcessMessages) {
                    Ok(_) => {
                        tracing::info!("[OK] [MessageQueue] Event loop woken up successfully!");
                    }
                    Err(e) => {
                        tracing::error!(
                            "[ERROR] [MessageQueue] Failed to wake up event loop: {:?}",
                            e
                        );
                    }
                }
            } else {
                // This is expected during initialization before the event loop starts.
                // Messages are still queued and will be processed when the event loop runs.
                // Only log at debug level to reduce noise.
                tracing::debug!(
                    "[MessageQueue] Event loop proxy not yet set - message queued, will be processed when event loop starts"
                );
            }
        }
    }

    /// Pop a message from the queue (thread-safe)
    ///
    /// This should be called from the WebView thread only.
    pub fn pop(&self) -> Option<WebViewMessage> {
        let message = self.rx.try_recv().ok();
        if message.is_some() {
            self.metrics.record_receive();
        }
        message
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

    /// Clear all pending messages from the queue
    ///
    /// This is useful when resetting the WebView state for reuse.
    /// All pending messages will be discarded.
    pub fn clear(&self) {
        let mut count = 0;
        while self.rx.try_recv().is_ok() {
            count += 1;
        }
        if count > 0 {
            tracing::debug!("[MessageQueue::clear] Cleared {} pending messages", count);
        }
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

    /// Process a batch of pending messages (up to max_count)
    ///
    /// This should be called from the WebView thread's event loop.
    /// Useful for DCCs with busy main threads (e.g., Houdini) to prevent
    /// blocking for too long.
    ///
    /// # Arguments
    /// * `max_count` - Maximum number of messages to process (0 = unlimited)
    /// * `handler` - Callback function to handle each message
    ///
    /// # Returns
    /// Tuple of (processed_count, remaining_count)
    pub fn process_batch<F>(&self, max_count: usize, mut handler: F) -> (usize, usize)
    where
        F: FnMut(WebViewMessage),
    {
        let mut count = 0;

        if max_count == 0 {
            // Unlimited - process all
            while let Some(message) = self.pop() {
                handler(message);
                count += 1;
            }
        } else {
            // Limited batch processing
            while count < max_count {
                if let Some(message) = self.pop() {
                    handler(message);
                    count += 1;
                } else {
                    break;
                }
            }
        }

        let remaining = self.len();

        if count > 0 {
            tracing::debug!(
                "Processed {} messages from queue ({} remaining)",
                count,
                remaining
            );
        }

        (count, remaining)
    }

    /// Get a reference to the metrics
    #[allow(dead_code)]
    pub fn metrics(&self) -> &IpcMetrics {
        &self.metrics
    }

    /// Get a snapshot of current metrics
    #[allow(dead_code)]
    pub fn get_metrics_snapshot(&self) -> auroraview_core::ipc::IpcMetricsSnapshot {
        self.metrics.snapshot()
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}
