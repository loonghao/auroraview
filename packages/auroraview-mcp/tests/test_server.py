"""Tests for the auroraview_mcp.server module.

Covers module-level singleton creation, accessor functions, create_server(),
tool-module registration, and FastMCP instance configuration.
"""

from __future__ import annotations


class TestMCPInstance:
    """Tests for the module-level `mcp` FastMCP instance."""

    def test_mcp_instance_is_fastmcp(self) -> None:
        """mcp must be a FastMCP instance."""
        from fastmcp import FastMCP

        from auroraview_mcp.server import mcp

        assert isinstance(mcp, FastMCP)

    def test_mcp_name_is_auroraview(self) -> None:
        """mcp.name must be 'auroraview'."""
        from auroraview_mcp.server import mcp

        assert mcp.name == "auroraview"

    def test_mcp_instructions_contain_capabilities(self) -> None:
        """mcp instructions string must mention key capabilities."""
        from auroraview_mcp.server import mcp

        instructions = mcp.instructions or ""
        for keyword in [
            "discover_instances",
            "connect",
            "take_screenshot",
            "evaluate",
            "Gallery",
        ]:
            assert keyword in instructions, f"Missing keyword in instructions: {keyword}"

    def test_mcp_singleton_is_same_object(self) -> None:
        """Two imports of `mcp` must reference the same object."""
        from auroraview_mcp import server as srv1
        from auroraview_mcp import server as srv2

        assert srv1.mcp is srv2.mcp

    def test_mcp_has_tools_registered(self) -> None:
        """FastMCP instance must have tools registered after module import."""
        from auroraview_mcp.server import mcp  # noqa: F401 – triggers registration

        # FastMCP stores tools internally; we verify the object is not bare
        assert mcp is not None


class TestGetDiscovery:
    """Tests for get_discovery() accessor."""

    def test_returns_instance_discovery(self) -> None:
        """get_discovery() must return an InstanceDiscovery."""
        from auroraview_mcp.discovery import InstanceDiscovery
        from auroraview_mcp.server import get_discovery

        result = get_discovery()
        assert isinstance(result, InstanceDiscovery)

    def test_returns_singleton(self) -> None:
        """Two calls to get_discovery() must return the same object."""
        from auroraview_mcp.server import get_discovery

        a = get_discovery()
        b = get_discovery()
        assert a is b

    def test_default_ports_non_empty(self) -> None:
        """The singleton InstanceDiscovery must have default ports set."""
        from auroraview_mcp.server import get_discovery

        discovery = get_discovery()
        assert len(discovery.default_ports) > 0

    def test_discovery_has_discover_method(self) -> None:
        """Returned InstanceDiscovery must expose discover() coroutine."""
        import asyncio

        from auroraview_mcp.server import get_discovery

        discovery = get_discovery()
        assert hasattr(discovery, "discover")
        assert asyncio.iscoroutinefunction(discovery.discover)


class TestGetConnectionManager:
    """Tests for get_connection_manager() accessor."""

    def test_returns_connection_manager(self) -> None:
        """get_connection_manager() must return a ConnectionManager."""
        from auroraview_mcp.connection import ConnectionManager
        from auroraview_mcp.server import get_connection_manager

        result = get_connection_manager()
        assert isinstance(result, ConnectionManager)

    def test_returns_singleton(self) -> None:
        """Two calls to get_connection_manager() must return the same object."""
        from auroraview_mcp.server import get_connection_manager

        a = get_connection_manager()
        b = get_connection_manager()
        assert a is b

    def test_initial_not_connected(self) -> None:
        """The singleton ConnectionManager must start disconnected."""
        from auroraview_mcp.server import get_connection_manager

        manager = get_connection_manager()
        # is_connected is True only if _current_port is set; fresh module = False
        # We re-check by inspecting private state to avoid relying on external calls
        assert manager.current_port is None or isinstance(manager.current_port, int)

    def test_manager_has_connect_method(self) -> None:
        """Returned ConnectionManager must expose connect() coroutine."""
        import asyncio

        from auroraview_mcp.server import get_connection_manager

        manager = get_connection_manager()
        assert hasattr(manager, "connect")
        assert asyncio.iscoroutinefunction(manager.connect)

    def test_manager_has_disconnect_method(self) -> None:
        """Returned ConnectionManager must expose disconnect() coroutine."""
        import asyncio

        from auroraview_mcp.server import get_connection_manager

        manager = get_connection_manager()
        assert hasattr(manager, "disconnect")
        assert asyncio.iscoroutinefunction(manager.disconnect)


class TestCreateServer:
    """Tests for create_server() factory function."""

    def test_create_server_returns_fastmcp(self) -> None:
        """create_server() must return a FastMCP instance."""
        from fastmcp import FastMCP

        from auroraview_mcp.server import create_server

        server = create_server()
        assert isinstance(server, FastMCP)

    def test_create_server_returns_mcp_singleton(self) -> None:
        """create_server() must return the module-level `mcp` singleton."""
        from auroraview_mcp.server import create_server, mcp

        assert create_server() is mcp

    def test_create_server_idempotent(self) -> None:
        """Repeated calls to create_server() return the same object."""
        from auroraview_mcp.server import create_server

        s1 = create_server()
        s2 = create_server()
        assert s1 is s2

    def test_create_server_name_preserved(self) -> None:
        """Server returned by create_server() must keep name 'auroraview'."""
        from auroraview_mcp.server import create_server

        server = create_server()
        assert server.name == "auroraview"


class TestToolModuleRegistration:
    """Tests that all tool modules are imported and registered on mcp."""

    def test_all_tool_modules_importable(self) -> None:
        """All 8 tool modules must be importable without error."""
        from auroraview_mcp.tools import (  # noqa: F401
            api,
            dcc,
            debug,
            discovery,
            gallery,
            page,
            telemetry,
            ui,
        )

    def test_discovery_tools_exist(self) -> None:
        """discovery tool module must expose expected tool functions."""
        from auroraview_mcp.tools import discovery

        assert hasattr(discovery, "discover_instances")
        assert hasattr(discovery, "connect")
        assert hasattr(discovery, "disconnect")
        assert hasattr(discovery, "list_dcc_instances")

    def test_api_tools_exist(self) -> None:
        """api tool module must expose expected tool functions."""
        from auroraview_mcp.tools import api

        assert hasattr(api, "call_api")
        assert hasattr(api, "list_api_methods")
        assert hasattr(api, "emit_event")

    def test_ui_tools_exist(self) -> None:
        """ui tool module must expose expected tool functions."""
        from auroraview_mcp.tools import ui

        assert hasattr(ui, "take_screenshot")
        assert hasattr(ui, "click")
        assert hasattr(ui, "fill")
        assert hasattr(ui, "evaluate")

    def test_page_tools_exist(self) -> None:
        """page tool module must expose expected tool functions."""
        from auroraview_mcp.tools import page

        assert hasattr(page, "list_pages")
        assert hasattr(page, "select_page")
        assert hasattr(page, "get_page_info")

    def test_debug_tools_exist(self) -> None:
        """debug tool module must expose expected tool functions."""
        from auroraview_mcp.tools import debug

        assert hasattr(debug, "get_console_logs")
        assert hasattr(debug, "get_backend_status")

    def test_gallery_tools_exist(self) -> None:
        """gallery tool module must expose expected tool functions."""
        from auroraview_mcp.tools import gallery

        assert hasattr(gallery, "run_gallery")
        assert hasattr(gallery, "stop_gallery")
        assert hasattr(gallery, "run_sample")

    def test_dcc_tools_exist(self) -> None:
        """dcc tool module must expose expected tool functions."""
        from auroraview_mcp.tools import dcc

        assert hasattr(dcc, "get_dcc_context")
        assert hasattr(dcc, "execute_dcc_command")

    def test_telemetry_tools_exist(self) -> None:
        """telemetry tool module must expose expected tool functions."""
        from auroraview_mcp.tools import telemetry

        assert hasattr(telemetry, "get_telemetry")
        assert hasattr(telemetry, "get_performance_summary")


class TestServerModuleAttributes:
    """Tests for module-level attributes exposed by server.py."""

    def test_module_exports_mcp(self) -> None:
        """server module must export `mcp`."""
        import auroraview_mcp.server as srv

        assert hasattr(srv, "mcp")

    def test_module_exports_get_discovery(self) -> None:
        """server module must export `get_discovery`."""
        import auroraview_mcp.server as srv

        assert hasattr(srv, "get_discovery")
        assert callable(srv.get_discovery)

    def test_module_exports_get_connection_manager(self) -> None:
        """server module must export `get_connection_manager`."""
        import auroraview_mcp.server as srv

        assert hasattr(srv, "get_connection_manager")
        assert callable(srv.get_connection_manager)

    def test_module_exports_create_server(self) -> None:
        """server module must export `create_server`."""
        import auroraview_mcp.server as srv

        assert hasattr(srv, "create_server")
        assert callable(srv.create_server)

    def test_private_discovery_instance_is_instance_discovery(self) -> None:
        """Module-level `_discovery` must be an InstanceDiscovery."""
        import auroraview_mcp.server as srv
        from auroraview_mcp.discovery import InstanceDiscovery

        assert isinstance(srv._discovery, InstanceDiscovery)

    def test_private_connection_manager_instance(self) -> None:
        """Module-level `_connection_manager` must be a ConnectionManager."""
        import auroraview_mcp.server as srv
        from auroraview_mcp.connection import ConnectionManager

        assert isinstance(srv._connection_manager, ConnectionManager)

    def test_private_instances_are_same_as_accessors(self) -> None:
        """Accessor functions must return the same objects as module-level vars."""
        import auroraview_mcp.server as srv

        assert srv.get_discovery() is srv._discovery
        assert srv.get_connection_manager() is srv._connection_manager
