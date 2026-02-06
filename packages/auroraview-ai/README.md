# AuroraView AI

[中文文档](README_zh.md)

[![PyPI](https://img.shields.io/pypi/v/auroraview-ai.svg)](https://pypi.org/project/auroraview-ai/)
[![Python](https://img.shields.io/pypi/pyversions/auroraview-ai.svg)](https://pypi.org/project/auroraview-ai/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

AI Agent framework for DCC application integration with [Pydantic AI](https://ai.pydantic.dev/).

## Features

- 🤖 **Multi-Provider Support** - OpenAI, Anthropic, Google Gemini, and more
- 📡 **AG-UI Protocol** - Streaming events for real-time UI updates
- 🔧 **DCC Tools** - Auto-discover APIs from WebView bindings
- 💬 **Session Management** - Conversation history and context

## Installation

```bash
pip install auroraview-ai
```

With all providers:
```bash
pip install auroraview-ai[all]
```

## Quick Start

```python
from auroraview_ai import AuroraAgent, AgentConfig

# Create agent with OpenAI
agent = AuroraAgent(config=AgentConfig.openai())

# Register a tool
@agent.tool
def export_scene(format: str = "fbx") -> dict:
    """Export the current scene."""
    return {"status": "ok", "format": format}

# Chat (sync)
response = agent.chat_sync("Export the scene as FBX")
print(response)

# Chat with streaming
import asyncio

async def main():
    async for delta in agent.chat_stream("What tools are available?"):
        print(delta, end="", flush=True)

asyncio.run(main())
```

## Configuration

```python
from auroraview_ai import AgentConfig, ProviderType

# OpenAI
config = AgentConfig.openai(model="gpt-4o")

# Anthropic
config = AgentConfig.anthropic(model="claude-sonnet-4-20250514")

# Google
config = AgentConfig.google(model="gemini-2.0-flash")

# Custom
config = AgentConfig(
    provider=ProviderType.OPENAI,
    model="gpt-4o",
    api_key="sk-...",
    system_prompt="You are a DCC assistant.",
)
```

## DCC Integration

```python
from auroraview_ai import AuroraAgent, dcc_tool, DCCToolCategory

agent = AuroraAgent()

@dcc_tool(category=DCCToolCategory.SCENE, confirm=True)
def create_object(name: str, type: str = "cube") -> dict:
    """Create a new object in the scene."""
    # DCC-specific implementation
    return {"id": "obj_001", "name": name, "type": type}

# Register with agent
agent.tool(create_object)
```

## WebView Integration

```python
from auroraview import webview
from auroraview_ai import AuroraAgent

def emit_callback(event_name: str, data: dict):
    """Send events to WebView frontend."""
    webview.emit(event_name, data)

agent = AuroraAgent(
    webview=webview,
    emit_callback=emit_callback,
    auto_discover_apis=True,
)
```

## AG-UI Events

The agent emits standardized AG-UI events:

| Event | Description |
|-------|-------------|
| `RUN_STARTED` | Agent started processing |
| `TEXT_MESSAGE_START` | Text response started |
| `TEXT_MESSAGE_CONTENT` | Text delta (streaming) |
| `TEXT_MESSAGE_END` | Text response completed |
| `TOOL_CALL_START` | Tool invocation started |
| `TOOL_CALL_END` | Tool invocation completed |
| `RUN_FINISHED` | Agent finished processing |
| `RUN_ERROR` | Error occurred |

## Requirements

- Python >= 3.10
- Supported DCC applications: Maya 2024+, Houdini 20+, Blender 4+

## License

MIT License - see [LICENSE](LICENSE) for details.

