# -*- coding: utf-8 -*-
"""Anthropic provider implementation."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from .base import AIProviderStrategy


class AnthropicProvider(AIProviderStrategy):
    """Anthropic API provider.

    Supports Claude models via Anthropic's API.
    """

    def get_env_key_name(self) -> str:
        return "ANTHROPIC_API_KEY"

    def get_default_base_url(self) -> Optional[str]:
        return None

    async def complete(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        try:
            import anthropic
        except ImportError as err:
            raise ImportError("anthropic package required. Install: pip install anthropic") from err

        client = anthropic.AsyncAnthropic(
            api_key=self.get_api_key(),
            timeout=self.config.timeout,
        )

        # Extract system message (Anthropic handles it separately)
        system = None
        filtered_messages = []
        for msg in messages:
            if msg["role"] == "system":
                system = msg["content"]
            else:
                filtered_messages.append(msg)

        kwargs: Dict[str, Any] = {
            "model": self.config.model,
            "messages": filtered_messages,
            "max_tokens": self.config.max_tokens,
        }
        if system:
            kwargs["system"] = system
        if tools:
            kwargs["tools"] = tools

        if stream:
            return await self._stream_completion(client, kwargs, message_id)
        else:
            response = await client.messages.create(**kwargs)
            return response.content[0].text if response.content else ""

    async def _stream_completion(
        self,
        client: Any,
        kwargs: Dict[str, Any],
        message_id: str,
    ) -> str:
        self._emit_text_start(message_id)

        full_response = ""
        async with client.messages.stream(**kwargs) as stream_ctx:
            async for text in stream_ctx.text_stream:
                full_response += text
                self._emit_text_delta(message_id, text)

        self._emit_text_end(message_id)
        return full_response
