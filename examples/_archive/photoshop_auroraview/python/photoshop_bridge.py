"""Photoshop Bridge - WebSocket server for Photoshop UXP plugin communication."""

import asyncio
import json
import logging
from typing import Set, Optional, Callable, Dict, Any
import websockets
from websockets.server import WebSocketServerProtocol

logger = logging.getLogger(__name__)


class PhotoshopBridge:
    """Bridge between Photoshop UXP plugin and Python backend.
    
    This class manages WebSocket connections from Photoshop and routes
    messages to appropriate handlers.
    """
    
    def __init__(self, host: str = "localhost", port: int = 9001):
        """Initialize the bridge.
        
        Args:
            host: WebSocket server host
            port: WebSocket server port
        """
        self.host = host
        self.port = port
        self.clients: Set[WebSocketServerProtocol] = set()
        self.message_handlers: Dict[str, Callable] = {}
        self.webview_callback: Optional[Callable] = None
        
    def register_handler(self, action: str, handler: Callable):
        """Register a message handler for specific action.
        
        Args:
            action: Action name (e.g., 'layer_created', 'get_image')
            handler: Async function to handle the message
        """
        self.message_handlers[action] = handler
        logger.info(f"Registered handler for action: {action}")
        
    def set_webview_callback(self, callback: Callable):
        """Set callback to communicate with WebView UI.
        
        Args:
            callback: Function to call when UI needs to be updated
        """
        self.webview_callback = callback
        
    async def start(self):
        """Start the WebSocket server."""
        logger.info(f"🚀 Starting Photoshop Bridge on {self.host}:{self.port}")
        
        async with websockets.serve(self._handle_client, self.host, self.port):
            logger.info(f"✅ WebSocket server listening on ws://{self.host}:{self.port}")
            logger.info("📡 Waiting for Photoshop UXP plugin to connect...")
            await asyncio.Future()  # Run forever
            
    async def _handle_client(self, websocket: WebSocketServerProtocol):
        """Handle a new client connection.
        
        Args:
            websocket: WebSocket connection
        """
        client_addr = websocket.remote_address
        logger.info(f"✅ New connection from Photoshop: {client_addr}")
        
        self.clients.add(websocket)
        
        try:
            async for message in websocket:
                await self._process_message(message, websocket)
        except websockets.exceptions.ConnectionClosed:
            logger.info(f"🔌 Connection closed: {client_addr}")
        except Exception as e:
            logger.error(f"❌ Error handling client {client_addr}: {e}", exc_info=True)
        finally:
            self.clients.remove(websocket)
            
    async def _process_message(self, message: str, websocket: WebSocketServerProtocol):
        """Process incoming message from Photoshop.
        
        Args:
            message: JSON message string
            websocket: WebSocket connection
        """
        try:
            data = json.loads(message)
            action = data.get('action')
            
            logger.info(f"📨 Received from Photoshop: {action}")
            logger.debug(f"Message data: {data}")
            
            # Route to appropriate handler
            if action in self.message_handlers:
                handler = self.message_handlers[action]
                result = await handler(data, websocket)
                
                # Send response back to Photoshop
                if result:
                    await self.send_to_client(websocket, result)
                    
                # Notify WebView UI if callback is set
                if self.webview_callback:
                    self.webview_callback(action, data, result)
            else:
                logger.warning(f"⚠️  No handler registered for action: {action}")
                
        except json.JSONDecodeError as e:
            logger.error(f"❌ Invalid JSON: {e}")
        except Exception as e:
            logger.error(f"❌ Error processing message: {e}", exc_info=True)
            
    async def send_to_client(self, websocket: WebSocketServerProtocol, data: Dict[str, Any]):
        """Send message to a specific Photoshop client.
        
        Args:
            websocket: Target WebSocket connection
            data: Data to send (will be JSON serialized)
        """
        try:
            message = json.dumps(data)
            await websocket.send(message)
            logger.info(f"📤 Sent to Photoshop: {data.get('action', 'unknown')}")
        except Exception as e:
            logger.error(f"❌ Error sending message: {e}")
            
    async def broadcast(self, data: Dict[str, Any]):
        """Broadcast message to all connected Photoshop clients.
        
        Args:
            data: Data to broadcast (will be JSON serialized)
        """
        if not self.clients:
            logger.warning("⚠️  No clients connected to broadcast to")
            return
            
        message = json.dumps(data)
        
        # Send to all clients concurrently
        await asyncio.gather(
            *[client.send(message) for client in self.clients],
            return_exceptions=True
        )
        
        logger.info(f"📡 Broadcast to {len(self.clients)} clients: {data.get('action', 'unknown')}")
        
    def execute_photoshop_command(self, command: str, params: Dict[str, Any] = None):
        """Send command to Photoshop for execution.
        
        Args:
            command: Command name
            params: Command parameters
        """
        data = {
            "type": "request",
            "action": "execute_command",
            "data": {
                "command": command,
                "params": params or {}
            }
        }
        
        # Broadcast to all connected clients
        asyncio.create_task(self.broadcast(data))


# Example usage
if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    bridge = PhotoshopBridge()
    
    # Register example handlers
    async def handle_layer_created(data, websocket):
        logger.info(f"🎨 Layer created: {data.get('data', {})}")
        return {
            "type": "response",
            "action": "layer_created_ack",
            "data": {"status": "received"}
        }
        
    bridge.register_handler("layer_created", handle_layer_created)
    
    # Start server
    asyncio.run(bridge.start())

