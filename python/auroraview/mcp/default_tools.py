# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Default MCP tools for AuroraView.

This module provides built-in MCP tools that are automatically registered
when MCP is enabled on a WebView instance.

Tools:
    - get_logs: Retrieve recent log messages from the application
    - clear_logs: Clear the log buffer
    - get_webview_info: Get information about the current WebView state (Python-side only)

Note:
    JavaScript-based tools (evaluate_js, get_dom_snapshot, click_element, fill_input,
    take_screenshot) are not available in the default tools because MCP tool handlers
    run in a separate thread and cannot safely access the WebView object which is
    thread-bound (!Send). Use the chrome-devtools MCP server for these features.
"""

from __future__ import annotations

import logging
from collections import deque
from datetime import datetime
from typing import TYPE_CHECKING, Any, Dict, Optional

if TYPE_CHECKING:
    from auroraview.core.webview import WebView


# Global log buffer for capturing logs
_log_buffer: deque = deque(maxlen=1000)
_log_handler: Optional["McpLogHandler"] = None

# Global storage for webview info (thread-safe snapshot)
_webview_info_snapshot: Dict[str, Any] = {}


class McpLogHandler(logging.Handler):
    """Custom log handler that captures logs for MCP access."""

    def __init__(self, buffer: deque, max_size: int = 1000):
        super().__init__()
        self.buffer = buffer
        self.max_size = max_size
        self.setFormatter(
            logging.Formatter(
                "%(asctime)s [%(levelname)s] %(name)s: %(message)s",
                datefmt="%Y-%m-%d %H:%M:%S",
            )
        )

    def emit(self, record: logging.LogRecord) -> None:
        try:
            entry = {
                "timestamp": datetime.fromtimestamp(record.created).isoformat(),
                "level": record.levelname,
                "logger": record.name,
                "message": record.getMessage(),
                "module": record.module,
                "lineno": record.lineno,
            }
            if record.exc_info:
                entry["exception"] = self.formatter.formatException(record.exc_info)
            self.buffer.append(entry)
        except Exception:
            pass  # Don't let logging errors break the app


def setup_log_capture(level: int = logging.DEBUG) -> None:
    """Setup log capture for MCP access.
    
    Args:
        level: Minimum log level to capture (default: DEBUG)
    """
    global _log_handler
    if _log_handler is not None:
        return  # Already setup
    
    _log_handler = McpLogHandler(_log_buffer)
    _log_handler.setLevel(level)
    
    # Add to root logger to capture all logs
    root_logger = logging.getLogger()
    root_logger.addHandler(_log_handler)
    
    # Also capture auroraview logs specifically
    av_logger = logging.getLogger("auroraview")
    av_logger.addHandler(_log_handler)


def update_webview_info_snapshot(webview: "WebView") -> None:
    """Update the thread-safe snapshot of webview info.
    
    This should be called from the main thread before MCP tools might access it.
    
    Args:
        webview: The WebView instance
    """
    global _webview_info_snapshot
    try:
        _webview_info_snapshot = {
            "title": getattr(webview, "_title", "Unknown"),
            "width": getattr(webview, "_width", 0),
            "height": getattr(webview, "_height", 0),
            "debug": getattr(webview, "_debug", False),
            "visible": getattr(webview, "_visible", False),
            "mcp_enabled": getattr(webview, "_mcp_enabled", False),
            "mcp_port": getattr(webview, "mcp_port", None),
            "snapshot_time": datetime.now().isoformat(),
        }
    except Exception as e:
        logging.getLogger("auroraview.mcp").warning(f"Failed to update webview info snapshot: {e}")


def register_default_tools(webview: "WebView", server: Any) -> None:
    """Register default MCP tools on the server.
    
    IMPORTANT: The tools registered here must NOT access the webview object directly
    because MCP tool handlers run in a separate tokio thread, and AuroraView is !Send.
    Instead, we use global state that is updated from the main thread.
    
    Args:
        webview: The WebView instance (used only for initial snapshot)
        server: The MCP server to register tools on
    """
    # Setup log capture
    setup_log_capture()
    
    # Take initial snapshot of webview info (from main thread)
    update_webview_info_snapshot(webview)
    
    # Tool: get_logs
    # Note: This function does NOT access webview - it only reads from global _log_buffer
    def get_logs(
        level: str = "DEBUG",
        limit: int = 100,
        logger_filter: str = "",
    ) -> Dict[str, Any]:
        """Get recent log messages from the application.
        
        Args:
            level: Minimum log level - DEBUG, INFO, WARNING, ERROR (default: DEBUG)
            limit: Maximum number of logs to return (default: 100)
            logger_filter: Filter logs by logger name prefix (default: "" for all)
        
        Returns:
            Dictionary with list of log entries
        """
        level_map = {
            "DEBUG": logging.DEBUG,
            "INFO": logging.INFO,
            "WARNING": logging.WARNING,
            "ERROR": logging.ERROR,
            "CRITICAL": logging.CRITICAL,
        }
        min_level = level_map.get(level.upper(), logging.DEBUG)
        
        logs = []
        for entry in list(_log_buffer)[-limit:]:
            entry_level = level_map.get(entry.get("level", "DEBUG"), logging.DEBUG)
            if entry_level < min_level:
                continue
            if logger_filter and not entry.get("logger", "").startswith(logger_filter):
                continue
            logs.append(entry)
        
        return {
            "ok": True,
            "data": {
                "logs": logs[-limit:],
                "total_buffered": len(_log_buffer),
                "returned": len(logs),
            }
        }
    
    # Tool: clear_logs
    # Note: This function does NOT access webview - it only modifies global _log_buffer
    def clear_logs() -> Dict[str, Any]:
        """Clear the log buffer.
        
        Returns:
            Dictionary with status
        """
        count = len(_log_buffer)
        _log_buffer.clear()
        return {
            "ok": True,
            "data": {"cleared": count}
        }
    
    # Tool: get_webview_info
    # Note: This function does NOT access webview - it reads from global _webview_info_snapshot
    def get_webview_info() -> Dict[str, Any]:
        """Get information about the current WebView state.
        
        Note: This returns a snapshot of Python-side state taken when MCP was initialized.
        For real-time JavaScript-based info (URL, document title, viewport),
        use chrome-devtools MCP server.
        
        Returns:
            Dictionary with WebView information
        """
        if not _webview_info_snapshot:
            return {
                "ok": False,
                "error": "WebView info snapshot not available. MCP may not be fully initialized."
            }
        return {"ok": True, "data": _webview_info_snapshot}

    # Register all tools
    tools = [
        ("mcp.get_logs", get_logs, "Get recent application logs"),
        ("mcp.clear_logs", clear_logs, "Clear the log buffer"),
        ("mcp.get_webview_info", get_webview_info, "Get WebView state information (snapshot)"),
    ]
    
    for name, handler, description in tools:
        try:
            server.register_tool(name, handler, description)
            logging.getLogger("auroraview.mcp").debug(f"Registered default tool: {name}")
        except Exception as e:
            logging.getLogger("auroraview.mcp").warning(f"Failed to register {name}: {e}")
