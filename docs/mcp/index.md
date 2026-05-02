# AuroraView MCP Server

AuroraView includes an MCP (Model Context Protocol) server (`auroraview-mcp`) that exposes WebView management tools to AI agents.

## Features

- **MCP Tools**: `screenshot`, `load_url`, `load_html`, `eval_js`, `send_event`, `get_hwnd`, `list_webviews`, `create_webview`, `close_webview`
- **mDNS Broadcast**: Auto-discoverable by `dcc-mcp-client`
- **AG-UI Protocol**: SSE endpoint at `/agui/events` for real-time event streaming
- **Python Bindings**: `McpServer` and `McpConfig` classes for easy integration

## Quick Start

### Rust Usage

```rust
use auroraview_mcp::{runner::McpRunner, types::McpServerConfig};

let config = McpServerConfig::default()
    .with_port(7890)
    .with_mdns(true);

let runner = McpRunner::new(config);
tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(runner.start())
    .unwrap();

// Server is now running at http://127.0.0.1:7890/mcp
// AG-UI SSE endpoint: http://127.0.0.1:7890/agui/events
```

### Python Usage

```python
from auroraview import McpServer

server = McpServer(port=7890)
server.start()  # non-blocking, runs in background thread

# Emit AG-UI events
server.emit_run_started("run-1", "thread-1")
server.emit_tool_call_start("run-1", "call-1", "screenshot")

# Get server URLs
print(server.mcp_url())   # http://127.0.0.1:7890/mcp
print(server.agui_url())  # http://127.0.0.1:7890/agui/events

# Stop server
server.stop()
```

## MCP Tools

### `screenshot`

Capture a screenshot of a WebView window.

```json
{
  "name": "screenshot",
  "arguments": {
    "id": "optional-webview-id"
  }
}
```

Returns: `ScreenshotData` (base64-encoded PNG)

### `load_url`

Load a URL in a WebView.

```json
{
  "name": "load_url",
  "arguments": {
    "id": "webview-id",
    "url": "https://example.com"
  }
}
```

### `eval_js`

Execute JavaScript in a WebView.

```json
{
  "name": "eval_js",
  "arguments": {
    "id": "webview-id",
    "script": "document.title"
  }
}
```

Returns: `JsResult` (JSON value)

## AG-UI Integration

The MCP server includes an AG-UI SSE endpoint for real-time event streaming:

```
GET /agui/events?run_id=optional-run-id
```

Events:
- `RunStarted`, `RunFinished`, `RunError`
- `StepStarted`, `StepFinished`
- `ToolCallStart`, `ToolCallArgs`, `ToolCallEnd`
- `TextMessageStart`, `TextMessageContent`, `TextMessageEnd`
- `StateSnapshot`, `StateDelta`
- `Custom`

## mDNS Broadcast

When enabled (`with_mdns(true)`), the server broadcasts itself as:

```
_auroraview-mcp._tcp.local.
```

Service metadata:
- `version`: AuroraView version
- `protocol`: "mcp"
- `transport`: "sse"
- `path`: "/mcp"

`dcc-mcp-client` can auto-discover the server via mDNS.

## Configuration

```rust
let config = McpServerConfig::default()
    .with_port(7890)           // MCP server port
    .with_mdns(true)           // enable mDNS broadcast
    .with_oauth(true)          // enable OAuth 2.0
    .with_max_webviews(10);    // limit concurrent WebViews
```

## Testing

```bash
# Run unit tests
cargo test -p auroraview-mcp

# Run integration tests
cargo test -p auroraview-mcp --features test-helpers
```

## Architecture

```
┌─────────────────────────────────┐
│               AuroraView MCP Server               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  MCP Tools   │  │  AG-UI SSE  │  │  mDNS Broadcast│ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│  ┌──────────────┐  ┌──────────────┐                │
│  │ WebView      │  │ OAuth 2.0   │                │
│  │ Registry     │  │ Auth         │                │
│  └──────────────┘  └──────────────┘                │
└─────────────────────────────────┘
           ↓
┌─────────────────────────────────┐
│         AuroraView WebView (wry/tao)        │
└─────────────────────────────────┘
```

## References

- [Model Context Protocol](https://modelcontextprotocol.io)
- [AG-UI Protocol](https://docs.ag-ui.com)
- [dcc-mcp-core](https://github.com/loonghao/dcc-mcp-core)
