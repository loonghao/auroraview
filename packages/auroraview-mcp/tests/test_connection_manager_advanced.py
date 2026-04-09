"""Advanced unit tests for ConnectionManager.

Covers connect() flow with mock httpx, get_pages() edge cases,
select_page() fnmatch edge cases, and get_page_connection() behavior.
"""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.connection import (
    CDPConnection,
    ConnectionManager,
    Page,
    PageConnection,
)


def _make_version_response(ws_url: str = "ws://127.0.0.1:9222/devtools/browser/abc") -> MagicMock:
    """Build a mock response for /json/version."""
    resp = MagicMock()
    resp.status_code = 200
    resp.json.return_value = {"webSocketDebuggerUrl": ws_url}
    return resp


def _make_list_response(pages: list[dict]) -> MagicMock:
    """Build a mock response for /json/list."""
    resp = MagicMock()
    resp.status_code = 200
    resp.json.return_value = pages
    return resp


class TestConnectionManagerConnect:
    """Tests for ConnectionManager.connect() using mocked httpx."""

    @pytest.mark.asyncio
    async def test_connect_fresh_creates_cdp_connection(self) -> None:
        """New connection is created when no cached connection exists."""
        version_resp = _make_version_response("ws://127.0.0.1:9222/devtools/browser/test")

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=version_resp)
            mock_client_cls.return_value = mock_http

            with patch("websockets.connect", new_callable=AsyncMock) as mock_ws:
                mock_ws.return_value = MagicMock()
                mock_ws.return_value.state.name = "OPEN"

                manager = ConnectionManager()
                conn = await manager.connect(9222)

        assert isinstance(conn, CDPConnection)
        assert conn.port == 9222
        assert conn.ws_url == "ws://127.0.0.1:9222/devtools/browser/test"
        assert manager.current_port == 9222

    @pytest.mark.asyncio
    async def test_connect_returns_cached_when_connected(self) -> None:
        """Returns existing connection without making HTTP request when already connected."""
        existing_conn = MagicMock(spec=CDPConnection)
        existing_conn.is_connected = True
        existing_conn.port = 9222

        manager = ConnectionManager()
        manager._connections[9222] = existing_conn

        with patch("httpx.AsyncClient") as mock_client_cls:
            conn = await manager.connect(9222)

        # Should not call httpx at all
        mock_client_cls.assert_not_called()
        assert conn is existing_conn
        assert manager.current_port == 9222

    @pytest.mark.asyncio
    async def test_connect_recreates_when_disconnected(self) -> None:
        """Creates new connection when cached connection is disconnected."""
        stale_conn = MagicMock(spec=CDPConnection)
        stale_conn.is_connected = False

        version_resp = _make_version_response("ws://127.0.0.1:9223/devtools/browser/new")

        manager = ConnectionManager()
        manager._connections[9223] = stale_conn

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=version_resp)
            mock_client_cls.return_value = mock_http

            with patch("websockets.connect", new_callable=AsyncMock) as mock_ws:
                mock_ws.return_value = MagicMock()
                mock_ws.return_value.state.name = "OPEN"

                conn = await manager.connect(9223)

        assert conn is not stale_conn
        assert conn.port == 9223

    @pytest.mark.asyncio
    async def test_connect_raises_when_no_ws_url_in_response(self) -> None:
        """Raises ConnectionError when /json/version returns no webSocketDebuggerUrl."""
        resp = MagicMock()
        resp.status_code = 200
        resp.json.return_value = {}  # no webSocketDebuggerUrl

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            with pytest.raises(ConnectionError, match="No WebSocket URL"):
                await manager.connect(9222)

    @pytest.mark.asyncio
    async def test_connect_updates_current_port(self) -> None:
        """connect() updates current_port to the new port."""
        version_resp_1 = _make_version_response("ws://127.0.0.1:9222/devtools/browser/a")
        version_resp_2 = _make_version_response("ws://127.0.0.1:9223/devtools/browser/b")

        mock_ws_conn = MagicMock()
        mock_ws_conn.state.name = "OPEN"

        manager = ConnectionManager()

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_client_cls.return_value = mock_http

            with patch("websockets.connect", new_callable=AsyncMock) as mock_ws:
                mock_ws.return_value = mock_ws_conn

                mock_http.get = AsyncMock(return_value=version_resp_1)
                await manager.connect(9222)
                assert manager.current_port == 9222

                mock_http.get = AsyncMock(return_value=version_resp_2)
                await manager.connect(9223)
                assert manager.current_port == 9223

    @pytest.mark.asyncio
    async def test_connect_stores_connection_in_dict(self) -> None:
        """Connection is stored under port key in _connections."""
        version_resp = _make_version_response("ws://127.0.0.1:9222/devtools/browser/x")

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=version_resp)
            mock_client_cls.return_value = mock_http

            with patch("websockets.connect", new_callable=AsyncMock) as mock_ws:
                mock_ws.return_value = MagicMock()
                mock_ws.return_value.state.name = "OPEN"

                manager = ConnectionManager()
                conn = await manager.connect(9222)

        assert 9222 in manager._connections
        assert manager._connections[9222] is conn


class TestConnectionManagerGetPages:
    """Tests for ConnectionManager.get_pages() edge cases."""

    @pytest.mark.asyncio
    async def test_get_pages_with_explicit_port(self) -> None:
        """get_pages() uses explicit port instead of current_port."""
        pages_data = [
            {"id": "p1", "url": "http://localhost:8080", "title": "App", "type": "page",
             "webSocketDebuggerUrl": "ws://127.0.0.1:9224/devtools/page/p1"},
        ]
        list_resp = _make_list_response(pages_data)

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=list_resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            manager._current_port = 9222  # default, but we pass explicit
            pages = await manager.get_pages(port=9224)

        # Should have used port 9224
        call_url = mock_http.get.call_args[0][0]
        assert "9224" in call_url
        assert len(pages) == 1

    @pytest.mark.asyncio
    async def test_get_pages_filters_about_blank(self) -> None:
        """about:blank pages are excluded."""
        pages_data = [
            {"id": "p1", "url": "about:blank", "title": "", "type": "page",
             "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/page/p1"},
            {"id": "p2", "url": "http://localhost:8080", "title": "Real", "type": "page",
             "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/page/p2"},
        ]
        list_resp = _make_list_response(pages_data)

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=list_resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            manager._current_port = 9222
            pages = await manager.get_pages()

        assert len(pages) == 1
        assert pages[0].url == "http://localhost:8080"

    @pytest.mark.asyncio
    async def test_get_pages_filters_non_page_type(self) -> None:
        """Non-'page' type targets (worker, iframe) are excluded."""
        pages_data = [
            {"id": "p1", "url": "http://localhost:8080", "title": "Page", "type": "page",
             "webSocketDebuggerUrl": "ws://..."},
            {"id": "w1", "url": "http://localhost:8080/worker.js", "title": "", "type": "worker",
             "webSocketDebuggerUrl": "ws://..."},
            {"id": "i1", "url": "http://localhost:8080/iframe", "title": "", "type": "iframe",
             "webSocketDebuggerUrl": "ws://..."},
        ]
        list_resp = _make_list_response(pages_data)

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=list_resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            manager._current_port = 9222
            pages = await manager.get_pages()

        assert len(pages) == 1
        assert pages[0].type == "page"

    @pytest.mark.asyncio
    async def test_get_pages_empty_list_returns_empty(self) -> None:
        """Empty JSON list response returns empty pages list."""
        list_resp = _make_list_response([])

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=list_resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            manager._current_port = 9222
            pages = await manager.get_pages()

        assert pages == []

    @pytest.mark.asyncio
    async def test_get_pages_all_blank_returns_empty(self) -> None:
        """All about:blank pages results in empty list."""
        pages_data = [
            {"id": "p1", "url": "about:blank", "title": "", "type": "page",
             "webSocketDebuggerUrl": "ws://..."},
            {"id": "p2", "url": "about:blank", "title": "", "type": "page",
             "webSocketDebuggerUrl": "ws://..."},
        ]
        list_resp = _make_list_response(pages_data)

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=list_resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            manager._current_port = 9222
            pages = await manager.get_pages()

        assert pages == []

    @pytest.mark.asyncio
    async def test_get_pages_page_with_missing_ws_url(self) -> None:
        """Page with missing webSocketDebuggerUrl gets empty string for ws_url."""
        pages_data = [
            {"id": "p1", "url": "http://localhost:8080", "title": "No WS", "type": "page"},
        ]
        list_resp = _make_list_response(pages_data)

        with patch("httpx.AsyncClient") as mock_client_cls:
            mock_http = AsyncMock()
            mock_http.__aenter__ = AsyncMock(return_value=mock_http)
            mock_http.__aexit__ = AsyncMock(return_value=None)
            mock_http.get = AsyncMock(return_value=list_resp)
            mock_client_cls.return_value = mock_http

            manager = ConnectionManager()
            manager._current_port = 9222
            pages = await manager.get_pages()

        assert len(pages) == 1
        assert pages[0].ws_url == ""

    @pytest.mark.asyncio
    async def test_get_pages_raises_when_no_port(self) -> None:
        """Raises RuntimeError when no current_port and no port argument."""
        manager = ConnectionManager()
        with pytest.raises(RuntimeError, match="Not connected"):
            await manager.get_pages()


class TestSelectPageFnmatch:
    """Tests for select_page() URL pattern matching edge cases."""

    def _make_pages(self, urls: list[str]) -> list[Page]:
        return [
            Page(id=str(i), url=url, title=f"Page {i}", ws_url="")
            for i, url in enumerate(urls)
        ]

    @pytest.mark.asyncio
    async def test_fnmatch_wildcard_matches_subdomain(self) -> None:
        """Pattern *.localhost matches sub.localhost."""
        pages = self._make_pages(["http://sub.localhost:8080/app"])

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = pages
            result = await manager.select_page(url_pattern="http://sub.localhost*")

        assert result is not None
        assert "sub.localhost" in result.url

    @pytest.mark.asyncio
    async def test_fnmatch_pattern_returns_first_match(self) -> None:
        """When multiple pages match, first one is returned."""
        pages = self._make_pages([
            "http://localhost:8080/app",
            "http://localhost:8080/other",
        ])

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = pages
            result = await manager.select_page(url_pattern="http://localhost*")

        assert result is not None
        assert result.id == "0"  # first match

    @pytest.mark.asyncio
    async def test_fnmatch_no_match_returns_none(self) -> None:
        """Non-matching pattern returns None."""
        pages = self._make_pages(["http://localhost:8080/app"])

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = pages
            result = await manager.select_page(url_pattern="http://remote-host*")

        assert result is None

    @pytest.mark.asyncio
    async def test_fnmatch_exact_url_match(self) -> None:
        """Exact URL string matches (fnmatch with no wildcards)."""
        target_url = "http://localhost:8080/exact"
        pages = self._make_pages([target_url, "http://localhost:8080/other"])

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = pages
            result = await manager.select_page(url_pattern=target_url)

        assert result is not None
        assert result.url == target_url

    @pytest.mark.asyncio
    async def test_fnmatch_pattern_with_question_mark(self) -> None:
        """? wildcard matches a single character."""
        pages = self._make_pages([
            "http://localhost:8080/v1/api",
            "http://localhost:8080/v2/api",
            "http://localhost:8080/v10/api",  # won't match v? pattern (2 chars)
        ])

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = pages
            result = await manager.select_page(
                url_pattern="http://localhost:8080/v?/api"
            )

        # Either v1 or v2 (first one wins), but NOT v10
        assert result is not None
        assert result.url in ("http://localhost:8080/v1/api", "http://localhost:8080/v2/api")

    @pytest.mark.asyncio
    async def test_select_page_by_id_takes_priority(self) -> None:
        """page_id lookup is checked before url_pattern when both provided."""
        target = Page(id="target-id", url="http://target", title="T", ws_url="")
        other = Page(id="other-id", url="http://localhost:8080", title="O", ws_url="")

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = [target, other]
            result = await manager.select_page(page_id="target-id", url_pattern="http://localhost*")

        assert result is not None
        assert result.id == "target-id"

    @pytest.mark.asyncio
    async def test_select_page_updates_current_page(self) -> None:
        """Selected page is stored as current_page."""
        page = Page(id="p1", url="http://app.test", title="App", ws_url="ws://...")

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = [page]
            result = await manager.select_page(url_pattern="http://app*")

        assert manager.current_page is result
        assert manager.current_page.id == "p1"

    @pytest.mark.asyncio
    async def test_select_page_auto_first_page(self) -> None:
        """No criteria: first page is selected."""
        pages = [
            Page(id="first", url="http://a.test", title="A", ws_url=""),
            Page(id="second", url="http://b.test", title="B", ws_url=""),
        ]

        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = pages
            result = await manager.select_page()

        assert result is not None
        assert result.id == "first"

    @pytest.mark.asyncio
    async def test_select_page_empty_pages_returns_none(self) -> None:
        """Returns None when no pages available."""
        manager = ConnectionManager()
        manager._current_port = 9222

        with patch.object(manager, "get_pages", new_callable=AsyncMock) as mock_get:
            mock_get.return_value = []
            result = await manager.select_page()

        assert result is None


class TestConnectionManagerDisconnect:
    """Tests for disconnect() and disconnect_all() behavior."""

    @pytest.mark.asyncio
    async def test_disconnect_by_port_removes_connection(self) -> None:
        """disconnect(port) removes the connection from dict."""
        conn = AsyncMock(spec=CDPConnection)
        manager = ConnectionManager()
        manager._connections[9222] = conn
        manager._current_port = 9222

        await manager.disconnect(9222)

        assert 9222 not in manager._connections
        assert manager.current_port is None
        conn.disconnect.assert_called_once()

    @pytest.mark.asyncio
    async def test_disconnect_none_uses_current_port(self) -> None:
        """disconnect() without port argument uses current_port."""
        conn = AsyncMock(spec=CDPConnection)
        manager = ConnectionManager()
        manager._connections[9223] = conn
        manager._current_port = 9223

        await manager.disconnect()

        assert 9223 not in manager._connections
        conn.disconnect.assert_called_once()

    @pytest.mark.asyncio
    async def test_disconnect_noop_when_no_port(self) -> None:
        """disconnect() when no current_port does nothing."""
        manager = ConnectionManager()
        # Should not raise
        await manager.disconnect()

    @pytest.mark.asyncio
    async def test_disconnect_all_clears_page_connections(self) -> None:
        """disconnect_all() also clears page connections."""
        conn = AsyncMock(spec=CDPConnection)
        page_conn = AsyncMock(spec=PageConnection)
        manager = ConnectionManager()
        manager._connections[9222] = conn
        manager._page_connections["page-1"] = page_conn
        manager._current_port = 9222
        manager._current_page = MagicMock()

        await manager.disconnect_all()

        assert manager._connections == {}
        assert manager._page_connections == {}
        assert manager._current_page is None
        page_conn.disconnect.assert_called_once()

    @pytest.mark.asyncio
    async def test_disconnect_unknown_port_is_noop(self) -> None:
        """Disconnecting a port not in connections is safe."""
        manager = ConnectionManager()
        manager._current_port = 9222
        # 9999 is not in _connections
        await manager.disconnect(9999)
        # current_port unchanged because target_port != current_port
        assert manager.current_port == 9222


class TestConnectionManagerProperties:
    """Tests for ConnectionManager property accessors."""

    def test_is_connected_false_when_no_port(self) -> None:
        """is_connected returns False when current_port is None."""
        manager = ConnectionManager()
        assert manager.is_connected is False

    def test_is_connected_true_when_port_set(self) -> None:
        """is_connected returns True when current_port is set."""
        manager = ConnectionManager()
        manager._current_port = 9222
        assert manager.is_connected is True

    def test_current_port_reflects_state(self) -> None:
        """current_port returns the stored port."""
        manager = ConnectionManager()
        assert manager.current_port is None
        manager._current_port = 9225
        assert manager.current_port == 9225

    def test_current_page_reflects_state(self) -> None:
        """current_page returns the stored page."""
        manager = ConnectionManager()
        assert manager.current_page is None
        page = Page(id="x", url="http://test", title="", ws_url="")
        manager._current_page = page
        assert manager.current_page is page
