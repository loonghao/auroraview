# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView AI - Agent framework for DCC application integration.

This package provides a Pydantic AI-based agent system for integrating
AI capabilities into Digital Content Creation (DCC) applications via
AuroraView WebView.

Features:
- Multi-provider support (OpenAI, Anthropic, Google, etc.)
- AG-UI protocol for streaming events
- DCC tool auto-discovery from WebView bindings
- Session management and conversation history
"""

from auroraview_ai.agent import AuroraAgent
from auroraview_ai.config import (
    AVAILABLE_MODELS,
    AgentConfig,
    ModelInfo,
    ProviderType,
    SidebarConfig,
    get_model_by_id,
    get_models_for_provider,
)
from auroraview_ai.protocol import AGUIEvent, AGUIEventEmitter, EventType
from auroraview_ai.tools import DCCTool, DCCToolCategory, dcc_tool

__version__ = "0.1.0"

__all__ = [
    # Core
    "AuroraAgent",
    "AgentConfig",
    "SidebarConfig",
    "ProviderType",
    # Models
    "ModelInfo",
    "AVAILABLE_MODELS",
    "get_models_for_provider",
    "get_model_by_id",
    # Protocol
    "AGUIEventEmitter",
    "AGUIEvent",
    "EventType",
    # Tools
    "DCCTool",
    "DCCToolCategory",
    "dcc_tool",
    # Version
    "__version__",
]

