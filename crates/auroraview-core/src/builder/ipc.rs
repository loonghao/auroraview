//! Shared IPC message handler for WebView
//!
//! This module provides a reusable IPC message parser and router
//! that can be used in both standalone and DCC embedded modes.

use std::sync::Arc;

/// IPC message types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcMessageType {
    /// Event from JavaScript
    Event,
    /// Method call from JavaScript
    Call,
    /// Plugin invoke from JavaScript
    Invoke,
    /// JavaScript callback result
    JsCallbackResult,
    /// Unknown message type
    Unknown(String),
}

impl IpcMessageType {
    /// Parse message type from string
    pub fn parse(s: &str) -> Self {
        match s {
            "event" => Self::Event,
            "call" => Self::Call,
            "invoke" => Self::Invoke,
            "js_callback_result" => Self::JsCallbackResult,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// Parsed IPC message
#[derive(Debug, Clone)]
pub struct ParsedIpcMessage {
    /// Message type
    pub msg_type: IpcMessageType,
    /// Event name (for Event type) or method name (for Call type)
    pub name: Option<String>,
    /// Message data/params
    pub data: serde_json::Value,
    /// Message ID (for Call type)
    pub id: Option<String>,
    /// Raw message for custom processing
    pub raw: serde_json::Value,
}

/// Callback type for handling parsed IPC messages
pub type IpcCallback = Arc<dyn Fn(ParsedIpcMessage) + Send + Sync>;

/// Shared IPC message handler
///
/// This handler parses raw IPC messages from wry and converts them
/// to a structured format for further processing.
pub struct IpcMessageHandler {
    callback: IpcCallback,
}

impl IpcMessageHandler {
    /// Create a new IPC message handler with a callback
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(ParsedIpcMessage) + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(callback),
        }
    }

    /// Parse and handle an IPC message
    pub fn handle(&self, body: &str) {
        if let Some(parsed) = Self::parse(body) {
            (self.callback)(parsed);
        }
    }

    /// Parse an IPC message body
    pub fn parse(body: &str) -> Option<ParsedIpcMessage> {
        let message: serde_json::Value = serde_json::from_str(body).ok()?;

        let msg_type_str = message.get("type").and_then(|v| v.as_str())?;
        let msg_type = IpcMessageType::parse(msg_type_str);

        let (name, data, id) = match msg_type {
            IpcMessageType::Event => {
                let event_name = message
                    .get("event")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let detail = message
                    .get("detail")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                (event_name, detail, None)
            }
            IpcMessageType::Call => {
                let method = message
                    .get("method")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let params = message
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let call_id = message.get("id").and_then(|v| v.as_str()).map(String::from);
                (method, params, call_id)
            }
            IpcMessageType::Invoke => {
                let cmd = message
                    .get("cmd")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let args = message
                    .get("args")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let invoke_id = message.get("id").and_then(|v| v.as_str()).map(String::from);
                (cmd, args, invoke_id)
            }
            IpcMessageType::JsCallbackResult => {
                let callback_id = message
                    .get("callback_id")
                    .and_then(|v| v.as_u64())
                    .map(|id| id.to_string());
                let result = message.get("result").cloned();
                let error = message.get("error").cloned();

                let mut data = serde_json::Map::new();
                if let Some(r) = result {
                    data.insert("result".to_string(), r);
                }
                if let Some(e) = error {
                    data.insert("error".to_string(), e);
                }

                (callback_id, serde_json::Value::Object(data), None)
            }
            IpcMessageType::Unknown(_) => (None, serde_json::Value::Null, None),
        };

        Some(ParsedIpcMessage {
            msg_type,
            name,
            data,
            id,
            raw: message,
        })
    }

    /// Create a handler function for wry's with_ipc_handler
    ///
    /// Returns a closure that can be passed to `WebViewBuilder::with_ipc_handler`
    pub fn into_handler(self) -> impl Fn(wry::http::Request<String>) + 'static {
        let callback = self.callback;

        move |request: wry::http::Request<String>| {
            let body = request.body();
            tracing::debug!("[IpcMessageHandler] Received: {}", body);

            if let Some(parsed) = Self::parse(body) {
                callback(parsed);
            }
        }
    }
}
