# -*- coding: utf-8 -*-
"""Browser Extension Bridge for Chrome/Firefox extension integration.

This module provides a WebSocket + HTTP server that enables communication between
browser extensions (Chrome, Firefox, Edge) and AuroraView Python applications.

Architecture:
    ┌─────────────────────────────────────────────────────────────────┐
    │                    Browser Extension                             │
    │  (Content Script / Side Panel / Background Service Worker)       │
    └───────────────────────────┬─────────────────────────────────────┘
                                │ WebSocket / HTTP
                                ▼
    ┌─────────────────────────────────────────────────────────────────┐
    │                 BrowserExtensionBridge                           │
    │  - WebSocket Server (bidirectional real-time communication)      │
    │  - HTTP Server (REST API for request/response)                   │
    │  - Event routing to Python handlers                              │
    └───────────────────────────┬─────────────────────────────────────┘
                                │ Python API
                                ▼
    ┌─────────────────────────────────────────────────────────────────┐
    │              AuroraView / DCC Application                        │
    │  (Maya, Houdini, Blender, Photoshop, etc.)                      │
    └─────────────────────────────────────────────────────────────────┘

Example:
    >>> from auroraview.integration import BrowserExtensionBridge
    >>>
    >>> # Create bridge
    >>> bridge = BrowserExtensionBridge(
    ...     ws_port=49152,
    ...     http_port=49153,
    ...     allowed_origins=["chrome-extension://*"]
    ... )
    >>>
    >>> # Register handlers
    >>> @bridge.on("get_scene_info")
    ... async def handle_get_scene_info(data, client):
    ...     return {"scene": "my_scene.ma", "frame": 120}
    >>>
    >>> @bridge.on("execute_command")
    ... async def handle_execute(data, client):
    ...     cmd = data.get("command")
    ...     # Execute in DCC...
    ...     return {"success": True}
    >>>
    >>> # Start bridge
    >>> bridge.start_background()
    >>>
    >>> # From Chrome extension:
    >>> # const ws = new WebSocket("ws://localhost:49152")
    >>> # ws.send(JSON.stringify({action: "get_scene_info", data: {}}))
"""

import asyncio
import json
import logging
import threading
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import Any, Callable, Dict, List, Optional, Set
from urllib.parse import urlparse

logger = logging.getLogger(__name__)

# Check for websockets availability
try:
    import websockets

    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False


class CORSHTTPRequestHandler(BaseHTTPRequestHandler):
    """HTTP request handler with CORS support for browser extensions."""

    # Reference to the bridge instance (set by BrowserExtensionBridge)
    bridge: Optional["BrowserExtensionBridge"] = None

    def log_message(self, format: str, *args):
        """Override to use logging instead of stderr."""
        logger.debug(f"HTTP: {format % args}")

    def _set_cors_headers(self):
        """Set CORS headers for browser extension access."""
        origin = self.headers.get("Origin", "*")
        self.send_header("Access-Control-Allow-Origin", origin)
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, X-Requested-With")
        self.send_header("Access-Control-Allow-Credentials", "true")
        self.send_header("Access-Control-Max-Age", "86400")

    def _send_json_response(self, data: Dict[str, Any], status: int = 200):
        """Send JSON response with CORS headers."""
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self._set_cors_headers()
        self.end_headers()
        self.wfile.write(json.dumps(data).encode("utf-8"))

    def do_OPTIONS(self):
        """Handle CORS preflight requests."""
        self.send_response(200)
        self._set_cors_headers()
        self.end_headers()

    def do_GET(self):
        """Handle GET requests."""
        parsed = urlparse(self.path)
        path = parsed.path

        if path == "/health":
            self._send_json_response(
                {
                    "status": "ok",
                    "service": "auroraview-browser-bridge",
                    "ws_port": self.bridge.ws_port if self.bridge else None,
                    "http_port": self.bridge.http_port if self.bridge else None,
                }
            )
        elif path == "/info":
            self._send_json_response(
                {
                    "name": "AuroraView Browser Extension Bridge",
                    "version": "1.0.0",
                    "ws_url": f"ws://localhost:{self.bridge.ws_port}" if self.bridge else None,
                    "capabilities": ["websocket", "http", "events"],
                    "handlers": list(self.bridge._handlers.keys()) if self.bridge else [],
                }
            )
        else:
            self._send_json_response({"error": "Not found"}, 404)

    def do_POST(self):
        """Handle POST requests (RPC-style calls)."""
        parsed = urlparse(self.path)
        path = parsed.path

        # Read request body
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length).decode("utf-8") if content_length > 0 else "{}"

        try:
            data = json.loads(body)
        except json.JSONDecodeError:
            self._send_json_response({"error": "Invalid JSON"}, 400)
            return

        if path == "/call":
            # RPC-style call: {"action": "...", "data": {...}}
            action = data.get("action")
            if not action:
                self._send_json_response({"error": "Missing 'action' field"}, 400)
                return

            if self.bridge and action in self.bridge._handlers:
                try:
                    # Run async handler in the bridge's event loop
                    handler = self.bridge._handlers[action]

                    async def run_handler():
                        return await handler(data.get("data", {}), None)

                    if self.bridge._loop and self.bridge._loop.is_running():
                        future = asyncio.run_coroutine_threadsafe(run_handler(), self.bridge._loop)
                        result = future.result(timeout=30)  # 30 second timeout
                        self._send_json_response({"success": True, "result": result})
                    else:
                        self._send_json_response({"error": "Bridge not running"}, 503)
                except Exception as e:
                    logger.error(f"Handler error: {e}", exc_info=True)
                    self._send_json_response({"error": str(e)}, 500)
            else:
                self._send_json_response({"error": f"Unknown action: {action}"}, 404)
        else:
            self._send_json_response({"error": "Not found"}, 404)


class BrowserExtensionBridge:
    """WebSocket + HTTP Bridge for browser extension integration.

    This bridge provides two communication channels:
    1. WebSocket (ws://localhost:PORT) - Real-time bidirectional communication
    2. HTTP (http://localhost:PORT) - Request/response style API

    Args:
        ws_port: WebSocket server port (default: 49152)
        http_port: HTTP server port (default: 49153)
        host: Server host (default: "127.0.0.1")
        allowed_origins: List of allowed origins for CORS (default: ["*"])

    Example:
        >>> bridge = BrowserExtensionBridge(ws_port=49152, http_port=49153)
        >>>
        >>> @bridge.on("get_data")
        ... async def handle_get_data(data, client):
        ...     return {"items": [1, 2, 3]}
        >>>
        >>> bridge.start_background()
    """

    def __init__(
        self,
        ws_port: int = 49152,
        http_port: int = 49153,
        host: str = "127.0.0.1",
        allowed_origins: Optional[List[str]] = None,
        name: str = "AuroraView",
    ):
        if not WEBSOCKETS_AVAILABLE:
            raise ImportError(
                "websockets library is required. Install with: pip install websockets"
            )

        self.ws_port = ws_port
        self.http_port = http_port
        self.host = host
        self.allowed_origins = allowed_origins or ["*"]
        self.name = name

        self._handlers: Dict[str, Callable] = {}
        self._clients: Set[Any] = set()
        self._is_running = False
        self._loop: Optional[asyncio.AbstractEventLoop] = None
        self._ws_thread: Optional[threading.Thread] = None
        self._http_thread: Optional[threading.Thread] = None
        self._http_server: Optional[HTTPServer] = None

        logger.info(
            f"BrowserExtensionBridge initialized: ws={host}:{ws_port}, http={host}:{http_port}"
        )

    def on(self, action: str) -> Callable:
        """Decorator to register a message handler.

        Args:
            action: Action name to handle

        Returns:
            Decorator function

        Example:
            >>> @bridge.on("get_scene")
            ... async def handle_get_scene(data, client):
            ...     return {"scene_name": "untitled"}
        """

        def decorator(func: Callable) -> Callable:
            self.register_handler(action, func)
            return func

        return decorator

    def register_handler(self, action: str, handler: Callable):
        """Register a message handler.

        Args:
            action: Action name
            handler: Async function(data, client) -> response
        """
        self._handlers[action] = handler
        logger.info(f"Registered handler: {action}")

    async def _handle_ws_client(self, websocket: Any):
        """Handle WebSocket client connection."""
        client_addr = websocket.remote_address
        logger.info(f"WebSocket client connected: {client_addr}")

        self._clients.add(websocket)

        try:
            async for message in websocket:
                await self._process_ws_message(message, websocket)
        except websockets.exceptions.ConnectionClosed:
            logger.info(f"WebSocket client disconnected: {client_addr}")
        except Exception as e:
            logger.error(f"WebSocket error: {e}", exc_info=True)
        finally:
            self._clients.discard(websocket)

    async def _process_ws_message(self, message: str, websocket: Any):
        """Process incoming WebSocket message."""
        try:
            data = json.loads(message)
            action = data.get("action")

            logger.debug(f"Received action: {action}")

            if action in self._handlers:
                handler = self._handlers[action]
                result = await handler(data.get("data", {}), websocket)

                if result is not None:
                    response = {
                        "type": "response",
                        "action": action,
                        "requestId": data.get("requestId"),
                        "data": result,
                    }
                    await websocket.send(json.dumps(response))
            else:
                logger.warning(f"No handler for action: {action}")
                error_response = {
                    "type": "error",
                    "action": action,
                    "requestId": data.get("requestId"),
                    "error": f"Unknown action: {action}",
                }
                await websocket.send(json.dumps(error_response))

        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON: {e}")
        except Exception as e:
            logger.error(f"Error processing message: {e}", exc_info=True)

    async def broadcast(self, action: str, data: Dict[str, Any]):
        """Broadcast message to all connected WebSocket clients.

        Args:
            action: Action name
            data: Data to broadcast
        """
        if not self._clients:
            return

        message = json.dumps(
            {
                "type": "event",
                "action": action,
                "data": data,
            }
        )

        await asyncio.gather(
            *[client.send(message) for client in self._clients], return_exceptions=True
        )

        logger.debug(f"Broadcast '{action}' to {len(self._clients)} clients")

    def emit(self, action: str, data: Dict[str, Any]):
        """Emit event to all connected clients (non-blocking).

        This is a convenience method for sending events from synchronous code.

        Args:
            action: Event action name
            data: Event data
        """
        if self._loop and self._loop.is_running():
            asyncio.run_coroutine_threadsafe(self.broadcast(action, data), self._loop)
        else:
            logger.warning("Bridge not running, cannot emit event")

    async def _start_ws_server(self):
        """Start WebSocket server."""
        logger.info(f"Starting WebSocket server on ws://{self.host}:{self.ws_port}")

        async with websockets.serve(
            self._handle_ws_client,
            self.host,
            self.ws_port,
            # Allow connections from any origin (browser extensions)
            origins=None,
        ):
            logger.info(f"WebSocket server listening on ws://{self.host}:{self.ws_port}")
            await asyncio.Future()  # Run forever

    def _start_http_server(self):
        """Start HTTP server in a separate thread."""

        # Create custom handler class with bridge reference
        class Handler(CORSHTTPRequestHandler):
            bridge = self

        self._http_server = HTTPServer((self.host, self.http_port), Handler)
        logger.info(f"HTTP server listening on http://{self.host}:{self.http_port}")
        self._http_server.serve_forever()

    def start_background(self):
        """Start both WebSocket and HTTP servers in background threads.

        This is the recommended way to start the bridge. Both servers run
        in daemon threads and stop when the main program exits.
        """
        if self._is_running:
            logger.warning("Bridge is already running")
            return

        self._is_running = True

        # Start WebSocket server thread
        def run_ws():
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            self._loop = loop
            try:
                loop.run_until_complete(self._start_ws_server())
            except Exception as e:
                logger.error(f"WebSocket server error: {e}", exc_info=True)

        self._ws_thread = threading.Thread(target=run_ws, daemon=True)
        self._ws_thread.start()

        # Start HTTP server thread
        self._http_thread = threading.Thread(target=self._start_http_server, daemon=True)
        self._http_thread.start()

        # Wait for servers to start
        time.sleep(0.5)

        logger.info(
            f"Bridge started: ws://{self.host}:{self.ws_port}, http://{self.host}:{self.http_port}"
        )

    def stop(self):
        """Stop both servers."""
        logger.info("Stopping bridge...")

        if self._http_server:
            self._http_server.shutdown()

        self._is_running = False
        logger.info("Bridge stopped")

    @property
    def is_running(self) -> bool:
        """Check if bridge is running."""
        return self._is_running

    @property
    def client_count(self) -> int:
        """Get number of connected WebSocket clients."""
        return len(self._clients)

    def get_status(self) -> Dict[str, Any]:
        """Get the current status of the bridge.

        Returns:
            Dictionary with status information including:
            - is_running: Whether the bridge is running
            - ws_port: WebSocket server port
            - http_port: HTTP server port
            - connected_clients: Number of connected WebSocket clients
            - name: Bridge name
        """
        return {
            "is_running": self._is_running,
            "ws_port": self.ws_port,
            "http_port": self.http_port,
            "connected_clients": len(self._clients),
            "name": self.name,
            "handlers": list(self._handlers.keys()),
        }

    def __repr__(self) -> str:
        status = "running" if self._is_running else "stopped"
        return f"BrowserExtensionBridge(ws={self.ws_port}, http={self.http_port}, {status}, clients={self.client_count})"


# Convenience function to create and start bridge
def create_browser_bridge(
    ws_port: int = 49152,
    http_port: int = 49153,
    auto_start: bool = True,
) -> BrowserExtensionBridge:
    """Create and optionally start a browser extension bridge.

    Args:
        ws_port: WebSocket server port
        http_port: HTTP server port
        auto_start: Start servers immediately

    Returns:
        BrowserExtensionBridge instance

    Example:
        >>> bridge = create_browser_bridge()
        >>>
        >>> @bridge.on("get_data")
        ... async def handle(data, client):
        ...     return {"result": "ok"}
    """
    bridge = BrowserExtensionBridge(ws_port=ws_port, http_port=http_port)
    if auto_start:
        bridge.start_background()
    return bridge
