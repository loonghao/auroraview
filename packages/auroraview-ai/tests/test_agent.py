"""Tests for auroraview_ai.agent module.

Tests use mocking to avoid real LLM API calls.
"""

from __future__ import annotations

import asyncio
from typing import Any
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_ai.agent import AgentDeps, AuroraAgent
from auroraview_ai.config import AgentConfig, ProviderType
from auroraview_ai.tools import DCCTool, DCCToolCategory


def make_mock_agent() -> MagicMock:
    """Create a mock Pydantic AI agent."""
    mock = MagicMock()
    mock.run = AsyncMock()
    mock.run_stream = MagicMock()
    mock.tool = MagicMock(return_value=lambda f: f)
    return mock


class TestAgentDeps:
    """Tests for AgentDeps dataclass."""

    def test_minimal_construction(self) -> None:
        deps = AgentDeps(session_id="test-session")
        assert deps.session_id == "test-session"
        assert deps.webview is None
        assert deps.emit_callback is None

    def test_with_all_fields(self) -> None:
        webview = MagicMock()
        cb = MagicMock()
        deps = AgentDeps(session_id="s1", webview=webview, emit_callback=cb)
        assert deps.webview is webview
        assert deps.emit_callback is cb


class TestAuroraAgentInit:
    """Tests for AuroraAgent initialization."""

    def test_default_init(self) -> None:
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent()
            assert agent.config is not None
            assert agent.webview is None
            assert agent.sidebar_config is None

    def test_init_with_config(self) -> None:
        config = AgentConfig(model="gpt-4o-mini")
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent(config=config)
            assert agent.config.model == "gpt-4o-mini"

    def test_init_with_emit_callback(self) -> None:
        cb = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent(emit_callback=cb)
            assert agent._emitter._emit_callback is cb

    def test_model_property(self) -> None:
        config = AgentConfig(model="gpt-4o")
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent(config=config)
            assert agent.model == "gpt-4o"

    def test_model_setter_recreates_agent(self) -> None:
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent()
            agent.model = "gpt-4o-mini"
            assert agent.config.model == "gpt-4o-mini"
            # Agent should have been recreated
            assert mock_agent_cls.call_count >= 2

    def test_deepseek_raises_without_api_key(self) -> None:
        config = AgentConfig(model="deepseek-chat")
        with patch.dict("os.environ", {}, clear=True):
            # Remove DEEPSEEK_API_KEY if present
            import os
            os.environ.pop("DEEPSEEK_API_KEY", None)
            with patch("auroraview_ai.agent.Agent"):
                with pytest.raises(ValueError, match="DeepSeek API key"):
                    AuroraAgent(config=config)

    def test_deepseek_with_api_key_in_config(self) -> None:
        config = AgentConfig(model="deepseek-chat", api_key="sk-test-key")
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            with patch("auroraview_ai.agent.OpenAIProvider") as mock_provider:
                mock_agent_cls.return_value = make_mock_agent()
                mock_provider.return_value = MagicMock()
                agent = AuroraAgent(config=config)
                assert agent is not None
                mock_provider.assert_called_once()

    def test_deepseek_with_env_api_key(self) -> None:
        config = AgentConfig(model="deepseek-chat")
        with patch.dict("os.environ", {"DEEPSEEK_API_KEY": "sk-env-key"}):
            with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
                with patch("auroraview_ai.agent.OpenAIProvider") as mock_provider:
                    mock_agent_cls.return_value = make_mock_agent()
                    mock_provider.return_value = MagicMock()
                    agent = AuroraAgent(config=config)
                    assert agent is not None

    def test_auto_discover_without_webview(self) -> None:
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent(auto_discover_apis=True, webview=None)
            # Should not crash, just skip discovery
            assert agent.get_tools() == []

    def test_auto_discover_with_webview(self) -> None:
        webview = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            agent = AuroraAgent(auto_discover_apis=True, webview=webview)
            # discover_tools returns 0 (TODO: not implemented)
            assert isinstance(agent.get_tools(), list)


class TestAuroraAgentSessions:
    """Tests for session management."""

    def _make_agent(self) -> AuroraAgent:
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            return AuroraAgent()

    def test_get_session_creates_new(self) -> None:
        agent = self._make_agent()
        sid = agent.get_session()
        assert sid is not None
        assert len(sid) > 0

    def test_get_session_returns_same_id(self) -> None:
        agent = self._make_agent()
        sid1 = agent.get_session()
        sid2 = agent.get_session()
        assert sid1 == sid2

    def test_get_session_with_explicit_id(self) -> None:
        agent = self._make_agent()
        sid = agent.get_session("my-session")
        assert sid == "my-session"

    def test_get_session_creates_history(self) -> None:
        agent = self._make_agent()
        sid = agent.get_session("s1")
        assert sid in agent._sessions
        assert agent._sessions[sid] == []

    def test_clear_session(self) -> None:
        agent = self._make_agent()
        sid = agent.get_session()
        agent._sessions[sid].append({"role": "user", "content": "hello"})
        agent.clear_session()
        assert agent._sessions[sid] == []

    def test_clear_session_explicit_id(self) -> None:
        agent = self._make_agent()
        agent.get_session("s1")
        agent._sessions["s1"].append({"role": "user", "content": "test"})
        agent.clear_session("s1")
        assert agent._sessions["s1"] == []


class TestAuroraAgentTools:
    """Tests for tool registration."""

    def test_register_tool_decorator(self) -> None:
        mock_inner_agent = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner_agent
            agent = AuroraAgent()

            @agent.tool
            def my_tool(value: str) -> str:
                """My test tool."""
                return value

            tools = agent.get_tools()
            assert len(tools) == 1
            assert tools[0].name == "my_tool"

    def test_register_tool_with_custom_name(self) -> None:
        mock_inner_agent = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner_agent
            agent = AuroraAgent()

            @agent.tool(name="custom_tool")
            def my_func() -> None:
                """Custom tool."""
                pass

            tools = agent.get_tools()
            assert len(tools) == 1
            assert tools[0].name == "custom_tool"

    def test_register_multiple_tools(self) -> None:
        mock_inner_agent = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner_agent
            agent = AuroraAgent()

            @agent.tool
            def tool_a() -> None:
                pass

            @agent.tool
            def tool_b() -> None:
                pass

            assert len(agent.get_tools()) == 2

    def test_get_tools_returns_copy(self) -> None:
        mock_inner_agent = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner_agent
            agent = AuroraAgent()
            tools1 = agent.get_tools()
            tools1.append(DCCTool(name="fake", description="fake"))
            tools2 = agent.get_tools()
            assert len(tools2) == 0  # Original not modified

    def test_tool_description_from_docstring(self) -> None:
        mock_inner_agent = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner_agent
            agent = AuroraAgent()

            @agent.tool
            def my_func() -> None:
                """This is the description."""
                pass

            tools = agent.get_tools()
            assert tools[0].description == "This is the description."

    def test_tool_custom_description(self) -> None:
        mock_inner_agent = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner_agent
            agent = AuroraAgent()

            @agent.tool(description="Override description")
            def my_func() -> None:
                pass

            tools = agent.get_tools()
            assert tools[0].description == "Override description"


class TestAuroraAgentChat:
    """Tests for chat methods using mocked agent."""

    def _make_agent_with_mock(self) -> tuple[AuroraAgent, MagicMock]:
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent()
        return agent, mock_inner

    @pytest.mark.asyncio
    async def test_chat_basic(self) -> None:
        agent, mock_inner = self._make_agent_with_mock()

        mock_result = MagicMock()
        mock_result.output = "Hello, I am the AI!"
        mock_inner.run = AsyncMock(return_value=mock_result)

        response = await agent.chat("Hello")
        assert response == "Hello, I am the AI!"

    @pytest.mark.asyncio
    async def test_chat_stores_session_history(self) -> None:
        agent, mock_inner = self._make_agent_with_mock()

        mock_result = MagicMock()
        mock_result.output = "Response"
        mock_inner.run = AsyncMock(return_value=mock_result)

        await agent.chat("User message")
        sid = agent._active_session_id
        history = agent._sessions[sid]
        assert len(history) == 2
        assert history[0]["role"] == "user"
        assert history[0]["content"] == "User message"
        assert history[1]["role"] == "assistant"

    @pytest.mark.asyncio
    async def test_chat_emits_agui_events(self) -> None:
        cb = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_inner = make_mock_agent()
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb)

        mock_result = MagicMock()
        mock_result.output = "Response"
        mock_inner.run = AsyncMock(return_value=mock_result)

        await agent.chat("Hello")
        # Should have called run_started and run_finished
        assert cb.call_count >= 2

    @pytest.mark.asyncio
    async def test_chat_error_emits_run_error(self) -> None:
        cb = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_inner = make_mock_agent()
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb)

        mock_inner.run = AsyncMock(side_effect=ValueError("API Error"))

        with pytest.raises(ValueError, match="API Error"):
            await agent.chat("Hello")

        # Should have emitted run_error
        call_names = [call[0][0] for call in cb.call_args_list]
        assert "agui:run_error" in call_names

    def test_chat_sync(self) -> None:
        agent, mock_inner = self._make_agent_with_mock()

        mock_result = MagicMock()
        mock_result.output = "Sync response"
        mock_inner.run = AsyncMock(return_value=mock_result)

        response = agent.chat_sync("Hello")
        assert response == "Sync response"

    def test_set_emit_callback(self) -> None:
        agent, _ = self._make_agent_with_mock()
        cb = MagicMock()
        agent.set_emit_callback(cb)
        assert agent._emitter._emit_callback is cb

    def test_discover_tools_no_webview(self) -> None:
        agent, _ = self._make_agent_with_mock()
        result = agent.discover_tools()
        assert result == 0

    def test_discover_tools_with_webview(self) -> None:
        agent, _ = self._make_agent_with_mock()
        agent.webview = MagicMock()
        result = agent.discover_tools()
        assert result == 0  # TODO not implemented yet


class TestExecuteTool:
    """Tests for _execute_tool."""

    def _make_agent(self) -> AuroraAgent:
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = make_mock_agent()
            return AuroraAgent()

    @pytest.mark.asyncio
    async def test_execute_sync_func(self) -> None:
        agent = self._make_agent()

        def sync_func(value: str) -> str:
            return f"result:{value}"

        result = await agent._execute_tool(sync_func, {"value": "test"})
        assert result == "result:test"

    @pytest.mark.asyncio
    async def test_execute_async_func(self) -> None:
        agent = self._make_agent()

        async def async_func(value: str) -> str:
            return f"async_result:{value}"

        result = await agent._execute_tool(async_func, {"value": "test"})
        assert result == "async_result:test"

    @pytest.mark.asyncio
    async def test_default_instructions(self) -> None:
        agent = self._make_agent()
        instructions = agent._default_instructions()
        assert "DCC" in instructions
        assert isinstance(instructions, str)


class TestChatStream:
    """Tests for chat_stream method."""

    def _make_agent_with_mock(self) -> tuple[AuroraAgent, MagicMock]:
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent()
        return agent, mock_inner

    @pytest.mark.asyncio
    async def test_chat_stream_yields_text(self) -> None:
        agent, mock_inner = self._make_agent_with_mock()

        # Create async context manager that yields text deltas
        mock_response = MagicMock()

        async def async_text_gen():
            yield "Hello"
            yield " World"

        mock_response.stream_text = MagicMock(return_value=async_text_gen())

        class AsyncCtxMgr:
            async def __aenter__(self):
                return mock_response

            async def __aexit__(self, *args):
                pass

        mock_inner.run_stream = MagicMock(return_value=AsyncCtxMgr())

        results = []
        async for text in agent.chat_stream("Hello"):
            results.append(text)

        assert results == ["Hello", " World"]

    @pytest.mark.asyncio
    async def test_chat_stream_stores_session(self) -> None:
        agent, mock_inner = self._make_agent_with_mock()

        mock_response = MagicMock()

        async def async_text_gen():
            yield "Response"

        mock_response.stream_text = MagicMock(return_value=async_text_gen())

        class AsyncCtxMgr:
            async def __aenter__(self):
                return mock_response

            async def __aexit__(self, *args):
                pass

        mock_inner.run_stream = MagicMock(return_value=AsyncCtxMgr())

        results = []
        async for text in agent.chat_stream("My message"):
            results.append(text)

        sid = agent._active_session_id
        history = agent._sessions[sid]
        assert len(history) == 2
        assert history[0]["content"] == "My message"
        assert history[1]["content"] == "Response"

    @pytest.mark.asyncio
    async def test_chat_stream_emits_events(self) -> None:
        cb = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_inner = make_mock_agent()
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb)

        mock_response = MagicMock()

        async def async_text_gen():
            yield "text"

        mock_response.stream_text = MagicMock(return_value=async_text_gen())

        class AsyncCtxMgr:
            async def __aenter__(self):
                return mock_response

            async def __aexit__(self, *args):
                pass

        mock_inner.run_stream = MagicMock(return_value=AsyncCtxMgr())

        results = []
        async for text in agent.chat_stream("Hello"):
            results.append(text)

        # Should have: run_started, text_start, text_delta, text_end, run_finished
        assert cb.call_count >= 5

    @pytest.mark.asyncio
    async def test_chat_stream_error_emits_run_error(self) -> None:
        cb = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_inner = make_mock_agent()
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb)

        class FailingCtxMgr:
            async def __aenter__(self):
                raise RuntimeError("Stream failed")

            async def __aexit__(self, *args):
                pass

        mock_inner.run_stream = MagicMock(return_value=FailingCtxMgr())

        with pytest.raises(RuntimeError, match="Stream failed"):
            async for _ in agent.chat_stream("Hello"):
                pass

        call_names = [call[0][0] for call in cb.call_args_list]
        assert "agui:run_error" in call_names


class TestChatSyncNewEventLoop:
    """Tests for chat_sync RuntimeError path (no event loop)."""

    def test_chat_sync_creates_new_event_loop_when_none(self) -> None:
        """Simulate RuntimeError from asyncio.get_event_loop() → creates new loop."""
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_agent_cls:
            mock_agent_cls.return_value = mock_inner
            agent = AuroraAgent()

        mock_result = MagicMock()
        mock_result.output = "New loop response"
        mock_inner.run = AsyncMock(return_value=mock_result)

        with patch("auroraview_ai.agent.asyncio.get_event_loop", side_effect=RuntimeError("no loop")):
            new_loop = asyncio.new_event_loop()
            with patch("auroraview_ai.agent.asyncio.new_event_loop", return_value=new_loop):
                with patch("auroraview_ai.agent.asyncio.set_event_loop") as mock_set:
                    result = agent.chat_sync("test")
                    assert result == "New loop response"
                    mock_set.assert_called_once_with(new_loop)


class TestAgentDepsEdgeCases:
    """Additional edge-case tests for AgentDeps."""

    def test_session_id_empty_string(self) -> None:
        deps = AgentDeps(session_id="")
        assert deps.session_id == ""

    def test_session_id_unicode(self) -> None:
        deps = AgentDeps(session_id="セッション-001")
        assert deps.session_id == "セッション-001"

    def test_emit_callback_is_callable(self) -> None:
        calls: list[Any] = []
        cb = lambda event, data=None: calls.append((event, data))  # noqa: E731
        deps = AgentDeps(session_id="s", emit_callback=cb)
        assert deps.emit_callback is cb

    def test_webview_set_to_non_none(self) -> None:
        wv = object()
        deps = AgentDeps(session_id="s", webview=wv)
        assert deps.webview is wv

    def test_dataclass_repr_contains_session_id(self) -> None:
        deps = AgentDeps(session_id="my-session")
        assert "my-session" in repr(deps)


class TestAuroraAgentSessionManagement:
    """Tests for session handling inside AuroraAgent."""

    def _make_agent(self) -> tuple[AuroraAgent, MagicMock]:
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = mock_inner
            agent = AuroraAgent()
        return agent, mock_inner

    @pytest.mark.asyncio
    async def test_multiple_chats_create_one_session_per_call(self) -> None:
        agent, mock_inner = self._make_agent()
        mock_result = MagicMock()
        mock_result.output = "R"
        mock_inner.run = AsyncMock(return_value=mock_result)

        await agent.chat("msg1")
        sid1 = agent._active_session_id
        await agent.chat("msg2")
        sid2 = agent._active_session_id

        # Each call may reuse or create a session; both must be in _sessions
        assert sid1 in agent._sessions
        assert sid2 in agent._sessions

    @pytest.mark.asyncio
    async def test_session_history_has_user_and_assistant_messages(self) -> None:
        agent, mock_inner = self._make_agent()
        mock_result = MagicMock()
        mock_result.output = "assistant answer"
        mock_inner.run = AsyncMock(return_value=mock_result)

        await agent.chat("user question")
        sid = agent._active_session_id
        history = agent._sessions[sid]

        roles = [m["role"] for m in history]
        assert "user" in roles
        assert "assistant" in roles

    @pytest.mark.asyncio
    async def test_chat_returns_output_string(self) -> None:
        agent, mock_inner = self._make_agent()
        mock_result = MagicMock()
        mock_result.output = "my output"
        mock_inner.run = AsyncMock(return_value=mock_result)

        result = await agent.chat("question")
        assert result == "my output"

    @pytest.mark.asyncio
    async def test_chat_no_emit_callback_does_not_raise(self) -> None:
        agent, mock_inner = self._make_agent()
        assert agent._emitter._emit_callback is None
        mock_result = MagicMock()
        mock_result.output = "ok"
        mock_inner.run = AsyncMock(return_value=mock_result)
        result = await agent.chat("hello")
        assert result == "ok"

    def test_chat_sync_empty_input(self) -> None:
        agent, mock_inner = self._make_agent()
        mock_result = MagicMock()
        mock_result.output = ""
        mock_inner.run = AsyncMock(return_value=mock_result)
        result = agent.chat_sync("")
        assert result == ""


class TestAuroraAgentEmitterCallback:
    """Tests for emit callback behavior."""

    def _make_agent_with_cb(self) -> tuple[AuroraAgent, MagicMock, MagicMock]:
        cb = MagicMock()
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb)
        return agent, mock_inner, cb

    @pytest.mark.asyncio
    async def test_run_started_event_is_emitted(self) -> None:
        agent, mock_inner, cb = self._make_agent_with_cb()
        mock_result = MagicMock()
        mock_result.output = "R"
        mock_inner.run = AsyncMock(return_value=mock_result)

        await agent.chat("hello")
        call_names = [call[0][0] for call in cb.call_args_list]
        assert "agui:run_started" in call_names

    @pytest.mark.asyncio
    async def test_run_finished_event_is_emitted(self) -> None:
        agent, mock_inner, cb = self._make_agent_with_cb()
        mock_result = MagicMock()
        mock_result.output = "R"
        mock_inner.run = AsyncMock(return_value=mock_result)

        await agent.chat("hello")
        call_names = [call[0][0] for call in cb.call_args_list]
        assert "agui:run_finished" in call_names

    def test_set_emit_callback_replaces_callback(self) -> None:
        cb1 = MagicMock()
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb1)

        cb2 = MagicMock()
        agent.set_emit_callback(cb2)
        assert agent._emitter._emit_callback is cb2

    def test_set_emit_callback_to_none(self) -> None:
        cb = MagicMock()
        mock_inner = make_mock_agent()
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = mock_inner
            agent = AuroraAgent(emit_callback=cb)

        agent.set_emit_callback(None)
        assert agent._emitter._emit_callback is None


class TestExecuteToolEdgeCases:
    """Additional edge cases for _execute_tool."""

    def _make_agent(self) -> AuroraAgent:
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = make_mock_agent()
            return AuroraAgent()

    @pytest.mark.asyncio
    async def test_execute_tool_with_no_args(self) -> None:
        agent = self._make_agent()

        def no_arg_func() -> str:
            return "no_args"

        result = await agent._execute_tool(no_arg_func, {})
        assert result == "no_args"

    @pytest.mark.asyncio
    async def test_execute_tool_with_multiple_kwargs(self) -> None:
        agent = self._make_agent()

        def multi_func(a: int, b: int) -> int:
            return a + b

        result = await agent._execute_tool(multi_func, {"a": 3, "b": 5})
        assert result == 8

    @pytest.mark.asyncio
    async def test_execute_async_tool_returns_list(self) -> None:
        agent = self._make_agent()

        async def list_func(n: int) -> list[int]:
            return list(range(n))

        result = await agent._execute_tool(list_func, {"n": 4})
        assert result == [0, 1, 2, 3]

    @pytest.mark.asyncio
    async def test_execute_tool_propagates_exception(self) -> None:
        agent = self._make_agent()

        def raising_func() -> str:
            raise ValueError("tool error")

        with pytest.raises(ValueError, match="tool error"):
            await agent._execute_tool(raising_func, {})


class TestAuroraAgentWebview:
    """Tests for webview attribute on AuroraAgent."""

    def test_webview_default_none(self) -> None:
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = make_mock_agent()
            agent = AuroraAgent()
        assert agent.webview is None

    def test_webview_set_via_init(self) -> None:
        wv = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = make_mock_agent()
            agent = AuroraAgent(webview=wv)
        assert agent.webview is wv

    def test_webview_can_be_replaced(self) -> None:
        wv1 = MagicMock()
        wv2 = MagicMock()
        with patch("auroraview_ai.agent.Agent") as mock_cls:
            mock_cls.return_value = make_mock_agent()
            agent = AuroraAgent(webview=wv1)
        agent.webview = wv2
        assert agent.webview is wv2
