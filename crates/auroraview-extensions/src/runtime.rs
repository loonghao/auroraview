//! Extension Runtime - Background script execution and message passing
//!
//! Manages the execution of extension background scripts (service workers)
//! and provides the message passing infrastructure for chrome.runtime API.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Message sent between extension components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionMessage {
    /// Unique message ID
    pub id: String,
    /// Source extension ID
    pub source: ExtensionId,
    /// Target extension ID (or None for broadcast)
    pub target: Option<ExtensionId>,
    /// Message type
    pub message_type: MessageType,
    /// Message payload
    pub payload: Value,
    /// Response callback ID (for request-response pattern)
    pub callback_id: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

impl ExtensionMessage {
    /// Create a new message
    pub fn new(source: ExtensionId, payload: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            target: None,
            message_type: MessageType::Message,
            payload,
            callback_id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Create a request message (expects response)
    pub fn request(source: ExtensionId, payload: Value) -> Self {
        let callback_id = uuid::Uuid::new_v4().to_string();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            target: None,
            message_type: MessageType::Request,
            payload,
            callback_id: Some(callback_id),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Create a response to a request
    pub fn response(request: &ExtensionMessage, payload: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: request.target.clone().unwrap_or_default(),
            target: Some(request.source.clone()),
            message_type: MessageType::Response,
            payload,
            callback_id: request.callback_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Set the target extension
    pub fn to(mut self, target: ExtensionId) -> Self {
        self.target = Some(target);
        self
    }
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Regular message (fire and forget)
    Message,
    /// Request (expects response)
    Request,
    /// Response to a request
    Response,
    /// Event notification
    Event,
    /// API call from content script
    ApiCall,
    /// Port message
    PortMessage,
    /// Port connect
    PortConnect,
    /// Port disconnect
    PortDisconnect,
}

/// Message sender information (chrome.runtime.MessageSender)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSender {
    /// Extension ID
    pub id: Option<String>,
    /// Tab that opened the connection
    pub tab: Option<TabInfo>,
    /// Frame ID
    pub frame_id: Option<i32>,
    /// URL of the page/frame
    pub url: Option<String>,
    /// Origin of the page
    pub origin: Option<String>,
    /// Whether from native messaging
    pub native_application: Option<String>,
}

/// Tab information for MessageSender
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabInfo {
    pub id: i32,
    pub index: i32,
    pub window_id: i32,
    pub url: Option<String>,
    pub title: Option<String>,
    pub active: bool,
    pub pinned: bool,
    pub status: String,
}

impl Default for MessageSender {
    fn default() -> Self {
        Self {
            id: None,
            tab: None,
            frame_id: Some(0),
            url: None,
            origin: None,
            native_application: None,
        }
    }
}

/// Port for long-lived connections (chrome.runtime.Port)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    /// Unique port ID
    pub id: String,
    /// Port name
    pub name: String,
    /// Sender information
    pub sender: MessageSender,
    /// Source extension ID
    pub source_extension_id: String,
    /// Target extension ID
    pub target_extension_id: Option<String>,
    /// Whether the port is connected
    pub connected: bool,
    /// Creation timestamp
    pub created_at: i64,
}

impl Port {
    /// Create a new port
    pub fn new(name: &str, source_extension_id: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            sender: MessageSender {
                id: Some(source_extension_id.to_string()),
                ..Default::default()
            },
            source_extension_id: source_extension_id.to_string(),
            target_extension_id: None,
            connected: true,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// Pending response handler
pub struct PendingResponse {
    /// Callback ID
    pub callback_id: String,
    /// Timeout timestamp
    pub timeout_at: i64,
    /// Response handler
    pub handler: Box<dyn FnOnce(Value) + Send + Sync>,
}

/// Extension runtime state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeState {
    /// Not started
    Stopped,
    /// Starting up
    Starting,
    /// Running
    Running,
    /// Suspended (idle)
    Suspended,
    /// Error state
    Error(String),
}

/// Message handler callback type
pub type MessageHandler =
    Box<dyn Fn(&ExtensionMessage, &MessageSender) -> Option<Value> + Send + Sync>;

/// Extension runtime - manages background script execution
pub struct ExtensionRuntime {
    /// Extension ID
    extension_id: ExtensionId,
    /// Current state
    state: Arc<RwLock<RuntimeState>>,
    /// Pending messages
    pending_messages: Arc<RwLock<Vec<ExtensionMessage>>>,
    /// Message handlers
    message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    /// Active ports
    ports: Arc<RwLock<HashMap<String, Port>>>,
    /// Pending responses
    pending_responses: Arc<RwLock<HashMap<String, PendingResponse>>>,
    /// Event listeners
    event_listeners: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ExtensionRuntime {
    /// Create a new runtime for an extension
    pub fn new(extension_id: ExtensionId) -> Self {
        Self {
            extension_id,
            state: Arc::new(RwLock::new(RuntimeState::Stopped)),
            pending_messages: Arc::new(RwLock::new(Vec::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            ports: Arc::new(RwLock::new(HashMap::new())),
            pending_responses: Arc::new(RwLock::new(HashMap::new())),
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the extension ID
    pub fn extension_id(&self) -> &str {
        &self.extension_id
    }

    /// Get the current state
    pub fn state(&self) -> RuntimeState {
        self.state.read().clone()
    }

    /// Start the runtime
    pub fn start(&self) -> ExtensionResult<()> {
        let mut state = self.state.write();
        match &*state {
            RuntimeState::Running => {
                return Ok(()); // Already running
            }
            RuntimeState::Starting => {
                return Ok(()); // Already starting
            }
            _ => {}
        }

        *state = RuntimeState::Starting;

        // In a real implementation, this would:
        // 1. Create a JavaScript execution context
        // 2. Load the service worker script
        // 3. Inject the Chrome API polyfills
        // 4. Start the event loop

        *state = RuntimeState::Running;

        // Process pending messages
        drop(state);
        self.process_pending_messages();

        tracing::info!("Extension runtime started: {}", self.extension_id);
        Ok(())
    }

    /// Stop the runtime
    pub fn stop(&self) -> ExtensionResult<()> {
        let mut state = self.state.write();
        *state = RuntimeState::Stopped;

        // Disconnect all ports
        let mut ports = self.ports.write();
        for port in ports.values_mut() {
            port.connected = false;
        }
        ports.clear();

        tracing::info!("Extension runtime stopped: {}", self.extension_id);
        Ok(())
    }

    /// Send a message to the extension
    pub fn send_message(&self, message: ExtensionMessage) -> ExtensionResult<Option<Value>> {
        let state = self.state.read();
        if *state != RuntimeState::Running {
            // Queue message for later
            let mut pending = self.pending_messages.write();
            pending.push(message);
            return Ok(None);
        }
        drop(state);

        // Dispatch to handlers
        let handlers = self.message_handlers.read();
        let sender = MessageSender {
            id: Some(message.source.clone()),
            ..Default::default()
        };

        for handler in handlers.values() {
            if let Some(response) = handler(&message, &sender) {
                return Ok(Some(response));
            }
        }

        tracing::debug!(
            "Message sent to extension {}: {:?}",
            self.extension_id,
            message.message_type
        );
        Ok(None)
    }

    /// Send a message and wait for response
    pub fn send_message_async(
        &self,
        message: ExtensionMessage,
        timeout_ms: i64,
        handler: impl FnOnce(Value) + Send + Sync + 'static,
    ) -> ExtensionResult<()> {
        let callback_id = message
            .callback_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let pending = PendingResponse {
            callback_id: callback_id.clone(),
            timeout_at: chrono::Utc::now().timestamp_millis() + timeout_ms,
            handler: Box::new(handler),
        };

        self.pending_responses.write().insert(callback_id, pending);
        self.send_message(message)?;

        Ok(())
    }

    /// Handle a response
    pub fn handle_response(&self, callback_id: &str, response: Value) {
        if let Some(pending) = self.pending_responses.write().remove(callback_id) {
            (pending.handler)(response);
        }
    }

    /// Register a message handler
    pub fn on_message<F>(&self, handler_id: &str, handler: F)
    where
        F: Fn(&ExtensionMessage, &MessageSender) -> Option<Value> + Send + Sync + 'static,
    {
        let mut handlers = self.message_handlers.write();
        handlers.insert(handler_id.to_string(), Box::new(handler));
    }

    /// Remove a message handler
    pub fn remove_handler(&self, handler_id: &str) {
        let mut handlers = self.message_handlers.write();
        handlers.remove(handler_id);
    }

    /// Create a port connection
    pub fn connect(&self, name: &str, target_extension_id: Option<&str>) -> Port {
        let mut port = Port::new(name, &self.extension_id);
        port.target_extension_id = target_extension_id.map(String::from);

        let port_id = port.id.clone();
        self.ports.write().insert(port_id, port.clone());

        port
    }

    /// Disconnect a port
    pub fn disconnect(&self, port_id: &str) {
        if let Some(port) = self.ports.write().get_mut(port_id) {
            port.connected = false;
        }
    }

    /// Send a message through a port
    pub fn port_post_message(&self, port_id: &str, message: Value) -> ExtensionResult<()> {
        let ports = self.ports.read();
        let port = ports
            .get(port_id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Port not found: {}", port_id)))?;

        if !port.connected {
            return Err(ExtensionError::Runtime("Port is disconnected".to_string()));
        }

        let ext_message = ExtensionMessage {
            id: uuid::Uuid::new_v4().to_string(),
            source: self.extension_id.clone(),
            target: port.target_extension_id.clone(),
            message_type: MessageType::PortMessage,
            payload: serde_json::json!({
                "portId": port_id,
                "message": message
            }),
            callback_id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        drop(ports);
        self.send_message(ext_message)?;

        Ok(())
    }

    /// Get a port by ID
    pub fn get_port(&self, port_id: &str) -> Option<Port> {
        self.ports.read().get(port_id).cloned()
    }

    /// Add an event listener
    pub fn add_event_listener(&self, event: &str, listener_id: &str) {
        let mut listeners = self.event_listeners.write();
        let event_listeners = listeners.entry(event.to_string()).or_default();
        if !event_listeners.contains(&listener_id.to_string()) {
            event_listeners.push(listener_id.to_string());
        }
    }

    /// Remove an event listener
    pub fn remove_event_listener(&self, event: &str, listener_id: &str) {
        let mut listeners = self.event_listeners.write();
        if let Some(event_listeners) = listeners.get_mut(event) {
            event_listeners.retain(|id| id != listener_id);
        }
    }

    /// Dispatch an event
    pub fn dispatch_event(&self, event: &str, data: Value) -> ExtensionResult<()> {
        let message = ExtensionMessage {
            id: uuid::Uuid::new_v4().to_string(),
            source: self.extension_id.clone(),
            target: None,
            message_type: MessageType::Event,
            payload: serde_json::json!({
                "event": event,
                "data": data
            }),
            callback_id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.send_message(message)?;
        Ok(())
    }

    /// Process pending messages
    fn process_pending_messages(&self) {
        let messages: Vec<ExtensionMessage> = {
            let mut pending = self.pending_messages.write();
            std::mem::take(&mut *pending)
        };

        for message in messages {
            let _ = self.send_message(message);
        }
    }

    /// Clean up expired pending responses
    pub fn cleanup_expired_responses(&self) {
        let now = chrono::Utc::now().timestamp_millis();
        let mut pending = self.pending_responses.write();
        pending.retain(|_, response| response.timeout_at > now);
    }
}

/// Runtime manager - coordinates multiple extension runtimes
pub struct RuntimeManager {
    /// Active runtimes
    runtimes: Arc<RwLock<HashMap<ExtensionId, Arc<ExtensionRuntime>>>>,
}

impl RuntimeManager {
    /// Create a new runtime manager
    pub fn new() -> Self {
        Self {
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a runtime for an extension
    pub fn create_runtime(
        &self,
        extension_id: ExtensionId,
    ) -> ExtensionResult<Arc<ExtensionRuntime>> {
        let mut runtimes = self.runtimes.write();
        if runtimes.contains_key(&extension_id) {
            return Err(ExtensionError::AlreadyLoaded(extension_id));
        }

        let runtime = Arc::new(ExtensionRuntime::new(extension_id.clone()));
        runtimes.insert(extension_id, runtime.clone());
        Ok(runtime)
    }

    /// Get a runtime
    pub fn get_runtime(&self, extension_id: &str) -> Option<Arc<ExtensionRuntime>> {
        self.runtimes.read().get(extension_id).cloned()
    }

    /// Remove a runtime
    pub fn remove_runtime(&self, extension_id: &str) -> Option<Arc<ExtensionRuntime>> {
        self.runtimes.write().remove(extension_id)
    }

    /// Start all runtimes
    pub fn start_all(&self) -> ExtensionResult<()> {
        let runtimes = self.runtimes.read();
        for runtime in runtimes.values() {
            runtime.start()?;
        }
        Ok(())
    }

    /// Stop all runtimes
    pub fn stop_all(&self) -> ExtensionResult<()> {
        let runtimes = self.runtimes.read();
        for runtime in runtimes.values() {
            runtime.stop()?;
        }
        Ok(())
    }

    /// Broadcast a message to all extensions
    pub fn broadcast(&self, message: ExtensionMessage) -> ExtensionResult<()> {
        let runtimes = self.runtimes.read();
        for runtime in runtimes.values() {
            runtime.send_message(message.clone())?;
        }
        Ok(())
    }

    /// Send a message to a specific extension
    pub fn send_to(
        &self,
        extension_id: &str,
        message: ExtensionMessage,
    ) -> ExtensionResult<Option<Value>> {
        let runtimes = self.runtimes.read();
        let runtime = runtimes
            .get(extension_id)
            .ok_or_else(|| ExtensionError::NotFound(extension_id.to_string()))?;
        runtime.send_message(message)
    }

    /// Get all extension IDs
    pub fn extension_ids(&self) -> Vec<ExtensionId> {
        self.runtimes.read().keys().cloned().collect()
    }

    /// Dispatch an event to all extensions
    pub fn dispatch_event(&self, event: &str, data: Value) -> ExtensionResult<()> {
        let runtimes = self.runtimes.read();
        for runtime in runtimes.values() {
            runtime.dispatch_event(event, data.clone())?;
        }
        Ok(())
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared runtime manager
pub type SharedRuntimeManager = Arc<RuntimeManager>;

/// Create a shared runtime manager
pub fn create_runtime_manager() -> SharedRuntimeManager {
    Arc::new(RuntimeManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = ExtensionMessage::new("test-ext".to_string(), serde_json::json!({"test": true}));
        assert_eq!(msg.source, "test-ext");
        assert_eq!(msg.message_type, MessageType::Message);
    }

    #[test]
    fn test_request_response() {
        let req = ExtensionMessage::request(
            "test-ext".to_string(),
            serde_json::json!({"action": "test"}),
        );
        assert_eq!(req.message_type, MessageType::Request);
        assert!(req.callback_id.is_some());

        let resp = ExtensionMessage::response(&req, serde_json::json!({"result": "ok"}));
        assert_eq!(resp.message_type, MessageType::Response);
        assert_eq!(resp.callback_id, req.callback_id);
    }

    #[test]
    fn test_runtime_lifecycle() {
        let runtime = ExtensionRuntime::new("test-ext".to_string());
        assert_eq!(runtime.state(), RuntimeState::Stopped);

        runtime.start().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Running);

        runtime.stop().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Stopped);
    }

    #[test]
    fn test_port_connection() {
        let runtime = ExtensionRuntime::new("test-ext".to_string());
        runtime.start().unwrap();

        let port = runtime.connect("test-port", None);
        assert!(port.connected);
        assert_eq!(port.name, "test-port");

        runtime.disconnect(&port.id);
        let updated_port = runtime.get_port(&port.id).unwrap();
        assert!(!updated_port.connected);
    }
}
