# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView AI Agent Module.

This module provides AI-powered assistant capabilities for AuroraView applications,
enabling natural language interaction with the UI and automatic API discovery.

Key Features:
- Multi-provider support (OpenAI, Anthropic, Gemini, DeepSeek, Ollama, etc.)
- AG-UI protocol for standardized AI-UI communication
- Automatic discovery and registration of WebView-bound APIs as tools
- Sidebar/drawer mode for attaching to existing WebViews
- Streaming responses with thinking/reasoning support

Example Usage:

    # Basic usage with WebView
    from auroraview import create_webview
    from auroraview.ai import AIAgent, AIConfig

    webview = create_webview(url="http://localhost:3000")

    # Create AI agent with auto-discovery of bound APIs
    agent = AIAgent(
        webview=webview,
        config=AIConfig(model="gpt-4o"),
        auto_discover_apis=True,
    )

    # Send a message
    response = await agent.chat("Help me export the scene")

    # Sidebar mode
    agent = AIAgent.as_sidebar(webview, config=AIConfig(model="gemini-2.0-flash"))
"""

from .config import AIConfig, SidebarConfig
from .agent import AIAgent
from .tools import Tool, ToolRegistry
from .protocol import AGUIEvent, EventType, sanitize_string, sanitize_dict, sanitize_value

__all__ = [
    # Core
    "AIAgent",
    "AIConfig",
    # Sidebar
    "SidebarConfig",
    # Tools
    "Tool",
    "ToolRegistry",
    # Protocol
    "AGUIEvent",
    "EventType",
    # Utils
    "sanitize_string",
    "sanitize_dict",
    "sanitize_value",
]
