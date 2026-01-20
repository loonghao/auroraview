# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AI Agent configuration classes."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional


class ProviderType(Enum):
    """Supported AI provider types."""

    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GEMINI = "gemini"
    DEEPSEEK = "deepseek"
    OLLAMA = "ollama"
    GROQ = "groq"
    XAI = "xai"
    COHERE = "cohere"
    CUSTOM = "custom"


@dataclass
class AIConfig:
    """AI Agent configuration.

    Attributes:
        model: Model identifier (e.g., "gpt-4o", "claude-3-5-sonnet-20241022")
        temperature: Sampling temperature (0.0 - 2.0)
        max_tokens: Maximum response tokens
        system_prompt: System prompt for the AI
        stream: Enable streaming responses
        api_key: Optional API key override (uses env vars by default)
        base_url: Optional base URL for custom endpoints
        timeout: Request timeout in seconds
    """

    model: str = "gpt-4o"
    temperature: float = 0.7
    max_tokens: int = 4096
    system_prompt: Optional[str] = None
    stream: bool = True
    api_key: Optional[str] = None
    base_url: Optional[str] = None
    timeout: float = 60.0

    # Provider-specific settings
    provider_options: Dict[str, Any] = field(default_factory=dict)

    @classmethod
    def openai(cls, model: str = "gpt-4o", **kwargs) -> "AIConfig":
        """Create config for OpenAI models."""
        return cls(model=model, **kwargs)

    @classmethod
    def anthropic(cls, model: str = "claude-3-5-sonnet-20241022", **kwargs) -> "AIConfig":
        """Create config for Anthropic Claude models."""
        return cls(model=model, **kwargs)

    @classmethod
    def gemini(cls, model: str = "gemini-2.0-flash-exp", **kwargs) -> "AIConfig":
        """Create config for Google Gemini models."""
        return cls(model=model, **kwargs)

    @classmethod
    def deepseek(cls, model: str = "deepseek-chat", **kwargs) -> "AIConfig":
        """Create config for DeepSeek models."""
        return cls(model=model, **kwargs)

    @classmethod
    def ollama(cls, model: str = "llama3.2", **kwargs) -> "AIConfig":
        """Create config for local Ollama models."""
        return cls(model=model, **kwargs)

    def infer_provider(self) -> ProviderType:
        """Infer provider type from model name."""
        model_lower = self.model.lower()

        if model_lower.startswith("gpt-") or model_lower.startswith("o1"):
            return ProviderType.OPENAI
        elif model_lower.startswith("claude-"):
            return ProviderType.ANTHROPIC
        elif model_lower.startswith("gemini-"):
            return ProviderType.GEMINI
        elif model_lower.startswith("deepseek-"):
            return ProviderType.DEEPSEEK
        elif any(model_lower.startswith(p) for p in ["llama", "mistral", "phi", "qwen"]):
            return ProviderType.OLLAMA
        elif model_lower.startswith("grok-"):
            return ProviderType.XAI
        elif model_lower.startswith("command-"):
            return ProviderType.COHERE
        else:
            return ProviderType.CUSTOM


@dataclass
class SidebarConfig:
    """Configuration for AI Agent sidebar mode.

    Attributes:
        position: Sidebar position ("left" or "right")
        width: Initial width in pixels
        min_width: Minimum width when resizing
        max_width: Maximum width when resizing
        collapsed: Start collapsed
        resizable: Allow user to resize
        keyboard_shortcut: Shortcut to toggle (e.g., "Ctrl+Shift+A")
        theme: Color theme ("light", "dark", "auto")
    """

    position: str = "right"
    width: int = 380
    min_width: int = 280
    max_width: int = 600
    collapsed: bool = False
    resizable: bool = True
    keyboard_shortcut: str = "Ctrl+Shift+A"
    theme: str = "auto"

    # Animation settings
    animation_duration: int = 200  # ms
    animation_easing: str = "ease-in-out"

    # UI customization
    header_title: str = "AI Assistant"
    placeholder_text: str = "Ask me anything..."
    show_thinking: bool = True  # Show thinking/reasoning process


@dataclass
class ModelInfo:
    """Information about an AI model.

    Attributes:
        id: Model identifier
        name: Display name
        provider: Provider type
        description: Model description
        context_window: Maximum context window size
        supports_vision: Whether model supports image input
        supports_tools: Whether model supports tool/function calling
    """

    id: str
    name: str
    provider: ProviderType
    description: str = ""
    context_window: int = 128000
    supports_vision: bool = False
    supports_tools: bool = True


# Pre-defined models for quick access
AVAILABLE_MODELS: List[ModelInfo] = [
    # OpenAI
    ModelInfo("gpt-4o", "GPT-4o", ProviderType.OPENAI, "Most capable GPT-4", 128000, True, True),
    ModelInfo(
        "gpt-4o-mini", "GPT-4o Mini", ProviderType.OPENAI, "Fast and affordable", 128000, True, True
    ),
    ModelInfo("o1", "O1", ProviderType.OPENAI, "Reasoning model", 200000, False, False),
    ModelInfo("o1-mini", "O1 Mini", ProviderType.OPENAI, "Fast reasoning", 128000, False, False),
    # Anthropic
    ModelInfo(
        "claude-3-5-sonnet-20241022",
        "Claude 3.5 Sonnet",
        ProviderType.ANTHROPIC,
        "Most intelligent Claude",
        200000,
        True,
        True,
    ),
    ModelInfo(
        "claude-3-5-haiku-20241022",
        "Claude 3.5 Haiku",
        ProviderType.ANTHROPIC,
        "Fast and efficient",
        200000,
        False,
        True,
    ),
    # Gemini
    ModelInfo(
        "gemini-2.0-flash-exp",
        "Gemini 2.0 Flash",
        ProviderType.GEMINI,
        "Latest Gemini",
        1000000,
        True,
        True,
    ),
    ModelInfo(
        "gemini-1.5-pro",
        "Gemini 1.5 Pro",
        ProviderType.GEMINI,
        "Advanced reasoning",
        2000000,
        True,
        True,
    ),
    # DeepSeek
    ModelInfo(
        "deepseek-chat", "DeepSeek Chat", ProviderType.DEEPSEEK, "General chat", 64000, False, True
    ),
    ModelInfo(
        "deepseek-reasoner",
        "DeepSeek R1",
        ProviderType.DEEPSEEK,
        "Reasoning with CoT",
        64000,
        False,
        True,
    ),
    # Ollama (local)
    ModelInfo(
        "llama3.2", "Llama 3.2", ProviderType.OLLAMA, "Meta's open model", 128000, False, True
    ),
    ModelInfo(
        "qwen2.5", "Qwen 2.5", ProviderType.OLLAMA, "Alibaba multilingual", 128000, False, True
    ),
]


def get_models_for_provider(provider: ProviderType) -> List[ModelInfo]:
    """Get available models for a specific provider."""
    return [m for m in AVAILABLE_MODELS if m.provider == provider]


def get_model_by_id(model_id: str) -> Optional[ModelInfo]:
    """Get model info by ID."""
    for model in AVAILABLE_MODELS:
        if model.id == model_id:
            return model
    return None
