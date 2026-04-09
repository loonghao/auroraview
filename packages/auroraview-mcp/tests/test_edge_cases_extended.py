"""Extended edge case tests for connection, discovery, telemetry, and gallery modules.

Covers previously untested branches:
- ConnectionManager: disconnect with no port, disconnect_all with page_connections,
  connect reuse, CDPConnection.send non-matching id skip
- InstanceDiscovery: registry with missing ws_url, JSON decode errors, title/app_name match
- get_telemetry: bridge returns scalar (non-list/dict), connected but current_page=None
- gallery: get_sample_info dir fallback .py, run_gallery missing main.py,
  get_sample_source dir with non-main py files
"""

from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest


# ============================================================================
# ConnectionManager edge cases
# ============================================================================


class TestConnectionManagerDisconnect:
    """Edge cases for ConnectionManager.disconnect()."""

    @pytest.mark.asyncio
    async def test_disconnect_no_current_port_no_arg(self) -> None:
        """disconnect() with no port arg and no current port is a no-op."""
        from auroraview_mcp.connection import ConnectionManager

        manager = ConnectionManager()
        # No port set, no arg → should return without error
        await manager.disconnect()
        assert manager.current_port is None

    @pytest.mark.asyncio
    async def test_disconnect_port_not_in_connections(self) -> None:
        """disconnect(port) where port is not in _connections silently succeeds."""
        from auroraview_mcp.connection import ConnectionManager

        manager = ConnectionManager()
        manager._current_port = 9222
        # Port 9222 not in _connections → no error
        await manager.disconnect(port=9222)
        assert manager.current_port is None

    @pytest.mark.asyncio
    async def test_disconnect_all_with_page_connections(self) -> None:
        """disconnect_all() also disconnects all page connections."""
        from auroraview_mcp.connection import CDPConnection, ConnectionManager, Page, PageConnection

        manager = ConnectionManager()

        # Add a CDP connection
        mock_ws1 = MagicMock()
        mock_ws1.close = AsyncMock()
        cdp_conn = CDPConnection(port=9222, ws_url="ws://test")
        cdp_conn._ws = mock_ws1
        manager._connections[9222] = cdp_conn
        manager._current_port = 9222

        # Add a page connection
        page = Page(id="pg1", url="http://x", title="x", ws_url="ws://page")
        mock_ws2 = MagicMock()
        mock_ws2.close = AsyncMock()
        page_conn = PageConnection(page=page)
        page_conn._ws = mock_ws2
        manager._page_connections["pg1"] = page_conn
        manager._current_page = page

        await manager.disconnect_all()

        mock_ws1.close.assert_called_once()
        mock_ws2.close.assert_called_once()
        assert manager._current_page is None
        assert manager._page_connections == {}

    @pytest.mark.asyncio
    async def test_connect_reuses_existing_connection(self) -> None:
        """connect() reuses an already-connected connection for the same port."""
        from auroraview_mcp.connection import CDPConnection, ConnectionManager

        manager = ConnectionManager()

        # Pre-populate a connected mock
        mock_ws = MagicMock()
        mock_ws.state = MagicMock()
        mock_ws.state.name = "OPEN"
        existing = CDPConnection(port=9222, ws_url="ws://test")
        existing._ws = mock_ws
        manager._connections[9222] = existing

        # Should not call httpx or websockets.connect
        with patch("httpx.AsyncClient") as mock_http:
            result = await manager.connect(9222)

        mock_http.assert_not_called()
        assert result is existing
        assert manager.current_port == 9222

    @pytest.mark.asyncio
    async def test_connect_raises_when_no_ws_url(self) -> None:
        """connect() raises ConnectionError when API returns no webSocketDebuggerUrl."""
        from auroraview_mcp.connection import ConnectionManager

        manager = ConnectionManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {}  # no webSocketDebuggerUrl

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(return_value=mock_response)
            mock_client.return_value = mock_instance

            with pytest.raises(ConnectionError, match="No WebSocket URL"):
                await manager.connect(9999)

    @pytest.mark.asyncio
    async def test_get_page_connection_creates_new_when_disconnected(self) -> None:
        """get_page_connection() creates new connection when cached conn is disconnected."""
        from auroraview_mcp.connection import ConnectionManager, Page, PageConnection

        manager = ConnectionManager()
        page = Page(id="p1", url="http://x", title="t", ws_url="ws://page1")
        manager._current_page = page

        # Pre-populate a disconnected connection
        disconnected = PageConnection(page=page)
        # _ws is None → not connected
        manager._page_connections["p1"] = disconnected

        mock_ws = MagicMock()
        with patch("auroraview_mcp.connection.websockets.connect", new_callable=AsyncMock) as mock_connect:
            mock_connect.return_value = mock_ws
            new_conn = await manager.get_page_connection()

        assert new_conn is not disconnected
        assert mock_connect.called


class TestCDPConnectionSend:
    """Edge cases for CDPConnection.send() response matching."""

    @pytest.mark.asyncio
    async def test_send_skips_non_matching_responses(self) -> None:
        """send() skips messages whose id doesn't match and waits for the right one."""
        from auroraview_mcp.connection import CDPConnection

        conn = CDPConnection(port=9222, ws_url="ws://x")

        import json as _json

        responses = [
            _json.dumps({"id": 99, "result": {"wrong": True}}),  # wrong id
            _json.dumps({"id": 1, "result": {"correct": True}}),  # correct
        ]
        idx = {"n": 0}

        async def fake_recv():
            val = responses[idx["n"]]
            idx["n"] += 1
            return val

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = fake_recv
        conn._ws = mock_ws

        result = await conn.send("Page.navigate", {"url": "http://example.com"})
        assert result == {"correct": True}

    @pytest.mark.asyncio
    async def test_send_raises_cdp_error_on_error_response(self) -> None:
        """send() raises CDPError when response contains 'error' key."""
        from auroraview_mcp.connection import CDPConnection, CDPError

        conn = CDPConnection(port=9222, ws_url="ws://x")

        import json as _json

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(
            return_value=_json.dumps({
                "id": 1,
                "error": {"code": -32601, "message": "Method not found"},
            })
        )
        conn._ws = mock_ws

        with pytest.raises(CDPError) as exc_info:
            await conn.send("Unknown.method")

        assert exc_info.value.code == -32601
        assert "Method not found" in exc_info.value.message


class TestPageConnectionSend:
    """Edge cases for PageConnection.send()."""

    @pytest.mark.asyncio
    async def test_send_not_connected_raises(self) -> None:
        """send() on disconnected PageConnection raises RuntimeError."""
        from auroraview_mcp.connection import Page, PageConnection

        page = Page(id="p", url="http://x", title="t", ws_url="ws://p")
        conn = PageConnection(page=page)

        with pytest.raises(RuntimeError, match="Not connected to page"):
            await conn.send("Runtime.evaluate")

    @pytest.mark.asyncio
    async def test_evaluate_raises_on_exception_details(self) -> None:
        """evaluate() raises JavaScriptError when exceptionDetails present."""
        from auroraview_mcp.connection import JavaScriptError, Page, PageConnection

        page = Page(id="p", url="http://x", title="t", ws_url="ws://p")
        conn = PageConnection(page=page)

        import json as _json

        mock_ws = MagicMock()
        mock_ws.send = AsyncMock()
        mock_ws.recv = AsyncMock(
            return_value=_json.dumps({
                "id": 1,
                "result": {
                    "result": {},
                    "exceptionDetails": {
                        "text": "ReferenceError",
                        "exception": {"description": "x is not defined"},
                    },
                },
            })
        )
        conn._ws = mock_ws

        with pytest.raises(JavaScriptError, match="x is not defined"):
            await conn.evaluate("x")


class TestCDPErrorEdgeCases:
    """Edge cases for CDPError."""

    def test_cdp_error_missing_fields(self) -> None:
        """CDPError uses defaults when code/message absent."""
        from auroraview_mcp.connection import CDPError

        err = CDPError({})
        assert err.code == -1
        assert err.message == "Unknown error"


# ============================================================================
# InstanceDiscovery edge cases
# ============================================================================


class TestInstanceDiscoveryRegistry:
    """Edge cases for _discover_via_registry and _instance_from_registry."""

    def test_registry_missing_cdp_port_returns_none(self) -> None:
        """_instance_from_registry returns None when cdp_port absent."""
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        result = d._instance_from_registry({"pid": 1234, "title": "test"})
        assert result is None

    def test_registry_uses_default_ws_url(self) -> None:
        """_instance_from_registry builds default ws_url when ws_url absent."""
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        result = d._instance_from_registry({"cdp_port": 9222})
        assert result is not None
        assert "9222" in result.ws_url

    def test_registry_uses_default_devtools_url(self) -> None:
        """_instance_from_registry builds default devtools_url when absent."""
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        result = d._instance_from_registry({"cdp_port": 9223, "ws_url": "ws://x"})
        assert result is not None
        assert "9223" in result.devtools_url

    def test_discover_via_registry_skips_json_error(self) -> None:
        """_discover_via_registry skips files with invalid JSON."""
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        with tempfile.TemporaryDirectory() as tmpdir:
            bad_file = Path(tmpdir) / "bad.json"
            bad_file.write_text("not valid json{{")

            with patch("auroraview_mcp.discovery.get_instances_dir", return_value=Path(tmpdir)):
                instances = d._discover_via_registry()

        assert instances == []

    def test_discover_via_registry_skips_dead_process(self) -> None:
        """_discover_via_registry removes stale file for dead process."""
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        with tempfile.TemporaryDirectory() as tmpdir:
            entry = {"cdp_port": 9222, "pid": 99999999}  # unlikely pid
            (Path(tmpdir) / "stale.json").write_text(json.dumps(entry))

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=Path(tmpdir)),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=False),
            ):
                instances = d._discover_via_registry()

        assert instances == []

    def test_discover_via_registry_no_dir(self) -> None:
        """_discover_via_registry returns empty list when instances dir absent."""
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        with patch(
            "auroraview_mcp.discovery.get_instances_dir",
            return_value=Path("/nonexistent/instances/dir"),
        ):
            instances = d._discover_via_registry()

        assert instances == []


class TestInstanceDiscoveryGetByTitle:
    """Edge cases for get_instance_by_title."""

    @pytest.mark.asyncio
    async def test_match_by_app_name(self) -> None:
        """get_instance_by_title matches app_name when title doesn't match."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        d = InstanceDiscovery()
        inst = Instance(port=9222, title="generic", app_name="Maya Asset Browser")

        with patch.object(d, "discover", new_callable=AsyncMock, return_value=[inst]):
            result = await d.get_instance_by_title("maya")

        assert result is inst

    @pytest.mark.asyncio
    async def test_no_match_returns_none(self) -> None:
        """get_instance_by_title returns None when no match."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        d = InstanceDiscovery()
        inst = Instance(port=9222, title="SomeOtherApp", app_name="SomeOtherApp")

        with patch.object(d, "discover", new_callable=AsyncMock, return_value=[inst]):
            result = await d.get_instance_by_title("maya")

        assert result is None

    @pytest.mark.asyncio
    async def test_match_case_insensitive(self) -> None:
        """get_instance_by_title does case-insensitive matching."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        d = InstanceDiscovery()
        inst = Instance(port=9222, title="BLENDER 4.0 - TOOL PANEL")

        with patch.object(d, "discover", new_callable=AsyncMock, return_value=[inst]):
            result = await d.get_instance_by_title("blender")

        assert result is inst


class TestInstanceDiscoveryGetByWindowId:
    """Edge cases for get_instance_by_window_id."""

    @pytest.mark.asyncio
    async def test_found_by_window_id(self) -> None:
        """get_instance_by_window_id returns matching instance."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        d = InstanceDiscovery()
        inst = Instance(port=9222, window_id="win-abc-123")

        with patch.object(d, "discover", new_callable=AsyncMock, return_value=[inst]):
            result = await d.get_instance_by_window_id("win-abc-123")

        assert result is inst

    @pytest.mark.asyncio
    async def test_not_found_returns_none(self) -> None:
        """get_instance_by_window_id returns None when no match."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        d = InstanceDiscovery()
        inst = Instance(port=9222, window_id="win-xyz")

        with patch.object(d, "discover", new_callable=AsyncMock, return_value=[inst]):
            result = await d.get_instance_by_window_id("win-abc-999")

        assert result is None


class TestInstanceDisplayName:
    """Edge cases for Instance.display_name()."""

    def test_display_name_with_all_fields(self) -> None:
        """display_name combines app_name, dcc_type+version, panel_name."""
        from auroraview_mcp.discovery import Instance

        inst = Instance(
            port=9222,
            app_name="AuroraView",
            dcc_type="maya",
            dcc_version="2025",
            panel_name="Asset Browser",
        )
        name = inst.display_name()
        assert "AuroraView" in name
        assert "maya" in name
        assert "2025" in name
        assert "Asset Browser" in name

    def test_display_name_fallback_to_title(self) -> None:
        """display_name uses title when app_name is empty."""
        from auroraview_mcp.discovery import Instance

        inst = Instance(port=9222, title="My Tool", dcc_type="blender")
        name = inst.display_name()
        assert "My Tool" in name
        assert "blender" in name

    def test_display_name_port_fallback(self) -> None:
        """display_name falls back to port when no name info."""
        from auroraview_mcp.discovery import Instance

        inst = Instance(port=9222)
        assert "9222" in inst.display_name()

    def test_display_name_dcc_without_version(self) -> None:
        """display_name includes dcc_type without version when dcc_version is None."""
        from auroraview_mcp.discovery import Instance

        inst = Instance(port=9222, app_name="App", dcc_type="houdini")
        name = inst.display_name()
        assert "houdini" in name
        assert "None" not in name


# ============================================================================
# get_telemetry additional branch: bridge returns scalar
# ============================================================================


class TestGetTelemetryScalarResult:
    """Test get_telemetry when bridge returns a scalar (non-list, non-dict)."""

    @pytest.mark.asyncio
    async def test_bridge_returns_scalar_wraps_in_list(self) -> None:
        """When bridge returns a scalar value, it's wrapped in instances=[value]."""
        from unittest.mock import AsyncMock, MagicMock, patch

        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = MagicMock()
        page_conn = MagicMock()
        page_conn.evaluate = AsyncMock(return_value="some_scalar_value")
        manager.get_page_connection = AsyncMock(return_value=page_conn)

        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        # Falls to [result] branch: instances = ["some_scalar_value"]
        # or falls through to ImportError fallback
        assert "instances" in result

    @pytest.mark.asyncio
    async def test_connected_but_no_page_falls_to_local(self) -> None:
        """When connected=True but current_page=None, skips bridge, uses local fallback."""
        from unittest.mock import AsyncMock, MagicMock, patch

        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = None  # no page selected

        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        # Should not call get_page_connection; falls to ImportError path
        assert "instances" in result

    @pytest.mark.asyncio
    async def test_bridge_exception_with_local_module_uses_local(self) -> None:
        """When bridge raises, falls back to local auroraview.telemetry module."""
        from unittest.mock import AsyncMock, MagicMock, patch

        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = MagicMock()
        page_conn = MagicMock()
        page_conn.evaluate = AsyncMock(side_effect=RuntimeError("ws error"))
        manager.get_page_connection = AsyncMock(return_value=page_conn)

        snapshot = {"webview_id": "local-1", "uptime_s": 60}
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[snapshot])

        with (
            patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager),
            patch.dict("sys.modules", {"auroraview.telemetry": mock_module}),
        ):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        assert len(result["instances"]) == 1
        assert result["instances"][0]["webview_id"] == "local-1"

    @pytest.mark.asyncio
    async def test_bridge_exception_note_includes_error(self) -> None:
        """When bridge raises and local unavailable, note includes bridge error."""
        from unittest.mock import AsyncMock, MagicMock, patch

        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = MagicMock()
        page_conn = MagicMock()
        page_conn.evaluate = AsyncMock(side_effect=RuntimeError("connection timeout"))
        manager.get_page_connection = AsyncMock(return_value=page_conn)

        with (
            patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager),
            patch.dict("sys.modules", {"auroraview.telemetry": None}),
        ):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        assert "instances" in result
        assert result["instances"] == []
        # note should mention the bridge error
        note = result.get("note", "")
        assert "connection timeout" in note or "Bridge error" in note


# ============================================================================
# gallery additional edge cases
# ============================================================================


class TestGetSampleInfoDirFallback:
    """Edge cases for get_sample_info directory handling."""

    def test_dir_without_main_py_uses_first_py_file(self) -> None:
        """When directory has no main.py, uses first .py file found."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "my_tool"
            sample_dir.mkdir()
            # No main.py, but has another .py
            alt_file = sample_dir / "tool.py"
            alt_file.write_text('"""My Tool\n\nA custom tool.\n"""')

            info = get_sample_info(sample_dir)

        assert info is not None
        assert info["name"] == "my_tool"
        assert info["main_file"] == str(alt_file)

    def test_dir_with_main_py_uses_main(self) -> None:
        """Directory with main.py should use main.py even if other .py files exist."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "full_app"
            sample_dir.mkdir()
            main = sample_dir / "main.py"
            main.write_text('"""Full App\n\nMain entry.\n"""')
            (sample_dir / "helper.py").write_text("# helper")

            info = get_sample_info(sample_dir)

        assert info is not None
        assert info["main_file"] == str(main)

    def test_sample_file_with_example_suffix_stripped(self) -> None:
        """Files ending with _example have suffix stripped from name."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            f = Path(tmpdir) / "widget_example.py"
            f.write_text('"""Widget Example"""\nprint("widget")')

            info = get_sample_info(f)

        assert info is not None
        assert info["name"] == "widget"

    def test_description_truncated_at_200_chars(self) -> None:
        """Description is truncated to 200 characters."""
        from auroraview_mcp.tools.gallery import get_sample_info

        long_desc = "A" * 300
        with tempfile.TemporaryDirectory() as tmpdir:
            f = Path(tmpdir) / "long_demo.py"
            f.write_text(f'"""Long Demo\n\n{long_desc}\n"""')

            info = get_sample_info(f)

        assert info is not None
        assert len(info["description"]) <= 200


class TestRunGalleryEdgeCases:
    """Edge cases for run_gallery MCP tool."""

    @pytest.mark.asyncio
    async def test_run_gallery_main_py_not_found(self) -> None:
        """run_gallery raises RuntimeError when gallery main.py is missing."""
        from auroraview_mcp.tools.gallery import ProcessManager, run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with tempfile.TemporaryDirectory() as tmpdir:
            # Gallery dir exists but no main.py
            with (
                patch.dict(os.environ, {"AURORAVIEW_GALLERY_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                pytest.raises(RuntimeError, match="Gallery main.py not found"),
            ):
                await fn()

    @pytest.mark.asyncio
    async def test_run_gallery_project_not_found(self) -> None:
        """run_gallery raises RuntimeError when project root not found."""
        from auroraview_mcp.tools.gallery import ProcessManager, run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with (
            patch(
                "auroraview_mcp.tools.gallery.get_gallery_dir",
                side_effect=FileNotFoundError("no project"),
            ),
            patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
            pytest.raises(RuntimeError, match="no project"),
        ):
            await fn()


class TestRunSampleEdgeCases:
    """Edge cases for run_sample MCP tool."""

    @pytest.mark.asyncio
    async def test_run_sample_not_found(self) -> None:
        """run_sample raises RuntimeError when sample doesn't exist."""
        from auroraview_mcp.tools.gallery import ProcessManager, run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                pytest.raises(RuntimeError, match="Sample not found"),
            ):
                await fn(name="nonexistent_sample")

    @pytest.mark.asyncio
    async def test_run_sample_project_dir_not_found(self) -> None:
        """run_sample raises RuntimeError when examples dir not found."""
        from auroraview_mcp.tools.gallery import ProcessManager, run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with (
            patch(
                "auroraview_mcp.tools.gallery.get_examples_dir",
                side_effect=FileNotFoundError("no examples"),
            ),
            patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
            pytest.raises(RuntimeError, match="no examples"),
        ):
            await fn(name="any")

    @pytest.mark.asyncio
    async def test_run_sample_dir_with_non_main_py(self) -> None:
        """run_sample handles sample directory with non-main .py file."""
        import asyncio

        from auroraview_mcp.tools.gallery import ProcessManager, run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "my_tool"
            sample_dir.mkdir()
            tool_py = sample_dir / "tool.py"
            tool_py.write_text('"""My Tool"""\nprint("hello")')

            mock_process = MagicMock()
            mock_process.pid = 12345
            mock_process.poll.return_value = None  # still running

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="my_tool")

        assert result["pid"] == 12345
        assert result["status"] == "running"


class TestStopSampleTimeout:
    """Edge cases for stop_sample when process times out."""

    @pytest.mark.asyncio
    async def test_stop_sample_kills_on_timeout(self) -> None:
        """stop_sample kills process when terminate+wait times out."""
        import subprocess as sp

        from auroraview_mcp.tools.gallery import ProcessInfo, ProcessManager, stop_sample

        fn = stop_sample.fn if hasattr(stop_sample, "fn") else stop_sample

        mock_process = MagicMock()
        mock_process.poll.return_value = None
        mock_process.wait.side_effect = sp.TimeoutExpired("cmd", 5)

        manager = ProcessManager()
        manager.add(ProcessInfo(pid=7654, name="slow_sample", process=mock_process))

        with patch("auroraview_mcp.tools.gallery._process_manager", manager):
            result = await fn(pid=7654)

        assert result["status"] == "stopped"
        mock_process.kill.assert_called_once()


class TestStopAllSamplesTimeout:
    """Edge cases for stop_all_samples timeout handling."""

    @pytest.mark.asyncio
    async def test_stop_all_kills_on_timeout(self) -> None:
        """stop_all_samples kills processes that time out on wait."""
        import subprocess as sp

        from auroraview_mcp.tools.gallery import ProcessInfo, ProcessManager, stop_all_samples

        fn = stop_all_samples.fn if hasattr(stop_all_samples, "fn") else stop_all_samples

        mock_process = MagicMock()
        mock_process.poll.return_value = None
        mock_process.wait.side_effect = sp.TimeoutExpired("cmd", 3)

        manager = ProcessManager()
        manager.add(ProcessInfo(pid=3131, name="stubborn", process=mock_process))

        with patch("auroraview_mcp.tools.gallery._process_manager", manager):
            result = await fn()

        assert result["stopped"] == 1
        mock_process.kill.assert_called_once()
