"""Edge-case tests for UI, debug, and API tools — covering previously untested branches.

Targets:
- take_screenshot: missing 'data' key → empty base64 suffix
- click: non-dict result from evaluate → 'Click failed' fallback
- fill: non-dict result (None) from evaluate → 'Fill failed' fallback
- get_snapshot: property value=None filtered out; node with missing role/name; all-ignored nodes
- get_console_logs: evaluate returns a dict (non-list) → []
- get_network_requests: evaluate returns a dict (non-list) → []
- get_backend_status: evaluate returns empty string → {"ready": False}
- call_api: ok=False with missing error key → 'Unknown error' fallback
- call_api: method with single segment (no dot) → generic call path
- emit_event: ok key absent in response (dict without 'ok') → raises
- list_api_methods: evaluate returns string → []
- get_page_info: auroraview_ready True, api_methods evaluate returns dict (non-list) → []
"""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest


def make_manager(page=True, connected=True, page_conn=None):
    """Build a mock ConnectionManager."""
    manager = MagicMock()
    manager.is_connected = connected
    manager.current_page = MagicMock() if page else None
    _conn = page_conn or MagicMock()
    manager.get_page_connection = AsyncMock(return_value=_conn)
    return manager, _conn


# ---------------------------------------------------------------------------
# take_screenshot edge cases
# ---------------------------------------------------------------------------


class TestTakeScreenshotEdgeCases:
    """Edge cases for take_screenshot not covered by test_ui_tools.py."""

    @pytest.mark.asyncio
    async def test_missing_data_key_returns_empty_base64(self) -> None:
        """When 'data' key is absent from result, returns 'data:image/png;base64,'."""
        page_conn = MagicMock()
        # No 'data' key in response
        page_conn.send = AsyncMock(return_value={"format": "png"})
        page_conn.evaluate = AsyncMock(return_value=None)

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn()

        assert result == "data:image/png;base64,"

    @pytest.mark.asyncio
    async def test_empty_data_value_returns_empty_base64(self) -> None:
        """When 'data' is empty string, returns 'data:image/png;base64,'."""
        page_conn = MagicMock()
        page_conn.send = AsyncMock(return_value={"data": ""})
        page_conn.evaluate = AsyncMock(return_value=None)

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn()

        assert result == "data:image/png;base64,"

    @pytest.mark.asyncio
    async def test_selector_bounds_scale_defaults_to_1(self) -> None:
        """When selector bounds has no 'scale' key, clip.scale defaults to 1."""
        import base64

        page_conn = MagicMock()
        png_data = base64.b64encode(b"px").decode()
        page_conn.send = AsyncMock(return_value={"data": png_data})
        # No 'scale' key in bounds
        page_conn.evaluate = AsyncMock(
            return_value={"x": 0.0, "y": 0.0, "width": 100.0, "height": 50.0}
        )

        manager, _ = make_manager(page_conn=page_conn)
        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn(selector=".box")

        send_call = page_conn.send.call_args
        params = send_call[0][1]
        assert params["clip"]["scale"] == 1  # default fallback
        assert result.startswith("data:image/png;base64,")


# ---------------------------------------------------------------------------
# click edge cases
# ---------------------------------------------------------------------------


class TestClickEdgeCases:
    """Edge cases for click not covered by test_ui_tools.py."""

    @pytest.mark.asyncio
    async def test_non_dict_result_raises_with_default_message(self) -> None:
        """When evaluate returns a non-dict (e.g. None), raises 'Click failed'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(RuntimeError, match="Click failed"):
                await fn(selector="#btn")

    @pytest.mark.asyncio
    async def test_false_string_result_raises(self) -> None:
        """When evaluate returns a string (truthy non-dict), raises 'Click failed'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value="error_string")

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(RuntimeError, match="Click failed"):
                await fn(selector="#btn")

    @pytest.mark.asyncio
    async def test_dict_ok_false_no_error_key_raises_generic(self) -> None:
        """When ok=False but no error key, raises with generic message."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": False})

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import click

            fn = click.fn if hasattr(click, "fn") else click
            with pytest.raises(RuntimeError, match="Click failed"):
                await fn(selector="#btn")


# ---------------------------------------------------------------------------
# fill edge cases
# ---------------------------------------------------------------------------


class TestFillEdgeCases:
    """Edge cases for fill not covered by test_ui_tools.py."""

    @pytest.mark.asyncio
    async def test_non_dict_result_raises_fill_failed(self) -> None:
        """When evaluate returns None (non-dict), raises 'Fill failed'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            with pytest.raises(RuntimeError, match="Fill failed"):
                await fn(selector="#input", value="test")

    @pytest.mark.asyncio
    async def test_integer_result_raises_fill_failed(self) -> None:
        """When evaluate returns an integer (non-dict), raises 'Fill failed'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=0)

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            with pytest.raises(RuntimeError, match="Fill failed"):
                await fn(selector="#input", value="test")

    @pytest.mark.asyncio
    async def test_ok_false_no_error_key_raises_fill_failed(self) -> None:
        """When ok=False and no error key, raises 'Fill failed'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": False})

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import fill

            fn = fill.fn if hasattr(fill, "fn") else fill
            with pytest.raises(RuntimeError, match="Fill failed"):
                await fn(selector="#input", value="test")


# ---------------------------------------------------------------------------
# get_snapshot edge cases
# ---------------------------------------------------------------------------


class TestGetSnapshotEdgeCases:
    """Edge cases for get_snapshot not covered by test_ui_tools.py."""

    @pytest.mark.asyncio
    async def test_property_with_none_value_is_skipped(self) -> None:
        """Properties whose value dict contains None are not added to node."""
        manager, page_conn = make_manager()
        ax_nodes = [
            {
                "nodeId": "1",
                "ignored": False,
                "role": {"value": "button"},
                "name": {"value": "OK"},
                "properties": [
                    {"name": "focusable", "value": {"value": None}},  # None → skip
                    {"name": "disabled", "value": {"value": False}},  # False is not None → include
                ],
            }
        ]
        page_conn.send = AsyncMock(return_value={"nodes": ax_nodes})

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            result = await fn()

        node = result["nodes"][0]
        assert "focusable" not in node  # None value was skipped
        assert node["disabled"] is False  # False was included

    @pytest.mark.asyncio
    async def test_node_missing_role_and_name_uses_empty_string(self) -> None:
        """Nodes without role/name dicts default to empty string."""
        manager, page_conn = make_manager()
        ax_nodes = [
            {
                "nodeId": "99",
                "ignored": False,
                # No 'role' or 'name' keys
                "properties": [],
            }
        ]
        page_conn.send = AsyncMock(return_value={"nodes": ax_nodes})

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            result = await fn()

        assert result["count"] == 1
        assert result["nodes"][0]["role"] == ""
        assert result["nodes"][0]["name"] == ""

    @pytest.mark.asyncio
    async def test_all_ignored_nodes_returns_empty(self) -> None:
        """When all nodes are ignored, returns empty list."""
        manager, page_conn = make_manager()
        ax_nodes = [
            {"nodeId": str(i), "ignored": True, "role": {}, "name": {}, "properties": []}
            for i in range(5)
        ]
        page_conn.send = AsyncMock(return_value={"nodes": ax_nodes})

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            result = await fn()

        assert result["count"] == 0
        assert result["nodes"] == []

    @pytest.mark.asyncio
    async def test_property_without_name_key_is_skipped(self) -> None:
        """Property dicts missing 'name' key are safely skipped."""
        manager, page_conn = make_manager()
        ax_nodes = [
            {
                "nodeId": "1",
                "ignored": False,
                "role": {"value": "link"},
                "name": {"value": "Click"},
                "properties": [
                    {"value": {"value": "something"}},  # no 'name' key → skipped
                ],
            }
        ]
        page_conn.send = AsyncMock(return_value={"nodes": ax_nodes})

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import get_snapshot

            fn = get_snapshot.fn if hasattr(get_snapshot, "fn") else get_snapshot
            result = await fn()

        node = result["nodes"][0]
        # Only uid, role, name should be present (no extra property added)
        assert set(node.keys()) == {"uid", "role", "name"}


# ---------------------------------------------------------------------------
# get_console_logs edge cases
# ---------------------------------------------------------------------------


class TestGetConsoleLogsEdgeCases:
    """Edge cases for get_console_logs not fully covered."""

    @pytest.mark.asyncio
    async def test_evaluate_returns_dict_not_list_returns_empty(self) -> None:
        """When evaluate returns a dict (not a list), returns []."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        page_conn.evaluate = AsyncMock(return_value={"level": "log", "text": "oops"})

        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_evaluate_returns_integer_returns_empty(self) -> None:
        """When evaluate returns an integer, returns []."""
        manager, page_conn = make_manager()
        page_conn.send = AsyncMock(return_value={})
        page_conn.evaluate = AsyncMock(return_value=42)

        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_console_logs

            fn = get_console_logs.fn if hasattr(get_console_logs, "fn") else get_console_logs
            result = await fn()

        assert result == []


# ---------------------------------------------------------------------------
# get_network_requests edge cases
# ---------------------------------------------------------------------------


class TestGetNetworkRequestsEdgeCases:
    """Edge cases for get_network_requests not fully covered."""

    @pytest.mark.asyncio
    async def test_evaluate_returns_dict_returns_empty(self) -> None:
        """When evaluate returns a dict (not a list), returns []."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"url": "http://example.com"})

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
    async def test_evaluate_returns_string_returns_empty(self) -> None:
        """When evaluate returns a string, returns []."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value="not-a-list")

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
    async def test_no_url_pattern_no_method_empty_filters(self) -> None:
        """Without filters both url_filter and method_filter are empty strings."""
        manager, page_conn = make_manager()
        requests = [{"url": "http://a.com", "method": "GET", "status": 200, "time": 10}]
        page_conn.evaluate = AsyncMock(return_value=requests)

        with patch("auroraview_mcp.tools.debug.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.debug import get_network_requests

            fn = (
                get_network_requests.fn
                if hasattr(get_network_requests, "fn")
                else get_network_requests
            )
            result = await fn()

        assert len(result) == 1


# ---------------------------------------------------------------------------
# call_api edge cases
# ---------------------------------------------------------------------------


class TestCallApiEdgeCases:
    """Edge cases for call_api not fully covered by test_api_tools.py."""

    @pytest.mark.asyncio
    async def test_ok_false_missing_error_key_raises_unknown_error(self) -> None:
        """When ok=False and 'error' key is absent, raises 'Unknown error'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": False})

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            with pytest.raises(RuntimeError, match="Unknown error"):
                await fn(method="api.missing_key_test")

    @pytest.mark.asyncio
    async def test_single_segment_method_uses_generic_call_path(self) -> None:
        """Method with no dot (e.g. 'refresh') uses the generic call path."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": "ok"})

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="refresh")

        assert result == "ok"

    @pytest.mark.asyncio
    async def test_three_segment_method_uses_generic_call_path(self) -> None:
        """Method like 'a.b.c' (3 parts) uses the generic call path."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": 123})

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="a.b.c")

        assert result == 123

    @pytest.mark.asyncio
    async def test_non_api_prefix_method_uses_generic_path(self) -> None:
        """Method like 'tool.apply' (non-api prefix) uses generic call path."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": True, "result": None})

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import call_api

            fn = call_api.fn if hasattr(call_api, "fn") else call_api
            result = await fn(method="scene.export")

        assert result is None


# ---------------------------------------------------------------------------
# emit_event edge cases
# ---------------------------------------------------------------------------


class TestEmitEventEdgeCases:
    """Edge cases for emit_event not covered by test_api_tools.py."""

    @pytest.mark.asyncio
    async def test_ok_key_absent_in_dict_raises(self) -> None:
        """When result is dict but 'ok' key is absent, raises RuntimeError."""
        manager, page_conn = make_manager()
        # dict without 'ok' key → result.get("ok") == None (falsy)
        page_conn.evaluate = AsyncMock(return_value={"status": "partial"})

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            with pytest.raises(RuntimeError):
                await fn(event="test_no_ok")

    @pytest.mark.asyncio
    async def test_ok_false_missing_error_key_raises_unknown(self) -> None:
        """When ok=False and 'error' key missing, error message is 'Unknown error'."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value={"ok": False})

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import emit_event

            fn = emit_event.fn if hasattr(emit_event, "fn") else emit_event
            with pytest.raises(RuntimeError, match="Unknown error"):
                await fn(event="test_no_error_key")


# ---------------------------------------------------------------------------
# list_api_methods edge cases
# ---------------------------------------------------------------------------


class TestListApiMethodsEdgeCases:
    """Edge cases for list_api_methods not covered by test_api_tools.py."""

    @pytest.mark.asyncio
    async def test_evaluate_returns_string_returns_empty(self) -> None:
        """When evaluate returns a string, returns []."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value="echo,get_scene")

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_evaluate_returns_number_returns_empty(self) -> None:
        """When evaluate returns a number, returns []."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=0)

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            result = await fn()

        assert result == []

    @pytest.mark.asyncio
    async def test_empty_list_returns_empty(self) -> None:
        """When evaluate returns [], returns []."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.api.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.api import list_api_methods

            fn = list_api_methods.fn if hasattr(list_api_methods, "fn") else list_api_methods
            result = await fn()

        assert result == []


# ---------------------------------------------------------------------------
# get_page_info edge cases
# ---------------------------------------------------------------------------


class TestGetPageInfoEdgeCases:
    """Edge cases for get_page_info not covered by test_page_tools.py."""

    @pytest.mark.asyncio
    async def test_api_methods_is_dict_returns_empty(self) -> None:
        """When second evaluate returns a dict (not list), api_methods is []."""
        from auroraview_mcp.connection import Page

        page = Page(id="P1", url="http://localhost/", title="T", ws_url="ws://localhost/")
        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = page
        page_conn = MagicMock()
        # First call: ready=True, second call: non-list dict
        page_conn.evaluate = AsyncMock(side_effect=[True, {"methods": ["echo"]}])
        manager.get_page_connection = AsyncMock(return_value=page_conn)

        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            result = await fn()

        assert result["auroraview_ready"] is True
        assert result["api_methods"] == []

    @pytest.mark.asyncio
    async def test_api_methods_is_string_returns_empty(self) -> None:
        """When second evaluate returns a string, api_methods is []."""
        from auroraview_mcp.connection import Page

        page = Page(id="P2", url="http://localhost/", title="T2", ws_url="ws://localhost/")
        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = page
        page_conn = MagicMock()
        page_conn.evaluate = AsyncMock(side_effect=[True, "echo,get_scene"])
        manager.get_page_connection = AsyncMock(return_value=page_conn)

        with patch("auroraview_mcp.tools.page.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.page import get_page_info

            fn = get_page_info.fn if hasattr(get_page_info, "fn") else get_page_info
            result = await fn()

        assert result["api_methods"] == []
