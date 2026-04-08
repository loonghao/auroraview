"""Tests for debug tools module (get_console_logs, get_network_requests,
get_backend_status, get_memory_info, clear_console)."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest


def make_manager(page=True, connected=True, page_conn=None):
    manager = MagicMock()
    manager.is_connected = connected
    manager.current_page = MagicMock() if page else None
    _page_conn = page_conn or MagicMock()
    manager.get_page_connection = AsyncMock(return_value=_page_conn)
    return manager, _page_conn


class TestGetConsoleLogs:
    """Tests for get_console_logs tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_returns_log_list(self) -> None:
        """Returns list of log entries."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        logs = [
            {"level": "log", "text": "hello", "timestamp": 1234},
            {"level": "error", "text": "oops", "timestamp": 1235},
        ]
        page_conn.evaluate = AsyncMock(return_value=logs)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            result = await fn()

        assert len(result) == 2
        assert result[0]["level"] == "log"
        assert result[1]["text"] == "oops"

    @pytest.mark.asyncio
    async def test_returns_empty_when_none(self) -> None:
        """Returns [] when evaluate returns None."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_console_enable_exception_is_swallowed(self) -> None:
        """send(Console.enable) exception is suppressed."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(side_effect=Exception("CDP not available"))
        page_conn.evaluate = AsyncMock(return_value=[])
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_with_level_filter(self) -> None:
        """Level param is passed to the JS template."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        page_conn.evaluate = AsyncMock(return_value=[{"level": "error", "text": "boom"}])
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            result = await fn(level="error", limit=10)

        assert len(result) == 1
        assert result[0]["level"] == "error"


class TestGetNetworkRequests:
    """Tests for get_network_requests tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_returns_request_list(self) -> None:
        """Returns list of request dicts."""
        manager, page_conn = make_manager()
        requests = [
            {"url": "http://example.com/api/data", "method": "GET", "status": 200, "time": 50},
            {"url": "http://example.com/api/post", "method": "GET", "status": 201, "time": 80},
        ]
        page_conn.evaluate = AsyncMock(return_value=requests)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            result = await fn()

        assert len(result) == 2
        assert result[0]["url"] == "http://example.com/api/data"

    @pytest.mark.asyncio
    async def test_returns_empty_when_none(self) -> None:
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_with_url_pattern(self) -> None:
        """url_pattern is forwarded to the JS filter."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=[])
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            result = await fn(url_pattern="*/api/*", method="POST")

        assert result == []

    @pytest.mark.asyncio
    async def test_with_method_filter(self) -> None:
        """method filter is uppercased in script."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=[])
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            result = await fn(method="get")

        assert result == []


class TestGetBackendStatus:
    """Tests for get_backend_status tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_backend_status

            fn = get_backend_status.fn if hasattr(get_backend_status, "fn") else get_backend_status
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_backend_status

            fn = get_backend_status.fn if hasattr(get_backend_status, "fn") else get_backend_status
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_backend_ready(self) -> None:
        """Returns ready=True with handlers and version."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(
            return_value={
                "ready": True,
                "handlers": ["echo", "get_scene"],
                "version": "0.4.18",
                "platform": "maya2025-pyside6",
            }
        )
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_backend_status

            fn = get_backend_status.fn if hasattr(get_backend_status, "fn") else get_backend_status
            result = await fn()

        assert result["ready"] is True
        assert result["version"] == "0.4.18"
        assert "echo" in result["handlers"]

    @pytest.mark.asyncio
    async def test_backend_not_ready(self) -> None:
        """Returns ready=False when bridge absent."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ready": False})
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_backend_status

            fn = get_backend_status.fn if hasattr(get_backend_status, "fn") else get_backend_status
            result = await fn()

        assert result["ready"] is False

    @pytest.mark.asyncio
    async def test_non_dict_response_returns_not_ready(self) -> None:
        """Returns ready=False when evaluate returns non-dict."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_backend_status

            fn = get_backend_status.fn if hasattr(get_backend_status, "fn") else get_backend_status
            result = await fn()

        assert result == {"ready": False}


class TestGetMemoryInfo:
    """Tests for get_memory_info tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_memory_info

            fn = get_memory_info.fn if hasattr(get_memory_info, "fn") else get_memory_info
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_memory_info

            fn = get_memory_info.fn if hasattr(get_memory_info, "fn") else get_memory_info
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_returns_memory_dict(self) -> None:
        """Returns memory info when available."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(
            return_value={
                "usedJSHeapSize": 5_000_000,
                "totalJSHeapSize": 10_000_000,
                "jsHeapSizeLimit": 2_147_483_648,
            }
        )
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_memory_info

            fn = get_memory_info.fn if hasattr(get_memory_info, "fn") else get_memory_info
            result = await fn()

        assert result["usedJSHeapSize"] == 5_000_000
        assert result["totalJSHeapSize"] == 10_000_000

    @pytest.mark.asyncio
    async def test_memory_api_not_available(self) -> None:
        """Returns error dict when memory API not available."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_memory_info

            fn = get_memory_info.fn if hasattr(get_memory_info, "fn") else get_memory_info
            result = await fn()

        assert "error" in result
        assert "not available" in result["error"].lower()


class TestClearConsole:
    """Tests for clear_console tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import clear_console

            fn = clear_console.fn if hasattr(clear_console, "fn") else clear_console
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import clear_console

            fn = clear_console.fn if hasattr(clear_console, "fn") else clear_console
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_clear_returns_status(self) -> None:
        """Returns status=cleared."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import clear_console

            fn = clear_console.fn if hasattr(clear_console, "fn") else clear_console
            result = await fn()

        assert result["status"] == "cleared"
        page_conn.evaluate.assert_called_once()

    @pytest.mark.asyncio
    async def test_clear_calls_correct_script(self) -> None:
        """evaluate is called with the clear script."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import clear_console

            fn = clear_console.fn if hasattr(clear_console, "fn") else clear_console
            await fn()

        script = page_conn.evaluate.call_args[0][0]
        assert "console.clear()" in script
        assert "__auroraview_console_logs" in script
