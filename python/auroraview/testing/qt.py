"""Qt testing utilities for AuroraView.

This module provides pytest fixtures and utilities for testing Qt-based
AuroraView components using pytest-qt.

Example:
    ```python
    import pytest
    from auroraview.testing.qt import qt_webview

    @pytest.fixture
    def webview(qtbot):
        from auroraview.testing.qt import create_qt_webview
        return create_qt_webview(qtbot)

    def test_load_html(webview):
        webview.load_html("<h1>Hello</h1>")
        # Test assertions...
    ```
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Callable, Optional

if TYPE_CHECKING:
    from pytestqt.qtbot import QtBot

    from auroraview import QtWebView


def create_qt_webview(
    qtbot: "QtBot",
    url: Optional[str] = None,
    html: Optional[str] = None,
    title: str = "Test WebView",
    width: int = 800,
    height: int = 600,
    **kwargs: Any,
) -> "QtWebView":
    """Create a QtWebView for testing with qtbot.

    Args:
        qtbot: pytest-qt's QtBot fixture
        url: Initial URL to load
        html: Initial HTML content to load
        title: Window title
        width: Window width
        height: Window height
        **kwargs: Additional arguments passed to QtWebView

    Returns:
        QtWebView instance registered with qtbot

    Example:
        ```python
        def test_webview(qtbot):
            webview = create_qt_webview(qtbot, html="<h1>Test</h1>")
            assert webview is not None
        ```
    """
    from auroraview import QtWebView

    webview = QtWebView(
        url=url,
        html=html,
        title=title,
        width=width,
        height=height,
        **kwargs,
    )

    # Register widget with qtbot for proper cleanup
    qtbot.addWidget(webview)

    return webview


def wait_for_loaded(
    qtbot: "QtBot",
    webview: "QtWebView",
    timeout: int = 5000,
) -> None:
    """Wait for WebView to finish loading.

    Args:
        qtbot: pytest-qt's QtBot fixture
        webview: QtWebView instance
        timeout: Timeout in milliseconds

    Raises:
        pytestqt.exceptions.TimeoutError: If loading doesn't complete in time
    """
    # Wait for the loaded signal
    with qtbot.waitSignal(webview.signals.loaded, timeout=timeout):
        pass


def wait_for_message(
    qtbot: "QtBot",
    webview: "QtWebView",
    timeout: int = 5000,
) -> dict:
    """Wait for a message from JavaScript.

    Args:
        qtbot: pytest-qt's QtBot fixture
        webview: QtWebView instance
        timeout: Timeout in milliseconds

    Returns:
        The message data received from JavaScript

    Raises:
        pytestqt.exceptions.TimeoutError: If no message received in time
    """
    with qtbot.waitSignal(webview.signals.message_received, timeout=timeout) as blocker:
        pass
    return blocker.args[0] if blocker.args else {}


class QtWebViewTestHelper:
    """Helper class for testing QtWebView with pytest-qt.

    This class provides convenient methods for common testing patterns.

    Example:
        ```python
        def test_with_helper(qtbot):
            helper = QtWebViewTestHelper(qtbot)
            webview = helper.create_webview(html="<button id='btn'>Click</button>")

            helper.wait_loaded(webview)
            webview.eval_js("document.getElementById('btn').click()")

            message = helper.wait_message(webview)
            assert message['type'] == 'click'
        ```
    """

    def __init__(self, qtbot: "QtBot"):
        """Initialize the helper.

        Args:
            qtbot: pytest-qt's QtBot fixture
        """
        self.qtbot = qtbot
        self._webviews: "list[QtWebView]" = []

    def create_webview(self, **kwargs: Any) -> "QtWebView":
        """Create a QtWebView and track it for cleanup.

        Args:
            **kwargs: Arguments passed to create_qt_webview

        Returns:
            QtWebView instance
        """
        webview = create_qt_webview(self.qtbot, **kwargs)
        self._webviews.append(webview)
        return webview

    def wait_loaded(self, webview: "QtWebView", timeout: int = 5000) -> None:
        """Wait for WebView to finish loading.

        Args:
            webview: QtWebView instance
            timeout: Timeout in milliseconds
        """
        wait_for_loaded(self.qtbot, webview, timeout)

    def wait_message(self, webview: "QtWebView", timeout: int = 5000) -> dict:
        """Wait for a message from JavaScript.

        Args:
            webview: QtWebView instance
            timeout: Timeout in milliseconds

        Returns:
            The message data
        """
        return wait_for_message(self.qtbot, webview, timeout)

    def wait_signal(
        self,
        signal: Any,
        timeout: int = 5000,
        check_params_cb: Optional[Callable[..., bool]] = None,
    ) -> Any:
        """Wait for a Qt signal.

        Args:
            signal: Qt signal to wait for
            timeout: Timeout in milliseconds
            check_params_cb: Optional callback to check signal parameters

        Returns:
            Signal blocker with captured arguments
        """
        return self.qtbot.waitSignal(
            signal,
            timeout=timeout,
            check_params_cb=check_params_cb,
        )

    def process_events(self, timeout: int = 100) -> None:
        """Process Qt events for a short time.

        Args:
            timeout: Time to process events in milliseconds
        """
        self.qtbot.wait(timeout)

    def cleanup(self) -> None:
        """Close all created webviews."""
        for webview in self._webviews:
            try:
                webview.close()
            except Exception:
                pass
        self._webviews.clear()


# Pytest fixture factory
def qt_webview_fixture(qtbot: "QtBot") -> "QtWebView":
    """Pytest fixture that provides a QtWebView instance.

    This is a factory function that can be used to create a pytest fixture.

    Example in conftest.py:
        ```python
        import pytest
        from auroraview.testing.qt import qt_webview_fixture

        @pytest.fixture
        def webview(qtbot):
            return qt_webview_fixture(qtbot)
        ```
    """
    return create_qt_webview(qtbot)


def qt_test_helper_fixture(qtbot: "QtBot") -> QtWebViewTestHelper:
    """Pytest fixture that provides a QtWebViewTestHelper instance.

    Example in conftest.py:
        ```python
        import pytest
        from auroraview.testing.qt import qt_test_helper_fixture

        @pytest.fixture
        def qt_helper(qtbot):
            helper = qt_test_helper_fixture(qtbot)
            yield helper
            helper.cleanup()
        ```
    """
    return QtWebViewTestHelper(qtbot)


__all__ = [
    "create_qt_webview",
    "wait_for_loaded",
    "wait_for_message",
    "QtWebViewTestHelper",
    "qt_webview_fixture",
    "qt_test_helper_fixture",
]
