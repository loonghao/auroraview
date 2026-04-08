"""Tests for page tools module (list_pages, select_page, get_page_info, reload_page)."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.connection import Page


def make_page(id_="P1", url="http://localhost:8080", title="Test"):
    return Page(id=id_, url=url, title=title, ws_url=f"ws://localhost:9222/devtools/page/{id_}")


def make_manager(page=True, connected=True, page_conn=None):
    manager = MagicMock()
    manager.is_connected = connected
    manager.current_page = make_page() if page else None
    _page_conn = page_conn or MagicMock()
    manager.get_page_connection = AsyncMock(return_value=_page_conn)
    return manager, _page_conn


class TestListPages:
    """Tests for list_pages tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import list_pages

            fn = list_pages.fn if hasattr(list_pages, "fn") else list_pages
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_returns_page_dicts(self) -> None:
        """Returns list of page dicts from manager.get_pages()."""
        manager, _ = make_manager()
        pages = [
            make_page("A", "http://localhost:8080/a", "Page A"),
            make_page("B", "http://localhost:8080/b", "Page B"),
        ]
        manager.get_pages = AsyncMock(return_value=pages)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import list_pages

            fn = list_pages.fn if hasattr(list_pages, "fn") else list_pages
            result = await fn()

        assert len(result) == 2
        assert result[0]["id"] == "A"
        assert result[0]["url"] == "http://localhost:8080/a"
        assert result[1]["title"] == "Page B"

    @pytest.mark.asyncio
    async def test_returns_empty_list(self) -> None:
        """Returns [] when no pages."""
        manager, _ = make_manager()
        manager.get_pages = AsyncMock(return_value=[])
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import list_pages

            fn = list_pages.fn if hasattr(list_pages, "fn") else list_pages
            result = await fn()

        assert result == []


class TestSelectPage:
    """Tests for select_page tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import select_page

            fn = select_page.fn if hasattr(select_page, "fn") else select_page
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(page_id="P1")

    @pytest.mark.asyncio
    async def test_select_by_id_success(self) -> None:
        """Returns selected=True with page info when found by ID."""
        manager, _ = make_manager()
        page = make_page("P1", "http://localhost/", "My Page")
        manager.select_page = AsyncMock(return_value=page)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import select_page

            fn = select_page.fn if hasattr(select_page, "fn") else select_page
            result = await fn(page_id="P1")

        assert result["selected"] is True
        assert result["id"] == "P1"
        assert result["url"] == "http://localhost/"
        assert result["title"] == "My Page"

    @pytest.mark.asyncio
    async def test_select_by_url_pattern_success(self) -> None:
        """Returns selected=True when page found by URL pattern."""
        manager, _ = make_manager()
        page = make_page("P2", "http://localhost:8080/app", "App")
        manager.select_page = AsyncMock(return_value=page)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import select_page

            fn = select_page.fn if hasattr(select_page, "fn") else select_page
            result = await fn(url_pattern="*localhost*")

        assert result["selected"] is True
        assert result["id"] == "P2"

    @pytest.mark.asyncio
    async def test_select_not_found_returns_false(self) -> None:
        """Returns selected=False when no matching page."""
        manager, _ = make_manager()
        manager.select_page = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import select_page

            fn = select_page.fn if hasattr(select_page, "fn") else select_page
            result = await fn(page_id="MISSING")

        assert result["selected"] is False
        assert "error" in result

    @pytest.mark.asyncio
    async def test_select_no_criteria_auto_selects_first(self) -> None:
        """Selecting without criteria auto-selects first page."""
        manager, _ = make_manager()
        page = make_page("P3", "http://localhost/default", "Default")
        manager.select_page = AsyncMock(return_value=page)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import select_page

            fn = select_page.fn if hasattr(select_page, "fn") else select_page
            result = await fn()

        assert result["selected"] is True
        assert result["id"] == "P3"


class TestGetPageInfo:
    """Tests for get_page_info tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_auroraview_ready(self) -> None:
        """Returns auroraview_ready=True with methods list."""
        manager, page_conn = make_manager()
        # Two evaluate calls: first for ready check, second for methods
        page_conn.evaluate = AsyncMock(side_effect=[True, ["echo", "get_scene"]])
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            result = await fn()

        assert result["auroraview_ready"] is True
        assert result["api_methods"] == ["echo", "get_scene"]
        assert "id" in result
        assert "url" in result
        assert "title" in result

    @pytest.mark.asyncio
    async def test_auroraview_not_ready(self) -> None:
        """Returns auroraview_ready=False when bridge absent."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            result = await fn()

        assert result["auroraview_ready"] is False
        assert result["api_methods"] == []

    @pytest.mark.asyncio
    async def test_evaluate_exception_is_swallowed(self) -> None:
        """Exceptions from evaluate are silently caught."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(side_effect=Exception("CDP error"))
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            result = await fn()

        assert result["auroraview_ready"] is False
        assert result["api_methods"] == []

    @pytest.mark.asyncio
    async def test_methods_not_list_returns_empty(self) -> None:
        """Returns empty api_methods when evaluate returns non-list."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(side_effect=[True, None])
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            result = await fn()

        assert result["api_methods"] == []


class TestReloadPage:
    """Tests for reload_page tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import reload_page

            fn = reload_page.fn if hasattr(reload_page, "fn") else reload_page
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import reload_page

            fn = reload_page.fn if hasattr(reload_page, "fn") else reload_page
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_soft_reload(self) -> None:
        """Calls Page.reload with ignoreCache=False by default."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import reload_page

            fn = reload_page.fn if hasattr(reload_page, "fn") else reload_page
            result = await fn()

        assert result["status"] == "reloaded"
        assert result["hard"] is False
        page_conn.send.assert_called_once_with("Page.reload", {"ignoreCache": False})

    @pytest.mark.asyncio
    async def test_hard_reload(self) -> None:
        """Calls Page.reload with ignoreCache=True when hard=True."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import reload_page

            fn = reload_page.fn if hasattr(reload_page, "fn") else reload_page
            result = await fn(hard=True)

        assert result["status"] == "reloaded"
        assert result["hard"] is True
        page_conn.send.assert_called_once_with("Page.reload", {"ignoreCache": True})
