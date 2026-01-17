//! Core Inspector type for AI-friendly WebView inspection and automation

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde_json::Value;
use tracing::{debug, info};

use crate::a11y::{format_tree, process_a11y_tree};
use crate::cdp::{CdpClient, WebSocketCdpClient};
use crate::error::{InspectorError, Result};
use crate::snapshot::{
    ActionResult, RefId, RefInfo, ScrollDirection, Snapshot, SnapshotFormat, WaitCondition,
};

/// Inspector configuration
#[derive(Debug, Clone)]
pub struct InspectorConfig {
    /// Default timeout for operations
    pub timeout: Duration,
    /// Whether to capture screenshots with action results
    pub capture_screenshots: bool,
    /// Whether to detect changes after actions
    pub detect_changes: bool,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            capture_screenshots: false,
            detect_changes: true,
        }
    }
}

/// AI-friendly WebView inspector and automation tool
pub struct Inspector {
    client: Arc<dyn CdpClient>,
    config: InspectorConfig,
    /// Cached refs from last snapshot
    refs_cache: Mutex<std::collections::HashMap<String, RefInfo>>,
}

impl Inspector {
    /// Create inspector with existing CDP client
    pub fn new(client: Arc<dyn CdpClient>, config: InspectorConfig) -> Self {
        Self {
            client,
            config,
            refs_cache: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Connect to running CDP endpoint
    ///
    /// # Arguments
    /// * `endpoint` - CDP HTTP endpoint (e.g., "http://localhost:9222")
    ///
    /// # Example
    /// ```ignore
    /// let inspector = Inspector::connect("http://localhost:9222").await?;
    /// ```
    pub async fn connect(endpoint: &str) -> Result<Self> {
        Self::connect_with_config(endpoint, InspectorConfig::default()).await
    }

    /// Connect with custom configuration
    pub async fn connect_with_config(endpoint: &str, config: InspectorConfig) -> Result<Self> {
        info!("Connecting to CDP endpoint: {}", endpoint);
        let client = WebSocketCdpClient::connect(endpoint).await?;
        Ok(Self::new(Arc::new(client), config))
    }

    /// Connect to specific WebSocket URL
    pub async fn connect_ws(ws_url: &str) -> Result<Self> {
        let client = WebSocketCdpClient::connect_ws(ws_url).await?;
        Ok(Self::new(Arc::new(client), InspectorConfig::default()))
    }

    // === Snapshot ===

    /// Get page snapshot (accessibility tree + interactive refs)
    ///
    /// Returns an AI-readable snapshot with:
    /// - Page title and URL
    /// - Viewport dimensions
    /// - Interactive element refs (@1, @2, ...)
    /// - Page structure tree
    pub async fn snapshot(&self) -> Result<Snapshot> {
        self.snapshot_as(SnapshotFormat::default()).await
    }

    /// Get snapshot in specific format
    pub async fn snapshot_as(&self, _format: SnapshotFormat) -> Result<Snapshot> {
        debug!("Taking page snapshot");

        // Get page info
        let frame_tree = self.client.send_simple("Page.getFrameTree").await?;
        let frame = &frame_tree["frameTree"]["frame"];
        let title = frame["name"]
            .as_str()
            .or_else(|| frame_tree["frameTree"]["frame"]["url"].as_str())
            .unwrap_or("")
            .to_string();
        let url = frame["url"].as_str().unwrap_or("").to_string();

        // Get viewport
        let layout = self.client.send_simple("Page.getLayoutMetrics").await?;
        let viewport = (
            layout["layoutViewport"]["clientWidth"]
                .as_u64()
                .unwrap_or(1280) as u32,
            layout["layoutViewport"]["clientHeight"]
                .as_u64()
                .unwrap_or(720) as u32,
        );

        // Get accessibility tree
        let ax_tree = self.client.get_accessibility_tree().await?;
        let (nodes, refs) = process_a11y_tree(ax_tree);

        // Update cache
        {
            let mut cache = self.refs_cache.lock();
            cache.clear();
            cache.extend(refs.clone());
        }

        // Format tree
        let tree = format_tree(&nodes, &refs);

        // Get actual title from document
        let doc_title = self
            .client
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v["value"].as_str().map(|s| s.to_string()))
            .unwrap_or(title);

        Ok(Snapshot {
            title: doc_title,
            url,
            viewport,
            refs,
            tree,
        })
    }

    /// Take screenshot
    ///
    /// # Arguments
    /// * `path` - Optional file path to save screenshot
    ///
    /// # Returns
    /// PNG bytes
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        self.client.screenshot("png").await
    }

    // === Interaction ===

    /// Click element by ref
    ///
    /// # Arguments
    /// * `ref_id` - Ref ID (e.g., "@3", "3", or 3)
    ///
    /// # Example
    /// ```ignore
    /// inspector.click("@3").await?;
    /// inspector.click(3).await?;
    /// ```
    pub async fn click(&self, ref_id: impl Into<RefId>) -> Result<ActionResult> {
        let ref_id = ref_id.into();
        let normalized = ref_id.normalized();
        let action = format!("click {}", normalized);
        let start = Instant::now();

        // Get ref info
        let ref_info = self.get_ref_info(&normalized)?;
        let backend_id = ref_info
            .backend_node_id
            .ok_or_else(|| InspectorError::ElementNotFound(normalized.clone()))?;

        // Get before state
        let before = self.get_brief_state().await;

        // Click
        self.client.click_node(backend_id).await?;

        // Small delay for UI update
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get after state
        let after = self.get_brief_state().await;

        // Detect changes
        let changes = if self.config.detect_changes {
            self.detect_changes(&before, &after)
        } else {
            vec![]
        };

        Ok(ActionResult::success(action)
            .with_before(before)
            .with_after(after)
            .with_changes(changes)
            .with_duration(start.elapsed().as_millis() as u64))
    }

    /// Fill input by ref
    ///
    /// # Arguments
    /// * `ref_id` - Ref ID of input element
    /// * `text` - Text to fill
    pub async fn fill(&self, ref_id: impl Into<RefId>, text: &str) -> Result<ActionResult> {
        let ref_id = ref_id.into();
        let normalized = ref_id.normalized();
        let action = format!("fill {} \"{}\"", normalized, text);
        let start = Instant::now();

        // Get ref info
        let ref_info = self.get_ref_info(&normalized)?;
        let backend_id = ref_info
            .backend_node_id
            .ok_or_else(|| InspectorError::ElementNotFound(normalized.clone()))?;

        let before = self.get_brief_state().await;

        // Focus element
        self.client.focus_node(backend_id).await?;

        // Clear existing content
        self.client.press_key("Control+a").await.ok();
        self.client.press_key("Backspace").await.ok();

        // Type new text
        self.client.type_text(text).await?;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let after = self.get_brief_state().await;
        let changes = if self.config.detect_changes {
            self.detect_changes(&before, &after)
        } else {
            vec![]
        };

        Ok(ActionResult::success(action)
            .with_before(before)
            .with_after(after)
            .with_changes(changes)
            .with_duration(start.elapsed().as_millis() as u64))
    }

    /// Press a key
    ///
    /// # Arguments
    /// * `key` - Key to press (e.g., "Enter", "Tab", "Escape")
    pub async fn press(&self, key: &str) -> Result<ActionResult> {
        let action = format!("press {}", key);
        let start = Instant::now();

        let before = self.get_brief_state().await;
        self.client.press_key(key).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        let after = self.get_brief_state().await;

        let changes = if self.config.detect_changes {
            self.detect_changes(&before, &after)
        } else {
            vec![]
        };

        Ok(ActionResult::success(action)
            .with_before(before)
            .with_after(after)
            .with_changes(changes)
            .with_duration(start.elapsed().as_millis() as u64))
    }

    /// Scroll page
    ///
    /// # Arguments
    /// * `direction` - Scroll direction (up/down/left/right)
    /// * `amount` - Scroll amount in pixels
    pub async fn scroll(&self, direction: ScrollDirection, amount: i32) -> Result<ActionResult> {
        let action = format!("scroll {:?} {}", direction, amount);
        let start = Instant::now();

        let (delta_x, delta_y) = match direction {
            ScrollDirection::Up => (0, -amount),
            ScrollDirection::Down => (0, amount),
            ScrollDirection::Left => (-amount, 0),
            ScrollDirection::Right => (amount, 0),
        };

        self.client.scroll(delta_x, delta_y).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(ActionResult::success(action).with_duration(start.elapsed().as_millis() as u64))
    }

    // === Navigation ===

    /// Navigate to URL
    pub async fn goto(&self, url: &str) -> Result<ActionResult> {
        let action = format!("goto {}", url);
        let start = Instant::now();

        self.client.navigate(url).await?;

        // Wait for load
        self.wait_for_load().await?;

        Ok(ActionResult::success(action)
            .with_change("page navigated")
            .with_duration(start.elapsed().as_millis() as u64))
    }

    /// Go back
    pub async fn back(&self) -> Result<ActionResult> {
        let action = "back".to_string();
        let start = Instant::now();

        self.client.go_back().await?;
        self.wait_for_load().await?;

        Ok(ActionResult::success(action)
            .with_change("navigated back")
            .with_duration(start.elapsed().as_millis() as u64))
    }

    /// Go forward
    pub async fn forward(&self) -> Result<ActionResult> {
        let action = "forward".to_string();
        let start = Instant::now();

        self.client.go_forward().await?;
        self.wait_for_load().await?;

        Ok(ActionResult::success(action)
            .with_change("navigated forward")
            .with_duration(start.elapsed().as_millis() as u64))
    }

    /// Reload page
    pub async fn reload(&self) -> Result<ActionResult> {
        let action = "reload".to_string();
        let start = Instant::now();

        self.client.reload().await?;
        self.wait_for_load().await?;

        Ok(ActionResult::success(action)
            .with_change("page reloaded")
            .with_duration(start.elapsed().as_millis() as u64))
    }

    // === Query ===

    /// Get element text
    pub async fn text(&self, ref_id: impl Into<RefId>) -> Result<String> {
        let ref_info = self.get_ref_info(&ref_id.into().normalized())?;

        if let Some(backend_id) = ref_info.backend_node_id {
            // Get text via JS
            let result = self
                .client
                .evaluate(&format!(
                    "(function() {{ \
                        var node = document.evaluate( \
                            '//*[@data-backend-node-id=\"{}\"]', \
                            document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null \
                        ).singleNodeValue; \
                        return node ? node.textContent : ''; \
                    }})()",
                    backend_id
                ))
                .await?;

            Ok(result["value"].as_str().unwrap_or("").to_string())
        } else {
            // Fall back to cached name
            Ok(ref_info.name.clone())
        }
    }

    /// Get input value
    pub async fn value(&self, ref_id: impl Into<RefId>) -> Result<String> {
        let ref_info = self.get_ref_info(&ref_id.into().normalized())?;

        if let Some(backend_id) = ref_info.backend_node_id {
            let result = self
                .client
                .evaluate(&format!(
                    "(function() {{ \
                        var node = document.evaluate( \
                            '//*[@data-backend-node-id=\"{}\"]', \
                            document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null \
                        ).singleNodeValue; \
                        return node ? node.value : ''; \
                    }})()",
                    backend_id
                ))
                .await?;

            Ok(result["value"].as_str().unwrap_or("").to_string())
        } else {
            Ok(String::new())
        }
    }

    /// Execute JavaScript
    pub async fn eval(&self, script: &str) -> Result<Value> {
        self.client.evaluate(script).await
    }

    // === Wait ===

    /// Wait for condition
    ///
    /// # Arguments
    /// * `condition` - Condition string (e.g., "text:Welcome", "ref:@5", "idle")
    /// * `timeout` - Optional timeout (default: 30s)
    pub async fn wait(&self, condition: &str, timeout: Option<Duration>) -> Result<bool> {
        let timeout = timeout.unwrap_or(self.config.timeout);
        let condition = WaitCondition::parse(condition).map_err(|e| InspectorError::Parse(e))?;

        self.wait_for(condition, timeout).await
    }

    /// Wait for condition (typed)
    pub async fn wait_for(&self, condition: WaitCondition, timeout: Duration) -> Result<bool> {
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Ok(false);
            }

            let satisfied = match &condition {
                WaitCondition::Text(text) => {
                    let body = self
                        .client
                        .evaluate("document.body.innerText")
                        .await
                        .ok()
                        .and_then(|v| v["value"].as_str().map(|s| s.to_string()))
                        .unwrap_or_default();
                    body.contains(text)
                }

                WaitCondition::Ref(ref_id) => {
                    let snap = self.snapshot().await.ok();
                    snap.map(|s| s.get_ref(&ref_id.normalized()).is_some())
                        .unwrap_or(false)
                }

                WaitCondition::Url(pattern) => {
                    let current = self.url().await.unwrap_or_default();
                    if pattern.contains('*') {
                        // Simple glob matching
                        let regex_pattern = pattern.replace("*", ".*");
                        regex::Regex::new(&regex_pattern)
                            .map(|r| r.is_match(&current))
                            .unwrap_or(false)
                    } else {
                        current.contains(pattern)
                    }
                }

                WaitCondition::NetworkIdle => {
                    // Simple heuristic: check if page is loaded
                    let ready_state = self
                        .client
                        .evaluate("document.readyState")
                        .await
                        .ok()
                        .and_then(|v| v["value"].as_str().map(|s| s.to_string()))
                        .unwrap_or_default();
                    ready_state == "complete"
                }

                WaitCondition::DomContentLoaded => {
                    let ready_state = self
                        .client
                        .evaluate("document.readyState")
                        .await
                        .ok()
                        .and_then(|v| v["value"].as_str().map(|s| s.to_string()))
                        .unwrap_or_default();
                    ready_state == "interactive" || ready_state == "complete"
                }

                WaitCondition::Js(expr) => self
                    .client
                    .evaluate(expr)
                    .await
                    .ok()
                    .and_then(|v| v["value"].as_bool())
                    .unwrap_or(false),
            };

            if satisfied {
                return Ok(true);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    // === Properties ===

    /// Get current page URL
    pub async fn url(&self) -> Result<String> {
        let result = self.client.evaluate("window.location.href").await?;
        Ok(result["value"].as_str().unwrap_or("").to_string())
    }

    /// Get current page title
    pub async fn title(&self) -> Result<String> {
        let result = self.client.evaluate("document.title").await?;
        Ok(result["value"].as_str().unwrap_or("").to_string())
    }

    /// Close connection
    pub async fn close(&self) -> Result<()> {
        self.client.close().await
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    // === Private helpers ===

    /// Get ref info from cache or error
    fn get_ref_info(&self, ref_id: &str) -> Result<RefInfo> {
        let cache = self.refs_cache.lock();
        cache
            .get(ref_id)
            .cloned()
            .ok_or_else(|| InspectorError::InvalidRef(ref_id.to_string()))
    }

    /// Get brief state summary
    async fn get_brief_state(&self) -> String {
        let url = self.url().await.unwrap_or_default();
        let title = self.title().await.unwrap_or_default();
        format!("{} - {}", title, url)
    }

    /// Detect changes between states
    fn detect_changes(&self, _before: &str, _after: &str) -> Vec<String> {
        // Simple change detection - in a full implementation,
        // this would compare snapshots
        vec![]
    }

    /// Wait for page load
    async fn wait_for_load(&self) -> Result<()> {
        self.wait_for(WaitCondition::DomContentLoaded, Duration::from_secs(30))
            .await?;
        Ok(())
    }
}
