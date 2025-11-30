"""Headless testing support for AuroraView.

This module provides utilities for running WebView tests without
requiring user interaction or visible windows.

Example:
    ```python
    from auroraview.testing import HeadlessTestRunner, DomAssertions

    def test_login_form():
        with HeadlessTestRunner() as runner:
            # Load HTML
            runner.load_html('''
                <form id="login">
                    <input id="email" type="email">
                    <input id="password" type="password">
                    <button id="submit">Login</button>
                    <div id="result"></div>
                </form>
            ''')

            # Interact with DOM
            runner.dom("#email").set_value("test@example.com")
            runner.dom("#password").set_value("secret123")
            runner.dom("#submit").click()

            # Assert results
            runner.assertions.wait_for_text("#result", "Success", timeout=2)
    ```
"""

from __future__ import annotations

import logging
import threading
import time
from contextlib import contextmanager
from typing import TYPE_CHECKING, Iterator, Optional

if TYPE_CHECKING:
    from ..dom import Element, ElementCollection
    from ..webview import WebView

from .dom_assertions import DomAssertions

logger = logging.getLogger(__name__)


class HeadlessTestRunner:
    """A test runner for headless WebView testing.

    Provides a context manager that creates a WebView, runs tests,
    and cleans up automatically. The WebView is hidden (no decorations)
    and runs in a background thread.
    """

    def __init__(
        self,
        title: str = "AuroraView Test",
        width: int = 800,
        height: int = 600,
        timeout: float = 10.0,
    ):
        """Initialize the headless test runner.

        Args:
            title: Window title (not visible in headless mode).
            width: Window width in pixels.
            height: Window height in pixels.
            timeout: Default timeout for operations in seconds.
        """
        self.title = title
        self.width = width
        self.height = height
        self.timeout = timeout
        self._webview: Optional[WebView] = None
        self._assertions: Optional[DomAssertions] = None
        self._ready = threading.Event()
        self._closed = threading.Event()

    @property
    def webview(self) -> WebView:
        """Get the WebView instance."""
        if self._webview is None:
            raise RuntimeError("HeadlessTestRunner not started. Use 'with' context.")
        return self._webview

    @property
    def assertions(self) -> DomAssertions:
        """Get the DomAssertions instance."""
        if self._assertions is None:
            raise RuntimeError("HeadlessTestRunner not started. Use 'with' context.")
        return self._assertions

    def __enter__(self) -> "HeadlessTestRunner":
        """Start the headless test runner."""
        from ..webview import WebView

        # Create WebView with no decorations (pseudo-headless)
        self._webview = WebView(
            title=self.title,
            width=self.width,
            height=self.height,
            decorations=False,
            resizable=False,
        )
        self._assertions = DomAssertions(self._webview, timeout=self.timeout)

        # Start in background thread
        self._webview.show(wait=False)

        # Wait a bit for WebView to initialize
        time.sleep(0.5)
        self._ready.set()

        logger.info("HeadlessTestRunner started")
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Stop the headless test runner."""
        if self._webview:
            try:
                self._webview.close()
            except Exception as e:
                logger.warning(f"Error closing WebView: {e}")
        self._closed.set()
        logger.info("HeadlessTestRunner stopped")
        return False

    # ========== WebView Delegation ==========

    def load_html(self, html: str) -> None:
        """Load HTML content into the WebView."""
        self.webview.load_html(html)
        # Give time for HTML to load
        time.sleep(0.2)

    def load_url(self, url: str) -> None:
        """Navigate to a URL."""
        self.webview.load_url(url)
        # Give time for page to load
        time.sleep(0.5)

    def eval_js(self, script: str) -> None:
        """Execute JavaScript in the WebView."""
        self.webview.eval_js(script)

    # ========== DOM Access ==========

    def dom(self, selector: str) -> Element:
        """Get a DOM element by CSS selector."""
        return self.webview.dom(selector)

    def dom_all(self, selector: str) -> ElementCollection:
        """Get all DOM elements matching a CSS selector."""
        return self.webview.dom_all(selector)

    def dom_by_id(self, element_id: str) -> Element:
        """Get a DOM element by ID."""
        return self.webview.dom_by_id(element_id)

    # ========== Convenience Methods ==========

    def click(self, selector: str) -> None:
        """Click an element."""
        self.dom(selector).click()

    def type_text(self, selector: str, text: str) -> None:
        """Type text into an input element."""
        self.dom(selector).type_text(text)

    def set_value(self, selector: str, value: str) -> None:
        """Set the value of an input element."""
        self.dom(selector).set_value(value)

    def get_text(self, selector: str) -> str:
        """Get the text content of an element."""
        return self.dom(selector).get_text()

    def wait(self, seconds: float) -> None:
        """Wait for a specified duration."""
        time.sleep(seconds)


@contextmanager
def headless_test(
    html: Optional[str] = None,
    url: Optional[str] = None,
    timeout: float = 10.0,
) -> Iterator[HeadlessTestRunner]:
    """Context manager for quick headless testing.

    Args:
        html: HTML content to load initially.
        url: URL to navigate to initially.
        timeout: Default timeout for operations.

    Yields:
        HeadlessTestRunner instance.

    Example:
        ```python
        from auroraview.testing import headless_test

        def test_button_click():
            with headless_test(html="<button id='btn'>Click me</button>") as runner:
                runner.click("#btn")
                # ... assertions
        ```
    """
    runner = HeadlessTestRunner(timeout=timeout)
    with runner:
        if html:
            runner.load_html(html)
        elif url:
            runner.load_url(url)
        yield runner
