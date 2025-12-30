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

# Using uv (recommended)
uv pip install auroraview-mcp

# Using vx (auto-installs uv if needed)
vx uv pip install auroraview-mcp

# From source (development)
cd packages/auroraview-mcp
vx uv sync
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

## Local Debugging

AuroraView MCP Server provides multiple ways to debug locally during development.

### Method 1: FastMCP Built-in Client (Recommended for Unit Testing)

FastMCP 2.x includes a built-in `Client` that can test MCP servers without starting a separate process:

```bash
# Run all debug tests
just mcp-debug

# Interactive debug mode
just mcp-debug-interactive
```

Or run directly:

```bash
cd packages/auroraview-mcp
vx uv run python scripts/debug_client.py

# Test specific functionality
vx uv run python scripts/debug_client.py --test discover
vx uv run python scripts/debug_client.py --test connect --port 9222
vx uv run python scripts/debug_client.py --test interactive
```

Example debug script usage:

```python
from fastmcp import Client
from auroraview_mcp.server import mcp

async def test():
    async with Client(mcp) as client:
        # List all available tools
        tools = await client.list_tools()
        print(tools)
        
        # Call a tool
        result = await client.call_tool("discover_instances", {})
        print(result)

import asyncio
asyncio.run(test())
```

### Method 2: MCP Inspector (Recommended for Visual Debugging)

[MCP Inspector](https://github.com/modelcontextprotocol/inspector) provides a web-based UI for testing MCP servers:

```bash
# Start MCP Inspector
just mcp-inspector

# Or run directly
vx npx @modelcontextprotocol/inspector vx uv --directory packages/auroraview-mcp run auroraview-mcp
```

Then open http://localhost:5173 in your browser to:
- View all available tools and resources
- Call tools with custom parameters
- Inspect responses in real-time
- Debug connection issues

### Method 3: Direct Server Execution

Run the MCP server directly for stdio-based debugging:

```bash
# Start server in development mode
just mcp-dev

# Or run directly
cd packages/auroraview-mcp
vx uv run auroraview-mcp
```

### justfile Commands

| Command | Description |
|---------|-------------|
| `just mcp-dev` | Start MCP server in development mode |
| `just mcp-debug` | Run built-in debug client tests |
| `just mcp-debug-interactive` | Start interactive debug mode |
| `just mcp-inspector` | Launch MCP Inspector web UI |
| `just mcp-test` | Run unit tests |
| `just mcp-test-cov` | Run tests with coverage |
| `just mcp-lint` | Lint code with ruff |
| `just mcp-format` | Format code with ruff |
| `just mcp-build` | Build package |
| `just mcp-ci` | Run full CI check |

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
| `run_gallery` | Start the Gallery application |
| `stop_gallery` | Stop the running Gallery |
| `get_gallery_status` | Get Gallery running status |
| `get_samples` | List available samples |
| `run_sample` | Run a sample application |
| `stop_sample` | Stop a running sample |
| `get_sample_source` | Get sample source code |
| `list_processes` | List running sample processes |
| `stop_all_samples` | Stop all running samples |
| `get_project_info` | Get AuroraView project info |

### Debug Tools

| Tool | Description |
|------|-------------|
| `get_console_logs` | Get console log messages |
| `get_network_requests` | Get network request history |
| `get_backend_status` | Get Python backend status |
| `get_memory_info` | Get memory usage info |
| `clear_console` | Clear console logs |

### DCC Tools

| Tool | Description |
|------|-------------|
| `get_dcc_context` | Get current DCC environment context (scene, selection, frame) |
| `execute_dcc_command` | Execute DCC native commands (Maya cmds, Blender bpy, etc.) |
| `sync_selection` | Synchronize selection between DCC and WebView |
| `set_dcc_selection` | Set selection in DCC application |
| `get_dcc_scene_info` | Get detailed scene information |
| `get_dcc_timeline` | Get timeline/animation information |
| `set_dcc_frame` | Set current frame in DCC application |

## Usage Examples

### Start and Debug Gallery

```
User: Start the Gallery for debugging

AI: I'll start the Gallery application.

[Call run_gallery(port=9222)]
→ Gallery started, PID: 12345, port: 9222

[Call connect(port=9222)]
→ Connected to AuroraView Gallery

[Call get_page_info]
→ Gallery is ready with API methods available

Gallery is running and ready for debugging.
```

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

[Call get_dcc_context]
→ DCC: Maya 2025.1, Scene: /projects/scene.ma, Selection: ["pCube1"]

[Call execute_dcc_command(command="maya.cmds.ls", kwargs={"selection": True})]
→ Returns: ["pCube1", "pSphere1"]

[Call sync_selection]
→ DCC and WebView selections are in sync

Panel is working correctly in Maya.
```

### DCC Timeline Control

```
User: Go to frame 50 in Maya

AI: I'll set the current frame.

[Call get_dcc_timeline]
→ Current: 1, Range: 1-120, FPS: 24

[Call set_dcc_frame(frame=50)]
→ Frame set to 50

[Call get_dcc_scene_info]
→ Scene info with current frame at 50

Frame has been set to 50.
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
| `auroraview://gallery` | Gallery status and info |
| `auroraview://project` | Project information |
| `auroraview://processes` | Running sample processes |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AURORAVIEW_DEFAULT_PORT` | Default CDP port | `9222` |
| `AURORAVIEW_SCAN_PORTS` | Ports to scan (comma-separated) | `9222,9223,9224,9225` |
| `AURORAVIEW_EXAMPLES_DIR` | Path to examples directory | Auto-detected |
| `AURORAVIEW_GALLERY_DIR` | Path to Gallery directory | Auto-detected |
| `AURORAVIEW_PROJECT_ROOT` | Path to project root | Auto-detected |
| `AURORAVIEW_DCC_MODE` | DCC mode (maya, blender, etc.) | None |

## Development

```bash
# Install dev dependencies
cd packages/auroraview-mcp
vx uv sync

# Run tests
just mcp-test

# Run with coverage
just mcp-test-cov

# Lint and format
just mcp-lint
just mcp-format

# Full CI check
just mcp-ci
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
- [FastMCP](https://github.com/jlowin/fastmcp) - Fast, Pythonic MCP framework
- [MCP Inspector](https://github.com/modelcontextprotocol/inspector) - MCP debugging tool

## License

MIT License - see [LICENSE](../../LICENSE) for details.
