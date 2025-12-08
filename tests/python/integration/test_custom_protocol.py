"""
Test custom protocol handlers for AuroraView

Tests the built-in auroraview:// protocol and custom protocol registration.
"""

import os
import tempfile
from pathlib import Path

import pytest

# Skip WebView creation tests in CI - these require display environment
_skip_webview_in_ci = pytest.mark.skipif(
    os.environ.get("CI") == "true",
    reason="WebView creation requires display environment, skipped in CI",
)


@_skip_webview_in_ci
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


@_skip_webview_in_ci
@pytest.mark.skip(reason="register_protocol not yet exposed to Python API")
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


@_skip_webview_in_ci
@pytest.mark.skip(reason="register_protocol not yet exposed to Python API")
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


@_skip_webview_in_ci
@pytest.mark.skip(reason="register_protocol not yet exposed to Python API")
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


@_skip_webview_in_ci
@pytest.mark.skip(reason="register_protocol not yet exposed to Python API")
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


@_skip_webview_in_ci
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


class TestRewriteHtmlForCustomProtocol:
    """Tests for the rewrite_html_for_custom_protocol function."""

    def test_rewrite_relative_paths(self):
        """Test that relative paths are rewritten to auroraview:// protocol."""
        from auroraview import rewrite_html_for_custom_protocol

        html = """
        <html>
            <head>
                <link rel="stylesheet" href="./style.css">
                <link rel="stylesheet" href="styles/main.css">
            </head>
            <body>
                <script src="./script.js"></script>
                <script src="js/app.js"></script>
                <img src="./logo.png">
                <img src="images/icon.png">
            </body>
        </html>
        """

        result = rewrite_html_for_custom_protocol(html)

        # Check that relative paths are rewritten
        assert 'href="auroraview://style.css"' in result  # ./style.css -> style.css
        assert 'href="auroraview://styles/main.css"' in result
        assert 'src="auroraview://script.js"' in result  # ./script.js -> script.js
        assert 'src="auroraview://js/app.js"' in result
        assert 'src="auroraview://logo.png"' in result  # ./logo.png -> logo.png
        assert 'src="auroraview://images/icon.png"' in result

    def test_preserve_absolute_urls(self):
        """Test that absolute URLs are not rewritten."""
        from auroraview import rewrite_html_for_custom_protocol

        html = """
        <html>
            <head>
                <link rel="stylesheet" href="https://cdn.example.com/style.css">
                <link rel="stylesheet" href="http://example.com/style.css">
            </head>
            <body>
                <script src="https://cdn.example.com/script.js"></script>
                <img src="data:image/png;base64,ABC123">
                <img src="//cdn.example.com/image.png">
            </body>
        </html>
        """

        result = rewrite_html_for_custom_protocol(html)

        # Check that absolute URLs are preserved
        assert 'href="https://cdn.example.com/style.css"' in result
        assert 'href="http://example.com/style.css"' in result
        assert 'src="https://cdn.example.com/script.js"' in result
        assert 'src="data:image/png;base64,ABC123"' in result
        assert 'src="//cdn.example.com/image.png"' in result

    def test_preserve_existing_auroraview_protocol(self):
        """Test that existing auroraview:// URLs are not double-rewritten."""
        from auroraview import rewrite_html_for_custom_protocol

        html = """
        <html>
            <head>
                <link rel="stylesheet" href="auroraview://style.css">
            </head>
            <body>
                <script src="auroraview://script.js"></script>
            </body>
        </html>
        """

        result = rewrite_html_for_custom_protocol(html)

        # Check that auroraview:// URLs are preserved (not double-rewritten)
        assert 'href="auroraview://style.css"' in result
        assert 'src="auroraview://script.js"' in result
        # Make sure there's no double protocol
        assert "auroraview://auroraview://" not in result

    def test_rewrite_parent_directory_paths(self):
        """Test that parent directory paths (../) are rewritten."""
        from auroraview import rewrite_html_for_custom_protocol

        html = """
        <html>
            <head>
                <link rel="stylesheet" href="../assets/style.css">
            </head>
            <body>
                <script src="../js/app.js"></script>
                <img src="../../images/logo.png">
            </body>
        </html>
        """

        result = rewrite_html_for_custom_protocol(html)

        # Check that parent directory paths are rewritten
        assert 'href="auroraview://../assets/style.css"' in result
        assert 'src="auroraview://../js/app.js"' in result
        assert 'src="auroraview://../../images/logo.png"' in result

    def test_rewrite_css_url(self):
        """Test that CSS url() references are rewritten."""
        from auroraview import rewrite_html_for_custom_protocol

        html = """
        <style>
            body {
                background: url('./images/bg.png');
            }
            .icon {
                background-image: url(icons/icon.svg);
            }
        </style>
        """

        result = rewrite_html_for_custom_protocol(html)

        # Check that CSS url() references are rewritten
        assert 'url("auroraview://images/bg.png")' in result
        assert 'url("auroraview://icons/icon.svg")' in result

    def test_preserve_anchor_links(self):
        """Test that anchor links (#) are not rewritten."""
        from auroraview import rewrite_html_for_custom_protocol

        html = """
        <html>
            <body>
                <a href="#section1">Go to section 1</a>
                <a href="#top">Back to top</a>
            </body>
        </html>
        """

        result = rewrite_html_for_custom_protocol(html)

        # Check that anchor links are preserved
        assert 'href="#section1"' in result
        assert 'href="#top"' in result


class TestLoadLocalHtml:
    """Tests for the load_local_html method."""

    def test_load_local_html_with_rewriting(self):
        """Test loading local HTML file with path rewriting."""
        from auroraview import WebView

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create HTML file with relative paths
            html_file = Path(tmpdir) / "index.html"
            html_file.write_text("""
            <html>
                <head>
                    <link rel="stylesheet" href="./style.css">
                </head>
                <body>
                    <script src="./script.js"></script>
                </body>
            </html>
            """)

            # Create asset files
            (Path(tmpdir) / "style.css").write_text("body { color: red; }")
            (Path(tmpdir) / "script.js").write_text("console.log('test');")

            # Create WebView with asset_root
            webview = WebView(
                title="Local HTML Test",
                asset_root=str(tmpdir),
            )

            # Load local HTML with rewriting
            webview.load_local_html(html_file)

            # Verify the HTML was loaded with rewritten paths
            assert webview._stored_html is not None
            assert 'href="auroraview://style.css"' in webview._stored_html
            assert 'src="auroraview://script.js"' in webview._stored_html

    def test_load_local_html_without_rewriting(self):
        """Test loading local HTML file without path rewriting."""
        from auroraview import WebView

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create HTML file with relative paths
            html_file = Path(tmpdir) / "index.html"
            html_file.write_text("""
            <html>
                <head>
                    <link rel="stylesheet" href="./style.css">
                </head>
            </html>
            """)

            webview = WebView(title="No Rewrite Test")

            # Load local HTML without rewriting
            webview.load_local_html(html_file, rewrite_paths=False)

            # Verify the HTML was loaded without rewriting
            assert webview._stored_html is not None
            assert 'href="./style.css"' in webview._stored_html

    def test_load_local_html_file_not_found(self):
        """Test that FileNotFoundError is raised for missing files."""
        from auroraview import WebView

        webview = WebView(title="Error Test")

        with pytest.raises(FileNotFoundError):
            webview.load_local_html("/nonexistent/path/index.html")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
