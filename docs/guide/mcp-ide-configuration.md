# MCP IDE Configuration Guide

This guide explains how to configure various IDEs and AI assistants to connect to AuroraView's embedded MCP Server.

## Transport Protocol

AuroraView uses **Streamable HTTP** transport (MCP Protocol 2025-03-26), which is the latest standard replacing the deprecated HTTP+SSE transport.

### Important: Accept Header Requirement

Per the [MCP Specification](https://spec.modelcontextprotocol.io/specification/2025-03-26/basic/transports/#streamable-http), clients **MUST** include an `Accept` header with both content types:

```
Accept: application/json, text/event-stream
```

This is a protocol requirement, not an AuroraView-specific limitation.

## Quick Start

After starting your AuroraView application with MCP enabled:

```python
from auroraview import WebView

webview = WebView(
    title="My App",
    mcp=True,
    mcp_port=8765,  # Optional: fixed port
    mcp_name="my-app"
)
webview.show()

# MCP endpoint: http://127.0.0.1:8765/mcp
```

---

## IDE Configuration

### Claude Desktop

**Config file location:**
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "auroraview": {
      "url": "http://127.0.0.1:8765/mcp",
      "transport": {
        "type": "streamable-http"
      }
    }
  }
}
```

### Cursor

**Config file:** `.cursor/mcp.json` in your project root

```json
{
  "mcpServers": {
    "auroraview": {
      "url": "http://127.0.0.1:8765/mcp",
      "transport": "streamable-http"
    }
  }
}
```

### VS Code with Continue

**Config file:** `~/.continue/config.json`

```json
{
  "mcpServers": [
    {
      "name": "auroraview",
      "transport": {
        "type": "streamable-http",
        "url": "http://127.0.0.1:8765/mcp"
      }
    }
  ]
}
```

### Windsurf

**Config file:** `~/.windsurf/mcp_config.json`

```json
{
  "mcpServers": {
    "auroraview": {
      "serverUrl": "http://127.0.0.1:8765/mcp",
      "transport": "streamable-http"
    }
  }
}
```

### Augment Code

Augment Code supports MCP servers via its settings. Add in your workspace or user settings:

```json
{
  "augment.mcp.servers": [
    {
      "name": "auroraview",
      "url": "http://127.0.0.1:8765/mcp",
      "transport": "streamable-http"
    }
  ]
}
```

### CodeBuddy (Tencent)

**Config file:** `~/.codebuddy/mcp.json` or project `.codebuddy/mcp.json`

```json
{
  "mcpServers": {
    "auroraview": {
      "url": "http://127.0.0.1:8765/mcp",
      "transportType": "streamable-http",
      "headers": {
        "Accept": "application/json, text/event-stream"
      }
    }
  }
}
```

### Cline (VS Code Extension)

**Config:** VS Code settings or `.vscode/cline_mcp_settings.json`

```json
{
  "mcpServers": {
    "auroraview": {
      "url": "http://127.0.0.1:8765/mcp",
      "transportType": "streamable-http",
      "autoApprove": ["list_windows", "api.get_samples"]
    }
  }
}
```

### Zed Editor

**Config file:** `~/.config/zed/settings.json`

```json
{
  "context_servers": {
    "auroraview": {
      "url": "http://127.0.0.1:8765/mcp"
    }
  }
}
```

### JetBrains IDEs (AI Assistant)

**Config file:** IDE Settings → Tools → AI Assistant → MCP Servers

```json
{
  "servers": [
    {
      "name": "auroraview",
      "url": "http://127.0.0.1:8765/mcp",
      "transport": "streamable-http"
    }
  ]
}
```

---

## Manual Testing

You can test the MCP endpoint using curl or PowerShell:

### Using curl

```bash
curl -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
```

### Using PowerShell

```powershell
$headers = @{
  "Accept" = "application/json, text/event-stream"
  "Content-Type" = "application/json"
}
$body = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
Invoke-RestMethod -Uri "http://127.0.0.1:8765/mcp" -Method Post -Headers $headers -Body $body
```

### Health Check

```bash
curl http://127.0.0.1:8765/health
# Response: {"status":"ok","transport":"streamable-http"}
```

---

## Troubleshooting

### "Not Acceptable" Error (HTTP 406)

**Cause:** Client is not sending the required `Accept` header.

**Solution:** Ensure your client includes:
```
Accept: application/json, text/event-stream
```

This is required by the MCP Streamable HTTP specification.

### Connection Refused

**Cause:** MCP server not running or wrong port.

**Solution:**
1. Check that your WebView has `mcp=True`
2. Verify the port in application logs
3. Ensure firewall allows local connections

### Tools Not Appearing

**Cause:** Tools not registered or auto_expose disabled.

**Solution:**
1. Set `auto_expose_api=True` in McpConfig
2. Ensure functions are registered with `@view.bind_call()`
3. Function names should not start with `_`

---

## Available Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/mcp` | POST | Main MCP JSON-RPC endpoint |
| `/mcp` | GET | SSE stream for server-initiated messages |
| `/health` | GET | Health check |
| `/tools` | GET | List registered tools (convenience) |

---

## See Also

- [MCP Usage Guide](../mcp-usage-guide.md)
- [MCP Prompts Guide](../mcp-prompts-guide.md)
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/specification/2025-03-26/)
- [RFC 0002: Embedded MCP Server](../rfcs/0002-embedded-mcp-server.md)

