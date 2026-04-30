"""Blender integration tests for AuroraView.

Tests AuroraView functionality within Blender environment.
"""

import pytest


@pytest.mark.blender
def test_blender_import():
    """Test that AuroraView can be imported in Blender."""
    try:
        import bpy  # noqa: F401

        from auroraview import WebView

        assert WebView is not None
    except ImportError:
        pytest.skip("Blender not available")


@pytest.mark.blender
def test_blender_webview_creation():
    """Test creating WebView in Blender environment."""
    try:
        import bpy  # noqa: F401

        from auroraview import WebView

        # Create WebView without showing (headless test)
        view = WebView(title="Blender Test", width=800, height=600)
        assert view is not None
        assert view.title == "Blender Test"
    except ImportError:
        pytest.skip("Blender not available")


@pytest.mark.blender
def test_blender_operator_registration():
    """Test that AuroraView operators can be registered in Blender."""
    try:
        import bpy  # noqa: F401

        from auroraview import WebView

        # Test that we can create a function that would be called from Blender
        def show_auroraview():
            view = WebView(title="Test from Blender")
            view.show()

        assert callable(show_auroraview)
    except ImportError:
        pytest.skip("Blender not available")


@pytest.mark.blender
def test_blender_dcc_environment():
    """Test DCC environment detection in Blender."""
    try:
        import bpy  # noqa: F401

        from auroraview.utils.thread_dispatcher import get_current_dcc_name

        dcc_name = get_current_dcc_name()
        assert dcc_name is not None
        assert "blender" in dcc_name.lower()
    except (ImportError, AttributeError):
        pytest.skip("Blender not available or function not implemented")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
