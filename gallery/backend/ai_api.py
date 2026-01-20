# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AI Agent API for AuroraView Gallery.

This module provides the backend API handlers for the AI Agent sidebar,
enabling natural language interaction with the Gallery application.
"""

from __future__ import annotations

import asyncio
import logging
import os
import sys
import threading
from typing import Any, Callable, Dict, List, Optional, TYPE_CHECKING

# Add project root to path for imports
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))
if str(PROJECT_ROOT / "python") not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT / "python"))

from auroraview.ai import AIAgent, AIConfig  # noqa: E402
from auroraview.ai.config import AVAILABLE_MODELS, ProviderType, get_models_for_provider  # noqa: E402
from auroraview.ai.protocol import AGUIEvent  # noqa: E402

if TYPE_CHECKING:
    from auroraview import WebView

logger = logging.getLogger(__name__)

# Global AI agent instance
_ai_agent: Optional[AIAgent] = None
_agent_lock = threading.Lock()


def get_ai_agent() -> Optional[AIAgent]:
    """Get the global AI agent instance."""
    return _ai_agent


def _get_api_key_status() -> Dict[str, bool]:
    """Check which API keys are configured."""
    return {
        "openai": bool(os.environ.get("OPENAI_API_KEY")),
        "anthropic": bool(os.environ.get("ANTHROPIC_API_KEY")),
        "gemini": bool(os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")),
        "deepseek": bool(os.environ.get("DEEPSEEK_API_KEY")),
        "groq": bool(os.environ.get("GROQ_API_KEY")),
        "ollama": True,  # Ollama is local, always "available"
    }


def register_ai_apis(webview: "WebView") -> Dict[str, Callable]:
    """Register AI Agent API handlers with WebView.

    Args:
        webview: WebView instance to register APIs with

    Returns:
        Dict of API function references for cleanup
    """
    global _ai_agent

    # Create AI agent with the WebView
    # Default to GPT-4o if API key is available, otherwise use local Ollama
    api_keys = _get_api_key_status()
    logger.info("API key status checked.")

    if api_keys["openai"]:
        default_model = "gpt-4o"
    elif api_keys["anthropic"]:
        default_model = "claude-3-5-sonnet-20241022"
    elif api_keys["gemini"]:
        default_model = "gemini-2.0-flash-exp"
    elif api_keys["deepseek"]:
        default_model = "deepseek-chat"
    else:
        default_model = "llama3.2"  # Fallback to local Ollama

    logger.info("Selected default model: %s", default_model)

    with _agent_lock:
        _ai_agent = AIAgent(
            webview=webview,
            config=AIConfig(
                model=default_model,
                system_prompt="""You are an AI assistant integrated into AuroraView Gallery.
You help users explore and run code examples, understand AuroraView features,
and provide guidance on WebView development.

Available capabilities:
- Run Python examples from the gallery
- Explain code and concepts
- Help with AuroraView API usage
- Answer questions about WebView development

Be concise and helpful. When referencing code examples, mention them by name.""",
            ),
            auto_discover_apis=True,  # Discover bound APIs as tools
        )

    # Create thread-safe emitter once at registration time
    # This avoids calling create_emitter() from background threads
    _thread_safe_emitter = None

    def get_emitter():
        """Get or create thread-safe emitter lazily on main thread."""
        nonlocal _thread_safe_emitter
        if _thread_safe_emitter is None:
            try:
                _thread_safe_emitter = webview.create_emitter()
            except Exception as e:
                logger.warning("Failed to create emitter: %s", e)
                return None
        return _thread_safe_emitter

    # Initialize emitter on main thread
    get_emitter()

    # Set up event emission to WebView
    def emit_agui_event(event: AGUIEvent) -> None:
        """Forward AG-UI events to WebView."""
        try:
            emitter = get_emitter()
            if emitter:
                event_type = event.type.value.lower()
                emitter.emit(f"agui:{event_type}", event.to_dict())
        except Exception as e:
            logger.debug("Failed to emit AG-UI event: %s", e)

    _ai_agent.on_event(emit_agui_event)

    # Discover tools from bound APIs
    tool_count = _ai_agent.discover_tools()
    logger.info("AI Agent initialized with %d tools discovered", tool_count)

    # ==================== API Handlers ====================

    @webview.bind_call("ai.chat")
    def ai_chat(message: str, session_id: Optional[str] = None) -> Dict[str, Any]:
        """Send a message to the AI agent.

        Args:
            message: User message text
            session_id: Optional session ID for conversation continuity

        Returns:
            Dict with status and response or error
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        try:
            # Run async chat in sync context
            # Use chat_sync for simplicity in the Gallery demo
            response = _ai_agent.chat_sync(message, session_id=session_id)
            return {"status": "ok", "response": response}
        except ImportError as e:
            # Missing AI provider package
            return {
                "status": "error",
                "message": f"Missing AI package: {e}. Install with: pip install openai anthropic google-generativeai",
            }
        except Exception as e:
            logger.exception("Error in ai.chat")
            return {"status": "error", "message": str(e)}

    @webview.bind_call("ai.chat_stream")
    def ai_chat_stream(message: str, session_id: Optional[str] = None) -> Dict[str, Any]:
        """Send a message with streaming response.

        The response will be streamed via AG-UI events (agui:text_message_content).

        Args:
            message: User message text
            session_id: Optional session ID

        Returns:
            Dict with status (streaming is handled via events)
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        def run_async():
            try:
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
                loop.run_until_complete(_ai_agent.chat(message, session_id=session_id, stream=True))
            except Exception as e:
                logger.exception("Error in streaming chat")
                # Emit error event
                emit_agui_event(AGUIEvent.run_error("", str(e)))

        # Run in background thread
        thread = threading.Thread(target=run_async, daemon=True)
        thread.start()

        return {"status": "streaming"}

    @webview.bind_call("ai.get_config")
    def ai_get_config() -> Dict[str, Any]:
        """Get current AI configuration.

        Returns:
            Dict with model, temperature, provider, etc.
        """
        if not _ai_agent:
            return {}

        return {
            "model": _ai_agent.config.model,
            "temperature": _ai_agent.config.temperature,
            "max_tokens": _ai_agent.config.max_tokens,
            "provider": _ai_agent.config.infer_provider().value,
            "stream": _ai_agent.config.stream,
        }

    @webview.bind_call("ai.set_config")
    def ai_set_config(
        model: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None,
        system_prompt: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Update AI configuration.

        Args:
            model: New model name
            temperature: New temperature (0.0 - 2.0)
            max_tokens: New max tokens
            system_prompt: New system prompt

        Returns:
            Updated configuration
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        if model is not None:
            _ai_agent.config.model = model
        if temperature is not None:
            _ai_agent.config.temperature = max(0.0, min(2.0, temperature))
        if max_tokens is not None:
            _ai_agent.config.max_tokens = max_tokens
        if system_prompt is not None:
            _ai_agent.config.system_prompt = system_prompt

        return ai_get_config()

    @webview.bind_call("ai.get_models")
    def ai_get_models(provider: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get available AI models.

        Args:
            provider: Optional provider filter (openai, anthropic, etc.)

        Returns:
            List of model info dicts
        """
        api_keys = _get_api_key_status()

        if provider:
            try:
                ptype = ProviderType(provider)
                models = get_models_for_provider(ptype)
            except ValueError:
                models = []
        else:
            models = AVAILABLE_MODELS

        return [
            {
                "id": m.id,
                "name": m.name,
                "provider": m.provider.value,
                "description": m.description,
                "context_window": m.context_window,
                "supports_vision": m.supports_vision,
                "supports_tools": m.supports_tools,
                "available": api_keys.get(m.provider.value, False),
            }
            for m in models
        ]

    @webview.bind_call("ai.get_api_keys")
    def ai_get_api_keys() -> Dict[str, bool]:
        """Check which API keys are configured.

        Returns:
            Dict mapping provider name to availability
        """
        return _get_api_key_status()

    @webview.bind_call("ai.get_tools")
    def ai_get_tools() -> List[Dict[str, Any]]:
        """Get available AI tools.

        Returns:
            List of tool definitions
        """
        if not _ai_agent:
            return []

        return [
            {
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            }
            for t in _ai_agent.tools.all()
        ]

    @webview.bind_call("ai.execute_tool")
    def ai_execute_tool(name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Execute an AI tool directly.

        Args:
            name: Tool name
            arguments: Tool arguments

        Returns:
            Tool execution result
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        try:
            # Run async tool execution
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            result = loop.run_until_complete(_ai_agent.execute_tool(name, arguments))
            return {"status": "ok", "result": result}
        except Exception as e:
            logger.exception("Error executing tool %s", name)
            return {"status": "error", "message": str(e)}

    @webview.bind_call("ai.get_session")
    def ai_get_session(session_id: Optional[str] = None) -> Dict[str, Any]:
        """Get session information.

        Args:
            session_id: Optional specific session ID

        Returns:
            Session info with messages
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        session = _ai_agent.get_session(session_id)
        return {
            "id": session.id,
            "messages": [
                {
                    "id": m.id,
                    "role": m.role,
                    "content": m.content,
                }
                for m in session.messages
            ],
            "system_prompt": session.system_prompt,
        }

    @webview.bind_call("ai.clear_session")
    def ai_clear_session(session_id: Optional[str] = None) -> Dict[str, Any]:
        """Clear chat session.

        Args:
            session_id: Optional specific session ID

        Returns:
            Status dict
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        _ai_agent.clear_session(session_id)
        return {"status": "ok"}

    @webview.bind_call("ai.discover_tools")
    def ai_discover_tools() -> Dict[str, Any]:
        """Re-discover tools from bound APIs.

        Returns:
            Number of tools discovered
        """
        if not _ai_agent:
            return {"status": "error", "message": "AI agent not initialized"}

        count = _ai_agent.discover_tools()
        return {"status": "ok", "count": count}

    logger.info("Registered AI Agent APIs")

    return {
        "ai_chat": ai_chat,
        "ai_get_config": ai_get_config,
        "ai_set_config": ai_set_config,
        "ai_get_models": ai_get_models,
        "ai_get_tools": ai_get_tools,
    }


def cleanup_ai_agent() -> None:
    """Cleanup AI agent resources."""
    global _ai_agent
    with _agent_lock:
        _ai_agent = None
    logger.info("AI Agent cleaned up")
