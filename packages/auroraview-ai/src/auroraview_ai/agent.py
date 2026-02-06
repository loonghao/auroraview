# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView AI Agent implementation using Pydantic AI."""

from __future__ import annotations

import asyncio
import logging
import uuid
from collections.abc import AsyncIterator, Callable
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

import os

from pydantic_ai import Agent, RunContext
from pydantic_ai.models import KnownModelName
from pydantic_ai.providers.openai import OpenAIProvider

from auroraview_ai.config import AgentConfig, ProviderType, SidebarConfig
from auroraview_ai.protocol import AGUIEventEmitter, EmitCallback, EventType
from auroraview_ai.tools import DCCTool

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)


@dataclass
class AgentDeps:
    """Dependencies passed to agent tools."""

    session_id: str
    webview: Any | None = None
    emit_callback: EmitCallback | None = None


class AuroraAgent:
    """AI Agent for DCC applications using Pydantic AI.

    This agent provides natural language interaction capabilities for
    Digital Content Creation applications via AuroraView WebView.

    Example:
        >>> from auroraview_ai import AuroraAgent, AgentConfig
        >>>
        >>> agent = AuroraAgent(config=AgentConfig.openai())
        >>>
        >>> # Register tools
        >>> @agent.tool
        >>> def export_scene(format: str = "fbx") -> dict:
        ...     '''Export the current scene.'''
        ...     return {"status": "ok"}
        >>>
        >>> # Chat with streaming
        >>> async for delta in agent.chat_stream("Export the scene as FBX"):
        ...     print(delta, end="")
    """

    def __init__(
        self,
        config: AgentConfig | None = None,
        *,
        webview: Any | None = None,
        emit_callback: EmitCallback | None = None,
        sidebar_config: SidebarConfig | None = None,
        auto_discover_apis: bool = False,
    ):
        """Initialize the Aurora Agent.

        Args:
            config: Agent configuration
            webview: Optional WebView instance for API discovery
            emit_callback: Callback for emitting AG-UI events
            sidebar_config: Configuration for sidebar mode
            auto_discover_apis: Auto-discover APIs from WebView bindings
        """
        self.config = config or AgentConfig()
        self.webview = webview
        self.sidebar_config = sidebar_config
        self._emitter = AGUIEventEmitter(emit_callback)
        self._tools: list[DCCTool] = []
        self._sessions: dict[str, list[dict[str, Any]]] = {}
        self._active_session_id: str | None = None

        # Create Pydantic AI agent with proper provider configuration
        self._agent = self._create_agent()

        # Auto-discover if requested
        if auto_discover_apis and webview:
            self.discover_tools()

    def _create_agent(self) -> Agent[AgentDeps, str]:
        """Create Pydantic AI agent with proper provider configuration."""
        provider = self.config.infer_provider()
        model_name = self.config.get_pydantic_model_name()

        # DeepSeek uses OpenAI-compatible API but with its own endpoint
        if provider == ProviderType.DEEPSEEK:
            return self._create_deepseek_agent(model_name)

        # Default: use standard pydantic-ai initialization
        return Agent(
            model_name,  # type: ignore[arg-type]
            deps_type=AgentDeps,
            output_type=str,
            instructions=self.config.system_prompt or self._default_instructions(),
        )

    def _create_deepseek_agent(self, model_name: str) -> Agent[AgentDeps, str]:
        """Create agent configured for DeepSeek API.

        DeepSeek provides an OpenAI-compatible API endpoint.
        We need to use OpenAIProvider with custom base_url and api_key.
        """
        api_key = self.config.api_key or os.environ.get("DEEPSEEK_API_KEY")
        if not api_key:
            raise ValueError(
                "DeepSeek API key not found. Set DEEPSEEK_API_KEY environment variable "
                "or pass api_key in AgentConfig."
            )

        # Create OpenAI provider with DeepSeek endpoint
        provider = OpenAIProvider(
            base_url=self.config.base_url or "https://api.deepseek.com",
            api_key=api_key,
        )

        # Extract model name without 'openai:' prefix
        deepseek_model = model_name.replace("openai:", "")

        return Agent(
            deepseek_model,
            deps_type=AgentDeps,
            output_type=str,
            instructions=self.config.system_prompt or self._default_instructions(),
            provider=provider,
        )

    def _default_instructions(self) -> str:
        """Get default system instructions."""
        return """You are an AI assistant integrated into a DCC (Digital Content Creation) application.
You help users with their creative work by understanding natural language commands
and executing appropriate tools. Be concise and helpful."""

    def set_emit_callback(self, callback: EmitCallback) -> None:
        """Set the emit callback for AG-UI events."""
        self._emitter.set_callback(callback)

    def tool(
        self,
        func: Callable[..., Any] | None = None,
        *,
        name: str | None = None,
        description: str | None = None,
    ) -> Callable[..., Any]:
        """Decorator to register a tool with the agent.

        Can be used with or without arguments:
            @agent.tool
            def my_tool(): ...

            @agent.tool(name="custom_name")
            def my_tool(): ...
        """
        def decorator(f: Callable[..., Any]) -> Callable[..., Any]:
            tool_name = name or f.__name__
            tool_desc = description or f.__doc__ or f"Call {tool_name}"

            # Register with Pydantic AI agent
            @self._agent.tool
            async def wrapper(ctx: RunContext[AgentDeps], **kwargs: Any) -> Any:
                return await self._execute_tool(f, kwargs)

            # Update wrapper metadata
            wrapper.__name__ = tool_name
            wrapper.__doc__ = tool_desc

            self._tools.append(DCCTool(
                name=tool_name,
                description=tool_desc,
                handler=f,
            ))
            return f

        if func is not None:
            return decorator(func)
        return decorator

    async def _execute_tool(
        self,
        func: Callable[..., Any],
        kwargs: dict[str, Any],
    ) -> Any:
        """Execute a tool function."""
        import inspect
        if inspect.iscoroutinefunction(func):
            return await func(**kwargs)
        return func(**kwargs)

    def discover_tools(self) -> int:
        """Discover tools from WebView bound APIs."""
        if not self.webview:
            return 0
        # TODO: Implement WebView API discovery
        return 0

    def get_session(self, session_id: str | None = None) -> str:
        """Get or create a session ID."""
        if session_id is None:
            if self._active_session_id is None:
                self._active_session_id = str(uuid.uuid4())
            session_id = self._active_session_id
        if session_id not in self._sessions:
            self._sessions[session_id] = []
        return session_id

    def clear_session(self, session_id: str | None = None) -> None:
        """Clear a chat session."""
        sid = self.get_session(session_id)
        self._sessions[sid] = []

    async def chat(
        self,
        message: str,
        *,
        session_id: str | None = None,
    ) -> str:
        """Send a chat message and get a response.

        Args:
            message: User message
            session_id: Optional session ID for conversation continuity

        Returns:
            Assistant response text
        """
        sid = self.get_session(session_id)
        self._emitter.run_started(sid)

        try:
            deps = AgentDeps(
                session_id=sid,
                webview=self.webview,
                emit_callback=self._emitter._emit_callback,
            )

            # Get message history for context
            history = self._sessions.get(sid, [])

            # Run agent
            result = await self._agent.run(message, deps=deps)
            response = result.output

            # Store in session
            self._sessions[sid].append({"role": "user", "content": message})
            self._sessions[sid].append({"role": "assistant", "content": response})

            self._emitter.run_finished(sid)
            return response

        except Exception as e:
            self._emitter.run_error(str(e))
            raise

    async def chat_stream(
        self,
        message: str,
        *,
        session_id: str | None = None,
    ) -> AsyncIterator[str]:
        """Send a chat message with streaming response.

        Args:
            message: User message
            session_id: Optional session ID

        Yields:
            Text deltas as they arrive
        """
        sid = self.get_session(session_id)
        self._emitter.run_started(sid)

        try:
            deps = AgentDeps(
                session_id=sid,
                webview=self.webview,
                emit_callback=self._emitter._emit_callback,
            )

            self._emitter.text_start()
            full_response = ""

            async with self._agent.run_stream(message, deps=deps) as response:
                async for text in response.stream_text(delta=True):
                    self._emitter.text_delta(text)
                    full_response += text
                    yield text

            self._emitter.text_end()

            # Store in session
            self._sessions[sid].append({"role": "user", "content": message})
            self._sessions[sid].append({"role": "assistant", "content": full_response})

            self._emitter.run_finished(sid)

        except Exception as e:
            self._emitter.run_error(str(e))
            raise

    def chat_sync(
        self,
        message: str,
        *,
        session_id: str | None = None,
    ) -> str:
        """Synchronous version of chat().

        Args:
            message: User message
            session_id: Optional session ID

        Returns:
            Assistant response text
        """
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)

        return loop.run_until_complete(self.chat(message, session_id=session_id))

    def get_tools(self) -> list[DCCTool]:
        """Get all registered tools."""
        return self._tools.copy()

    @property
    def model(self) -> str:
        """Get current model name."""
        return self.config.model

    @model.setter
    def model(self, value: str) -> None:
        """Set model name and recreate agent."""
        self.config.model = value
        self._agent = self._create_agent()

