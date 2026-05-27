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
        from qtpy.QtGui import QResizeEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            # Simulate a dead container
            mock_container = MagicMock()
            mock_container.setGeometry.side_effect = RuntimeError(
                "Internal C++ object already deleted"
            )
            webview._webview_container = mock_container

            # Use a real QResizeEvent (required by super().resizeEvent())
            resize_event = QResizeEvent(QSize(800, 600), QSize(400, 300))

            with caplog.at_level(logging.WARNING):
                webview.resizeEvent(resize_event)

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
        """_teardown_signal_bridge should clear bridge handlers."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            core_webview = webview._webview
            # Verify signal bridge events are registered
            with core_webview._event_handlers_lock:
                has_nav = "navigation_started" in core_webview._event_handlers

            if has_nav:
                # Get count before teardown
                with core_webview._event_handlers_lock:
                    count_before = len(core_webview._event_handlers.get("navigation_started", []))
                assert count_before >= 1  # at least bridge handler

                # Teardown
                webview._teardown_signal_bridge()

                # Verify bridge handlers are cleared
                with core_webview._event_handlers_lock:
                    count_after = len(core_webview._event_handlers.get("navigation_started", []))
                assert count_after == count_before - 1  # bridge handler removed
        finally:
            webview.deleteLater()

    def test_user_handler_survives_teardown(self, qapp):
        """User handlers on bridge event names should survive teardown.

        This verifies P0.1 fix: _teardown_signal_bridge() only removes
        bridge-internal callbacks, not user-registered ones.
        """
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            core_webview = webview._webview

            # Register a user handler on a bridge event name
            user_calls = []

            @webview.on("navigation_started")
            def user_nav_handler(data):
                user_calls.append(data)

            # Verify it's registered
            with core_webview._event_handlers_lock:
                handlers_before = list(core_webview._event_handlers.get("navigation_started", []))
            # Should have at least 2: bridge handler + user handler
            assert len(handlers_before) >= 2

            # Teardown signal bridge
            webview._teardown_signal_bridge()

            # User handler should still be in the list
            with core_webview._event_handlers_lock:
                handlers_after = list(core_webview._event_handlers.get("navigation_started", []))
            # Bridge handler removed, user handler remains
            assert len(handlers_after) == len(handlers_before) - 1
            assert len(handlers_after) >= 1

            # Verify user handler is still callable
            for h in handlers_after:
                h({"url": "https://test.com"})
            assert len(user_calls) == 1
            assert user_calls[0] == {"url": "https://test.com"}
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

    def test_cpp_dead_flag_latches_on_runtime_error(self, qapp):
        """_cpp_dead should latch True once C++ object is confirmed destroyed.

        This tests the P1 performance optimization: once objectName() raises
        RuntimeError, subsequent is_alive calls skip the try/except via the
        _cpp_dead fast-path, avoiding overhead on high-frequency events.
        """
        from unittest.mock import patch

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            # Initially _cpp_dead is False
            assert webview._cpp_dead is False
            assert webview.is_alive is True

            # Simulate C++ object destruction by patching objectName
            with patch.object(
                type(webview), "objectName",
                side_effect=RuntimeError("Internal C++ object already deleted"),
            ):
                assert webview.is_alive is False
                # Flag should now be latched
                assert webview._cpp_dead is True

            # Even after un-patching, _cpp_dead short-circuits
            # (without resetting, is_alive stays False due to the flag)
            assert webview._cpp_dead is True
            assert webview.is_alive is False
        finally:
            # Reset for cleanup
            webview._cpp_dead = False
            webview.deleteLater()

    def test_cpp_dead_flag_resets_on_reshow(self, qapp):
        """_cpp_dead should reset to False on re-show (DCC reuse)."""
        from qtpy.QtGui import QCloseEvent, QShowEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            # Force _cpp_dead (simulating edge case)
            webview._cpp_dead = True
            assert webview.is_alive is False

            # Close first (so showEvent triggers the reset path)
            webview._is_closing = True

            # Re-show should reset both flags
            show_event = QShowEvent()
            webview.showEvent(show_event)
            assert webview._cpp_dead is False
            assert webview._is_closing is False
            assert webview.is_alive is True
        finally:
            webview.deleteLater()
