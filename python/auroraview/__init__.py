# -*- coding: utf-8 -*-
"""AuroraView - Rust-powered WebView for Python & DCC embedding.

This package provides a modern web-based UI solution for professional DCC applications
like Maya, 3ds Max, Houdini, Blender, Nuke, and Unreal Engine.

## Quick Start

Choose the right API for your use case:

| Use Case | API | Description |
|----------|-----|-------------|
| Maya/Houdini/Nuke | ``QtWebView`` | Qt widget with docking support |
| Unreal Engine | ``AuroraView`` | HWND-based for non-Qt apps |
| Desktop App | ``run_desktop()`` | One-liner for standalone apps |
| Advanced | ``auroraview.core.WebView`` | Low-level API (not recommended) |

## Integration Modes

### 1. Qt Native Mode (QtWebView) - For Qt-based DCC

Best for Maya, Houdini, Nuke, 3ds Max, and other Qt-based applications.
Supports QDockWidget docking and native Qt widget integration::

    from auroraview import QtWebView

    # Create WebView as Qt widget (dockable!)
    webview = QtWebView(
        parent=maya_main_window(),
        url="http://localhost:3000",
        width=800,
        height=600
    )
    webview.show()

### 2. HWND Mode (AuroraView) - For Unreal Engine & Other Apps

Best for Unreal Engine or any application that needs HWND access::

    from auroraview import AuroraView

    # Create standalone WebView
    webview = AuroraView(url="http://localhost:3000")
    webview.show()

    # Get HWND for Unreal Engine embedding
    hwnd = webview.get_hwnd()
    if hwnd:
        import unreal
        unreal.parent_external_window_to_slate(hwnd)

### 3. Desktop Mode - For Desktop Apps

Best for standalone desktop applications::

    from auroraview import run_desktop

    # Quick one-liner for desktop apps
    run_desktop(
        title="My App",
        url="https://example.com",
        width=1024,
        height=768
    )

## Bidirectional Communication

Python -> JavaScript::

    webview.emit("update_data", {"frame": 120})

JavaScript -> Python::

    @webview.on("export_scene")
    def handle_export(data):
        print(f"Exporting to: {data['path']}")

## Advanced Usage

For advanced users who need low-level control, use ``auroraview.core.WebView``::

    from auroraview.core import WebView

    webview = WebView.create(
        title="Advanced Tool",
        url="http://localhost:3000",
        parent=parent_hwnd,
        mode="owner"
    )
    webview.show()

Note: Direct use of ``WebView`` is not recommended for most use cases.
Use ``QtWebView`` for Qt-based DCC apps or ``AuroraView`` for HWND-based apps.
"""

_CORE_IMPORT_ERROR = None

# Add DLL search paths for Windows (required for DCC applications like Substance Painter)
# This must be done BEFORE importing _core to ensure all required DLLs can be found.
#
# Background: Windows DLL search behavior changed in Python 3.8+
# - PATH environment variable is no longer sufficient for DLL discovery
# - Must explicitly call os.add_dll_directory() for each DLL search path
#
# Required DLLs:
# - python3.dll: Located in sys.prefix (e.g., DCC's pythonsdk directory)
# - WebView2Loader.dll: Located in auroraview package directory
import os as _os
import sys as _sys
from pathlib import Path as _Path

if _sys.platform == "win32" and hasattr(_os, "add_dll_directory"):
    # List of directories to add for DLL search
    _dll_dirs = [
        _Path(__file__).parent,  # auroraview package dir (for WebView2Loader.dll)
        _Path(_sys.prefix),  # Python install dir (for python3.dll in DCC apps)
    ]

    for _dll_dir in _dll_dirs:
        if _dll_dir.exists():
            try:
                _os.add_dll_directory(str(_dll_dir))
            except OSError:
                pass  # Directory may already be added or not a valid DLL directory

try:
    from ._core import (
        # High-performance DOM batch operations (Rust-powered)
        DomBatch,
        # Window utilities
        WindowInfo,
        __author__,
        __version__,
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
        EventEmitter,
        # High-performance JSON functions (orjson-equivalent, no Python deps)
        json_loads,
        json_dumps,
        json_dumps_bytes,
    )
except ImportError as e:
    # Capture the import error for diagnostics
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

    # Placeholder for JSON functions
    json_loads = None  # type: ignore
    json_dumps = None  # type: ignore
    json_dumps_bytes = None  # type: ignore


def diagnose_core_library() -> dict:
    """Diagnose core library loading issues.

    Returns a dict with diagnostic information useful for troubleshooting
    when the Rust core library fails to load.

    Returns:
        dict: Diagnostic information including Python version, platform,
              import error details, and file locations.
    """
    import sys
    from pathlib import Path

    result = {
        "python_version": sys.version,
        "python_executable": sys.executable,
        "platform": sys.platform,
        "core_import_error": _CORE_IMPORT_ERROR,
        "core_loaded": _CORE_IMPORT_ERROR is None,
    }

    # Check for _core.pyd location
    try:
        import auroraview

        pkg_dir = Path(auroraview.__file__).parent
        pyd_path = pkg_dir / "_core.pyd"
        so_path = pkg_dir / "_core.so"

        result["package_dir"] = str(pkg_dir)
        result["pyd_exists"] = pyd_path.exists()
        result["so_exists"] = so_path.exists()

        if pyd_path.exists():
            result["pyd_path"] = str(pyd_path)
            result["pyd_size"] = pyd_path.stat().st_size
        if so_path.exists():
            result["so_path"] = str(so_path)
            result["so_size"] = so_path.stat().st_size
    except Exception as e:
        result["path_check_error"] = str(e)

    # Check sys.path
    result["sys_path"] = sys.path[:10]  # First 10 entries

    return result


# Import from submodules
from .core import (
    DEFAULT_SETTINGS,
    BackendType,
    ConnectionGuard,
    ConnectionId,
    Cookie,
    EventEmitter,
    EventHandler,
    LoadEvent,
    NavigationEvent,
    Signal,
    SignalRegistry,
    WebView,  # Kept for backward compatibility, prefer QtWebView/AuroraView
    WebViewSettings,
    WebViewSignals,
    WindowEvent,
    WindowEventData,
    deprecated,
    get_available_backends,
    get_backend_type,
    get_default_backend,
    is_backend_available,
    set_backend_type,
)

# Note: WebView is exported for backward compatibility, but for new code:
# - Use QtWebView for Qt-based DCC apps (Maya, Houdini, Nuke)
# - Use AuroraView for HWND-based apps (Unreal Engine)
# - Use run_desktop() for standalone desktop applications
from .integration import AuroraView, Bridge, QtWebView
from .ui import Element, ElementCollection, Menu, MenuBar, MenuItem, MenuItemType
from .utils import (
    Automation,
    BrowserBackend,
    EventTimer,
    LocalWebViewBackend,
    QtTimerBackend,
    SteelBrowserBackend,
    ThreadTimerBackend,
    TimerBackend,
    get_available_backend,
    list_registered_backends,
    path_to_file_url,
    prepare_html_with_local_assets,
    register_timer_backend,
)

# Service Discovery (optional - requires Rust core)
_SERVICE_DISCOVERY_IMPORT_ERROR = None
try:
    from ._core import ServiceDiscovery, ServiceInfo
except ImportError as e:
    _SERVICE_DISCOVERY_IMPORT_ERROR = str(e)

    class ServiceDiscovery:  # type: ignore
        """ServiceDiscovery placeholder - Rust core not available."""

        def __init__(self, *_args, **_kwargs):
            raise ImportError(
                "ServiceDiscovery requires Rust core module. "
                "Rebuild the package with: pip install -e .\n"
                f"Original error: {_SERVICE_DISCOVERY_IMPORT_ERROR}"
            )

    class ServiceInfo:  # type: ignore
        """ServiceInfo placeholder - Rust core not available."""

        pass


# Qt availability flag for tests
_QT_IMPORT_ERROR = None
try:
    from qtpy import QtCore as _QtCore

    _HAS_QT = True
except ImportError as e:
    _HAS_QT = False
    _QT_IMPORT_ERROR = str(e)

# Import submodules for backward-compatibility aliases
from . import core, integration, ui, utils

# Backward-compatibility aliases for old import paths
# These allow: from auroraview.webview import WebView
# and: from auroraview.event_timer import EventTimer
webview = core.webview  # auroraview.webview -> auroraview.core.webview
event_timer = utils.event_timer  # auroraview.event_timer -> auroraview.utils.event_timer
file_protocol = utils.file_protocol  # auroraview.file_protocol -> auroraview.utils.file_protocol
timer_backends = (
    utils.timer_backends
)  # auroraview.timer_backends -> auroraview.utils.timer_backends
dom = ui.dom  # auroraview.dom -> auroraview.ui.dom
qt_integration = integration.qt  # auroraview.qt_integration -> auroraview.integration.qt

# Simple top-level event decorator (for tests/backward-compat)
_EVENT_HANDLERS = {}


def on_event(event_name: str):
    """Top-level event decorator used in basic examples/tests.

    Note: This is a lightweight registry; core event routing is per-WebView via
    webview.on(). This helper exists for compatibility with older code/tests.
    """

    def decorator(func):
        _EVENT_HANDLERS.setdefault(event_name, []).append(func)
        return func

    return decorator


# Submodule imports for organized access
from . import core  # auroraview.core - WebView, Backend, Settings, Cookies
from . import integration  # auroraview.integration - AuroraView, Bridge, Qt
from . import ui  # auroraview.ui - DOM, Menu
from . import utils  # auroraview.utils - EventTimer, FileProtocol, Automation

__all__ = [
    # ============================================================
    # Submodules (organized access)
    # ============================================================
    "core",  # auroraview.core - WebView, Backend, Settings, Cookies
    "ui",  # auroraview.ui - DOM, Menu
    "integration",  # auroraview.integration - AuroraView, Bridge, Qt
    "utils",  # auroraview.utils - EventTimer, FileProtocol, Automation
    # ============================================================
    # Primary APIs (recommended)
    # ============================================================
    # Qt-based DCC integration (Maya, Houdini, Nuke, 3ds Max)
    "QtWebView",
    # HWND-based integration (Unreal Engine, non-Qt apps)
    "AuroraView",
    # Desktop standalone apps
    "run_desktop",
    "run_standalone",  # Legacy alias for run_desktop
    # Bridge for DCC integration
    "Bridge",
    # ============================================================
    # Core WebView (backward compatibility)
    # ============================================================
    # Note: Prefer QtWebView/AuroraView/run_desktop for new code
    "WebView",
    # ============================================================
    # Core utilities (auroraview.core)
    # ============================================================
    # Backend abstraction
    "BackendType",
    "get_backend_type",
    "set_backend_type",
    "get_default_backend",
    "get_available_backends",
    "is_backend_available",
    # Settings
    "WebViewSettings",
    "DEFAULT_SETTINGS",
    # Cookie management
    "Cookie",
    # Events
    "WindowEvent",
    "WindowEventData",
    "EventHandler",
    # EventEmitter pattern
    "EventEmitter",
    "NavigationEvent",
    "LoadEvent",
    "deprecated",
    # Signals (Qt-inspired)
    "Signal",
    "SignalRegistry",
    "ConnectionId",
    "ConnectionGuard",
    "WebViewSignals",
    # ============================================================
    # UI (auroraview.ui)
    # ============================================================
    # DOM manipulation
    "Element",
    "ElementCollection",
    # High-performance DOM batch (Rust-powered)
    "DomBatch",
    # Menu support
    "Menu",
    "MenuBar",
    "MenuItem",
    "MenuItemType",
    # ============================================================
    # Service Discovery
    # ============================================================
    "ServiceDiscovery",
    "ServiceInfo",
    # ============================================================
    # Utils (auroraview.utils)
    # ============================================================
    # Event Timer
    "EventTimer",
    # Timer Backends
    "TimerBackend",
    "QtTimerBackend",
    "ThreadTimerBackend",
    "register_timer_backend",
    "get_available_backend",
    "list_registered_backends",
    # File protocol utilities
    "path_to_file_url",
    "prepare_html_with_local_assets",
    # Automation (Steel Browser compatible)
    "Automation",
    "BrowserBackend",
    "LocalWebViewBackend",
    "SteelBrowserBackend",
    # ============================================================
    # Window utilities (Rust-powered)
    # ============================================================
    "WindowInfo",
    "get_foreground_window",
    "find_windows_by_title",
    "find_window_by_exact_title",
    "get_all_windows",
    "close_window_by_hwnd",
    "destroy_window_by_hwnd",
    "fix_webview2_child_windows",  # Qt6 compatibility
    # ============================================================
    # CLI utilities (Rust-powered)
    # ============================================================
    "normalize_url",
    "rewrite_html_for_custom_protocol",
    # ============================================================
    # WebView2 warmup (Windows performance optimization)
    # ============================================================
    "start_warmup",
    "warmup_sync",
    "is_warmup_complete",
    "get_warmup_progress",
    "get_warmup_stage",
    "get_warmup_status",
    "get_shared_user_data_folder",
    # ============================================================
    # High-performance JSON (Rust-powered, orjson-equivalent)
    # ============================================================
    "json_loads",
    "json_dumps",
    "json_dumps_bytes",
    # ============================================================
    # Plugin system
    # ============================================================
    "PluginManager",
    # ============================================================
    # Helpers
    # ============================================================
    "on_event",
    # ============================================================
    # Backward-compatibility aliases
    # ============================================================
    "_HAS_QT",  # Qt availability flag
    "_QT_IMPORT_ERROR",  # Qt import error message (for tests)
    "webview",  # auroraview.webview -> auroraview.core.webview
    "event_timer",  # auroraview.event_timer -> auroraview.utils.event_timer
    "file_protocol",  # auroraview.file_protocol -> auroraview.utils.file_protocol
    "timer_backends",  # auroraview.timer_backends -> auroraview.utils.timer_backends
    "dom",  # auroraview.dom -> auroraview.ui.dom
    "qt_integration",  # auroraview.qt_integration -> auroraview.integration.qt
    # ============================================================
    # Metadata
    # ============================================================
    "__version__",
    "__author__",
]


# ============================================================
# Auto-start WebView2 warmup on Windows (performance optimization)
# ============================================================
# Multiple WebView instances share a single pre-warmed WebView2 environment.
# By starting warmup during module import, subsequent WebView creation is fast.
# This is especially beneficial in DCC applications where startup time matters.
#
# Note: The warmup runs in a background thread and does not block imports.
# Users can still call start_warmup() manually for custom user_data_folder.
#
# Environment variables:
#   AURORAVIEW_DISABLE_WARMUP=1 - Disable auto-warmup (useful for CI/testing)
_disable_warmup = _os.environ.get("AURORAVIEW_DISABLE_WARMUP", "").lower() in ("1", "true", "yes")
if _sys.platform == "win32" and start_warmup is not None and not _disable_warmup:
    try:
        start_warmup()
    except Exception:
        # Silently ignore warmup errors - they don't affect core functionality
        pass
