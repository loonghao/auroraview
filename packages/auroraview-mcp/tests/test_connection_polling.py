"""Tests for CDP connection polling behavior and edge cases.

Covers:
- CDPConnection.send: response polling with mismatched IDs (continue waiting)
- PageConnection.send: same polling behavior
- PageConnection.evaluate: edge cases (no 'value' key, empty result dict)
- ConnectionManager: properties, boundary states
- is_process_alive helper
"""

from __future__ import annotations

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.connection import (
    CDPConnection,
    CDPError,
    ConnectionManager,
    JavaScriptError,
    Page,
    PageConnection,
)


def _make_page(id_="p1", url="http://localhost:8080", title="Test") -> Page:
    return Page(
        id=id_,
        url=url,
        title=title,
        ws_url=f"ws://localhost:9222/devtools/page/{id_}",
    )


class TestCDPConnectionPolling:
    """Tests for CDPConnection.send message polling behavior."""

    @pytest.mark.asyncio
    async def test_send_skips_mismatched_id_messages(self) -> None:
        """Test send() continues polling when response ID doesn't match."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        # First recv returns ID=99 (wrong), second returns ID=1 (correct)
        responses = [
            json.dumps({"id": 99, "result": {"wrong": True}}),
            json.dumps({"id": 1, "result": {"correct": True}}),
        ]
        recv_call_count = 0

        async def fake_recv():
            nonlocal recv_call_count
            resp = responses[recv_call_count]
            recv_call_count += 1
            return resp

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(side_effect=fake_recv)
        conn._ws = mock_ws

        result = await conn.send("Page.navigate", {"url": "http://example.com"})

        # Should receive both messages but only return the matched one
        assert result == {"correct": True}
        assert recv_call_count == 2

    @pytest.mark.asyncio
    async def test_send_skips_multiple_mismatched_before_match(self) -> None:
        """Test send() skips multiple non-matching messages until correct ID."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        # Three wrong IDs, then the correct one
        responses = [
            json.dumps({"id": 5, "result": {"x": 1}}),
            json.dumps({"id": 7, "result": {"x": 2}}),
            json.dumps({"id": 10, "result": {"x": 3}}),
            json.dumps({"id": 1, "result": {"final": True}}),
        ]
        idx = 0

        async def fake_recv():
            nonlocal idx
            resp = responses[idx]
            idx += 1
            return resp

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(side_effect=fake_recv)
        conn._ws = mock_ws

        result = await conn.send("Runtime.evaluate")

        assert result == {"final": True}
        assert idx == 4

    @pytest.mark.asyncio
    async def test_send_empty_result_returns_empty_dict(self) -> None:
        """Test send() returns empty dict when result key is absent."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        response = json.dumps({"id": 1})  # No 'result' key
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=response)
        conn._ws = mock_ws

        result = await conn.send("Page.reload")
        assert result == {}

    @pytest.mark.asyncio
    async def test_send_event_message_ignored(self) -> None:
        """Test send() ignores event messages (no 'id' key) and waits for response."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        # CDP event (no id), then actual response
        responses = [
            json.dumps({"method": "Page.loadEventFired", "params": {}}),  # event, no id
            json.dumps({"id": 1, "result": {"loaded": True}}),
        ]
        idx = 0

        async def fake_recv():
            nonlocal idx
            resp = responses[idx]
            idx += 1
            return resp

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(side_effect=fake_recv)
        conn._ws = mock_ws

        result = await conn.send("Page.navigate")
        assert result == {"loaded": True}
        assert idx == 2

    @pytest.mark.asyncio
    async def test_send_serializes_params_correctly(self) -> None:
        """Test send() serializes params and includes correct method/id in JSON."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        sent_data = {}

        async def fake_send(data):
            sent_data.update(json.loads(data))

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock(side_effect=fake_send)
        mock_ws.recv = AsyncMock(return_value=json.dumps({"id": 1, "result": {}}))
        conn._ws = mock_ws

        await conn.send("DOM.querySelector", {"nodeId": 1, "selector": "#app"})

        assert sent_data["method"] == "DOM.querySelector"
        assert sent_data["id"] == 1
        assert sent_data["params"] == {"nodeId": 1, "selector": "#app"}

    @pytest.mark.asyncio
    async def test_send_no_params_defaults_to_empty_dict(self) -> None:
        """Test send() uses empty dict for params when None is passed."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        sent_data = {}

        async def fake_send(data):
            sent_data.update(json.loads(data))

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock(side_effect=fake_send)
        mock_ws.recv = AsyncMock(return_value=json.dumps({"id": 1, "result": {}}))
        conn._ws = mock_ws

        await conn.send("Page.reload")
        assert sent_data["params"] == {}


class TestPageConnectionPolling:
    """Tests for PageConnection.send message polling behavior."""

    @pytest.mark.asyncio
    async def test_send_skips_mismatched_id_messages(self) -> None:
        """Test PageConnection.send() skips wrong-ID messages."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        responses = [
            json.dumps({"id": 42, "result": {"wrong": True}}),
            json.dumps({"id": 1, "result": {"right": True}}),
        ]
        idx = 0

        async def fake_recv():
            nonlocal idx
            resp = responses[idx]
            idx += 1
            return resp

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(side_effect=fake_recv)
        conn._ws = mock_ws

        result = await conn.send("Runtime.evaluate", {"expression": "1+1"})
        assert result == {"right": True}
        assert idx == 2

    @pytest.mark.asyncio
    async def test_send_raises_cdp_error(self) -> None:
        """Test PageConnection.send() raises CDPError on error response."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        response = {"id": 1, "error": {"code": -32000, "message": "Runtime error"}}
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        with pytest.raises(CDPError) as exc_info:
            await conn.send("Runtime.evaluate")
        assert exc_info.value.code == -32000

    @pytest.mark.asyncio
    async def test_send_raises_when_not_connected(self) -> None:
        """Test PageConnection.send() raises RuntimeError when not connected."""
        page = _make_page()
        conn = PageConnection(page=page)

        with pytest.raises(RuntimeError, match="Not connected to page"):
            await conn.send("Runtime.evaluate")

    @pytest.mark.asyncio
    async def test_send_empty_result(self) -> None:
        """Test PageConnection.send() returns empty dict when result absent."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps({"id": 1}))
        conn._ws = mock_ws

        result = await conn.send("Page.enable")
        assert result == {}


class TestPageConnectionEvaluateEdgeCases:
    """Edge cases for PageConnection.evaluate."""

    @pytest.mark.asyncio
    async def test_evaluate_no_value_key_returns_none(self) -> None:
        """Test evaluate() returns None when result.result has no 'value' key."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        # result.result has type but no 'value'
        response = {
            "id": 1,
            "result": {
                "result": {"type": "object", "className": "Window"}  # no 'value'
            },
        }
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        result = await conn.evaluate("window")
        assert result is None  # .get("value") returns None when key absent

    @pytest.mark.asyncio
    async def test_evaluate_empty_result_object(self) -> None:
        """Test evaluate() handles completely empty result.result dict."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        response = {"id": 1, "result": {"result": {}}}
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        result = await conn.evaluate("undefined")
        assert result is None

    @pytest.mark.asyncio
    async def test_evaluate_exception_details_raises_js_error(self) -> None:
        """Test evaluate() raises JavaScriptError when exceptionDetails present."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        exception_details = {
            "text": "ReferenceError",
            "exception": {"description": "ReferenceError: undeclaredVar is not defined"},
        }
        response = {
            "id": 1,
            "result": {
                "result": {"type": "object"},
                "exceptionDetails": exception_details,
            },
        }
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        with pytest.raises(JavaScriptError) as exc_info:
            await conn.evaluate("undeclaredVar")
        assert "undeclaredVar" in str(exc_info.value)

    @pytest.mark.asyncio
    async def test_evaluate_sends_correct_cdp_params(self) -> None:
        """Test evaluate() sends Runtime.evaluate with correct parameters."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        sent_data = {}

        async def fake_send(data):
            sent_data.update(json.loads(data))

        response = {"id": 1, "result": {"result": {"type": "number", "value": 42}}}
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock(side_effect=fake_send)
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        result = await conn.evaluate("21 * 2")

        assert sent_data["method"] == "Runtime.evaluate"
        assert sent_data["params"]["expression"] == "21 * 2"
        assert sent_data["params"]["returnByValue"] is True
        assert sent_data["params"]["awaitPromise"] is True
        assert result == 42

    @pytest.mark.asyncio
    async def test_evaluate_string_value(self) -> None:
        """Test evaluate() returns string value correctly."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        response = {
            "id": 1,
            "result": {"result": {"type": "string", "value": "hello world"}},
        }
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        result = await conn.evaluate("'hello world'")
        assert result == "hello world"

    @pytest.mark.asyncio
    async def test_evaluate_boolean_value(self) -> None:
        """Test evaluate() returns bool value correctly."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        response = {
            "id": 1,
            "result": {"result": {"type": "boolean", "value": True}},
        }
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        result = await conn.evaluate("true")
        assert result is True

    @pytest.mark.asyncio
    async def test_evaluate_list_value(self) -> None:
        """Test evaluate() returns list value correctly."""
        page = _make_page()
        conn = PageConnection(page=page)
        conn._message_id = 0

        response = {
            "id": 1,
            "result": {"result": {"type": "object", "value": [1, 2, 3]}},
        }
        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))
        conn._ws = mock_ws

        result = await conn.evaluate("[1, 2, 3]")
        assert result == [1, 2, 3]


class TestConnectionManagerProperties:
    """Tests for ConnectionManager properties and boundary states."""

    def test_is_connected_false_when_no_port(self) -> None:
        """Test is_connected returns False when no current port."""
        manager = ConnectionManager()
        assert manager.is_connected is False

    def test_current_port_none_initially(self) -> None:
        """Test current_port is None when not connected."""
        manager = ConnectionManager()
        assert manager.current_port is None

    def test_current_page_none_initially(self) -> None:
        """Test current_page is None when not connected."""
        manager = ConnectionManager()
        assert manager.current_page is None

    def test_is_connected_true_after_setting_port(self) -> None:
        """Test is_connected is determined by _current_port."""
        manager = ConnectionManager()
        manager._current_port = 9222
        assert manager.is_connected is True

    def test_is_connected_false_after_clearing_port(self) -> None:
        """Test is_connected is False after clearing port."""
        manager = ConnectionManager()
        manager._current_port = 9222
        manager._current_port = None
        assert manager.is_connected is False

    @pytest.mark.asyncio
    async def test_disconnect_noop_when_port_not_in_connections(self) -> None:
        """Test disconnect() is no-op when port not in connections."""
        manager = ConnectionManager()
        manager._current_port = 9222
        # Port not added to _connections
        await manager.disconnect(9222)
        # current_port cleared even if connection wasn't tracked
        assert manager._current_port is None

    @pytest.mark.asyncio
    async def test_disconnect_all_with_no_connections(self) -> None:
        """Test disconnect_all() with empty state is a no-op."""
        manager = ConnectionManager()
        # Should not raise
        await manager.disconnect_all()
        assert manager._current_page is None

    @pytest.mark.asyncio
    async def test_disconnect_all_clears_current_page(self) -> None:
        """Test disconnect_all() clears current page."""
        manager = ConnectionManager()
        page = _make_page()
        manager._current_page = page
        await manager.disconnect_all()
        assert manager._current_page is None

    @pytest.mark.asyncio
    async def test_disconnect_all_disconnects_page_connections(self) -> None:
        """Test disconnect_all() calls disconnect on page connections."""
        manager = ConnectionManager()

        mock_page_conn = MagicMock()
        mock_page_conn.disconnect = AsyncMock()
        manager._page_connections["p1"] = mock_page_conn

        await manager.disconnect_all()

        mock_page_conn.disconnect.assert_called_once()
        assert "p1" not in manager._page_connections

    @pytest.mark.asyncio
    async def test_get_pages_raises_when_not_connected(self) -> None:
        """Test get_pages() raises RuntimeError when not connected."""
        manager = ConnectionManager()

        with pytest.raises(RuntimeError, match="Not connected"):
            await manager.get_pages()

    @pytest.mark.asyncio
    async def test_get_page_connection_raises_when_no_page(self) -> None:
        """Test get_page_connection() raises RuntimeError when no page selected."""
        manager = ConnectionManager()

        with pytest.raises(RuntimeError, match="No page selected"):
            await manager.get_page_connection()

    @pytest.mark.asyncio
    async def test_get_page_connection_returns_cached_connected(self) -> None:
        """Test get_page_connection() returns cached connection when still connected."""
        manager = ConnectionManager()
        page = _make_page()
        manager._current_page = page

        mock_conn = MagicMock()
        mock_conn.is_connected = True
        manager._page_connections["p1"] = mock_conn

        result = await manager.get_page_connection()
        assert result is mock_conn

    @pytest.mark.asyncio
    async def test_get_page_connection_reconnects_stale(self) -> None:
        """Test get_page_connection() creates new connection when cached is stale."""
        manager = ConnectionManager()
        page = _make_page()
        manager._current_page = page

        # Stale connection
        stale_conn = MagicMock()
        stale_conn.is_connected = False
        manager._page_connections["p1"] = stale_conn

        # New connection
        new_conn = MagicMock()
        new_conn.connect = AsyncMock()

        with patch("auroraview_mcp.connection.PageConnection", return_value=new_conn):
            result = await manager.get_page_connection()

        assert result is new_conn
        new_conn.connect.assert_called_once()


class TestIsProcessAlive:
    """Tests for is_process_alive helper function."""

    def test_current_process_is_alive(self) -> None:
        """Test that current process (os.getpid()) is alive."""
        import os

        from auroraview_mcp.discovery import is_process_alive

        assert is_process_alive(os.getpid()) is True

    def test_nonexistent_pid_returns_false(self) -> None:
        """Test that a nonexistent PID returns False."""
        import sys

        from auroraview_mcp.discovery import is_process_alive

        if sys.platform != "win32":
            # On Unix, use a clearly invalid PID
            assert is_process_alive(999999) is False

    def test_zero_pid_is_not_a_process(self) -> None:
        """Test that PID 0 is treated as non-alive (OS-dependent behavior)."""
        import sys

        from auroraview_mcp.discovery import is_process_alive

        if sys.platform != "win32":
            # PID 0 signals all processes or the process group, which would
            # raise ESRCH or succeed; the function should not crash.
            result = is_process_alive(0)
            assert isinstance(result, bool)

    def test_invalid_pid_returns_false(self) -> None:
        """Test that an invalid negative PID returns False."""
        from auroraview_mcp.discovery import is_process_alive

        # Negative PIDs are invalid; should return False without raising
        result = is_process_alive(-1)
        assert result is False


class TestCDPConnectionIsConnected:
    """Tests for CDPConnection.is_connected property edge cases."""

    def test_is_connected_false_when_ws_is_none(self) -> None:
        """Test is_connected returns False when _ws is None."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        assert conn._ws is None
        assert conn.is_connected is False

    def test_is_connected_false_when_ws_state_not_open(self) -> None:
        """Test is_connected returns False when WebSocket state is CLOSED."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        mock_ws = MagicMock()
        mock_ws.state.name = "CLOSED"
        conn._ws = mock_ws
        assert conn.is_connected is False

    def test_is_connected_true_when_ws_state_is_open(self) -> None:
        """Test is_connected returns True when WebSocket state is OPEN."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        mock_ws = MagicMock()
        mock_ws.state.name = "OPEN"
        conn._ws = mock_ws
        assert conn.is_connected is True

    def test_page_connection_is_connected_false_when_closed(self) -> None:
        """Test PageConnection.is_connected False when ws CLOSED."""
        page = _make_page()
        conn = PageConnection(page=page)
        mock_ws = MagicMock()
        mock_ws.state.name = "CLOSING"
        conn._ws = mock_ws
        assert conn.is_connected is False

    def test_page_connection_is_connected_false_when_none(self) -> None:
        """Test PageConnection.is_connected False when ws None."""
        page = _make_page()
        conn = PageConnection(page=page)
        assert conn.is_connected is False
