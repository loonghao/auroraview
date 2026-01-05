# MCP API 使用指南

本指南介绍 AuroraView MCP Server 的最佳实践和使用方法。

## 快速开始

### 基本设置

```python
from auroraview import WebView

# 创建 WebView 并启用 MCP
view = WebView(
    title="My Application",
    url="http://localhost:3000",
    mcp=True,                    # 启用 MCP Server
    mcp_port=8765,               # 可选:固定端口
    mcp_name="my-app-mcp",        # 可选:Server 名称
    mcp_config=McpConfig(
        auto_expose_api=True,         # 自动暴露 bind_call 注册的 API
        host="127.0.0.1",
        timeout=30.0,
    )
)
```

### 注册 API 为 MCP 工具

```python
# 使用 bind_call 注册的 API 会自动暴露为 MCP 工具
@view.bind_call("api.get_user")
def get_user(user_id: str) -> dict:
    """Get user information by ID.

    这个 docstring 会自动被提取作为 MCP 工具描述。

    Args:
        user_id: The unique identifier of the user

    Returns:
        A dictionary containing user details:
        - id: User ID
        - name: User's full name
        - email: User's email address
    """
    # 返回标准格式
    return {
        "id": user_id,
        "name": "John Doe",
        "email": "john@example.com"
    }
```

## 使用 MCP 注解

### 只读工具

```python
from auroraview.mcp import McpServer

# 获取 MCP server 实例
mcp_server = view._mcp_server

# 标记工具为只读
mcp_server.register_tool("auroraview_get_config", get_config_handler)
```

在 Rust 端:
```rust
let tool = Tool::new("auroraview_get_config", "Get application configuration")
    .read_only()  # 添加 readOnlyHint=true
    .with_handler(|args| {
        // 只读操作
        Ok(config)
    });
```

### 幂等工具

```rust
let tool = Tool::new("auroraview_set_config", "Set application configuration")
    .idempotent()  // 添加 idempotentHint=true
    .with_handler(|args| {
        // 相同参数重复调用不会有额外效果
        Ok(())
    });
```

### 破坏性操作警告

```rust
let tool = Tool::new("auroraview_delete_user", "Delete a user account")
    .destructive()  // 添加 destructiveHint=true
    .with_handler(|args| {
        // 标记为可能执行破坏性操作
        Ok(())
    });
```

### 使用 Output Schema

```python
@view.bind_call("api.get_user")
def get_user(user_id: str) -> dict:
    """Get user information.

    Returns:
        User data with the following structure:
        {
            "id": string,      // User ID
            "name": string,    // Full name
            "email": string,   // Email address
            "created_at": string  // ISO 8601 timestamp
        }
    """
    return {
        "id": user_id,
        "name": "John Doe",
        "email": "john@example.com",
        "created_at": "2025-01-03T12:00:00Z"
    }
```

在 Rust 端,你也可以定义 output schema:
```rust
let tool = Tool::new("auroraview_get_user", "Get user information")
    .with_param("id", "string", "User ID")
    .with_output_schema(json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "name": {"type": "string"},
            "email": {"type": "string"}
        },
        "required": ["id", "name", "email"]
    }))
    .with_handler(|args| {
        // 返回符合 schema 的数据
        Ok(json!({
            "id": id,
            "name": "John Doe",
            "email": "john@example.com"
        }))
    });
```

## 错误处理最佳实践

### 使用标准响应格式

AuroraView 的 `bind_call` 自动规范化所有返回值为标准格式:

```python
from auroraview import ok, err

# 方法 1: 直接返回数据 (推荐用于简单情况)
@view.bind_call("api.get_user")
def get_user(id: int):
    user = database.find_user(id)
    return {"name": user.name, "id": user.id}
# JS 收到: {ok: true, data: {name: "John", id: 123}}

# 方法 2: 显式返回标准响应 (推荐用于错误处理)
@view.bind_call("api.delete_user")
def delete_user(id: int):
    if not user_exists(id):
        return err("User not found")
    database.delete(id)
    return ok({"deleted": id})

# 方法 3: 使用 ok() / err() 辅助函数 (最简洁)
@view.bind_call("api.create_user")
def create_user(name: str):
    if not name:
        return err("Name is required")
    user = database.create(name)
    return ok({"id": user.id, "name": user.name})
```

### 提供可操作的错误消息

```python
# 好的错误消息
@view.bind_call("api.get_user")
def get_user(user_id: str) -> dict:
    if not user_id:
        return err("User ID is required. Please provide a valid user ID.")
    
    user = database.find_user(user_id)
    if not user:
        return err(f"User '{user_id}' not found. Try listing users with api.list_users() first.")
    
    return ok(user)
```

### 返回不同类型

所有 JSON 可序列化的值都会被自动包装:

```python
# Dict
return {"name": "Alice"}
# JS: {ok: true, data: {name: "Alice"}}

# List
return [1, 2, 3]
# JS: {ok: true, data: [1, 2, 3]}

# String
return "hello"
# JS: {ok: true, data: "hello"}

# Number
return 42
# JS: {ok: true, data: 42}

# Boolean
return True
# JS: {ok: true, data: true}

# None
return None
# JS: {ok: true, data: null}
```

## 连接到 MCP Server

> 📚 完整的 IDE 配置指南请参考: [MCP IDE Configuration Guide](./guide/mcp-ide-configuration.md)

### 传输协议说明

AuroraView 使用 **Streamable HTTP** 传输协议 (MCP 2025-03-26 规范)。

根据 MCP 规范，客户端 **必须** 在请求中包含以下 Accept 头：

```
Accept: application/json, text/event-stream
```

这是协议标准要求，不是 AuroraView 的特殊限制。

### 在 Claude Desktop 中配置

**配置文件位置:**
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

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

### 其他 IDE 配置

支持的 IDE 和客户端包括:
- **Cursor** - `.cursor/mcp.json`
- **VS Code + Continue** - `~/.continue/config.json`
- **Windsurf** - `~/.windsurf/mcp_config.json`
- **Augment Code** - VS Code 设置
- **CodeBuddy** - `~/.codebuddy/mcp.json`
- **Cline** - `.vscode/cline_mcp_settings.json`
- **Zed** - `~/.config/zed/settings.json`
- **JetBrains AI Assistant** - IDE 设置

详细配置请参考 [MCP IDE Configuration Guide](./guide/mcp-ide-configuration.md)。

### 获取 MCP 端点信息

```python
@view.bind_call("api.get_mcp_info")
def get_mcp_info() -> dict:
    """Get MCP endpoint info for IDE/agent configuration."""
    config = getattr(view, "_mcp_config", None)
    enabled = getattr(view, "_mcp_enabled", False)
    port = getattr(view, "mcp_port", None)

    if not enabled or not config:
        return err("MCP is disabled in this session")

    if not port:
        return err("MCP server not started yet")

    host = getattr(config, "host", "127.0.0.1")
    name = getattr(config, "name", "auroraview-mcp")
    mcp_url = f"http://{host}:{port}/mcp"

    return ok({
        "name": name,
        "host": host,
        "port": port,
        "mcp_url": mcp_url,
        "tools_url": f"http://{host}:{port}/tools",
        "health_url": f"http://{host}:{port}/health",
    })
```

## 测试 MCP 工具

### 手动测试

```python
# 注册测试工具
@view.bind_call("api.test_echo")
def test_echo(message: str = "hello") -> dict:
    """Echo back the input message for testing."""
    return {"echo": message}

# 在 Claude 中测试
# User: Call api.test_echo with message "hello world"
# AI: [调用工具] -> 返回 {"echo": "hello world"}
```

### 自动化测试

```python
import pytest
import requests

def test_mcp_tools():
    # 获取 MCP 端点
    mcp_info = view._call_python_api("api.get_mcp_info", {})
    assert mcp_info["ok"]
    
    mcp_url = mcp_info["data"]["mcp_url"]
    
    # 列出工具
    response = requests.get(f"{mcp_info['data']['tools_url']}")
    tools = response.json()
    assert tools["ok"]
    assert len(tools["data"]) > 0
    
    # 测试工具调用
    tool_call = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "api.test_echo",
            "arguments": {"message": "test"}
        }
    }
    response = requests.post(mcp_url, json=tool_call)
    result = response.json()
    assert result["result"]["is_error"] is None
```

## 性能优化建议

### 1. 使用异步处理

```python
import asyncio

@view.bind_call("api.fetch_data")
async def fetch_data(url: str) -> dict:
    """Fetch data from URL asynchronously."""
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as response:
            data = await response.json()
    return ok(data)
```

### 2. 添加缓存

```python
from functools import lru_cache

@lru_cache(maxsize=100)
@view.bind_call("api.get_cached_data")
def get_cached_data(key: str) -> dict:
    """Get data with caching."""
    data = expensive_operation(key)
    return ok(data)
```

### 3. 限制数据量

```python
@view.bind_call("api.list_items")
def list_items(limit: int = 20, offset: int = 0) -> dict:
    """List items with pagination.

    Returns:
        {
            "items": [...],      // 项目列表
            "total": 150,        // 总数
            "offset": offset,       // 当前偏移
            "limit": limit,        // 限制
            "has_more": true        // 是否有更多
        }
    """
    items = database.get_items(limit=limit, offset=offset)
    total = database.count_items()
    has_more = offset + limit < total
    
    return ok({
        "items": items,
        "total": total,
        "offset": offset,
        "limit": limit,
        "has_more": has_more
    })
```

## 调试技巧

### 1. 查看 MCP 日志

```python
# 在启动时查看 MCP server 日志
# [MCP DEBUG] auto_expose_api=True, has_bound_functions=True
# [MCP DEBUG] Found 10 bound functions: [...]
# [MCP DEBUG] Registered tool: api.get_user
```

### 2. 测试工具注册

```python
# 列出所有已注册的工具
tools = view._mcp_server.list_tools()
print(f"Registered tools: {tools}")
```

### 3. 检查健康状态

```python
import requests

mcp_info = view._call_python_api("api.get_mcp_info", {})
health_url = mcp_info["data"]["health_url"]
response = requests.get(health_url)
print(response.json())  # {"status": "ok", "transport": "streamable-http"}
```

## 常见问题

### Q: MCP Server 没有启动?
A: 检查以下几点:
1. `mcp=True` 参数是否设置
2. 端口是否被占用
3. 查看 stderr 日志中的错误信息

### Q: 工具没有出现在 Claude 中?
A: 确保:
1. 工具使用 `@view.bind_call()` 注册
2. `auto_expose_api=True` 在配置中设置
3. 工具名称不以下划线开头
4. 检查 MCP 日志确认工具已注册

### Q: 如何调试工具调用?
A: 
1. 在 Python handler 中添加日志
2. 查看 stderr 输出
3. 使用 try-except 捕获异常并返回详细错误
4. 在 Claude 中查看工具返回的错误消息

## 参考资源

- [AuroraView MCP Optimization Summary](./mcp-optimization-summary.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io/specification/draft.md)
- [MCP Best Practices](https://modelcontextprotocol.io/sitemap.xml)
- [RFC 0003: API Design Guidelines](./rfcs/0003-api-design-guidelines.md)

---

**文档版本**: 1.0  
**创建日期**: 2025-01-03  
**作者**: AuroraView Team
