# AuroraView MCP Server

[![PyPI version](https://badge.fury.io/py/auroraview-mcp.svg)](https://badge.fury.io/py/auroraview-mcp)
[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

AuroraView 的 MCP (Model Context Protocol) 服务器 - 使 AI 助手能够与独立模式和 DCC 环境（Maya、Blender、Houdini 等）中的 WebView 应用进行交互。

## 功能特性

- **实例发现**：自动发现运行中的 AuroraView 实例
- **CDP 连接**：通过 Chrome DevTools Protocol 连接到 WebView2 实例
- **API 桥接**：通过 JS 桥接调用 Python 后端 API
- **UI 自动化**：点击、填充、截图等操作
- **Gallery 集成**：运行和管理 AuroraView 示例
- **DCC 支持**：连接 Maya、Blender、Houdini、Nuke、Unreal 中的 AuroraView 面板

## 安装

```bash
# 使用 pip
pip install auroraview-mcp

# 使用 uv（推荐）
uv pip install auroraview-mcp

# 使用 vx（自动安装 uv）
vx uv pip install auroraview-mcp

# 从源码安装（开发）
cd packages/auroraview-mcp
vx uv sync
```

## 快速开始

### 配置 Claude Desktop / CodeBuddy

添加到 MCP 配置：

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

### 开发模式

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

## 本地调试

AuroraView MCP Server 提供多种本地调试方式。

### 方法 1：FastMCP 内置 Client（推荐用于单元测试）

FastMCP 2.x 包含内置的 `Client`，可以在不启动单独进程的情况下测试 MCP 服务器：

```bash
# 运行所有调试测试
just mcp-debug

# 交互式调试模式
just mcp-debug-interactive
```

或直接运行：

```bash
cd packages/auroraview-mcp
vx uv run python scripts/debug_client.py

# 测试特定功能
vx uv run python scripts/debug_client.py --test discover
vx uv run python scripts/debug_client.py --test connect --port 9222
vx uv run python scripts/debug_client.py --test interactive
```

调试脚本使用示例：

```python
from fastmcp import Client
from auroraview_mcp.server import mcp

async def test():
    async with Client(mcp) as client:
        # 列出所有可用工具
        tools = await client.list_tools()
        print(tools)
        
        # 调用工具
        result = await client.call_tool("discover_instances", {})
        print(result)

import asyncio
asyncio.run(test())
```

### 方法 2：MCP Inspector（推荐用于可视化调试）

[MCP Inspector](https://github.com/modelcontextprotocol/inspector) 提供基于 Web 的 UI 来测试 MCP 服务器：

```bash
# 启动 MCP Inspector
just mcp-inspector

# 或直接运行
vx npx @modelcontextprotocol/inspector vx uv --directory packages/auroraview-mcp run auroraview-mcp
```

然后在浏览器中打开 http://localhost:5173：
- 查看所有可用的工具和资源
- 使用自定义参数调用工具
- 实时检查响应
- 调试连接问题

### 方法 3：直接执行服务器

直接运行 MCP 服务器进行 stdio 调试：

```bash
# 以开发模式启动服务器
just mcp-dev

# 或直接运行
cd packages/auroraview-mcp
vx uv run auroraview-mcp
```

### justfile 命令

| 命令 | 描述 |
|------|------|
| `just mcp-dev` | 以开发模式启动 MCP 服务器 |
| `just mcp-debug` | 运行内置调试客户端测试 |
| `just mcp-debug-interactive` | 启动交互式调试模式 |
| `just mcp-inspector` | 启动 MCP Inspector Web UI |
| `just mcp-test` | 运行单元测试 |
| `just mcp-test-cov` | 运行带覆盖率的测试 |
| `just mcp-lint` | 使用 ruff 检查代码 |
| `just mcp-format` | 使用 ruff 格式化代码 |
| `just mcp-build` | 构建包 |
| `just mcp-ci` | 运行完整 CI 检查 |

## 可用工具

### 发现工具

| 工具 | 描述 |
|------|------|
| `discover_instances` | 发现所有运行中的 AuroraView 实例 |
| `connect` | 连接到 AuroraView 实例 |
| `disconnect` | 断开当前连接 |
| `list_dcc_instances` | 发现 DCC 环境中的实例 |

### 页面工具

| 工具 | 描述 |
|------|------|
| `list_pages` | 列出已连接实例中的所有页面 |
| `select_page` | 通过 ID 或 URL 模式选择页面 |
| `get_page_info` | 获取详细的页面信息 |
| `reload_page` | 重新加载当前页面 |

### API 工具

| 工具 | 描述 |
|------|------|
| `call_api` | 调用 AuroraView Python API 方法 |
| `list_api_methods` | 列出可用的 API 方法 |
| `emit_event` | 向前端触发事件 |

### UI 工具

| 工具 | 描述 |
|------|------|
| `take_screenshot` | 截取页面或元素截图 |
| `get_snapshot` | 获取可访问性树快照 |
| `click` | 点击元素 |
| `fill` | 填充输入框 |
| `evaluate` | 执行 JavaScript 代码 |
| `hover` | 悬停在元素上 |

### Gallery 工具

| 工具 | 描述 |
|------|------|
| `run_gallery` | 启动 Gallery 应用 |
| `stop_gallery` | 停止运行中的 Gallery |
| `get_gallery_status` | 获取 Gallery 运行状态 |
| `get_samples` | 列出可用示例 |
| `run_sample` | 运行示例应用 |
| `stop_sample` | 停止运行中的示例 |
| `get_sample_source` | 获取示例源代码 |
| `list_processes` | 列出运行中的示例进程 |
| `stop_all_samples` | 停止所有运行中的示例 |
| `get_project_info` | 获取 AuroraView 项目信息 |

### 调试工具

| 工具 | 描述 |
|------|------|
| `get_console_logs` | 获取控制台日志消息 |
| `get_network_requests` | 获取网络请求历史 |
| `get_backend_status` | 获取 Python 后端状态 |
| `get_memory_info` | 获取内存使用信息 |
| `clear_console` | 清除控制台日志 |

### DCC 工具

| 工具 | 描述 |
|------|------|
| `get_dcc_context` | 获取当前 DCC 环境上下文（场景、选择、帧） |
| `execute_dcc_command` | 执行 DCC 原生命令（Maya cmds、Blender bpy 等） |
| `sync_selection` | 同步 DCC 和 WebView 之间的选择状态 |
| `set_dcc_selection` | 在 DCC 应用中设置选择 |
| `get_dcc_scene_info` | 获取详细的场景信息 |
| `get_dcc_timeline` | 获取时间线/动画信息 |
| `set_dcc_frame` | 在 DCC 应用中设置当前帧 |

## 使用示例

### 启动并调试 Gallery

```
用户：启动 Gallery 进行调试

AI：我来启动 Gallery 应用。

[调用 run_gallery(port=9222)]
→ Gallery 已启动，PID: 12345，端口: 9222

[调用 connect(port=9222)]
→ 已连接到 AuroraView Gallery

[调用 get_page_info]
→ Gallery 已就绪，API 方法可用

Gallery 正在运行，可以开始调试。
```

### 基本工作流

```
用户：帮我测试 Gallery 的搜索功能

AI：我来连接 Gallery 并测试搜索。

[调用 discover_instances]
→ 发现 1 个实例在端口 9222

[调用 connect(port=9222)]
→ 已连接到 AuroraView Gallery

[调用 get_snapshot]
→ 获取页面结构，找到搜索框

[调用 fill(selector="input[placeholder*='Search']", value="cookie")]
→ 已输入搜索词

[调用 take_screenshot]
→ 截图显示搜索结果

搜索功能正常工作。
```

### 运行示例

```
用户：运行 hello_world 示例

AI：[调用 run_sample(name="hello_world")]
→ 示例已启动，PID: 12345

[调用 list_processes]
→ 显示运行中的进程

示例已成功启动。
```

### DCC 环境

```
用户：在 Maya 中测试资产浏览器面板

AI：我来连接 Maya 的 AuroraView 面板。

[调用 list_dcc_instances]
→ 发现 Maya 2025 实例，端口 9223，面板: "Asset Browser"

[调用 connect(port=9223)]
→ 已连接到 Maya Asset Browser

[调用 get_dcc_context]
→ DCC: Maya 2025.1，场景: /projects/scene.ma，选择: ["pCube1"]

[调用 execute_dcc_command(command="maya.cmds.ls", kwargs={"selection": True})]
→ 返回: ["pCube1", "pSphere1"]

[调用 sync_selection]
→ DCC 和 WebView 选择已同步

面板在 Maya 中正常工作。
```

### DCC 时间线控制

```
用户：在 Maya 中跳转到第 50 帧

AI：我来设置当前帧。

[调用 get_dcc_timeline]
→ 当前: 1，范围: 1-120，FPS: 24

[调用 set_dcc_frame(frame=50)]
→ 帧已设置为 50

[调用 get_dcc_scene_info]
→ 场景信息，当前帧为 50

帧已设置为 50。
```

## 资源

服务器还提供 MCP 资源：

| 资源 | 描述 |
|------|------|
| `auroraview://instances` | 运行中的实例列表 |
| `auroraview://page/{id}` | 页面详情 |
| `auroraview://samples` | 可用示例 |
| `auroraview://sample/{name}/source` | 示例源代码 |
| `auroraview://logs` | 控制台日志 |
| `auroraview://gallery` | Gallery 状态和信息 |
| `auroraview://project` | 项目信息 |
| `auroraview://processes` | 运行中的示例进程 |

## 环境变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `AURORAVIEW_DEFAULT_PORT` | 默认 CDP 端口 | `9222` |
| `AURORAVIEW_SCAN_PORTS` | 扫描端口（逗号分隔） | `9222,9223,9224,9225` |
| `AURORAVIEW_EXAMPLES_DIR` | 示例目录路径 | 自动检测 |
| `AURORAVIEW_GALLERY_DIR` | Gallery 目录路径 | 自动检测 |
| `AURORAVIEW_PROJECT_ROOT` | 项目根目录路径 | 自动检测 |
| `AURORAVIEW_DCC_MODE` | DCC 模式（maya、blender 等） | 无 |

## 开发

```bash
# 安装开发依赖
cd packages/auroraview-mcp
vx uv sync

# 运行测试
just mcp-test

# 运行覆盖率测试
just mcp-test-cov

# 检查和格式化代码
just mcp-lint
just mcp-format

# 完整 CI 检查
just mcp-ci
```

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                      AI 助手                                 │
│                 (Claude, GPT 等)                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ MCP 协议 (stdio)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  AuroraView MCP 服务器                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │  发现模块   │ │  工具模块   │ │     资源模块        │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
│         │               │                   │               │
│         └───────────────┼───────────────────┘               │
│                         │                                    │
│              ┌──────────┴──────────┐                        │
│              │     连接池          │                        │
│              └──────────┬──────────┘                        │
└─────────────────────────┼───────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ WebView  │   │ WebView  │   │ WebView  │
    │   实例   │   │   实例   │   │   实例   │
    │  (CDP)   │   │  (CDP)   │   │  (CDP)   │
    └──────────┘   └──────────┘   └──────────┘
```

## 相关链接

- [AuroraView](https://github.com/loonghao/auroraview) - 主项目
- [MCP 协议](https://modelcontextprotocol.io/) - Model Context Protocol
- [FastMCP](https://github.com/jlowin/fastmcp) - 快速、Pythonic 的 MCP 框架
- [MCP Inspector](https://github.com/modelcontextprotocol/inspector) - MCP 调试工具

## 许可证

MIT 许可证 - 详见 [LICENSE](../../LICENSE)。
