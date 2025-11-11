"""WebSocket Bridge for DCC and Web application integration.

This module provides a generic WebSocket server that can be used to integrate
AuroraView with any DCC tool or web application that supports WebSocket communication.

Example:
    >>> from auroraview import WebView, Bridge
    >>>
    >>> # Create Bridge with decorator API
    >>> bridge = Bridge(port=9001)
    >>>
    >>> @bridge.on('layer_created')
    >>> async def handle_layer(data, client):
    ...     print(f"Layer created: {data}")
    ...     return {"status": "ok"}
    >>>
    >>> # Create WebView with Bridge
    >>> webview = WebView.create("My Tool", bridge=bridge)
    >>> webview.show()
"""

import asyncio
import json
import logging
import threading
from typing import Any, Callable, Dict, Optional, Set

try:
    import websockets
    from websockets.server import WebSocketServerProtocol

    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False
    WebSocketServerProtocol = None  # type: ignore

logger = logging.getLogger(__name__)


class Bridge:
    """WebSocket Bridge for DCC and Web application integration.

    This class provides a WebSocket server that can communicate with external
    applications (Photoshop, Maya, Blender, etc.) and automatically integrates
    with AuroraView WebView for bidirectional communication.

    Args:
        host: WebSocket server host (default: "localhost")
        port: WebSocket server port (0 = auto-allocate, default: 9001)
        auto_start: Auto-start server on creation (default: False)
        protocol: Message protocol - 'json' or 'msgpack' (default: 'json')
        service_discovery: Enable service discovery (default: False)
        discovery_port: HTTP discovery port (default: 9000)
        enable_mdns: Enable mDNS service discovery (default: True)

    Example:
        >>> # Auto-allocate port with service discovery
        >>> bridge = Bridge(port=0, service_discovery=True)
        >>> print(f"Bridge port: {bridge.port}")
        >>>
        >>> @bridge.on('handshake')
        >>> async def handle_handshake(data, client):
        ...     return {"server": "auroraview", "version": "1.0.0"}
        >>>
        >>> # Start manually
        >>> await bridge.start()
        >>>
        >>> # Or use with WebView (auto-start)
        >>> webview = WebView.create("Tool", bridge=bridge)
        >>> webview.show()
    """

    def __init__(
        self,
        host: str = "localhost",
        port: int = 9001,
        *,
        auto_start: bool = False,
        protocol: str = "json",
        service_discovery: bool = False,
        discovery_port: int = 9000,
        enable_mdns: bool = True,
    ):
        """Initialize the Bridge.

        Args:
            host: WebSocket server host
            port: WebSocket server port (0 = auto-allocate)
            auto_start: Auto-start server on creation
            protocol: Message protocol ('json' or 'msgpack')
            service_discovery: Enable service discovery
            discovery_port: HTTP discovery port
            enable_mdns: Enable mDNS service discovery
        """
        if not WEBSOCKETS_AVAILABLE:
            raise ImportError(
                "websockets library is required for Bridge. Install with: pip install websockets"
            )

        # Service discovery
        self._service_discovery = None
        if service_discovery:
            try:
                from ._core import ServiceDiscovery

                self._service_discovery = ServiceDiscovery(
                    bridge_port=port,
                    discovery_port=discovery_port,
                    enable_mdns=enable_mdns,
                )
                # Use allocated port
                port = self._service_discovery.bridge_port
                logger.info(
                    f"Service discovery enabled: bridge_port={port}, discovery_port={discovery_port}"
                )
            except ImportError as e:
                logger.warning(f"Service discovery not available: {e}")
            except Exception as e:
                logger.error(f"Failed to initialize service discovery: {e}")

        self.host = host
        self.port = port
        self.protocol = protocol
        self._clients: Set[WebSocketServerProtocol] = set()
        self._handlers: Dict[str, Callable] = {}
        self._webview_callback: Optional[Callable] = None
        self._server = None
        self._server_task: Optional[asyncio.Task] = None
        self._is_running = False
        self._loop: Optional[asyncio.AbstractEventLoop] = None
        self._thread: Optional[threading.Thread] = None

        logger.info(f"Bridge initialized: {self.host}:{self.port} (protocol={self.protocol})")

        if auto_start:
            self.start_background()

    def on(self, action: str) -> Callable:
        """Decorator to register a message handler.

        Args:
            action: Action name (e.g., 'layer_created', 'handshake')

        Returns:
            Decorator function

        Example:
            >>> @bridge.on('layer_created')
            >>> async def handle_layer(data, client):
            ...     print(f"Layer: {data}")
            ...     return {"status": "ok"}
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
                    - data: Message data dict
                    - client: WebSocket client connection
                    - return: Response dict (optional)
        """
        self._handlers[action] = handler
        logger.info(f"Registered handler for action: '{action}'")

    def set_webview_callback(self, callback: Callable):
        """Set callback to communicate with WebView UI.

        This is called automatically when Bridge is associated with a WebView.

        Args:
            callback: Function(action, data, result) to call when UI needs update
        """
        self._webview_callback = callback
        logger.debug("WebView callback registered")

    async def start(self):
        """Start the WebSocket server (blocking).

        This method blocks until the server is stopped. Use start_background()
        for non-blocking operation.

        Example:
            >>> await bridge.start()  # Blocks forever
        """
        logger.info(f"🚀 Starting Bridge on {self.host}:{self.port}")

        # Start service discovery if enabled
        if self._service_discovery:
            try:
                # Prepare metadata for service discovery
                metadata = {
                    "service": "AuroraView Bridge",
                    "version": "1.0.0",
                    "protocol": self.protocol,
                }
                self._service_discovery.start(metadata)
                logger.info("✅ Service discovery started")
            except Exception as e:
                logger.error(f"Failed to start service discovery: {e}")

        self._is_running = True
        self._loop = asyncio.get_event_loop()

        async with websockets.serve(self._handle_client, self.host, self.port):
            logger.info(f"✅ WebSocket server listening on ws://{self.host}:{self.port}")
            logger.info("📡 Waiting for clients to connect...")
            await asyncio.Future()  # Run forever

    def start_background(self):
        """Start the WebSocket server in a background thread (non-blocking).

        This is the recommended way to start the Bridge when using with WebView.
        The server runs in a daemon thread and stops when the main program exits.

        Example:
            >>> bridge = Bridge(port=9001)
            >>> bridge.start_background()  # Returns immediately
            >>> # Server is now running in background
        """
        if self._is_running:
            logger.warning("Bridge is already running")
            return

        logger.info("Starting Bridge in background thread...")

        def _run_server():
            """Run the server in background thread."""
            try:
                # Create new event loop for this thread
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
                self._loop = loop

                logger.info("Background thread: Starting WebSocket server")
                loop.run_until_complete(self.start())
            except Exception as e:
                logger.error(f"Error in Bridge background thread: {e}", exc_info=True)
            finally:
                logger.info("Background thread: Bridge stopped")

        self._thread = threading.Thread(target=_run_server, daemon=True)
        self._thread.start()
        logger.info("✅ Bridge background thread started")

        # Wait a bit for the server to start
        import time

        time.sleep(0.5)  # Give the thread time to start
        logger.info(f"Bridge status after start: {self}")

    async def stop(self):
        """Stop the WebSocket server.

        Closes all client connections and stops the server.
        """
        logger.info("Stopping Bridge...")

        # Close all client connections
        if self._clients:
            await asyncio.gather(
                *[client.close() for client in self._clients], return_exceptions=True
            )

        # Stop service discovery if enabled
        if self._service_discovery:
            try:
                self._service_discovery.stop()
                logger.info("✅ Service discovery stopped")
            except Exception as e:
                logger.error(f"Failed to stop service discovery: {e}")

        self._is_running = False
        logger.info("✅ Bridge stopped")

    async def _handle_client(self, websocket: WebSocketServerProtocol):
        """Handle a new client connection.

        Args:
            websocket: WebSocket connection
        """
        client_addr = websocket.remote_address
        logger.info(f"✅ New client connected: {client_addr}")

        self._clients.add(websocket)

        try:
            async for message in websocket:
                await self._process_message(message, websocket)
        except websockets.exceptions.ConnectionClosed:
            logger.info(f"🔌 Client disconnected: {client_addr}")
        except Exception as e:
            logger.error(f"❌ Error handling client {client_addr}: {e}", exc_info=True)
        finally:
            self._clients.remove(websocket)
            logger.info(f"Client removed: {client_addr} (total: {len(self._clients)})")

    async def _process_message(self, message: str, websocket: WebSocketServerProtocol):
        """Process incoming message from client.

        Args:
            message: JSON message string
            websocket: WebSocket connection
        """
        try:
            # Decode message
            if self.protocol == "json":
                data = json.loads(message)
            else:
                raise ValueError(f"Unsupported protocol: {self.protocol}")

            action = data.get("action")
            logger.info(f"📨 Received: {action}")
            logger.debug(f"Message data: {data}")

            # Route to handler
            if action in self._handlers:
                handler = self._handlers[action]
                result = await handler(data, websocket)

                # Send response back to client
                if result:
                    await self.send(websocket, result)

                # Notify WebView UI if callback is set
                if self._webview_callback:
                    self._webview_callback(action, data, result)
            else:
                logger.warning(f"⚠️  No handler registered for action: {action}")

        except json.JSONDecodeError as e:
            logger.error(f"❌ Invalid JSON: {e}")
        except Exception as e:
            logger.error(f"❌ Error processing message: {e}", exc_info=True)

    async def send(self, websocket: WebSocketServerProtocol, data: Dict[str, Any]):
        """Send message to a specific client.

        Args:
            websocket: Target WebSocket connection
            data: Data to send (will be JSON serialized)
        """
        try:
            if self.protocol == "json":
                message = json.dumps(data)
            else:
                raise ValueError(f"Unsupported protocol: {self.protocol}")

            await websocket.send(message)
            logger.info(f"📤 Sent to client: {data.get('action', 'unknown')}")
        except Exception as e:
            logger.error(f"❌ Error sending message: {e}")

    async def broadcast(self, data: Dict[str, Any]):
        """Broadcast message to all connected clients.

        Args:
            data: Data to broadcast (will be JSON serialized)
        """
        if not self._clients:
            logger.warning("⚠️  No clients connected to broadcast to")
            return

        if self.protocol == "json":
            message = json.dumps(data)
        else:
            raise ValueError(f"Unsupported protocol: {self.protocol}")

        # Send to all clients concurrently
        await asyncio.gather(
            *[client.send(message) for client in self._clients], return_exceptions=True
        )

        logger.info(
            f"📡 Broadcast to {len(self._clients)} clients: {data.get('action', 'unknown')}"
        )

    def execute_command(self, command: str, params: Dict[str, Any] = None):
        """Send command to all clients (non-blocking).

        This is a convenience method that broadcasts a command to all clients
        without blocking. Useful for sending commands from synchronous code.

        Args:
            command: Command name
            params: Command parameters

        Example:
            >>> bridge.execute_command('create_layer', {'name': 'New Layer'})
        """
        data = {
            "type": "request",
            "action": "execute_command",
            "data": {"command": command, "params": params or {}},
        }

        # Schedule broadcast in the event loop
        if self._loop and self._loop.is_running():
            asyncio.run_coroutine_threadsafe(self.broadcast(data), self._loop)
        else:
            logger.warning("Bridge event loop not running, cannot execute command")

    @property
    def clients(self) -> Set[WebSocketServerProtocol]:
        """Get set of connected clients."""
        return self._clients

    @property
    def is_running(self) -> bool:
        """Check if server is running."""
        return self._is_running

    @property
    def service_discovery(self):
        """Get the service discovery instance (if enabled).

        Returns:
            ServiceDiscovery instance or None
        """
        return self._service_discovery

    @property
    def client_count(self) -> int:
        """Get number of connected clients."""
        return len(self._clients)

    def __repr__(self) -> str:
        """String representation."""
        status = "running" if self._is_running else "stopped"
        return f"Bridge(ws://{self.host}:{self.port}, {status}, clients={self.client_count})"
