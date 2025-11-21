"""Tests for the CLI main entry point.

This module tests the Python CLI entry point that uses WebView directly.
"""

import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest


def test_main_success():
    """Test successful CLI execution with URL."""
    from auroraview.__main__ import main

    # Mock WebView to avoid actual window creation
    mock_webview = MagicMock()

    # Mock sys.argv with URL argument
    with patch("auroraview.WebView", return_value=mock_webview):
        with patch.object(sys, "argv", ["auroraview", "--url", "https://example.com"]):
            main()

            # Verify WebView was created with correct parameters
            mock_webview.show_blocking.assert_called_once()


def test_main_with_arguments():
    """Test CLI execution with HTML file and debug flag."""
    from auroraview.__main__ import main

    # Mock WebView to avoid actual window creation
    mock_webview = MagicMock()

    # Create a temporary HTML file
    import tempfile

    with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
        f.write("<html><body>Test</body></html>")
        html_path = f.name

    try:
        # Mock sys.argv to include arguments
        test_args = ["auroraview", "--html", html_path, "--debug"]

        with patch("auroraview.WebView", return_value=mock_webview):
            with patch.object(sys, "argv", test_args):
                main()

                # Verify WebView was created with debug flag
                mock_webview.show_blocking.assert_called_once()
    finally:
        # Clean up temp file
        Path(html_path).unlink(missing_ok=True)


def test_main_non_zero_exit_code():
    """Test CLI execution with WebView exception."""
    from auroraview.__main__ import main

    # Mock WebView to raise an exception
    with patch("auroraview.WebView", side_effect=RuntimeError("WebView error")):
        with patch.object(sys, "argv", ["auroraview", "--url", "https://example.com"]):
            with pytest.raises(SystemExit) as exc_info:
                main()

            # Verify exit code is 1 on error
            assert exc_info.value.code == 1


def test_main_html_file_not_found():
    """Test CLI execution when HTML file is not found."""
    from auroraview.__main__ import main

    # Mock sys.argv with non-existent HTML file
    with patch.object(sys, "argv", ["auroraview", "--html", "nonexistent.html"]):
        with patch("builtins.print") as mock_print:
            with pytest.raises(SystemExit) as exc_info:
                main()

            # Verify error message was printed
            assert mock_print.called
            assert exc_info.value.code == 1


def test_main_generic_exception():
    """Test CLI execution with generic exception."""
    from auroraview.__main__ import main

    # Mock WebView to raise a generic exception
    with patch("auroraview.WebView", side_effect=RuntimeError("Unexpected error")):
        with patch.object(sys, "argv", ["auroraview", "--url", "https://example.com"]):
            with pytest.raises(SystemExit) as exc_info:
                main()

            # Verify exit code is 1 on error
            assert exc_info.value.code == 1


def test_main_module_execution():
    """Test that __main__ module can be executed."""
    # This tests the if __name__ == "__main__": block
    import importlib.util
    import os

    # Get the path to __main__.py
    main_path = os.path.join(os.path.dirname(__file__), "..", "python", "auroraview", "__main__.py")
    main_path = os.path.abspath(main_path)

    # Load the module
    spec = importlib.util.spec_from_file_location("__main__", main_path)
    if spec and spec.loader:
        module = importlib.util.module_from_spec(spec)

        # Mock WebView to avoid actual window creation
        mock_webview = MagicMock()

        with patch("auroraview.WebView", return_value=mock_webview):
            with patch.object(sys, "argv", ["auroraview", "--url", "https://example.com"]):
                # Execute the module
                spec.loader.exec_module(module)


def test_main_url_normalization():
    """Test that URLs are normalized correctly."""
    from auroraview.__main__ import main

    # Mock WebView and normalize_url
    mock_webview = MagicMock()

    with patch("auroraview.WebView", return_value=mock_webview):
        with patch(
            "auroraview.normalize_url", return_value="https://example.com/"
        ) as mock_normalize:
            with patch.object(sys, "argv", ["auroraview", "--url", "example.com"]):
                main()

                # Verify normalize_url was called
                mock_normalize.assert_called_once_with("example.com")
                # Verify WebView was created with normalized URL
                mock_webview.show_blocking.assert_called_once()


def test_main_html_rewriting():
    """Test that HTML is rewritten for custom protocol."""
    # Create a temporary HTML file
    import tempfile

    from auroraview.__main__ import main

    with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
        f.write('<link href="style.css">')
        html_path = f.name

    try:
        # Mock WebView and rewrite function
        mock_webview = MagicMock()

        with patch("auroraview.WebView", return_value=mock_webview):
            with patch(
                "auroraview.rewrite_html_for_custom_protocol",
                return_value='<link href="auroraview://style.css">',
            ) as mock_rewrite:
                with patch.object(sys, "argv", ["auroraview", "--html", html_path]):
                    main()

                    # Verify rewrite function was called
                    mock_rewrite.assert_called_once()
                    # Verify WebView was created
                    mock_webview.show_blocking.assert_called_once()
    finally:
        # Clean up temp file
        Path(html_path).unlink(missing_ok=True)
