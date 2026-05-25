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

pytest.importorskip("qtpy", reason="Qt backend requires qtpy and Qt bindings")

# Check if _core module is available (needed for WebView instantiation)

try:
    import auroraview._core  # noqa: F401

    _CORE_AVAILABLE = True
except ImportError:
    _CORE_AVAILABLE = False

# Mark all tests as Qt tests
pytestmark = [pytest.mark.qt]

# Check if we're in CI environment
_IN_CI = os.environ.get("CI", "").lower() == "true"
# Check if we're on Windows
_IS_WINDOWS = sys.platform == "win32"
# Skip WebView instantiation tests in CI on non-Windows platforms
# Windows CI can run these tests with offscreen Qt and WebView2
# Also skip if Rust core is not available
_SKIP_WEBVIEW_TESTS = (_IN_CI and not _IS_WINDOWS) or not _CORE_AVAILABLE


@pytest.mark.skipif(
    _SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI or Rust core not available"
)
class TestQtWebViewLifecycle:
    """Test QtWebView lifecycle management for the new WebView2-based backend."""

    def test_qtwebview_close_event_sets_flag(self, qapp):
        """closeEvent should mark the widget as closing."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            assert webview._is_closing is False

            event = QCloseEvent()
            webview.closeEvent(event)

            assert webview._is_closing is True
        finally:
            webview.deleteLater()

    def test_qtwebview_multiple_close_events_safe(self, qapp):
        """Multiple closeEvent calls should not crash."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event1 = QCloseEvent()
            webview.closeEvent(event1)
            assert webview._is_closing is True

            event2 = QCloseEvent()
            webview.closeEvent(event2)  # Should not crash
        finally:
            webview.deleteLater()

    def test_qtwebview_embeds_webview_core(self, qapp):
        """QtWebView should create an internal WebView backend instance."""
        from auroraview import QtWebView
        from auroraview.core.webview import WebView

        webview = QtWebView()
        try:
            assert hasattr(webview, "_webview")
            assert isinstance(webview._webview, WebView)
        finally:
            webview.deleteLater()

    def test_qtwebview_emit_after_close_does_not_crash(self, qapp):
        """Calling emit after closeEvent should be a no-op and not crash."""
        from qtpy.QtGui import QCloseEvent

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            event = QCloseEvent()
            webview.closeEvent(event)

            # Should not raise even though the underlying WebView has been closed
            webview.emit("test_event", {"value": 1})
        finally:
            webview.deleteLater()


@pytest.mark.skipif(
    _SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI or Rust core not available"
)
class TestQtWebViewEventProcessing:
    """Test event processing and UI updates.

    Cleanup convention: all tests use ``deleteLater()`` only in ``finally``.
    ``WA_DeleteOnClose`` is set by ``QtWebView.__init__`` so an explicit
    ``close()`` is unnecessary — ``deleteLater()`` is sufficient to schedule
    C++ destructor dispatch on the next event-loop tick.
    """

    @staticmethod
    def _spy_event_processor(webview, monkeypatch):
        """Patch ``webview._event_processor.process`` with a counting spy.

        Returns a list that accumulates one ``True`` entry per call.
        The original method is still invoked so side-effects are preserved.
        """
        calls = []
        original = webview._event_processor.process

        def _spy():
            calls.append(True)
            return original()

        monkeypatch.setattr(webview._event_processor, "process", _spy)
        return calls

    def test_event_processor_processes_events(self, qapp, monkeypatch):
        """Test that event processor processes events correctly."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            process_called = self._spy_event_processor(webview, monkeypatch)

            # Trigger event processing via eval_js
            webview._webview.eval_js("console.log('test')")

            # Verify event processor was called
            assert len(process_called) > 0, "eval_js should trigger event processor"
        finally:
            webview.deleteLater()

    def test_emit_uses_event_processor(self, qapp, monkeypatch):
        """Test that WebView.emit() uses event processor strategy."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            process_called = self._spy_event_processor(webview, monkeypatch)

            # Emit event
            webview._webview.emit("test_event", {"data": "test"})

            # Verify event processor was called
            assert len(process_called) == 1, "emit() should trigger event processor"
        finally:
            webview.deleteLater()

    def test_eval_js_uses_event_processor(self, qapp, monkeypatch):
        """Test that WebView.eval_js() uses event processor strategy."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            process_called = self._spy_event_processor(webview, monkeypatch)

            # Execute JavaScript
            webview._webview.eval_js("console.log('test')")

            # Verify event processor was called
            assert len(process_called) == 1, "eval_js() should trigger event processor"
        finally:
            webview.deleteLater()

    def test_qtwebview_auto_installs_event_processor(self, qapp):
        """Test that QtWebView automatically installs QtEventProcessor."""
        from auroraview import QtWebView
        from auroraview.integration.qt import QtEventProcessor

        webview = QtWebView()
        try:
            # Verify event processor is installed
            assert hasattr(webview, "_event_processor")
            assert isinstance(webview._event_processor, QtEventProcessor)
            assert webview._webview._event_processor is webview._event_processor
        finally:
            webview.deleteLater()


@pytest.mark.skipif(
    _SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI or Rust core not available"
)
class TestQtWebViewAppIntegration:
    """Lightweight tests around Qt-specific integration flags."""

    def test_wa_delete_on_close_set(self, qapp):
        """QtWebView should delete itself when closed."""
        from qtpy.QtCore import Qt

        from auroraview import QtWebView

        webview = QtWebView()
        try:
            # Verify WA_DeleteOnClose is set
            assert webview.testAttribute(Qt.WA_DeleteOnClose) is True
        finally:
            webview.deleteLater()


@pytest.mark.skipif(
    _SKIP_WEBVIEW_TESTS, reason="WebView tests require display in CI or Rust core not available"
)
class TestQtWebViewMutexFlags:
    """Test the cross-task mutex flags initialised by ``QtWebView.__init__``.

    These attributes back the ``acquire_exclusive`` guards in
    ``EmbeddingMixin._schedule_child_window_fixes`` and
    ``LifecycleMixin._init_webview_progressive``. They must be present
    and false / ``None`` on a freshly constructed widget so the first
    callback to acquire them succeeds.

    Cleanup convention: ``deleteLater()`` only — see TestQtWebViewEventProcessing.
    """

    @pytest.mark.parametrize(
        "attr,expected",
        [
            ("_geometry_sync_in_progress", False),
            ("_child_window_fix_in_progress", False),
            ("_last_synced_bounds", None),
        ],
        ids=["geometry_sync", "child_window_fix", "last_synced_bounds"],
    )
    def test_mutex_flag_initialised(self, qapp, attr, expected):
        """Each mutex-related attribute must have the correct initial value."""
        from auroraview import QtWebView

        webview = QtWebView()
        try:
            assert getattr(webview, attr) == expected
        finally:
            webview.deleteLater()

    def test_mutex_flag_names_match_locks_module(self, qapp):
        """The flag names used by ``QtWebView`` must match the canonical
        constants in :mod:`auroraview.integration.qt._locks`; otherwise
        ``acquire_exclusive`` would silently use stale values via
        ``getattr(host, name, False)``.
        """
        from auroraview import QtWebView
        from auroraview.integration.qt._locks import (
            FLAG_CHILD_WINDOW_FIX,
            FLAG_GEOMETRY_SYNC,
        )

        webview = QtWebView()
        try:
            assert hasattr(webview, FLAG_GEOMETRY_SYNC)
            assert hasattr(webview, FLAG_CHILD_WINDOW_FIX)
        finally:
            webview.deleteLater()
