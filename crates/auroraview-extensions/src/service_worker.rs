//! Service Worker Simulation for Chrome Extensions
//!
//! Provides a simulation layer for Chrome Extension Service Workers
//! in the AuroraView environment. Since WebView2 doesn't support
//! extension service workers natively, we simulate the lifecycle
//! and message passing.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Service Worker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ServiceWorkerState {
    /// Worker is not running
    #[default]
    Stopped,
    /// Worker is starting up
    Starting,
    /// Worker is running and active
    Running,
    /// Worker is idle (may be terminated soon)
    Idle,
    /// Worker encountered an error
    Error,
}

/// Service Worker registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceWorkerRegistration {
    /// Extension ID this worker belongs to
    pub extension_id: String,
    /// Path to the service worker script
    pub script_path: String,
    /// Current state
    pub state: ServiceWorkerState,
    /// Last active timestamp
    pub last_active: Option<i64>,
    /// Error message if in error state
    pub error: Option<String>,
}

/// Message to be processed by service worker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceWorkerMessage {
    /// Unique message ID
    pub id: String,
    /// Message type
    pub message_type: ServiceWorkerMessageType,
    /// Message payload
    pub payload: serde_json::Value,
    /// Sender info
    pub sender: Option<MessageSender>,
    /// Timestamp
    pub timestamp: i64,
}

/// Types of messages that can be sent to service worker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceWorkerMessageType {
    /// Runtime message from extension page
    RuntimeMessage,
    /// Runtime message from external extension
    RuntimeMessageExternal,
    /// Tab message
    TabMessage,
    /// Alarm triggered
    Alarm,
    /// Context menu clicked
    ContextMenuClicked,
    /// Notification clicked
    NotificationClicked,
    /// Action clicked
    ActionClicked,
    /// Command triggered
    Command,
    /// Install/update event
    Installed,
    /// Startup event
    Startup,
    /// Custom event
    Custom(String),
}

/// Message sender information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSender {
    /// Extension ID
    pub id: Option<String>,
    /// Tab ID if from a tab
    pub tab: Option<TabInfo>,
    /// Frame ID
    pub frame_id: Option<i32>,
    /// URL of the sender
    pub url: Option<String>,
    /// Origin of the sender
    pub origin: Option<String>,
}

/// Tab information for message sender
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabInfo {
    pub id: i32,
    pub url: Option<String>,
    pub title: Option<String>,
}

/// Pending response for async message handling
pub struct PendingResponse {
    /// Message ID
    pub message_id: String,
    /// Callback to invoke with response
    pub callback: Box<dyn FnOnce(serde_json::Value) + Send + Sync>,
    /// Timeout timestamp
    pub timeout_at: i64,
}

impl std::fmt::Debug for PendingResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingResponse")
            .field("message_id", &self.message_id)
            .field("callback", &"<callback>")
            .field("timeout_at", &self.timeout_at)
            .finish()
    }
}

/// Service Worker Manager
///
/// Manages the lifecycle and message passing for extension service workers.
pub struct ServiceWorkerManager {
    /// Registered workers
    workers: DashMap<String, ServiceWorkerRegistration>,
    /// Message queue per extension
    message_queues: DashMap<String, Vec<ServiceWorkerMessage>>,
    /// Event listeners per extension
    event_listeners: DashMap<String, HashMap<String, Vec<String>>>,
}

impl ServiceWorkerManager {
    /// Create a new service worker manager
    pub fn new() -> Self {
        Self {
            workers: DashMap::new(),
            message_queues: DashMap::new(),
            event_listeners: DashMap::new(),
        }
    }

    /// Register a service worker for an extension
    pub fn register(
        &self,
        extension_id: &str,
        script_path: &str,
    ) -> Result<ServiceWorkerRegistration, String> {
        let registration = ServiceWorkerRegistration {
            extension_id: extension_id.to_string(),
            script_path: script_path.to_string(),
            state: ServiceWorkerState::Stopped,
            last_active: None,
            error: None,
        };

        self.workers
            .insert(extension_id.to_string(), registration.clone());

        // Initialize message queue
        self.message_queues
            .insert(extension_id.to_string(), Vec::new());

        Ok(registration)
    }

    /// Unregister a service worker
    pub fn unregister(&self, extension_id: &str) -> bool {
        let removed = self.workers.remove(extension_id).is_some();

        if removed {
            self.message_queues.remove(extension_id);
            self.event_listeners.remove(extension_id);
        }

        removed
    }

    /// Start a service worker
    pub fn start(&self, extension_id: &str) -> Result<(), String> {
        let mut worker = self
            .workers
            .get_mut(extension_id)
            .ok_or_else(|| format!("Service worker not registered: {}", extension_id))?;

        match worker.state {
            ServiceWorkerState::Running => {
                // Already running
                Ok(())
            }
            ServiceWorkerState::Error => {
                // Clear error and restart
                worker.error = None;
                worker.state = ServiceWorkerState::Starting;
                Ok(())
            }
            _ => {
                worker.state = ServiceWorkerState::Starting;
                Ok(())
            }
        }
    }

    /// Mark a service worker as running
    pub fn set_running(&self, extension_id: &str) {
        if let Some(mut worker) = self.workers.get_mut(extension_id) {
            worker.state = ServiceWorkerState::Running;
            worker.last_active = Some(chrono::Utc::now().timestamp_millis());
        }
    }

    /// Stop a service worker
    pub fn stop(&self, extension_id: &str) -> Result<(), String> {
        let mut worker = self
            .workers
            .get_mut(extension_id)
            .ok_or_else(|| format!("Service worker not registered: {}", extension_id))?;

        worker.state = ServiceWorkerState::Stopped;
        worker.last_active = Some(chrono::Utc::now().timestamp_millis());

        Ok(())
    }

    /// Set service worker to error state
    pub fn set_error(&self, extension_id: &str, error: &str) {
        if let Some(mut worker) = self.workers.get_mut(extension_id) {
            worker.state = ServiceWorkerState::Error;
            worker.error = Some(error.to_string());
        }
    }

    /// Get service worker state
    pub fn get_state(&self, extension_id: &str) -> Option<ServiceWorkerState> {
        self.workers.get(extension_id).map(|w| w.state)
    }

    /// Get service worker registration
    pub fn get_registration(&self, extension_id: &str) -> Option<ServiceWorkerRegistration> {
        self.workers.get(extension_id).map(|w| w.clone())
    }

    /// Queue a message for the service worker
    pub fn queue_message(&self, extension_id: &str, message: ServiceWorkerMessage) {
        if let Some(mut queue) = self.message_queues.get_mut(extension_id) {
            queue.push(message);
        }
    }

    /// Get and clear pending messages
    pub fn drain_messages(&self, extension_id: &str) -> Vec<ServiceWorkerMessage> {
        if let Some(mut queue) = self.message_queues.get_mut(extension_id) {
            std::mem::take(queue.value_mut())
        } else {
            Vec::new()
        }
    }

    /// Send a runtime message to the service worker
    pub fn send_message(
        &self,
        extension_id: &str,
        message: serde_json::Value,
        sender: Option<MessageSender>,
    ) -> String {
        let message_id = uuid::Uuid::new_v4().to_string();

        let sw_message = ServiceWorkerMessage {
            id: message_id.clone(),
            message_type: ServiceWorkerMessageType::RuntimeMessage,
            payload: message,
            sender,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.queue_message(extension_id, sw_message);

        // Ensure worker is running
        let _ = self.start(extension_id);

        message_id
    }

    /// Dispatch an event to the service worker
    pub fn dispatch_event(
        &self,
        extension_id: &str,
        event_type: ServiceWorkerMessageType,
        payload: serde_json::Value,
    ) {
        let message = ServiceWorkerMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: event_type,
            payload,
            sender: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.queue_message(extension_id, message);

        // Ensure worker is running
        let _ = self.start(extension_id);
    }

    /// Register an event listener
    pub fn add_event_listener(&self, extension_id: &str, event: &str, listener_id: &str) {
        let mut ext_listeners = self
            .event_listeners
            .entry(extension_id.to_string())
            .or_default();
        let event_listeners = ext_listeners.entry(event.to_string()).or_default();

        if !event_listeners.contains(&listener_id.to_string()) {
            event_listeners.push(listener_id.to_string());
        }
    }

    /// Remove an event listener
    pub fn remove_event_listener(&self, extension_id: &str, event: &str, listener_id: &str) {
        if let Some(mut ext_listeners) = self.event_listeners.get_mut(extension_id) {
            if let Some(event_listeners) = ext_listeners.get_mut(event) {
                event_listeners.retain(|id| id != listener_id);
            }
        }
    }

    /// Check if there are listeners for an event
    pub fn has_listeners(&self, extension_id: &str, event: &str) -> bool {
        self.event_listeners
            .get(extension_id)
            .and_then(|ext| ext.get(event).map(|l| !l.is_empty()))
            .unwrap_or(false)
    }

    /// Update last active timestamp
    pub fn touch(&self, extension_id: &str) {
        if let Some(mut worker) = self.workers.get_mut(extension_id) {
            worker.last_active = Some(chrono::Utc::now().timestamp_millis());
            if worker.state == ServiceWorkerState::Idle {
                worker.state = ServiceWorkerState::Running;
            }
        }
    }

    /// Get all registered workers
    pub fn list_workers(&self) -> Vec<ServiceWorkerRegistration> {
        self.workers.iter().map(|r| r.value().clone()).collect()
    }

    /// Check for idle workers that should be stopped
    pub fn check_idle_workers(&self, idle_timeout_ms: i64) -> Vec<String> {
        let now = chrono::Utc::now().timestamp_millis();
        let mut to_stop = Vec::new();

        for entry in self.workers.iter() {
            let (id, worker) = entry.pair();
            if worker.state == ServiceWorkerState::Running
                || worker.state == ServiceWorkerState::Idle
            {
                if let Some(last_active) = worker.last_active {
                    if now - last_active > idle_timeout_ms {
                        to_stop.push(id.clone());
                    }
                }
            }
        }

        to_stop
    }
}

impl Default for ServiceWorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared service worker manager
pub type SharedServiceWorkerManager = Arc<ServiceWorkerManager>;

/// Create a shared service worker manager
pub fn create_service_worker_manager() -> SharedServiceWorkerManager {
    Arc::new(ServiceWorkerManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_worker() {
        let manager = ServiceWorkerManager::new();
        let result = manager.register("test-ext", "background.js");
        assert!(result.is_ok());

        let reg = result.unwrap();
        assert_eq!(reg.extension_id, "test-ext");
        assert_eq!(reg.state, ServiceWorkerState::Stopped);
    }

    #[test]
    fn test_start_stop_worker() {
        let manager = ServiceWorkerManager::new();
        manager.register("test-ext", "background.js").unwrap();

        assert!(manager.start("test-ext").is_ok());
        assert_eq!(
            manager.get_state("test-ext"),
            Some(ServiceWorkerState::Starting)
        );

        manager.set_running("test-ext");
        assert_eq!(
            manager.get_state("test-ext"),
            Some(ServiceWorkerState::Running)
        );

        assert!(manager.stop("test-ext").is_ok());
        assert_eq!(
            manager.get_state("test-ext"),
            Some(ServiceWorkerState::Stopped)
        );
    }

    #[test]
    fn test_message_queue() {
        let manager = ServiceWorkerManager::new();
        manager.register("test-ext", "background.js").unwrap();

        let msg_id = manager.send_message("test-ext", serde_json::json!({"test": true}), None);
        assert!(!msg_id.is_empty());

        let messages = manager.drain_messages("test-ext");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].payload, serde_json::json!({"test": true}));

        // Queue should be empty now
        let messages = manager.drain_messages("test-ext");
        assert!(messages.is_empty());
    }
}
