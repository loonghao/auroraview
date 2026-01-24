# -*- coding: utf-8 -*-
"""Generic OpenAI-compatible provider implementation."""

from __future__ import annotations

import os
from typing import Any, Dict, List, Optional

from ..config import ProviderType
from .base import AIProviderStrategy


class GenericProvider(AIProviderStrategy):
    """Generic OpenAI-compatible API provider.

    Works with DeepSeek, Groq, local models via Ollama, and other
    OpenAI-compatible APIs.
    """

    def get_env_key_name(self) -> str:
        provider = self.config.infer_provider()
        env_map = {
            ProviderType.DEEPSEEK: "DEEPSEEK_API_KEY",
            ProviderType.GROQ: "GROQ_API_KEY",
            ProviderType.XAI: "XAI_API_KEY",
            ProviderType.OLLAMA: "OLLAMA_API_KEY",  # Usually not needed
        }
        return env_map.get(provider, "OPENAI_API_KEY")

    def get_default_base_url(self) -> Optional[str]:
        provider = self.config.infer_provider()
        url_map = {
            ProviderType.DEEPSEEK: "https://api.deepseek.com",
            ProviderType.OLLAMA: "http://localhost:11434/v1",
            ProviderType.GROQ: "https://api.groq.com/openai/v1",
        }
        return url_map.get(provider)

    def get_api_key(self) -> str:
        provider = self.config.infer_provider()

        # Ollama doesn't need a real key
        if provider == ProviderType.OLLAMA:
            return self.config.api_key or "ollama"

        api_key = self.config.api_key or os.environ.get(self.get_env_key_name())
        if not api_key:
            raise ValueError(
                f"API key required for {provider.value}. "
                f"Set {self.get_env_key_name()} environment variable."
            )
        return api_key

    async def complete(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        try:
            import openai
        except ImportError as err:
            raise ImportError("openai package required. Install: pip install openai") from err

        provider = self.config.infer_provider()
        base_url = self.config.base_url or self.get_default_base_url()

        client = openai.AsyncOpenAI(
            api_key=self.get_api_key(),
            base_url=base_url,
            timeout=self.config.timeout,
        )

        kwargs: Dict[str, Any] = {
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        }
        # Some providers don't support tools
        if tools and provider not in (ProviderType.OLLAMA,):
            kwargs["tools"] = tools

        if stream:
            return await self._stream_completion(client, kwargs, message_id)
        else:
            response = await client.chat.completions.create(**kwargs)
            return response.choices[0].message.content or ""

    async def _stream_completion(
        self,
        client: Any,
        kwargs: Dict[str, Any],
        message_id: str,
    ) -> str:
        self._emit_text_start(message_id)

        full_response = ""
        async for chunk in await client.chat.completions.create(stream=True, **kwargs):
            if chunk.choices and chunk.choices[0].delta.content:
                delta = chunk.choices[0].delta.content
                full_response += delta
                self._emit_text_delta(message_id, delta)

        self._emit_text_end(message_id)
        return full_response
