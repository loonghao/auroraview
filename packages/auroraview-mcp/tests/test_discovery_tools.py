"""Tests for tools/discovery.py – the MCP tool functions.

Covers discover_instances, connect, disconnect, and list_dcc_instances
by mocking the underlying InstanceDiscovery and ConnectionManager
singletons returned by get_discovery()/get_connection_manager().
"""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.connection import Page
from auroraview_mcp.discovery import Instance

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_instance(**kwargs) -> Instance:
    defaults = {
        "port": 9222,
        "browser": "Edg/120.0",
        "ws_url": "ws://localhost:9222/devtools/browser/xxx",
    }
    defaults.update(kwargs)
    return Instance(**defaults)


def _make_page(**kwargs) -> Page:
    defaults = {
        "id": "p1",
        "url": "http://localhost:8080",
        "title": "Test",
        "ws_url": "ws://localhost:9222/devtools/page/p1",
    }
    defaults.update(kwargs)
    return Page(**defaults)


# ---------------------------------------------------------------------------
# discover_instances
# ---------------------------------------------------------------------------

class TestDiscoverInstances:
    """Tests for discover_instances tool function."""

    @pytest.mark.asyncio
    async def test_returns_empty_list_when_no_instances(self) -> None:
        """discover_instances returns [] when discovery finds nothing."""
        from auroraview_mcp.tools.discovery import discover_instances

        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await discover_instances()

        assert result == []

    @pytest.mark.asyncio
    async def test_returns_serialized_instances(self) -> None:
        """discover_instances returns list of dicts (to_dict called)."""
        from auroraview_mcp.tools.discovery import discover_instances

        inst = _make_instance(port=9222, browser="Edg/120", dcc_type="maya")
        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[inst])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await discover_instances()

        assert len(result) == 1
        assert result[0]["port"] == 9222
        assert result[0]["browser"] == "Edg/120"
        assert result[0]["dcc_type"] == "maya"

    @pytest.mark.asyncio
    async def test_passes_ports_to_discovery(self) -> None:
        """discover_instances forwards ports argument to discovery.discover."""
        from auroraview_mcp.tools.discovery import discover_instances

        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            await discover_instances(ports=[9000, 9001])

        mock_discovery.discover.assert_called_once_with([9000, 9001])

    @pytest.mark.asyncio
    async def test_passes_none_ports_to_discovery(self) -> None:
        """discover_instances passes None when ports not provided."""
        from auroraview_mcp.tools.discovery import discover_instances

        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            await discover_instances()

        mock_discovery.discover.assert_called_once_with(None)

    @pytest.mark.asyncio
    async def test_multiple_instances_all_serialized(self) -> None:
        """discover_instances serializes all discovered instances."""
        from auroraview_mcp.tools.discovery import discover_instances

        instances = [
            _make_instance(port=9222, dcc_type="maya"),
            _make_instance(port=9223, dcc_type="blender"),
            _make_instance(port=9224, dcc_type="houdini"),
        ]
        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=instances)

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await discover_instances()

        assert len(result) == 3
        ports = [r["port"] for r in result]
        assert 9222 in ports
        assert 9223 in ports
        assert 9224 in ports

    @pytest.mark.asyncio
    async def test_to_dict_called_for_each_instance(self) -> None:
        """discover_instances calls to_dict on each returned instance."""
        from auroraview_mcp.tools.discovery import discover_instances

        mock_inst = MagicMock()
        mock_inst.to_dict.return_value = {"port": 9222, "browser": "test"}
        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[mock_inst, mock_inst])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await discover_instances()

        assert mock_inst.to_dict.call_count == 2
        assert len(result) == 2


# ---------------------------------------------------------------------------
# connect
# ---------------------------------------------------------------------------

class TestConnect:
    """Tests for connect tool function."""

    @pytest.mark.asyncio
    async def test_connect_returns_status_connected(self) -> None:
        """connect returns dict with status='connected'."""
        from auroraview_mcp.tools.discovery import connect

        page = _make_page()
        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[page])
        mock_manager.select_page = AsyncMock(return_value=page)
        mock_manager.current_page = page

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await connect(port=9222)

        assert result["status"] == "connected"
        assert result["port"] == 9222

    @pytest.mark.asyncio
    async def test_connect_calls_manager_connect_with_port(self) -> None:
        """connect invokes manager.connect(port)."""
        from auroraview_mcp.tools.discovery import connect

        page = _make_page()
        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[page])
        mock_manager.select_page = AsyncMock(return_value=page)
        mock_manager.current_page = page

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            await connect(port=9223)

        mock_manager.connect.assert_called_once_with(9223)

    @pytest.mark.asyncio
    async def test_connect_auto_selects_first_page(self) -> None:
        """connect calls select_page with first page id when pages exist."""
        from auroraview_mcp.tools.discovery import connect

        page = _make_page(id="first-page")
        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[page])
        mock_manager.select_page = AsyncMock(return_value=page)
        mock_manager.current_page = page

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            await connect(port=9222)

        mock_manager.select_page.assert_called_once_with(page_id="first-page")

    @pytest.mark.asyncio
    async def test_connect_no_pages_skips_select(self) -> None:
        """connect does not call select_page when no pages available."""
        from auroraview_mcp.tools.discovery import connect

        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[])
        mock_manager.current_page = None

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await connect(port=9222)

        mock_manager.select_page.assert_not_called()
        assert result["pages"] == []
        assert result["current_page"] is None

    @pytest.mark.asyncio
    async def test_connect_pages_serialized(self) -> None:
        """connect returns serialized pages list."""
        from auroraview_mcp.tools.discovery import connect

        page1 = _make_page(id="p1", url="http://localhost:8080")
        page2 = _make_page(id="p2", url="http://localhost:8081")

        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[page1, page2])
        mock_manager.select_page = AsyncMock(return_value=page1)
        mock_manager.current_page = page1

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await connect(port=9222)

        assert len(result["pages"]) == 2
        assert result["pages"][0]["id"] == "p1"
        assert result["pages"][1]["id"] == "p2"

    @pytest.mark.asyncio
    async def test_connect_current_page_serialized(self) -> None:
        """connect includes serialized current_page in result."""
        from auroraview_mcp.tools.discovery import connect

        page = _make_page(id="current-page", title="My Tool")
        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[page])
        mock_manager.select_page = AsyncMock(return_value=page)
        mock_manager.current_page = page

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await connect(port=9222)

        assert result["current_page"] is not None
        assert result["current_page"]["id"] == "current-page"
        assert result["current_page"]["title"] == "My Tool"

    @pytest.mark.asyncio
    async def test_connect_default_port_is_9222(self) -> None:
        """connect default port is 9222."""
        from auroraview_mcp.tools.discovery import connect

        mock_manager = MagicMock()
        mock_manager.connect = AsyncMock()
        mock_manager.get_pages = AsyncMock(return_value=[])
        mock_manager.current_page = None

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await connect()

        mock_manager.connect.assert_called_once_with(9222)
        assert result["port"] == 9222


# ---------------------------------------------------------------------------
# disconnect
# ---------------------------------------------------------------------------

class TestDisconnect:
    """Tests for disconnect tool function."""

    @pytest.mark.asyncio
    async def test_disconnect_returns_status_disconnected(self) -> None:
        """disconnect returns dict with status='disconnected'."""
        from auroraview_mcp.tools.discovery import disconnect

        mock_manager = MagicMock()
        mock_manager.current_port = 9222
        mock_manager.disconnect = AsyncMock()

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await disconnect()

        assert result["status"] == "disconnected"

    @pytest.mark.asyncio
    async def test_disconnect_includes_previous_port(self) -> None:
        """disconnect includes previously connected port in result."""
        from auroraview_mcp.tools.discovery import disconnect

        mock_manager = MagicMock()
        mock_manager.current_port = 9223
        mock_manager.disconnect = AsyncMock()

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await disconnect()

        assert result["port"] == 9223

    @pytest.mark.asyncio
    async def test_disconnect_calls_manager_disconnect(self) -> None:
        """disconnect calls manager.disconnect()."""
        from auroraview_mcp.tools.discovery import disconnect

        mock_manager = MagicMock()
        mock_manager.current_port = 9222
        mock_manager.disconnect = AsyncMock()

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            await disconnect()

        mock_manager.disconnect.assert_called_once()

    @pytest.mark.asyncio
    async def test_disconnect_when_not_connected_returns_none_port(self) -> None:
        """disconnect when not connected returns port=None."""
        from auroraview_mcp.tools.discovery import disconnect

        mock_manager = MagicMock()
        mock_manager.current_port = None
        mock_manager.disconnect = AsyncMock()

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await disconnect()

        assert result["port"] is None
        assert result["status"] == "disconnected"

    @pytest.mark.asyncio
    async def test_disconnect_reads_port_before_disconnect(self) -> None:
        """disconnect reads current_port before calling manager.disconnect."""
        from auroraview_mcp.tools.discovery import disconnect

        port_snapshot: list[int | None] = []

        mock_manager = MagicMock()
        mock_manager.current_port = 9224

        async def fake_disconnect():
            # Simulate port being cleared after disconnect
            port_snapshot.append(mock_manager.current_port)
            mock_manager.current_port = None

        mock_manager.disconnect = fake_disconnect

        with patch("auroraview_mcp.tools.discovery.get_connection_manager", return_value=mock_manager):
            result = await disconnect()

        # The port in the result must be the value BEFORE disconnect cleared it
        assert result["port"] == 9224
        assert port_snapshot == [9224]


# ---------------------------------------------------------------------------
# list_dcc_instances
# ---------------------------------------------------------------------------

class TestListDCCInstances:
    """Tests for list_dcc_instances tool function."""

    @pytest.mark.asyncio
    async def test_returns_empty_when_no_dcc_instances(self) -> None:
        """list_dcc_instances returns [] when no DCC instances found."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await list_dcc_instances()

        assert result == []

    @pytest.mark.asyncio
    async def test_returns_serialized_dcc_instances(self) -> None:
        """list_dcc_instances returns list of dicts with DCC info."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        inst = _make_instance(port=9222, dcc_type="maya", dcc_version="2025")
        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=[inst])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await list_dcc_instances()

        assert len(result) == 1
        assert result[0]["dcc_type"] == "maya"
        assert result[0]["dcc_version"] == "2025"

    @pytest.mark.asyncio
    async def test_passes_ports_to_discover_dcc_instances(self) -> None:
        """list_dcc_instances forwards ports to discover_dcc_instances."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            await list_dcc_instances(ports=[9300, 9301])

        mock_discovery.discover_dcc_instances.assert_called_once_with([9300, 9301])

    @pytest.mark.asyncio
    async def test_passes_none_ports_by_default(self) -> None:
        """list_dcc_instances passes None when ports not specified."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            await list_dcc_instances()

        mock_discovery.discover_dcc_instances.assert_called_once_with(None)

    @pytest.mark.asyncio
    async def test_multiple_dcc_types_serialized(self) -> None:
        """list_dcc_instances serializes instances of different DCC types."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        instances = [
            _make_instance(port=9222, dcc_type="maya"),
            _make_instance(port=9223, dcc_type="blender"),
            _make_instance(port=9224, dcc_type="houdini"),
            _make_instance(port=9225, dcc_type="unreal"),
        ]
        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=instances)

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await list_dcc_instances()

        assert len(result) == 4
        dcc_types = [r["dcc_type"] for r in result]
        assert "maya" in dcc_types
        assert "blender" in dcc_types
        assert "houdini" in dcc_types
        assert "unreal" in dcc_types

    @pytest.mark.asyncio
    async def test_to_dict_called_for_each_dcc_instance(self) -> None:
        """list_dcc_instances calls to_dict on each DCC instance."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        mock_inst = MagicMock()
        mock_inst.to_dict.return_value = {"port": 9222, "dcc_type": "maya"}
        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=[mock_inst, mock_inst])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery):
            result = await list_dcc_instances()

        assert mock_inst.to_dict.call_count == 2
        assert len(result) == 2

    @pytest.mark.asyncio
    async def test_uses_same_discovery_singleton(self) -> None:
        """list_dcc_instances uses get_discovery() singleton."""
        from auroraview_mcp.tools.discovery import list_dcc_instances

        mock_discovery = MagicMock()
        mock_discovery.discover_dcc_instances = AsyncMock(return_value=[])

        with patch("auroraview_mcp.tools.discovery.get_discovery", return_value=mock_discovery) as mock_get:
            await list_dcc_instances()

        mock_get.assert_called_once()
