"""Tests for UI tools module (take_screenshot, click, fill, evaluate, hover, get_snapshot)."""

from __future__ import annotations

import base64
import tempfile
from pathlib import Path
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


class TestTakeScreenshot:
    """Tests for take_screenshot tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        """Raises RuntimeError when not connected."""
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_selected_raises(self) -> None:
        """Raises RuntimeError when no page is selected."""
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_basic_screenshot_returns_data_url(self) -> None:
        """Returns base64 data URL when no path given."""
        page_conn = MagicMock()
        png_data = base64.b64encode(b"fake-png-data").decode()
        page_conn.send = AsyncMock(return_value={"data": png_data})
        page_conn.evaluate = AsyncMock(return_value=None)

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn()

        assert result.startswith("data:image/png;base64,")
        assert png_data in result

    @pytest.mark.asyncio
    async def test_screenshot_save_to_path(self) -> None:
        """Saves screenshot to file when path is given."""
        page_conn = MagicMock()
        png_bytes = b"\x89PNG fake"
        png_data = base64.b64encode(png_bytes).decode()
        page_conn.send = AsyncMock(return_value={"data": png_data})
        page_conn.evaluate = AsyncMock(return_value=None)

        manager, _ = make_manager(page_conn=page_conn)
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
            out_path = f.name

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn(path=out_path)

        assert result == out_path
        assert Path(out_path).read_bytes() == png_bytes

    @pytest.mark.asyncio
    async def test_screenshot_with_selector(self) -> None:
        """Passes clip params when selector has bounds."""
        page_conn = MagicMock()
        png_data = base64.b64encode(b"data").decode()
        page_conn.send = AsyncMock(return_value={"data": png_data})
        # evaluate returns bounds for the element
        page_conn.evaluate = AsyncMock(
            return_value={"x": 10.0, "y": 20.0, "width": 300.0, "height": 150.0, "scale": 2.0}
        )

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn(selector="#canvas")

        # send should have been called with clip params
        send_call = page_conn.send.call_args
        assert send_call[0][0] == "Page.captureScreenshot"
        params = send_call[0][1]
        assert "clip" in params
        assert params["clip"]["x"] == 10.0
        assert params["clip"]["width"] == 300.0
        assert result.startswith("data:image/png;base64,")

    @pytest.mark.asyncio
    async def test_screenshot_selector_not_found(self) -> None:
        """No clip when element not found (evaluate returns None)."""
        page_conn = MagicMock()
        png_data = base64.b64encode(b"data").decode()
        page_conn.send = AsyncMock(return_value={"data": png_data})
        page_conn.evaluate = AsyncMock(return_value=None)

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            await fn(selector="#nonexistent")

        send_call = page_conn.send.call_args
        params = send_call[0][1]
        assert "clip" not in params

    @pytest.mark.asyncio
    async def test_screenshot_full_page(self) -> None:
        """Sets captureBeyondViewport=True for full_page=True."""
        page_conn = MagicMock()
        page_conn.send = AsyncMock(return_value={"data": base64.b64encode(b"x").decode()})
        page_conn.evaluate = AsyncMock(return_value=None)

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            await fn(full_page=True)

        send_call = page_conn.send.call_args
        params = send_call[0][1]
        assert params["captureBeyondViewport"] is True


class TestClick:
    """Tests for click tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(selector="#btn")

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn(selector="#btn")

    @pytest.mark.asyncio
    async def test_no_selector_or_uid_raises(self) -> None:
        """Raises ValueError when neither selector nor uid is given."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(ValueError, match="selector or uid"):
                await fn()

    @pytest.mark.asyncio
    async def test_click_by_selector_success(self) -> None:
        """Returns status=clicked when element found."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            result = await fn(selector="#submit")

        assert result["status"] == "clicked"
        assert result["selector"] == "#submit"

    @pytest.mark.asyncio
    async def test_click_element_not_found_raises(self) -> None:
        """Raises RuntimeError when element is not found."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": False, "error": "Element not found"})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(RuntimeError, match="Element not found"):
                await fn(selector="#missing")

    @pytest.mark.asyncio
    async def test_click_by_uid_not_implemented(self) -> None:
        """Click by UID raises NotImplementedError."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(NotImplementedError):
                await fn(uid="12345")


class TestFill:
    """Tests for fill tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(selector="#input", value="hello")

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn(selector="#input", value="hello")

    @pytest.mark.asyncio
    async def test_fill_success(self) -> None:
        """Returns status=filled with selector and value."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            result = await fn(selector="#name", value="Alice")

        assert result["status"] == "filled"
        assert result["selector"] == "#name"
        assert result["value"] == "Alice"

    @pytest.mark.asyncio
    async def test_fill_element_not_found_raises(self) -> None:
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": False, "error": "Element not found"})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            with pytest.raises(RuntimeError, match="Element not found"):
                await fn(selector="#missing", value="x")

    @pytest.mark.asyncio
    async def test_fill_escapes_special_chars(self) -> None:
        """Value with quotes/backslashes does not break script."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            result = await fn(selector='input[name="q"]', value='say "hello" & \\escape')

        assert result["status"] == "filled"

    @pytest.mark.asyncio
    async def test_fill_multiline_value(self) -> None:
        """Multiline value is escaped properly."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            result = await fn(selector="textarea", value="line1\nline2")

        assert result["status"] == "filled"


class TestEvaluate:
    """Tests for evaluate tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import evaluate

            fn = evaluate.fn if hasattr(evaluate, "fn") else evaluate
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(script="1+1")

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import evaluate

            fn = evaluate.fn if hasattr(evaluate, "fn") else evaluate
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn(script="1+1")

    @pytest.mark.asyncio
    async def test_evaluate_returns_result(self) -> None:
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=42)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import evaluate

            fn = evaluate.fn if hasattr(evaluate, "fn") else evaluate
            result = await fn(script="21 + 21")

        assert result == 42

    @pytest.mark.asyncio
    async def test_evaluate_returns_dict(self) -> None:
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"key": "value"})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import evaluate

            fn = evaluate.fn if hasattr(evaluate, "fn") else evaluate
            result = await fn(script="({key:'value'})")

        assert result == {"key": "value"}

    @pytest.mark.asyncio
    async def test_evaluate_returns_none(self) -> None:
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import evaluate

            fn = evaluate.fn if hasattr(evaluate, "fn") else evaluate
            result = await fn(script="void 0")

        assert result is None


class TestHover:
    """Tests for hover tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import hover

            fn = hover.fn if hasattr(hover, "fn") else hover
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn(selector="#btn")

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import hover

            fn = hover.fn if hasattr(hover, "fn") else hover
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn(selector="#btn")

    @pytest.mark.asyncio
    async def test_hover_success(self) -> None:
        """Returns status=hovered when element found."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"x": 100.0, "y": 200.0})
        page_conn.send = AsyncMock(return_value={})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import hover

            fn = hover.fn if hasattr(hover, "fn") else hover
            result = await fn(selector="#menu")

        assert result["status"] == "hovered"
        assert result["selector"] == "#menu"
        page_conn.send.assert_called_once_with(
            "Input.dispatchMouseEvent",
            {"type": "mouseMoved", "x": 100.0, "y": 200.0},
        )

    @pytest.mark.asyncio
    async def test_hover_element_not_found_raises(self) -> None:
        """Raises RuntimeError when element not found."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        page_conn.send = AsyncMock(return_value={})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import hover

            fn = hover.fn if hasattr(hover, "fn") else hover
            with pytest.raises(RuntimeError, match="Element not found"):
                await fn(selector="#ghost")


class TestGetSnapshot:
    """Tests for get_snapshot tool."""

    @pytest.mark.asyncio
    async def test_not_connected_raises(self) -> None:
        manager, _ = make_manager(connected=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            with pytest.raises(RuntimeError, match="Not connected"):
                await fn()

    @pytest.mark.asyncio
    async def test_no_page_raises(self) -> None:
        manager, _ = make_manager(page=False)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            with pytest.raises(RuntimeError, match="No page selected"):
                await fn()

    @pytest.mark.asyncio
    async def test_snapshot_returns_nodes(self) -> None:
        """Returns simplified node list with count."""
        manager, page_conn = make_manager()
        ax_nodes = [
            {
                "nodeId": "1",
                "ignored": False,
                "role": {"value": "button"},
                "name": {"value": "Submit"},
                "properties": [
                    {"name": "disabled", "value": {"value": False}},
                ],
            },
            {
                "nodeId": "2",
                "ignored": True,  # should be filtered
                "role": {"value": "generic"},
                "name": {"value": ""},
                "properties": [],
            },
            {
                "nodeId": "3",
                "ignored": False,
                "role": {"value": "textbox"},
                "name": {"value": "Username"},
                "properties": [],
            },
        ]
        page_conn.send = AsyncMock(return_value={"nodes": ax_nodes})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            result = await fn()

        assert result["count"] == 2  # ignored node filtered out
        nodes = result["nodes"]
        assert nodes[0]["role"] == "button"
        assert nodes[0]["name"] == "Submit"
        assert nodes[0]["disabled"] is False
        assert nodes[1]["role"] == "textbox"

    @pytest.mark.asyncio
    async def test_snapshot_empty(self) -> None:
        """Returns empty list when no nodes."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={"nodes": []})
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            result = await fn()

        assert result["count"] == 0
        assert result["nodes"] == []
