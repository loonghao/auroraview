//! Action system for AI agent browser control
//!
//! Actions are functions that the AI agent can call to interact with
//! the browser, perform searches, navigate, and more.

mod registry;

pub use registry::*;

use crate::error::AIError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Action execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub success: bool,
    /// Result data
    pub data: Option<Value>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ActionResult {
    /// Create successful result
    pub fn ok(data: impl Serialize) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).ok(),
            error: None,
        }
    }

    /// Create error result
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }

    /// Create empty success result
    pub fn empty() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }
}

/// Context provided to action execution
#[derive(Debug, Clone, Default)]
pub struct ActionContext {
    /// Current URL (if in browser context)
    pub current_url: Option<String>,
    /// Page title
    pub page_title: Option<String>,
    /// Custom context data
    pub data: Value,
}

impl ActionContext {
    /// Create new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set current URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.current_url = Some(url.into());
        self
    }

    /// Set page title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.page_title = Some(title.into());
        self
    }

    /// Set custom data
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }
}

/// Action trait for defining custom actions
pub trait Action: Send + Sync {
    /// Action name (used in tool calls)
    fn name(&self) -> &str;

    /// Action description (shown to AI)
    fn description(&self) -> &str;

    /// Parameter schema (JSON Schema)
    fn parameters(&self) -> Value;

    /// Execute the action
    fn execute(&self, args: Value, ctx: &ActionContext) -> Result<ActionResult, AIError>;
}

/// Browser navigation action
pub struct NavigateAction;

impl Action for NavigateAction {
    fn name(&self) -> &str {
        "navigate"
    }

    fn description(&self) -> &str {
        "Navigate to a URL in the browser"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to"
                }
            },
            "required": ["url"]
        })
    }

    fn execute(&self, args: Value, _ctx: &ActionContext) -> Result<ActionResult, AIError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AIError::ActionExecutionFailed("Missing 'url' parameter".into()))?;

        // In a real implementation, this would call the WebView
        Ok(ActionResult::ok(serde_json::json!({
            "navigated_to": url
        })))
    }
}

/// Search action
pub struct SearchAction;

impl Action for SearchAction {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search the web using a search engine"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "engine": {
                    "type": "string",
                    "description": "Search engine to use (google, bing, duckduckgo)",
                    "default": "google"
                }
            },
            "required": ["query"]
        })
    }

    fn execute(&self, args: Value, _ctx: &ActionContext) -> Result<ActionResult, AIError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AIError::ActionExecutionFailed("Missing 'query' parameter".into()))?;

        let engine = args
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("google");

        let search_url = match engine {
            "bing" => format!(
                "https://www.bing.com/search?q={}",
                urlencoding::encode(query)
            ),
            "duckduckgo" => format!("https://duckduckgo.com/?q={}", urlencoding::encode(query)),
            _ => format!(
                "https://www.google.com/search?q={}",
                urlencoding::encode(query)
            ),
        };

        Ok(ActionResult::ok(serde_json::json!({
            "search_url": search_url,
            "query": query,
            "engine": engine
        })))
    }
}

/// Click element action
pub struct ClickAction;

impl Action for ClickAction {
    fn name(&self) -> &str {
        "click"
    }

    fn description(&self) -> &str {
        "Click on an element on the page"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the element to click"
                },
                "text": {
                    "type": "string",
                    "description": "Text content of the element to click (alternative to selector)"
                }
            }
        })
    }

    fn execute(&self, args: Value, _ctx: &ActionContext) -> Result<ActionResult, AIError> {
        let selector = args.get("selector").and_then(|v| v.as_str());
        let text = args.get("text").and_then(|v| v.as_str());

        if selector.is_none() && text.is_none() {
            return Err(AIError::ActionExecutionFailed(
                "Either 'selector' or 'text' parameter is required".into(),
            ));
        }

        Ok(ActionResult::ok(serde_json::json!({
            "clicked": true,
            "selector": selector,
            "text": text
        })))
    }
}

/// Type text action
pub struct TypeAction;

impl Action for TypeAction {
    fn name(&self) -> &str {
        "type_text"
    }

    fn description(&self) -> &str {
        "Type text into an input field"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the input element"
                },
                "text": {
                    "type": "string",
                    "description": "Text to type"
                },
                "clear": {
                    "type": "boolean",
                    "description": "Whether to clear the input first",
                    "default": false
                }
            },
            "required": ["selector", "text"]
        })
    }

    fn execute(&self, args: Value, _ctx: &ActionContext) -> Result<ActionResult, AIError> {
        let selector = args
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AIError::ActionExecutionFailed("Missing 'selector' parameter".into()))?;

        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AIError::ActionExecutionFailed("Missing 'text' parameter".into()))?;

        let clear = args.get("clear").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(ActionResult::ok(serde_json::json!({
            "typed": true,
            "selector": selector,
            "text": text,
            "cleared": clear
        })))
    }
}

/// Screenshot action
pub struct ScreenshotAction;

impl Action for ScreenshotAction {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> &str {
        "Take a screenshot of the current page"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "full_page": {
                    "type": "boolean",
                    "description": "Whether to capture the full page or just the viewport",
                    "default": false
                }
            }
        })
    }

    fn execute(&self, args: Value, _ctx: &ActionContext) -> Result<ActionResult, AIError> {
        let full_page = args
            .get("full_page")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // In real implementation, this would capture and return base64 image
        Ok(ActionResult::ok(serde_json::json!({
            "screenshot_taken": true,
            "full_page": full_page
        })))
    }
}

/// Scroll action
pub struct ScrollAction;

impl Action for ScrollAction {
    fn name(&self) -> &str {
        "scroll"
    }

    fn description(&self) -> &str {
        "Scroll the page"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "direction": {
                    "type": "string",
                    "enum": ["up", "down", "left", "right"],
                    "description": "Direction to scroll"
                },
                "amount": {
                    "type": "integer",
                    "description": "Amount to scroll in pixels",
                    "default": 300
                }
            },
            "required": ["direction"]
        })
    }

    fn execute(&self, args: Value, _ctx: &ActionContext) -> Result<ActionResult, AIError> {
        let direction = args
            .get("direction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AIError::ActionExecutionFailed("Missing 'direction' parameter".into())
            })?;

        let amount = args.get("amount").and_then(|v| v.as_i64()).unwrap_or(300);

        Ok(ActionResult::ok(serde_json::json!({
            "scrolled": true,
            "direction": direction,
            "amount": amount
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigate_action() {
        let action = NavigateAction;
        let args = serde_json::json!({"url": "https://example.com"});
        let ctx = ActionContext::default();

        let result = action.execute(args, &ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_search_action() {
        let action = SearchAction;
        let args = serde_json::json!({"query": "rust programming"});
        let ctx = ActionContext::default();

        let result = action.execute(args, &ctx).unwrap();
        assert!(result.success);

        let data = result.data.unwrap();
        assert!(data["search_url"].as_str().unwrap().contains("google.com"));
    }
}
