"""Photoshop Layers Demo - Complete example with service discovery

This example demonstrates:
1. Creating layers in Photoshop from WebView UI
2. Getting layer information
3. Deleting and renaming layers
4. Using service discovery for automatic port allocation
"""

import asyncio
import logging
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "python"))

from auroraview import Bridge, WebView

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


class PhotoshopLayersTool:
    """Photoshop Layers Tool with AuroraView integration."""
    
    def __init__(self):
        """Initialize the tool."""
        # Create Bridge with fixed port for UXP plugin compatibility
        self.bridge = Bridge(
            port=9001,                 # Fixed port (UXP plugin expects 9001)
            service_discovery=True,    # Enable service discovery
            discovery_port=9000,       # HTTP discovery endpoint
            enable_mdns=False,         # Disable mDNS for this example
        )
        
        self.webview = None
        self.layers_cache = []
        
        # Setup handlers
        self._setup_handlers()
        
    def _setup_handlers(self):
        """Setup Bridge message handlers."""
        
        @self.bridge.on('handshake')
        async def handle_handshake(data, client):
            """Handle Photoshop connection."""
            logger.info(f"🤝 Photoshop connected: {data}")
            
            # Notify WebView
            if self.webview:
                self.webview.emit('bridge:photoshop_connected', data)
            
            return {
                "type": "response",
                "action": "handshake_ack",
                "data": {
                    "server": "auroraview-layers-demo",
                    "version": "1.0.0"
                }
            }
        
        @self.bridge.on('layer_created')
        async def handle_layer_created(data, client):
            """Handle layer creation event."""
            logger.info(f"🎨 Layer created: {data}")
            
            # Update cache
            self.layers_cache.append(data)
            
            # Notify WebView
            if self.webview:
                self.webview.emit('bridge:layer_created', data)
            
            return None
        
        @self.bridge.on('layers_list')
        async def handle_layers_list(data, client):
            """Handle layers list response."""
            logger.info(f"📋 Received {data.get('count', 0)} layers")
            
            # Update cache
            self.layers_cache = data.get('layers', [])
            
            # Notify WebView
            if self.webview:
                self.webview.emit('bridge:layers_list', data)
            
            return None
        
        @self.bridge.on('layer_deleted')
        async def handle_layer_deleted(data, client):
            """Handle layer deletion event."""
            logger.info(f"🗑️  Layer deleted: {data}")
            
            # Update cache
            self.layers_cache = [l for l in self.layers_cache if l.get('id') != data.get('id')]
            
            # Notify WebView
            if self.webview:
                self.webview.emit('bridge:layer_deleted', data)
            
            return None
        
        @self.bridge.on('layer_renamed')
        async def handle_layer_renamed(data, client):
            """Handle layer rename event."""
            logger.info(f"✏️  Layer renamed: {data}")
            
            # Update cache
            for layer in self.layers_cache:
                if layer.get('id') == data.get('id'):
                    layer['name'] = data.get('newName')
            
            # Notify WebView
            if self.webview:
                self.webview.emit('bridge:layer_renamed', data)
            
            return None
        
        @self.bridge.on('document_info')
        async def handle_document_info(data, client):
            """Handle document info response."""
            logger.info(f"📄 Document: {data.get('name')}")
            
            # Notify WebView
            if self.webview:
                self.webview.emit('bridge:document_info', data)
            
            return None
    
    def create_webview(self):
        """Create WebView UI."""
        # Read HTML content
        html_path = Path(__file__).parent / "ui.html"
        with open(html_path, 'r', encoding='utf-8') as f:
            html_content = f.read()
        
        # Replace port placeholder
        html_content = html_content.replace('{{BRIDGE_PORT}}', str(self.bridge.port))
        
        # Create WebView
        self.webview = WebView.create(
            title="Photoshop Layers Demo",
            html=html_content,
            width=500,
            height=700,
            bridge=self.bridge  # Auto-connect Bridge
        )
        
        logger.info(f"✅ WebView created with Bridge on port {self.bridge.port}")
    
    def run(self):
        """Run the tool."""
        logger.info("=" * 80)
        logger.info("Photoshop Layers Demo")
        logger.info("=" * 80)
        logger.info(f"📡 Bridge port: {self.bridge.port}")
        logger.info(f"🔍 HTTP discovery: http://localhost:9000/discover")
        logger.info(f"🔧 Bridge status: {self.bridge}")
        logger.info("")
        logger.info("📝 Instructions:")
        logger.info("1. Open Photoshop")
        logger.info("2. Open plugin: Window → Plugins → AuroraView Bridge v2")
        logger.info("3. Make sure you have a document open in Photoshop")
        logger.info("4. Click 'Connect to Python' in the plugin")
        logger.info("5. Use the buttons in this window to create/manage layers")
        logger.info("=" * 80)

        # Create and show WebView
        logger.info("Creating WebView...")
        self.create_webview()
        logger.info(f"WebView created, Bridge status: {self.bridge}")
        logger.info("Showing WebView (this will start Bridge)...")
        self.webview.show()
        logger.info(f"WebView shown, Bridge status: {self.bridge}")


def main():
    """Main entry point."""
    tool = PhotoshopLayersTool()
    tool.run()


if __name__ == "__main__":
    main()

