# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView Core Module.

This module contains the core WebView functionality:
- WebView: The main WebView class
- Backend: Backend abstraction layer
- Settings: WebView configuration
- Cookies: Cookie management
- Rust bindings: _core.pyd and _signals.pyd

Example:
    >>> from auroraview.core import WebView, WebViewSettings
    >>> webview = WebView(title="My App")
    >>> webview.show()
"""

from __future__ import annotations

import os as _os
import sys as _sys
from pathlib import Path as _Path

# ============================================================
# DLL search paths for Windows (required for DCC applications)
# ============================================================
# This must be done BEFORE importing _core to ensure all required DLLs can be found.
#
# Background: Windows DLL search behavior changed in Python 3.8+
# - PATH environment variable is no longer sufficient for DLL discovery
# - Must explicitly call os.add_dll_directory() for each DLL search path
#
# Required DLLs:
# - python3.dll: Located in sys.prefix (e.g., DCC's pythonsdk directory)
# - WebView2Loader.dll: Located in auroraview package directory
if _sys.platform == "win32" and hasattr(_os, "add_dll_directory"):
    # List of directories to add for DLL search
    _dll_dirs = [
        _Path(__file__).parent,  # core package dir (for _core.pyd)
        _Path(__file__).parent.parent,  # auroraview package dir (for WebView2Loader.dll)
        _Path(_sys.prefix),  # Python install dir (for python3.dll in DCC apps)
    ]

    for _dll_dir in _dll_dirs:
        if _dll_dir.exists():
            try:
                _os.add_dll_directory(str(_dll_dir))
            except OSError:
                pass  # Directory may already be added or not a valid DLL directory

# ============================================================
# Rust bindings (_core.pyd)
# ============================================================
_CORE_IMPORT_ERROR = None
try:
    from ._core import (
        # Metadata
        __author__,
        __version__,
        # High-performance DOM batch operations (Rust-powered)
        DomBatch,
        # Window utilities
        WindowInfo,
        close_window_by_hwnd,
        destroy_window_by_hwnd,
        find_window_by_exact_title,
        find_windows_by_title,
        get_all_windows,
        get_foreground_window,
        fix_webview2_child_windows,  # Qt6 compatibility
        # CLI utilities
        normalize_url,
        rewrite_html_for_custom_protocol,
        # Desktop runner (new name)
        run_desktop,
        # Standalone runner (legacy alias)
        run_standalone,
        # WebView2 warmup (Windows performance optimization)
        start_warmup,
        warmup_sync,
        is_warmup_complete,
        get_warmup_progress,
        get_warmup_stage,
        get_warmup_status,
        get_shared_user_data_folder,
        # Plugin system for native desktop operations
        PluginManager,
        # Thread-safe event emitter for cross-thread operations
        EventEmitter as RustEventEmitter,
        # High-performance JSON functions (orjson-equivalent, no Python deps)
        json_loads,
        json_dumps,
        json_dumps_bytes,
    )
except ImportError as e:
    _CORE_IMPORT_ERROR = str(e)
    # Fallback for development without compiled extension
    __version__ = "0.1.0.dev"
    __author__ = "Hal Long <hal.long@outlook.com>"

    # Placeholder for window utilities
    WindowInfo = None  # type: ignore
    get_foreground_window = None  # type: ignore
    find_windows_by_title = None  # type: ignore
    find_window_by_exact_title = None  # type: ignore
    get_all_windows = None  # type: ignore
    close_window_by_hwnd = None  # type: ignore
    destroy_window_by_hwnd = None  # type: ignore
    fix_webview2_child_windows = None  # type: ignore

    # Placeholder for CLI utilities
    normalize_url = None  # type: ignore
    rewrite_html_for_custom_protocol = None  # type: ignore
    run_desktop = None  # type: ignore
    run_standalone = None  # type: ignore

    # Placeholder for DOM batch
    DomBatch = None  # type: ignore

    # Placeholder for warmup functions
    start_warmup = None  # type: ignore
    warmup_sync = None  # type: ignore
    is_warmup_complete = None  # type: ignore
    get_warmup_progress = None  # type: ignore
    get_warmup_stage = None  # type: ignore
    get_warmup_status = None  # type: ignore
    get_shared_user_data_folder = None  # type: ignore

    # Placeholder for plugin system
    PluginManager = None  # type: ignore

    # Placeholder for event emitter
    RustEventEmitter = None  # type: ignore

    # Placeholder for JSON functions
    json_loads = None  # type: ignore
    json_dumps = None  # type: ignore
    json_dumps_bytes = None  # type: ignore

# SidecarBridge is optional (mcp-sidecar feature)
try:
    from ._core import SidecarBridge
except ImportError:
    SidecarBridge = None  # type: ignore

from .backend import (
    BackendType,
    get_available_backends,
    get_backend_type,
    get_default_backend,
    is_backend_available,
    set_backend_type,
)
from .channel import Channel, ChannelManager
from .commands import CommandError, CommandErrorCode, CommandRegistry
from .ipc_channel import (
    IpcChannel,
    IpcChannelError,
    emit_event,
    report_progress,
    report_result,
    send_to_parent,
)
from .cookies import Cookie
from .event_emitter import (
    EventEmitter,
    LoadEvent,
    NavigationEvent,
    WindowEvent as WindowEventData2,
    deprecated,
)
from .events import EventHandler, WindowEvent, WindowEventData
from .response import (
    ApiResponse,
    Response,
    err,
    failure,
    is_response,
    normalize,
    ok,
    success,
    wrap_response,
)
from .settings import DEFAULT_SETTINGS, WebViewSettings
from .signals import ConnectionGuard, ConnectionId, Signal, SignalRegistry, WebViewSignals
from .state import State
from .webview import WebView

# Import submodules for attribute access
from . import backend as backend
from . import channel as channel
from . import commands as commands
from . import cookies as cookies
from . import event_emitter as event_emitter
from . import events as events
from . import ipc_channel as ipc_channel
from . import response as response
from . import settings as settings
from . import state as state
from . import webview as webview

__all__ = [
    # ============================================================
    # Metadata (from _core.pyd)
    # ============================================================
    "__version__",
    "__author__",
    # ============================================================
    # Rust bindings (from _core.pyd)
    # ============================================================
    # High-performance DOM batch operations
    "DomBatch",
    # Window utilities
    "WindowInfo",
    "get_foreground_window",
    "find_windows_by_title",
    "find_window_by_exact_title",
    "get_all_windows",
    "close_window_by_hwnd",
    "destroy_window_by_hwnd",
    "fix_webview2_child_windows",  # Qt6 compatibility
    # CLI utilities
    "normalize_url",
    "rewrite_html_for_custom_protocol",
    # Desktop runner
    "run_desktop",
    "run_standalone",  # Legacy alias
    # WebView2 warmup (Windows performance optimization)
    "start_warmup",
    "warmup_sync",
    "is_warmup_complete",
    "get_warmup_progress",
    "get_warmup_stage",
    "get_warmup_status",
    "get_shared_user_data_folder",
    # Plugin system
    "PluginManager",
    # Thread-safe event emitter (Rust)
    "RustEventEmitter",
    # High-performance JSON functions
    "json_loads",
    "json_dumps",
    "json_dumps_bytes",
    # MCP Sidecar Bridge (optional, requires mcp-sidecar feature)
    "SidecarBridge",
    # Core import error (for diagnostics)
    "_CORE_IMPORT_ERROR",
    # ============================================================
    # WebView
    # ============================================================
    "WebView",
    # ============================================================
    # Backend abstraction
    # ============================================================
    "BackendType",
    "get_backend_type",
    "set_backend_type",
    "get_default_backend",
    "get_available_backends",
    "is_backend_available",
    # ============================================================
    # Settings
    # ============================================================
    "WebViewSettings",
    "DEFAULT_SETTINGS",
    # ============================================================
    # Cookie management
    # ============================================================
    "Cookie",
    # ============================================================
    # Events
    # ============================================================
    "WindowEvent",
    "WindowEventData",
    "EventHandler",
    # EventEmitter pattern (Python implementation)
    "EventEmitter",
    "NavigationEvent",
    "LoadEvent",
    "deprecated",
    # ============================================================
    # Response utilities
    # ============================================================
    "Response",
    "ApiResponse",
    "ok",
    "err",
    "success",
    "failure",
    "is_response",
    "normalize",
    "wrap_response",
    # ============================================================
    # State
    # ============================================================
    "State",
    # ============================================================
    # Signals (Qt-inspired)
    # ============================================================
    "Signal",
    "SignalRegistry",
    "ConnectionId",
    "ConnectionGuard",
    "WebViewSignals",
    # ============================================================
    # Commands
    # ============================================================
    "CommandRegistry",
    "CommandError",
    "CommandErrorCode",
    # ============================================================
    # Channels
    # ============================================================
    "Channel",
    "ChannelManager",
    # ============================================================
    # IPC Channel (subprocess communication)
    # ============================================================
    "IpcChannel",
    "IpcChannelError",
    "send_to_parent",
    "emit_event",
    "report_progress",
    "report_result",
    # ============================================================
    # Submodules
    # ============================================================
    "backend",
    "channel",
    "commands",
    "cookies",
    "events",
    "event_emitter",
    "ipc_channel",
    "response",
    "settings",
    "state",
    "webview",
]
