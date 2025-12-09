"""Test Qt signal definitions and signal state management.

These tests verify:
1. All Qt signals are correctly defined
2. Signal emitter methods work correctly
3. State tracking is accurate

These are unit tests that don't require a running WebView or Qt event loop.
"""

import os
import sys

import pytest

# Mark all tests as Qt tests
pytestmark = [pytest.mark.qt]

# Check if we're in CI environment - skip QApplication tests that require display
_IN_CI = os.environ.get("CI", "").lower() == "true"

# Skip tests that need QApplication in CI (even with xvfb, Qt can crash)
skip_qapp_in_ci = pytest.mark.skipif(
    _IN_CI, reason="QApplication tests skipped in CI (requires display environment)"
)


class TestQtWebViewSignalsDefined:
    """Test that QtWebViewSignals has all required signals defined."""

    @pytest.fixture
    def signals_class(self):
        """Get the QtWebViewSignals class."""
        from auroraview.integration.qt.signals import QtWebViewSignals

        return QtWebViewSignals

    def test_navigation_signals_exist(self, signals_class):
        """Test navigation signals are defined."""
        assert hasattr(signals_class, "urlChanged")
        assert hasattr(signals_class, "loadStarted")
        assert hasattr(signals_class, "loadFinished")
        assert hasattr(signals_class, "loadProgress")

    def test_page_signals_exist(self, signals_class):
        """Test page signals are defined."""
        assert hasattr(signals_class, "titleChanged")
        assert hasattr(signals_class, "iconChanged")
        assert hasattr(signals_class, "iconUrlChanged")

    def test_error_signals_exist(self, signals_class):
        """Test error handling signals are defined."""
        assert hasattr(signals_class, "jsError")
        assert hasattr(signals_class, "consoleMessage")
        assert hasattr(signals_class, "renderProcessTerminated")

    def test_ipc_signals_exist(self, signals_class):
        """Test IPC signals are defined."""
        assert hasattr(signals_class, "ipcMessageReceived")

    def test_selection_signals_exist(self, signals_class):
        """Test selection signals are defined."""
        assert hasattr(signals_class, "selectionChanged")

    def test_window_signals_exist(self, signals_class):
        """Test window signals are defined."""
        assert hasattr(signals_class, "windowCloseRequested")
        assert hasattr(signals_class, "fullScreenRequested")


@skip_qapp_in_ci
class TestQtWebViewSignalsInstance:
    """Test QtWebViewSignals instance behavior."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    @pytest.fixture
    def signals(self, qapp):
        """Provide a QtWebViewSignals instance."""
        from auroraview.integration.qt.signals import QtWebViewSignals

        return QtWebViewSignals()

    def test_initial_state(self, signals):
        """Test initial state values."""
        assert signals.current_url == ""
        assert signals.current_title == ""
        assert signals.is_loading is False
        assert signals.load_progress_value == 0

    def test_emit_url_changed(self, signals):
        """Test emitting urlChanged signal updates state."""
        received = []
        signals.urlChanged.connect(lambda url: received.append(url))

        signals.emit_url_changed("https://example.com")

        assert received == ["https://example.com"]
        assert signals.current_url == "https://example.com"

    def test_emit_url_changed_no_duplicate(self, signals):
        """Test that urlChanged doesn't emit for same URL."""
        received = []
        signals.urlChanged.connect(lambda url: received.append(url))

        signals.emit_url_changed("https://example.com")
        signals.emit_url_changed("https://example.com")  # Same URL

        assert len(received) == 1

    def test_emit_load_started(self, signals):
        """Test emitting loadStarted signal."""
        received = []
        signals.loadStarted.connect(lambda: received.append(True))

        signals.emit_load_started()

        assert received == [True]
        assert signals.is_loading is True

    def test_emit_load_finished(self, signals):
        """Test emitting loadFinished signal."""
        received = []
        signals.loadFinished.connect(lambda ok: received.append(ok))

        signals.emit_load_finished(True)

        assert received == [True]
        assert signals.is_loading is False
        assert signals.load_progress_value == 100

    def test_emit_load_progress(self, signals):
        """Test emitting loadProgress signal."""
        received = []
        signals.loadProgress.connect(lambda p: received.append(p))

        signals.emit_load_progress(50)

        assert received == [50]
        assert signals.load_progress_value == 50

    def test_emit_load_progress_clamped(self, signals):
        """Test that progress is clamped to 0-100."""
        received = []
        signals.loadProgress.connect(lambda p: received.append(p))

        # Reset progress first so we can get 0
        signals._current_progress = -1

        signals.emit_load_progress(-10)  # Should be clamped to 0
        signals.emit_load_progress(150)  # Should be clamped to 100

        # First should be 0, second should be 100
        assert 0 in received
        assert 100 in received


@skip_qapp_in_ci
class TestQtWebViewErrorSignals:
    """Test error handling signals."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    @pytest.fixture
    def signals(self, qapp):
        """Provide a QtWebViewSignals instance."""
        from auroraview.integration.qt.signals import QtWebViewSignals

        return QtWebViewSignals()

    def test_emit_js_error(self, signals):
        """Test emitting jsError signal."""
        received = []
        signals.jsError.connect(lambda msg, line, src: received.append((msg, line, src)))

        signals.emit_js_error("Uncaught TypeError", 42, "app.js")

        assert received == [("Uncaught TypeError", 42, "app.js")]

    def test_emit_console_message(self, signals):
        """Test emitting consoleMessage signal."""
        received = []
        signals.consoleMessage.connect(
            lambda lvl, msg, line, src: received.append((lvl, msg, line, src))
        )

        signals.emit_console_message(2, "Error occurred", 10, "script.js")

        assert received == [(2, "Error occurred", 10, "script.js")]

    def test_emit_render_process_terminated(self, signals):
        """Test emitting renderProcessTerminated signal."""
        received = []
        signals.renderProcessTerminated.connect(
            lambda status, code: received.append((status, code))
        )

        signals.emit_render_process_terminated(1, 255)

        assert received == [(1, 255)]


@skip_qapp_in_ci
class TestQtWebViewIpcSignals:
    """Test IPC signals."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    @pytest.fixture
    def signals(self, qapp):
        """Provide a QtWebViewSignals instance."""
        from auroraview.integration.qt.signals import QtWebViewSignals

        return QtWebViewSignals()

    def test_emit_ipc_message(self, signals):
        """Test emitting ipcMessageReceived signal."""
        received = []
        signals.ipcMessageReceived.connect(lambda event, data: received.append((event, data)))

        signals.emit_ipc_message("save_file", {"path": "/tmp/test.txt"})

        assert len(received) == 1
        assert received[0][0] == "save_file"
        assert received[0][1] == {"path": "/tmp/test.txt"}

    def test_emit_ipc_message_with_none_data(self, signals):
        """Test emitting ipcMessageReceived with None data."""
        received = []
        signals.ipcMessageReceived.connect(lambda event, data: received.append((event, data)))

        signals.emit_ipc_message("ping", None)

        assert received == [("ping", None)]


@skip_qapp_in_ci
class TestQtWebViewSelectionSignals:
    """Test selection signals."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    @pytest.fixture
    def signals(self, qapp):
        """Provide a QtWebViewSignals instance."""
        from auroraview.integration.qt.signals import QtWebViewSignals

        return QtWebViewSignals()

    def test_emit_selection_changed(self, signals):
        """Test emitting selectionChanged signal."""
        received = []
        signals.selectionChanged.connect(lambda: received.append(True))

        signals.emit_selection_changed()

        assert received == [True]

    def test_emit_icon_url_changed(self, signals):
        """Test emitting iconUrlChanged signal."""
        received = []
        signals.iconUrlChanged.connect(lambda url: received.append(url))

        signals.emit_icon_url_changed("https://example.com/favicon.ico")

        assert received == ["https://example.com/favicon.ico"]


class TestQtWebViewSignalsOnClass:
    """Test that QtWebView class has all signals defined."""

    def test_qtwebview_has_navigation_signals(self):
        """Test QtWebView has navigation signals."""
        from auroraview import QtWebView

        assert hasattr(QtWebView, "urlChanged")
        assert hasattr(QtWebView, "loadStarted")
        assert hasattr(QtWebView, "loadFinished")
        assert hasattr(QtWebView, "loadProgress")

    def test_qtwebview_has_page_signals(self):
        """Test QtWebView has page signals."""
        from auroraview import QtWebView

        assert hasattr(QtWebView, "titleChanged")
        assert hasattr(QtWebView, "iconChanged")
        assert hasattr(QtWebView, "iconUrlChanged")

    def test_qtwebview_has_error_signals(self):
        """Test QtWebView has error signals."""
        from auroraview import QtWebView

        assert hasattr(QtWebView, "jsError")
        assert hasattr(QtWebView, "consoleMessage")
        assert hasattr(QtWebView, "renderProcessTerminated")

    def test_qtwebview_has_ipc_signals(self):
        """Test QtWebView has IPC signals."""
        from auroraview import QtWebView

        assert hasattr(QtWebView, "ipcMessageReceived")

    def test_qtwebview_has_selection_signals(self):
        """Test QtWebView has selection signals."""
        from auroraview import QtWebView

        assert hasattr(QtWebView, "selectionChanged")

    def test_qtwebview_has_window_signals(self):
        """Test QtWebView has window signals."""
        from auroraview import QtWebView

        assert hasattr(QtWebView, "windowCloseRequested")
        assert hasattr(QtWebView, "fullScreenRequested")
