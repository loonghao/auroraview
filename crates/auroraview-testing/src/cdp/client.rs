//! CDP client trait definition

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{InspectorError, Result};

/// CDP target info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    /// Target ID
    pub id: String,
    /// Target type (page, iframe, worker, etc.)
    #[serde(rename = "type")]
    pub target_type: String,
    /// Target title
    pub title: String,
    /// Target URL
    pub url: String,
    /// WebSocket debugger URL
    #[serde(default)]
    pub web_socket_debugger_url: String,
    /// DevTools frontend URL
    #[serde(default)]
    pub devtools_frontend_url: String,
    /// Favicon URL
    #[serde(default)]
    pub favicon_url: String,
    /// Whether target is attached
    #[serde(default)]
    pub attached: bool,
}

/// CDP client trait
#[async_trait]
pub trait CdpClient: Send + Sync {
    /// Send CDP command and get response
    async fn send(&self, method: &str, params: Value) -> Result<Value>;

    /// Send CDP command without params
    async fn send_simple(&self, method: &str) -> Result<Value> {
        self.send(method, Value::Null).await
    }

    /// Get page targets
    async fn targets(&self) -> Result<Vec<TargetInfo>>;

    /// Get current page info
    async fn current_target(&self) -> Result<Option<TargetInfo>>;

    /// Close connection
    async fn close(&self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    // === Convenience methods ===

    /// Navigate to URL
    async fn navigate(&self, url: &str) -> Result<Value> {
        self.send("Page.navigate", serde_json::json!({ "url": url }))
            .await
    }

    /// Reload page
    async fn reload(&self) -> Result<Value> {
        self.send_simple("Page.reload").await
    }

    /// Go back
    async fn go_back(&self) -> Result<Value> {
        // Get navigation history first
        let history = self.send_simple("Page.getNavigationHistory").await?;
        let current_index = history["currentIndex"].as_i64().unwrap_or(0);

        if current_index > 0 {
            let entries = history["entries"].as_array();
            if let Some(entries) = entries {
                if let Some(entry) = entries.get((current_index - 1) as usize) {
                    if let Some(id) = entry["id"].as_i64() {
                        return self
                            .send(
                                "Page.navigateToHistoryEntry",
                                serde_json::json!({ "entryId": id }),
                            )
                            .await;
                    }
                }
            }
        }

        Err(InspectorError::Navigation("Cannot go back".to_string()))
    }

    /// Go forward
    async fn go_forward(&self) -> Result<Value> {
        let history = self.send_simple("Page.getNavigationHistory").await?;
        let current_index = history["currentIndex"].as_i64().unwrap_or(0);
        let entries = history["entries"].as_array();

        if let Some(entries) = entries {
            if let Some(entry) = entries.get((current_index + 1) as usize) {
                if let Some(id) = entry["id"].as_i64() {
                    return self
                        .send(
                            "Page.navigateToHistoryEntry",
                            serde_json::json!({ "entryId": id }),
                        )
                        .await;
                }
            }
        }

        Err(InspectorError::Navigation("Cannot go forward".to_string()))
    }

    /// Evaluate JavaScript
    async fn evaluate(&self, expression: &str) -> Result<Value> {
        let result = self
            .send(
                "Runtime.evaluate",
                serde_json::json!({
                    "expression": expression,
                    "returnByValue": true
                }),
            )
            .await?;

        // Check for exception
        if let Some(exception) = result.get("exceptionDetails") {
            let text = exception["text"].as_str().unwrap_or("JavaScript error");
            return Err(InspectorError::JavaScript(text.to_string()));
        }

        Ok(result.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Take screenshot
    async fn screenshot(&self, format: &str) -> Result<Vec<u8>> {
        let result = self
            .send(
                "Page.captureScreenshot",
                serde_json::json!({ "format": format }),
            )
            .await?;

        let data = result["data"]
            .as_str()
            .ok_or_else(|| InspectorError::Screenshot("No data in response".to_string()))?;

        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| InspectorError::Screenshot(e.to_string()))
    }

    /// Get accessibility tree
    async fn get_accessibility_tree(&self) -> Result<Value> {
        // Enable accessibility domain if not already
        let _ = self.send_simple("Accessibility.enable").await;

        self.send_simple("Accessibility.getFullAXTree").await
    }

    /// Get document root node
    async fn get_document(&self) -> Result<Value> {
        self.send("DOM.getDocument", serde_json::json!({ "depth": -1 }))
            .await
    }

    /// Click element by backend node ID
    async fn click_node(&self, backend_node_id: i64) -> Result<()> {
        // Get box model for the node
        let box_model = self
            .send(
                "DOM.getBoxModel",
                serde_json::json!({ "backendNodeId": backend_node_id }),
            )
            .await?;

        let content = box_model["model"]["content"]
            .as_array()
            .ok_or_else(|| InspectorError::ElementNotFound("No box model".to_string()))?;

        // Calculate center point
        let x = (content[0].as_f64().unwrap_or(0.0) + content[2].as_f64().unwrap_or(0.0)) / 2.0;
        let y = (content[1].as_f64().unwrap_or(0.0) + content[5].as_f64().unwrap_or(0.0)) / 2.0;

        // Dispatch mouse events
        self.send(
            "Input.dispatchMouseEvent",
            serde_json::json!({
                "type": "mousePressed",
                "x": x,
                "y": y,
                "button": "left",
                "clickCount": 1
            }),
        )
        .await?;

        self.send(
            "Input.dispatchMouseEvent",
            serde_json::json!({
                "type": "mouseReleased",
                "x": x,
                "y": y,
                "button": "left",
                "clickCount": 1
            }),
        )
        .await?;

        Ok(())
    }

    /// Focus element by backend node ID
    async fn focus_node(&self, backend_node_id: i64) -> Result<()> {
        self.send(
            "DOM.focus",
            serde_json::json!({ "backendNodeId": backend_node_id }),
        )
        .await?;
        Ok(())
    }

    /// Type text into focused element
    async fn type_text(&self, text: &str) -> Result<()> {
        for c in text.chars() {
            self.send(
                "Input.dispatchKeyEvent",
                serde_json::json!({
                    "type": "keyDown",
                    "text": c.to_string()
                }),
            )
            .await?;

            self.send(
                "Input.dispatchKeyEvent",
                serde_json::json!({
                    "type": "keyUp",
                    "text": c.to_string()
                }),
            )
            .await?;
        }
        Ok(())
    }

    /// Press a key
    async fn press_key(&self, key: &str) -> Result<()> {
        // Map common key names
        let (key_code, code) = match key.to_lowercase().as_str() {
            "enter" | "return" => (13, "Enter"),
            "tab" => (9, "Tab"),
            "escape" | "esc" => (27, "Escape"),
            "backspace" => (8, "Backspace"),
            "delete" => (46, "Delete"),
            "arrowup" | "up" => (38, "ArrowUp"),
            "arrowdown" | "down" => (40, "ArrowDown"),
            "arrowleft" | "left" => (37, "ArrowLeft"),
            "arrowright" | "right" => (39, "ArrowRight"),
            "space" | " " => (32, "Space"),
            _ => (0, key),
        };

        self.send(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": "keyDown",
                "key": key,
                "code": code,
                "windowsVirtualKeyCode": key_code,
                "nativeVirtualKeyCode": key_code
            }),
        )
        .await?;

        self.send(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": "keyUp",
                "key": key,
                "code": code,
                "windowsVirtualKeyCode": key_code,
                "nativeVirtualKeyCode": key_code
            }),
        )
        .await?;

        Ok(())
    }

    /// Scroll page
    async fn scroll(&self, delta_x: i32, delta_y: i32) -> Result<()> {
        // Get viewport size first
        let layout = self.send_simple("Page.getLayoutMetrics").await?;
        let x = layout["layoutViewport"]["clientWidth"]
            .as_f64()
            .unwrap_or(640.0)
            / 2.0;
        let y = layout["layoutViewport"]["clientHeight"]
            .as_f64()
            .unwrap_or(480.0)
            / 2.0;

        self.send(
            "Input.dispatchMouseEvent",
            serde_json::json!({
                "type": "mouseWheel",
                "x": x,
                "y": y,
                "deltaX": delta_x,
                "deltaY": delta_y
            }),
        )
        .await?;

        Ok(())
    }
}
