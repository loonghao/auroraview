"""Service Discovery Demo - Bridge with Auto Port Allocation

This example demonstrates the new service discovery features:
1. Automatic port allocation (no more port conflicts!)
2. HTTP discovery endpoint for UXP plugins
3. mDNS service discovery for DCC tools

Usage:
    python bridge_with_discovery.py
"""

import asyncio
import logging
from auroraview import Bridge, WebView

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


def main():
    """Main function demonstrating service discovery."""
    
    logger.info("=" * 80)
    logger.info("Service Discovery Demo")
    logger.info("=" * 80)
    
    # Create Bridge with service discovery enabled
    # port=0 means auto-allocate a free port
    bridge = Bridge(
        port=0,                    # Auto-allocate port
        service_discovery=True,    # Enable service discovery
        discovery_port=9000,       # HTTP discovery endpoint
        enable_mdns=True,          # Enable mDNS (Zeroconf)
    )
    
    logger.info(f"\n✅ Bridge created with auto-allocated port: {bridge.port}")
    logger.info(f"📡 HTTP discovery endpoint: http://localhost:9000/discover")
    logger.info(f"🔍 mDNS service: _auroraview._tcp.local.")
    
    # Register a simple handler
    @bridge.on('ping')
    async def handle_ping(data, client):
        """Handle ping requests."""
        logger.info(f"📨 Received ping from client: {data}")
        return {
            "type": "response",
            "action": "pong",
            "data": {
                "message": "Hello from AuroraView!",
                "timestamp": data.get("timestamp", 0)
            }
        }
    
    # Create WebView with Bridge
    html = f"""
    <!DOCTYPE html>
    <html>
    <head>
        <title>Service Discovery Demo</title>
        <style>
            body {{
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                max-width: 800px;
                margin: 50px auto;
                padding: 20px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
            }}
            .card {{
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(10px);
                border-radius: 15px;
                padding: 30px;
                margin: 20px 0;
                box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
            }}
            h1 {{
                margin: 0 0 20px 0;
                font-size: 2.5em;
            }}
            .info {{
                background: rgba(255, 255, 255, 0.2);
                padding: 15px;
                border-radius: 10px;
                margin: 10px 0;
                font-family: 'Courier New', monospace;
            }}
            button {{
                background: #4CAF50;
                color: white;
                border: none;
                padding: 15px 30px;
                font-size: 16px;
                border-radius: 8px;
                cursor: pointer;
                margin: 10px 5px;
                transition: all 0.3s;
            }}
            button:hover {{
                background: #45a049;
                transform: translateY(-2px);
                box-shadow: 0 4px 8px rgba(0,0,0,0.2);
            }}
            #status {{
                margin-top: 20px;
                padding: 15px;
                border-radius: 8px;
                background: rgba(255, 255, 255, 0.1);
            }}
        </style>
    </head>
    <body>
        <div class="card">
            <h1>🚀 Service Discovery Demo</h1>
            
            <div class="info">
                <strong>Bridge Port:</strong> {bridge.port}<br>
                <strong>Discovery Endpoint:</strong> http://localhost:9000/discover<br>
                <strong>WebSocket URL:</strong> ws://localhost:{bridge.port}
            </div>
            
            <h2>Test Connection</h2>
            <button onclick="testDiscovery()">Test HTTP Discovery</button>
            <button onclick="testWebSocket()">Test WebSocket</button>
            
            <div id="status"></div>
        </div>
        
        <script>
            const status = document.getElementById('status');
            
            async function testDiscovery() {{
                status.innerHTML = '🔍 Testing HTTP discovery...';
                try {{
                    const response = await fetch('http://localhost:9000/discover');
                    const data = await response.json();
                    status.innerHTML = `
                        <strong>✅ Discovery Success!</strong><br>
                        <pre>${{JSON.stringify(data, null, 2)}}</pre>
                    `;
                }} catch (error) {{
                    status.innerHTML = `❌ Discovery failed: ${{error.message}}`;
                }}
            }}
            
            async function testWebSocket() {{
                status.innerHTML = '📡 Connecting to WebSocket...';
                try {{
                    const ws = new WebSocket('ws://localhost:{bridge.port}');
                    
                    ws.onopen = () => {{
                        status.innerHTML = '✅ WebSocket connected! Sending ping...';
                        ws.send(JSON.stringify({{
                            action: 'ping',
                            timestamp: Date.now()
                        }}));
                    }};
                    
                    ws.onmessage = (event) => {{
                        const data = JSON.parse(event.data);
                        status.innerHTML = `
                            <strong>✅ Received response!</strong><br>
                            <pre>${{JSON.stringify(data, null, 2)}}</pre>
                        `;
                    }};
                    
                    ws.onerror = (error) => {{
                        status.innerHTML = `❌ WebSocket error: ${{error}}`;
                    }};
                }} catch (error) {{
                    status.innerHTML = `❌ WebSocket failed: ${{error.message}}`;
                }}
            }}
        </script>
    </body>
    </html>
    """
    
    webview = WebView.create(
        title="Service Discovery Demo",
        html=html,
        width=900,
        height=700,
        bridge=bridge
    )
    
    logger.info("\n🎉 Starting WebView and Bridge...")
    logger.info("Try the buttons in the UI to test service discovery!\n")
    
    webview.show()


if __name__ == "__main__":
    main()

