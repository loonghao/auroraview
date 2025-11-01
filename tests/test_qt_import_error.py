"""Test Qt backend import error handling."""

import sys
from unittest.mock import patch

import pytest


def test_qt_import_available_in_all():
    """Test that QtWebView is always available in __all__ even if Qt is not installed."""
    import auroraview

    assert "QtWebView" in auroraview.__all__
    assert "AuroraViewQt" in auroraview.__all__


def test_qt_import_error_message():
    """Test that QtWebView raises helpful error when Qt is not installed."""
    # Mock the qt_integration import to fail
    with patch.dict(sys.modules, {"auroraview.qt_integration": None}):
        # Force reimport
        import importlib

        import auroraview

        importlib.reload(auroraview)

        # QtWebView should be importable but raise error on instantiation
        from auroraview import QtWebView

        with pytest.raises(ImportError) as exc_info:
            QtWebView()

        error_msg = str(exc_info.value)
        assert "Qt backend is not available" in error_msg
        assert "pip install auroraview[qt]" in error_msg


def test_native_backend_always_available():
    """Test that native backend is always available."""
    from auroraview import NativeWebView, WebView

    # These should always be importable
    assert WebView is not None
    assert NativeWebView is not None


def test_qt_backend_placeholder_for_auroraviewqt():
    """Test that AuroraViewQt also raises helpful error when Qt is not installed."""
    with patch.dict(sys.modules, {"auroraview.qt_integration": None}):
        import importlib

        import auroraview

        importlib.reload(auroraview)

        from auroraview import AuroraViewQt

        with pytest.raises(ImportError) as exc_info:
            AuroraViewQt()

        error_msg = str(exc_info.value)
        assert "Qt backend is not available" in error_msg
        assert "pip install auroraview[qt]" in error_msg

