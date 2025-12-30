"""AuroraView MCP Server implementation."""

from __future__ import annotations

from mcp.server.fastmcp import FastMCP

from auroraview_mcp.connection import ConnectionManager
from auroraview_mcp.discovery import InstanceDiscovery

# Create the MCP server instance
mcp = FastMCP("auroraview")

# Global instances
_discovery = InstanceDiscovery()
_connection_manager = ConnectionManager()


def get_discovery() -> InstanceDiscovery:
    """Get the global discovery instance."""
    return _discovery


def get_connection_manager() -> ConnectionManager:
    """Get the global connection manager instance."""
    return _connection_manager


# Import tools to register them with the MCP server
from auroraview_mcp.tools import api, debug, discovery, gallery, page, ui  # noqa: E402, F401


def create_server() -> FastMCP:
    """Create and return the MCP server instance."""
    return mcp
