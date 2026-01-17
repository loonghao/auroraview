# RFC 0008: AuroraView Testing Framework Redesign

## Status: Draft
## Authors: AuroraView Team
## Created: 2026-01-17

---

## Summary

Redesign the AuroraView testing framework from scratch, implementing a Rust-native `auroraview-testing` crate with Python bindings. The primary goal is to provide an AI-friendly interface for self-discovery, debugging, and iteration, while also serving developers for automated testing.

## Motivation

### Current Problems

1. **Over-complexity** - 12+ Python modules with scattered concepts
2. **Inconsistent APIs** - HeadlessWebView, AuroraTest, Midscene parallel APIs
3. **AI-unfriendly** - Lack of structured output for AI parsing
4. **Heavy dependencies** - Playwright, Hypothesis, Selenium mixed
5. **Low adoption** - Powerful but rarely used

### Design Goals

1. **Simple** - One core type (`Inspector`), 5-minute onboarding
2. **AI-native** - Structured snapshots, natural language descriptions, machine-parseable
3. **Zero-dependency core** - Only requires CDP (WebSocket)
4. **Self-describing** - Every action returns sufficient context for AI understanding
5. **Progressive enhancement** - Optional Playwright integration

---

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    auroraview-testing                        │
├─────────────────────────────────────────────────────────────┤
│  inspector.rs     - Core Inspector type                      │
│  snapshot.rs      - Snapshot, RefInfo, ActionResult          │
│  cdp/             - CDP client (WebSocket-based)             │
│  a11y/            - Accessibility tree processing            │
│  python.rs        - PyO3 bindings                            │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────┐    ┌──────────────────┐    ┌─────────────┐
│ CDP Backend │    │ chromiumoxide    │    │ Python SDK  │
│ (WebSocket) │    │ (optional)       │    │ (bindings)  │
└─────────────┘    └──────────────────┘    └─────────────┘
```

### Core Types

#### Rust Types

```rust
// crates/auroraview-testing/src/inspector.rs

/// AI-friendly WebView inspector and automation tool
pub struct Inspector {
    client: Box<dyn CdpClient>,
    config: InspectorConfig,
}

impl Inspector {
    // === Connection ===
    
    /// Connect to running CDP endpoint
    pub async fn connect(cdp_url: &str) -> Result<Self, InspectorError>;
    
    /// Connect to running CDP endpoint (sync version)
    pub fn connect_sync(cdp_url: &str) -> Result<Self, InspectorError>;
    
    /// Launch new browser via chromiumoxide (requires feature)
    #[cfg(feature = "chromiumoxide")]
    pub async fn launch(config: LaunchConfig) -> Result<Self, InspectorError>;
    
    // === Snapshot ===
    
    /// Get page snapshot (accessibility tree + interactive refs)
    pub async fn snapshot(&self) -> Result<Snapshot, InspectorError>;
    
    /// Get snapshot in specific format
    pub async fn snapshot_as(&self, format: SnapshotFormat) -> Result<String, InspectorError>;
    
    /// Take screenshot
    pub async fn screenshot(&self) -> Result<Vec<u8>, InspectorError>;
    
    // === Interaction ===
    
    /// Click element by ref (e.g., "@3" or 3)
    pub async fn click(&self, ref_id: impl Into<RefId>) -> Result<ActionResult, InspectorError>;
    
    /// Fill input by ref
    pub async fn fill(&self, ref_id: impl Into<RefId>, text: &str) -> Result<ActionResult, InspectorError>;
    
    /// Press key
    pub async fn press(&self, key: &str) -> Result<ActionResult, InspectorError>;
    
    /// Scroll page
    pub async fn scroll(&self, direction: ScrollDirection, amount: i32) -> Result<ActionResult, InspectorError>;
    
    // === Navigation ===
    
    /// Navigate to URL
    pub async fn goto(&self, url: &str) -> Result<ActionResult, InspectorError>;
    
    /// Go back
    pub async fn back(&self) -> Result<ActionResult, InspectorError>;
    
    /// Go forward  
    pub async fn forward(&self) -> Result<ActionResult, InspectorError>;
    
    /// Reload page
    pub async fn reload(&self) -> Result<ActionResult, InspectorError>;
    
    // === Query ===
    
    /// Get element text
    pub async fn text(&self, ref_id: impl Into<RefId>) -> Result<String, InspectorError>;
    
    /// Get input value
    pub async fn value(&self, ref_id: impl Into<RefId>) -> Result<String, InspectorError>;
    
    /// Execute JavaScript
    pub async fn eval<T: DeserializeOwned>(&self, script: &str) -> Result<T, InspectorError>;
    
    // === Wait ===
    
    /// Wait for condition
    pub async fn wait(&self, condition: WaitCondition, timeout: Duration) -> Result<bool, InspectorError>;
    
    // === Properties ===
    
    /// Current page URL
    pub async fn url(&self) -> Result<String, InspectorError>;
    
    /// Current page title
    pub async fn title(&self) -> Result<String, InspectorError>;
}
```

#### Snapshot Types

```rust
// crates/auroraview-testing/src/snapshot.rs

/// Page snapshot with accessibility tree and interactive element refs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Page title
    pub title: String,
    /// Page URL
    pub url: String,
    /// Viewport dimensions
    pub viewport: (u32, u32),
    /// Interactive element refs (@1, @2, ...)
    pub refs: HashMap<String, RefInfo>,
    /// Accessibility tree as text
    pub tree: String,
    /// Raw accessibility tree nodes (for programmatic access)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<A11yNode>>,
}

impl Snapshot {
    /// Get ref count
    pub fn ref_count(&self) -> usize;
    
    /// Find refs containing text
    pub fn find(&self, text: &str) -> Vec<&RefInfo>;
    
    /// Get ref by ID
    pub fn get_ref(&self, id: &str) -> Option<&RefInfo>;
    
    /// Format as AI-friendly text
    pub fn to_text(&self) -> String;
    
    /// Format as JSON
    pub fn to_json(&self) -> String;
}

impl std::fmt::Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    /// Bounding box (x, y, width, height)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<(f64, f64, f64, f64)>,
}

impl std::fmt::Display for RefInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}] \"{}\"", self.ref_id, self.role, self.name)
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

impl std::fmt::Display for ActionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "✓ {}\n  Changes: {}", 
                self.action, 
                if self.changes.is_empty() { "none".to_string() } else { self.changes.join(", ") }
            )
        } else {
            write!(f, "✗ {}\n  Error: {}", self.action, self.error.as_deref().unwrap_or("unknown"))
        }
    }
}
```

#### Wait Conditions

```rust
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
    /// Examples:
    ///   "text:Welcome" -> WaitCondition::Text("Welcome")
    ///   "ref:@5" -> WaitCondition::Ref(5)
    ///   "url:*/dashboard" -> WaitCondition::Url("*/dashboard")
    ///   "idle" -> WaitCondition::NetworkIdle
    pub fn parse(s: &str) -> Result<Self, ParseError>;
}
```

### CDP Client

```rust
// crates/auroraview-testing/src/cdp/mod.rs

mod client;
mod websocket;

pub use client::CdpClient;

/// CDP client trait
#[async_trait]
pub trait CdpClient: Send + Sync {
    /// Send CDP command and get response
    async fn send(&self, method: &str, params: Value) -> Result<Value, CdpError>;
    
    /// Subscribe to CDP events
    async fn subscribe(&self, event: &str, handler: Box<dyn Fn(Value) + Send + Sync>);
    
    /// Get page targets
    async fn targets(&self) -> Result<Vec<TargetInfo>, CdpError>;
    
    /// Attach to target
    async fn attach(&self, target_id: &str) -> Result<SessionId, CdpError>;
    
    /// Close connection
    async fn close(&self) -> Result<(), CdpError>;
}

/// WebSocket-based CDP client (default, zero external deps)
pub struct WebSocketCdpClient {
    ws: WebSocket,
    request_id: AtomicU64,
    pending: DashMap<u64, oneshot::Sender<Value>>,
}

impl WebSocketCdpClient {
    /// Connect to CDP endpoint
    pub async fn connect(url: &str) -> Result<Self, CdpError>;
}
```

### Accessibility Tree Processing

```rust
// crates/auroraview-testing/src/a11y/mod.rs

mod tree;
mod formatter;

/// Accessibility tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yNode {
    pub node_id: String,
    pub role: String,
    pub name: String,
    pub description: String,
    pub value: Option<String>,
    pub children: Vec<A11yNode>,
    pub properties: HashMap<String, Value>,
    /// Whether this node is interactive (clickable, editable, etc.)
    pub interactive: bool,
    /// Backend node ID for CDP operations
    pub backend_node_id: Option<i64>,
}

/// Process accessibility tree from CDP
pub fn process_a11y_tree(root: Value) -> Result<(Vec<A11yNode>, HashMap<String, RefInfo>), A11yError>;

/// Format tree as text with refs
pub fn format_tree(nodes: &[A11yNode], refs: &HashMap<String, RefInfo>) -> String;
```

### Python Bindings

```rust
// crates/auroraview-testing/src/python.rs

use pyo3::prelude::*;

/// Python Inspector class
#[pyclass(name = "Inspector")]
pub struct PyInspector {
    inner: Arc<Mutex<Inspector>>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyInspector {
    /// Connect to CDP endpoint
    #[classmethod]
    fn connect(_cls: &Bound<'_, PyType>, cdp_url: &str) -> PyResult<Self>;
    
    /// Get page snapshot
    fn snapshot(&self) -> PyResult<PySnapshot>;
    
    /// Take screenshot
    fn screenshot(&self, path: Option<&str>) -> PyResult<Vec<u8>>;
    
    /// Click element
    fn click(&self, ref_id: PyRefId) -> PyResult<PyActionResult>;
    
    /// Fill input
    fn fill(&self, ref_id: PyRefId, text: &str) -> PyResult<PyActionResult>;
    
    /// Press key
    fn press(&self, key: &str) -> PyResult<PyActionResult>;
    
    /// Navigate to URL
    fn goto(&self, url: &str) -> PyResult<PyActionResult>;
    
    /// Go back
    fn back(&self) -> PyResult<PyActionResult>;
    
    /// Go forward
    fn forward(&self) -> PyResult<PyActionResult>;
    
    /// Reload
    fn reload(&self) -> PyResult<PyActionResult>;
    
    /// Get element text
    fn text(&self, ref_id: PyRefId) -> PyResult<String>;
    
    /// Get input value
    fn value(&self, ref_id: PyRefId) -> PyResult<String>;
    
    /// Execute JavaScript
    fn eval(&self, script: &str) -> PyResult<PyObject>;
    
    /// Wait for condition
    fn wait(&self, condition: &str, timeout: Option<f64>) -> PyResult<bool>;
    
    /// Current URL
    #[getter]
    fn url(&self) -> PyResult<String>;
    
    /// Current title
    #[getter]
    fn title(&self) -> PyResult<String>;
    
    /// Close connection
    fn close(&self) -> PyResult<()>;
    
    /// Context manager enter
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self>;
    
    /// Context manager exit
    fn __exit__(&self, _exc_type: Option<&PyAny>, _exc_val: Option<&PyAny>, _exc_tb: Option<&PyAny>) -> PyResult<bool>;
}
```

---

## Snapshot Output Format

### Text Format (Default, AI-friendly)

```
Page: "AuroraView Gallery" (http://localhost:5173/)
Viewport: 1280x720

Interactive Elements (23 refs):
  @1  [link] "Home" - navigation
  @2  [link] "Gallery" - navigation, current
  @3  [textbox] "Search..." - search input
  @4  [button] "Filter: All" - dropdown trigger
  @5  [link] "Basic WebView" - sample card
  @6  [link] "Tab Browser" - sample card
  ...

Page Structure:
  header
    nav: Home [@1] | Gallery [@2] | Docs
    search: [@3 textbox]
  main
    filter-bar: [@4 dropdown] tags: All, Basic, Advanced
    grid (12 items):
      card: "Basic WebView" [@5] - basic, demo
      card: "Tab Browser" [@6] - advanced, tabs
      ...
  footer
    "AuroraView v1.0"
```

### JSON Format

```json
{
  "title": "AuroraView Gallery",
  "url": "http://localhost:5173/",
  "viewport": [1280, 720],
  "refs": {
    "@1": {
      "ref_id": "@1",
      "role": "link",
      "name": "Home",
      "description": "navigation",
      "selector": "nav a[href='/']",
      "bounds": [100, 20, 60, 24]
    },
    "@2": {
      "ref_id": "@2",
      "role": "link", 
      "name": "Gallery",
      "description": "navigation, current",
      "selector": "nav a[href='/gallery']",
      "bounds": [170, 20, 80, 24]
    }
  },
  "tree": "header\n  nav: Home [@1] | Gallery [@2] | Docs\n  ..."
}
```

---

## AI Skill Definition

### Skill: `auroraview-inspector`

```yaml
name: auroraview-inspector
description: |
  Inspect and interact with AuroraView WebView applications via CDP.
  Use this skill to debug, test, and automate AuroraView-based apps.

capabilities:
  - Get page snapshots with interactive element refs
  - Click, fill, and interact with elements
  - Navigate pages (goto, back, forward, reload)
  - Execute JavaScript
  - Take screenshots
  - Wait for conditions

usage:
  connect: |
    Connect to running AuroraView instance:
    ```python
    from auroraview.testing import Inspector
    inspector = Inspector.connect("http://localhost:9222")
    ```

  snapshot: |
    Get page state:
    ```python
    snap = inspector.snapshot()
    print(snap)  # AI-readable text format
    print(snap.refs)  # Interactive elements
    ```

  interact: |
    Interact with elements using ref IDs:
    ```python
    inspector.click("@3")  # or click(3)
    inspector.fill("@4", "search text")
    inspector.press("Enter")
    ```

  navigate: |
    Navigate pages:
    ```python
    inspector.goto("https://example.com")
    inspector.back()
    inspector.reload()
    ```

  wait: |
    Wait for conditions:
    ```python
    inspector.wait("text:Welcome")
    inspector.wait("ref:@5")
    inspector.wait("url:*/dashboard")
    inspector.wait("idle")
    ```

examples:
  - name: Debug Gallery
    steps:
      - Connect to http://localhost:9222
      - Get snapshot to understand page structure
      - Click search box (@3)
      - Fill with "browser"
      - Check results in new snapshot

  - name: Test Navigation
    steps:
      - Connect and get initial snapshot
      - Click "Home" link (@1)
      - Wait for url:*/home
      - Verify new page content

ref_format: |
  Refs are formatted as @N where N is a number.
  Use refs to target interactive elements:
    @1, @2, @3... or just 1, 2, 3...
  
  Ref info includes:
    - role: button, link, textbox, etc.
    - name: accessible name
    - description: context info
```

---

## File Structure

```
crates/auroraview-testing/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── inspector.rs      # Core Inspector type
│   ├── snapshot.rs       # Snapshot, RefInfo, ActionResult
│   ├── error.rs          # Error types
│   ├── config.rs         # Configuration
│   ├── cdp/
│   │   ├── mod.rs
│   │   ├── client.rs     # CdpClient trait
│   │   ├── websocket.rs  # WebSocket implementation
│   │   └── protocol.rs   # CDP protocol types (reuse auroraview-devtools)
│   ├── a11y/
│   │   ├── mod.rs
│   │   ├── tree.rs       # A11y tree processing
│   │   └── formatter.rs  # Tree formatting
│   └── python.rs         # PyO3 bindings
└── tests/
    ├── inspector_tests.rs
    ├── snapshot_tests.rs
    └── cdp_tests.rs
```

### Python Package Structure (Simplified)

```
python/auroraview/testing/
├── __init__.py           # Only exports Inspector
├── _legacy/              # Old code (for reference, not imported)
│   ├── headless_webview.py
│   ├── auroratest/
│   └── ...
```

---

## Dependencies

### Cargo.toml

```toml
[package]
name = "auroraview-testing"
version = "0.1.0"
edition = "2021"
description = "AI-friendly testing and inspection for AuroraView"
license = "MIT"
keywords = ["testing", "webview", "cdp", "automation", "ai"]

[dependencies]
# Reuse existing CDP types
auroraview-devtools = { path = "../auroraview-devtools" }

# Async runtime
tokio = { version = "1.48", features = ["rt-multi-thread", "sync", "time", "macros"] }

# WebSocket
tokio-tungstenite = "0.26"
futures-util = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Thread safety
parking_lot = "0.12"
dashmap = "6.1"

# URL handling
url = "2.5"

# Logging
tracing = "0.1"

# PyO3 (optional)
pyo3 = { version = "0.27", optional = true }

# chromiumoxide for browser launching (optional)
chromiumoxide = { version = "0.8", optional = true, default-features = false, features = ["tokio-runtime"] }

[features]
default = []
python = ["pyo3"]
chromiumoxide = ["dep:chromiumoxide"]

[dev-dependencies]
rstest = "0.26"
tokio-test = "0.4"
```

---

## Migration Plan

### Phase 1: Core Implementation (Week 1)

1. Create `crates/auroraview-testing/` directory structure
2. Implement `cdp/` module (WebSocket CDP client)
3. Implement `a11y/` module (accessibility tree processing)
4. Implement `snapshot.rs` (data types)
5. Implement `inspector.rs` (core API)
6. Add basic tests

### Phase 2: Python Bindings (Week 2)

1. Implement `python.rs` with PyO3
2. Update `python/auroraview/testing/__init__.py` to export `Inspector`
3. Move old code to `_legacy/` (do not delete yet)
4. Add integration tests

### Phase 3: AI Skill & Docs (Week 3)

1. Create AI skill definition file
2. Update documentation
3. Add examples
4. Delete legacy code

---

## Usage Examples

### Python: Debug Session

```python
from auroraview.testing import Inspector

# Connect to running Gallery
i = Inspector.connect("http://localhost:9222")

# Understand page state
print(i.snapshot())
# Page: "AuroraView Gallery" (http://localhost:5173/)
# Interactive Elements (23 refs):
#   @1  [link] "Home"
#   @2  [link] "Gallery" - current
#   @3  [textbox] "Search..."
#   ...

# Interact
result = i.click(3)  # Click search box
print(result)
# ✓ click @3
#   Changes: @3 focused

result = i.fill(3, "browser")
print(result)
# ✓ fill @3 "browser"
#   Changes: search filtered (12 -> 3 items)

# Verify
snap = i.snapshot()
print(snap.find("Tab Browser"))
# [@6 [link] "Tab Browser"]

i.close()
```

### Python: Test Case

```python
from auroraview.testing import Inspector
import pytest

def test_search_filter():
    with Inspector.connect("http://localhost:9222") as page:
        # Initial state
        snap = page.snapshot()
        assert snap.title == "AuroraView Gallery"
        assert snap.ref_count() > 10
        
        # Search
        page.fill("@3", "browser")
        page.wait("idle")
        
        # Verify
        snap = page.snapshot()
        assert len(snap.find("browser")) > 0
```

### AI Interaction Example

```
AI: I'll inspect the running AuroraView Gallery application.

> inspector.snapshot()

Page: "AuroraView Gallery" (http://localhost:5173/)
Viewport: 1280x720

Interactive Elements (23 refs):
  @1  [link] "Home"
  @2  [link] "Gallery" - current
  @3  [textbox] "Search..."
  @4  [button] "Filter: All"
  ...

AI: I see the Gallery page is loaded with 23 interactive elements. 
    The search box is @3. Let me search for "browser" examples.

> inspector.click(3)
✓ click @3
  Changes: @3 focused

> inspector.fill(3, "browser")
✓ fill @3 "browser"
  Changes: search filtered (12 -> 3 items)

> inspector.snapshot()

Page: "AuroraView Gallery" (http://localhost:5173/?search=browser)
...
Interactive Elements (8 refs):
  @5  [link] "Tab Browser" - advanced, tabs
  @6  [link] "Multi-Tab Browser" - advanced
  @7  [link] "Browser Demo" - basic
  ...

AI: The search filtered results to 3 browser-related examples:
    - Tab Browser (advanced)
    - Multi-Tab Browser (advanced)  
    - Browser Demo (basic)
```

---

## Comparison with Previous Design

| Aspect | Old (Python) | New (Rust) |
|--------|--------------|------------|
| Core module | 12+ Python files | 1 Rust crate |
| API surface | 50+ classes/functions | 1 `Inspector` type |
| Dependencies | Playwright, Hypothesis, Selenium | WebSocket only (zero deps) |
| AI support | Manual formatting | Built-in structured output |
| Performance | Python GIL limited | Native async |
| Portability | Python only | Rust + Python bindings |
| Learning curve | Steep (multiple APIs) | Flat (single API) |

---

## Decision

Proceed with Rust-native implementation:

1. Better performance (native async, no GIL)
2. Full control over CDP protocol
3. Consistent with AuroraView core tech stack
4. Single source of truth (Rust crate with Python bindings)
5. Future extensibility (CLI tool, other language bindings)

## References

- [chromiumoxide](https://github.com/mattsse/chromiumoxide) - Rust CDP client
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [Accessibility Tree](https://developer.chrome.com/docs/devtools/accessibility/reference)
- [agent-browser](https://github.com/vercel-labs/agent-browser) - AI browser automation
