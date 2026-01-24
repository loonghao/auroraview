# -*- coding: utf-8 -*-
"""AI provider implementations."""

from __future__ import annotations

from typing import TYPE_CHECKING, Callable

from ..config import ProviderType
from .anthropic_provider import AnthropicProvider
from .base import AIProviderStrategy
from .gemini_provider import GeminiProvider
from .generic_provider import GenericProvider
from .openai_provider import OpenAIProvider

if TYPE_CHECKING:
    from ..config import AIConfig
    from ..protocol import AGUIEvent


def create_provider(
    config: "AIConfig",
    event_emitter: Callable[["AGUIEvent"], None],
) -> AIProviderStrategy:
    """Create an AI provider based on configuration.

    Args:
        config: AI configuration
        event_emitter: Callback to emit AG-UI events

    Returns:
        Appropriate AIProviderStrategy implementation
    """
    provider_type = config.infer_provider()

    if provider_type == ProviderType.OPENAI:
        return OpenAIProvider(config, event_emitter)
    elif provider_type == ProviderType.ANTHROPIC:
        return AnthropicProvider(config, event_emitter)
    elif provider_type == ProviderType.GEMINI:
        return GeminiProvider(config, event_emitter)
    else:
        # Generic provider for DeepSeek, Groq, Ollama, etc.
        return GenericProvider(config, event_emitter)


__all__ = [
    "AIProviderStrategy",
    "OpenAIProvider",
    "AnthropicProvider",
    "GeminiProvider",
    "GenericProvider",
    "create_provider",
]
