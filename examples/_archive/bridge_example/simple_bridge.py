"""Simple Bridge example - demonstrates the new built-in Bridge API.

This example shows how to use the built-in Bridge functionality to create
a WebSocket server that can communicate with external applications.

Usage:
    python simple_bridge.py
"""

import asyncio
from auroraview import WebView, Bridge


async def main():
    """Main function demonstrating Bridge usage."""
    
    # Create Bridge with decorator API
    bridge = Bridge(port=9001)
    
    # Register message handlers using decorator
    @bridge.on('handshake')
    async def handle_handshake(data, client):
        """Handle client handshake."""
        print(f"✅ Client connected: {data}")
        return {
            "type": "response",
            "action": "handshake_ack",
            "data": {
                "server": "auroraview",
                "version": "1.0.0",
                "status": "connected"
            }
        }
    
    @bridge.on('ping')
    async def handle_ping(data, client):
        """Handle ping request."""
        print(f"📡 Ping received: {data}")
        return {
            "type": "response",
            "action": "pong",
            "data": {"timestamp": data.get('timestamp')}
        }
    
    @bridge.on('create_layer')
    async def handle_create_layer(data, client):
        """Handle layer creation request."""
        layer_name = data.get('name', 'New Layer')
        print(f"🎨 Creating layer: {layer_name}")
        
        # Simulate layer creation
        layer_info = {
            "id": "layer_001",
            "name": layer_name,
            "type": "normal",
            "opacity": 100
        }
        
        return {
            "type": "response",
            "action": "layer_created",
            "data": layer_info
        }
    
    # Create WebView with Bridge
    webview = WebView.create(
        title="Bridge Example",
        html="""
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    padding: 20px;
                    background: #1e1e1e;
                    color: #fff;
                }
                button {
                    padding: 10px 20px;
                    margin: 5px;
                    background: #007acc;
                    color: white;
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                }
                button:hover {
                    background: #005a9e;
                }
                #status {
                    margin-top: 20px;
                    padding: 10px;
                    background: #2d2d2d;
                    border-radius: 4px;
                }
            </style>
        </head>
        <body>
            <h1>🌉 Bridge Example</h1>
            <p>WebSocket Server: <strong>ws://localhost:9001</strong></p>
            
            <div>
                <button onclick="testConnection()">Test Connection</button>
                <button onclick="sendPing()">Send Ping</button>
                <button onclick="createLayer()">Create Layer</button>
            </div>
            
            <div id="status">
                <strong>Status:</strong> Ready
            </div>
            
            <script>
                let ws = null;
                
                function connect() {
                    ws = new WebSocket('ws://localhost:9001');
                    
                    ws.onopen = () => {
                        updateStatus('✅ Connected to Bridge');
                        // Send handshake
                        ws.send(JSON.stringify({
                            type: 'request',
                            action: 'handshake',
                            data: {client: 'WebView', version: '1.0.0'}
                        }));
                    };
                    
                    ws.onmessage = (event) => {
                        const msg = JSON.parse(event.data);
                        updateStatus(`📨 Received: ${msg.action} - ${JSON.stringify(msg.data)}`);
                    };
                    
                    ws.onerror = (error) => {
                        updateStatus(`❌ Error: ${error}`);
                    };
                    
                    ws.onclose = () => {
                        updateStatus('🔌 Disconnected');
                    };
                }
                
                function updateStatus(message) {
                    document.getElementById('status').innerHTML = 
                        `<strong>Status:</strong> ${message}`;
                }
                
                function testConnection() {
                    if (!ws || ws.readyState !== WebSocket.OPEN) {
                        connect();
                    } else {
                        updateStatus('Already connected');
                    }
                }
                
                function sendPing() {
                    if (ws && ws.readyState === WebSocket.OPEN) {
                        ws.send(JSON.stringify({
                            type: 'request',
                            action: 'ping',
                            data: {timestamp: Date.now()}
                        }));
                    } else {
                        updateStatus('❌ Not connected');
                    }
                }
                
                function createLayer() {
                    if (ws && ws.readyState === WebSocket.OPEN) {
                        ws.send(JSON.stringify({
                            type: 'request',
                            action: 'create_layer',
                            data: {name: 'My Layer ' + Date.now()}
                        }));
                    } else {
                        updateStatus('❌ Not connected');
                    }
                }
            </script>
        </body>
        </html>
        """,
        width=600,
        height=500,
        bridge=bridge  # Associate Bridge with WebView
    )
    
    print("🚀 Starting Bridge Example...")
    print("📡 WebSocket server will start on ws://localhost:9001")
    print("🌐 WebView will open with test UI")
    print("\nClick 'Test Connection' to connect to the Bridge")
    
    # Show WebView (Bridge will auto-start)
    webview.show()


if __name__ == "__main__":
    asyncio.run(main())

