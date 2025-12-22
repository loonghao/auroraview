# Photoshop Integration

AuroraView integrates with Adobe Photoshop through the UXP (Unified Extensibility Platform) plugin system and WebSocket communication.

## Architecture

```
┌─────────────────────────────────────────────┐
│              Photoshop                       │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────┐ │
│  │  UXP Panel  │ ◄──► │  AuroraView      │ │
│  │  (WebView)  │      │  Python Backend  │ │
│  └─────────────┘      └──────────────────┘ │
│         │                      │            │
│         │ WebSocket            │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │      Photoshop API (batchPlay)      │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## Requirements

| Component | Minimum Version | Recommended |
|-----------|-----------------|-------------|
| Adobe Photoshop | 24.0 (2023) | 26.0+ (2025) |
| Rust | 1.70 | 1.75+ |
| Node.js (optional) | 16.x | 20.x+ |
| OS | Windows 10, macOS 11 | Windows 11, macOS 14+ |

## Setup Guide

### Step 1: Install UXP Developer Tool

**Via Creative Cloud Desktop:**
1. Open **Creative Cloud Desktop**
2. Go to **All Apps**
3. Search for "UXP Developer Tool"
4. Click **Install**

**Direct Download:**
1. Visit [Adobe Developer Console](https://developer.adobe.com/console)
2. Download **UXP Developer Tool**
3. Run the installer

### Step 2: Configure Rust Environment

```bash
# Windows (PowerShell)
winget install Rustlang.Rustup

# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Step 3: Build WebSocket Server

```bash
cd examples/photoshop_examples
cargo build --release
```

### Step 4: Start the Server

```bash
# Development mode (with logs)
RUST_LOG=info cargo run --bin websocket_server

# Production mode
./target/release/websocket_server
```

You should see:
```
🚀 AuroraView WebSocket Server listening on: 127.0.0.1:9001
📡 Waiting for Photoshop UXP plugin to connect...
```

### Step 5: Load UXP Plugin

1. Launch **UXP Developer Tool**
2. Ensure Photoshop is running
3. Click **Add Plugin**
4. Select `examples/photoshop_examples/uxp_plugin/manifest.json`
5. Click **Load** on the plugin

### Step 6: Open in Photoshop

1. In Photoshop, go to **Plugins → AuroraView**
2. The plugin panel should appear

## Testing the Integration

### Test 1: Connection

1. Ensure WebSocket server is running
2. Click **Connect** in the plugin panel
3. Status should show "Connected" (green)
4. Server console should display:
   ```
   ✅ New connection from: 127.0.0.1:xxxxx
   🔗 WebSocket connection established
   🤝 Handshake from Photoshop
   ```

### Test 2: Layer Creation

1. Create or open a document in Photoshop
2. Click **Create New Layer** button
3. Verify:
   - New layer appears in Photoshop
   - Plugin log shows "Layer created successfully"
   - Server console shows "🎨 Layer created"

### Test 3: Document Info

1. Click **Get Document Info**
2. Verify server console displays document details

## Planned API

```python
from auroraview import PhotoshopWebView

class PhotoshopAPI:
    def get_active_document(self) -> dict:
        """Get active document info"""
        pass

    def get_layers(self) -> dict:
        """Get document layers"""
        pass

    def select_layer(self, name: str) -> dict:
        """Select a layer by name"""
        pass

    def apply_filter(self, filter_name: str, params: dict) -> dict:
        """Apply a filter to the active layer"""
        pass

webview = PhotoshopWebView(
    url="http://localhost:3000",
    api=PhotoshopAPI()
)
webview.show()
```

## Production Deployment

### Enable WSS (Secure WebSocket)

**Generate SSL Certificate:**
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

**Update manifest.json:**
```json
{
  "requiredPermissions": {
    "network": {
      "domains": ["wss://your-domain.com:9001"]
    }
  }
}
```

### Add Authentication

```rust
fn handle_photoshop_message(msg: &WsMessage, peer_map: &PeerMap, sender_addr: &SocketAddr) {
    // Verify authentication token
    if let Some(token) = msg.data.get("auth_token") {
        if !verify_token(token) {
            send_error(peer_map, sender_addr, "Invalid token");
            return;
        }
    }
    // ... rest of the logic
}
```

### Performance Optimization

**Message Batching:**
```rust
let mut message_buffer = Vec::new();
// ... collect messages
send_batch(peer_map, sender_addr, &message_buffer);
```

**Connection Pool Management:**
```rust
const MAX_CONNECTIONS: usize = 100;

if peer_map.lock().unwrap().len() >= MAX_CONNECTIONS {
    eprintln!("❌ Max connections reached");
    return;
}
```

### Logging Configuration

```bash
# Development
RUST_LOG=debug cargo run

# Production
RUST_LOG=info cargo run
```

## Troubleshooting

### Plugin cannot connect to server

**Symptoms**: Click Connect but status remains "Disconnected"

**Solutions**:
1. Check if server is running: `netstat -an | findstr 9001` (Windows) or `lsof -i :9001` (macOS/Linux)
2. Check firewall settings
3. Confirm URL is correct: `ws://localhost:9001`
4. Check browser console (UXP Developer Tool → Debug)

### Network permission error

**Symptoms**: UXP throws "Network access denied"

**Solution**: Ensure `manifest.json` has correct permissions:
```json
{
  "requiredPermissions": {
    "network": {
      "domains": ["ws://localhost:*"]
    }
  }
}
```

### Plugin fails to load

**Symptoms**: UXP Developer Tool shows "Failed to load"

**Solutions**:
1. Validate `manifest.json` syntax
2. Check Photoshop version compatibility
3. View UXP Developer Tool console for errors
4. Verify all file paths are correct

### Messages not received

**Symptoms**: Server doesn't receive messages

**Solutions**:
1. Check WebSocket connection status
2. Validate message format (must be valid JSON)
3. Check plugin logs
4. Use browser developer tools to debug

## Development Status

| Feature | Status |
|---------|--------|
| Basic Integration | 🚧 In Progress |
| Layer Management | 📋 Planned |
| Filter Application | 📋 Planned |
| Selection Sync | 📋 Planned |
| History Integration | 📋 Planned |

## Resources

- [Adobe UXP Documentation](https://developer.adobe.com/photoshop/uxp/)
- [Photoshop API Reference](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/)
- [tokio-tungstenite Documentation](https://docs.rs/tokio-tungstenite/)
- [WebSocket Protocol Specification](https://datatracker.ietf.org/doc/html/rfc6455)
