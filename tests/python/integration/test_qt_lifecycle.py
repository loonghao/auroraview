"""Test Qt backend lifecycle management.

This test module verifies proper cleanup and lifecycle management
of QtWebView to prevent errors like:
    RuntimeError: Internal C++ object (PySide2.QtWidgets.QLabel) already deleted.

These tests require Qt dependencies to be installed:
    pip install auroraview[qt]
"""

import os
import sys

import pytest

# Mark all tests as Qt tests
pytestmark = [pytest.mark.qt]

# Check if we're in CI environment - skip WebView tests that require native window
_IN_CI = os.environ.get("CI", "").lower() == "true"
# Skip WebView instantiation tests in CI - they crash even with xvfb due to WebView2/native issues
_SKIP_WEBVIEW_TESTS = _IN_CI


@pytest.mark.skipif(_SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI environment")
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


@pytest.mark.skipif(_SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI environment")
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

    def test_event_processor_processes_events(self, qapp):
        """Test that event processor processes events correctly."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Track event processor calls
        original_count = webview._event_processor._process_count

        # Trigger event processing via eval_js
        webview._webview.eval_js("console.log('test')")

        # Verify event processor was called
        assert webview._event_processor._process_count > original_count

        # Cleanup
        webview.close()
        webview.deleteLater()

    def test_emit_uses_event_processor(self, qapp):
        """Test that WebView.emit() uses event processor strategy."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Track event processor calls
        process_called = []
        original_process = webview._event_processor.process

        def mock_process():
            process_called.append(True)
            original_process()

        webview._event_processor.process = mock_process

        # Emit event
        webview._webview.emit("test_event", {"data": "test"})

        # Verify event processor was called
        assert len(process_called) == 1, "emit() should trigger event processor"

        # Cleanup
        webview.close()
        webview.deleteLater()

    def test_eval_js_uses_event_processor(self, qapp):
        """Test that WebView.eval_js() uses event processor strategy."""
        from auroraview import QtWebView

        webview = QtWebView()

        # Track event processor calls
        process_called = []
        original_process = webview._event_processor.process

        def mock_process():
            process_called.append(True)
            original_process()

        webview._event_processor.process = mock_process

        # Execute JavaScript
        webview._webview.eval_js("console.log('test')")

        # Verify event processor was called
        assert len(process_called) == 1, "eval_js() should trigger event processor"

        # Cleanup
        webview.close()
        webview.deleteLater()

    def test_qtwebview_auto_installs_event_processor(self, qapp):
        """Test that QtWebView automatically installs QtEventProcessor."""
        from auroraview import QtWebView
        from auroraview.qt_integration import QtEventProcessor

        webview = QtWebView()

        # Verify event processor is installed
        assert hasattr(webview, "_event_processor")
        assert isinstance(webview._event_processor, QtEventProcessor)
        assert webview._webview._event_processor is webview._event_processor

        # Cleanup
        webview.close()
        webview.deleteLater()


@pytest.mark.skipif(_SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI environment")
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
