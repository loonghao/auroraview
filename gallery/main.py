#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""AuroraView Gallery - Interactive showcase of all features and components.

This gallery uses a React frontend built with Vite and displays via AuroraView.
It demonstrates the full capabilities of AuroraView including:
- Rust-powered plugin system with IPC
- Process management with stdout/stderr streaming
- Native desktop integration
- Browser extension bridge for Chrome/Firefox communication

Usage:
    python gallery/main.py
    # or via just:
    just gallery

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import atexit
import json
import os
import signal
import sys
import threading
import time

# Import backend modules
from backend import (
    CATEGORIES,
    EXAMPLES_DIR,
    GALLERY_DIR,
    PROJECT_ROOT,
    get_samples_list,
)
from backend.config import DIST_DIR
from backend.child_api import register_child_apis
from backend.child_manager import cleanup_manager as cleanup_child_manager
from backend.extension_api import cleanup_extension_bridge, register_extension_apis
from backend.process_api import register_process_apis
from backend.webview_extension_api import register_webview_extension_apis

sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))

from auroraview import PluginManager, WebView

# Set restart command for shell plugin
# This enables proper restart when running as Python script
_main_script = os.path.abspath(__file__)
os.environ["AURORAVIEW_RESTART_CMD"] = f'"{sys.executable}" "{_main_script}"'

# Global reference to PluginManager for cleanup
_global_plugins: PluginManager | None = None
_cleanup_done = False


def _cleanup_all():
    """Cleanup all processes and resources. Called on exit."""
    global _cleanup_done
    if _cleanup_done:
        return
    _cleanup_done = True
    
    print("[Gallery] Running cleanup...", file=sys.stderr)
    
    # Kill all managed processes
    if _global_plugins is not None:
        try:
            result = _global_plugins.handle_command("plugin:process|kill_all", "{}")
            print(f"[Gallery] kill_all result: {result}", file=sys.stderr)
        except Exception as e:
            print(f"[Gallery] Error killing processes: {e}", file=sys.stderr)
    
    # Cleanup extension bridge
    try:
        cleanup_extension_bridge()
    except Exception as e:
        print(f"[Gallery] Error cleaning up extension bridge: {e}", file=sys.stderr)
    
    # Cleanup child window manager
    try:
        cleanup_child_manager()
    except Exception as e:
        print(f"[Gallery] Error cleaning up child manager: {e}", file=sys.stderr)
    
    print("[Gallery] Cleanup complete", file=sys.stderr)


def _signal_handler(signum, frame):
    """Handle termination signals."""
    print(f"[Gallery] Received signal {signum}, cleaning up...", file=sys.stderr)
    _cleanup_all()
    sys.exit(0)


# Register cleanup handlers
atexit.register(_cleanup_all)

# Register signal handlers for graceful shutdown
if sys.platform != "win32":
    signal.signal(signal.SIGTERM, _signal_handler)
    signal.signal(signal.SIGHUP, _signal_handler)
# SIGINT (Ctrl+C) is handled by Python's default handler


def run_gallery():
    """Run the AuroraView Gallery application.

    In packed mode, WebView.show() automatically switches to API server mode.
    No special handling needed - the framework handles it transparently.
    """
    print("[Python] Starting AuroraView Gallery...", file=sys.stderr)
    print("[Python] " + "=" * 50, file=sys.stderr)
    print("[Python] Interactive showcase of all features and components", file=sys.stderr)
    print("[Python] " + "=" * 50, file=sys.stderr)

    # Check if running in packed mode
    from auroraview.core.packed import is_packed_mode

    packed_mode = is_packed_mode()
    print(f"[Python] Packed mode: {packed_mode}", file=sys.stderr)

    # Pre-load samples during startup to avoid delay when frontend requests them
    start_time = time.time()
    samples = get_samples_list()
    elapsed = (time.time() - start_time) * 1000
    print(f"[Python] Pre-loaded {len(samples)} samples in {elapsed:.1f}ms", file=sys.stderr)

    # Check if dist exists (only needed in development mode)
    index_html = DIST_DIR / "index.html"
    if not index_html.exists():
        if not packed_mode:
            print(
                "[Python] Error: Gallery not built. Run 'just gallery-build' first.",
                file=sys.stderr,
            )
            print(f"[Python] Expected: {index_html}", file=sys.stderr)
            sys.exit(1)
        url = None  # Packed mode doesn't need URL
    else:
        url = str(index_html)
        print(f"[Python] Loading: {url}", file=sys.stderr)

    print(f"[Python] Creating WebView with allow_new_window=True, new_window_mode='child_webview'", file=sys.stderr)
    view = WebView(
        title="AuroraView Gallery",
        url=url,
        width=1200,
        height=800,
        debug=True,
        allow_new_window=True,
        new_window_mode="child_webview",  # Open new windows as child WebViews
    )
    print(f"[Python] WebView created successfully", file=sys.stderr)

    # Create plugin manager with permissive scope for demo
    plugins = PluginManager.permissive()
    
    # Store global reference for cleanup
    global _global_plugins
    _global_plugins = plugins

    # Set up event callback for plugins
    if packed_mode:
        _stdout_lock = threading.Lock()

        def packed_emit_callback(event_name, data):
            """Emit events to Rust CLI via stdout in packed mode."""
            try:
                event_msg = json.dumps({
                    "type": "event",
                    "event": event_name,
                    "data": data if isinstance(data, dict) else {"value": data}
                })
                with _stdout_lock:
                    print(event_msg, flush=True)
                print(f"[Python:emit] Sent event via stdout: {event_name}", file=sys.stderr)
            except Exception as e:
                print(f"[Python:emit] Error sending event: {e}", file=sys.stderr)

        plugins.set_emit_callback(packed_emit_callback)
        print("[Python] Using packed mode event callback (stdout)", file=sys.stderr)
    else:
        emitter = view.create_emitter()
        plugins.set_emit_callback(emitter.emit)
        print("[Python] Using development mode event callback (message queue)", file=sys.stderr)

    # Connect plugin manager to WebView
    view.set_plugin_manager(plugins)
    print("[Python] Plugin manager connected to WebView", file=sys.stderr)

    # Register all API handlers from backend modules
    api_refs = register_process_apis(view, plugins)
    register_webview_extension_apis(view)

    # Get run_sample function reference for extension bridge
    run_sample_func = api_refs.get("run_sample")
    register_extension_apis(view, run_sample_func)

    # Register child window APIs
    register_child_apis(view)

    # Register simple API handlers
    @view.bind_call("api.get_samples")
    def get_samples() -> list:
        """Get all samples."""
        return get_samples_list()

    @view.bind_call("api.get_categories")
    def get_categories() -> dict:
        """Get all categories."""
        return CATEGORIES

    # Handle native file drop events
    @view.on("file_drop")
    def on_file_drop(data):
        """Handle native file drop events from Wry/WebView."""
        paths = data.get("paths", [])
        position = data.get("position", {})
        print(f"[Python:on_file_drop] Received file drop: paths={paths}, position={position}", file=sys.stderr)

        if paths:
            emitter = view.create_emitter()
            emitter.emit("extension:file_drop", {
                "paths": paths,
                "position": position,
            })
            print("[Python:on_file_drop] Emitted extension:file_drop event to frontend", file=sys.stderr)

    # Cleanup on close - use "closing" event (before window closes)
    @view.on("closing")
    def on_closing():
        _cleanup_all()

    # Show the gallery
    view.show()


if __name__ == "__main__":
    run_gallery()
