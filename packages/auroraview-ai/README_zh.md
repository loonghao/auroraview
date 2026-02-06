# AuroraView AI

[English](README.md)

[![PyPI](https://img.shields.io/pypi/v/auroraview-ai.svg)](https://pypi.org/project/auroraview-ai/)
[![Python](https://img.shields.io/pypi/pyversions/auroraview-ai.svg)](https://pypi.org/project/auroraview-ai/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

基于 [Pydantic AI](https://ai.pydantic.dev/) 的 DCC 应用 AI Agent 框架。

## 特性

- 🤖 **多提供商支持** - OpenAI、Anthropic、Google Gemini 等
- 📡 **AG-UI 协议** - 实时 UI 更新的流式事件
- 🔧 **DCC 工具** - 从 WebView 绑定自动发现 API
- 💬 **会话管理** - 对话历史和上下文

## 安装

```bash
pip install auroraview-ai
```

安装所有提供商：
```bash
pip install auroraview-ai[all]
```

## 快速开始

```python
from auroraview_ai import AuroraAgent, AgentConfig

# 使用 OpenAI 创建 agent
agent = AuroraAgent(config=AgentConfig.openai())

# 注册工具
@agent.tool
def export_scene(format: str = "fbx") -> dict:
    """导出当前场景。"""
    return {"status": "ok", "format": format}

# 同步聊天
response = agent.chat_sync("将场景导出为 FBX")
print(response)

# 流式聊天
import asyncio

async def main():
    async for delta in agent.chat_stream("有哪些可用的工具？"):
        print(delta, end="", flush=True)

asyncio.run(main())
```

## 配置

```python
from auroraview_ai import AgentConfig, ProviderType

# OpenAI
config = AgentConfig.openai(model="gpt-4o")

# Anthropic
config = AgentConfig.anthropic(model="claude-sonnet-4-20250514")

# Google
config = AgentConfig.google(model="gemini-2.0-flash")

# 自定义
config = AgentConfig(
    provider=ProviderType.OPENAI,
    model="gpt-4o",
    api_key="sk-...",
    system_prompt="你是一个 DCC 助手。",
)
```

## DCC 集成

```python
from auroraview_ai import AuroraAgent, dcc_tool, DCCToolCategory

agent = AuroraAgent()

@dcc_tool(category=DCCToolCategory.SCENE, confirm=True)
def create_object(name: str, type: str = "cube") -> dict:
    """在场景中创建新对象。"""
    # DCC 特定实现
    return {"id": "obj_001", "name": name, "type": type}

# 注册到 agent
agent.tool(create_object)
```

## WebView 集成

```python
from auroraview import webview
from auroraview_ai import AuroraAgent

def emit_callback(event_name: str, data: dict):
    """发送事件到 WebView 前端。"""
    webview.emit(event_name, data)

agent = AuroraAgent(
    webview=webview,
    emit_callback=emit_callback,
    auto_discover_apis=True,
)
```

## AG-UI 事件

Agent 发射标准化的 AG-UI 事件：

| 事件 | 描述 |
|------|------|
| `RUN_STARTED` | Agent 开始处理 |
| `TEXT_MESSAGE_START` | 文本响应开始 |
| `TEXT_MESSAGE_CONTENT` | 文本增量（流式） |
| `TEXT_MESSAGE_END` | 文本响应完成 |
| `TOOL_CALL_START` | 工具调用开始 |
| `TOOL_CALL_END` | 工具调用完成 |
| `RUN_FINISHED` | Agent 完成处理 |
| `RUN_ERROR` | 发生错误 |

## 系统要求

- Python >= 3.10
- 支持的 DCC 应用：Maya 2024+、Houdini 20+、Blender 4+

## 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE)。

