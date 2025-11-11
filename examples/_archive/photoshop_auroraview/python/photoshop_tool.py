"""Photoshop Tool - Main entry point integrating AuroraView WebView."""

import asyncio
import logging
import sys
import threading
from pathlib import Path

# Add parent directory to path to import auroraview
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / "python"))

from auroraview import WebView
from photoshop_bridge import PhotoshopBridge
from image_processor import ImageProcessor

logger = logging.getLogger(__name__)


class PhotoshopTool:
    """Main Photoshop tool integrating WebView UI and Python backend.
    
    This class creates:
    1. AuroraView WebView for UI
    2. WebSocket server for Photoshop communication
    3. Image processing pipeline
    """
    
    def __init__(self, ui_url: str = "http://localhost:5173"):
        """Initialize the Photoshop tool.
        
        Args:
            ui_url: URL for WebView UI (Vite dev server or built app)
        """
        self.ui_url = ui_url
        self.webview: WebView = None
        self.bridge = PhotoshopBridge()
        self.processor = ImageProcessor()
        
        # Setup message handlers
        self._setup_handlers()
        
    def _setup_handlers(self):
        """Setup message handlers for Photoshop bridge."""
        
        # Handler for handshake
        async def handle_handshake(data, websocket):
            logger.info(f"🤝 Handshake from Photoshop: {data.get('data', {})}")
            
            # Update WebView UI
            if self.webview:
                self.webview.evaluate_js("""
                    window.dispatchEvent(new CustomEvent('photoshop-connected', {
                        detail: { status: 'connected' }
                    }));
                """)
                
            return {
                "type": "response",
                "action": "handshake_ack",
                "data": {
                    "server": "auroraview-photoshop",
                    "version": "1.0.0",
                    "status": "connected"
                }
            }
            
        # Handler for layer creation
        async def handle_layer_created(data, websocket):
            layer_info = data.get('data', {})
            logger.info(f"🎨 Layer created: {layer_info}")
            
            # Update WebView UI
            if self.webview:
                import json
                self.webview.evaluate_js(f"""
                    window.dispatchEvent(new CustomEvent('layer-created', {{
                        detail: {json.dumps(layer_info)}
                    }}));
                """)
                
            return None  # No response needed
            
        # Handler for image data
        async def handle_image_data(data, websocket):
            image_data = data.get('data', {}).get('image')
            logger.info("📷 Received image data from Photoshop")
            
            # Update WebView preview
            if self.webview and image_data:
                import json
                self.webview.evaluate_js(f"""
                    window.dispatchEvent(new CustomEvent('image-received', {{
                        detail: {{ image: {json.dumps(image_data)} }}
                    }}));
                """)
                
            return None
            
        # Register handlers
        self.bridge.register_handler("handshake", handle_handshake)
        self.bridge.register_handler("layer_created", handle_layer_created)
        self.bridge.register_handler("image_data", handle_image_data)
        
    def _setup_webview_bindings(self):
        """Setup Python function bindings for WebView."""
        
        def apply_filter(params):
            """Apply image filter from UI."""
            filter_type = params.get('type')
            image_data = params.get('image')
            
            logger.info(f"🎨 Applying filter: {filter_type}")
            
            if filter_type == 'gaussian_blur':
                radius = params.get('radius', 5)
                result = self.processor.apply_gaussian_blur(image_data, radius)
            elif filter_type == 'enhance_contrast':
                factor = params.get('factor', 1.5)
                result = self.processor.enhance_contrast(image_data, factor)
            elif filter_type == 'sharpen':
                factor = params.get('factor', 2.0)
                result = self.processor.sharpen(image_data, factor)
            elif filter_type == 'edge_detection':
                result = self.processor.edge_detection(image_data)
            else:
                result = {"error": f"Unknown filter type: {filter_type}"}
                
            return result
            
        def send_to_photoshop(params):
            """Send command to Photoshop."""
            command = params.get('command')
            command_params = params.get('params', {})
            
            logger.info(f"📤 Sending command to Photoshop: {command}")
            self.bridge.execute_photoshop_command(command, command_params)
            
            return {"status": "sent"}
            
        def get_status():
            """Get connection status."""
            return {
                "photoshop_connected": len(self.bridge.clients) > 0,
                "client_count": len(self.bridge.clients)
            }
            
        # Bind functions to WebView
        self.webview.bind("apply_filter", apply_filter)
        self.webview.bind("send_to_photoshop", send_to_photoshop)
        self.webview.bind("get_status", get_status)
        
    def create_webview(self):
        """Create AuroraView WebView UI."""
        logger.info(f"🌐 Creating WebView UI: {self.ui_url}")
        
        self.webview = WebView(
            title="Photoshop AI Tools",
            width=400,
            height=800,
            url=self.ui_url,
            debug=True,
            resizable=True
        )
        
        # Setup Python bindings
        self._setup_webview_bindings()
        
        # Show the window
        self.webview.show()
        
    def start_bridge(self):
        """Start WebSocket bridge in background thread."""
        def run_bridge():
            asyncio.run(self.bridge.start())
            
        bridge_thread = threading.Thread(target=run_bridge, daemon=True)
        bridge_thread.start()
        logger.info("✅ WebSocket bridge started in background")
        
    def run(self):
        """Run the Photoshop tool."""
        logger.info("🚀 Starting Photoshop Tool...")
        
        # Start WebSocket bridge
        self.start_bridge()
        
        # Create and show WebView UI
        self.create_webview()
        
        logger.info("✅ Photoshop Tool is running!")
        logger.info("📡 Waiting for Photoshop UXP plugin to connect...")
        logger.info("🌐 WebView UI is ready")


def main():
    """Main entry point."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Create and run tool
    tool = PhotoshopTool()
    tool.run()


if __name__ == "__main__":
    main()

