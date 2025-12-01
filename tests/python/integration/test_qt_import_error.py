"""Test Qt backend import error handling without Qt dependencies.

This test module verifies that auroraview handles missing Qt dependencies gracefully.
It tests the placeholder classes and error messages when Qt is not installed.

For tests that require Qt to be installed, see test_qt_backend.py
"""

import sys
from unittest.mock import patch

import pytest


class TestQtImportWithoutQt:
    """Test Qt import behavior when Qt is not available."""

    def test_qt_import_available_in_all(self):
        """Test that QtWebView is always available in __all__ even if Qt is not installed."""
        import auroraview

        assert "QtWebView" in auroraview.__all__

    def test_qt_classes_always_importable(self):
        """Test that Qt classes can always be imported (even if Qt is not installed)."""
        # This should not raise ImportError
        from auroraview import QtWebView

        assert QtWebView is not None

    def test_diagnostic_variables_exist(self):
        """Test that diagnostic variables are available."""
        import auroraview

        assert hasattr(auroraview, "_HAS_QT")
        assert hasattr(auroraview, "_QT_IMPORT_ERROR")
        assert isinstance(auroraview._HAS_QT, bool)


class TestQtPlaceholderBehavior:
    """Test placeholder class behavior when Qt is not installed."""

    def test_qt_import_error_message(self):
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

    def test_error_message_includes_original_error(self):
        """Test that error message includes the original import error."""
        with patch.dict(sys.modules, {"auroraview.qt_integration": None}):
            import importlib

            import auroraview

            importlib.reload(auroraview)

            from auroraview import QtWebView

            with pytest.raises(ImportError) as exc_info:
                QtWebView()

            error_msg = str(exc_info.value)
            assert "Original error:" in error_msg


class TestNativeBackendAvailability:
    """Test that native backend is always available regardless of Qt."""

    def test_native_backend_always_available(self):
        """Test that native backend is always available."""
        from auroraview import WebView

        # WebView should always be importable
        assert WebView is not None

    def test_native_backend_in_all(self):
        """Test that native backend classes are in __all__."""
        import auroraview

        assert "WebView" in auroraview.__all__

    def test_native_backend_has_factory_methods(self):
        """Test that unified WebView factory exists."""
        from auroraview import WebView

        assert hasattr(WebView, "create")


class TestQtBackendDiagnostics:
    """Test diagnostic capabilities for Qt backend."""

    def test_can_check_qt_availability(self):
        """Test that users can check if Qt is available."""
        import auroraview

        # This should work regardless of whether Qt is installed
        has_qt = auroraview._HAS_QT
        assert isinstance(has_qt, bool)

        if not has_qt:
            # If Qt is not available, error should be set
            assert auroraview._QT_IMPORT_ERROR is not None
            assert isinstance(auroraview._QT_IMPORT_ERROR, str)
        else:
            # If Qt is available, error should be None
            assert auroraview._QT_IMPORT_ERROR is None

    def test_diagnostic_code_example(self):
        """Test the diagnostic code example from documentation."""
        import auroraview

        # This is the code users would run to diagnose Qt issues
        has_qt = auroraview._HAS_QT
        qt_error = auroraview._QT_IMPORT_ERROR

        # Should not raise any exceptions
        assert isinstance(has_qt, bool)
        if not has_qt:
            assert qt_error is not None
