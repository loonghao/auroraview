# -*- coding: utf-8 -*-
"""Base classes for AI providers."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional

if TYPE_CHECKING:
    from ..config import AIConfig
    from ..protocol import AGUIEvent


class AIProviderStrategy(ABC):
    """Abstract base class for AI provider strategies.

    Each provider (OpenAI, Anthropic, Gemini, etc.) implements this interface
    to handle completion requests in a provider-specific way.
    """

    def __init__(self, config: "AIConfig", event_emitter: Callable[["AGUIEvent"], None]):
        """Initialize the provider.

        Args:
            config: AI configuration (model, temperature, etc.)
            event_emitter: Callback to emit AG-UI events
        """
        self.config = config
        self._emit = event_emitter
        self._client: Optional[Any] = None

    @abstractmethod
    def get_env_key_name(self) -> str:
        """Get the environment variable name for the API key.

        Returns:
            Environment variable name (e.g., "OPENAI_API_KEY")
        """
        pass

    @abstractmethod
    def get_default_base_url(self) -> Optional[str]:
        """Get the default base URL for the provider.

        Returns:
            Base URL or None if not applicable
        """
        pass

    def get_api_key(self) -> str:
        """Get the API key from config or environment.

        Returns:
            API key string

        Raises:
            ValueError: If API key is not found
        """
        import os

        api_key = self.config.api_key or os.environ.get(self.get_env_key_name())
        if not api_key:
            raise ValueError(
                f"API key required for {self.__class__.__name__}. "
                f"Set {self.get_env_key_name()} environment variable."
            )
        return api_key

    @abstractmethod
    async def complete(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """Generate a completion from the AI provider.

        Args:
            messages: List of chat messages
            tools: Optional list of tool definitions
            stream: Whether to stream the response
            message_id: Unique message ID for event tracking

        Returns:
            The generated response text
        """
        pass

    def _emit_text_start(self, message_id: str) -> None:
        """Emit text generation start event."""
        from ..protocol import AGUIEvent

        self._emit(AGUIEvent.text_start(message_id))

    def _emit_text_delta(self, message_id: str, delta: str) -> None:
        """Emit text delta event."""
        from ..protocol import AGUIEvent

        self._emit(AGUIEvent.text_delta(message_id, delta))

    def _emit_text_end(self, message_id: str) -> None:
        """Emit text generation end event."""
        from ..protocol import AGUIEvent

        self._emit(AGUIEvent.text_end(message_id))
