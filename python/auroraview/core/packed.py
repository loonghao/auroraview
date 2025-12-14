# -*- coding: utf-8 -*-
"""Packed mode support for AuroraView.

In packed mode, the Rust CLI creates the WebView window and loads the frontend.
The Python backend runs as a headless JSON-RPC API server, communicating with
the Rust process via stdin/stdout.

This module provides transparent packed mode support - developers write normal
WebView code, and the framework automatically switches to API server mode when
running in a packed application.
"""

from __future__ import annotations

import json
import os
import signal
import sys
from typing import TYPE_CHECKING, Any, Callable, Dict

if TYPE_CHECKING:
    from .webview import WebView

# Check if running in packed mode (set by Rust CLI)
PACKED_MODE = os.environ.get("AURORAVIEW_PACKED", "0") == "1"


def is_packed_mode() -> bool:
    """Check if running in packed mode."""
    return PACKED_MODE


def run_api_server(webview: "WebView") -> None:
    """Run the WebView as a headless JSON-RPC API server.

    This function is called automatically by WebView.show() when running
    in packed mode. It replaces the normal WebView window with a JSON-RPC
    server that handles API calls from the Rust frontend.

    All handlers registered via bind_call() are automatically available
    as API endpoints.

    Args:
        webview: The WebView instance with registered handlers
    """
    print("[AuroraView] Running in packed mode (API server)", file=sys.stderr)

    # Get the bound functions from the WebView
    bound_functions = getattr(webview, "_bound_functions", {})
    print(f"[AuroraView] Registered {len(bound_functions)} API handlers", file=sys.stderr)

    # Setup signal handler for graceful shutdown
    running = True

    def signal_handler(signum: int, frame: Any) -> None:
        nonlocal running
        print("[AuroraView] Received shutdown signal", file=sys.stderr)
        running = False

    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    # Send ready signal to Rust backend
    # This tells Rust that Python is ready to receive requests
    ready_signal = json.dumps({"type": "ready", "handlers": list(bound_functions.keys())})
    print(ready_signal, flush=True)
    print("[AuroraView] Ready signal sent", file=sys.stderr)

    # Main JSON-RPC loop
    while running:
        try:
            line = sys.stdin.readline()
            if not line:
                # EOF - parent process closed stdin
                print("[AuroraView] stdin closed, shutting down", file=sys.stderr)
                break

            line = line.strip()
            if not line:
                continue

            # Parse JSON-RPC request
            try:
                request = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"[AuroraView] Invalid JSON: {e}", file=sys.stderr)
                continue

            # Handle the request
            response = _handle_request(request, bound_functions)

            # Send response
            print(json.dumps(response), flush=True)

        except Exception as e:
            print(f"[AuroraView] Error in API server loop: {e}", file=sys.stderr)
            continue

    # Trigger close event if registered
    close_handlers = getattr(webview, "_event_handlers", {}).get("close", [])
    for handler in close_handlers:
        try:
            handler()
        except Exception as e:
            print(f"[AuroraView] Error in close handler: {e}", file=sys.stderr)

    print("[AuroraView] API server stopped", file=sys.stderr)


def _handle_request(
    request: Dict[str, Any],
    bound_functions: Dict[str, Callable[..., Any]],
) -> Dict[str, Any]:
    """Handle a single JSON-RPC request.

    Args:
        request: The JSON-RPC request object
        bound_functions: Dictionary of registered API handlers

    Returns:
        JSON-RPC response object
    """
    call_id = request.get("id", "")
    method = request.get("method", "")
    params = request.get("params")

    # Find the handler
    handler = bound_functions.get(method)
    if handler is None:
        return {
            "id": call_id,
            "ok": False,
            "error": {
                "name": "MethodNotFound",
                "message": f"Method not found: {method}",
            },
        }

    # Call the handler
    try:
        if params is None:
            result = handler()
        elif isinstance(params, dict):
            result = handler(**params)
        elif isinstance(params, list):
            result = handler(*params)
        else:
            result = handler(params)

        return {
            "id": call_id,
            "ok": True,
            "result": result,
        }

    except Exception as e:
        return {
            "id": call_id,
            "ok": False,
            "error": {
                "name": type(e).__name__,
                "message": str(e),
            },
        }
