"""Extended tests for the connection module.

Covers CDPConnection, PageConnection, CDPError, JavaScriptError,
and ConnectionManager advanced operations.
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


class TestPage:
    """Tests for Page dataclass."""

    def test_page_creation(self) -> None:
        """Test Page creation with all fields."""
        page = Page(
            id="p1",
            url="http://localhost:8080",
            title="Test Page",
            ws_url="ws://localhost:9222/devtools/page/p1",
            type="page",
        )
        assert page.id == "p1"
        assert page.url == "http://localhost:8080"
        assert page.title == "Test Page"
        assert page.ws_url == "ws://localhost:9222/devtools/page/p1"
        assert page.type == "page"

    def test_page_default_type(self) -> None:
        """Test Page defaults to 'page' type."""
        page = Page(
            id="p1",
            url="http://localhost",
            title="",
            ws_url="",
        )
        assert page.type == "page"

    def test_to_dict(self) -> None:
        """Test Page.to_dict() includes all fields."""
        page = Page(
            id="abc",
            url="http://localhost:9000",
            title="My App",
            ws_url="ws://localhost:9222/devtools/page/abc",
            type="page",
        )
        d = page.to_dict()
        assert d["id"] == "abc"
        assert d["url"] == "http://localhost:9000"
        assert d["title"] == "My App"
        assert d["ws_url"] == "ws://localhost:9222/devtools/page/abc"
        assert d["type"] == "page"


class TestCDPError:
    """Tests for CDPError exception."""

    def test_cdp_error_message_and_code(self) -> None:
        """Test CDPError stores code and message."""
        error = CDPError({"code": -32601, "message": "Method not found"})
        assert error.code == -32601
        assert error.message == "Method not found"

    def test_cdp_error_str(self) -> None:
        """Test CDPError string representation."""
        error = CDPError({"code": -32600, "message": "Invalid Request"})
        assert "CDP Error" in str(error)
        assert "-32600" in str(error)
        assert "Invalid Request" in str(error)

    def test_cdp_error_defaults(self) -> None:
        """Test CDPError with missing code/message fields."""
        error = CDPError({})
        assert error.code == -1
        assert error.message == "Unknown error"

    def test_cdp_error_is_exception(self) -> None:
        """Test CDPError can be raised and caught."""
        with pytest.raises(CDPError) as exc_info:
            raise CDPError({"code": 0, "message": "test"})
        assert exc_info.value.code == 0


class TestJavaScriptError:
    """Tests for JavaScriptError exception."""

    def test_javascript_error_with_description(self) -> None:
        """Test JavaScriptError with exception description."""
        error = JavaScriptError(
            {
                "text": "Uncaught ReferenceError",
                "exception": {"description": "ReferenceError: foo is not defined"},
            }
        )
        assert "foo is not defined" in str(error)

    def test_javascript_error_fallback_to_text(self) -> None:
        """Test JavaScriptError falls back to text when no description."""
        error = JavaScriptError(
            {
                "text": "Script failed",
                "exception": {},
            }
        )
        assert "Script failed" in str(error)

    def test_javascript_error_stores_details(self) -> None:
        """Test JavaScriptError stores raw details."""
        details = {
            "text": "TypeError",
            "exception": {"description": "TypeError: Cannot read property"},
            "lineNumber": 42,
        }
        error = JavaScriptError(details)
        assert error.details == details

    def test_javascript_error_is_exception(self) -> None:
        """Test JavaScriptError can be raised and caught."""
        with pytest.raises(JavaScriptError):
            raise JavaScriptError({"text": "Error", "exception": {}})


class TestCDPConnection:
    """Tests for CDPConnection class."""

    def test_cdp_connection_initial_state(self) -> None:
        """Test CDPConnection initial state."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        assert conn.port == 9222
        assert conn.ws_url == "ws://localhost:9222"
        assert conn.is_connected is False

    @pytest.mark.asyncio
    async def test_connect_creates_websocket(self) -> None:
        """Test connect() establishes WebSocket connection."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")

        mock_ws = MagicMock()
        mock_ws.state.name = "OPEN"

        with patch(
            "auroraview_mcp.connection.websockets.connect",
            new_callable=AsyncMock,
            return_value=mock_ws,
        ):
            await conn.connect()

        assert conn._ws == mock_ws
        assert conn.is_connected is True

    @pytest.mark.asyncio
    async def test_disconnect_closes_websocket(self) -> None:
        """Test disconnect() closes WebSocket."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")

        mock_ws = MagicMock()
        mock_ws.close = AsyncMock()

        conn._ws = mock_ws

        await conn.disconnect()

        mock_ws.close.assert_called_once()
        assert conn._ws is None

    @pytest.mark.asyncio
    async def test_disconnect_when_not_connected(self) -> None:
        """Test disconnect() when not connected is a no-op."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        # Should not raise
        await conn.disconnect()

    @pytest.mark.asyncio
    async def test_send_command_and_receive_response(self) -> None:
        """Test send() sends command and returns matching response."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")

        mock_ws = MagicMock()
        response = {"id": 1, "result": {"value": "test_result"}}
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))

        conn._ws = mock_ws
        conn._message_id = 0

        result = await conn.send("Page.navigate", {"url": "http://example.com"})

        assert result == {"value": "test_result"}
        mock_ws.send.assert_called_once()

    @pytest.mark.asyncio
    async def test_send_raises_cdp_error(self) -> None:
        """Test send() raises CDPError on CDP error response."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")

        mock_ws = MagicMock()
        response = {"id": 1, "error": {"code": -32601, "message": "Method not found"}}
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps(response))

        conn._ws = mock_ws

        with pytest.raises(CDPError) as exc_info:
            await conn.send("Page.nonexistent")

        assert exc_info.value.code == -32601

    @pytest.mark.asyncio
    async def test_send_raises_when_not_connected(self) -> None:
        """Test send() raises RuntimeError when not connected."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")

        with pytest.raises(RuntimeError, match="Not connected"):
            await conn.send("Page.navigate")

    @pytest.mark.asyncio
    async def test_send_increments_message_id(self) -> None:
        """Test send() increments message_id for each call."""
        conn = CDPConnection(port=9222, ws_url="ws://localhost:9222")
        conn._message_id = 0

        call_count = 0
        sent_ids = []

        async def fake_send(data):
            nonlocal call_count
            call_count += 1
            parsed = json.loads(data)
            sent_ids.append(parsed["id"])

        async def fake_recv():
            return json.dumps({"id": call_count, "result": {}})

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock(side_effect=fake_send)
        mock_ws.recv = AsyncMock(side_effect=fake_recv)
        conn._ws = mock_ws

        await conn.send("Method.one")
        await conn.send("Method.two")

        assert sent_ids[0] == 1
        assert sent_ids[1] == 2


class TestPageConnection:
    """Tests for PageConnection class."""

    def _make_page(self) -> Page:
        return Page(
            id="p1",
            url="http://localhost:8080",
            title="Test",
            ws_url="ws://localhost:9222/devtools/page/p1",
        )

    def test_page_connection_initial_state(self) -> None:
        """Test PageConnection initial state."""
        page = self._make_page()
        conn = PageConnection(page=page)
        assert conn.page == page
        assert conn.is_connected is False

    @pytest.mark.asyncio
    async def test_connect_creates_websocket(self) -> None:
        """Test connect() establishes WebSocket connection."""
        page = self._make_page()
        conn = PageConnection(page=page)

        mock_ws = MagicMock()
        mock_ws.state.name = "OPEN"

        with patch(
            "auroraview_mcp.connection.websockets.connect",
            new_callable=AsyncMock,
            return_value=mock_ws,
        ):
            await conn.connect()

        assert conn._ws is mock_ws
        assert conn.is_connected is True

    @pytest.mark.asyncio
    async def test_disconnect_closes_websocket(self) -> None:
        """Test disconnect() closes WebSocket."""
        page = self._make_page()
        conn = PageConnection(page=page)

        mock_ws = MagicMock()
        mock_ws.close = AsyncMock()
        conn._ws = mock_ws

        await conn.disconnect()

        mock_ws.close.assert_called_once()
        assert conn._ws is None

    @pytest.mark.asyncio
    async def test_evaluate_returns_value(self) -> None:
        """Test evaluate() returns JavaScript result value."""
        page = self._make_page()
        conn = PageConnection(page=page)

        result_response = {
            "result": {"type": "string", "value": "hello"},
        }

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps({"id": 1, "result": result_response}))
        conn._ws = mock_ws

        result = await conn.evaluate("'hello'")

        assert result == "hello"

    @pytest.mark.asyncio
    async def test_evaluate_returns_none_for_undefined(self) -> None:
        """Test evaluate() returns None when result has no value."""
        page = self._make_page()
        conn = PageConnection(page=page)

        result_response = {
            "result": {"type": "undefined"},
        }

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps({"id": 1, "result": result_response}))
        conn._ws = mock_ws

        result = await conn.evaluate("undefined")

        assert result is None

    @pytest.mark.asyncio
    async def test_evaluate_raises_javascript_error(self) -> None:
        """Test evaluate() raises JavaScriptError on exception."""
        page = self._make_page()
        conn = PageConnection(page=page)

        result_response = {
            "result": {"type": "object"},
            "exceptionDetails": {
                "text": "Uncaught ReferenceError",
                "exception": {"description": "ReferenceError: x is not defined"},
            },
        }

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value=json.dumps({"id": 1, "result": result_response}))
        conn._ws = mock_ws

        with pytest.raises(JavaScriptError):
            await conn.evaluate("x")

    @pytest.mark.asyncio
    async def test_send_raises_when_not_connected(self) -> None:
        """Test send() raises RuntimeError when not connected."""
        page = self._make_page()
        conn = PageConnection(page=page)

        with pytest.raises(RuntimeError, match="Not connected to page"):
            await conn.send("Runtime.evaluate")


class TestConnectionManager:
    """Tests for ConnectionManager class."""

    def test_initial_state(self) -> None:
        """Test ConnectionManager initial state."""
        manager = ConnectionManager()
        assert manager.is_connected is False
        assert manager.current_port is None
        assert manager.current_page is None

    @pytest.mark.asyncio
    async def test_connect_returns_cached_connection(self) -> None:
        """Test connect() returns existing connection when already connected."""
        manager = ConnectionManager()

        mock_conn = MagicMock()
        mock_conn.is_connected = True
        manager._connections[9222] = mock_conn

        with patch.object(manager, "_current_port", 9222):
            manager._connections[9222] = mock_conn
            result = await manager.connect(9222)

        assert result is mock_conn

    @pytest.mark.asyncio
    async def test_connect_creates_new_connection(self) -> None:
        """Test connect() creates new connection when not cached."""
        manager = ConnectionManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "webSocketDebuggerUrl": "ws://localhost:9222/devtools/browser/xxx"
        }

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        mock_conn = MagicMock()
        mock_conn.connect = AsyncMock()
        mock_conn.is_connected = True

        with (
            patch("auroraview_mcp.connection.httpx.AsyncClient", return_value=mock_client),
            patch("auroraview_mcp.connection.CDPConnection", return_value=mock_conn),
        ):
            result = await manager.connect(9222)

        assert manager._current_port == 9222
        assert result is mock_conn

    @pytest.mark.asyncio
    async def test_connect_raises_when_no_ws_url(self) -> None:
        """Test connect() raises ConnectionError when no WebSocket URL."""
        manager = ConnectionManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {}  # No webSocketDebuggerUrl

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with (
            patch("auroraview_mcp.connection.httpx.AsyncClient", return_value=mock_client),
            pytest.raises(ConnectionError, match="No WebSocket URL"),
        ):
            await manager.connect(9222)

    @pytest.mark.asyncio
    async def test_disconnect_removes_connection(self) -> None:
        """Test disconnect() removes connection and clears current port."""
        manager = ConnectionManager()

        mock_conn = MagicMock()
        mock_conn.disconnect = AsyncMock()
        manager._connections[9222] = mock_conn
        manager._current_port = 9222

        await manager.disconnect(9222)

        assert 9222 not in manager._connections
        assert manager._current_port is None

    @pytest.mark.asyncio
    async def test_disconnect_current_when_no_port_specified(self) -> None:
        """Test disconnect() disconnects current port when none specified."""
        manager = ConnectionManager()

        mock_conn = MagicMock()
        mock_conn.disconnect = AsyncMock()
        manager._connections[9222] = mock_conn
        manager._current_port = 9222

        await manager.disconnect()

        assert 9222 not in manager._connections

    @pytest.mark.asyncio
    async def test_disconnect_noop_when_not_connected(self) -> None:
        """Test disconnect() does nothing when not connected."""
        manager = ConnectionManager()
        # Should not raise
        await manager.disconnect()

    @pytest.mark.asyncio
    async def test_disconnect_all(self) -> None:
        """Test disconnect_all() clears all connections and pages."""
        manager = ConnectionManager()

        mock_conn1 = MagicMock()
        mock_conn1.disconnect = AsyncMock()
        mock_conn2 = MagicMock()
        mock_conn2.disconnect = AsyncMock()

        mock_page_conn = MagicMock()
        mock_page_conn.disconnect = AsyncMock()

        manager._connections = {9222: mock_conn1, 9223: mock_conn2}
        manager._page_connections = {"p1": mock_page_conn}
        manager._current_port = 9222
        manager._current_page = MagicMock()

        await manager.disconnect_all()

        assert manager._connections == {}
        assert manager._page_connections == {}
        assert manager._current_page is None
        mock_conn1.disconnect.assert_called_once()
        mock_conn2.disconnect.assert_called_once()
        mock_page_conn.disconnect.assert_called_once()

    @pytest.mark.asyncio
    async def test_get_pages_raises_when_not_connected(self) -> None:
        """Test get_pages() raises RuntimeError when not connected."""
        manager = ConnectionManager()

        with pytest.raises(RuntimeError, match="Not connected"):
            await manager.get_pages()

    @pytest.mark.asyncio
    async def test_get_pages_filters_blank_and_non_page(self) -> None:
        """Test get_pages() filters out about:blank and non-page targets."""
        manager = ConnectionManager()
        manager._current_port = 9222

        pages_data = [
            {
                "id": "p1",
                "url": "http://localhost:8080",
                "title": "App",
                "webSocketDebuggerUrl": "ws://localhost:9222/p1",
                "type": "page",
            },
            {
                "id": "p2",
                "url": "about:blank",
                "title": "",
                "webSocketDebuggerUrl": "ws://localhost:9222/p2",
                "type": "page",
            },
            {
                "id": "p3",
                "url": "chrome-extension://",
                "title": "Extension",
                "webSocketDebuggerUrl": "ws://localhost:9222/p3",
                "type": "background_page",
            },
        ]

        mock_response = MagicMock()
        mock_response.json.return_value = pages_data

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.connection.httpx.AsyncClient", return_value=mock_client):
            pages = await manager.get_pages()

        assert len(pages) == 1
        assert pages[0].id == "p1"

    @pytest.mark.asyncio
    async def test_select_page_by_id(self) -> None:
        """Test select_page() selects page by ID."""
        manager = ConnectionManager()

        pages = [
            Page(id="p1", url="http://localhost:8080", title="App", ws_url="ws://p1"),
            Page(id="p2", url="http://localhost:8081", title="App2", ws_url="ws://p2"),
        ]

        with patch.object(manager, "get_pages", new_callable=AsyncMock, return_value=pages):
            result = await manager.select_page(page_id="p2")

        assert result is not None
        assert result.id == "p2"
        assert manager._current_page == pages[1]

    @pytest.mark.asyncio
    async def test_select_page_by_url_pattern(self) -> None:
        """Test select_page() selects page by URL pattern."""
        manager = ConnectionManager()

        pages = [
            Page(id="p1", url="http://localhost:8080/tool", title="Tool", ws_url="ws://p1"),
            Page(id="p2", url="http://localhost:8081/panel", title="Panel", ws_url="ws://p2"),
        ]

        with patch.object(manager, "get_pages", new_callable=AsyncMock, return_value=pages):
            result = await manager.select_page(url_pattern="*:8081/*")

        assert result is not None
        assert result.id == "p2"

    @pytest.mark.asyncio
    async def test_select_page_auto_selects_first(self) -> None:
        """Test select_page() auto-selects first page when no criteria."""
        manager = ConnectionManager()

        pages = [
            Page(id="p1", url="http://localhost:8080", title="First", ws_url="ws://p1"),
            Page(id="p2", url="http://localhost:8081", title="Second", ws_url="ws://p2"),
        ]

        with patch.object(manager, "get_pages", new_callable=AsyncMock, return_value=pages):
            result = await manager.select_page()

        assert result is not None
        assert result.id == "p1"

    @pytest.mark.asyncio
    async def test_select_page_returns_none_when_not_found(self) -> None:
        """Test select_page() returns None when page not found."""
        manager = ConnectionManager()

        pages = [Page(id="p1", url="http://localhost:8080", title="App", ws_url="ws://p1")]

        with patch.object(manager, "get_pages", new_callable=AsyncMock, return_value=pages):
            result = await manager.select_page(page_id="nonexistent")

        assert result is None

    @pytest.mark.asyncio
    async def test_select_page_returns_none_when_no_pages(self) -> None:
        """Test select_page() returns None when no pages available."""
        manager = ConnectionManager()

        with patch.object(manager, "get_pages", new_callable=AsyncMock, return_value=[]):
            result = await manager.select_page()

        assert result is None

    @pytest.mark.asyncio
    async def test_get_page_connection_raises_when_no_page(self) -> None:
        """Test get_page_connection() raises RuntimeError when no page selected."""
        manager = ConnectionManager()

        with pytest.raises(RuntimeError, match="No page selected"):
            await manager.get_page_connection()

    @pytest.mark.asyncio
    async def test_get_page_connection_returns_cached(self) -> None:
        """Test get_page_connection() returns cached connection."""
        manager = ConnectionManager()
        page = Page(id="p1", url="http://localhost", title="App", ws_url="ws://p1")
        manager._current_page = page

        mock_conn = MagicMock()
        mock_conn.is_connected = True
        manager._page_connections["p1"] = mock_conn

        result = await manager.get_page_connection()

        assert result is mock_conn

    @pytest.mark.asyncio
    async def test_get_page_connection_creates_new(self) -> None:
        """Test get_page_connection() creates new connection when not cached."""
        manager = ConnectionManager()
        page = Page(id="p1", url="http://localhost", title="App", ws_url="ws://p1")
        manager._current_page = page

        mock_conn = MagicMock()
        mock_conn.connect = AsyncMock()
        mock_conn.is_connected = True

        with patch("auroraview_mcp.connection.PageConnection", return_value=mock_conn):
            result = await manager.get_page_connection()

        assert result is mock_conn
        assert manager._page_connections["p1"] is mock_conn

    @pytest.mark.asyncio
    async def test_get_page_connection_reconnects_if_disconnected(self) -> None:
        """Test get_page_connection() creates new connection if existing is disconnected."""
        manager = ConnectionManager()
        page = Page(id="p1", url="http://localhost", title="App", ws_url="ws://p1")
        manager._current_page = page

        # Stale disconnected connection
        stale_conn = MagicMock()
        stale_conn.is_connected = False
        manager._page_connections["p1"] = stale_conn

        new_conn = MagicMock()
        new_conn.connect = AsyncMock()
        new_conn.is_connected = True

        with patch("auroraview_mcp.connection.PageConnection", return_value=new_conn):
            result = await manager.get_page_connection()

        assert result is new_conn

    @pytest.mark.asyncio
    async def test_get_page_connection_accepts_explicit_page(self) -> None:
        """Test get_page_connection() accepts explicit page parameter."""
        manager = ConnectionManager()
        # No current page set
        explicit_page = Page(id="p_explicit", url="http://other", title="Other", ws_url="ws://e1")

        mock_conn = MagicMock()
        mock_conn.connect = AsyncMock()
        mock_conn.is_connected = True

        with patch("auroraview_mcp.connection.PageConnection", return_value=mock_conn):
            result = await manager.get_page_connection(page=explicit_page)

        assert result is mock_conn
