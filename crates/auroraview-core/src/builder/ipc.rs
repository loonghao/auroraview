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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_message_type_parsing() {
        assert_eq!(IpcMessageType::parse("event"), IpcMessageType::Event);
        assert_eq!(IpcMessageType::parse("call"), IpcMessageType::Call);
        assert_eq!(IpcMessageType::parse("invoke"), IpcMessageType::Invoke);
        assert_eq!(
            IpcMessageType::parse("js_callback_result"),
            IpcMessageType::JsCallbackResult
        );
        assert!(matches!(
            IpcMessageType::parse("unknown"),
            IpcMessageType::Unknown(_)
        ));
    }

    #[test]
    fn test_message_type_unknown_preserves_value() {
        if let IpcMessageType::Unknown(val) = IpcMessageType::parse("custom_type") {
            assert_eq!(val, "custom_type");
        } else {
            panic!("Expected Unknown variant");
        }
    }

    #[test]
    fn test_parse_event_message() {
        let body = r#"{"type":"event","event":"click","detail":{"x":100,"y":200}}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Event);
        assert_eq!(parsed.name, Some("click".to_string()));
        assert_eq!(parsed.data["x"], 100);
        assert_eq!(parsed.data["y"], 200);
        assert!(parsed.id.is_none());
    }

    #[test]
    fn test_parse_event_without_detail() {
        let body = r#"{"type":"event","event":"ready"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Event);
        assert_eq!(parsed.name, Some("ready".to_string()));
        assert!(parsed.data.is_null());
    }

    #[test]
    fn test_parse_call_message() {
        let body = r#"{"type":"call","method":"api.echo","params":{"msg":"hello"},"id":"123"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Call);
        assert_eq!(parsed.name, Some("api.echo".to_string()));
        assert_eq!(parsed.data["msg"], "hello");
        assert_eq!(parsed.id, Some("123".to_string()));
    }

    #[test]
    fn test_parse_call_without_params() {
        let body = r#"{"type":"call","method":"api.ping","id":"1"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Call);
        assert_eq!(parsed.name, Some("api.ping".to_string()));
        assert!(parsed.data.is_null());
        assert_eq!(parsed.id, Some("1".to_string()));
    }

    #[test]
    fn test_parse_call_without_id() {
        let body = r#"{"type":"call","method":"api.fire_and_forget","params":{}}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Call);
        assert!(parsed.id.is_none());
    }

    #[test]
    fn test_parse_invoke_message() {
        let body = r#"{"type":"invoke","cmd":"plugin.test","args":{"value":42},"id":"456"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Invoke);
        assert_eq!(parsed.name, Some("plugin.test".to_string()));
        assert_eq!(parsed.data["value"], 42);
        assert_eq!(parsed.id, Some("456".to_string()));
    }

    #[test]
    fn test_parse_invoke_without_args() {
        let body = r#"{"type":"invoke","cmd":"plugin.init","id":"1"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::Invoke);
        assert_eq!(parsed.name, Some("plugin.init".to_string()));
        assert!(parsed.data.is_object());
    }

    #[test]
    fn test_parse_js_callback_result() {
        let body = r#"{"type":"js_callback_result","callback_id":789,"result":"success"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::JsCallbackResult);
        assert_eq!(parsed.name, Some("789".to_string()));
        assert_eq!(parsed.data["result"], "success");
    }

    #[test]
    fn test_parse_js_callback_with_error() {
        let body = r#"{"type":"js_callback_result","callback_id":100,"error":"failed"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.msg_type, IpcMessageType::JsCallbackResult);
        assert_eq!(parsed.name, Some("100".to_string()));
        assert_eq!(parsed.data["error"], "failed");
    }

    #[test]
    fn test_parse_js_callback_with_both() {
        let body = r#"{"type":"js_callback_result","callback_id":50,"result":"ok","error":"warn"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.data["result"], "ok");
        assert_eq!(parsed.data["error"], "warn");
    }

    #[test]
    fn test_parse_unknown_type() {
        let body = r#"{"type":"custom"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert!(matches!(parsed.msg_type, IpcMessageType::Unknown(_)));
        assert!(parsed.name.is_none());
        assert!(parsed.data.is_null());
    }

    #[test]
    fn test_parse_invalid_json() {
        let body = "not valid json";
        assert!(IpcMessageHandler::parse(body).is_none());
    }

    #[test]
    fn test_parse_missing_type() {
        let body = r#"{"event":"click"}"#;
        assert!(IpcMessageHandler::parse(body).is_none());
    }

    #[test]
    fn test_parse_null_type() {
        let body = r#"{"type":null}"#;
        assert!(IpcMessageHandler::parse(body).is_none());
    }

    #[test]
    fn test_parse_numeric_type() {
        let body = r#"{"type":123}"#;
        assert!(IpcMessageHandler::parse(body).is_none());
    }

    #[test]
    fn test_parsed_message_raw_field() {
        let body = r#"{"type":"event","event":"test","extra":"data"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();

        assert_eq!(parsed.raw["extra"], "data");
    }

    #[test]
    fn test_handler_new_and_handle() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let handler = IpcMessageHandler::new(move |_msg| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Handle a valid message
        handler.handle(r#"{"type":"event","event":"test"}"#);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Handle another valid message
        handler.handle(r#"{"type":"call","method":"api.test","id":"1"}"#);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_handler_handle_invalid_message() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let handler = IpcMessageHandler::new(move |_msg| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Invalid JSON should not trigger callback
        handler.handle("invalid json");
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        // Missing type should not trigger callback
        handler.handle(r#"{"event":"test"}"#);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_handler_callback_receives_correct_data() {
        let received = Arc::new(std::sync::Mutex::new(None));
        let received_clone = received.clone();

        let handler = IpcMessageHandler::new(move |msg| {
            *received_clone.lock().unwrap() = Some(msg);
        });

        handler.handle(r#"{"type":"call","method":"api.echo","params":{"value":42},"id":"abc"}"#);

        let msg = received.lock().unwrap().take().unwrap();
        assert_eq!(msg.msg_type, IpcMessageType::Call);
        assert_eq!(msg.name, Some("api.echo".to_string()));
        assert_eq!(msg.data["value"], 42);
        assert_eq!(msg.id, Some("abc".to_string()));
    }

    #[test]
    fn test_ipc_message_type_debug() {
        // Test Debug implementation
        let event = IpcMessageType::Event;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Event"));

        let unknown = IpcMessageType::Unknown("test".to_string());
        let debug_str = format!("{:?}", unknown);
        assert!(debug_str.contains("Unknown"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_ipc_message_type_clone() {
        let original = IpcMessageType::Unknown("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_parsed_message_clone() {
        let body = r#"{"type":"event","event":"test","detail":{"x":1}}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();
        let cloned = parsed.clone();

        assert_eq!(parsed.msg_type, cloned.msg_type);
        assert_eq!(parsed.name, cloned.name);
        assert_eq!(parsed.data, cloned.data);
        assert_eq!(parsed.id, cloned.id);
    }

    #[test]
    fn test_parsed_message_debug() {
        let body = r#"{"type":"event","event":"test"}"#;
        let parsed = IpcMessageHandler::parse(body).unwrap();
        let debug_str = format!("{:?}", parsed);

        assert!(debug_str.contains("ParsedIpcMessage"));
        assert!(debug_str.contains("Event"));
    }
}
