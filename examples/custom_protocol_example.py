"""
Custom Protocol Example - AuroraView

This example demonstrates how to use custom protocols to load resources
without CORS restrictions.

Features:
1. Built-in auroraview:// protocol for static assets
2. Custom fbx:// protocol for loading FBX files
3. Custom maya:// protocol for Maya scene thumbnails
"""

import os
import sys

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

from auroraview import AuroraView


def handle_fbx_protocol(uri: str) -> dict:
    """
    Handle fbx:// protocol requests

    Args:
        uri: Full URI like "fbx://models/character.fbx"

    Returns:
        dict with keys: data (bytes), mime_type (str), status (int)
    """
    print(f"[FBX Protocol] Request: {uri}")

    # Extract path from URI
    path = uri.replace("fbx://", "")

    # FBX root directory (change this to your actual path)
    fbx_root = "C:/projects/models"
    full_path = os.path.join(fbx_root, path)

    try:
        with open(full_path, "rb") as f:
            data = f.read()

        print(f"[FBX Protocol] Loaded {len(data)} bytes from {full_path}")

        return {"data": data, "mime_type": "application/octet-stream", "status": 200}
    except FileNotFoundError:
        print(f"[FBX Protocol] File not found: {full_path}")
        return {"data": b"Not Found", "mime_type": "text/plain", "status": 404}


def handle_maya_protocol(uri: str) -> dict:
    """
    Handle maya:// protocol requests

    Args:
        uri: Full URI like "maya://thumbnails/character.jpg"

    Returns:
        dict with keys: data (bytes), mime_type (str), status (int)
    """
    print(f"[Maya Protocol] Request: {uri}")

    # Extract path from URI
    path = uri.replace("maya://", "")

    # Maya project directory (change this to your actual path)
    maya_project = "C:/maya_projects/current"
    full_path = os.path.join(maya_project, path)

    try:
        with open(full_path, "rb") as f:
            data = f.read()

        print(f"[Maya Protocol] Loaded {len(data)} bytes from {full_path}")

        # Determine MIME type from extension
        ext = os.path.splitext(path)[1].lower()
        mime_types = {
            ".jpg": "image/jpeg",
            ".jpeg": "image/jpeg",
            ".png": "image/png",
            ".gif": "image/gif",
        }
        mime_type = mime_types.get(ext, "application/octet-stream")

        return {"data": data, "mime_type": mime_type, "status": 200}
    except FileNotFoundError:
        print(f"[Maya Protocol] File not found: {full_path}")
        return {"data": b"Not Found", "mime_type": "text/plain", "status": 404}


def main():
    """Main example"""

    # Create asset directory for auroraview:// protocol
    asset_root = os.path.join(os.path.dirname(__file__), "assets")
    os.makedirs(asset_root, exist_ok=True)
    os.makedirs(os.path.join(asset_root, "css"), exist_ok=True)
    os.makedirs(os.path.join(asset_root, "js"), exist_ok=True)

    # Create sample CSS file
    with open(os.path.join(asset_root, "css", "style.css"), "w") as f:
        f.write("""
body {
    font-family: Arial, sans-serif;
    margin: 20px;
    background: #f0f0f0;
}
h1 {
    color: #333;
}
.protocol-demo {
    background: white;
    padding: 20px;
    border-radius: 8px;
    margin: 10px 0;
}
        """)

    # Create sample JS file
    with open(os.path.join(asset_root, "js", "app.js"), "w") as f:
        f.write("""
console.log('App loaded via auroraview:// protocol!');

// Test custom protocols
async function testProtocols() {
    console.log('Testing custom protocols...');

    // This would work if you have actual FBX files
    // fetch('fbx://models/character.fbx')
    //     .then(r => r.arrayBuffer())
    //     .then(data => console.log('FBX loaded:', data.byteLength, 'bytes'));
}

testProtocols();
        """)

    # Create WebView with asset_root
    webview = AuroraView(
        title="Custom Protocol Demo",
        width=1024,
        height=768,
        asset_root=asset_root,  # Enable auroraview:// protocol
    )

    # Register custom protocols
    webview.register_protocol("fbx", handle_fbx_protocol)
    webview.register_protocol("maya", handle_maya_protocol)

    print("[Main] Registered protocols: auroraview, fbx, maya")
    print(f"[Main] Asset root: {asset_root}")

    # Load HTML that uses all protocols
    html = """
    <html>
        <head>
            <title>Custom Protocol Demo</title>
            <link rel="stylesheet" href="auroraview://css/style.css">
        </head>
        <body>
            <h1>Custom Protocol Demo</h1>

            <div class="protocol-demo">
                <h2>1. auroraview:// Protocol (Built-in)</h2>
                <p>Loads static assets from asset_root directory</p>
                <p>CSS and JS loaded via auroraview:// protocol ✓</p>
            </div>

            <div class="protocol-demo">
                <h2>2. fbx:// Protocol (Custom)</h2>
                <p>Custom protocol for loading FBX files</p>
                <button onclick="testFBX()">Test FBX Protocol</button>
            </div>

            <div class="protocol-demo">
                <h2>3. maya:// Protocol (Custom)</h2>
                <p>Custom protocol for Maya project resources</p>
                <button onclick="testMaya()">Test Maya Protocol</button>
            </div>

            <script src="auroraview://js/app.js"></script>
            <script>
                function testFBX() {
                    fetch('fbx://models/test.fbx')
                        .then(r => r.arrayBuffer())
                        .then(data => alert('FBX loaded: ' + data.byteLength + ' bytes'))
                        .catch(e => alert('FBX error: ' + e));
                }

                function testMaya() {
                    fetch('maya://thumbnails/test.jpg')
                        .then(r => r.blob())
                        .then(blob => alert('Maya thumbnail loaded: ' + blob.size + ' bytes'))
                        .catch(e => alert('Maya error: ' + e));
                }
            </script>
        </body>
    </html>
    """

    webview.load_html(html)
    webview.show()


if __name__ == "__main__":
    main()
