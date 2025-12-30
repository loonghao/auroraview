"""Tests for connection management module."""

from __future__ import annotations

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
        """Test creating a Page."""
        page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test Page",
            ws_url="ws://127.0.0.1:9222/devtools/page/ABC123",
        )
        assert page.id == "ABC123"
        assert page.url == "http://localhost:8080"
        assert page.title == "Test Page"
        assert page.type == "page"

    def test_page_to_dict(self) -> None:
        """Test Page.to_dict()."""
        page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test Page",
            ws_url="ws://127.0.0.1:9222/devtools/page/ABC123",
        )
        d = page.to_dict()
        assert d["id"] == "ABC123"
        assert d["url"] == "http://localhost:8080"
        assert d["title"] == "Test Page"


class TestCDPConnection:
    """Tests for CDPConnection class."""

    def test_connection_creation(self) -> None:
        """Test creating a CDPConnection."""
        conn = CDPConnection(
            port=9222,
            ws_url="ws://127.0.0.1:9222/devtools/browser/xxx",
        )
        assert conn.port == 9222
        assert conn.ws_url == "ws://127.0.0.1:9222/devtools/browser/xxx"
        assert not conn.is_connected

    @pytest.mark.asyncio
    async def test_connect(self) -> None:
        """Test connecting to WebSocket."""
        conn = CDPConnection(
            port=9222,
            ws_url="ws://127.0.0.1:9222/devtools/browser/xxx",
        )

        mock_ws = MagicMock()
        mock_ws.open = True

        with patch(
            "auroraview_mcp.connection.websockets.connect", new_callable=AsyncMock
        ) as mock_connect:
            mock_connect.return_value = mock_ws
            await conn.connect()

            # After connect, _ws should be set
            assert conn._ws is not None
            mock_connect.assert_called_once_with(conn.ws_url)

    @pytest.mark.asyncio
    async def test_disconnect(self) -> None:
        """Test disconnecting from WebSocket."""
        conn = CDPConnection(
            port=9222,
            ws_url="ws://127.0.0.1:9222/devtools/browser/xxx",
        )

        mock_ws = MagicMock()
        mock_ws.close = AsyncMock()
        conn._ws = mock_ws

        await conn.disconnect()

        mock_ws.close.assert_called_once()
        assert conn._ws is None

    @pytest.mark.asyncio
    async def test_send_not_connected(self) -> None:
        """Test send when not connected."""
        conn = CDPConnection(
            port=9222,
            ws_url="ws://127.0.0.1:9222/devtools/browser/xxx",
        )

        with pytest.raises(RuntimeError, match="Not connected"):
            await conn.send("Page.navigate", {"url": "http://example.com"})


class TestCDPError:
    """Tests for CDPError exception."""

    def test_cdp_error(self) -> None:
        """Test CDPError creation."""
        error = CDPError({"code": -32600, "message": "Invalid Request"})
        assert error.code == -32600
        assert error.message == "Invalid Request"
        assert "CDP Error -32600: Invalid Request" in str(error)


class TestJavaScriptError:
    """Tests for JavaScriptError exception."""

    def test_javascript_error(self) -> None:
        """Test JavaScriptError creation."""
        error = JavaScriptError(
            {
                "text": "ReferenceError",
                "exception": {"description": "foo is not defined"},
            }
        )
        assert "foo is not defined" in str(error)


class TestPageConnection:
    """Tests for PageConnection class."""

    def test_page_connection_creation(self) -> None:
        """Test creating a PageConnection."""
        page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test",
            ws_url="ws://127.0.0.1:9222/devtools/page/ABC123",
        )
        conn = PageConnection(page=page)
        assert conn.page == page
        assert not conn.is_connected

    @pytest.mark.asyncio
    async def test_evaluate(self) -> None:
        """Test JavaScript evaluation."""
        page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test",
            ws_url="ws://127.0.0.1:9222/devtools/page/ABC123",
        )
        conn = PageConnection(page=page)

        mock_ws = MagicMock()
        mock_ws.open = True
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(return_value='{"id": 1, "result": {"result": {"value": 42}}}')
        conn._ws = mock_ws

        result = await conn.evaluate("1 + 1")
        assert result == 42


class TestConnectionManager:
    """Tests for ConnectionManager class."""

    def test_manager_creation(self) -> None:
        """Test creating a ConnectionManager."""
        manager = ConnectionManager()
        assert manager.current_port is None
        assert manager.current_page is None
        assert not manager.is_connected

    @pytest.mark.asyncio
    async def test_connect(self) -> None:
        """Test connecting to an instance."""
        manager = ConnectionManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/xxx",
        }

        mock_ws = MagicMock()
        mock_ws.open = True

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(return_value=mock_response)
            mock_client.return_value = mock_instance

            with patch("websockets.connect", new_callable=AsyncMock) as mock_connect:
                mock_connect.return_value = mock_ws

                conn = await manager.connect(9222)

                assert conn is not None
                assert manager.current_port == 9222
                assert manager.is_connected

    @pytest.mark.asyncio
    async def test_disconnect(self) -> None:
        """Test disconnecting from an instance."""
        manager = ConnectionManager()

        # Setup mock connection
        mock_ws = MagicMock()
        mock_ws.open = True
        mock_ws.close = AsyncMock()

        mock_conn = CDPConnection(port=9222, ws_url="ws://test")
        mock_conn._ws = mock_ws

        manager._connections[9222] = mock_conn
        manager._current_port = 9222

        await manager.disconnect()

        assert manager.current_port is None
        assert 9222 not in manager._connections

    @pytest.mark.asyncio
    async def test_get_pages_not_connected(self) -> None:
        """Test get_pages when not connected."""
        manager = ConnectionManager()

        with pytest.raises(RuntimeError, match="Not connected"):
            await manager.get_pages()

    @pytest.mark.asyncio
    async def test_select_page_by_id(self) -> None:
        """Test selecting a page by ID."""
        manager = ConnectionManager()
        manager._current_port = 9222

        mock_response = MagicMock()
        mock_response.json.return_value = [
            {
                "id": "ABC123",
                "url": "http://localhost:8080",
                "title": "Test Page",
                "type": "page",
                "webSocketDebuggerUrl": "ws://test",
            },
        ]

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(return_value=mock_response)
            mock_client.return_value = mock_instance

            page = await manager.select_page(page_id="ABC123")

            assert page is not None
            assert page.id == "ABC123"
            assert manager.current_page == page

    @pytest.mark.asyncio
    async def test_select_page_by_url_pattern(self) -> None:
        """Test selecting a page by URL pattern."""
        manager = ConnectionManager()
        manager._current_port = 9222

        mock_response = MagicMock()
        mock_response.json.return_value = [
            {
                "id": "ABC123",
                "url": "http://localhost:8080/app",
                "title": "Test Page",
                "type": "page",
                "webSocketDebuggerUrl": "ws://test",
            },
        ]

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(return_value=mock_response)
            mock_client.return_value = mock_instance

            page = await manager.select_page(url_pattern="*localhost*")

            assert page is not None
            assert "localhost" in page.url
