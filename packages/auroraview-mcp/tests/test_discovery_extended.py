"""Extended tests for the discovery module.

Covers Instance dataclass, InstanceDiscovery internal methods, registry-based
discovery, CDP probing, DCC-context enrichment, and helper functions.
"""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.discovery import Instance, InstanceDiscovery, get_instances_dir


class TestGetInstancesDir:
    """Tests for get_instances_dir helper."""

    def test_windows_path(self) -> None:
        """Test instances dir on Windows."""
        with (
            patch.object(sys, "platform", "win32"),
            patch.dict("os.environ", {"LOCALAPPDATA": "C:\\Users\\test\\AppData\\Local"}),
        ):
            result = get_instances_dir()
        assert "AuroraView" in str(result)
        assert "instances" in str(result)

    def test_darwin_path(self) -> None:
        """Test instances dir on macOS."""
        with patch.object(sys, "platform", "darwin"):
            result = get_instances_dir()
        assert "AuroraView" in str(result)
        assert "instances" in str(result)

    def test_linux_path(self) -> None:
        """Test instances dir on Linux."""
        with (
            patch.object(sys, "platform", "linux"),
            patch.dict("os.environ", {"XDG_DATA_HOME": "/home/test/.local/share"}),
        ):
            result = get_instances_dir()
        assert "auroraview" in str(result).lower()
        assert "instances" in str(result)


class TestInstance:
    """Tests for Instance dataclass."""

    def test_instance_defaults(self) -> None:
        """Test Instance has correct defaults."""
        inst = Instance(port=9222)
        assert inst.port == 9222
        assert inst.browser == ""
        assert inst.ws_url == ""
        assert inst.user_agent == ""
        assert inst.protocol_version == ""
        assert inst.pid is None
        assert inst.title == ""
        assert inst.url == ""
        assert inst.dcc_type is None
        assert inst.dcc_version is None
        assert inst.panel_name is None
        assert inst.window_id is None
        assert inst.app_name is None
        assert inst.app_version is None
        assert inst.dcc_python_version is None
        assert inst.dock_area is None
        assert inst.start_time is None
        assert inst.devtools_url is None
        assert inst.html_title is None
        assert inst.is_loading is False
        assert inst.metadata == {}

    def test_to_dict_full(self) -> None:
        """Test Instance.to_dict() with all fields."""
        inst = Instance(
            port=9222,
            browser="Chrome/120",
            ws_url="ws://localhost:9222",
            user_agent="Mozilla/5.0",
            protocol_version="1.3",
            pid=1234,
            title="Maya Panel",
            url="file:///panel.html",
            dcc_type="maya",
            dcc_version="2025",
            panel_name="Asset Browser",
            window_id="wnd-001",
            app_name="AuroraView",
            app_version="1.0.0",
            dcc_python_version="3.11",
            dock_area="right",
            start_time=1234567890.0,
            devtools_url="devtools://devtools/...",
            html_title="Asset Browser",
            is_loading=False,
            metadata={"custom": "value"},
        )

        d = inst.to_dict()
        assert d["port"] == 9222
        assert d["browser"] == "Chrome/120"
        assert d["ws_url"] == "ws://localhost:9222"
        assert d["pid"] == 1234
        assert d["dcc_type"] == "maya"
        assert d["dcc_version"] == "2025"
        assert d["panel_name"] == "Asset Browser"
        assert d["window_id"] == "wnd-001"
        assert d["app_name"] == "AuroraView"
        assert d["dock_area"] == "right"
        assert d["start_time"] == 1234567890.0
        assert d["is_loading"] is False
        assert d["metadata"] == {"custom": "value"}

    def test_display_name_with_app_and_dcc(self) -> None:
        """Test display_name with app_name and dcc_type."""
        inst = Instance(port=9222, app_name="Asset Browser", dcc_type="maya", dcc_version="2025")
        name = inst.display_name()
        assert "Asset Browser" in name
        assert "maya" in name
        assert "2025" in name

    def test_display_name_with_title_only(self) -> None:
        """Test display_name with only title."""
        inst = Instance(port=9222, title="My Panel")
        name = inst.display_name()
        assert "My Panel" in name

    def test_display_name_with_panel_name(self) -> None:
        """Test display_name includes panel name."""
        inst = Instance(port=9222, app_name="AuroraView", panel_name="Tool Panel")
        name = inst.display_name()
        assert "Tool Panel" in name

    def test_display_name_fallback_to_port(self) -> None:
        """Test display_name fallback when no name info."""
        inst = Instance(port=9222)
        name = inst.display_name()
        assert "9222" in name

    def test_display_name_dcc_without_version(self) -> None:
        """Test display_name with dcc_type but no dcc_version."""
        inst = Instance(port=9222, app_name="Panel", dcc_type="blender")
        name = inst.display_name()
        assert "blender" in name
        assert "[blender]" in name


class TestInstanceDiscoveryRegistry:
    """Tests for file-based registry discovery."""

    def test_discover_via_registry_empty_dir(self) -> None:
        """Test _discover_via_registry when instances dir doesn't exist."""
        discovery = InstanceDiscovery()

        with patch("auroraview_mcp.discovery.get_instances_dir") as mock_dir:
            mock_dir.return_value = Path("/nonexistent/path/12345")
            result = discovery._discover_via_registry()

        assert result == []

    def test_discover_via_registry_with_valid_file(self) -> None:
        """Test _discover_via_registry reads valid instance files."""
        discovery = InstanceDiscovery()

        with tempfile.TemporaryDirectory() as tmpdir:
            instances_dir = Path(tmpdir)

            # Create valid instance file
            instance_data = {
                "cdp_port": 9222,
                "pid": None,
                "ws_url": "ws://127.0.0.1:9222/devtools/page/1",
                "title": "Test Panel",
                "url": "file:///test.html",
                "dcc_type": "maya",
                "dcc_version": "2025",
                "app_name": "AuroraView",
            }
            (instances_dir / "test_instance.json").write_text(
                json.dumps(instance_data), encoding="utf-8"
            )

            with patch("auroraview_mcp.discovery.get_instances_dir", return_value=instances_dir):
                result = discovery._discover_via_registry()

        assert len(result) == 1
        assert result[0].port == 9222
        assert result[0].dcc_type == "maya"
        assert result[0].title == "Test Panel"

    def test_discover_via_registry_skips_invalid_json(self) -> None:
        """Test _discover_via_registry skips malformed JSON files."""
        discovery = InstanceDiscovery()

        with tempfile.TemporaryDirectory() as tmpdir:
            instances_dir = Path(tmpdir)
            (instances_dir / "bad.json").write_text("not valid json", encoding="utf-8")

            with patch("auroraview_mcp.discovery.get_instances_dir", return_value=instances_dir):
                result = discovery._discover_via_registry()

        assert result == []

    def test_discover_via_registry_removes_stale_entry(self) -> None:
        """Test _discover_via_registry removes files for dead processes."""
        discovery = InstanceDiscovery()

        with tempfile.TemporaryDirectory() as tmpdir:
            instances_dir = Path(tmpdir)

            instance_data = {
                "cdp_port": 9222,
                "pid": 99999,  # Non-existent PID
            }
            stale_file = instances_dir / "stale.json"
            stale_file.write_text(json.dumps(instance_data), encoding="utf-8")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=instances_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=False),
            ):
                result = discovery._discover_via_registry()

        assert result == []
        # Stale file should be removed
        assert not stale_file.exists()

    def test_discover_via_registry_keeps_alive_process(self) -> None:
        """Test _discover_via_registry keeps entries for alive processes."""
        discovery = InstanceDiscovery()

        with tempfile.TemporaryDirectory() as tmpdir:
            instances_dir = Path(tmpdir)

            instance_data = {
                "cdp_port": 9222,
                "pid": 1234,
                "app_name": "AuroraView",
            }
            (instances_dir / "alive.json").write_text(json.dumps(instance_data), encoding="utf-8")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=instances_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                result = discovery._discover_via_registry()

        assert len(result) == 1
        assert result[0].port == 9222

    def test_instance_from_registry_no_cdp_port(self) -> None:
        """Test _instance_from_registry returns None when no cdp_port."""
        discovery = InstanceDiscovery()
        result = discovery._instance_from_registry({"title": "No Port"})
        assert result is None

    def test_instance_from_registry_full_data(self) -> None:
        """Test _instance_from_registry creates Instance with all fields."""
        discovery = InstanceDiscovery()
        data = {
            "cdp_port": 9223,
            "ws_url": "ws://127.0.0.1:9223/devtools/page/1",
            "title": "Houdini Panel",
            "url": "file:///panel.html",
            "pid": 5678,
            "dcc_type": "houdini",
            "dcc_version": "20.5",
            "panel_name": "Network Editor",
            "window_id": "wnd-002",
            "app_name": "AuroraView",
            "app_version": "1.0.0",
            "dock_area": "bottom",
            "start_time": 1700000000.0,
            "html_title": "Network Editor",
            "is_loading": False,
            "metadata": {"scene": "scene.hip"},
        }
        inst = discovery._instance_from_registry(data)
        assert inst is not None
        assert inst.port == 9223
        assert inst.dcc_type == "houdini"
        assert inst.panel_name == "Network Editor"
        assert inst.dock_area == "bottom"
        assert inst.metadata == {"scene": "scene.hip"}


class TestInstanceDiscoveryCDPProbe:
    """Tests for CDP port probing."""

    @pytest.mark.asyncio
    async def test_probe_port_webview2(self) -> None:
        """Test _probe_port returns Instance for WebView2 endpoint."""
        discovery = InstanceDiscovery()

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "Browser": "Edg/120.0.0.0",
            "webSocketDebuggerUrl": "ws://localhost:9222/devtools/browser/xxx",
            "User-Agent": "Mozilla/5.0",
            "Protocol-Version": "1.3",
        }

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._probe_port(9222)

        assert result is not None
        assert result.port == 9222
        assert "Edg" in result.browser

    @pytest.mark.asyncio
    async def test_probe_port_chrome_webview(self) -> None:
        """Test _probe_port returns Instance for Chrome WebView."""
        discovery = InstanceDiscovery()

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "Browser": "Chrome/119.0.6045.199",
            "webSocketDebuggerUrl": "ws://localhost:9223/devtools/browser/yyy",
        }

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._probe_port(9223)

        assert result is not None
        assert result.port == 9223

    @pytest.mark.asyncio
    async def test_probe_port_non_webview(self) -> None:
        """Test _probe_port returns None for non-WebView endpoints."""
        discovery = InstanceDiscovery()

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "Browser": "Firefox/120.0",
        }

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._probe_port(9224)

        assert result is None

    @pytest.mark.asyncio
    async def test_probe_port_connection_refused(self) -> None:
        """Test _probe_port returns None when connection refused."""
        import httpx

        discovery = InstanceDiscovery()

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(side_effect=httpx.RequestError("Connection refused"))

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._probe_port(9999)

        assert result is None

    @pytest.mark.asyncio
    async def test_probe_port_non_200_status(self) -> None:
        """Test _probe_port returns None for non-200 status."""
        discovery = InstanceDiscovery()

        mock_response = MagicMock()
        mock_response.status_code = 404

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._probe_port(9998)

        assert result is None

    def test_is_webview_edge(self) -> None:
        """Test _is_webview detects Edge (WebView2)."""
        discovery = InstanceDiscovery()
        assert discovery._is_webview({"Browser": "Edg/120.0"}) is True

    def test_is_webview_chrome(self) -> None:
        """Test _is_webview detects Chrome."""
        discovery = InstanceDiscovery()
        assert discovery._is_webview({"Browser": "Chrome/119.0"}) is True

    def test_is_webview_firefox(self) -> None:
        """Test _is_webview rejects Firefox."""
        discovery = InstanceDiscovery()
        assert discovery._is_webview({"Browser": "Firefox/120.0"}) is False

    def test_is_webview_empty(self) -> None:
        """Test _is_webview with empty browser string."""
        discovery = InstanceDiscovery()
        assert discovery._is_webview({}) is False


class TestInstanceDiscoveryVerify:
    """Tests for CDP instance verification."""

    @pytest.mark.asyncio
    async def test_verify_instance_reachable(self) -> None:
        """Test _verify_instance returns True for reachable instance."""
        discovery = InstanceDiscovery()
        inst = Instance(port=9222)

        mock_response = MagicMock()
        mock_response.status_code = 200

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._verify_instance(inst)

        assert result is True

    @pytest.mark.asyncio
    async def test_verify_instance_unreachable(self) -> None:
        """Test _verify_instance returns False for unreachable instance."""
        discovery = InstanceDiscovery()
        inst = Instance(port=9999)

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(side_effect=Exception("Connection refused"))

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._verify_instance(inst)

        assert result is False

    @pytest.mark.asyncio
    async def test_discover_with_verify_cdp_filters_unreachable(self) -> None:
        """Test discover() filters out unreachable instances when verify_cdp=True."""
        discovery = InstanceDiscovery(verify_cdp=True)

        inst_reachable = Instance(port=9222)
        inst_unreachable = Instance(port=9223)

        async def fake_verify(instance: Instance) -> bool:
            return instance.port == 9222

        with (
            patch.object(
                discovery, "_discover_via_registry", return_value=[inst_reachable, inst_unreachable]
            ),
            patch.object(discovery, "_verify_instance", side_effect=fake_verify),
            patch.object(discovery, "_probe_port", new_callable=AsyncMock, return_value=None),
        ):
            results = await discovery.discover(ports=[])

        assert len(results) == 1
        assert results[0].port == 9222


class TestInstanceDiscoveryGetters:
    """Tests for get_instance_by_* methods."""

    @pytest.mark.asyncio
    async def test_get_instance_by_window_id_found(self) -> None:
        """Test get_instance_by_window_id returns matching instance."""
        discovery = InstanceDiscovery()
        instances = [
            Instance(port=9222, window_id="wnd-001"),
            Instance(port=9223, window_id="wnd-002"),
        ]

        with patch.object(discovery, "discover", new_callable=AsyncMock, return_value=instances):
            result = await discovery.get_instance_by_window_id("wnd-002")

        assert result is not None
        assert result.port == 9223

    @pytest.mark.asyncio
    async def test_get_instance_by_window_id_not_found(self) -> None:
        """Test get_instance_by_window_id returns None when not found."""
        discovery = InstanceDiscovery()
        instances = [Instance(port=9222, window_id="wnd-001")]

        with patch.object(discovery, "discover", new_callable=AsyncMock, return_value=instances):
            result = await discovery.get_instance_by_window_id("wnd-999")

        assert result is None

    @pytest.mark.asyncio
    async def test_get_instance_by_title_found(self) -> None:
        """Test get_instance_by_title returns matching instance."""
        discovery = InstanceDiscovery()
        instances = [
            Instance(port=9222, title="Maya Asset Browser"),
            Instance(port=9223, title="Houdini Network Editor"),
        ]

        with patch.object(discovery, "discover", new_callable=AsyncMock, return_value=instances):
            result = await discovery.get_instance_by_title("houdini")

        assert result is not None
        assert result.port == 9223

    @pytest.mark.asyncio
    async def test_get_instance_by_title_via_app_name(self) -> None:
        """Test get_instance_by_title searches app_name too."""
        discovery = InstanceDiscovery()
        instances = [
            Instance(port=9222, title="Panel", app_name="AuroraView Tool"),
        ]

        with patch.object(discovery, "discover", new_callable=AsyncMock, return_value=instances):
            result = await discovery.get_instance_by_title("aurora")

        assert result is not None
        assert result.port == 9222

    @pytest.mark.asyncio
    async def test_get_instance_by_title_not_found(self) -> None:
        """Test get_instance_by_title returns None when no match."""
        discovery = InstanceDiscovery()
        instances = [Instance(port=9222, title="Maya Panel")]

        with patch.object(discovery, "discover", new_callable=AsyncMock, return_value=instances):
            result = await discovery.get_instance_by_title("blender")

        assert result is None

    @pytest.mark.asyncio
    async def test_get_instance_by_dcc_found(self) -> None:
        """Test get_instance_by_dcc returns matching instance."""
        discovery = InstanceDiscovery()
        instances = [
            Instance(port=9222, dcc_type="maya"),
            Instance(port=9223, dcc_type="blender"),
        ]

        with patch.object(
            discovery,
            "discover_dcc_instances",
            new_callable=AsyncMock,
            return_value=instances,
        ):
            result = await discovery.get_instance_by_dcc("maya")

        assert result is not None
        assert result.port == 9222

    @pytest.mark.asyncio
    async def test_get_instance_by_dcc_case_insensitive(self) -> None:
        """Test get_instance_by_dcc is case-insensitive."""
        discovery = InstanceDiscovery()
        instances = [Instance(port=9222, dcc_type="Maya")]

        with patch.object(
            discovery,
            "discover_dcc_instances",
            new_callable=AsyncMock,
            return_value=instances,
        ):
            result = await discovery.get_instance_by_dcc("maya")

        assert result is not None

    @pytest.mark.asyncio
    async def test_get_instance_by_dcc_not_found(self) -> None:
        """Test get_instance_by_dcc returns None when no match."""
        discovery = InstanceDiscovery()
        instances = [Instance(port=9222, dcc_type="maya")]

        with patch.object(
            discovery,
            "discover_dcc_instances",
            new_callable=AsyncMock,
            return_value=instances,
        ):
            result = await discovery.get_instance_by_dcc("houdini")

        assert result is None


class TestInstanceDiscoveryEnrich:
    """Tests for _enrich_dcc_context method."""

    @pytest.mark.asyncio
    async def test_enrich_detects_dcc_from_page_title(self) -> None:
        """Test _enrich_dcc_context sets dcc_type from page title."""
        discovery = InstanceDiscovery()
        inst = Instance(port=9222)

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = [
            {
                "title": "Blender 4.0 - Properties",
                "url": "file:///panel.html",
            }
        ]

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(return_value=mock_response)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._enrich_dcc_context(inst)

        assert result.dcc_type == "blender"
        assert result.title == "Blender 4.0 - Properties"

    @pytest.mark.asyncio
    async def test_enrich_handles_exception_gracefully(self) -> None:
        """Test _enrich_dcc_context returns original instance on error."""
        discovery = InstanceDiscovery()
        inst = Instance(port=9222)

        mock_client = MagicMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=None)
        mock_client.get = AsyncMock(side_effect=Exception("timeout"))

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._enrich_dcc_context(inst)

        # Original instance returned unchanged
        assert result is inst
        assert result.dcc_type is None

    @pytest.mark.asyncio
    async def test_discover_dcc_instances_skips_enrich_if_dcc_known(self) -> None:
        """Test discover_dcc_instances skips enrichment for known DCC type."""
        discovery = InstanceDiscovery()
        instances = [Instance(port=9222, dcc_type="maya")]

        with (
            patch.object(discovery, "discover", new_callable=AsyncMock, return_value=instances),
            patch.object(discovery, "_enrich_dcc_context", new_callable=AsyncMock) as mock_enrich,
        ):
            result = await discovery.discover_dcc_instances()

        # enrich should NOT be called for instance with known dcc_type
        mock_enrich.assert_not_called()
        assert result[0].dcc_type == "maya"

    @pytest.mark.asyncio
    async def test_discover_dcc_instances_enriches_unknown(self) -> None:
        """Test discover_dcc_instances enriches instances with unknown DCC type."""
        discovery = InstanceDiscovery()
        inst = Instance(port=9222)  # No dcc_type
        enriched = Instance(port=9222, dcc_type="nuke", title="Nuke Panel")

        with (
            patch.object(discovery, "discover", new_callable=AsyncMock, return_value=[inst]),
            patch.object(
                discovery,
                "_enrich_dcc_context",
                new_callable=AsyncMock,
                return_value=enriched,
            ),
        ):
            result = await discovery.discover_dcc_instances()

        assert result[0].dcc_type == "nuke"


class TestDetectDCCType:
    """Tests for _detect_dcc_type method."""

    def test_all_dcc_types_detected(self) -> None:
        """Test all known DCC types are detected."""
        discovery = InstanceDiscovery()

        test_cases = [
            ("Maya 2025 Panel", "", "maya"),
            ("Autodesk Maya", "", "maya"),
            ("Blender Properties", "", "blender"),
            ("SideFX Houdini", "", "houdini"),
            ("Houdini 20.5", "", "houdini"),
            ("Nuke 14 Custom Panel", "", "nuke"),
            ("Foundry Nuke Studio", "", "nuke"),
            ("Unreal Editor", "", "unreal"),
            ("UE5 Widget", "", "unreal"),
            ("3ds Max 2025", "", "3dsmax"),
            ("3dsmax Panel", "", "3dsmax"),
        ]

        for title, url, expected in test_cases:
            result = discovery._detect_dcc_type(title, url)
            assert result == expected, f"Expected {expected} for title='{title}'"

    def test_detect_from_url(self) -> None:
        """Test DCC detection from URL."""
        discovery = InstanceDiscovery()
        assert discovery._detect_dcc_type("Panel", "http://localhost/maya/tool") == "maya"
        assert discovery._detect_dcc_type("Panel", "file:///blender/panel.html") == "blender"

    def test_unknown_returns_none(self) -> None:
        """Test unknown DCC type returns None."""
        discovery = InstanceDiscovery()
        assert discovery._detect_dcc_type("Generic App", "http://localhost") is None
        assert discovery._detect_dcc_type("", "") is None
