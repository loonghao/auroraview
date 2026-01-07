# MCP IDE 配置指南

本指南介绍如何配置各种 IDE 和 AI 助手连接到 AuroraView 的嵌入式 MCP Server。

## 传输协议

AuroraView 使用 **Streamable HTTP** 传输协议 (MCP 2025-03-26 规范)，这是最新标准，取代了已弃用的 HTTP+SSE 传输。

### 重要：Accept 头要求

根据 [MCP 规范](https://spec.modelcontextprotocol.io/specification/2025-03-26/basic/transports/#streamable-http)，客户端 **必须** 包含同时支持两种内容类型的 Accept 头：

```
Accept: application/json, text/event-stream
```

这是协议要求，不是 AuroraView 的特殊限制。

## 快速开始

启动启用 MCP 的 AuroraView 应用后：

```python
from auroraview import WebView

webview = WebView(
    title="我的应用",
    mcp=True,
    mcp_port=8765,  # 可选：固定端口
    mcp_name="my-app"
)
webview.show()

# MCP 端点: http://127.0.0.1:8765/mcp
```

---

## IDE 配置

### Claude Desktop

**配置文件位置：**
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

**配置文件：** 项目根目录 `.cursor/mcp.json`

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

### VS Code + Continue

**配置文件：** `~/.continue/config.json`

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

**配置文件：** `~/.windsurf/mcp_config.json`

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

在工作区或用户设置中添加：

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

### CodeBuddy (腾讯)

**配置文件：** `~/.codebuddy/mcp.json` 或项目目录 `.codebuddy/mcp.json`

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

### Cline (VS Code 扩展)

**配置：** VS Code 设置或 `.vscode/cline_mcp_settings.json`

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

### Zed 编辑器

**配置文件：** `~/.config/zed/settings.json`

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

**配置：** IDE 设置 → Tools → AI Assistant → MCP Servers

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

## 手动测试

可以使用 curl 或 PowerShell 测试 MCP 端点：

### 使用 curl

```bash
curl -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
```

### 使用 PowerShell

```powershell
$headers = @{
  "Accept" = "application/json, text/event-stream"
  "Content-Type" = "application/json"
}
$body = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
Invoke-RestMethod -Uri "http://127.0.0.1:8765/mcp" -Method Post -Headers $headers -Body $body
```

### 健康检查

```bash
curl http://127.0.0.1:8765/health
# 响应: {"status":"ok","transport":"streamable-http"}
```

---

## 故障排除

### "Not Acceptable" 错误 (HTTP 406)

**原因：** 客户端未发送必需的 Accept 头。

**解决方案：** 确保客户端包含：
```
Accept: application/json, text/event-stream
```

这是 MCP Streamable HTTP 规范的要求。

### 连接被拒绝

**原因：** MCP 服务器未运行或端口错误。

**解决方案：**
1. 检查 WebView 是否设置了 `mcp=True`
2. 在应用日志中验证端口
3. 确保防火墙允许本地连接

### 工具未显示

**原因：** 工具未注册或 auto_expose 已禁用。

**解决方案：**
1. 在 McpConfig 中设置 `auto_expose_api=True`
2. 确保函数使用 `@view.bind_call()` 注册
3. 函数名不应以 `_` 开头

---

## 可用端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/mcp` | POST | 主 MCP JSON-RPC 端点 |
| `/mcp` | GET | 服务器发起消息的 SSE 流 |
| `/health` | GET | 健康检查 |
| `/tools` | GET | 列出已注册工具（便捷接口） |

---

## 另请参阅

- [MCP 使用指南](../../mcp-usage-guide.md)
- [MCP Prompts 指南](../../mcp-prompts-guide.md)
- [MCP 协议规范](https://spec.modelcontextprotocol.io/specification/2025-03-26/)
- [RFC 0002: 嵌入式 MCP 服务器](../../rfcs/0002-embedded-mcp-server.md)

