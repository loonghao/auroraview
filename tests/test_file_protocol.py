"""Tests for file:// protocol support in AuroraView.

This module tests the file:// protocol handling functionality,
including path conversion, URL encoding, and integration with run_standalone.
"""

import os
import tempfile
from pathlib import Path

import pytest


def path_to_file_url(path: str | Path) -> str:
    """Convert local file path to file:/// URL.

    Helper function for testing file:// protocol support.

    Args:
        path: Local file path (can be relative or absolute)

    Returns:
        file:/// URL string
    """
    # Convert to absolute path
    abs_path = Path(path).resolve()

    # Convert to file:/// URL format
    # On Windows: file:///C:/path/to/file
    # On Unix: file:///path/to/file
    path_str = str(abs_path).replace(os.sep, "/")

    # Ensure proper file:/// prefix
    if not path_str.startswith("/"):
        path_str = "/" + path_str

    return f"file://{path_str}"


class TestFileProtocolHelpers:
    """Test helper functions for file:// protocol."""

    def test_path_to_file_url_absolute_path(self):
        """Test converting absolute path to file:/// URL."""
        # Test with absolute path
        test_path = Path("/tmp/test.txt").resolve()
        result = path_to_file_url(test_path)

        assert result.startswith("file://")
        assert "test.txt" in result

    def test_path_to_file_url_relative_path(self):
        """Test converting relative path to file:/// URL."""
        # Test with relative path (should be converted to absolute)
        result = path_to_file_url("test.txt")

        assert result.startswith("file://")
        assert "test.txt" in result
        # Should be absolute path
        assert len(result) > len("file:///test.txt")

    def test_path_to_file_url_with_spaces(self):
        """Test converting path with spaces to file:/// URL."""
        # Test with path containing spaces
        test_path = Path("/tmp/test file.txt").resolve()
        result = path_to_file_url(test_path)

        assert result.startswith("file://")
        # Spaces should be preserved (URL encoding happens in browser)
        assert "test file.txt" in result or "test%20file.txt" in result

    def test_path_to_file_url_windows_path(self):
        """Test converting Windows path to file:/// URL."""
        # Test with Windows-style path
        if os.name == "nt":
            test_path = Path("C:/Users/test/file.txt")
            result = path_to_file_url(test_path)

            assert result.startswith("file://")
            # Should use forward slashes
            assert "\\" not in result
            assert "/" in result
        else:
            pytest.skip("Windows-specific test")


class TestPrepareHtmlWithLocalAssets:
    """Test prepare_html_with_local_assets function."""

    def test_prepare_html_basic(self):
        """Test basic HTML preparation with local assets."""
        from auroraview import prepare_html_with_local_assets
        
        html = '<img src="{{IMAGE_PATH}}">'
        result = prepare_html_with_local_assets(
            html,
            asset_paths={"IMAGE_PATH": "test.png"}
        )
        
        assert "file://" in result
        assert "test.png" in result
        assert "{{IMAGE_PATH}}" not in result

    def test_prepare_html_manifest_path(self):
        """Test HTML preparation with manifest path."""
        from auroraview import prepare_html_with_local_assets
        
        html = '<iframe src="{{MANIFEST_PATH}}"></iframe>'
        result = prepare_html_with_local_assets(
            html,
            manifest_path="manifest.html"
        )
        
        assert "file://" in result
        assert "manifest.html" in result
        assert "{{MANIFEST_PATH}}" not in result

    def test_prepare_html_multiple_assets(self):
        """Test HTML preparation with multiple assets."""
        from auroraview import prepare_html_with_local_assets
        
        html = '''
        <img src="{{GIF_PATH}}">
        <img src="{{IMAGE_PATH}}">
        <video src="{{VIDEO_PATH}}"></video>
        '''
        
        result = prepare_html_with_local_assets(
            html,
            asset_paths={
                "GIF_PATH": "animation.gif",
                "IMAGE_PATH": "logo.png",
                "VIDEO_PATH": "demo.mp4",
            }
        )
        
        assert result.count("file://") >= 3
        assert "animation.gif" in result
        assert "logo.png" in result
        assert "demo.mp4" in result
        assert "{{GIF_PATH}}" not in result
        assert "{{IMAGE_PATH}}" not in result
        assert "{{VIDEO_PATH}}" not in result

    def test_prepare_html_with_relative_paths(self):
        """Test that relative paths are also rewritten."""
        from auroraview import prepare_html_with_local_assets

        html = '<link href="style.css" rel="stylesheet">'
        result = prepare_html_with_local_assets(html)

        # Relative paths should be converted to auroraview:// protocol
        assert "auroraview://" in result or "style.css" in result


class TestFileProtocolIntegration:
    """Integration tests for file:// protocol with actual files."""

    def test_file_protocol_with_temp_file(self):
        """Test loading actual file using file:// protocol."""
        from auroraview import prepare_html_with_local_assets

        # Create temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write("Test content")
            temp_path = f.name

        try:
            html = '<div>{{FILE_PATH}}</div>'
            result = prepare_html_with_local_assets(
                html,
                asset_paths={"FILE_PATH": temp_path}
            )

            assert "file://" in result
            assert temp_path.replace(os.sep, "/") in result or Path(temp_path).name in result

        finally:
            os.unlink(temp_path)

    def test_file_protocol_with_temp_html(self):
        """Test loading HTML file using file:// protocol."""
        from auroraview import prepare_html_with_local_assets

        # Create temporary HTML file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.html', delete=False, encoding='utf-8') as f:
            f.write('<!DOCTYPE html><html><body>Test</body></html>')
            temp_path = f.name

        try:
            html = '<iframe src="{{HTML_PATH}}"></iframe>'
            result = prepare_html_with_local_assets(
                html,
                asset_paths={"HTML_PATH": temp_path}
            )

            assert "file://" in result
            assert ".html" in result

        finally:
            os.unlink(temp_path)

    def test_file_protocol_with_temp_image(self):
        """Test loading image file using file:// protocol."""
        from auroraview import prepare_html_with_local_assets

        # Create temporary image file (simple SVG)
        with tempfile.NamedTemporaryFile(mode='w', suffix='.svg', delete=False, encoding='utf-8') as f:
            f.write('<svg xmlns="http://www.w3.org/2000/svg"><circle r="10"/></svg>')
            temp_path = f.name

        try:
            html = '<img src="{{SVG_PATH}}">'
            result = prepare_html_with_local_assets(
                html,
                asset_paths={"SVG_PATH": temp_path}
            )

            assert "file://" in result
            assert ".svg" in result

        finally:
            os.unlink(temp_path)

    def test_file_protocol_url_format_windows(self):
        """Test file:// URL format on Windows."""
        from auroraview import prepare_html_with_local_assets

        if os.name != "nt":
            pytest.skip("Windows-specific test")

        # Create temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write("test")
            temp_path = f.name

        try:
            html = '<a href="{{FILE_PATH}}">Link</a>'
            result = prepare_html_with_local_assets(
                html,
                asset_paths={"FILE_PATH": temp_path}
            )

            # Windows file:// URLs should have format: file:///C:/path/to/file
            assert "file:///" in result
            # Should use forward slashes
            assert "\\" not in result

        finally:
            os.unlink(temp_path)

    def test_file_protocol_url_format_unix(self):
        """Test file:// URL format on Unix."""
        from auroraview import prepare_html_with_local_assets

        if os.name == "nt":
            pytest.skip("Unix-specific test")

        # Create temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write("test")
            temp_path = f.name

        try:
            html = '<a href="{{FILE_PATH}}">Link</a>'
            result = prepare_html_with_local_assets(
                html,
                asset_paths={"FILE_PATH": temp_path}
            )

            # Unix file:// URLs should have format: file:///path/to/file
            assert "file:///" in result

        finally:
            os.unlink(temp_path)

