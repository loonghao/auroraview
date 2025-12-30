# AuroraView MCP Server

[![PyPI version](https://badge.fury.io/py/auroraview-mcp.svg)](https://badge.fury.io/py/auroraview-mcp)
[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP (Model Context Protocol) Server for AuroraView - enabling AI assistants to interact with WebView applications in standalone mode and DCC environments (Maya, Blender, Houdini, etc.).

## Features

- **Instance Discovery**: Automatically discover running AuroraView instances
- **CDP Connection**: Connect to WebView2 instances via Chrome DevTools Protocol
- **API Bridge**: Call Python backend APIs through the JS bridge
- **UI Automation**: Click, fill, screenshot, and more
- **Gallery Integration**: Run and manage AuroraView samples
- **DCC Support**: Connect to AuroraView panels in Maya, Blender, Houdini, Nuke, Unreal

## Installation

```bash
# Using pip
pip install auroraview-mcp

# Using uv
uv pip install auroraview-mcp

# From source
cd packages/auroraview-mcp
uv pip install -e ".[dev]"
```

## Quick Start

### Configure with Claude Desktop / CodeBuddy

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "auroraview": {
      "command": "uvx",
      "args": ["auroraview-mcp"],
      "env": {
        "AURORAVIEW_DEFAULT_PORT": "9222"
      }
    }
  }
}
```

### Development Mode

```json
{
  "mcpServers": {
    "auroraview": {
      "command": "uv",
      "args": [
        "--directory",
        "/path/to/dcc_webview/packages/auroraview-mcp",
        "run",
        "auroraview-mcp"
      ]
    }
  }
}
```

## Available Tools

### Discovery Tools

| Tool | Description |
|------|-------------|
| `discover_instances` | Discover all running AuroraView instances |
| `connect` | Connect to an AuroraView instance |
| `disconnect` | Disconnect from current instance |
| `list_dcc_instances` | Discover instances in DCC environments |

### Page Tools

| Tool | Description |
|------|-------------|
| `list_pages` | List all pages in connected instance |
| `select_page` | Select a page by ID or URL pattern |
| `get_page_info` | Get detailed page information |
| `reload_page` | Reload the current page |

### API Tools

| Tool | Description |
|------|-------------|
| `call_api` | Call AuroraView Python API method |
| `list_api_methods` | List available API methods |
| `emit_event` | Emit event to frontend |

### UI Tools

| Tool | Description |
|------|-------------|
| `take_screenshot` | Capture page or element screenshot |
| `get_snapshot` | Get accessibility tree snapshot |
| `click` | Click on an element |
| `fill` | Fill input with text |
| `evaluate` | Execute JavaScript code |
| `hover` | Hover over an element |

### Gallery Tools

| Tool | Description |
|------|-------------|
| `get_samples` | List available samples |
| `run_sample` | Run a sample application |
| `stop_sample` | Stop a running sample |
| `get_sample_source` | Get sample source code |
| `list_processes` | List running sample processes |

### Debug Tools

| Tool | Description |
|------|-------------|
| `get_console_logs` | Get console log messages |
| `get_network_requests` | Get network request history |
| `get_backend_status` | Get Python backend status |
| `get_memory_info` | Get memory usage info |
| `clear_console` | Clear console logs |

## Usage Examples

### Basic Workflow

```
User: Help me test the Gallery search function

AI: I'll connect to Gallery and test the search.

[Call discover_instances]
→ Found 1 instance on port 9222

[Call connect(port=9222)]
→ Connected to AuroraView Gallery

[Call get_snapshot]
→ Got page structure, found search box

[Call fill(selector="input[placeholder*='Search']", value="cookie")]
→ Entered search term

[Call take_screenshot]
→ Screenshot shows search results

Search function works correctly.
```

### Running Samples

```
User: Run the hello_world sample

AI: [Call run_sample(name="hello_world")]
→ Sample started, PID: 12345

[Call list_processes]
→ Shows running processes

Sample successfully started.
```

### DCC Environment

```
User: Test the asset browser panel in Maya

AI: I'll connect to Maya's AuroraView panel.

[Call list_dcc_instances]
→ Found Maya 2025 instance, port 9223, panel: "Asset Browser"

[Call connect(port=9223)]
→ Connected to Maya Asset Browser

[Call get_page_info]
→ AuroraView ready, API methods available

Panel is working correctly in Maya.
```

## Resources

The server also provides MCP resources:

| Resource | Description |
|----------|-------------|
| `auroraview://instances` | List of running instances |
| `auroraview://page/{id}` | Page details |
| `auroraview://samples` | Available samples |
| `auroraview://sample/{name}/source` | Sample source code |
| `auroraview://logs` | Console logs |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AURORAVIEW_DEFAULT_PORT` | Default CDP port | `9222` |
| `AURORAVIEW_SCAN_PORTS` | Ports to scan (comma-separated) | `9222,9223,9224,9225` |
| `AURORAVIEW_EXAMPLES_DIR` | Path to examples directory | Auto-detected |
| `AURORAVIEW_DCC_MODE` | DCC mode (maya, blender, etc.) | None |

## Development

```bash
# Install dev dependencies
cd packages/auroraview-mcp
uv pip install -e ".[dev]"

# Run tests
pytest

# Run with coverage
pytest --cov=auroraview_mcp

# Type checking
mypy src/auroraview_mcp
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Assistant                            │
│                 (Claude, GPT, etc.)                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ MCP Protocol (stdio)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  AuroraView MCP Server                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │  Discovery  │ │   Tools     │ │     Resources       │   │
│  │   Module    │ │   Module    │ │      Module         │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
│         │               │                   │               │
│         └───────────────┼───────────────────┘               │
│                         │                                    │
│              ┌──────────┴──────────┐                        │
│              │   Connection Pool   │                        │
│              └──────────┬──────────┘                        │
└─────────────────────────┼───────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ WebView  │   │ WebView  │   │ WebView  │
    │ Instance │   │ Instance │   │ Instance │
    │  (CDP)   │   │  (CDP)   │   │  (CDP)   │
    └──────────┘   └──────────┘   └──────────┘
```

## Related

- [AuroraView](https://github.com/loonghao/auroraview) - Main project
- [MCP Protocol](https://modelcontextprotocol.io/) - Model Context Protocol
- [MCP Python SDK](https://github.com/modelcontextprotocol/python-sdk) - Official Python SDK

## License

MIT License - see [LICENSE](../../LICENSE) for details.
