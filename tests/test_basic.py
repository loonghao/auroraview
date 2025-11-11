"""Basic tests for auroraview package."""

import pytest


@pytest.mark.unit
def test_import():
    """Test that the package can be imported."""
    try:
        import auroraview

        assert auroraview is not None
    except ImportError:
        pytest.skip("Package not built yet")


@pytest.mark.unit
def test_version():
    """Test that version is accessible."""
    try:
        import auroraview

        assert hasattr(auroraview, "__version__")
        assert isinstance(auroraview.__version__, str)
        # Version should be in format X.Y.Z
        parts = auroraview.__version__.split(".")
        assert len(parts) >= 2
    except ImportError:
        pytest.skip("Package not built yet")


@pytest.mark.unit
def test_author():
    """Test that author is accessible."""
    try:
        import auroraview

        assert hasattr(auroraview, "__author__")
        assert isinstance(auroraview.__author__, str)
        assert len(auroraview.__author__) > 0
    except ImportError:
        pytest.skip("Package not built yet")


@pytest.mark.unit
def test_webview_class_exists():
    """Test that WebView class exists."""
    try:
        from auroraview import WebView

        assert WebView is not None
    except ImportError:
        pytest.skip("Package not built yet")


@pytest.mark.unit
def test_webview_on_decorator_exists():
    """Test that @webview.on() decorator exists."""
    try:
        from auroraview import WebView

        webview = WebView()
        assert hasattr(webview, "on")
        assert callable(webview.on)
    except ImportError:
        pytest.skip("Package not built yet")


@pytest.mark.unit
def test_all_exports():
    """Test that all expected exports are available."""
    try:
        import auroraview

        expected_exports = ["WebView", "on_event", "__version__", "__author__"]
        for export in expected_exports:
            assert hasattr(auroraview, export), f"Missing export: {export}"
    except ImportError:
        pytest.skip("Package not built yet")
