"""Tests for DCC tools.

Tests for Digital Content Creation (DCC) application integration tools.
"""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest


class TestDCCToolRegistration:
    """Tests for DCC tool registration."""

    def test_dcc_tools_registered(self) -> None:
        """Test that DCC tools are registered."""
        from auroraview_mcp.tools import dcc

        assert hasattr(dcc, "get_dcc_context")
        assert hasattr(dcc, "execute_dcc_command")
        assert hasattr(dcc, "sync_selection")
        assert hasattr(dcc, "set_dcc_selection")
        assert hasattr(dcc, "get_dcc_scene_info")
        assert hasattr(dcc, "get_dcc_timeline")
        assert hasattr(dcc, "set_dcc_frame")


class TestGetDCCContext:
    """Tests for get_dcc_context function."""

    @pytest.mark.asyncio
    async def test_get_dcc_context_with_bridge(self) -> None:
        """Test getting DCC context when bridge is available."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "dcc_type": "maya",
                "dcc_version": "2025.1",
                "scene_path": "/projects/scene.ma",
                "scene_name": "scene.ma",
                "selected_objects": ["pCube1", "pSphere1"],
                "current_frame": 1,
                "frame_range": {"start": 1, "end": 100},
                "project_path": "/projects",
                "fps": 24,
                "units": "cm",
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import get_dcc_context

            # Get the underlying function from the FunctionTool
            fn = get_dcc_context.fn if hasattr(get_dcc_context, "fn") else get_dcc_context
            result = await fn()

            assert result["dcc_type"] == "maya"
            assert result["dcc_version"] == "2025.1"
            assert result["scene_path"] == "/projects/scene.ma"
            assert len(result["selected_objects"]) == 2

    @pytest.mark.asyncio
    async def test_get_dcc_context_bridge_not_available(self) -> None:
        """Test getting DCC context when bridge is not available."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "error": "AuroraView bridge not available",
                "dcc_type": None,
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import get_dcc_context

            fn = get_dcc_context.fn if hasattr(get_dcc_context, "fn") else get_dcc_context
            result = await fn()

            assert result["dcc_type"] is None
            assert "error" in result

    @pytest.mark.asyncio
    async def test_get_dcc_context_fallback_detection(self) -> None:
        """Test DCC type detection from page title/URL."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "dcc_type": "blender",
                "dcc_version": None,
                "scene_path": None,
                "selected_objects": [],
                "error": "API not implemented",
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import get_dcc_context

            fn = get_dcc_context.fn if hasattr(get_dcc_context, "fn") else get_dcc_context
            result = await fn()

            assert result["dcc_type"] == "blender"


class TestExecuteDCCCommand:
    """Tests for execute_dcc_command function."""

    @pytest.mark.asyncio
    async def test_execute_command_success(self) -> None:
        """Test successful command execution."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "success": True,
                "result": ["pCube1", "pSphere1"],
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import execute_dcc_command

            fn = (
                execute_dcc_command.fn
                if hasattr(execute_dcc_command, "fn")
                else execute_dcc_command
            )
            result = await fn(command="maya.cmds.ls", kwargs={"selection": True})

            assert result["success"] is True
            assert result["result"] == ["pCube1", "pSphere1"]

    @pytest.mark.asyncio
    async def test_execute_command_with_args(self) -> None:
        """Test command execution with positional arguments."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "success": True,
                "result": None,
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import execute_dcc_command

            fn = (
                execute_dcc_command.fn
                if hasattr(execute_dcc_command, "fn")
                else execute_dcc_command
            )
            result = await fn(command="maya.cmds.select", args=["pCube1"])

            assert result["success"] is True

    @pytest.mark.asyncio
    async def test_execute_command_failure(self) -> None:
        """Test command execution failure."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "success": False,
                "error": "Command not found: invalid.command",
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import execute_dcc_command

            fn = (
                execute_dcc_command.fn
                if hasattr(execute_dcc_command, "fn")
                else execute_dcc_command
            )
            result = await fn(command="invalid.command")

            assert result["success"] is False
            assert "error" in result


class TestSyncSelection:
    """Tests for sync_selection function."""

    @pytest.mark.asyncio
    async def test_sync_selection_success(self) -> None:
        """Test successful selection sync."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "dcc_selection": ["pCube1", "pSphere1"],
                "webview_selection": ["pCube1", "pSphere1"],
                "synced": True,
                "dcc_type": "maya",
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import sync_selection

            fn = sync_selection.fn if hasattr(sync_selection, "fn") else sync_selection
            result = await fn()

            assert result["synced"] is True
            assert result["dcc_selection"] == ["pCube1", "pSphere1"]
            assert result["webview_selection"] == ["pCube1", "pSphere1"]

    @pytest.mark.asyncio
    async def test_sync_selection_not_synced(self) -> None:
        """Test selection sync when not in sync."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "dcc_selection": ["pCube1"],
                "webview_selection": ["pSphere1"],
                "synced": False,
                "dcc_type": "houdini",
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import sync_selection

            fn = sync_selection.fn if hasattr(sync_selection, "fn") else sync_selection
            result = await fn()

            assert result["synced"] is False
            assert result["dcc_selection"] != result["webview_selection"]


class TestSetDCCSelection:
    """Tests for set_dcc_selection function."""

    @pytest.mark.asyncio
    async def test_set_selection_success(self) -> None:
        """Test successful selection set."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "success": True,
                "selected": ["pCube1", "pCube2"],
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import set_dcc_selection

            fn = set_dcc_selection.fn if hasattr(set_dcc_selection, "fn") else set_dcc_selection
            result = await fn(objects=["pCube1", "pCube2"])

            assert result["success"] is True
            assert result["selected"] == ["pCube1", "pCube2"]

    @pytest.mark.asyncio
    async def test_set_selection_empty(self) -> None:
        """Test setting empty selection (clear selection)."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "success": True,
                "selected": [],
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import set_dcc_selection

            fn = set_dcc_selection.fn if hasattr(set_dcc_selection, "fn") else set_dcc_selection
            result = await fn(objects=[])

            assert result["success"] is True
            assert result["selected"] == []


class TestGetDCCSceneInfo:
    """Tests for get_dcc_scene_info function."""

    @pytest.mark.asyncio
    async def test_get_scene_info_success(self) -> None:
        """Test successful scene info retrieval."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "scene_path": "/projects/my_scene.ma",
                "scene_name": "my_scene.ma",
                "modified": False,
                "objects_count": 150,
                "selected_count": 3,
                "cameras": ["persp", "front", "side", "top"],
                "lights": ["directionalLight1", "pointLight1"],
                "materials": ["lambert1", "blinn1"],
                "render_settings": {"renderer": "arnold", "resolution": [1920, 1080]},
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import get_dcc_scene_info

            fn = get_dcc_scene_info.fn if hasattr(get_dcc_scene_info, "fn") else get_dcc_scene_info
            result = await fn()

            assert result["scene_name"] == "my_scene.ma"
            assert result["objects_count"] == 150
            assert len(result["cameras"]) == 4


class TestGetDCCTimeline:
    """Tests for get_dcc_timeline function."""

    @pytest.mark.asyncio
    async def test_get_timeline_success(self) -> None:
        """Test successful timeline info retrieval."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "current_frame": 24,
                "start_frame": 1,
                "end_frame": 120,
                "fps": 24,
                "playing": False,
                "time_unit": "frames",
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import get_dcc_timeline

            fn = get_dcc_timeline.fn if hasattr(get_dcc_timeline, "fn") else get_dcc_timeline
            result = await fn()

            assert result["current_frame"] == 24
            assert result["fps"] == 24
            assert result["end_frame"] == 120


class TestSetDCCFrame:
    """Tests for set_dcc_frame function."""

    @pytest.mark.asyncio
    async def test_set_frame_success(self) -> None:
        """Test successful frame set."""
        mock_page_conn = MagicMock()
        mock_page_conn.evaluate = AsyncMock(
            return_value={
                "success": True,
                "frame": 50,
            }
        )

        mock_manager = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_page_conn)

        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=mock_manager):
            from auroraview_mcp.tools.dcc import set_dcc_frame

            fn = set_dcc_frame.fn if hasattr(set_dcc_frame, "fn") else set_dcc_frame
            result = await fn(frame=50)

            assert result["success"] is True
            assert result["frame"] == 50


class TestDCCTypeDetection:
    """Tests for DCC type detection in discovery module."""

    def test_detect_maya(self) -> None:
        """Test Maya detection."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("Maya 2025 - Asset Browser", "") == "maya"
        assert discovery._detect_dcc_type("Autodesk Maya Panel", "") == "maya"

    def test_detect_blender(self) -> None:
        """Test Blender detection."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("Blender 4.0 - Tool Panel", "") == "blender"

    def test_detect_houdini(self) -> None:
        """Test Houdini detection."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("Houdini 20 - Asset Manager", "") == "houdini"
        assert discovery._detect_dcc_type("SideFX Panel", "") == "houdini"

    def test_detect_nuke(self) -> None:
        """Test Nuke detection."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("Nuke 14 - Custom Panel", "") == "nuke"
        assert discovery._detect_dcc_type("Foundry Nuke", "") == "nuke"

    def test_detect_unreal(self) -> None:
        """Test Unreal detection."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("Unreal Editor - Widget", "") == "unreal"
        assert discovery._detect_dcc_type("UE5 Panel", "") == "unreal"

    def test_detect_3dsmax(self) -> None:
        """Test 3ds Max detection."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("3ds Max 2025", "") == "3dsmax"
        assert discovery._detect_dcc_type("3dsmax Panel", "") == "3dsmax"

    def test_detect_unknown(self) -> None:
        """Test unknown DCC type."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        assert discovery._detect_dcc_type("Generic WebView", "") is None
        assert discovery._detect_dcc_type("AuroraView Gallery", "") is None


class TestListDCCInstances:
    """Tests for list_dcc_instances tool."""

    @pytest.mark.asyncio
    async def test_list_dcc_instances_empty(self) -> None:
        """Test listing DCC instances when none found."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        with patch.object(discovery, "discover", new_callable=AsyncMock) as mock_discover:
            mock_discover.return_value = []

            instances = await discovery.discover_dcc_instances([9999])
            assert instances == []

    @pytest.mark.asyncio
    async def test_list_dcc_instances_with_context(self) -> None:
        """Test listing DCC instances with context enrichment."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        discovery = InstanceDiscovery()

        mock_instance = Instance(
            port=9222,
            browser="Chrome/120.0",
            ws_url="ws://localhost:9222/devtools/browser/xxx",
        )

        with (
            patch.object(discovery, "discover", new_callable=AsyncMock) as mock_discover,
            patch.object(discovery, "_enrich_dcc_context", new_callable=AsyncMock) as mock_enrich,
        ):
            mock_discover.return_value = [mock_instance]

            enriched_instance = Instance(
                port=9222,
                browser="Chrome/120.0",
                ws_url="ws://localhost:9222/devtools/browser/xxx",
                dcc_type="maya",
                title="Maya 2025 - Asset Browser",
                url="file:///asset_browser.html",
            )
            mock_enrich.return_value = enriched_instance

            instances = await discovery.discover_dcc_instances([9222])

            assert len(instances) == 1
            assert instances[0].dcc_type == "maya"
