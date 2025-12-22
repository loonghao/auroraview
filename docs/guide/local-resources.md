# Local Resource Loading

This guide compares different approaches for loading local resources (images, CSS, JS, fonts) in WebView applications.

## Overview

| Method | CORS | Complexity | Performance | Security | Recommendation |
|--------|------|------------|-------------|----------|----------------|
| `file://` | Limited | Simple | Good | Low | Development only |
| Data URL | None | Simple | Medium | High | Small resources |
| HTTP Server | None | Complex | Medium | Low | Development |
| Custom Protocol | None | Medium | Good | High | **Production** |

## Method 1: File URL (`file://`)

```python
from auroraview import WebView

webview = WebView(
    title="Local Resources",
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="file:///C:/projects/my_app/style.css">
        </head>
        <body>
            <img src="file:///C:/projects/my_app/logo.png">
            <script src="file:///C:/projects/my_app/app.js"></script>
        </body>
    </html>
    """
)
```

**Pros:**
- Simple, no extra configuration
- All platforms supported
- Direct file system access

**Cons:**
- **CORS restrictions** - Cannot use fetch/XHR for local files
- **Security limitations** - Modern browsers restrict `file://`
- **Path issues** - Absolute paths differ across environments
- **Cross-platform** - Windows (`C:\`) vs Unix (`/home/`)

**CORS Problem Example:**
```javascript
// This will fail!
fetch('file:///C:/data/config.json')
    .then(r => r.json())
    .catch(e => console.error('CORS error:', e));
```

## Method 2: Data URL (Base64)

```python
import base64

# Read and encode image
with open('logo.png', 'rb') as f:
    logo_data = base64.b64encode(f.read()).decode()

webview = WebView(
    html=f"""
    <html>
        <body>
            <img src="data:image/png;base64,{logo_data}">
        </body>
    </html>
    """
)
```

**Pros:**
- No CORS restrictions
- Single file distribution
- Cross-platform consistent

**Cons:**
- **33% size increase** - Base64 encoding overhead
- **Large HTML files** - Not suitable for many resources
- **No caching** - Resources reloaded every time
- **Not for large files** - Videos, large images

## Method 3: Local HTTP Server

```python
from auroraview import WebView
import http.server
import threading
import os

# Start local server
def start_server():
    os.chdir('/path/to/resources')
    server = http.server.HTTPServer(('localhost', 8080), 
                                     http.server.SimpleHTTPRequestHandler)
    server.serve_forever()

threading.Thread(target=start_server, daemon=True).start()

# Use http:// to load resources
webview = WebView(
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="http://localhost:8080/style.css">
        </head>
        <body>
            <img src="http://localhost:8080/logo.png">
            <script src="http://localhost:8080/app.js"></script>
        </body>
    </html>
    """
)
```

**Pros:**
- **No CORS restrictions** - Free use of fetch/XHR
- **Full HTTP features** - Caching, compression, range requests
- **Good dev experience** - Similar to web development

**Cons:**
- **Extra process** - Manage server lifecycle
- **Port conflicts** - Need dynamic port allocation
- **Security risk** - Other processes can access
- **Complexity** - Handle startup, shutdown, errors

## Method 4: Custom Protocol (Recommended)

```python
from auroraview import WebView

webview = WebView(
    title="Custom Protocol",
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="asset://style.css">
        </head>
        <body>
            <img src="asset://images/logo.png">
            <script src="asset://js/app.js"></script>
        </body>
    </html>
    """,
    # Configure resource root directory
    asset_root="/path/to/resources"
)
```

**Pros:**
- **No CORS restrictions** - Custom protocols treated as same-origin
- **Secure** - Only access specified directory
- **Clean URLs** - `asset://logo.png` vs `file:///C:/long/path/logo.png`
- **Cross-platform** - Path handling unified in Rust
- **Flexible** - Load from memory, database, network
- **Good performance** - Direct file read, no HTTP overhead

**Cons:**
- Requires implementation in Rust backend
- One-time configuration at WebView creation

### Rust Implementation

```rust
// In backend/native.rs
webview_builder.with_custom_protocol("asset".into(), move |_id, request| {
    let path = request.uri().path().trim_start_matches('/');
    let full_path = asset_root.join(path);
    
    match std::fs::read(&full_path) {
        Ok(data) => {
            let mime = mime_guess::from_path(&full_path)
                .first_or_octet_stream();
            
            http::Response::builder()
                .header("Content-Type", mime.as_ref())
                .body(data.into())
                .unwrap()
        }
        Err(_) => {
            http::Response::builder()
                .status(404)
                .body(b"Not Found".to_vec().into())
                .unwrap()
        }
    }
})
```

## Practical Recommendations

### Simple Applications (Few Resources)

**Use**: Data URL
```python
webview = WebView(html=f"""
    <style>{css_content}</style>
    <img src="data:image/png;base64,{logo_base64}">
""")
```

### Development Phase

**Use**: Local HTTP Server
```python
# Python built-in server
# Benefits: Hot reload, easy debugging
```

### Production (DCC Integration)

**Use**: Custom Protocol
```python
# Maya/Houdini plugin
webview = WebView(
    html="""
    <link rel="stylesheet" href="dcc://ui/style.css">
    <img src="dcc://icons/tool.png">
    """,
    asset_root=os.path.join(os.path.dirname(__file__), "resources")
)
```

**Why**:
- No CORS issues - Free use of fetch
- Secure - Only access plugin directory
- Clean - URLs don't expose file system paths
- Flexible - Can load from Maya scene, database

## Real-World Example: Maya Plugin UI

### Using file:// (Problematic)

```python
import os

plugin_dir = os.path.dirname(__file__)

webview = WebView(
    html=f"""
    <html>
        <head>
            <link rel="stylesheet" href="file:///{plugin_dir}/ui/style.css">
        </head>
        <body>
            <img src="file:///{plugin_dir}/icons/logo.png">
            <script>
                // This will fail! CORS error
                fetch('file:///{plugin_dir}/data/config.json')
                    .then(r => r.json())
                    .catch(e => console.error('CORS error:', e));
            </script>
        </body>
    </html>
    """
)
```

**Problems**:
- CORS blocks fetch for local files
- Windows paths need conversion
- Exposes file system structure

### Using Custom Protocol (Best)

```python
from auroraview import WebView
import os

plugin_dir = os.path.dirname(__file__)

webview = WebView(
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="maya://ui/style.css">
        </head>
        <body>
            <img src="maya://icons/logo.png">
            <script>
                // Works perfectly! No CORS restrictions
                fetch('maya://data/config.json')
                    .then(r => r.json())
                    .then(data => console.log(data));

                // Can load scene resources
                fetch('maya://scenes/current/metadata.json')
                    .then(r => r.json())
                    .then(meta => updateUI(meta));
            </script>
        </body>
    </html>
    """,
    asset_root=plugin_dir
)
```

**Benefits**:
- No CORS restrictions
- Clean URLs
- Secure (only access plugin directory)
- Cross-platform consistent
- No extra processes

## Summary

Custom Protocol is the **recommended approach** for production DCC integrations. It provides:

- Clean, simple URLs
- No CORS limitations
- Secure, controlled access
- Great developer experience

This is the same approach used by Tauri, Electron, and other modern WebView frameworks.
