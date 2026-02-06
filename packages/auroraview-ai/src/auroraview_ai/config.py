# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Configuration classes for AuroraView AI Agent."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class ProviderType(Enum):
    """Supported AI provider types."""

    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GOOGLE = "google"
    GEMINI = "gemini"  # Alias for google
    DEEPSEEK = "deepseek"
    OLLAMA = "ollama"
    GROQ = "groq"
    MISTRAL = "mistral"
    COHERE = "cohere"
    XAI = "xai"
    CUSTOM = "custom"


# Model name to Pydantic AI model string mapping
MODEL_MAPPING: dict[str, str] = {
    # OpenAI
    "gpt-4o": "openai:gpt-4o",
    "gpt-4o-mini": "openai:gpt-4o-mini",
    "gpt-4-turbo": "openai:gpt-4-turbo",
    "o1": "openai:o1",
    "o1-mini": "openai:o1-mini",
    # Anthropic
    "claude-3-5-sonnet-20241022": "anthropic:claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022": "anthropic:claude-3-5-haiku-20241022",
    "claude-sonnet-4-0": "anthropic:claude-sonnet-4-0",
    # Google/Gemini
    "gemini-2.0-flash-exp": "google-gla:gemini-2.0-flash-exp",
    "gemini-1.5-pro": "google-gla:gemini-1.5-pro",
    "gemini-1.5-flash": "google-gla:gemini-1.5-flash",
    # DeepSeek (via OpenAI-compatible API)
    "deepseek-chat": "openai:deepseek-chat",
    "deepseek-reasoner": "openai:deepseek-reasoner",
    # Groq
    "llama-3.3-70b-versatile": "groq:llama-3.3-70b-versatile",
    "mixtral-8x7b-32768": "groq:mixtral-8x7b-32768",
    # Ollama (local)
    "llama3.2": "ollama:llama3.2",
    "qwen2.5": "ollama:qwen2.5",
    "mistral": "ollama:mistral",
}


@dataclass
class AgentConfig:
    """AI Agent configuration.

    Attributes:
        model: Model identifier (e.g., "gpt-4o", "claude-sonnet-4-0")
        temperature: Sampling temperature (0.0 - 2.0)
        max_tokens: Maximum response tokens
        system_prompt: System prompt/instructions for the AI
        api_key: Optional API key override (uses env vars by default)
        base_url: Optional base URL for custom endpoints
        timeout: Request timeout in seconds
    """

    model: str = "gpt-4o"
    temperature: float = 0.7
    max_tokens: int = 4096
    system_prompt: str | None = None
    api_key: str | None = None
    base_url: str | None = None
    timeout: float = 60.0

    # Provider-specific settings
    provider_options: dict[str, Any] = field(default_factory=dict)

    def get_pydantic_model_name(self) -> str:
        """Get the Pydantic AI model name string."""
        if self.model in MODEL_MAPPING:
            return MODEL_MAPPING[self.model]
        # If already in pydantic format (e.g., "openai:gpt-4o"), return as-is
        if ":" in self.model:
            return self.model
        # Default: assume OpenAI-compatible
        return f"openai:{self.model}"

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

    @classmethod
    def openai(cls, model: str = "gpt-4o", **kwargs: Any) -> AgentConfig:
        """Create config for OpenAI models."""
        return cls(model=model, **kwargs)

    @classmethod
    def anthropic(cls, model: str = "claude-sonnet-4-0", **kwargs: Any) -> AgentConfig:
        """Create config for Anthropic Claude models."""
        return cls(model=model, **kwargs)

    @classmethod
    def gemini(cls, model: str = "gemini-2.0-flash-exp", **kwargs: Any) -> AgentConfig:
        """Create config for Google Gemini models."""
        return cls(model=model, **kwargs)


@dataclass
class SidebarConfig:
    """Configuration for AI Agent sidebar mode."""

    position: str = "right"
    width: int = 380
    min_width: int = 280
    max_width: int = 600
    collapsed: bool = False
    resizable: bool = True
    keyboard_shortcut: str = "Ctrl+Shift+A"
    theme: str = "auto"
    animation_duration: int = 200
    animation_easing: str = "ease-in-out"
    header_title: str = "AI Assistant"
    placeholder_text: str = "Ask me anything..."
    show_thinking: bool = True


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
AVAILABLE_MODELS: list[ModelInfo] = [
    # OpenAI
    ModelInfo("gpt-4o", "GPT-4o", ProviderType.OPENAI, "Most capable GPT-4", 128000, True, True),
    ModelInfo("gpt-4o-mini", "GPT-4o Mini", ProviderType.OPENAI, "Fast and affordable", 128000, True, True),
    ModelInfo("o1", "O1", ProviderType.OPENAI, "Reasoning model", 200000, False, False),
    ModelInfo("o1-mini", "O1 Mini", ProviderType.OPENAI, "Fast reasoning", 128000, False, False),
    # Anthropic
    ModelInfo("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", ProviderType.ANTHROPIC,
              "Most intelligent Claude", 200000, True, True),
    ModelInfo("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", ProviderType.ANTHROPIC,
              "Fast and efficient", 200000, False, True),
    ModelInfo("claude-sonnet-4-0", "Claude Sonnet 4", ProviderType.ANTHROPIC,
              "Latest Claude Sonnet", 200000, True, True),
    # Gemini
    ModelInfo("gemini-2.0-flash-exp", "Gemini 2.0 Flash", ProviderType.GOOGLE,
              "Latest Gemini", 1000000, True, True),
    ModelInfo("gemini-1.5-pro", "Gemini 1.5 Pro", ProviderType.GOOGLE,
              "Advanced reasoning", 2000000, True, True),
    # DeepSeek
    ModelInfo("deepseek-chat", "DeepSeek Chat", ProviderType.DEEPSEEK, "General chat", 64000, False, True),
    ModelInfo("deepseek-reasoner", "DeepSeek R1", ProviderType.DEEPSEEK, "Reasoning with CoT", 64000, False, True),
    # Ollama (local)
    ModelInfo("llama3.2", "Llama 3.2", ProviderType.OLLAMA, "Meta's open model", 128000, False, True),
    ModelInfo("qwen2.5", "Qwen 2.5", ProviderType.OLLAMA, "Alibaba multilingual", 128000, False, True),
]


def get_models_for_provider(provider: ProviderType) -> list[ModelInfo]:
    """Get available models for a specific provider."""
    return [m for m in AVAILABLE_MODELS if m.provider == provider]


def get_model_by_id(model_id: str) -> ModelInfo | None:
    """Get model info by ID."""
    for model in AVAILABLE_MODELS:
        if model.id == model_id:
            return model
    return None

