# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AI Agent implementation for AuroraView."""

from __future__ import annotations

import asyncio
import json
import logging
import uuid
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional

from .config import AIConfig, ProviderType, SidebarConfig
from .protocol import (
    AGUIEvent,
    Session,
    sanitize_list,
)
from .tools import Tool, ToolRegistry

if TYPE_CHECKING:
    from ..core.webview import WebView

logger = logging.getLogger(__name__)


class AIAgent:
    """AI Agent for natural language interaction with AuroraView applications.

    The AIAgent provides a bridge between AI language models and your application,
    enabling:
    - Natural language commands
    - Automatic tool/function calling based on bound APIs
    - Streaming responses with AG-UI protocol events
    - Sidebar/drawer mode for attachment to existing WebViews

    Example:
        >>> from auroraview import create_webview
        >>> from auroraview.ai import AIAgent, AIConfig
        >>>
        >>> webview = create_webview(url="http://localhost:3000")
        >>>
        >>> # Bind some APIs
        >>> @webview.bind_call("api.export_scene")
        >>> def export_scene(format: str = "fbx"):
        ...     '''Export the current scene'''
        ...     return {"status": "ok"}
        >>>
        >>> # Create AI agent with auto-discovery
        >>> agent = AIAgent(webview=webview, config=AIConfig.openai())
        >>> agent.discover_tools()  # Finds export_scene as a tool
        >>>
        >>> # Chat with the AI
        >>> response = await agent.chat("Export the scene as FBX")
    """

    def __init__(
        self,
        webview: Optional["WebView"] = None,
        config: Optional[AIConfig] = None,
        *,
        auto_discover_apis: bool = False,
        sidebar_config: Optional[SidebarConfig] = None,
    ):
        """Initialize the AI Agent.

        Args:
            webview: WebView instance for API discovery and event emission
            config: AI configuration (model, temperature, etc.)
            auto_discover_apis: Automatically discover bound APIs as tools
            sidebar_config: Configuration for sidebar mode (if used)
        """
        self.webview = webview
        self.config = config or AIConfig()
        self.sidebar_config = sidebar_config
        self.tools = ToolRegistry()
        self._sessions: Dict[str, Session] = {}
        self._active_session_id: Optional[str] = None
        self._event_handlers: List[Callable[[AGUIEvent], None]] = []
        self._client: Optional[Any] = None

        # Auto-discover APIs if requested and webview is provided
        if auto_discover_apis and webview:
            self.discover_tools()

    @classmethod
    def as_sidebar(
        cls,
        webview: "WebView",
        config: Optional[AIConfig] = None,
        *,
        sidebar_config: Optional[SidebarConfig] = None,
        auto_discover_apis: bool = True,
    ) -> "AIAgent":
        """Create an AI Agent in sidebar mode attached to a WebView.

        This creates the agent and sets up the sidebar UI that can be toggled
        with the configured keyboard shortcut.

        Args:
            webview: WebView to attach sidebar to
            config: AI configuration
            sidebar_config: Sidebar UI configuration
            auto_discover_apis: Automatically discover bound APIs

        Returns:
            AIAgent instance configured for sidebar mode

        Example:
            >>> agent = AIAgent.as_sidebar(
            ...     webview,
            ...     config=AIConfig.gemini(),
            ...     sidebar_config=SidebarConfig(position="right", width=400),
            ... )
        """
        agent = cls(
            webview=webview,
            config=config,
            auto_discover_apis=auto_discover_apis,
            sidebar_config=sidebar_config or SidebarConfig(),
        )
        agent._enable_sidebar_mode()
        return agent

    def _enable_sidebar_mode(self) -> None:
        """Enable sidebar mode by injecting UI and handlers."""
        if not self.webview or not self.sidebar_config:
            return

        # Register sidebar API handlers
        self._register_sidebar_apis()

        # Inject sidebar CSS and HTML
        self._inject_sidebar_ui()

        logger.info(
            "AI Agent sidebar enabled (shortcut: %s)",
            self.sidebar_config.keyboard_shortcut,
        )

    def _register_sidebar_apis(self) -> None:
        """Register API handlers for sidebar communication."""
        if not self.webview:
            return

        @self.webview.bind_call("ai.chat")
        def ai_chat(message: str) -> Dict[str, Any]:
            """Send a message to the AI agent."""
            # Run async chat in event loop
            try:
                loop = asyncio.get_event_loop()
                if loop.is_running():
                    # Schedule the coroutine - result will be sent via event
                    asyncio.ensure_future(self._async_chat_and_emit(message))
                    return {"status": "pending", "message": "Processing..."}
                else:
                    result = loop.run_until_complete(self.chat(message))
                    return {"status": "ok", "response": result}
            except Exception as e:
                logger.exception("Error in ai.chat")
                return {"status": "error", "message": str(e)}

        @self.webview.bind_call("ai.get_config")
        def ai_get_config() -> Dict[str, Any]:
            """Get current AI configuration."""
            return {
                "model": self.config.model,
                "temperature": self.config.temperature,
                "max_tokens": self.config.max_tokens,
                "provider": self.config.infer_provider().value,
            }

        @self.webview.bind_call("ai.set_model")
        def ai_set_model(model: str) -> Dict[str, Any]:
            """Change the AI model."""
            self.config.model = model
            return {"status": "ok", "model": model}

        @self.webview.bind_call("ai.get_tools")
        def ai_get_tools() -> List[Dict[str, Any]]:
            """Get available tools."""
            return [
                {
                    "name": t.name,
                    "description": t.description,
                }
                for t in self.tools.all()
            ]

        @self.webview.bind_call("ai.clear_session")
        def ai_clear_session() -> Dict[str, Any]:
            """Clear the current chat session."""
            self.clear_session()
            return {"status": "ok"}

        logger.debug("Registered AI sidebar APIs")

    async def _async_chat_and_emit(self, message: str) -> None:
        """Process chat asynchronously and emit result via event.

        This method is called when ai.chat is invoked from an already running
        event loop. The result will be sent back to the frontend via an event.

        Args:
            message: User message to process
        """
        try:
            result = await self.chat(message)
            if self.webview:
                self.webview.emit("ai.chat.response", {"status": "ok", "response": result})
        except Exception as e:
            logger.exception("Error in async chat")
            if self.webview:
                self.webview.emit("ai.chat.response", {"status": "error", "message": str(e)})

    def _inject_sidebar_ui(self) -> None:
        """Inject sidebar UI into the WebView."""
        if not self.webview or not self.sidebar_config:
            return

        cfg = self.sidebar_config

        # CSS for sidebar
        css = f"""
        .ai-sidebar {{
            position: fixed;
            top: 0;
            {cfg.position}: 0;
            width: {cfg.width}px;
            height: 100vh;
            background: var(--ai-sidebar-bg, #1e1e1e);
            border-{("left" if cfg.position == "right" else "right")}: 1px solid var(--ai-sidebar-border, #333);
            display: flex;
            flex-direction: column;
            z-index: 10000;
            transform: translateX({"100%" if cfg.position == "right" else "-100%"});
            transition: transform {cfg.animation_duration}ms {cfg.animation_easing};
        }}
        .ai-sidebar.open {{
            transform: translateX(0);
        }}
        .ai-sidebar-header {{
            padding: 12px 16px;
            border-bottom: 1px solid var(--ai-sidebar-border, #333);
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        .ai-sidebar-title {{
            font-weight: 600;
            color: var(--ai-sidebar-text, #fff);
        }}
        .ai-sidebar-close {{
            background: none;
            border: none;
            color: var(--ai-sidebar-text-muted, #888);
            cursor: pointer;
            font-size: 18px;
        }}
        .ai-sidebar-messages {{
            flex: 1;
            overflow-y: auto;
            padding: 16px;
        }}
        .ai-sidebar-input-container {{
            padding: 12px 16px;
            border-top: 1px solid var(--ai-sidebar-border, #333);
        }}
        .ai-sidebar-input {{
            width: 100%;
            padding: 10px 12px;
            border: 1px solid var(--ai-sidebar-border, #333);
            border-radius: 8px;
            background: var(--ai-sidebar-input-bg, #2d2d2d);
            color: var(--ai-sidebar-text, #fff);
            resize: none;
        }}
        .ai-message {{
            margin-bottom: 12px;
            padding: 10px 12px;
            border-radius: 8px;
        }}
        .ai-message.user {{
            background: var(--ai-sidebar-user-bg, #0066cc);
            margin-left: 20%;
        }}
        .ai-message.assistant {{
            background: var(--ai-sidebar-assistant-bg, #333);
            margin-right: 20%;
        }}
        .ai-thinking {{
            color: var(--ai-sidebar-thinking, #888);
            font-style: italic;
            font-size: 0.9em;
        }}
        """

        # HTML for sidebar
        html = f"""
        <div id="ai-sidebar" class="ai-sidebar">
            <div class="ai-sidebar-header">
                <span class="ai-sidebar-title">{cfg.header_title}</span>
                <button class="ai-sidebar-close" onclick="window.auroraview.ai.toggleSidebar()">&times;</button>
            </div>
            <div id="ai-sidebar-messages" class="ai-sidebar-messages"></div>
            <div class="ai-sidebar-input-container">
                <textarea
                    id="ai-sidebar-input"
                    class="ai-sidebar-input"
                    placeholder="{cfg.placeholder_text}"
                    rows="2"
                ></textarea>
            </div>
        </div>
        """

        # JavaScript for sidebar functionality
        js = f"""
        (function() {{
            // Add CSS
            const style = document.createElement('style');
            style.textContent = `{css}`;
            document.head.appendChild(style);

            // Add HTML
            const container = document.createElement('div');
            container.innerHTML = `{html}`;
            document.body.appendChild(container.firstElementChild);

            // Sidebar state
            let isOpen = {"true" if not cfg.collapsed else "false"};
            const sidebar = document.getElementById('ai-sidebar');
            const input = document.getElementById('ai-sidebar-input');
            const messages = document.getElementById('ai-sidebar-messages');

            // Initialize sidebar state
            if (isOpen) sidebar.classList.add('open');

            // Toggle function
            window.auroraview = window.auroraview || {{}};
            window.auroraview.ai = window.auroraview.ai || {{}};

            window.auroraview.ai.toggleSidebar = function() {{
                isOpen = !isOpen;
                sidebar.classList.toggle('open', isOpen);
            }};

            // Send message
            window.auroraview.ai.sendMessage = async function() {{
                const text = input.value.trim();
                if (!text) return;

                // Add user message
                addMessage('user', text);
                input.value = '';

                // Send to backend
                try {{
                    const response = await window.auroraview.call('ai.chat', {{ message: text }});
                    if (response.status === 'ok') {{
                        addMessage('assistant', response.response);
                    }} else {{
                        addMessage('assistant', 'Error: ' + (response.message || 'Unknown error'));
                    }}
                }} catch (e) {{
                    addMessage('assistant', 'Error: ' + e.message);
                }}
            }};

            function addMessage(role, content) {{
                const div = document.createElement('div');
                div.className = 'ai-message ' + role;
                div.textContent = content;
                messages.appendChild(div);
                messages.scrollTop = messages.scrollHeight;
            }}

            // Handle Enter key
            input.addEventListener('keydown', function(e) {{
                if (e.key === 'Enter' && !e.shiftKey) {{
                    e.preventDefault();
                    window.auroraview.ai.sendMessage();
                }}
            }});

            // Keyboard shortcut
            document.addEventListener('keydown', function(e) {{
                const shortcut = '{cfg.keyboard_shortcut}';
                const parts = shortcut.toLowerCase().split('+');
                const key = parts.pop();
                const ctrl = parts.includes('ctrl');
                const shift = parts.includes('shift');
                const alt = parts.includes('alt');

                if (e.key.toLowerCase() === key &&
                    e.ctrlKey === ctrl &&
                    e.shiftKey === shift &&
                    e.altKey === alt) {{
                    e.preventDefault();
                    window.auroraview.ai.toggleSidebar();
                }}
            }});

            // Listen for AG-UI events
            if (window.auroraview && window.auroraview.on) {{
                window.auroraview.on('agui:text_delta', function(data) {{
                    // Handle streaming text
                    const lastMsg = messages.lastElementChild;
                    if (lastMsg && lastMsg.classList.contains('assistant')) {{
                        lastMsg.textContent += data.delta;
                    }} else {{
                        addMessage('assistant', data.delta);
                    }}
                }});

                window.auroraview.on('agui:thinking_delta', function(data) {{
                    // Handle thinking/reasoning
                    const thinkingDiv = document.querySelector('.ai-thinking');
                    if (thinkingDiv) {{
                        thinkingDiv.textContent += data.delta;
                    }} else {{
                        const div = document.createElement('div');
                        div.className = 'ai-thinking';
                        div.textContent = data.delta;
                        messages.appendChild(div);
                    }}
                }});
            }}

            console.log('[AuroraView AI] Sidebar initialized');
        }})();
        """

        # Inject into WebView
        self.webview.eval_js(js)

    def discover_tools(self) -> int:
        """Discover tools from WebView bound APIs.

        This scans all methods bound via webview.bind_call() and creates
        Tool definitions with inferred JSON schemas from type hints.

        Returns:
            Number of tools discovered
        """
        if not self.webview:
            logger.warning("No WebView attached, cannot discover tools")
            return 0

        return self.tools.discover_from_webview(self.webview)

    def register_tool(self, tool: Tool) -> None:
        """Register a custom tool.

        Args:
            tool: Tool definition to register
        """
        self.tools.register(tool)

    def on_event(self, handler: Callable[[AGUIEvent], None]) -> None:
        """Register an event handler for AG-UI events.

        Args:
            handler: Callback function that receives AGUIEvent
        """
        self._event_handlers.append(handler)

    def emit_event(self, event: AGUIEvent) -> None:
        """Emit an AG-UI event to all handlers and WebView.

        Args:
            event: Event to emit
        """
        # Call registered handlers
        for handler in self._event_handlers:
            try:
                handler(event)
            except Exception as e:
                logger.exception("Error in event handler: %s", e)

        # Emit to WebView if attached
        if self.webview:
            event_name = f"agui:{event.type.value.lower()}"
            self.webview.emit(event_name, event.to_dict())

    def get_session(self, session_id: Optional[str] = None) -> Session:
        """Get or create a chat session.

        Args:
            session_id: Optional session ID, uses active session if None

        Returns:
            Session instance
        """
        if session_id is None:
            if self._active_session_id is None:
                session_id = str(uuid.uuid4())
                self._sessions[session_id] = Session(
                    id=session_id,
                    system_prompt=self.config.system_prompt,
                )
                self._active_session_id = session_id
            session_id = self._active_session_id

        if session_id not in self._sessions:
            self._sessions[session_id] = Session(
                id=session_id,
                system_prompt=self.config.system_prompt,
            )

        return self._sessions[session_id]

    def clear_session(self, session_id: Optional[str] = None) -> None:
        """Clear a chat session.

        Args:
            session_id: Session to clear, uses active session if None
        """
        session = self.get_session(session_id)
        session.clear()

    async def chat(
        self,
        message: str,
        *,
        session_id: Optional[str] = None,
        stream: Optional[bool] = None,
    ) -> str:
        """Send a chat message and get a response.

        Args:
            message: User message
            session_id: Optional session ID
            stream: Override streaming setting

        Returns:
            Assistant response text

        Note:
            This requires an async context. For synchronous usage, use chat_sync().
        """
        session = self.get_session(session_id)
        run_id = str(uuid.uuid4())
        message_id = str(uuid.uuid4())

        # Add user message
        session.add_user_message(message)

        # Emit run started
        self.emit_event(AGUIEvent.run_started(run_id, session.id))

        try:
            # Get AI response
            response = await self._get_completion(
                session,
                stream=stream if stream is not None else self.config.stream,
                message_id=message_id,
            )

            # Add assistant message
            session.add_assistant_message(response, message_id)

            # Emit run finished
            self.emit_event(AGUIEvent.run_finished(run_id, session.id))

            return response

        except Exception as e:
            self.emit_event(AGUIEvent.run_error(run_id, str(e)))
            raise

    def chat_sync(
        self,
        message: str,
        *,
        session_id: Optional[str] = None,
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

        return loop.run_until_complete(self.chat(message, session_id=session_id, stream=False))

    async def _get_completion(
        self,
        session: Session,
        *,
        stream: bool = True,
        message_id: str = "",
    ) -> str:
        """Get completion from AI provider.

        This is an internal method that handles the actual API call.
        Override this for custom provider implementations.
        """
        provider = self.config.infer_provider()

        # Build messages and sanitize to remove invalid Unicode surrogates
        # This is important on Windows where paths can contain surrogate pairs
        messages = sanitize_list(session.get_messages_for_api())

        # Get tools in appropriate format
        tools = None
        if self.tools.all():
            if provider in (ProviderType.OPENAI, ProviderType.DEEPSEEK):
                tools = sanitize_list(self.tools.to_openai_tools())
            elif provider == ProviderType.ANTHROPIC:
                tools = sanitize_list(self.tools.to_anthropic_tools())

        # Call provider-specific implementation
        if provider == ProviderType.OPENAI:
            return await self._openai_completion(messages, tools, stream, message_id)
        elif provider == ProviderType.ANTHROPIC:
            return await self._anthropic_completion(messages, tools, stream, message_id)
        elif provider == ProviderType.GEMINI:
            return await self._gemini_completion(messages, tools, stream, message_id)
        else:
            # Generic completion (works with most OpenAI-compatible APIs)
            return await self._generic_completion(messages, tools, stream, message_id)

    async def _openai_completion(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """OpenAI completion implementation."""
        import os

        try:
            import openai
        except ImportError as err:
            raise ImportError("openai package is required for OpenAI provider") from err

        # Get API key from config or environment
        api_key = self.config.api_key or os.environ.get("OPENAI_API_KEY")
        if not api_key:
            raise ValueError("OpenAI API key required. Set OPENAI_API_KEY environment variable.")

        client = openai.AsyncOpenAI(
            api_key=api_key,
            base_url=self.config.base_url,
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
            self.emit_event(AGUIEvent.text_start(message_id))

            full_response = ""
            async for chunk in await client.chat.completions.create(stream=True, **kwargs):
                if chunk.choices and chunk.choices[0].delta.content:
                    delta = chunk.choices[0].delta.content
                    full_response += delta
                    self.emit_event(AGUIEvent.text_delta(message_id, delta))

            self.emit_event(AGUIEvent.text_end(message_id))
            return full_response
        else:
            response = await client.chat.completions.create(**kwargs)
            return response.choices[0].message.content or ""

    async def _anthropic_completion(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """Anthropic completion implementation."""
        import os

        try:
            import anthropic
        except ImportError as err:
            raise ImportError("anthropic package is required for Anthropic provider") from err

        # Get API key from config or environment
        api_key = self.config.api_key or os.environ.get("ANTHROPIC_API_KEY")
        if not api_key:
            raise ValueError(
                "Anthropic API key required. Set ANTHROPIC_API_KEY environment variable."
            )

        client = anthropic.AsyncAnthropic(
            api_key=api_key,
            timeout=self.config.timeout,
        )

        # Extract system message
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
            self.emit_event(AGUIEvent.text_start(message_id))

            full_response = ""
            async with client.messages.stream(**kwargs) as stream_ctx:
                async for text in stream_ctx.text_stream:
                    full_response += text
                    self.emit_event(AGUIEvent.text_delta(message_id, text))

            self.emit_event(AGUIEvent.text_end(message_id))
            return full_response
        else:
            response = await client.messages.create(**kwargs)
            return response.content[0].text if response.content else ""

    async def _gemini_completion(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """Google Gemini completion implementation.

        Uses the new google-genai SDK (REST-based, lightweight).
        Falls back to legacy google-generativeai if available.
        """
        # Try new lightweight SDK first
        try:
            from google import genai

            return await self._gemini_new_sdk(genai, messages, tools, stream, message_id)
        except ImportError:
            pass

        # Fall back to legacy SDK
        try:
            import google.generativeai as genai_legacy

            return await self._gemini_legacy_sdk(genai_legacy, messages, tools, stream, message_id)
        except ImportError as err:
            raise ImportError(
                "google-genai package is required for Gemini provider. "
                "Install with: pip install google-genai"
            ) from err

    async def _gemini_new_sdk(
        self,
        genai,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """Gemini implementation using new google-genai SDK."""
        import os

        api_key = (
            self.config.api_key
            or os.environ.get("GEMINI_API_KEY")
            or os.environ.get("GOOGLE_API_KEY")
        )
        client = genai.Client(api_key=api_key)

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
            self.emit_event(AGUIEvent.text_start(message_id))

            full_response = ""
            response = client.models.generate_content_stream(
                model=self.config.model,
                contents=contents,
                config=config,
            )
            for chunk in response:
                if chunk.text:
                    full_response += chunk.text
                    self.emit_event(AGUIEvent.text_delta(message_id, chunk.text))

            self.emit_event(AGUIEvent.text_end(message_id))
            return full_response
        else:
            response = client.models.generate_content(
                model=self.config.model,
                contents=contents,
                config=config,
            )
            return response.text or ""

    async def _gemini_legacy_sdk(
        self,
        genai,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """Gemini implementation using legacy google-generativeai SDK."""
        if self.config.api_key:
            genai.configure(api_key=self.config.api_key)

        model = genai.GenerativeModel(self.config.model)

        # Convert messages to Gemini format
        history = []
        last_content = ""
        for msg in messages:
            if msg["role"] == "system":
                # Gemini doesn't have system messages, prepend to first user message
                continue
            elif msg["role"] == "user":
                history.append({"role": "user", "parts": [msg["content"]]})
            elif msg["role"] == "assistant":
                history.append({"role": "model", "parts": [msg["content"]]})
            last_content = msg["content"]

        chat = model.start_chat(history=history[:-1] if history else [])

        if stream:
            self.emit_event(AGUIEvent.text_start(message_id))

            full_response = ""
            response = await chat.send_message_async(
                last_content,
                stream=True,
                generation_config=genai.types.GenerationConfig(
                    temperature=self.config.temperature,
                    max_output_tokens=self.config.max_tokens,
                ),
            )
            async for chunk in response:
                if chunk.text:
                    full_response += chunk.text
                    self.emit_event(AGUIEvent.text_delta(message_id, chunk.text))

            self.emit_event(AGUIEvent.text_end(message_id))
            return full_response
        else:
            response = await chat.send_message_async(
                last_content,
                generation_config=genai.types.GenerationConfig(
                    temperature=self.config.temperature,
                    max_output_tokens=self.config.max_tokens,
                ),
            )
            return response.text or ""

    async def _generic_completion(
        self,
        messages: List[Dict[str, str]],
        tools: Optional[List[Dict[str, Any]]],
        stream: bool,
        message_id: str,
    ) -> str:
        """Generic OpenAI-compatible completion implementation.

        Works with DeepSeek, Groq, local models via Ollama, etc.
        """
        import os

        try:
            import openai
        except ImportError as err:
            raise ImportError("openai package is required") from err

        # Determine base URL for different providers
        provider = self.config.infer_provider()
        base_url = self.config.base_url
        api_key = self.config.api_key

        # Get provider-specific API key if not configured
        if api_key is None:
            if provider == ProviderType.DEEPSEEK:
                api_key = os.environ.get("DEEPSEEK_API_KEY")
            elif provider == ProviderType.GROQ:
                api_key = os.environ.get("GROQ_API_KEY")
            elif provider == ProviderType.XAI:
                api_key = os.environ.get("XAI_API_KEY")
            elif provider == ProviderType.OLLAMA:
                api_key = "ollama"  # Ollama doesn't need a real key

        if base_url is None:
            if provider == ProviderType.DEEPSEEK:
                base_url = "https://api.deepseek.com"
            elif provider == ProviderType.OLLAMA:
                base_url = "http://localhost:11434/v1"
            elif provider == ProviderType.GROQ:
                base_url = "https://api.groq.com/openai/v1"

        # Validate API key for non-Ollama providers
        if not api_key and provider != ProviderType.OLLAMA:
            raise ValueError(
                f"API key required for {provider.value}. "
                f"Set {provider.value.upper()}_API_KEY environment variable."
            )

        client = openai.AsyncOpenAI(
            api_key=api_key or "ollama",
            base_url=base_url,
            timeout=self.config.timeout,
        )

        kwargs: Dict[str, Any] = {
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        }
        if tools and provider not in (ProviderType.OLLAMA,):  # Some don't support tools
            kwargs["tools"] = tools

        if stream:
            self.emit_event(AGUIEvent.text_start(message_id))

            full_response = ""
            async for chunk in await client.chat.completions.create(stream=True, **kwargs):
                if chunk.choices and chunk.choices[0].delta.content:
                    delta = chunk.choices[0].delta.content
                    full_response += delta
                    self.emit_event(AGUIEvent.text_delta(message_id, delta))

            self.emit_event(AGUIEvent.text_end(message_id))
            return full_response
        else:
            response = await client.chat.completions.create(**kwargs)
            return response.choices[0].message.content or ""

    async def execute_tool(self, name: str, arguments: Dict[str, Any]) -> Any:
        """Execute a tool by name.

        Args:
            name: Tool name
            arguments: Tool arguments

        Returns:
            Tool execution result
        """
        tool_call_id = str(uuid.uuid4())

        self.emit_event(AGUIEvent.tool_call_start("", tool_call_id, name))
        self.emit_event(AGUIEvent.tool_call_args(tool_call_id, json.dumps(arguments)))

        try:
            result = await self.tools.execute(name, arguments)
            self.emit_event(AGUIEvent.tool_call_end(tool_call_id))
            self.emit_event(AGUIEvent.tool_call_result(tool_call_id, json.dumps(result)))
            return result
        except Exception as e:
            self.emit_event(AGUIEvent.tool_call_end(tool_call_id))
            self.emit_event(AGUIEvent.tool_call_result(tool_call_id, json.dumps({"error": str(e)})))
            raise
