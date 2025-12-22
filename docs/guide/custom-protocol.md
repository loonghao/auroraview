# Custom Protocol

AuroraView supports custom protocols for loading local resources without CORS restrictions.

## Built-in Protocol: `auroraview://`

The built-in `auroraview://` protocol allows loading local static resources (HTML, CSS, JS, images) from a specified asset root directory.

### URL Format

```
auroraview://css/style.css
auroraview://js/app.js
auroraview://icons/logo.png
```

### Path Mapping

```
auroraview://css/style.css → {asset_root}/css/style.css
```

### Usage

```python
from auroraview import WebView

webview = WebView.create(
    "My App",
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="auroraview://css/style.css">
        </head>
        <body>
            <img src="auroraview://icons/logo.png">
            <script src="auroraview://js/app.js"></script>
        </body>
    </html>
    """,
    asset_root="C:/projects/my_app/assets"  # Asset root directory
)
webview.show()
```

## Custom Protocol Registration

You can register custom protocols for DCC-specific resource loading.

### Use Cases

- Maya: `maya://scenes/character.ma`
- Houdini: `houdini://hip/project.hip`
- Nuke: `nuke://scripts/comp.nk`
- Custom: `fbx://models/character.fbx`

### Python API

```python
from auroraview import WebView

def handle_fbx_protocol(uri: str) -> dict:
    """
    Handle fbx:// protocol requests
    
    Args:
        uri: Full URI, e.g. "fbx://models/character.fbx"
    
    Returns:
        {
            "data": bytes,        # File content (bytes)
            "mime_type": str,     # MIME type
            "status": int         # HTTP status code (200, 404, etc.)
        }
    """
    # Parse path
    path = uri.replace("fbx://", "")  # "models/character.fbx"
    
    # Read FBX file
    fbx_root = "C:/projects/models"
    full_path = f"{fbx_root}/{path}"
    
    try:
        with open(full_path, "rb") as f:
            data = f.read()
        
        return {
            "data": data,
            "mime_type": "application/octet-stream",
            "status": 200
        }
    except FileNotFoundError:
        return {
            "data": b"Not Found",
            "mime_type": "text/plain",
            "status": 404
        }

# Create WebView
webview = WebView.create("FBX Viewer", asset_root="C:/assets")

# Register custom protocol
webview.register_protocol("fbx", handle_fbx_protocol)

# Use in HTML
webview.load_html("""
<html>
    <body>
        <h1>FBX Viewer</h1>
        <script>
            // Load FBX file via fetch
            fetch('fbx://models/character.fbx')
                .then(r => r.arrayBuffer())
                .then(data => {
                    console.log('FBX loaded:', data.byteLength, 'bytes');
                    // Parse FBX...
                });
        </script>
    </body>
</html>
""")

webview.show()
```

## Maya Plugin Example

```python
from auroraview import WebView
import maya.cmds as cmds
import os

def handle_maya_protocol(uri: str) -> dict:
    """Handle maya:// protocol - load Maya scene file thumbnails"""
    path = uri.replace("maya://", "")
    
    # Maya project directory
    project_dir = cmds.workspace(q=True, rd=True)
    full_path = os.path.join(project_dir, path)
    
    if os.path.exists(full_path):
        with open(full_path, "rb") as f:
            return {
                "data": f.read(),
                "mime_type": "image/jpeg",
                "status": 200
            }
    else:
        return {
            "data": b"Not Found",
            "mime_type": "text/plain",
            "status": 404
        }

# Create WebView
webview = WebView.create(
    "Maya Asset Browser",
    asset_root="C:/maya_plugin/ui",  # UI resource directory
    parent=maya_hwnd,
    mode="owner"
)

# Register maya:// protocol
webview.register_protocol("maya", handle_maya_protocol)

# Load UI
webview.load_html("""
<html>
    <head>
        <link rel="stylesheet" href="auroraview://css/style.css">
    </head>
    <body>
        <h1>Asset Browser</h1>
        <div class="thumbnails">
            <img src="maya://thumbnails/character_rig.jpg">
            <img src="maya://thumbnails/environment.jpg">
        </div>
        <script src="auroraview://js/app.js"></script>
    </body>
</html>
""")

webview.show()
```

## Rust Implementation

### Extending WebViewConfig

```rust
pub struct WebViewConfig {
    // ... existing fields
    
    /// Asset root directory (for auroraview:// protocol)
    pub asset_root: Option<PathBuf>,
    
    /// Custom protocol handlers (scheme -> handler)
    pub custom_protocols: HashMap<String, ProtocolCallback>,
}
```

### Integration with NativeBackend

```rust
fn create_webview(
    window: &tao::window::Window,
    config: &WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
) -> Result<WryWebView, Box<dyn std::error::Error>> {
    let mut builder = WryWebViewBuilder::new();
    
    // 1. Register built-in auroraview:// protocol
    if let Some(asset_root) = &config.asset_root {
        let asset_root = asset_root.clone();
        builder = builder.with_custom_protocol("auroraview".into(), move |_id, request| {
            handle_auroraview_protocol(&asset_root, request)
        });
    }
    
    // 2. Register custom protocols
    for (scheme, handler) in &config.custom_protocols {
        let handler = handler.clone();
        let scheme = scheme.clone();
        builder = builder.with_custom_protocol(scheme, move |_id, request| {
            handle_custom_protocol(&handler, request)
        });
    }
    
    // ... other configuration
}
```

## Advantages

1. **No CORS restrictions** - Custom protocols bypass browser CORS restrictions
2. **Simple API** - Python function registers as protocol handler
3. **Flexible** - Load resources from files, memory, database, or any source
4. **Secure** - Each protocol independently controls access permissions
5. **High performance** - Direct file reading, no HTTP server overhead

## MIME Type Reference

Common MIME types for resources:

| Extension | MIME Type |
|-----------|-----------|
| `.html` | `text/html` |
| `.css` | `text/css` |
| `.js` | `application/javascript` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.jpg`, `.jpeg` | `image/jpeg` |
| `.gif` | `image/gif` |
| `.svg` | `image/svg+xml` |
| `.woff` | `font/woff` |
| `.woff2` | `font/woff2` |
| `.ttf` | `font/ttf` |
| `.fbx`, `.obj` | `application/octet-stream` |

## Security Considerations

When implementing custom protocol handlers:

1. **Validate all URIs** - Sanitize input to prevent injection attacks
2. **Sanitize file paths** - Prevent directory traversal (`../`)
3. **Restrict access** - Only allow access to intended directories
4. **Handle errors gracefully** - Return proper status codes for invalid requests
