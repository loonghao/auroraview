"""
Test custom protocol handlers for AuroraView

Tests the built-in auroraview:// protocol and custom protocol registration.
"""

import tempfile
from pathlib import Path

import pytest


def test_auroraview_protocol_basic():
    """Test basic auroraview:// protocol with asset_root"""
    from auroraview import WebView

    # Create temporary asset directory
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test files
        css_dir = Path(tmpdir) / "css"
        css_dir.mkdir()
        (css_dir / "style.css").write_text("body { color: red; }")

        js_dir = Path(tmpdir) / "js"
        js_dir.mkdir()
        (js_dir / "app.js").write_text("console.log('test');")

        # Create WebView with asset_root
        webview = WebView(
            title="Protocol Test",
            asset_root=str(tmpdir),
            html="""
            <html>
                <head>
                    <link rel="stylesheet" href="auroraview://css/style.css">
                </head>
                <body>
                    <h1>Test</h1>
                    <script src="auroraview://js/app.js"></script>
                </body>
            </html>
            """,
        )

        # Verify WebView was created
        assert webview is not None
        assert webview.title == "Protocol Test"


def test_custom_protocol_registration():
    """Test custom protocol registration"""
    from auroraview import WebView

    # Track calls to handler
    calls = []

    def handle_test_protocol(uri: str) -> dict:
        """Test protocol handler"""
        calls.append(uri)

        if uri == "test://hello.txt":
            return {"data": b"Hello, World!", "mime_type": "text/plain", "status": 200}
        else:
            return {"data": b"Not Found", "mime_type": "text/plain", "status": 404}

    # Create WebView and register protocol
    webview = WebView(title="Custom Protocol Test")
    webview.register_protocol("test", handle_test_protocol)

    # Verify WebView was created
    assert webview is not None


def test_custom_protocol_with_file_loading():
    """Test custom protocol that loads actual files"""
    from auroraview import WebView

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test file
        test_file = Path(tmpdir) / "data.json"
        test_file.write_text('{"message": "Hello from custom protocol"}')

        def handle_data_protocol(uri: str) -> dict:
            """Load files from tmpdir"""
            path = uri.replace("data://", "")
            full_path = Path(tmpdir) / path

            try:
                with open(full_path, "rb") as f:
                    return {"data": f.read(), "mime_type": "application/json", "status": 200}
            except FileNotFoundError:
                return {"data": b"Not Found", "mime_type": "text/plain", "status": 404}

        # Create WebView and register protocol
        webview = WebView(title="File Protocol Test")
        webview.register_protocol("data", handle_data_protocol)

        # Load HTML that uses the protocol
        webview.load_html("""
        <html>
            <body>
                <h1>Custom Protocol Test</h1>
                <script>
                    fetch('data://data.json')
                        .then(r => r.json())
                        .then(data => console.log(data.message));
                </script>
            </body>
        </html>
        """)

        assert webview is not None


def test_protocol_error_handling():
    """Test protocol handler error handling"""
    from auroraview import WebView

    def handle_error_protocol(uri: str) -> dict:
        """Protocol that returns errors"""
        if "error" in uri:
            return {"data": b"Internal Server Error", "mime_type": "text/plain", "status": 500}
        else:
            return {"data": b"OK", "mime_type": "text/plain", "status": 200}

    webview = WebView(title="Error Test")
    webview.register_protocol("error", handle_error_protocol)

    assert webview is not None


def test_multiple_protocols():
    """Test registering multiple custom protocols"""
    from auroraview import WebView

    def handle_protocol_a(uri: str) -> dict:
        return {"data": b"Protocol A", "mime_type": "text/plain", "status": 200}

    def handle_protocol_b(uri: str) -> dict:
        return {"data": b"Protocol B", "mime_type": "text/plain", "status": 200}

    webview = WebView(title="Multiple Protocols")
    webview.register_protocol("prota", handle_protocol_a)
    webview.register_protocol("protb", handle_protocol_b)

    assert webview is not None


def test_asset_root_with_subdirectories():
    """Test auroraview:// protocol with nested directories"""
    from auroraview import WebView

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create nested structure
        assets = Path(tmpdir) / "assets"
        images = assets / "images" / "icons"
        images.mkdir(parents=True)

        (images / "logo.png").write_bytes(b"PNG_DATA")

        webview = WebView(
            title="Nested Assets",
            asset_root=str(assets),
            html='<img src="auroraview://images/icons/logo.png">',
        )

        assert webview is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
