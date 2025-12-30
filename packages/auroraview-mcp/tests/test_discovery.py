"""Tests for instance discovery module."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.discovery import Instance, InstanceDiscovery


class TestInstance:
    """Tests for Instance dataclass."""

    def test_instance_creation(self) -> None:
        """Test creating an Instance."""
        instance = Instance(
            port=9222,
            browser="Chrome/120.0.0.0",
            ws_url="ws://127.0.0.1:9222/devtools/browser/xxx",
        )
        assert instance.port == 9222
        assert instance.browser == "Chrome/120.0.0.0"
        assert instance.ws_url == "ws://127.0.0.1:9222/devtools/browser/xxx"

    def test_instance_to_dict(self) -> None:
        """Test Instance.to_dict()."""
        instance = Instance(
            port=9222,
            browser="Edge/120.0.0.0",
            ws_url="ws://127.0.0.1:9222/devtools/browser/xxx",
            dcc_type="maya",
        )
        d = instance.to_dict()
        assert d["port"] == 9222
        assert d["browser"] == "Edge/120.0.0.0"
        assert d["dcc_type"] == "maya"

    def test_instance_defaults(self) -> None:
        """Test Instance default values."""
        instance = Instance(port=9222)
        assert instance.browser == ""
        assert instance.ws_url == ""
        assert instance.dcc_type is None
        assert instance.pid is None


class TestInstanceDiscovery:
    """Tests for InstanceDiscovery class."""

    def test_default_ports(self) -> None:
        """Test default port list."""
        discovery = InstanceDiscovery()
        assert discovery.default_ports == [9222, 9223, 9224, 9225]

    def test_custom_ports(self) -> None:
        """Test custom port list."""
        discovery = InstanceDiscovery(default_ports=[8080, 8081])
        assert discovery.default_ports == [8080, 8081]

    def test_is_webview_chrome(self) -> None:
        """Test _is_webview with Chrome."""
        discovery = InstanceDiscovery()
        assert discovery._is_webview({"Browser": "Chrome/120.0.0.0"})

    def test_is_webview_edge(self) -> None:
        """Test _is_webview with Edge."""
        discovery = InstanceDiscovery()
        assert discovery._is_webview({"Browser": "Edg/120.0.0.0"})

    def test_is_webview_other(self) -> None:
        """Test _is_webview with other browser."""
        discovery = InstanceDiscovery()
        assert not discovery._is_webview({"Browser": "Firefox/120.0"})

    def test_detect_dcc_type_maya(self) -> None:
        """Test DCC type detection for Maya."""
        discovery = InstanceDiscovery()
        assert discovery._detect_dcc_type("Maya 2025", "") == "maya"
        assert discovery._detect_dcc_type("Autodesk Maya", "") == "maya"

    def test_detect_dcc_type_blender(self) -> None:
        """Test DCC type detection for Blender."""
        discovery = InstanceDiscovery()
        assert discovery._detect_dcc_type("Blender 4.0", "") == "blender"

    def test_detect_dcc_type_houdini(self) -> None:
        """Test DCC type detection for Houdini."""
        discovery = InstanceDiscovery()
        assert discovery._detect_dcc_type("Houdini 20.0", "") == "houdini"
        assert discovery._detect_dcc_type("SideFX Houdini", "") == "houdini"

    def test_detect_dcc_type_none(self) -> None:
        """Test DCC type detection for unknown."""
        discovery = InstanceDiscovery()
        assert discovery._detect_dcc_type("Unknown App", "") is None

    @pytest.mark.asyncio
    async def test_discover_no_instances(self) -> None:
        """Test discover with no running instances."""
        discovery = InstanceDiscovery(timeout=0.1)

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(side_effect=Exception("Connection refused"))
            mock_client.return_value = mock_instance

            instances = await discovery.discover([9999])
            assert instances == []

    @pytest.mark.asyncio
    async def test_probe_port_success(self) -> None:
        """Test _probe_port with successful response."""
        discovery = InstanceDiscovery()

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "Browser": "Chrome/120.0.0.0",
            "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/xxx",
            "User-Agent": "Mozilla/5.0",
            "Protocol-Version": "1.3",
        }

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(return_value=mock_response)
            mock_client.return_value = mock_instance

            instance = await discovery._probe_port(9222)

            assert instance is not None
            assert instance.port == 9222
            assert instance.browser == "Chrome/120.0.0.0"

    @pytest.mark.asyncio
    async def test_probe_port_not_webview(self) -> None:
        """Test _probe_port with non-WebView response."""
        discovery = InstanceDiscovery()

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "Browser": "Firefox/120.0",
        }

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(return_value=mock_response)
            mock_client.return_value = mock_instance

            instance = await discovery._probe_port(9222)
            assert instance is None
