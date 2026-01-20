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

import errno
import json
import os
import signal
import sys
import threading
import time
from typing import TYPE_CHECKING, Any, Callable, Dict, Optional

if TYPE_CHECKING:
    from .webview import WebView

# Check if running in packed mode (set by Rust CLI)
PACKED_MODE = os.environ.get("AURORAVIEW_PACKED", "0") == "1"

# Windows error codes
_ERROR_NO_DATA = 232  # Pipe is being closed
_ERROR_BROKEN_PIPE = 109  # The pipe has been ended

# Pipe error codes (POSIX + Windows)
_PIPE_ERRORS = (errno.EINVAL, errno.EPIPE, errno.EBADF)
_PIPE_WIN_ERRORS = (_ERROR_NO_DATA, _ERROR_BROKEN_PIPE)


class StdioWriter:
    """Thread-safe stdout writer with pipe closure detection.

    This class provides a robust way to write to stdout in packed mode,
    automatically detecting when the pipe is closed and preventing
    further write attempts to avoid cascading errors.
    """

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._closed = False

    def is_closed(self) -> bool:
        """Check if the pipe has been detected as closed."""
        return self._closed

    def write(self, data: str) -> bool:
        """Write data to stdout.

        Args:
            data: The string data to write

        Returns:
            True if write succeeded, False if pipe is closed
        """
        if self._closed:
            return False

        with self._lock:
            if self._closed:
                return False

            try:
                print(data, flush=True)
                return True
            except OSError as e:
                if self._is_pipe_error(e):
                    self._closed = True
                    return False
                raise

    def write_json(self, obj: Dict[str, Any]) -> bool:
        """Write a JSON object to stdout.

        Args:
            obj: The dictionary to JSON serialize and write

        Returns:
            True if write succeeded, False if pipe is closed
        """
        return self.write(json.dumps(obj, ensure_ascii=False))

    @staticmethod
    def _is_pipe_error(e: OSError) -> bool:
        """Check if an OSError indicates a closed/broken pipe."""
        winerror = getattr(e, "winerror", None)
        if winerror in _PIPE_WIN_ERRORS:
            return True
        if e.errno in _PIPE_ERRORS:
            return True
        return False


# Global writer instance for packed mode
_stdout_writer: Optional[StdioWriter] = None


def _get_writer() -> StdioWriter:
    """Get or create the global stdout writer."""
    global _stdout_writer
    if _stdout_writer is None:
        _stdout_writer = StdioWriter()
    return _stdout_writer


def is_packed_mode() -> bool:
    """Check if running in packed mode."""
    return PACKED_MODE


def is_pipe_closed() -> bool:
    """Check if the stdout pipe has been closed.

    This can be used to detect when the Rust process has terminated
    and avoid attempting further communication.
    """
    if not is_packed_mode():
        return False
    return _get_writer().is_closed()


def send_command(command: Dict[str, Any]) -> bool:
    """Send a command to the Rust backend via stdout.

    This is used for fire-and-forget commands like set_html that don't
    expect a response in the JSON-RPC request/response cycle.

    Args:
        command: The command dictionary to send (will be JSON serialized)

    Returns:
        True if command was sent, False if pipe is closed
    """
    if not is_packed_mode():
        return True

    return _get_writer().write_json(command)


def send_set_html(html: str, title: Optional[str] = None) -> bool:
    """Send HTML content to the Rust WebView for dynamic loading.

    This allows Python components like Browser to dynamically set HTML
    content in packed mode, where Rust controls the WebView.

    Args:
        html: The HTML content to load
        title: Optional window title to set

    Returns:
        True if command was sent, False if pipe is closed
    """
    command: Dict[str, Any] = {
        "type": "set_html",
        "html": html,
    }
    if title is not None:
        command["title"] = title

    print(f"[AuroraView] Sending set_html command (html_len: {len(html)})", file=sys.stderr)
    return send_command(command)


def send_event(event: str, data: Optional[Dict[str, Any]] = None) -> bool:
    """Send an event to the Rust WebView in packed mode.

    This is used by WebView.emit() to forward events to the Rust CLI,
    which then triggers them in the WebView via JavaScript.

    The Rust CLI reads this from Python's stdout and forwards it to
    the WebView using `window.auroraview.trigger()`.

    Args:
        event: Event name
        data: Event data (will be JSON serialized)

    Returns:
        True if event was sent, False if pipe is closed
    """
    if not is_packed_mode():
        return True

    message: Dict[str, Any] = {
        "type": "event",
        "event": event,
        "data": data or {},
    }
    return _get_writer().write_json(message)


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

    # Get the global writer for consistent pipe state tracking
    writer = _get_writer()

    # Get the bound functions from the WebView
    bound_functions = getattr(webview, "_bound_functions", {})
    print(f"[AuroraView] Registered {len(bound_functions)} API handlers", file=sys.stderr)

    # Setup signal handler for graceful shutdown
    running = True
    stop_event = threading.Event()

    def signal_handler(signum: int, frame: Any) -> None:
        nonlocal running
        print("[AuroraView] Received shutdown signal", file=sys.stderr)
        running = False
        stop_event.set()

    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    # Send ready signal to Rust backend
    # This tells Rust that Python is ready to receive requests
    ready_signal = {"type": "ready", "handlers": list(bound_functions.keys())}
    if not writer.write_json(ready_signal):
        print("[AuroraView] Failed to send ready signal, pipe closed", file=sys.stderr)
        return
    print("[AuroraView] Ready signal sent", file=sys.stderr)
    send_event("backend_health", {"status": "ready", "schema_version": 1})

    def heartbeat_loop() -> None:
        """Send periodic heartbeat events to Rust."""
        while not stop_event.is_set() and not writer.is_closed():
            if not send_event(
                "backend_health", {"status": "alive", "ts": time.time(), "schema_version": 1}
            ):
                # Pipe closed, signal main loop to stop
                print("[AuroraView] Heartbeat: pipe closed, stopping", file=sys.stderr)
                stop_event.set()
                break
            stop_event.wait(2.0)

    heartbeat_thread = threading.Thread(
        target=heartbeat_loop,
        name="AuroraViewHeartbeat",
        daemon=True,
    )
    heartbeat_thread.start()

    # Main JSON-RPC loop
    while running and not writer.is_closed():
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

            # Send response - break if pipe is closed
            if not writer.write_json(response):
                print("[AuroraView] stdout closed, shutting down", file=sys.stderr)
                break

        except OSError as e:
            # Check for pipe errors
            winerror = getattr(e, "winerror", None)
            if winerror in _PIPE_WIN_ERRORS or e.errno in _PIPE_ERRORS:
                print(f"[AuroraView] Pipe closed, shutting down: {e}", file=sys.stderr)
                break
            print(f"[AuroraView] OSError in API server loop: {e}", file=sys.stderr)
            continue
        except Exception as e:
            print(f"[AuroraView] Error in API server loop: {e}", file=sys.stderr)
            continue

    stop_event.set()

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

    if method == "__ping__":
        return {
            "id": call_id,
            "ok": True,
            "result": {
                "status": "pong",
                "ts": time.time(),
                "schema_version": 1,
            },
        }

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
