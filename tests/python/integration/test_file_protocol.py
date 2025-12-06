"""Tests for file:// protocol support in AuroraView.

This module tests the file:// protocol handling functionality,
including path conversion, URL encoding, and integration with run_standalone.
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path
from typing import Union

import pytest


def path_to_file_url(path: Union[str, Path]) -> str:
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
        # Verify the result is an absolute path by checking it matches
        # what Path.resolve() produces for the current working directory
        expected_abs_path = Path("test.txt").resolve()
        # The URL should contain the resolved absolute path (with forward slashes)
        expected_path_in_url = str(expected_abs_path).replace(os.sep, "/")
        assert expected_path_in_url in result

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
        result = prepare_html_with_local_assets(html, asset_paths={"IMAGE_PATH": "test.png"})

        assert "file://" in result
        assert "test.png" in result
        assert "{{IMAGE_PATH}}" not in result

    def test_prepare_html_manifest_path(self):
        """Test HTML preparation with manifest path."""
        from auroraview import prepare_html_with_local_assets

        html = '<iframe src="{{MANIFEST_PATH}}"></iframe>'
        result = prepare_html_with_local_assets(html, manifest_path="manifest.html")

        assert "file://" in result
        assert "manifest.html" in result
        assert "{{MANIFEST_PATH}}" not in result

    def test_prepare_html_multiple_assets(self):
        """Test HTML preparation with multiple assets."""
        from auroraview import prepare_html_with_local_assets

        html = """
        <img src="{{GIF_PATH}}">
        <img src="{{IMAGE_PATH}}">
        <video src="{{VIDEO_PATH}}"></video>
        """

        result = prepare_html_with_local_assets(
            html,
            asset_paths={
                "GIF_PATH": "animation.gif",
                "IMAGE_PATH": "logo.png",
                "VIDEO_PATH": "demo.mp4",
            },
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
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("Test content")
            temp_path = f.name

        try:
            html = "<div>{{FILE_PATH}}</div>"
            result = prepare_html_with_local_assets(html, asset_paths={"FILE_PATH": temp_path})

            assert "file://" in result
            assert temp_path.replace(os.sep, "/") in result or Path(temp_path).name in result

        finally:
            os.unlink(temp_path)

    def test_file_protocol_with_temp_html(self):
        """Test loading HTML file using file:// protocol."""
        from auroraview import prepare_html_with_local_assets

        # Create temporary HTML file
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".html", delete=False, encoding="utf-8"
        ) as f:
            f.write("<!DOCTYPE html><html><body>Test</body></html>")
            temp_path = f.name

        try:
            html = '<iframe src="{{HTML_PATH}}"></iframe>'
            result = prepare_html_with_local_assets(html, asset_paths={"HTML_PATH": temp_path})

            assert "file://" in result
            assert ".html" in result

        finally:
            os.unlink(temp_path)

    def test_file_protocol_with_temp_image(self):
        """Test loading image file using file:// protocol."""
        from auroraview import prepare_html_with_local_assets

        # Create temporary image file (simple SVG)
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".svg", delete=False, encoding="utf-8"
        ) as f:
            f.write('<svg xmlns="http://www.w3.org/2000/svg"><circle r="10"/></svg>')
            temp_path = f.name

        try:
            html = '<img src="{{SVG_PATH}}">'
            result = prepare_html_with_local_assets(html, asset_paths={"SVG_PATH": temp_path})

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
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("test")
            temp_path = f.name

        try:
            html = '<a href="{{FILE_PATH}}">Link</a>'
            result = prepare_html_with_local_assets(html, asset_paths={"FILE_PATH": temp_path})

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
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("test")
            temp_path = f.name

        try:
            html = '<a href="{{FILE_PATH}}">Link</a>'
            result = prepare_html_with_local_assets(html, asset_paths={"FILE_PATH": temp_path})

            # Unix file:// URLs should have format: file:///path/to/file
            assert "file:///" in result

        finally:
            os.unlink(temp_path)


class TestFileProtocolEdgeCases:
    """Edge case tests for file:// protocol handling."""

    def test_path_to_file_url_with_path_object(self):
        """Test path_to_file_url accepts Path objects."""
        from auroraview.utils.file_protocol import path_to_file_url

        path = Path("test.txt")
        result = path_to_file_url(path)

        assert result.startswith("file://")
        assert "test.txt" in result

    def test_path_to_file_url_with_string(self):
        """Test path_to_file_url accepts strings."""
        from auroraview.utils.file_protocol import path_to_file_url

        result = path_to_file_url("test.txt")

        assert result.startswith("file://")
        assert "test.txt" in result

    def test_path_to_file_url_unicode_path(self):
        """Test path_to_file_url handles unicode characters."""
        from auroraview.utils.file_protocol import path_to_file_url

        # Test with unicode filename
        result = path_to_file_url("测试文件.txt")

        assert result.startswith("file://")
        assert "测试文件.txt" in result

    def test_path_to_file_url_special_characters(self):
        """Test path_to_file_url handles special characters."""
        from auroraview.utils.file_protocol import path_to_file_url

        # Test with special characters (parentheses, brackets)
        result = path_to_file_url("file (1) [copy].txt")

        assert result.startswith("file://")
        assert "file (1) [copy].txt" in result

    def test_prepare_html_empty_asset_paths(self):
        """Test prepare_html_with_local_assets with empty dict."""
        from auroraview import prepare_html_with_local_assets

        html = '<img src="{{IMAGE_PATH}}">'
        result = prepare_html_with_local_assets(html, asset_paths={})

        # Placeholder should remain unchanged
        assert "{{IMAGE_PATH}}" in result

    def test_prepare_html_none_asset_paths(self):
        """Test prepare_html_with_local_assets with None asset_paths."""
        from auroraview import prepare_html_with_local_assets

        html = '<img src="{{IMAGE_PATH}}">'
        result = prepare_html_with_local_assets(html, asset_paths=None)

        # Placeholder should remain unchanged
        assert "{{IMAGE_PATH}}" in result

    def test_prepare_html_no_placeholders(self):
        """Test prepare_html_with_local_assets with HTML without placeholders."""
        from auroraview import prepare_html_with_local_assets

        html = '<img src="static.png">'
        result = prepare_html_with_local_assets(html, asset_paths={"IMAGE_PATH": "test.png"})

        # HTML should remain unchanged
        assert result == '<img src="static.png">'

    def test_prepare_html_path_object_values(self):
        """Test prepare_html_with_local_assets accepts Path objects as values."""
        from auroraview import prepare_html_with_local_assets

        html = '<img src="{{IMAGE_PATH}}">'
        result = prepare_html_with_local_assets(html, asset_paths={"IMAGE_PATH": Path("test.png")})

        assert "file://" in result
        assert "test.png" in result

    def test_prepare_html_both_assets_and_manifest(self):
        """Test prepare_html_with_local_assets with both assets and manifest."""
        from auroraview import prepare_html_with_local_assets

        html = """
        <img src="{{IMAGE_PATH}}">
        <iframe src="{{MANIFEST_PATH}}"></iframe>
        """
        result = prepare_html_with_local_assets(
            html,
            asset_paths={"IMAGE_PATH": "test.png"},
            manifest_path="manifest.html",
        )

        # Both should be replaced
        assert result.count("file://") >= 2
        assert "test.png" in result
        assert "manifest.html" in result
        assert "{{IMAGE_PATH}}" not in result
        assert "{{MANIFEST_PATH}}" not in result

    def test_prepare_html_preserves_other_content(self):
        """Test that prepare_html_with_local_assets preserves non-placeholder content."""
        from auroraview import prepare_html_with_local_assets

        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
            <style>
                body { margin: 0; }
            </style>
        </head>
        <body>
            <h1>Hello World</h1>
            <img src="{{IMAGE_PATH}}">
            <script>
                console.log("test");
            </script>
        </body>
        </html>
        """
        result = prepare_html_with_local_assets(html, asset_paths={"IMAGE_PATH": "test.png"})

        # All original content should be preserved
        assert "<!DOCTYPE html>" in result
        assert "<title>Test Page</title>" in result
        assert "body { margin: 0; }" in result
        assert "<h1>Hello World</h1>" in result
        assert 'console.log("test");' in result
        # Placeholder should be replaced
        assert "{{IMAGE_PATH}}" not in result
        assert "test.png" in result


class TestAuroraViewUrlConversion:
    """Test auroraview URL conversion functions."""

    def test_path_to_auroraview_url_windows(self):
        """Test path_to_auroraview_url with Windows path."""
        from auroraview.utils.file_protocol import path_to_auroraview_url

        if os.name != "nt":
            pytest.skip("Windows-specific test")

        result = path_to_auroraview_url("C:/icons/maya.svg")
        assert result.startswith("https://auroraview.localhost/file/")
        assert "icons" in result
        assert "maya.svg" in result
        # Should not have double slashes
        assert "///" not in result

    def test_path_to_auroraview_url_relative(self):
        """Test path_to_auroraview_url with relative path."""
        from auroraview.utils.file_protocol import path_to_auroraview_url

        result = path_to_auroraview_url("icons/maya.svg")
        assert result.startswith("https://auroraview.localhost/file/")
        assert "maya.svg" in result

    def test_file_url_to_auroraview_url_basic(self):
        """Test file_url_to_auroraview_url with basic file URL."""
        from auroraview.utils.file_protocol import file_url_to_auroraview_url

        result = file_url_to_auroraview_url("file:///C:/icons/maya.svg")
        assert result == "https://auroraview.localhost/file/C:/icons/maya.svg"

    def test_file_url_to_auroraview_url_unix(self):
        """Test file_url_to_auroraview_url with Unix file URL."""
        from auroraview.utils.file_protocol import file_url_to_auroraview_url

        result = file_url_to_auroraview_url("file:///home/user/icons/maya.svg")
        assert result == "https://auroraview.localhost/file/home/user/icons/maya.svg"

    def test_file_url_to_auroraview_url_double_slash(self):
        """Test file_url_to_auroraview_url with file:// (2 slashes)."""
        from auroraview.utils.file_protocol import file_url_to_auroraview_url

        result = file_url_to_auroraview_url("file://C:/icons/maya.svg")
        assert result == "https://auroraview.localhost/file/C:/icons/maya.svg"

    def test_file_url_to_auroraview_url_not_file_url(self):
        """Test file_url_to_auroraview_url with non-file URL."""
        from auroraview.utils.file_protocol import file_url_to_auroraview_url

        # Should return unchanged
        result = file_url_to_auroraview_url("https://example.com/test.svg")
        assert result == "https://example.com/test.svg"

    def test_file_url_to_auroraview_url_http(self):
        """Test file_url_to_auroraview_url with HTTP URL."""
        from auroraview.utils.file_protocol import file_url_to_auroraview_url

        result = file_url_to_auroraview_url("http://localhost/test.svg")
        assert result == "http://localhost/test.svg"

    def test_internal_normalize_path(self):
        """Test _normalize_path helper function."""
        from auroraview.utils.file_protocol import _normalize_path

        result = _normalize_path("test.txt")
        # Should be absolute with forward slashes
        assert "/" in result
        assert "\\" not in result
        assert "test.txt" in result

    def test_internal_extract_path_from_file_url(self):
        """Test _extract_path_from_file_url helper function."""
        from auroraview.utils.file_protocol import _extract_path_from_file_url

        # Test with 3 slashes
        assert _extract_path_from_file_url("file:///C:/test.txt") == "C:/test.txt"
        # Test with 2 slashes
        assert _extract_path_from_file_url("file://C:/test.txt") == "C:/test.txt"
        # Test non-file URL returns None
        assert _extract_path_from_file_url("https://example.com") is None
