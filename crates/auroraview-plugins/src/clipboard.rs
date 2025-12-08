//! Clipboard Plugin
//!
//! Provides system clipboard access from JavaScript.
//!
//! ## Commands
//!
//! - `read_text` - Read text from clipboard
//! - `write_text` - Write text to clipboard
//! - `clear` - Clear clipboard contents
//!
//! ## Example
//!
//! ```javascript
//! // Read text from clipboard
//! const text = await auroraview.invoke("plugin:clipboard|read_text");
//!
//! // Write text to clipboard
//! await auroraview.invoke("plugin:clipboard|write_text", { text: "Hello!" });
//! ```

use crate::{PluginError, PluginHandler, PluginResult, ScopeConfig};
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;

/// Clipboard plugin
pub struct ClipboardPlugin {
    name: String,
    clipboard: Mutex<Option<Clipboard>>,
}

impl ClipboardPlugin {
    /// Create a new clipboard plugin
    pub fn new() -> Self {
        Self {
            name: "clipboard".to_string(),
            clipboard: Mutex::new(None),
        }
    }

    fn get_clipboard(&self) -> PluginResult<std::sync::MutexGuard<'_, Option<Clipboard>>> {
        let mut guard = self
            .clipboard
            .lock()
            .map_err(|_| PluginError::clipboard_error("Failed to acquire clipboard lock"))?;

        if guard.is_none() {
            *guard = Some(Clipboard::new().map_err(|e| {
                PluginError::clipboard_error(format!("Failed to access clipboard: {}", e))
            })?);
        }

        Ok(guard)
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for writing text to clipboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextOptions {
    /// Text to write
    pub text: String,
}

impl PluginHandler for ClipboardPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "read_text" => {
                let mut guard = self.get_clipboard()?;
                let clipboard = guard.as_mut().unwrap();

                let text = clipboard.get_text().map_err(|e| {
                    PluginError::clipboard_error(format!("Failed to read clipboard: {}", e))
                })?;

                Ok(serde_json::json!({ "text": text }))
            }
            "write_text" => {
                let opts: WriteTextOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut guard = self.get_clipboard()?;
                let clipboard = guard.as_mut().unwrap();

                clipboard.set_text(&opts.text).map_err(|e| {
                    PluginError::clipboard_error(format!("Failed to write clipboard: {}", e))
                })?;

                Ok(serde_json::json!({ "success": true }))
            }
            "clear" => {
                let mut guard = self.get_clipboard()?;
                let clipboard = guard.as_mut().unwrap();

                clipboard.clear().map_err(|e| {
                    PluginError::clipboard_error(format!("Failed to clear clipboard: {}", e))
                })?;

                Ok(serde_json::json!({ "success": true }))
            }
            "has_text" => {
                let mut guard = self.get_clipboard()?;
                let clipboard = guard.as_mut().unwrap();

                // Try to get text, if it succeeds and is not empty, we have text
                let has_text = clipboard.get_text().map(|t| !t.is_empty()).unwrap_or(false);

                Ok(serde_json::json!({ "hasText": has_text }))
            }
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec!["read_text", "write_text", "clear", "has_text"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_plugin_commands() {
        let plugin = ClipboardPlugin::new();
        let commands = plugin.commands();
        assert!(commands.contains(&"read_text"));
        assert!(commands.contains(&"write_text"));
        assert!(commands.contains(&"clear"));
        assert!(commands.contains(&"has_text"));
    }

    // Note: Clipboard tests require a display server on Linux
    // and may not work in headless CI environments
    #[test]
    #[ignore = "Requires display server"]
    fn test_clipboard_write_read() {
        let plugin = ClipboardPlugin::new();
        let scope = ScopeConfig::new();

        // Write text
        let write_result = plugin.handle(
            "write_text",
            serde_json::json!({ "text": "Test clipboard content" }),
            &scope,
        );
        assert!(write_result.is_ok());

        // Read text
        let read_result = plugin.handle("read_text", serde_json::json!({}), &scope);
        assert!(read_result.is_ok());
        let data = read_result.unwrap();
        assert_eq!(data["text"], "Test clipboard content");
    }
}
