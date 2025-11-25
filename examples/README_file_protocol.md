# Using file:// Protocol in AuroraView

This guide explains how to load local files (images, GIFs, CSS, JS, HTML) in AuroraView using the `file://` protocol.

## Quick Start

### 1. Enable file:// Protocol Support

**IMPORTANT**: You must explicitly enable `file://` protocol support:

```python
from auroraview import run_standalone

run_standalone(
    title="My App",
    html=html_content,
    allow_file_protocol=True,  # ← Required!
)
```

### 2. Convert File Paths to file:/// URLs

Use this helper function to convert local paths to `file://` URLs:

```python
import os
from pathlib import Path

def path_to_file_url(path: str | Path) -> str:
    """Convert local file path to file:/// URL."""
    abs_path = Path(path).resolve()
    path_str = str(abs_path).replace(os.sep, "/")
    
    if not path_str.startswith("/"):
        path_str = "/" + path_str
    
    return f"file://{path_str}"

# Example usage
gif_url = path_to_file_url("templates/animation.gif")
# Result: file:///C:/path/to/templates/animation.gif (Windows)
# Result: file:///home/user/templates/animation.gif (Unix)
```

### 3. Use file:// URLs in HTML

```python
html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href="{path_to_file_url('styles/main.css')}">
    <script src="{path_to_file_url('scripts/app.js')}"></script>
</head>
<body>
    <img src="{path_to_file_url('images/logo.png')}">
    <img src="{path_to_file_url('images/animation.gif')}">
    <iframe src="{path_to_file_url('pages/about.html')}"></iframe>
</body>
</html>
"""

run_standalone(
    title="My App",
    html=html_content,
    allow_file_protocol=True,
)
```

## Complete Example

See `local_assets_example.py` for a complete working example.

## Supported File Types

The `file://` protocol supports all file types:

- **Images**: PNG, JPEG, GIF, SVG, WebP, etc.
- **Stylesheets**: CSS files
- **Scripts**: JavaScript files
- **Documents**: HTML files (can be loaded in iframes)
- **Media**: Video (MP4, WebM), Audio (MP3, WAV)
- **Fonts**: TTF, WOFF, WOFF2
- **Data**: JSON, XML, TXT, etc.

## Security Considerations

⚠️ **Warning**: Enabling `file://` protocol allows the WebView to access **any** file on the local file system that the process has permission to read.

**Best Practices**:

1. Only enable `file://` protocol when necessary
2. Use it for trusted content only
3. Consider using `auroraview://` protocol for better security (restricts access to specific directories)
4. Validate and sanitize any user-provided file paths

## Alternative: auroraview:// Protocol

For better security, use the `auroraview://` protocol which restricts file access to a specific directory:

```python
from auroraview import WebView

webview = WebView.create(
    title="My App",
    html='<img src="auroraview://images/logo.png">',
    asset_root="/path/to/assets",  # Only files in this directory are accessible
)
```

## CLI Usage

You can also use `file://` protocol from the command line:

```bash
# Enable file:// protocol support
auroraview --html index.html --allow-file-protocol

# Or with URL
auroraview --url "file:///C:/path/to/index.html" --allow-file-protocol
```

## Troubleshooting

### File not loading?

1. **Check if `allow_file_protocol=True` is set**
2. **Verify the file path is correct** (use absolute paths)
3. **Check file permissions** (ensure the process can read the file)
4. **Open DevTools** (`dev_tools=True`) and check the Console for errors
5. **Verify the URL format**:
   - Windows: `file:///C:/path/to/file.ext`
   - Unix: `file:///path/to/file.ext`

### CORS errors?

`file://` protocol doesn't have CORS restrictions, but if you're loading resources from different origins (e.g., mixing `file://` and `http://`), you may encounter issues.

**Solution**: Keep all resources on the same protocol (all `file://` or all `http://`).

## Examples

- `local_assets_example.py` - Complete example with GIF and HTML loading
- See also: `auroraview --help` for CLI options

