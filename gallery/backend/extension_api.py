"""Browser extension bridge API handlers for AuroraView Gallery.

This module provides API handlers for:
- Starting/stopping the browser extension bridge server
- Getting extension bridge status
- Broadcasting events to connected extensions
- Installing browser extensions
"""

from __future__ import annotations

import os
import subprocess
import sys
import threading
import webbrowser
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from auroraview import WebView

from .config import CATEGORIES
from .samples import get_sample_by_id, get_samples_list, get_source_code

# Browser Extension Bridge (global instance)
_extension_bridge = None
_extension_bridge_lock = threading.Lock()


def register_extension_apis(view: WebView, run_sample_func) -> None:
    """Register all browser extension bridge API handlers.

    Args:
        view: The WebView instance to register handlers on
        run_sample_func: Reference to the run_sample function for extension commands
    """
    global _extension_bridge

    @view.bind_call("api.start_extension_bridge")
    def start_extension_bridge(ws_port: int = 49152, http_port: int = 49153) -> dict:
        """Start the browser extension bridge server.

        This creates a WebSocket + HTTP server that browser extensions can connect to.
        Uses high port range (49152-65535) to avoid conflicts.
        """
        global _extension_bridge
        print(
            f"[Python:start_extension_bridge] ws_port={ws_port}, http_port={http_port}",
            file=sys.stderr,
        )

        with _extension_bridge_lock:
            if _extension_bridge is not None:
                return {"ok": False, "error": "Extension bridge is already running"}

            try:
                from auroraview.integration import BrowserExtensionBridge

                _extension_bridge = BrowserExtensionBridge(
                    ws_port=ws_port, http_port=http_port, name="AuroraView Gallery"
                )

                # Register handlers for extension commands
                @_extension_bridge.on("get_samples")
                async def handle_get_samples(data, client_id):
                    """Return available samples to browser extension."""
                    return {"samples": get_samples_list()}

                @_extension_bridge.on("get_categories")
                async def handle_get_categories(data, client_id):
                    """Return categories to browser extension."""
                    return {"categories": CATEGORIES}

                @_extension_bridge.on("run_sample")
                async def handle_run_sample(data, client_id):
                    """Run a sample from browser extension request."""
                    sample_id = data.get("sample_id", "")
                    show_console = data.get("show_console", False)
                    result = run_sample_func(sample_id=sample_id, show_console=show_console)
                    return result

                @_extension_bridge.on("get_source")
                async def handle_get_source(data, client_id):
                    """Get source code for browser extension."""
                    sample_id = data.get("sample_id", "")
                    sample = get_sample_by_id(sample_id)
                    if sample:
                        return {"source": get_source_code(sample["source_file"])}
                    return {"error": f"Sample not found: {sample_id}"}

                # Start the bridge in background
                _extension_bridge.start_background()

                print(
                    f"[Python:start_extension_bridge] SUCCESS: Bridge started on ws:{ws_port}, http:{http_port}",
                    file=sys.stderr,
                )
                return {
                    "ok": True,
                    "ws_port": ws_port,
                    "http_port": http_port,
                    "message": f"Extension bridge started on ws://127.0.0.1:{ws_port}",
                }
            except ImportError as e:
                error_msg = f"BrowserExtensionBridge not available: {e}"
                print(f"[Python:start_extension_bridge] ERROR: {error_msg}", file=sys.stderr)
                return {"ok": False, "error": error_msg}
            except Exception as e:
                error_msg = f"Failed to start extension bridge: {e}"
                print(f"[Python:start_extension_bridge] ERROR: {error_msg}", file=sys.stderr)
                import traceback

                traceback.print_exc(file=sys.stderr)
                return {"ok": False, "error": error_msg}

    @view.bind_call("api.stop_extension_bridge")
    def stop_extension_bridge() -> dict:
        """Stop the browser extension bridge server."""
        global _extension_bridge
        print("[Python:stop_extension_bridge] Stopping bridge...", file=sys.stderr)

        with _extension_bridge_lock:
            if _extension_bridge is None:
                return {"ok": False, "error": "Extension bridge is not running"}

            try:
                _extension_bridge.stop()
                _extension_bridge = None
                print(
                    "[Python:stop_extension_bridge] SUCCESS: Bridge stopped",
                    file=sys.stderr,
                )
                return {"ok": True, "message": "Extension bridge stopped"}
            except Exception as e:
                error_msg = f"Failed to stop extension bridge: {e}"
                print(f"[Python:stop_extension_bridge] ERROR: {error_msg}", file=sys.stderr)
                return {"ok": False, "error": error_msg}

    @view.bind_call("api.get_extension_status")
    def get_extension_status() -> dict:
        """Get the status of the browser extension bridge."""
        global _extension_bridge

        with _extension_bridge_lock:
            if _extension_bridge is None:
                return {
                    "enabled": False,
                    "wsPort": 49152,
                    "httpPort": 49153,
                    "connectedClients": 0,
                    "isRunning": False,
                }

            try:
                status = _extension_bridge.get_status()
                return {
                    "enabled": True,
                    "wsPort": status.get("ws_port", 49152),
                    "httpPort": status.get("http_port", 49153),
                    "connectedClients": status.get("connected_clients", 0),
                    "isRunning": status.get("is_running", False),
                }
            except Exception as e:
                print(f"[Python:get_extension_status] ERROR: {e}", file=sys.stderr)
                return {
                    "enabled": False,
                    "wsPort": 49152,
                    "httpPort": 49153,
                    "connectedClients": 0,
                    "isRunning": False,
                }

    @view.bind_call("api.broadcast_to_extensions")
    def broadcast_to_extensions(event: str = "", data: dict = None) -> dict:
        """Broadcast an event to all connected browser extensions."""
        global _extension_bridge
        print(f"[Python:broadcast_to_extensions] event={event}", file=sys.stderr)

        with _extension_bridge_lock:
            if _extension_bridge is None:
                return {"ok": False, "error": "Extension bridge is not running"}

            try:
                _extension_bridge.broadcast(event, data or {})
                return {"ok": True}
            except Exception as e:
                error_msg = f"Failed to broadcast: {e}"
                print(
                    f"[Python:broadcast_to_extensions] ERROR: {error_msg}",
                    file=sys.stderr,
                )
                return {"ok": False, "error": error_msg}

    @view.bind_call("api.install_extension")
    def install_extension(path: str = "", browser: str = "chrome") -> dict:
        """Install a browser extension from a local file or folder.

        For packaged extensions (.crx/.xpi), opens the browser's extension page.
        For unpacked folders (development), opens the extensions page and the folder.

        Args:
            path: Path to the extension file (.crx/.xpi) or unpacked folder
            browser: "chrome" or "firefox"
        """
        print(
            f"[Python:install_extension] path={path}, browser={browser}",
            file=sys.stderr,
        )

        if not path:
            return {"ok": False, "error": "No path provided"}

        ext_path = Path(path)
        if not ext_path.exists():
            error_msg = f"Extension path not found: {path}"
            print(f"[Python:install_extension] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

        # Get absolute path
        abs_path = str(ext_path.resolve())
        print(f"[Python:install_extension] Absolute path: {abs_path}", file=sys.stderr)

        # Check if it's a directory (unpacked/development extension)
        is_folder = ext_path.is_dir()
        print(f"[Python:install_extension] Is folder: {is_folder}", file=sys.stderr)

        try:
            if is_folder:
                # Development version folder
                if browser == "firefox":
                    # Firefox: Open about:debugging for temporary add-ons
                    webbrowser.open("about:debugging#/runtime/this-firefox")
                    # Also open the folder so user can select manifest.json
                    if sys.platform == "win32":
                        os.startfile(abs_path)
                    elif sys.platform == "darwin":
                        subprocess.run(["open", abs_path], check=True)
                    else:
                        subprocess.run(["xdg-open", abs_path], check=True)

                    message = "Firefox debugging page opened. Click 'Load Temporary Add-on' and select manifest.json from the opened folder."
                else:
                    # Chrome: Open extensions page in developer mode
                    webbrowser.open("chrome://extensions/")
                    # Open the folder so user can load unpacked
                    if sys.platform == "win32":
                        os.startfile(abs_path)
                    elif sys.platform == "darwin":
                        subprocess.run(["open", abs_path], check=True)
                    else:
                        subprocess.run(["xdg-open", abs_path], check=True)

                    message = "Chrome extensions page opened. Enable 'Developer mode', click 'Load unpacked' and select the opened folder."

                print(
                    f"[Python:install_extension] SUCCESS (folder): {message}",
                    file=sys.stderr,
                )
                return {
                    "ok": True,
                    "success": True,
                    "path": abs_path,
                    "browser": browser,
                    "isFolder": True,
                    "message": message,
                }
            else:
                # Packaged extension file
                ext = ext_path.suffix.lower()
                expected_ext = ".xpi" if browser == "firefox" else ".crx"

                if ext != expected_ext:
                    error_msg = (
                        f"Invalid extension file. Expected {expected_ext} for {browser}, got {ext}"
                    )
                    print(f"[Python:install_extension] ERROR: {error_msg}", file=sys.stderr)
                    return {"ok": False, "error": error_msg}

                if browser == "firefox":
                    # Firefox: Open the XPI file directly
                    webbrowser.open(f"file://{abs_path}")
                    message = "Firefox extension installation dialog opened"
                else:
                    # Chrome: Open extensions page (Chrome doesn't allow direct CRX installation)
                    webbrowser.open("chrome://extensions/")
                    # Open the folder containing the extension
                    parent_dir = str(ext_path.parent.resolve())
                    if sys.platform == "win32":
                        os.startfile(parent_dir)
                    elif sys.platform == "darwin":
                        subprocess.run(["open", parent_dir], check=True)
                    else:
                        subprocess.run(["xdg-open", parent_dir], check=True)

                    message = "Chrome extensions page opened. Please drag the .crx file to install."

                print(
                    f"[Python:install_extension] SUCCESS (file): {message}",
                    file=sys.stderr,
                )
                return {
                    "ok": True,
                    "success": True,
                    "path": abs_path,
                    "browser": browser,
                    "isFolder": False,
                    "message": message,
                }
        except Exception as e:
            error_msg = f"Failed to install extension: {e}"
            print(f"[Python:install_extension] EXCEPTION: {error_msg}", file=sys.stderr)
            import traceback

            traceback.print_exc(file=sys.stderr)
            return {"ok": False, "error": error_msg}


def cleanup_extension_bridge() -> None:
    """Cleanup the extension bridge on application close."""
    global _extension_bridge

    with _extension_bridge_lock:
        if _extension_bridge is not None:
            try:
                _extension_bridge.stop()
                _extension_bridge = None
            except Exception:
                pass
