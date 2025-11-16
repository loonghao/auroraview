"""Test Qt backend lifecycle management.

This test module verifies proper cleanup and lifecycle management
of QtWebView to prevent errors like:
    RuntimeError: Internal C++ object (PySide2.QtWidgets.QLabel) already deleted.

These tests require Qt dependencies to be installed:
    pip install auroraview[qt]
"""

import sys

import pytest

# Check if Qt is available
try:
    import auroraview

    HAS_QT = auroraview._HAS_QT
    QT_IMPORT_ERROR = auroraview._QT_IMPORT_ERROR
except ImportError:
    HAS_QT = False
    QT_IMPORT_ERROR = "auroraview not installed"

# Skip all tests in this module if Qt is not available
pytestmark = pytest.mark.skipif(not HAS_QT, reason=f"Qt backend not available: {QT_IMPORT_ERROR}")


class TestQtWebViewLifecycle:
    """Test QtWebView lifecycle management for the new WebView2-based backend."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_qtwebview_close_event_sets_flag(self, qapp):
        """closeEvent should mark the widget as closing."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        assert webview._is_closing is False

        event = QCloseEvent()
        webview.closeEvent(event)

        assert webview._is_closing is True

        webview.deleteLater()

    def test_qtwebview_multiple_close_events_safe(self, qapp):
        """Multiple closeEvent calls should not crash."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()

        event1 = QCloseEvent()
        webview.closeEvent(event1)
        assert webview._is_closing is True

        event2 = QCloseEvent()
        webview.closeEvent(event2)  # Should not crash

        webview.deleteLater()

    def test_qtwebview_embeds_webview_core(self, qapp):
        """QtWebView should create an internal WebView backend instance."""
        from auroraview import QtWebView
        from auroraview.webview import WebView

        webview = QtWebView()
        assert hasattr(webview, "_webview")
        assert isinstance(webview._webview, WebView)

        webview.close()
        webview.deleteLater()

    def test_qtwebview_emit_after_close_does_not_crash(self, qapp):
        """Calling emit after closeEvent should be a no-op and not crash."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()

        event = QCloseEvent()
        webview.closeEvent(event)

        # Should not raise even though the underlying WebView has been closed
        webview.emit("test_event", {"value": 1})

        webview.deleteLater()


class TestQtWebViewEventProcessing:
    """Test event processing and UI updates."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_process_pending_events(self, qapp):
        """Test that _process_pending_events doesn't crash."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Should not crash
        webview._process_pending_events()

        # Cleanup
        webview.close()
        webview.deleteLater()


class TestQtWebViewAppIntegration:
    """Lightweight tests around Qt-specific integration flags."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_wa_delete_on_close_set(self, qapp):
        """QtWebView should delete itself when closed."""
        from qtpy.QtCore import Qt

        from auroraview import QtWebView

        webview = QtWebView()

        # Verify WA_DeleteOnClose is set
        assert webview.testAttribute(Qt.WA_DeleteOnClose) is True

        # Cleanup
        webview.close()
