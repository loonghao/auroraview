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


class TestEventBridgeLifecycle:
    """Test EventBridge lifecycle management."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_event_bridge_cleanup(self, qapp):
        """Test that EventBridge cleanup prevents further event processing."""
        from auroraview.qt_integration import EventBridge

        bridge = EventBridge()

        # Register a handler
        called = []

        def handler(data):
            called.append(data)

        bridge.register_handler("test_event", handler)

        # Send event before cleanup - should work
        bridge.js_to_python("test_event", '{"value": 1}')
        assert len(called) == 1
        assert called[0]["value"] == 1

        # Cleanup bridge
        bridge.cleanup()

        # Send event after cleanup - should be ignored
        bridge.js_to_python("test_event", '{"value": 2}')
        assert len(called) == 1  # Should not have increased

    def test_event_bridge_emit_after_cleanup(self, qapp):
        """Test that emit_to_js after cleanup doesn't crash."""
        from auroraview.qt_integration import EventBridge

        bridge = EventBridge()

        # Emit before cleanup - should work
        bridge.emit_to_js("test", {"value": 1})

        # Cleanup
        bridge.cleanup()

        # Emit after cleanup - should not crash
        bridge.emit_to_js("test", {"value": 2})  # Should be silently ignored


class TestQtWebViewLifecycle:
    """Test QtWebView lifecycle management."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_qtwebview_close_event(self, qapp):
        """Test that closeEvent properly cleans up resources."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()

        # Verify bridge is created
        assert hasattr(webview, "_bridge")
        assert webview._bridge is not None

        # Simulate close event
        event = QCloseEvent()
        webview.closeEvent(event)

        # Verify cleanup was called
        assert webview._is_closing is True
        assert webview._bridge._is_destroyed is True

        # Cleanup
        webview.deleteLater()

    def test_qtwebview_multiple_close_events(self, qapp):
        """Test that multiple closeEvent calls don't cause errors."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()

        # First close
        event1 = QCloseEvent()
        webview.closeEvent(event1)
        assert webview._is_closing is True

        # Second close - should be handled gracefully
        event2 = QCloseEvent()
        webview.closeEvent(event2)  # Should not crash

        # Cleanup
        webview.deleteLater()

    def test_qtwebview_parent_child_cleanup(self, qapp):
        """Test that Qt parent-child relationship ensures proper cleanup."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Verify parent-child relationship
        assert webview._bridge.parent() == webview
        assert webview._channel.parent() == webview

        # Close and cleanup
        webview.close()
        webview.deleteLater()

    def test_qtwebview_emit_after_close(self, qapp):
        """Test that emit after close doesn't crash."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()

        # Close the webview
        event = QCloseEvent()
        webview.closeEvent(event)

        # Try to emit - should not crash
        webview.emit("test_event", {"value": 1})

        # Cleanup
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


class TestQtWebViewAppQuit:
    """Test application quit handling."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_app_quit_handler_registered(self, qapp):
        """Test that app quit handler is registered."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Verify handler is registered by checking if it can be disconnected
        try:
            from qtpy.QtCore import QCoreApplication

            app = QCoreApplication.instance()
            if app:
                # Try to disconnect - if it was connected, this should work
                app.aboutToQuit.disconnect(webview._on_app_quit)
                # Reconnect for cleanup
                app.aboutToQuit.connect(webview._on_app_quit)
        except (RuntimeError, TypeError):
            # If disconnect fails, handler wasn't registered
            pytest.fail("App quit handler was not registered")

        # Cleanup
        webview.close()
        webview.deleteLater()

    def test_on_app_quit_closes_window(self, qapp):
        """Test that _on_app_quit closes the window."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Verify window is not closing
        assert webview._is_closing is False

        # Call _on_app_quit
        webview._on_app_quit()

        # Verify window is now closing
        assert webview._is_closing is True

        # Cleanup
        webview.deleteLater()

    def test_wa_delete_on_close_set(self, qapp):
        """Test that WA_DeleteOnClose attribute is set."""
        from qtpy.QtCore import Qt

        from auroraview import QtWebView

        webview = QtWebView()

        # Verify WA_DeleteOnClose is set
        assert webview.testAttribute(Qt.WA_DeleteOnClose) is True

        # Cleanup
        webview.close()
        # No need for deleteLater() since WA_DeleteOnClose is set
