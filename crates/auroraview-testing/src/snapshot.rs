//! Snapshot types for page state representation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Page snapshot with accessibility tree and interactive element refs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Page title
    pub title: String,
    /// Page URL
    pub url: String,
    /// Viewport dimensions (width, height)
    pub viewport: (u32, u32),
    /// Interactive element refs (@1, @2, ...)
    pub refs: HashMap<String, RefInfo>,
    /// Accessibility tree as formatted text
    pub tree: String,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(title: String, url: String, viewport: (u32, u32)) -> Self {
        Self {
            title,
            url,
            viewport,
            refs: HashMap::new(),
            tree: String::new(),
        }
    }

    /// Get ref count
    pub fn ref_count(&self) -> usize {
        self.refs.len()
    }

    /// Find refs containing text (case-insensitive)
    pub fn find(&self, text: &str) -> Vec<&RefInfo> {
        let text_lower = text.to_lowercase();
        self.refs
            .values()
            .filter(|r| {
                r.name.to_lowercase().contains(&text_lower)
                    || r.description.to_lowercase().contains(&text_lower)
            })
            .collect()
    }

    /// Get ref by ID (accepts "@3" or "3")
    pub fn get_ref(&self, id: &str) -> Option<&RefInfo> {
        let normalized = if id.starts_with('@') {
            id.to_string()
        } else {
            format!("@{}", id)
        };
        self.refs.get(&normalized)
    }

    /// Format as AI-friendly text
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("Page: \"{}\" ({})\n", self.title, self.url));
        output.push_str(&format!(
            "Viewport: {}x{}\n\n",
            self.viewport.0, self.viewport.1
        ));

        // Interactive elements
        output.push_str(&format!(
            "Interactive Elements ({} refs):\n",
            self.refs.len()
        ));

        // Sort refs by numeric ID
        let mut sorted_refs: Vec<_> = self.refs.values().collect();
        sorted_refs.sort_by_key(|r| {
            r.ref_id
                .trim_start_matches('@')
                .parse::<u32>()
                .unwrap_or(u32::MAX)
        });

        for ref_info in sorted_refs {
            let desc = if ref_info.description.is_empty() {
                String::new()
            } else {
                format!(" - {}", ref_info.description)
            };
            output.push_str(&format!(
                "  {}  [{}] \"{}\"{}\n",
                ref_info.ref_id, ref_info.role, ref_info.name, desc
            ));
        }

        // Page structure
        if !self.tree.is_empty() {
            output.push_str("\nPage Structure:\n");
            for line in self.tree.lines() {
                output.push_str(&format!("  {}\n", line));
            }
        }

        output
    }

    /// Format as JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Format for refs only (minimal)
    pub fn to_refs_text(&self) -> String {
        let mut output = String::new();
        let mut sorted_refs: Vec<_> = self.refs.values().collect();
        sorted_refs.sort_by_key(|r| {
            r.ref_id
                .trim_start_matches('@')
                .parse::<u32>()
                .unwrap_or(u32::MAX)
        });

        for ref_info in sorted_refs {
            output.push_str(&format!("{}\n", ref_info));
        }
        output
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

/// Element reference info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefInfo {
    /// Ref ID (e.g., "@1")
    pub ref_id: String,
    /// ARIA role
    pub role: String,
    /// Accessible name
    pub name: String,
    /// Short description
    pub description: String,
    /// CSS selector for precise targeting
    pub selector: String,
    /// Backend node ID for CDP operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<i64>,
    /// Bounding box (x, y, width, height)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<(f64, f64, f64, f64)>,
}

impl RefInfo {
    /// Create a new ref info
    pub fn new(
        ref_id: impl Into<String>,
        role: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            ref_id: ref_id.into(),
            role: role.into(),
            name: name.into(),
            description: String::new(),
            selector: String::new(),
            backend_node_id: None,
            bounds: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set selector
    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = selector.into();
        self
    }

    /// Set backend node ID
    pub fn with_backend_node_id(mut self, id: i64) -> Self {
        self.backend_node_id = Some(id);
        self
    }

    /// Set bounds
    pub fn with_bounds(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.bounds = Some((x, y, width, height));
        self
    }
}

impl fmt::Display for RefInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = if self.description.is_empty() {
            String::new()
        } else {
            format!(" - {}", self.description)
        };
        write!(
            f,
            "{} [{}] \"{}\"{}",
            self.ref_id, self.role, self.name, desc
        )
    }
}

/// Action result with before/after context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether action succeeded
    pub success: bool,
    /// Action description (e.g., "click @3")
    pub action: String,
    /// State summary before action
    pub before: String,
    /// State summary after action
    pub after: String,
    /// Detected changes
    pub changes: Vec<String>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ActionResult {
    /// Create a successful result
    pub fn success(action: impl Into<String>) -> Self {
        Self {
            success: true,
            action: action.into(),
            before: String::new(),
            after: String::new(),
            changes: Vec::new(),
            error: None,
            duration_ms: 0,
        }
    }

    /// Create a failed result
    pub fn failure(action: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            action: action.into(),
            before: String::new(),
            after: String::new(),
            changes: Vec::new(),
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Set before state
    pub fn with_before(mut self, before: impl Into<String>) -> Self {
        self.before = before.into();
        self
    }

    /// Set after state
    pub fn with_after(mut self, after: impl Into<String>) -> Self {
        self.after = after.into();
        self
    }

    /// Add a change
    pub fn with_change(mut self, change: impl Into<String>) -> Self {
        self.changes.push(change.into());
        self
    }

    /// Set changes
    pub fn with_changes(mut self, changes: Vec<String>) -> Self {
        self.changes = changes;
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

impl fmt::Display for ActionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            let changes = if self.changes.is_empty() {
                "none".to_string()
            } else {
                self.changes.join(", ")
            };
            write!(f, "✓ {}\n  Changes: {}", self.action, changes)
        } else {
            write!(
                f,
                "✗ {}\n  Error: {}",
                self.action,
                self.error.as_deref().unwrap_or("unknown")
            )
        }
    }
}

/// Ref ID type (accepts "@3", "3", or numeric 3)
#[derive(Debug, Clone)]
pub struct RefId(pub String);

impl RefId {
    /// Get normalized ref ID (always starts with @)
    pub fn normalized(&self) -> String {
        if self.0.starts_with('@') {
            self.0.clone()
        } else {
            format!("@{}", self.0)
        }
    }

    /// Get numeric ID
    pub fn numeric(&self) -> Option<u32> {
        self.0.trim_start_matches('@').parse().ok()
    }
}

impl From<&str> for RefId {
    fn from(s: &str) -> Self {
        RefId(s.to_string())
    }
}

impl From<String> for RefId {
    fn from(s: String) -> Self {
        RefId(s)
    }
}

impl From<u32> for RefId {
    fn from(n: u32) -> Self {
        RefId(format!("@{}", n))
    }
}

impl From<i32> for RefId {
    fn from(n: i32) -> Self {
        RefId(format!("@{}", n))
    }
}

/// Wait condition types
#[derive(Debug, Clone)]
pub enum WaitCondition {
    /// Wait for text to appear on page
    Text(String),
    /// Wait for ref to be visible
    Ref(RefId),
    /// Wait for URL to match pattern
    Url(String),
    /// Wait for network idle
    NetworkIdle,
    /// Wait for DOM content loaded
    DomContentLoaded,
    /// Wait for custom JavaScript expression
    Js(String),
}

impl WaitCondition {
    /// Parse from string (AI-friendly)
    ///
    /// Examples:
    /// - "text:Welcome" -> WaitCondition::Text("Welcome")
    /// - "ref:@5" -> WaitCondition::Ref(5)
    /// - "url:*/dashboard" -> WaitCondition::Url("*/dashboard")
    /// - "idle" -> WaitCondition::NetworkIdle
    /// - "loaded" -> WaitCondition::DomContentLoaded
    /// - "js:document.ready" -> WaitCondition::Js("document.ready")
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();

        if s == "idle" {
            return Ok(WaitCondition::NetworkIdle);
        }
        if s == "loaded" {
            return Ok(WaitCondition::DomContentLoaded);
        }

        if let Some(text) = s.strip_prefix("text:") {
            return Ok(WaitCondition::Text(text.to_string()));
        }
        if let Some(ref_str) = s.strip_prefix("ref:") {
            return Ok(WaitCondition::Ref(RefId::from(ref_str)));
        }
        if let Some(url) = s.strip_prefix("url:") {
            return Ok(WaitCondition::Url(url.to_string()));
        }
        if let Some(js) = s.strip_prefix("js:") {
            return Ok(WaitCondition::Js(js.to_string()));
        }

        // Default: treat as text search
        Ok(WaitCondition::Text(s.to_string()))
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "up" => Some(ScrollDirection::Up),
            "down" => Some(ScrollDirection::Down),
            "left" => Some(ScrollDirection::Left),
            "right" => Some(ScrollDirection::Right),
            _ => None,
        }
    }
}

/// Snapshot format
#[derive(Debug, Clone, Copy, Default)]
pub enum SnapshotFormat {
    /// AI-friendly text format
    #[default]
    Text,
    /// JSON format
    Json,
    /// Refs only (minimal)
    Refs,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_display() {
        let mut snapshot = Snapshot::new(
            "Test Page".to_string(),
            "http://localhost:5173/".to_string(),
            (1280, 720),
        );

        snapshot.refs.insert(
            "@1".to_string(),
            RefInfo::new("@1", "button", "Click Me").with_description("primary action"),
        );

        let text = snapshot.to_text();
        assert!(text.contains("Test Page"));
        assert!(text.contains("1280x720"));
        assert!(text.contains("@1"));
        assert!(text.contains("button"));
        assert!(text.contains("Click Me"));
    }

    #[test]
    fn test_ref_info_display() {
        let ref_info = RefInfo::new("@3", "textbox", "Search").with_description("search input");

        assert_eq!(
            ref_info.to_string(),
            "@3 [textbox] \"Search\" - search input"
        );
    }

    #[test]
    fn test_action_result_display() {
        let success = ActionResult::success("click @3").with_change("@3 focused");

        assert!(success.to_string().contains("✓"));
        assert!(success.to_string().contains("@3 focused"));

        let failure = ActionResult::failure("click @99", "ref not found");
        assert!(failure.to_string().contains("✗"));
        assert!(failure.to_string().contains("ref not found"));
    }

    #[test]
    fn test_ref_id_conversion() {
        assert_eq!(RefId::from("@3").normalized(), "@3");
        assert_eq!(RefId::from("3").normalized(), "@3");
        assert_eq!(RefId::from(3u32).normalized(), "@3");
        assert_eq!(RefId::from(3i32).normalized(), "@3");
    }

    #[test]
    fn test_wait_condition_parse() {
        assert!(matches!(
            WaitCondition::parse("text:Welcome").unwrap(),
            WaitCondition::Text(s) if s == "Welcome"
        ));
        assert!(matches!(
            WaitCondition::parse("ref:@5").unwrap(),
            WaitCondition::Ref(_)
        ));
        assert!(matches!(
            WaitCondition::parse("idle").unwrap(),
            WaitCondition::NetworkIdle
        ));
        assert!(matches!(
            WaitCondition::parse("url:*/dashboard").unwrap(),
            WaitCondition::Url(s) if s == "*/dashboard"
        ));
    }

    #[test]
    fn test_snapshot_find() {
        let mut snapshot =
            Snapshot::new("Test".to_string(), "http://test/".to_string(), (800, 600));
        snapshot.refs.insert(
            "@1".to_string(),
            RefInfo::new("@1", "button", "Submit Form"),
        );
        snapshot
            .refs
            .insert("@2".to_string(), RefInfo::new("@2", "link", "Go Home"));

        let results = snapshot.find("form");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Submit Form");

        let results = snapshot.find("FORM"); // Case insensitive
        assert_eq!(results.len(), 1);
    }
}
