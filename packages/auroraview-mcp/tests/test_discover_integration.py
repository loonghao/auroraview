"""Integration tests for discover() combining registry + fallback CDP probe.

Covers:
- discover() with only registry hits
- discover() with only fallback CDP ports
- discover() combining both with deduplication
- discover() with verify_cdp filtering unreachable instances
- discover() with custom port list
- discover_dcc_instances() enrichment flow
- get_instance_by_window_id / title / dcc with real discover() call structure
"""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.discovery import InstanceDiscovery


def _make_registry_file(
    tmp_dir: Path,
    cdp_port: int,
    pid: int = 12345,
    *,
    window_id: str = "win-1",
    dcc_type: str | None = None,
    app_name: str = "AuroraView",
) -> Path:
    """Write a registry JSON file and return its path."""
    data = {
        "cdp_port": cdp_port,
        "pid": pid,
        "window_id": window_id,
        "app_name": app_name,
        "title": f"View @ {cdp_port}",
        "url": f"http://localhost:{cdp_port}",
        "ws_url": f"ws://127.0.0.1:{cdp_port}/devtools/page/1",
    }
    if dcc_type:
        data["dcc_type"] = dcc_type
    file_path = tmp_dir / f"instance_{cdp_port}.json"
    file_path.write_text(json.dumps(data), encoding="utf-8")
    return file_path


class TestDiscoverRegistryOnly:
    """Tests where all instances come from the file registry."""

    @pytest.mark.asyncio
    async def test_single_registry_instance_returned(self) -> None:
        """Single valid registry entry is discovered."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=111)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover()

        assert len(instances) == 1
        assert instances[0].port == 9222

    @pytest.mark.asyncio
    async def test_multiple_registry_instances_returned(self) -> None:
        """Multiple valid registry entries are discovered."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)
            _make_registry_file(tmp_dir, cdp_port=9223, pid=101)
            _make_registry_file(tmp_dir, cdp_port=9224, pid=102)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover()

        ports = {inst.port for inst in instances}
        assert ports == {9222, 9223, 9224}

    @pytest.mark.asyncio
    async def test_stale_registry_entry_excluded(self) -> None:
        """Dead process entries are filtered out."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=999)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=False),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover()

        assert instances == []

    @pytest.mark.asyncio
    async def test_empty_registry_dir_returns_empty(self) -> None:
        """Empty registry directory yields no instances."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)

            with patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir):
                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover()

        assert instances == []


class TestDiscoverFallbackCDPProbe:
    """Tests where fallback CDP probing is the only source."""

    @pytest.mark.asyncio
    async def test_fallback_probe_finds_webview(self) -> None:
        """Probe finds a WebView2 instance on default ports."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "Browser": "Edg/120.0",
            "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/abc",
        }

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_response)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222])
                instances = await discovery.discover()

        assert len(instances) == 1
        assert instances[0].port == 9222
        assert "Edg" in instances[0].browser

    @pytest.mark.asyncio
    async def test_fallback_probe_skips_non_webview(self) -> None:
        """Probe skips instances that don't identify as WebView."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"Browser": "Firefox/100.0"}

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_response)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222])
                instances = await discovery.discover()

        assert instances == []

    @pytest.mark.asyncio
    async def test_fallback_probe_connection_error_skipped(self) -> None:
        """Ports that refuse connection are silently skipped."""
        import httpx

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(
                    side_effect=httpx.ConnectError("Connection refused")
                )
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222, 9223])
                instances = await discovery.discover()

        assert instances == []

    @pytest.mark.asyncio
    async def test_fallback_probe_non_200_status_skipped(self) -> None:
        """Non-200 status codes from probe are skipped."""
        mock_response = MagicMock()
        mock_response.status_code = 404

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_response)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222])
                instances = await discovery.discover()

        assert instances == []


class TestDiscoverCombinedDeduplication:
    """Tests that registry + fallback probe deduplicate by port."""

    @pytest.mark.asyncio
    async def test_registry_port_not_reprobed(self) -> None:
        """Port already in registry is not added again via fallback probe."""
        mock_probe_response = MagicMock()
        mock_probe_response.status_code = 200
        mock_probe_response.json.return_value = {
            "Browser": "Edg/120.0",
            "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/abc",
        }

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)  # port already in registry

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_probe_response)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222])
                instances = await discovery.discover()

        # Only 1 instance (from registry), not 2
        ports = [inst.port for inst in instances]
        assert ports.count(9222) == 1

    @pytest.mark.asyncio
    async def test_registry_and_probe_different_ports(self) -> None:
        """Registry instance at 9222, probe finds new instance at 9223."""
        mock_probe_response = MagicMock()
        mock_probe_response.status_code = 200
        mock_probe_response.json.return_value = {
            "Browser": "Chrome/120.0",
            "webSocketDebuggerUrl": "ws://127.0.0.1:9223/devtools/browser/xyz",
        }
        # Probe for 9222 fails (already in registry, not probed)
        # Probe for 9223 succeeds

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                # Only 9223 would be probed (9222 already in registry)
                mock_client.get = AsyncMock(return_value=mock_probe_response)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222, 9223])
                instances = await discovery.discover()

        ports = {inst.port for inst in instances}
        assert 9222 in ports  # from registry
        assert 9223 in ports  # from fallback probe
        assert len(instances) == 2

    @pytest.mark.asyncio
    async def test_custom_ports_override_defaults(self) -> None:
        """Custom ports list passed to discover() replaces default ports for fallback."""
        mock_fail_response = MagicMock()
        mock_fail_response.status_code = 404

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_fail_response)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[9222, 9223, 9224])
                # Override with a different port set - only 8080 is probed
                instances = await discovery.discover(ports=[8080])

        # No results (all fail), but confirms custom ports were used
        assert instances == []
        # Verify only 1 call was made (port 8080 only)
        assert mock_client.get.call_count == 1
        call_url = mock_client.get.call_args[0][0]
        assert "8080" in call_url


class TestDiscoverWithVerifyCDP:
    """Tests for verify_cdp=True filtering."""

    @pytest.mark.asyncio
    async def test_verify_cdp_removes_unreachable_registry_instance(self) -> None:
        """Registry instances that fail CDP verification are filtered out."""
        mock_fail = MagicMock()
        mock_fail.status_code = 500

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_fail)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[], verify_cdp=True)
                instances = await discovery.discover()

        assert instances == []

    @pytest.mark.asyncio
    async def test_verify_cdp_keeps_reachable_instance(self) -> None:
        """Registry instances that pass CDP verification are kept."""
        mock_ok = MagicMock()
        mock_ok.status_code = 200

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=mock_ok)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[], verify_cdp=True)
                instances = await discovery.discover()

        assert len(instances) == 1
        assert instances[0].port == 9222

    @pytest.mark.asyncio
    async def test_verify_cdp_exception_removes_instance(self) -> None:
        """Instance that raises during verification is removed."""
        import httpx

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(side_effect=httpx.ConnectError("timeout"))
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[], verify_cdp=True)
                instances = await discovery.discover()

        assert instances == []


class TestDiscoverDCCInstances:
    """Tests for discover_dcc_instances() enrichment integration."""

    @pytest.mark.asyncio
    async def test_enriches_instance_without_dcc_type(self) -> None:
        """Instances without dcc_type are enriched via CDP page list."""
        page_list_resp = MagicMock()
        page_list_resp.status_code = 200
        page_list_resp.json.return_value = [
            {"title": "Maya 2025 - AuroraView", "url": "http://localhost:8080"}
        ]

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)  # no dcc_type

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=page_list_resp)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover_dcc_instances()

        assert len(instances) == 1
        assert instances[0].dcc_type == "maya"

    @pytest.mark.asyncio
    async def test_skips_enrich_when_dcc_type_known(self) -> None:
        """Instances that already have dcc_type are not enriched."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, dcc_type="blender")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover_dcc_instances()

        # httpx should NOT have been called (no enrichment needed)
        mock_client.get.assert_not_called()
        assert instances[0].dcc_type == "blender"

    @pytest.mark.asyncio
    async def test_enrich_exception_does_not_crash(self) -> None:
        """Enrichment failure is silently handled, instance still returned."""
        import httpx

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100)

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(side_effect=httpx.ConnectError("refused"))
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover_dcc_instances()

        # Instance is still returned, just without dcc_type
        assert len(instances) == 1
        assert instances[0].dcc_type is None

    @pytest.mark.asyncio
    async def test_multiple_instances_mixed_dcc(self) -> None:
        """Mix of DCC-known and unknown instances correctly handled."""
        page_list_resp = MagicMock()
        page_list_resp.status_code = 200
        page_list_resp.json.return_value = [
            {"title": "Houdini FX 20.0", "url": "http://localhost:9223"}
        ]

        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, dcc_type="maya")
            _make_registry_file(tmp_dir, cdp_port=9223, pid=101)  # no dcc_type

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client.get = AsyncMock(return_value=page_list_resp)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                instances = await discovery.discover_dcc_instances()

        assert len(instances) == 2
        dcc_types = {inst.dcc_type for inst in instances}
        assert "maya" in dcc_types
        assert "houdini" in dcc_types


class TestGetInstanceByWindowId:
    """Tests for get_instance_by_window_id() using real discover() structure."""

    @pytest.mark.asyncio
    async def test_finds_by_window_id(self) -> None:
        """Returns instance matching window_id."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, window_id="target-win")
            _make_registry_file(tmp_dir, cdp_port=9223, pid=101, window_id="other-win")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_window_id("target-win")

        assert result is not None
        assert result.window_id == "target-win"
        assert result.port == 9222

    @pytest.mark.asyncio
    async def test_returns_none_when_not_found(self) -> None:
        """Returns None when no instance has the given window_id."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, window_id="win-A")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_window_id("nonexistent")

        assert result is None


class TestGetInstanceByTitle:
    """Tests for get_instance_by_title() using real discover() structure."""

    @pytest.mark.asyncio
    async def test_finds_by_title_substring(self) -> None:
        """Returns instance that has matching title substring."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            data = {
                "cdp_port": 9222,
                "pid": 100,
                "title": "Maya Tool - AuroraView",
                "url": "http://localhost:9222",
                "ws_url": "ws://127.0.0.1:9222/devtools/page/1",
            }
            (tmp_dir / "instance_9222.json").write_text(json.dumps(data), encoding="utf-8")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_title("Maya Tool")

        assert result is not None
        assert result.port == 9222

    @pytest.mark.asyncio
    async def test_title_search_case_insensitive(self) -> None:
        """Title search is case-insensitive."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            data = {
                "cdp_port": 9222,
                "pid": 100,
                "app_name": "Blender Plugin",
                "title": "",
                "url": "http://localhost:9222",
                "ws_url": "ws://127.0.0.1:9222/devtools/page/1",
            }
            (tmp_dir / "instance_9222.json").write_text(json.dumps(data), encoding="utf-8")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_title("blender plugin")

        assert result is not None
        assert result.port == 9222

    @pytest.mark.asyncio
    async def test_title_not_found_returns_none(self) -> None:
        """Returns None when no title matches."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, app_name="Nuke Panel")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
            ):
                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_title("Houdini")

        assert result is None


class TestGetInstanceByDCC:
    """Tests for get_instance_by_dcc() with discover_dcc_instances() under the hood."""

    @pytest.mark.asyncio
    async def test_finds_known_dcc_type(self) -> None:
        """Returns instance with matching dcc_type (already registered)."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, dcc_type="maya")
            _make_registry_file(tmp_dir, cdp_port=9223, pid=101, dcc_type="houdini")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_dcc("maya")

        assert result is not None
        assert result.dcc_type == "maya"
        assert result.port == 9222

    @pytest.mark.asyncio
    async def test_dcc_search_case_insensitive(self) -> None:
        """DCC type lookup is case-insensitive."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, dcc_type="blender")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_dcc("BLENDER")

        assert result is not None
        assert result.dcc_type == "blender"

    @pytest.mark.asyncio
    async def test_dcc_not_found_returns_none(self) -> None:
        """Returns None when no instance matches the requested DCC type."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            _make_registry_file(tmp_dir, cdp_port=9222, pid=100, dcc_type="maya")

            with (
                patch("auroraview_mcp.discovery.get_instances_dir", return_value=tmp_dir),
                patch("auroraview_mcp.discovery.is_process_alive", return_value=True),
                patch("httpx.AsyncClient") as mock_client_cls,
            ):
                mock_client = AsyncMock()
                mock_client.__aenter__ = AsyncMock(return_value=mock_client)
                mock_client.__aexit__ = AsyncMock(return_value=None)
                mock_client_cls.return_value = mock_client

                discovery = InstanceDiscovery(default_ports=[])
                result = await discovery.get_instance_by_dcc("nuke")

        assert result is None
