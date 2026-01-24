# -*- coding: utf-8 -*-
"""OpenAI provider implementation."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from .base import AIProviderStrategy


class OpenAIProvider(AIProviderStrategy):
    """OpenAI API provider.

    Supports OpenAI's chat completion API including GPT-4, GPT-3.5-turbo, etc.
    """

    def get_env_key_name(self) -> str:
        return "OPENAI_API_KEY"

    def get_default_base_url(self) -> Optional[str]:
        return None  # Uses default OpenAI URL

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

        client = openai.AsyncOpenAI(
            api_key=self.get_api_key(),
            base_url=self.config.base_url or self.get_default_base_url(),
            timeout=self.config.timeout,
        )

        kwargs: Dict[str, Any] = {
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        }
        if tools:
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
