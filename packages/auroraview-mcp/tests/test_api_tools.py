"""Tests for API tools module (call_api, list_api_methods, emit_event)."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest


def make_manager(page=True, connected=True, page_conn=None):
    """Helper to build a mock ConnectionManager."""
    manager = MagicMock()
    manager.is_connected = connected
    manager.current_page = MagicMock() if page else None
    _page_conn = page_conn or MagicMock()
    manager.get_page_connection = AsyncMock(return_value=_page_conn)
    return manager, _page_conn


class TestCallApi:
    """Tests for call_api tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(method="api.echo")

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn(method="api.echo")

    @pytest.mark.asyncio
    async def test_call_api_with_params_success(self) -> None:
        """Returns result when ok=True with params."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": {"pong": 1}})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="api.echo", params={"msg": "hello"})

        assert result == {"pong": 1}

    @pytest.mark.asyncio
    async def test_call_api_no_params_api_method(self) -> None:
        """Uses api shorthand path when no params and method is api.*."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": ["a", "b"]})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="api.list_methods")

        assert result == ["a", "b"]

    @pytest.mark.asyncio
    async def test_call_api_no_params_non_api_method(self) -> None:
        """Uses generic call path for non-api.* methods."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": "done"})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="tool.apply")

        assert result == "done"

    @pytest.mark.asyncio
    async def test_call_api_error_raises(self) -> None:
        """Raises RuntimeError when ok=False."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(
            return_value={"ok": False, "error": "Method not found: api.bad"}
        )
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            with pytest.raises(RuntimeError, match="Method not found"):
                await fn(method="api.bad")

    @pytest.mark.asyncio
    async def test_call_api_unexpected_format_raises(self) -> None:
        """Raises RuntimeError when result is not a dict."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value="not-a-dict")
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            with pytest.raises(RuntimeError, match="Unexpected response"):
                await fn(method="api.echo", params={})

    @pytest.mark.asyncio
    async def test_call_api_bridge_not_ready_raises(self) -> None:
        """Raises RuntimeError when bridge returns not-ready error."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(
            return_value={"ok": False, "error": "AuroraView bridge not ready"}
        )
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            with pytest.raises(RuntimeError, match="bridge not ready"):
                await fn(method="api.any")

    @pytest.mark.asyncio
    async def test_call_api_returns_none_result(self) -> None:
        """Returns None when result field is missing."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="api.void")

        assert result is None

    @pytest.mark.asyncio
    async def test_call_api_with_complex_params(self) -> None:
        """Handles complex nested params correctly."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": True})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(
                method="tool.render",
                params={"objects": ["mesh1", "mesh2"], "settings": {"quality": "high", "aa": 4}},
            )

        assert result is True


class TestListApiMethods:
    """Tests for list_api_methods tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_returns_method_list(self) -> None:
        """Returns list of methods as dicts."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=["echo", "get_scene", "apply_tool"])
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            result = await fn()

        assert len(result) == 3
        assert result[0] == {"name": "echo", "signature": None, "docstring": None}
        assert result[2]["name"] == "apply_tool"

    @pytest.mark.asyncio
    async def test_returns_empty_when_none(self) -> None:
        """Returns [] when evaluate returns None."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_returns_empty_when_non_list(self) -> None:
        """Returns [] when evaluate returns non-list."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"methods": ["echo"]})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            result = await fn()

        assert result == []


class TestEmitEvent:
    """Tests for emit_event tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(event="my_event")

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn(event="my_event")

    @pytest.mark.asyncio
    async def test_emit_event_success(self) -> None:
        """Returns status=emitted with event name."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            result = await fn(event="scene_changed", data={"frame": 42})

        assert result["status"] == "emitted"
        assert result["event"] == "scene_changed"

    @pytest.mark.asyncio
    async def test_emit_event_no_data(self) -> None:
        """Emitting without data works (data=None)."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            result = await fn(event="ready")

        assert result["status"] == "emitted"
        assert result["event"] == "ready"

    @pytest.mark.asyncio
    async def test_emit_event_bridge_not_ready_raises(self) -> None:
        """Raises RuntimeError when bridge is not ready."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(
            return_value={"ok": False, "error": "AuroraView bridge not ready"}
        )
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            with pytest.raises(RuntimeError, match="bridge not ready"):
                await fn(event="test_event")

    @pytest.mark.asyncio
    async def test_emit_event_with_nested_data(self) -> None:
        """Event with nested dict data is serialized correctly."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            result = await fn(
                event="selection_changed",
                data={"selected": ["mesh1", "mesh2"], "meta": {"source": "python"}},
            )

        assert result["status"] == "emitted"

    @pytest.mark.asyncio
    async def test_emit_event_non_dict_response_raises(self) -> None:
        """Raises RuntimeError on unexpected response type."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            with pytest.raises(RuntimeError):
                await fn(event="test_event")
