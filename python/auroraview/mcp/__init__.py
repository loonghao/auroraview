# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Embedded MCP Server for AuroraView (Rust-only).

This module exposes the Rust MCP Server bindings. Python fallback has been
removed; building with ``--features mcp-server`` is required.

Example:
    >>> from auroraview import WebView
    >>> webview = WebView(title="My Tool", mcp=True)
    >>> webview.show()
"""

from __future__ import annotations

from importlib import import_module


# Try to import from standalone _mcp module first (modular build),
# then fallback to core._core with mcp-server feature (monolithic build)
try:
    # Modular build: _mcp.pyd in auroraview/mcp/
    from ._mcp import McpConfig, McpServer
    _USE_RUST_MCP = True
except ImportError:
    try:
        # Monolithic build: core/_core.pyd with mcp-server feature
        _core = import_module("auroraview.core._core")
        McpConfig = getattr(_core, "McpConfig")
        McpServer = getattr(_core, "McpServer")
        _USE_RUST_MCP = True
    except Exception as exc:  # pragma: no cover - hard fail when Rust binding missing
        raise ImportError(
            "Rust MCP bindings are required. Either:\n"
            "  1. Build standalone MCP module: just rebuild-mcp\n"
            "  2. Build core with MCP feature: just rebuild-pylib-with-mcp\n"
            "Ensure the PyO3 extension is available."
        ) from exc


def __getattr__(name: str):
    """Lazy import for default_tools to avoid circular imports."""
    if name == "register_default_tools":
        from .default_tools import register_default_tools
        return register_default_tools
    if name == "setup_log_capture":
        from .default_tools import setup_log_capture
        return setup_log_capture
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


__all__ = [
    "McpConfig",
    "McpServer",
    "register_default_tools",
    "setup_log_capture",
    "_USE_RUST_MCP",
]

