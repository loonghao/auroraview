# RFC 0002: 嵌入式 MCP Server 与 AI 测试调试方案

> **状态**: Draft
> **作者**: AuroraView Team
> **创建日期**: 2025-12-31
> **目标版本**: v0.5.0

## 摘要

本 RFC 提议为 AuroraView 框架实现嵌入式 MCP Server 能力，使开发者只需通过 `WebView(..., mcp=True)` 即可启动内置的 MCP 服务，实现 AI 助手对应用的实时调试、测试和控制。该方案采用 2025 年主流的 SSE (Server-Sent Events) 传输机制，支持动态 API 发现和暴露，为 AI 驱动的开发和测试提供无缝集成体验。

**核心目标**：
1. **零配置启动**：`WebView(mcp=True)` 一行代码启用 MCP 服务
2. **动态 API 发现**：自动暴露所有通过 `bind_call()` 注册的 API
3. **SSE 实时通信**：支持 AI 助手实时控制和监控应用
4. **DCC 环境支持**：在 Maya/Blender/Houdini 等 DCC 中同样可用
5. **AI 测试调试**：结合 2025 主流 AI Agent 测试方案

## 最新实现（Rust-only，2025-12 更新）
- MCP 现已完全由 Rust 实现并通过 PyO3 暴露，Python 版本已移除。
- 构建需启用 `--features mcp-server`，确保 `_core` 扩展提供 `McpConfig` / `McpServer`。
- WebView 中启用：
  ```python
  from auroraview import WebView

  def echo(message: str):
      """Echo back"""
      return {"echo": message}

  webview = WebView(
      title="MCP Demo",
      url="http://localhost:3000",
      mcp=True,              # 启用 MCP（自动分配端口）
      mcp_port=8765,         # 可选：固定端口
      mcp_name="auroraview-mcp",  # 可选：对外名称
  )

  webview.bind_call("api.echo", echo)  # 将自动暴露为 MCP 工具
  webview.show()
  ```
- 连接方式：SSE 端点为 `http://<host>:<port>/sse`，工具列表 `GET /tools`，调用 `POST /message` (JSON-RPC)。
- 在 IDE/Agent 中使用：加载上面的示例脚本，启动后在 Claude/Cursor 等配置 MCP Endpoint 指向输出端口即可。

## 动机


### 当前状态分析

RFC 0001 实现的 MCP Server 是独立运行的服务，需要：

1. **外部启动**：通过 `uvx auroraview-mcp` 或 Claude Desktop 配置启动
2. **CDP 依赖**：需要应用启用 `remote_debugging_port` 才能连接
3. **配置复杂**：需要配置端口、环境变量等
4. **无法内嵌**：MCP Server 与应用是分离的进程

### 2025 年 AI 测试调试趋势

| 趋势 | 描述 | 可借鉴 |
|------|------|--------|
| **MCP 内嵌** | 应用内置 MCP Server，无需外部配置 | FastMCP SSE 传输 |
| **AI Agent 测试** | 使用 AI 进行自动化测试和调试 | Midscene.js、Playwright AI |
| **动态 API 发现** | 运行时自动发现和暴露 API | OpenAPI、GraphQL Introspection |
| **SSE 实时通信** | 替代轮询，实现服务器推送 | MCP SSE Transport |
| **开发者体验优先** | 最小配置，即开即用 | Vite、Next.js |

### 需求分析

1. **开发者体验**：一行代码启用 MCP，无需额外配置
2. **动态发现**：自动暴露所有注册的 API 方法
3. **实时通信**：SSE 支持 AI 助手实时控制应用
4. **DCC 集成**：在 DCC 环境中同样可用
5. **测试调试**：支持 AI 驱动的自动化测试
6. **打包支持**：支持打包后的应用启用 MCP

## 设计方案

### 架构概览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         AI Assistant                                 │
│                    (Claude, Cursor, Copilot)                        │
└─────────────────────────────────────────────────────────────────────┘
                                │
                    MCP Protocol (SSE Transport)
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      AuroraView Application                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Embedded MCP Server                       │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │   │
│  │  │  SSE HTTP   │ │   Dynamic   │ │     Tool/Resource   │   │   │
│  │  │  Endpoint   │ │   Registry  │ │      Providers      │   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      WebView Core                            │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │   │
│  │  │  bind_call  │ │    emit     │ │     JS Bridge       │   │   │
│  │  │  handlers   │ │   events    │ │                     │   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 核心 API 设计

#### 1. WebView 参数扩展

```python
from auroraview import WebView

class MyTool:
    def __init__(self):
        self.webview = WebView(
            title="My Tool",
            width=800,
            height=600,
            url="http://localhost:3000",
            # 新增 MCP 参数
            mcp=True,                    # 启用嵌入式 MCP Server
            mcp_port=8765,               # MCP SSE 端口（可选，默认自动分配）
            mcp_name="my-tool",          # MCP Server 名称（可选）
        )
        
        # 注册 API 方法 - 自动暴露为 MCP Tools
        self.webview.bind_call("get_data", self.get_data)
        self.webview.bind_call("process", self.process)
        
    def get_data(self) -> dict:
        """获取数据 - 自动成为 MCP Tool"""
        return {"items": [...]}
    
    def process(self, data: dict) -> dict:
        """处理数据 - 自动成为 MCP Tool"""
        return {"result": "processed"}
```

#### 2. MCP 配置类

```python
from dataclasses import dataclass
from typing import Optional, List, Callable

@dataclass
class MCPConfig:
    """嵌入式 MCP Server 配置"""
    
    enabled: bool = False
    port: Optional[int] = None           # None = 自动分配
    host: str = "127.0.0.1"              # 默认只监听本地
    name: Optional[str] = None           # Server 名称
    version: str = "1.0.0"
    
    # 功能开关
    auto_expose_api: bool = True         # 自动暴露 bind_call 注册的方法
    expose_events: bool = True           # 暴露事件系统
    expose_state: bool = True            # 暴露状态管理
    expose_dom: bool = True              # 暴露 DOM 操作
    expose_debug: bool = True            # 暴露调试工具
    
    # 安全设置
    allowed_origins: List[str] = None    # 允许的来源（CORS）
    require_auth: bool = False           # 是否需要认证
    auth_token: Optional[str] = None     # 认证 Token
    
    # 高级设置
    max_connections: int = 10            # 最大连接数
    timeout: float = 30.0                # 请求超时（秒）
    
    @classmethod
    def from_bool(cls, value: bool) -> "MCPConfig":
        """从布尔值创建配置"""
        return cls(enabled=value)
    
    @classmethod
    def from_dict(cls, config: dict) -> "MCPConfig":
        """从字典创建配置"""
        return cls(**config)
```

#### 3. 嵌入式 MCP Server 实现

```python
# python/auroraview/mcp/embedded_server.py

from __future__ import annotations

import asyncio
import json
import threading
from typing import Any, Callable, Dict, List, Optional
from dataclasses import dataclass, field
from http.server import HTTPServer, BaseHTTPRequestHandler
import queue

@dataclass
class MCPTool:
    """MCP Tool 定义"""
    name: str
    description: str
    parameters: dict
    handler: Callable

@dataclass
class MCPResource:
    """MCP Resource 定义"""
    uri: str
    name: str
    description: str
    mime_type: str
    handler: Callable

class EmbeddedMCPServer:
    """嵌入式 MCP Server - SSE 传输"""
    
    def __init__(
        self,
        webview: "WebView",
        config: MCPConfig,
    ):
        self.webview = webview
        self.config = config
        self._tools: Dict[str, MCPTool] = {}
        self._resources: Dict[str, MCPResource] = {}
        self._server: Optional[HTTPServer] = None
        self._thread: Optional[threading.Thread] = None
        self._running = False
        self._sse_clients: List[queue.Queue] = []
        
    def start(self) -> int:
        """启动 MCP Server，返回端口号"""
        if self._running:
            return self._get_port()
        
        # 自动发现并注册工具
        if self.config.auto_expose_api:
            self._auto_register_tools()
        
        # 注册内置工具
        self._register_builtin_tools()
        
        # 启动 HTTP/SSE Server
        port = self.config.port or self._find_free_port()
        self._server = HTTPServer(
            (self.config.host, port),
            self._create_handler(),
        )
        
        self._thread = threading.Thread(target=self._run_server, daemon=True)
        self._thread.start()
        self._running = True
        
        return port
    
    def stop(self):
        """停止 MCP Server"""
        self._running = False
        if self._server:
            self._server.shutdown()
    
    def _auto_register_tools(self):
        """自动从 WebView 的 bind_call 注册工具"""
        # 获取所有注册的 API 方法
        handlers = self.webview._get_bound_handlers()
        
        for name, handler in handlers.items():
            # 从函数签名和文档字符串生成 MCP Tool
            tool = self._create_tool_from_handler(name, handler)
            self._tools[name] = tool
    
    def _register_builtin_tools(self):
        """注册内置工具"""
        
        # 截图工具
        self._tools["take_screenshot"] = MCPTool(
            name="take_screenshot",
            description="Take a screenshot of the WebView",
            parameters={
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Save path (optional)"},
                    "full_page": {"type": "boolean", "description": "Capture full page"},
                },
            },
            handler=self._take_screenshot,
        )
        
        # 执行 JS
        self._tools["evaluate"] = MCPTool(
            name="evaluate",
            description="Execute JavaScript in the WebView",
            parameters={
                "type": "object",
                "properties": {
                    "script": {"type": "string", "description": "JavaScript code"},
                },
                "required": ["script"],
            },
            handler=self._evaluate,
        )
        
        # 获取页面信息
        self._tools["get_page_info"] = MCPTool(
            name="get_page_info",
            description="Get current page information",
            parameters={"type": "object", "properties": {}},
            handler=self._get_page_info,
        )
        
        # 触发事件
        self._tools["emit_event"] = MCPTool(
            name="emit_event",
            description="Emit an event to the frontend",
            parameters={
                "type": "object",
                "properties": {
                    "event": {"type": "string", "description": "Event name"},
                    "data": {"type": "object", "description": "Event data"},
                },
                "required": ["event"],
            },
            handler=self._emit_event,
        )
        
        # DOM 操作
        if self.config.expose_dom:
            self._tools["click"] = MCPTool(
                name="click",
                description="Click an element by selector",
                parameters={
                    "type": "object",
                    "properties": {
                        "selector": {"type": "string", "description": "CSS selector"},
                    },
                    "required": ["selector"],
                },
                handler=self._click,
            )
            
            self._tools["fill"] = MCPTool(
                name="fill",
                description="Fill an input element",
                parameters={
                    "type": "object",
                    "properties": {
                        "selector": {"type": "string", "description": "CSS selector"},
                        "value": {"type": "string", "description": "Value to fill"},
                    },
                    "required": ["selector", "value"],
                },
                handler=self._fill,
            )
            
            self._tools["get_snapshot"] = MCPTool(
                name="get_snapshot",
                description="Get accessibility tree snapshot",
                parameters={"type": "object", "properties": {}},
                handler=self._get_snapshot,
            )
        
        # 调试工具
        if self.config.expose_debug:
            self._tools["get_console_logs"] = MCPTool(
                name="get_console_logs",
                description="Get console logs",
                parameters={
                    "type": "object",
                    "properties": {
                        "level": {"type": "string", "description": "Log level filter"},
                        "limit": {"type": "integer", "description": "Max logs to return"},
                    },
                },
                handler=self._get_console_logs,
            )
    
    def _create_handler(self):
        """创建 HTTP 请求处理器"""
        server = self
        
        class MCPHandler(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/sse":
                    # SSE 端点
                    self._handle_sse()
                elif self.path == "/health":
                    self._send_json({"status": "ok"})
                elif self.path == "/tools":
                    self._send_json(server._list_tools())
                elif self.path == "/resources":
                    self._send_json(server._list_resources())
                else:
                    self.send_error(404)
            
            def do_POST(self):
                if self.path == "/message":
                    # JSON-RPC 消息处理
                    content_length = int(self.headers.get("Content-Length", 0))
                    body = self.rfile.read(content_length)
                    message = json.loads(body)
                    response = server._handle_message(message)
                    self._send_json(response)
                else:
                    self.send_error(404)
            
            def _handle_sse(self):
                """处理 SSE 连接"""
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("Cache-Control", "no-cache")
                self.send_header("Connection", "keep-alive")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                
                # 创建消息队列
                client_queue = queue.Queue()
                server._sse_clients.append(client_queue)
                
                try:
                    # 发送初始化消息
                    init_msg = {
                        "jsonrpc": "2.0",
                        "method": "initialized",
                        "params": {
                            "protocolVersion": "2024-11-05",
                            "serverInfo": {
                                "name": server.config.name or "auroraview-embedded",
                                "version": server.config.version,
                            },
                            "capabilities": {
                                "tools": {"listChanged": True},
                                "resources": {"listChanged": True},
                            },
                        },
                    }
                    self._send_sse_event("message", init_msg)
                    
                    # 持续发送事件
                    while server._running:
                        try:
                            msg = client_queue.get(timeout=1.0)
                            self._send_sse_event("message", msg)
                        except queue.Empty:
                            # 发送心跳
                            self.wfile.write(b": heartbeat\n\n")
                            self.wfile.flush()
                finally:
                    server._sse_clients.remove(client_queue)
            
            def _send_sse_event(self, event: str, data: dict):
                """发送 SSE 事件"""
                self.wfile.write(f"event: {event}\n".encode())
                self.wfile.write(f"data: {json.dumps(data)}\n\n".encode())
                self.wfile.flush()
            
            def _send_json(self, data: dict):
                """发送 JSON 响应"""
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(json.dumps(data).encode())
            
            def log_message(self, format, *args):
                pass  # 禁用日志
        
        return MCPHandler
    
    def _handle_message(self, message: dict) -> dict:
        """处理 JSON-RPC 消息"""
        method = message.get("method")
        params = message.get("params", {})
        msg_id = message.get("id")
        
        try:
            if method == "tools/list":
                result = self._list_tools()
            elif method == "tools/call":
                tool_name = params.get("name")
                tool_args = params.get("arguments", {})
                result = self._call_tool(tool_name, tool_args)
            elif method == "resources/list":
                result = self._list_resources()
            elif method == "resources/read":
                uri = params.get("uri")
                result = self._read_resource(uri)
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "error": {"code": -32601, "message": f"Method not found: {method}"},
                }
            
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": result,
            }
        except Exception as e:
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "error": {"code": -32000, "message": str(e)},
            }
    
    def _list_tools(self) -> dict:
        """列出所有工具"""
        tools = []
        for name, tool in self._tools.items():
            tools.append({
                "name": name,
                "description": tool.description,
                "inputSchema": tool.parameters,
            })
        return {"tools": tools}
    
    def _call_tool(self, name: str, arguments: dict) -> dict:
        """调用工具"""
        if name not in self._tools:
            raise ValueError(f"Tool not found: {name}")
        
        tool = self._tools[name]
        result = tool.handler(**arguments)
        
        return {
            "content": [
                {"type": "text", "text": json.dumps(result) if isinstance(result, (dict, list)) else str(result)}
            ]
        }
    
    def broadcast_event(self, event: str, data: Any):
        """向所有 SSE 客户端广播事件"""
        message = {
            "jsonrpc": "2.0",
            "method": "notifications/event",
            "params": {"event": event, "data": data},
        }
        for client_queue in self._sse_clients:
            client_queue.put(message)
    
    # 内置工具实现
    async def _take_screenshot(self, path: str = None, full_page: bool = False) -> dict:
        """截图实现"""
        # 委托给 WebView
        return await self.webview.take_screenshot(path=path, full_page=full_page)
    
    async def _evaluate(self, script: str) -> Any:
        """执行 JS"""
        return await self.webview.evaluate_js(script)
    
    async def _get_page_info(self) -> dict:
        """获取页面信息"""
        return {
            "title": self.webview.title,
            "url": await self.webview.get_current_url(),
            "ready": self.webview.is_ready,
        }
    
    async def _emit_event(self, event: str, data: dict = None) -> dict:
        """触发事件"""
        self.webview.emit(event, data or {})
        return {"status": "emitted"}
    
    async def _click(self, selector: str) -> dict:
        """点击元素"""
        script = f'document.querySelector("{selector}")?.click()'
        await self.webview.evaluate_js(script)
        return {"status": "clicked", "selector": selector}
    
    async def _fill(self, selector: str, value: str) -> dict:
        """填充输入"""
        escaped_value = value.replace('"', '\\"')
        script = f'''
        (() => {{
            const el = document.querySelector("{selector}");
            if (el) {{
                el.value = "{escaped_value}";
                el.dispatchEvent(new Event("input", {{ bubbles: true }}));
                el.dispatchEvent(new Event("change", {{ bubbles: true }}));
            }}
        }})()
        '''
        await self.webview.evaluate_js(script)
        return {"status": "filled", "selector": selector}
    
    async def _get_snapshot(self) -> dict:
        """获取 A11y 快照"""
        script = '''
        (() => {
            function buildTree(node, depth = 0) {
                if (depth > 10) return null;
                const result = {
                    role: node.tagName?.toLowerCase() || 'text',
                    name: node.getAttribute?.('aria-label') || node.textContent?.slice(0, 50) || '',
                };
                if (node.children?.length > 0) {
                    result.children = Array.from(node.children)
                        .map(child => buildTree(child, depth + 1))
                        .filter(Boolean);
                }
                return result;
            }
            return buildTree(document.body);
        })()
        '''
        return await self.webview.evaluate_js(script)
    
    async def _get_console_logs(self, level: str = None, limit: int = 100) -> list:
        """获取控制台日志"""
        # 使用调试拦截器
        script = f'''
        (() => {{
            const logs = window.__auroraview_console_logs || [];
            let filtered = logs;
            if ("{level or ""}") {{
                filtered = logs.filter(log => log.level === "{level}");
            }}
            return filtered.slice(-{limit});
        }})()
        '''
        return await self.webview.evaluate_js(script)
```

### SSE 传输协议

#### 端点定义

| 端点 | 方法 | 描述 |
|------|------|------|
| `/sse` | GET | SSE 事件流端点 |
| `/message` | POST | JSON-RPC 消息处理 |
| `/health` | GET | 健康检查 |
| `/tools` | GET | 列出所有工具 |
| `/resources` | GET | 列出所有资源 |

#### SSE 事件格式

```
event: message
data: {"jsonrpc":"2.0","method":"initialized","params":{...}}

event: message
data: {"jsonrpc":"2.0","id":1,"result":{...}}

: heartbeat
```

### AI 助手配置

#### Claude Desktop / Cursor 配置

```json
{
  "mcpServers": {
    "my-tool": {
      "url": "http://127.0.0.1:8765/sse",
      "transport": {
        "type": "sse"
      }
    }
  }
}
```

#### 自动发现配置

```json
{
  "mcpServers": {
    "auroraview-discovery": {
      "command": "uvx",
      "args": ["auroraview-mcp", "--discover"],
      "env": {
        "AURORAVIEW_SCAN_PORTS": "8765-8775"
      }
    }
  }
}
```

### 使用场景

#### 场景 1：开发调试

```python
# my_tool.py
from auroraview import WebView

class MyTool:
    def __init__(self):
        self.webview = WebView(
            title="My Tool",
            url="http://localhost:3000",
            mcp=True,  # 启用 MCP
        )
        
        self.webview.bind_call("api.get_items", self.get_items)
        self.webview.bind_call("api.save_item", self.save_item)
    
    def get_items(self) -> list:
        return [{"id": 1, "name": "Item 1"}]
    
    def save_item(self, item: dict) -> dict:
        return {"status": "saved", "id": item["id"]}
    
    def run(self):
        self.webview.show()

if __name__ == "__main__":
    tool = MyTool()
    tool.run()
```

AI 助手交互：
```
User: 帮我测试这个工具的保存功能

AI: 我来连接并测试这个工具。

[连接到 http://127.0.0.1:8765/sse]

[调用 tools/list]
→ 发现工具: api.get_items, api.save_item, take_screenshot, evaluate, ...

[调用 api.get_items]
→ 返回 [{"id": 1, "name": "Item 1"}]

[调用 api.save_item({"id": 2, "name": "New Item"})]
→ 返回 {"status": "saved", "id": 2}

[调用 take_screenshot]
→ 截图保存成功

保存功能测试通过！API 正常工作。
```

#### 场景 2：DCC 环境

```python
# maya_tool.py
import maya.cmds as cmds
from auroraview import WebView
from auroraview.host.maya import MayaHost

class MayaTool:
    def __init__(self):
        host = MayaHost()
        self.webview = WebView(
            title="Maya Tool",
            url="http://localhost:3000",
            parent=host.get_main_window(),
            mcp=True,
            mcp_name="maya-tool",
        )
        
        # 暴露 Maya 相关 API
        self.webview.bind_call("maya.get_selection", self.get_selection)
        self.webview.bind_call("maya.create_cube", self.create_cube)
    
    def get_selection(self) -> list:
        return cmds.ls(selection=True)
    
    def create_cube(self, name: str = "cube1") -> str:
        return cmds.polyCube(name=name)[0]
```

#### 场景 3：自动化测试

```python
# test_my_tool.py
import pytest
from auroraview.testing import MCPTestClient

@pytest.fixture
async def tool_client():
    """启动工具并获取 MCP 客户端"""
    from my_tool import MyTool
    
    tool = MyTool()
    tool.webview.show()
    
    # 等待 MCP Server 启动
    client = MCPTestClient(port=tool.webview.mcp_port)
    await client.connect()
    
    yield client
    
    await client.disconnect()
    tool.webview.close()

async def test_get_items(tool_client):
    """测试获取项目"""
    result = await tool_client.call_tool("api.get_items")
    assert len(result) > 0
    assert "id" in result[0]

async def test_save_item(tool_client):
    """测试保存项目"""
    result = await tool_client.call_tool(
        "api.save_item",
        {"item": {"id": 99, "name": "Test"}}
    )
    assert result["status"] == "saved"

async def test_ui_interaction(tool_client):
    """测试 UI 交互"""
    # 点击按钮
    await tool_client.call_tool("click", {"selector": "#add-button"})
    
    # 填充表单
    await tool_client.call_tool("fill", {"selector": "#name-input", "value": "New Item"})
    
    # 截图验证
    screenshot = await tool_client.call_tool("take_screenshot")
    assert screenshot is not None
```

### 与现有 MCP Server 的关系

| 特性 | 外部 MCP Server (RFC 0001) | 嵌入式 MCP Server (本 RFC) |
|------|---------------------------|--------------------------|
| 启动方式 | `uvx auroraview-mcp` | `WebView(mcp=True)` |
| 传输协议 | stdio | SSE |
| 连接方式 | CDP | 直接 HTTP |
| 适用场景 | 连接任意 WebView | 单应用内嵌 |
| 配置复杂度 | 需要配置 | 零配置 |
| API 发现 | 需要 CDP 连接后发现 | 启动时自动暴露 |
| 多实例支持 | 支持 | 每个实例独立 |

**推荐使用场景**：

- **外部 MCP Server**：需要连接多个应用、DCC 环境调试、Gallery 管理
- **嵌入式 MCP Server**：单应用开发调试、自动化测试、AI 辅助开发

### 项目结构

```
python/auroraview/
├── mcp/
│   ├── __init__.py
│   ├── config.py           # MCPConfig 配置类
│   ├── embedded_server.py  # 嵌入式 MCP Server
│   ├── sse_handler.py      # SSE 传输处理
│   ├── tool_registry.py    # 工具注册表
│   └── resource_registry.py # 资源注册表
├── testing/
│   ├── __init__.py
│   └── mcp_client.py       # 测试用 MCP 客户端
└── core/
    └── webview.py          # 添加 mcp 参数支持
```

## 向后兼容性

### 兼容策略

1. **可选功能**：`mcp=True` 是可选参数，默认 `False`
2. **无破坏性变更**：现有代码无需修改
3. **与外部 MCP Server 共存**：可同时使用两种方式

### 依赖要求

```toml
[project.optional-dependencies]
mcp = [
    # 无额外依赖，使用标准库实现
]
```

## 实现计划

### Phase 1: 核心实现 (v0.5.0) - 2 周

- [ ] `MCPConfig` 配置类
- [ ] `EmbeddedMCPServer` 基础实现
- [ ] SSE 传输处理
- [ ] 自动 API 发现和注册
- [ ] 内置工具（截图、evaluate、DOM 操作）
- [ ] WebView 参数扩展（`mcp=True`）

### Phase 2: 测试框架 (v0.5.1) - 1 周

- [ ] `MCPTestClient` 测试客户端
- [ ] pytest 集成
- [ ] 示例测试用例

### Phase 3: DCC 集成 (v0.5.2) - 1 周

- [ ] Maya 环境测试
- [ ] Blender 环境测试
- [ ] DCC 特定工具暴露

### Phase 4: 高级功能 (v0.6.0) - 2 周

- [ ] 认证支持
- [ ] 多实例管理
- [ ] 性能监控工具
- [ ] 文档和示例

## 风险与缓解

| 风险 | 影响 | 缓解策略 |
|------|------|----------|
| 端口冲突 | 中 | 自动端口分配，支持配置 |
| 安全风险 | 高 | 默认只监听 localhost，支持认证 |
| 性能影响 | 低 | 惰性启动，按需加载 |
| DCC 线程问题 | 中 | 使用独立线程运行 HTTP Server |

## 参考资料

- [MCP SSE Transport Specification](https://spec.modelcontextprotocol.io/specification/basic/transports/#http-with-sse)
- [FastMCP Python SDK](https://github.com/jlowin/fastmcp)
- [RFC 0001: AuroraView MCP Server](./0001-auroraview-mcp-server.md)
- [Server-Sent Events (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-31 | Draft | 初始草案 |
