"""Tests for the CLI main entry point.

This module tests the Python CLI entry point that uses WebView directly.
"""

import sys
from pathlib import Path
from unittest.mock import patch

import pytest


def test_main_success():
    """Test successful CLI execution with URL."""
    from auroraview.__main__ import main

    # Mock run_standalone to avoid actual window creation
    with patch("auroraview._core.run_standalone") as mock_run_standalone:
        with patch.object(sys, "argv", ["auroraview", "--url", "https://example.com"]):
            main()

            # Verify run_standalone was called with correct parameters
            mock_run_standalone.assert_called_once()
            call_kwargs = mock_run_standalone.call_args.kwargs
            assert call_kwargs["title"] == "AuroraView"
            # URL is normalized (trailing slash added)
            assert call_kwargs["url"].startswith("https://example.com")


def test_main_with_arguments():
    """Test CLI execution with HTML file and debug flag."""
    import tempfile

    from auroraview.__main__ import main

    # Create a temporary HTML file

    with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
        f.write("<html><body>Test</body></html>")
        html_path = f.name

    try:
        # Mock sys.argv to include arguments
        test_args = ["auroraview", "--html", html_path, "--debug"]

        # Mock run_standalone to avoid actual window creation
        with patch("auroraview._core.run_standalone") as mock_run_standalone:
            with patch.object(sys, "argv", test_args):
                main()

                # Verify run_standalone was called with debug flag
                mock_run_standalone.assert_called_once()
                call_kwargs = mock_run_standalone.call_args.kwargs
                assert call_kwargs["dev_tools"] is True
                # Verify new parameters are passed
                assert call_kwargs["rewrite_relative_paths"] is True
                assert call_kwargs["html_path"] is not None
                # asset_root should be auto-derived from HTML file location
                assert call_kwargs["asset_root"] is not None
    finally:
        # Clean up temp file
        Path(html_path).unlink(missing_ok=True)


def _normalize_path(path_str: str) -> str:
    """Normalize path for comparison across different Windows path formats.

    On Windows, paths can be returned in 8.3 format (e.g., RUNNER~1) or full format
    (e.g., runneradmin). This function normalizes both to a consistent format.
    """
    import os

    if not path_str:
        return path_str

    # Use os.path.normpath and resolve to get canonical path
    # On Windows, this handles both short and long path names
    try:
        # Path.resolve() converts 8.3 short names to long names
        return str(Path(path_str).resolve())
    except (OSError, ValueError):
        # Fallback to normpath if resolve fails
        return os.path.normpath(path_str)


def test_main_with_html_auto_asset_root():
    """Test that asset_root is auto-derived from HTML file location."""
    import tempfile

    from auroraview.__main__ import main

    # Create a temporary directory with HTML file
    with tempfile.TemporaryDirectory() as tmpdir:
        html_file = Path(tmpdir) / "index.html"
        html_file.write_text('<html><link href="./style.css"></html>')

        test_args = ["auroraview", "--html", str(html_file)]

        with patch("auroraview._core.run_standalone") as mock_run_standalone:
            with patch.object(sys, "argv", test_args):
                main()

                call_kwargs = mock_run_standalone.call_args.kwargs
                # asset_root should be the directory containing the HTML file
                # Use path normalization to handle Windows 8.3 short names
                assert _normalize_path(call_kwargs["asset_root"]) == _normalize_path(tmpdir)
                # html_path should be the absolute path to the HTML file
                assert _normalize_path(call_kwargs["html_path"]) == _normalize_path(str(html_file))


def test_main_with_explicit_assets_root():
    """Test that explicit --assets-root overrides auto-detection."""
    import tempfile

    from auroraview.__main__ import main

    with tempfile.TemporaryDirectory() as tmpdir:
        html_file = Path(tmpdir) / "index.html"
        html_file.write_text("<html><body>Test</body></html>")

        # Create a separate assets directory
        assets_dir = Path(tmpdir) / "assets"
        assets_dir.mkdir()

        test_args = ["auroraview", "--html", str(html_file), "--assets-root", str(assets_dir)]

        with patch("auroraview._core.run_standalone") as mock_run_standalone:
            with patch.object(sys, "argv", test_args):
                main()

                call_kwargs = mock_run_standalone.call_args.kwargs
                # asset_root should be the explicitly provided directory
                # Use path normalization to handle Windows 8.3 short names
                assert _normalize_path(call_kwargs["asset_root"]) == _normalize_path(
                    str(assets_dir)
                )


def test_main_non_zero_exit_code():
    """Test CLI execution with WebView exception."""
    from auroraview.__main__ import main

    # Mock run_standalone to raise an exception
    with patch("auroraview._core.run_standalone", side_effect=RuntimeError("WebView error")):
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

    # Mock run_standalone to raise a generic exception
    with patch("auroraview._core.run_standalone", side_effect=RuntimeError("Unexpected error")):
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

    # Get the path to __main__.py (relative to project root)
    project_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(__file__))))
    main_path = os.path.join(project_root, "python", "auroraview", "__main__.py")
    main_path = os.path.abspath(main_path)

    # Load the module
    spec = importlib.util.spec_from_file_location("__main__", main_path)
    if spec and spec.loader:
        module = importlib.util.module_from_spec(spec)

        # Mock run_standalone to avoid actual window creation
        with patch("auroraview._core.run_standalone") as mock_run_standalone:
            with patch.object(sys, "argv", ["auroraview", "--url", "https://example.com"]):
                # Execute the module
                spec.loader.exec_module(module)

                # Verify run_standalone was called
                mock_run_standalone.assert_called_once()


def test_main_url_normalization():
    """Test that URLs are normalized correctly."""
    from auroraview.__main__ import main

    # Mock run_standalone and normalize_url
    with patch("auroraview._core.run_standalone") as mock_run_standalone:
        with patch(
            "auroraview.normalize_url", return_value="https://example.com/"
        ) as mock_normalize:
            with patch.object(sys, "argv", ["auroraview", "--url", "example.com"]):
                main()

                # Verify normalize_url was called
                mock_normalize.assert_called_once_with("example.com")
                # Verify run_standalone was called with normalized URL
                mock_run_standalone.assert_called_once()
                call_kwargs = mock_run_standalone.call_args.kwargs
                assert call_kwargs["url"] == "https://example.com/"


def test_main_html_rewriting():
    """Test that HTML rewriting parameters are passed to run_standalone.

    Note: The actual HTML rewriting is done in Rust (run_standalone handles
    html_path and rewrite_relative_paths). This test verifies that the CLI
    correctly passes these parameters.
    """
    # Create a temporary HTML file
    import tempfile

    from auroraview.__main__ import main

    with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
        f.write('<link href="style.css">')
        html_path = f.name

    try:
        # Mock run_standalone to verify parameters
        with patch("auroraview._core.run_standalone") as mock_run_standalone:
            with patch.object(sys, "argv", ["auroraview", "--html", html_path]):
                main()

                # Verify run_standalone was called with correct parameters
                mock_run_standalone.assert_called_once()
                call_kwargs = mock_run_standalone.call_args.kwargs

                # Verify html content was read
                assert call_kwargs["html"] == '<link href="style.css">'

                # Verify html_path is passed (for Rust-side rewriting)
                assert call_kwargs["html_path"] == str(Path(html_path).resolve())

                # Verify rewrite_relative_paths is enabled
                assert call_kwargs["rewrite_relative_paths"] is True

                # Verify asset_root is auto-derived from html file location
                assert call_kwargs["asset_root"] == str(Path(html_path).resolve().parent)
    finally:
        # Clean up temp file
        Path(html_path).unlink(missing_ok=True)
