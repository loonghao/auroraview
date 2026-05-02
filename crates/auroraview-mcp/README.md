# AuroraView MCP Server

AuroraView adapter crate for `dcc-mcp-core` — exposes an AuroraView instance as a DccAdapter over CDP WebSocket, and provides MCP tools for WebView management.

## Features

- **MCP Server** — exposes WebView management tools via Model Context Protocol
- **CDP Adapter** — implements `DccAdapter` trait for `dcc-mcp-core` integration
- **mDNS Broadcast** — auto-discovery by `dcc-mcp-client`
- **AG-UI Protocol** — SSE interface for real-time event streaming
- **Python Bindings** — optional `pyo3` bindings for `McpServer` start/stop

## MCP Tools

The following tools are exposed via the MCP Server:

### `screenshot`

Capture a screenshot of a WebView window.

**Parameters:**
- `id` (optional): WebView ID (uses default if not specified)

**Returns:** `ScreenshotOutput` with base64-encoded PNG data

**Example:**
```json
{
  "name": "screenshot",
  "parameters": {"id": "550e8400-e29b-41d4-a716-446655440000"}
}
```

### `load_url`

Load a URL in the specified WebView.

**Parameters:**
- `id` (optional): WebView ID
- `url` (required): URL to load (`http://`, `https://`, or `file://`)

**Returns:** `SuccessOutput`

**Example:**
```json
{
  "name": "load_url",
  "parameters": {"url": "https://example.com"}
}
```

### `load_html`

Load HTML content directly into a WebView.

**Parameters:**
- `id` (optional): WebView ID
- `html` (required): HTML content to load

**Returns:** `SuccessOutput`

**Example:**
```json
{
  "name": "load_html",
  "parameters": {"html": "<h1>Hello World</h1>"}
}
```

### `eval_js`

Execute JavaScript in the specified WebView and return the result.

**Parameters:**
- `id` (optional): WebView ID
- `script` (required): JavaScript code to execute

**Returns:** `JsResultOutput` with result value or error

**Example:**
```json
{
  "name": "eval_js",
  "parameters": {"script": "document.title"}
}
```

### `send_event`

Send a named event to the WebView's JavaScript context.

**Parameters:**
- `id` (optional): WebView ID
- `event` (required): Event name
- `payload` (optional): Event payload (JSON)

**Returns:** `SuccessOutput`

**Example:**
```json
{
  "name": "send_event",
  "parameters": {
    "event": "progress_update",
    "payload": {"progress": 42, "total": 100}
  }
}
```

### `get_hwnd`

Get the native window handle (HWND on Windows) for embedding in UE.

**Parameters:**
- `id` (optional): WebView ID

**Returns:** `HwndOutput` with the handle value

**Example:**
```json
{
  "name": "get_hwnd",
  "parameters": {}
}
```

### `list_webviews`

List all active WebView instances.

**Returns:** `ListWebViewsOutput` with array of WebView info

**Example:**
```json
{
  "name": "list_webviews",
  "parameters": {}
}
```

### `create_webview`

Create a new WebView instance.

**Parameters:**
- `title` (optional): Window title
- `url` (optional): Initial URL
- `html` (optional): Initial HTML
- `width` (optional): Window width (default: 800)
- `height` (optional): Window height (default: 600)
- `visible` (optional): Whether window is visible (default: true)
- `debug` (optional): Enable dev tools (default: false)

**Returns:** `SuccessOutput` with new WebView ID

**Example:**
```json
{
  "name": "create_webview",
  "parameters": {
    "title": "My WebView",
    "url": "https://example.com",
    "width": 1024,
    "height": 768
  }
}
```

### `close_webview`

Close and remove a WebView instance.

**Parameters:**
- `id` (required): WebView ID to close

**Returns:** `SuccessOutput`

**Example:**
```json
{
  "name": "close_webview",
  "parameters": {"id": "550e8400-e29b-41d4-a716-446655440000"}
}
```

## AG-UI Protocol Support

The MCP Server supports AG-UI protocol for real-time event streaming.

### `subscribe_agui_events`

Subscribe to AG-UI events via Server-Sent Events (SSE).

**Endpoint:** `GET /agui/events?run_id=<optional>`

**Example (curl):**
```bash
curl -N http://localhost:7890/agui/events?run_id=run-123
```

**Event Format:**
```
data: {"type": "StepStarted", "run_id": "run-123", "step_name": "process", "step_id": "step-1"}

data: {"type": "StepProgress", "run_id": "run-123", "progress": 42, "total": 100}

data: {"type": "StepFinished", "run_id": "run-123", "step_id": "step-1"}
```

## Configuration

### `McpServerConfig`

```rust
pub struct McpServerConfig {
    pub port: u16,              // MCP Server port (default: 7890)
    pub host: String,           // Bind address (default: "127.0.0.1")
    pub service_name: String,     // mDNS service name (default: "auroraview-mcp")
    pub enable_mdns: bool,      // Enable mDNS broadcast (default: false)
    pub enable_oauth: bool,    // Enable OAuth 2.0 (default: false)
    pub max_webviews: Option<usize>, // Max WebView instances (default: None = unlimited)
}
```

### Example: Create MCP Server

```rust
use auroraview_mcp::{
    McpServerConfig, 
    server::AuroraViewMcpServer,
    runner::McpRunner,
};

let config = McpServerConfig::with_all(
    7890,           // port
    "127.0.0.1",  // host
    "auroraview-mcp", // service_name
    true,          // enable_mdns
    false,         // enable_oauth
    Some(10),      // max_webviews
);

let runner = McpRunner::new(config);
// runner.start().await?; // Start the server
```

## Python Bindings

When compiled with `python-bindings` feature, the following Python API is available:

```python
from auroraview_mcp import McpServer

# Create and start MCP Server
server = McpServer(port=7890, enable_mdns=True)
server.start()  # Non-blocking, runs in background thread

# dcc-mcp-core's McpClient can auto-discover via mDNS
# and call screenshot, load_url, eval_js, etc.
```

## Testing

```bash
# Run unit tests
cargo test -p auroraview-mcp --lib

# Run integration tests
cargo test -p auroraview-mcp --test integration_test

# Run all tests
cargo test -p auroraview-mcp

# Run clippy
cargo clippy -p auroraview-mcp -- -D warnings
```

## Dependencies

- `rmcp` (1.6.0) — MCP SDK
- `axum` (0.7) — HTTP framework for MCP Streamable HTTP transport
- `mdns-sd` (0.10) — mDNS broadcast
- `tokio` (1.48) — Async runtime
- `dcc-mcp-protocols` (0.13.2) — DCC adapter traits

## License

Same as AuroraView project (MIT OR Apache-2.0).
