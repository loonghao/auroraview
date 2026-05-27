# tests/python/integration/test_qt_zombie_fix.py

"""Tests for the zombie reference bug fix.

These tests verify that QtWebView safely handles calls after close,
DCC reuse scenarios, and C++ object destruction edge cases.
"""

import logging
import os
import sys

import pytest

pytest.importorskip("qtpy", reason="Qt backend requires qtpy and Qt bindings")

try:
    import auroraview._core  # noqa: F401

    _CORE_AVAILABLE = True
except ImportError:
    _CORE_AVAILABLE = False

_IN_CI = os.environ.get("CI", "").lower() == "true"
_IS_WINDOWS = sys.platform == "win32"
_SKIP_WEBVIEW_TESTS = (_IN_CI and not _IS_WINDOWS) or not _CORE_AVAILABLE

pytestmark = [pytest.mark.qt]


@pytest.mark.skipif(_SKIP_WEBVIEW_TESTS, reason="WebView tests require display or Rust core")
class TestZombieReferenceFix:
    """Verify the zombie reference bug fix."""

    def test_no_wa_delete_on_close(self, qapp):
        """WA_DeleteOnClose should NOT be set."""
        from qtpy.QtCore import Qt

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            assert not webview.testAttribute(Qt.WA_DeleteOnClose)
        finally:
            webview.deleteLater()

    def test_is_alive_true_initially(self, qapp):
        """is_alive should be True for a fresh widget."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            assert webview.is_alive is True
        finally:
            webview.deleteLater()

    def test_is_alive_false_after_close(self, qapp):
        """is_alive should return False after closeEvent."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)
            assert webview.is_alive is False
        finally:
            webview.deleteLater()

    def test_emit_after_close_is_noop(self, qapp):
        """emit() should be a no-op after closeEvent, not raise."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)
            # Should NOT raise RuntimeError
            webview.emit("test_event", {"value": 42})
        finally:
            webview.deleteLater()

    def test_load_url_after_close_is_noop(self, qapp):
        """load_url() should be a no-op after closeEvent."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)
            # Should NOT raise
            webview.load_url("https://example.com")
        finally:
            webview.deleteLater()

    def test_load_html_after_close_is_noop(self, qapp):
        """load_html() should be a no-op after closeEvent."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)
            webview.load_html("<html><body>test</body></html>")
        finally:
            webview.deleteLater()

    def test_load_file_after_close_is_noop(self, qapp):
        """load_file() should be a no-op after closeEvent."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)
            webview.load_file("C:/nonexistent/index.html")
        finally:
            webview.deleteLater()

    def test_eval_js_after_close_is_noop(self, qapp):
        """eval_js() should be a no-op after closeEvent."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)
            webview.eval_js("console.log('test')")
        finally:
            webview.deleteLater()

    def test_reuse_after_close_resets_is_closing(self, qapp):
        """showEvent after close should reset _is_closing for reuse."""
        from qtpy.QtGui import QCloseEvent, QShowEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            # Close
            close_event = QCloseEvent()
            webview.closeEvent(close_event)
            assert webview._is_closing is True

            # Re-show
            show_event = QShowEvent()
            webview.showEvent(show_event)
            assert webview._is_closing is False
            assert webview.is_alive is True
        finally:
            webview.deleteLater()

    def test_about_to_close_signal_before_state_change(self, qapp):
        """aboutToClose should fire while widget is still alive."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            alive_at_signal_time = []

            def on_about_to_close():
                # At this point, is_alive should still be True
                alive_at_signal_time.append(webview.is_alive)

            webview.aboutToClose.connect(on_about_to_close)

            event = QCloseEvent()
            webview.closeEvent(event)

            assert len(alive_at_signal_time) == 1
            assert alive_at_signal_time[0] is True
        finally:
            webview.deleteLater()

    def test_resize_after_container_deleted_logs_warning(self, qapp, caplog):
        """resizeEvent with dead container should log warning, not crash."""
        from unittest.mock import MagicMock

        from qtpy.QtCore import QSize

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            # Simulate a dead container
            mock_container = MagicMock()
            mock_container.setGeometry.side_effect = RuntimeError(
                "Internal C++ object already deleted"
            )
            webview._webview_container = mock_container

            mock_event = MagicMock()
            mock_event.size.return_value = QSize(800, 600)

            with caplog.at_level(logging.WARNING):
                webview.resizeEvent(mock_event)

            # Container should be cleared
            assert webview._webview_container is None
            assert "container C++ object already deleted" in caplog.text
        finally:
            webview.deleteLater()

    def test_destroy_method(self, qapp):
        """destroy() should mark widget as closing and schedule deletion."""
        from auroraview import QtWebView

        webview = QtWebView()
        received = []
        webview.aboutToClose.connect(lambda: received.append(True))

        webview.destroy()

        assert webview._is_closing is True
        assert len(received) == 1

    def test_multiple_destroy_calls_safe(self, qapp):
        """Calling destroy() multiple times should not crash."""
        from auroraview import QtWebView

        webview = QtWebView()
        webview.destroy()
        webview.destroy()  # Should not crash

    def test_on_callback_guarded_after_close(self, qapp):
        """Callbacks registered via on() should not fire after close.

        This test directly invokes the core's event dispatch mechanism
        to verify that the wrapper guard prevents handler execution.
        """
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            call_count = []

            @webview.on("test_guard_event")
            def handler(data):
                call_count.append(data)

            # Close the widget
            event = QCloseEvent()
            webview.closeEvent(event)
            assert webview._is_closing is True

            # Directly simulate event dispatch through the core's
            # _event_handlers mechanism. After close, teardown should have
            # cleared the handlers, but even if it didn't, the is_alive
            # guard in the wrapper ensures the handler is not called.
            core_webview = webview._webview
            if core_webview is not None:
                # Try dispatching through the event_handlers dict
                with core_webview._event_handlers_lock:
                    handlers = core_webview._event_handlers.get("test_guard_event", [])
                # Call each handler directly (simulating Rust core dispatch)
                for h in handlers:
                    h({"key": "val"})

            # Handler should NOT have been called (either cleared by
            # teardown or guarded by is_alive)
            assert len(call_count) == 0
        finally:
            webview.deleteLater()

    def test_signal_bridge_teardown_clears_handlers(self, qapp):
        """_teardown_signal_bridge should clear event handlers."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            core_webview = webview._webview
            # Verify signal bridge events are registered
            with core_webview._event_handlers_lock:
                has_nav = "navigation_started" in core_webview._event_handlers

            if has_nav:
                # Teardown
                webview._teardown_signal_bridge()

                # Verify cleared
                with core_webview._event_handlers_lock:
                    assert "navigation_started" not in core_webview._event_handlers
                    assert "load_progress" not in core_webview._event_handlers
        finally:
            webview.deleteLater()

    def test_setup_signal_bridge_idempotent(self, qapp):
        """Calling _setup_signal_bridge twice should not double handlers."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            core_webview = webview._webview

            # Get handler count after initial setup
            with core_webview._event_handlers_lock:
                initial_count = len(core_webview._event_handlers.get("navigation_started", []))

            # Call setup again (should teardown first internally)
            webview._setup_signal_bridge()

            with core_webview._event_handlers_lock:
                new_count = len(core_webview._event_handlers.get("navigation_started", []))

            # Should be the same (not doubled)
            assert new_count == initial_count
        finally:
            webview.deleteLater()
