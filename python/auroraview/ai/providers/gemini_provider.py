# -*- coding: utf-8 -*-
"""Google Gemini provider implementation."""

from __future__ import annotations

import os
from typing import Any, Dict, List, Optional

from .base import AIProviderStrategy


class GeminiProvider(AIProviderStrategy):
    """Google Gemini API provider.

    Supports Gemini models via Google's AI API.
    Uses the new google-genai SDK (REST-based, lightweight).
    Falls back to legacy google-generativeai if available.
    """

    def get_env_key_name(self) -> str:
        return "GEMINI_API_KEY"

    def get_default_base_url(self) -> Optional[str]:
        return None

    def get_api_key(self) -> str:
        api_key = (
            self.config.api_key
            or os.environ.get("GEMINI_API_KEY")
            or os.environ.get("GOOGLE_API_KEY")
        )
        if not api_key:
            raise ValueError("Gemini API key required. Set GEMINI_API_KEY or GOOGLE_API_KEY.")
        return api_key

    async def complete(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        # Try new lightweight SDK first
        try:
            from google import genai

            return await self._complete_new_sdk(genai, messages, tools, stream, message_id)
        except ImportError:
            pass

        # Fall back to legacy SDK
        try:
            import google.generativeai as genai_legacy

            return await self._complete_legacy_sdk(
                genai_legacy, messages, tools, stream, message_id
            )
        except ImportError as err:
            raise ImportError(
                "google-genai package required. Install: pip install google-genai"
            ) from err

    async def _complete_new_sdk(
        self,
        genai: Any,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        client = genai.Client(api_key=self.get_api_key())

        # Convert messages to Gemini format
        contents = []
        system_instruction = None

        for msg in messages:
            if msg["role"] == "system":
                system_instruction = msg["content"]
            elif msg["role"] == "user":
                contents.append({"role": "user", "parts": [{"text": msg["content"]}]})
            elif msg["role"] == "assistant":
                contents.append({"role": "model", "parts": [{"text": msg["content"]}]})

        config = {
            "temperature": self.config.temperature,
            "max_output_tokens": self.config.max_tokens,
        }
        if system_instruction:
            config["system_instruction"] = system_instruction

        if stream:
            return self._stream_new_sdk(client, contents, config, message_id)
        else:
            response = client.models.generate_content(
                model=self.config.model,
                contents=contents,
                config=config,
            )
            return response.text or ""

    def _stream_new_sdk(
        self,
        client: Any,
        contents: List[Dict[str, Any]],
        config: Dict[str, Any],
        message_id: str,
    ) -> str:
        self._emit_text_start(message_id)

        full_response = ""
        response = client.models.generate_content_stream(
            model=self.config.model,
            contents=contents,
            config=config,
        )
        for chunk in response:
            if chunk.text:
                full_response += chunk.text
                self._emit_text_delta(message_id, chunk.text)

        self._emit_text_end(message_id)
        return full_response

    async def _complete_legacy_sdk(
        self,
        genai: Any,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        if self.config.api_key:
            genai.configure(api_key=self.config.api_key)

        model = genai.GenerativeModel(self.config.model)

        # Convert messages to Gemini format
        history = []
        last_content = ""
        for msg in messages:
            if msg["role"] == "system":
                continue  # Gemini doesn't have system messages
            elif msg["role"] == "user":
                history.append({"role": "user", "parts": [msg["content"]]})
            elif msg["role"] == "assistant":
                history.append({"role": "model", "parts": [msg["content"]]})
            last_content = msg["content"]

        chat = model.start_chat(history=history[:-1] if history else [])

        gen_config = genai.types.GenerationConfig(
            temperature=self.config.temperature,
            max_output_tokens=self.config.max_tokens,
        )

        if stream:
            return await self._stream_legacy_sdk(chat, last_content, gen_config, message_id)
        else:
            response = await chat.send_message_async(last_content, generation_config=gen_config)
            return response.text or ""

    async def _stream_legacy_sdk(
        self,
        chat: Any,
        content: str,
        gen_config: Any,
        message_id: str,
    ) -> str:
        self._emit_text_start(message_id)

        full_response = ""
        response = await chat.send_message_async(content, stream=True, generation_config=gen_config)
        async for chunk in response:
            if chunk.text:
                full_response += chunk.text
                self._emit_text_delta(message_id, chunk.text)

        self._emit_text_end(message_id)
        return full_response
