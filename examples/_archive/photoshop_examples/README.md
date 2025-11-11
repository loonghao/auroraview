# Photoshop Integration Example

[![中文文档](https://img.shields.io/badge/docs-中文-blue)](./README_zh.md)

This example demonstrates bidirectional communication between Adobe Photoshop and AuroraView using WebSocket protocol.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Adobe Photoshop 2025+                      │
│                                                         │
│  ┌───────────────────────────────────────────────┐    │
│  │         UXP Plugin (JavaScript)               │    │
│  │  - WebSocket Client                           │    │
│  │  - Photoshop Imaging API                      │    │
│  │  - Layer Management                           │    │
│  └──────────────────┬────────────────────────────┘    │
└─────────────────────┼──────────────────────────────────┘
                      │
                      │ WebSocket (ws://localhost:9001)
                      │
┌─────────────────────▼──────────────────────────────────┐
│         Rust WebSocket Server                          │
│  - tokio-tungstenite                                   │
│  - Message routing                                     │
│  - Multi-client support                                │
└─────────────────────┬──────────────────────────────────┘
                      │
                      │ IPC / API
                      │
┌─────────────────────▼──────────────────────────────────┐
│              AuroraView Core                           │
│  - DCC Integration                                     │
│  - Asset Management                                    │
└────────────────────────────────────────────────────────┘
```

## Features

- ✅ Real-time bidirectional communication
- ✅ Layer creation and management
- ✅ Selection information retrieval
- ✅ Document metadata access
- ✅ Auto-reconnection mechanism
- ✅ Multi-client broadcast support

## Prerequisites

### Photoshop Side
- Adobe Photoshop 2024 or later (v24.0+)
- [UXP Developer Tool](https://developer.adobe.com/photoshop/uxp/2022/guides/devtool/)

### Server Side
- Rust 1.70+ with Cargo
- tokio runtime

## Quick Start

### 1. Start WebSocket Server

```bash
cd examples/photoshop_examples
cargo run --bin websocket_server
```

You should see:
```
🚀 AuroraView WebSocket Server listening on: 127.0.0.1:9001
📡 Waiting for Photoshop UXP plugin to connect...
```

### 2. Load UXP Plugin in Photoshop

1. Open **UXP Developer Tool**
2. Click **Add Plugin**
3. Navigate to `examples/photoshop_examples/uxp_plugin/manifest.json`
4. Click **Load**
5. In Photoshop, go to **Plugins → AuroraView**

### 3. Connect Plugin to Server

1. In the AuroraView panel, verify server URL: `ws://localhost:9001`
2. Click **Connect** button
3. Status should change to "Connected" (green)

### 4. Test Communication

Click any action button:
- **Create New Layer**: Creates a layer and sends info to server
- **Get Selection Info**: Retrieves current selection bounds
- **Get Document Info**: Sends document metadata

Check the server console for received messages!

## Message Protocol

### Message Format

All messages use JSON format:

```json
{
  "type": "request|response|event",
  "id": "unique-message-id",
  "action": "action_name",
  "data": {
    // Action-specific payload
  },
  "timestamp": 1704067200000
}
```

### Supported Actions

#### From Photoshop → Server

| Action | Description | Data |
|--------|-------------|------|
| `handshake` | Initial connection | `{ client, version, app, appVersion }` |
| `layer_created` | Layer created event | `{ name, id }` |
| `selection_info` | Selection data | `{ hasSelection, bounds, documentName }` |
| `document_info` | Document metadata | `{ name, width, height, resolution, colorMode }` |
| `command_result` | Command execution result | `{ command, success, result/error }` |

#### From Server → Photoshop

| Action | Description | Data |
|--------|-------------|------|
| `handshake_ack` | Handshake acknowledgment | `{ server, version, status }` |
| `execute_command` | Execute Photoshop command | `{ command, params }` |

## Project Structure

```
photoshop_examples/
├── Cargo.toml                 # Rust dependencies
├── websocket_server.rs        # WebSocket server implementation
├── README.md                  # This file
├── README_zh.md              # Chinese documentation
└── uxp_plugin/               # Photoshop UXP plugin
    ├── manifest.json         # Plugin manifest (v5)
    ├── index.html            # Plugin UI
    └── index.js              # Plugin logic
```

## Development

### Modify Server Logic

Edit `websocket_server.rs` and customize the `handle_photoshop_message` function:

```rust
fn handle_photoshop_message(msg: &WsMessage, peer_map: &PeerMap, sender_addr: &SocketAddr) {
    match msg.action.as_str() {
        "your_custom_action" => {
            // Your custom logic here
        }
        _ => {}
    }
}
```

### Add New Photoshop Actions

Edit `uxp_plugin/index.js`:

```javascript
async function yourCustomAction() {
    try {
        // Use Photoshop APIs
        const result = await app.batchPlay([...], {});
        
        // Send to server
        sendMessage('your_custom_action', { result });
    } catch (error) {
        log(`Error: ${error.message}`, 'error');
    }
}
```

## Troubleshooting

### Connection Failed

**Problem**: Plugin shows "Disconnected" status

**Solutions**:
1. Verify server is running: `cargo run --bin websocket_server`
2. Check firewall settings (allow port 9001)
3. Ensure URL is `ws://localhost:9001` (not `wss://`)

### Network Permission Error

**Problem**: UXP throws network permission error

**Solution**: Verify `manifest.json` includes:
```json
{
  "requiredPermissions": {
    "network": {
      "domains": ["ws://localhost:*"]
    }
  }
}
```

### Plugin Not Loading

**Problem**: UXP Developer Tool shows error

**Solutions**:
1. Check Photoshop version (must be 24.0+)
2. Validate `manifest.json` syntax
3. Check UXP Developer Tool console for errors

## Next Steps

- [ ] Implement secure WebSocket (wss://)
- [ ] Add authentication mechanism
- [ ] Integrate with AuroraView core
- [ ] Add batch operations support
- [ ] Implement asset export functionality

## References

- [Adobe UXP Documentation](https://developer.adobe.com/photoshop/uxp/)
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/)
- [Photoshop Imaging API](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/)

## License

This example is part of the AuroraView project.

