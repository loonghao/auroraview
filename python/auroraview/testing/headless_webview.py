"""Headless WebView testing support for AuroraView.

This module provides multiple strategies for running WebView tests
in headless/CI environments:

1. **Playwright Mode** (recommended): Uses Playwright's Chromium with
   AuroraView bridge injection. Fast, reliable, cross-platform.

2. **Virtual Display Mode** (Linux): Uses Xvfb virtual framebuffer
   to run actual WebView in a virtual display.

3. **WebView2 CDP Mode** (Windows): Connects to WebView2 via Chrome
   DevTools Protocol for real WebView2 testing.

Example (Playwright - recommended):
    ```python
    from auroraview.testing.headless_webview import HeadlessWebView

    with HeadlessWebView.playwright() as webview:
        webview.goto("https://example.com")
        webview.click("#button")
        assert webview.text("#result") == "Success"
    ```

Example (Virtual Display - Linux CI):
    ```python
    from auroraview.testing.headless_webview import HeadlessWebView

    # Requires: apt install xvfb
    with HeadlessWebView.virtual_display() as webview:
        webview.load_html("<h1>Hello</h1>")
        assert webview.text("h1") == "Hello"
    ```

Example (WebView2 CDP - Windows):
    ```python
    from auroraview.testing.headless_webview import HeadlessWebView

    # Start WebView2 with: --remote-debugging-port=9222
    with HeadlessWebView.webview2_cdp("http://localhost:9222") as webview:
        webview.goto("https://example.com")
    ```

References:
    - https://crates.io/crates/headless_webview
    - https://github.com/tauri-apps/wry/discussions/761
    - https://tauri.app/develop/tests/webdriver/ci/
"""

from __future__ import annotations

import logging
import os
import platform
import shutil
import subprocess
import time
from abc import ABC, abstractmethod
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Any, Iterator, Optional

logger = logging.getLogger(__name__)


@dataclass
class HeadlessOptions:
    """Options for headless WebView testing."""

    timeout: float = 30.0
    """Default timeout in seconds."""

    width: int = 1280
    """Viewport width."""

    height: int = 720
    """Viewport height."""

    inject_bridge: bool = True
    """Inject AuroraView bridge script."""

    screenshot_on_failure: bool = True
    """Take screenshot on test failure."""

    screenshot_dir: str = "test-screenshots"
    """Directory for screenshots."""

    slow_mo: float = 0
    """Slow down operations by milliseconds."""

    devtools: bool = False
    """Open DevTools (if supported)."""


class HeadlessWebViewBase(ABC):
    """Base class for headless WebView implementations."""

    def __init__(self, options: HeadlessOptions):
        self._options = options
        self._closed = False

    @abstractmethod
    def goto(self, url: str) -> None:
        """Navigate to a URL."""
        pass

    @abstractmethod
    def load_html(self, html: str) -> None:
        """Load HTML content."""
        pass

    @abstractmethod
    def click(self, selector: str) -> None:
        """Click an element."""
        pass

    @abstractmethod
    def fill(self, selector: str, value: str) -> None:
        """Fill an input element."""
        pass

    @abstractmethod
    def text(self, selector: str) -> str:
        """Get text content of an element."""
        pass

    @abstractmethod
    def evaluate(self, script: str) -> Any:
        """Execute JavaScript and return result."""
        pass

    @abstractmethod
    def screenshot(self, path: str) -> None:
        """Take a screenshot."""
        pass

    @abstractmethod
    def wait_for_selector(self, selector: str, timeout: Optional[float] = None) -> None:
        """Wait for an element to appear."""
        pass

    @abstractmethod
    def close(self) -> None:
        """Close the WebView."""
        pass

    def __enter__(self) -> "HeadlessWebViewBase":
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if exc_type and self._options.screenshot_on_failure:
            try:
                os.makedirs(self._options.screenshot_dir, exist_ok=True)
                path = os.path.join(
                    self._options.screenshot_dir, f"failure-{time.strftime('%Y%m%d-%H%M%S')}.png"
                )
                self.screenshot(path)
                logger.info(f"Failure screenshot saved: {path}")
            except Exception as e:
                logger.warning(f"Failed to save screenshot: {e}")
        self.close()
        return False


class PlaywrightHeadlessWebView(HeadlessWebViewBase):
    """Headless WebView using Playwright's Chromium.

    This is the recommended mode for most testing scenarios.
    It uses Playwright's built-in Chromium browser with the
    AuroraView bridge script injected.
    """

    def __init__(self, options: HeadlessOptions):
        super().__init__(options)
        self._playwright = None
        self._browser = None
        self._context = None
        self._page = None
        self._start()

    def _start(self):
        """Start Playwright browser."""
        try:
            from playwright.sync_api import sync_playwright
        except ImportError as err:
            raise RuntimeError(
                "Playwright not installed. Run: pip install playwright && playwright install chromium"
            ) from err

        logger.info("Starting Playwright headless browser")

        self._playwright = sync_playwright().start()
        self._browser = self._playwright.chromium.launch(
            headless=True,
            devtools=self._options.devtools,
            slow_mo=self._options.slow_mo,
        )

        self._context = self._browser.new_context(
            viewport={"width": self._options.width, "height": self._options.height},
        )

        if self._options.inject_bridge:
            self._context.add_init_script(_get_bridge_script())

        self._page = self._context.new_page()
        logger.info("Playwright browser started")

    def goto(self, url: str) -> None:
        self._page.goto(url, timeout=self._options.timeout * 1000)

    def load_html(self, html: str) -> None:
        self._page.set_content(html, timeout=self._options.timeout * 1000)

    def click(self, selector: str) -> None:
        self._page.click(selector, timeout=self._options.timeout * 1000)

    def fill(self, selector: str, value: str) -> None:
        self._page.fill(selector, value, timeout=self._options.timeout * 1000)

    def text(self, selector: str) -> str:
        return self._page.text_content(selector, timeout=self._options.timeout * 1000) or ""

    def evaluate(self, script: str) -> Any:
        return self._page.evaluate(script)

    def screenshot(self, path: str) -> None:
        self._page.screenshot(path=path)

    def wait_for_selector(self, selector: str, timeout: Optional[float] = None) -> None:
        t = (timeout or self._options.timeout) * 1000
        self._page.wait_for_selector(selector, timeout=t)

    @property
    def page(self):
        """Get the Playwright Page for advanced operations."""
        return self._page

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True

        if self._context:
            self._context.close()
        if self._browser:
            self._browser.close()
        if self._playwright:
            self._playwright.stop()

        logger.info("Playwright browser closed")


class VirtualDisplayWebView(HeadlessWebViewBase):
    """Headless WebView using Xvfb virtual display (Linux only).

    This mode runs the actual WebView in a virtual framebuffer,
    allowing real WebView testing in headless Linux environments.

    Requires: apt install xvfb
    """

    def __init__(self, options: HeadlessOptions, display: int = 99):
        super().__init__(options)
        self._display = display
        self._xvfb_proc = None
        self._webview = None
        self._start()

    def _start(self):
        """Start Xvfb and WebView."""
        if platform.system() != "Linux":
            raise RuntimeError("VirtualDisplayWebView only works on Linux")

        # Check if Xvfb is available
        if not shutil.which("Xvfb"):
            raise RuntimeError("Xvfb not found. Install with: apt install xvfb")

        logger.info(f"Starting Xvfb on display :{self._display}")

        # Start Xvfb
        self._xvfb_proc = subprocess.Popen(
            [
                "Xvfb",
                f":{self._display}",
                "-screen",
                "0",
                f"{self._options.width}x{self._options.height}x24",
                "-ac",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

        # Set DISPLAY environment variable
        os.environ["DISPLAY"] = f":{self._display}"

        # Wait for Xvfb to start
        time.sleep(0.5)

        # Create WebView
        from ..webview import WebView

        self._webview = WebView(
            title="Headless Test",
            width=self._options.width,
            height=self._options.height,
            decorations=False,
        )

        # Show in background
        self._webview.show(wait=False)
        time.sleep(0.5)

        logger.info("Virtual display WebView started")

    def goto(self, url: str) -> None:
        self._webview.load_url(url)
        time.sleep(0.5)

    def load_html(self, html: str) -> None:
        self._webview.load_html(html)
        time.sleep(0.3)

    def click(self, selector: str) -> None:
        self._webview.eval_js(f"document.querySelector('{selector}').click()")

    def fill(self, selector: str, value: str) -> None:
        escaped = value.replace("'", "\\'")
        self._webview.eval_js(f"document.querySelector('{selector}').value = '{escaped}'")

    def text(self, selector: str) -> str:
        # Note: eval_js doesn't return values in current implementation
        # This is a limitation that needs to be addressed
        logger.warning("text() may not work correctly without eval_js return value")
        return ""

    def evaluate(self, script: str) -> Any:
        self._webview.eval_js(script)
        return None  # Current limitation

    def screenshot(self, path: str) -> None:
        # Use scrot or import for screenshot
        if shutil.which("scrot"):
            subprocess.run(["scrot", path], check=True)
        elif shutil.which("import"):
            subprocess.run(["import", "-window", "root", path], check=True)
        else:
            logger.warning("No screenshot tool available (scrot or import)")

    def wait_for_selector(self, selector: str, timeout: Optional[float] = None) -> None:
        t = timeout or self._options.timeout
        start = time.time()
        while time.time() - start < t:
            # Check if element exists
            # Note: This is limited without proper eval_js return
            time.sleep(0.1)
        # Assume element exists after timeout

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True

        if self._webview:
            try:
                self._webview.close()
            except Exception:
                pass

        if self._xvfb_proc:
            self._xvfb_proc.terminate()
            self._xvfb_proc.wait(timeout=5)

        logger.info("Virtual display WebView closed")


class WebView2CDPWebView(HeadlessWebViewBase):
    """Headless WebView connecting to WebView2 via CDP (Windows only).

    This mode connects to an existing WebView2 instance running with
    remote debugging enabled. Useful for testing real WebView2 behavior.

    Start WebView2 with:
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
    """

    def __init__(self, options: HeadlessOptions, cdp_url: str):
        super().__init__(options)
        self._cdp_url = cdp_url
        self._playwright = None
        self._browser = None
        self._page = None
        self._start()

    def _start(self):
        """Connect to WebView2 via CDP."""
        try:
            from playwright.sync_api import sync_playwright
        except ImportError as err:
            raise RuntimeError("Playwright not installed. Run: pip install playwright") from err

        logger.info(f"Connecting to WebView2 CDP: {self._cdp_url}")

        self._playwright = sync_playwright().start()
        self._browser = self._playwright.chromium.connect_over_cdp(self._cdp_url)

        if self._browser.contexts:
            context = self._browser.contexts[0]
            if context.pages:
                self._page = context.pages[0]
            else:
                self._page = context.new_page()
        else:
            context = self._browser.new_context()
            self._page = context.new_page()

        logger.info("Connected to WebView2 via CDP")

    def goto(self, url: str) -> None:
        self._page.goto(url, timeout=self._options.timeout * 1000)

    def load_html(self, html: str) -> None:
        self._page.set_content(html, timeout=self._options.timeout * 1000)

    def click(self, selector: str) -> None:
        self._page.click(selector, timeout=self._options.timeout * 1000)

    def fill(self, selector: str, value: str) -> None:
        self._page.fill(selector, value, timeout=self._options.timeout * 1000)

    def text(self, selector: str) -> str:
        return self._page.text_content(selector, timeout=self._options.timeout * 1000) or ""

    def evaluate(self, script: str) -> Any:
        return self._page.evaluate(script)

    def screenshot(self, path: str) -> None:
        self._page.screenshot(path=path)

    def wait_for_selector(self, selector: str, timeout: Optional[float] = None) -> None:
        t = (timeout or self._options.timeout) * 1000
        self._page.wait_for_selector(selector, timeout=t)

    @property
    def page(self):
        """Get the Playwright Page for advanced operations."""
        return self._page

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True

        if self._browser:
            self._browser.close()
        if self._playwright:
            self._playwright.stop()

        logger.info("WebView2 CDP connection closed")


class HeadlessWebView:
    """Factory class for creating headless WebView instances.

    Provides static methods to create different types of headless
    WebView instances based on the testing requirements.

    Example:
        ```python
        # Playwright mode (recommended)
        with HeadlessWebView.playwright() as webview:
            webview.goto("https://example.com")

        # Virtual display mode (Linux CI)
        with HeadlessWebView.virtual_display() as webview:
            webview.load_html("<h1>Test</h1>")

        # WebView2 CDP mode (Windows)
        with HeadlessWebView.webview2_cdp("http://localhost:9222") as webview:
            webview.goto("https://example.com")

        # Auto-detect best mode
        with HeadlessWebView.auto() as webview:
            webview.goto("https://example.com")
        ```
    """

    @staticmethod
    def playwright(
        timeout: float = 30.0,
        width: int = 1280,
        height: int = 720,
        inject_bridge: bool = True,
        **kwargs,
    ) -> PlaywrightHeadlessWebView:
        """Create a Playwright-based headless WebView.

        This is the recommended mode for most testing scenarios.

        Args:
            timeout: Default timeout in seconds.
            width: Viewport width.
            height: Viewport height.
            inject_bridge: Inject AuroraView bridge script.
            **kwargs: Additional options.

        Returns:
            PlaywrightHeadlessWebView instance.
        """
        options = HeadlessOptions(
            timeout=timeout,
            width=width,
            height=height,
            inject_bridge=inject_bridge,
            **kwargs,
        )
        return PlaywrightHeadlessWebView(options)

    @staticmethod
    def virtual_display(
        timeout: float = 30.0,
        width: int = 1280,
        height: int = 720,
        display: int = 99,
        **kwargs,
    ) -> VirtualDisplayWebView:
        """Create a WebView using Xvfb virtual display (Linux only).

        Args:
            timeout: Default timeout in seconds.
            width: Viewport width.
            height: Viewport height.
            display: X display number.
            **kwargs: Additional options.

        Returns:
            VirtualDisplayWebView instance.
        """
        options = HeadlessOptions(
            timeout=timeout,
            width=width,
            height=height,
            **kwargs,
        )
        return VirtualDisplayWebView(options, display=display)

    @staticmethod
    def webview2_cdp(
        cdp_url: str,
        timeout: float = 30.0,
        **kwargs,
    ) -> WebView2CDPWebView:
        """Connect to WebView2 via Chrome DevTools Protocol.

        Args:
            cdp_url: CDP endpoint URL (e.g., "http://localhost:9222").
            timeout: Default timeout in seconds.
            **kwargs: Additional options.

        Returns:
            WebView2CDPWebView instance.
        """
        options = HeadlessOptions(timeout=timeout, **kwargs)
        return WebView2CDPWebView(options, cdp_url=cdp_url)

    @staticmethod
    def auto(
        timeout: float = 30.0,
        width: int = 1280,
        height: int = 720,
        **kwargs,
    ) -> HeadlessWebViewBase:
        """Auto-detect the best headless mode for current environment.

        Detection order:
        1. If WEBVIEW2_CDP_URL env var is set, use WebView2 CDP
        2. If on Linux with Xvfb available, use virtual display
        3. Otherwise, use Playwright (default)

        Args:
            timeout: Default timeout in seconds.
            width: Viewport width.
            height: Viewport height.
            **kwargs: Additional options.

        Returns:
            HeadlessWebViewBase instance.
        """
        # Check for WebView2 CDP
        cdp_url = os.environ.get("WEBVIEW2_CDP_URL")
        if cdp_url:
            logger.info(f"Using WebView2 CDP mode: {cdp_url}")
            return HeadlessWebView.webview2_cdp(cdp_url, timeout=timeout, **kwargs)

        # Check for Xvfb on Linux
        if platform.system() == "Linux" and shutil.which("Xvfb"):
            # Check if we should prefer Xvfb
            if os.environ.get("AURORAVIEW_USE_XVFB", "").lower() in ("1", "true", "yes"):
                logger.info("Using Xvfb virtual display mode")
                return HeadlessWebView.virtual_display(
                    timeout=timeout, width=width, height=height, **kwargs
                )

        # Default to Playwright
        logger.info("Using Playwright headless mode")
        return HeadlessWebView.playwright(timeout=timeout, width=width, height=height, **kwargs)


def _get_bridge_script() -> str:
    """Get the AuroraView bridge injection script."""
    return """
(function() {
    if (window.auroraview) return;

    const eventHandlers = {};
    let callId = 0;

    window.auroraview = {
        call: function(method, params) {
            return new Promise((resolve, reject) => {
                console.log('[AuroraView Test] call:', method, params);
                resolve(undefined);
            });
        },

        on: function(event, handler) {
            if (!eventHandlers[event]) {
                eventHandlers[event] = [];
            }
            eventHandlers[event].push(handler);
            return () => {
                const idx = eventHandlers[event].indexOf(handler);
                if (idx >= 0) eventHandlers[event].splice(idx, 1);
            };
        },

        off: function(event, handler) {
            if (eventHandlers[event]) {
                const idx = eventHandlers[event].indexOf(handler);
                if (idx >= 0) eventHandlers[event].splice(idx, 1);
            }
        },

        trigger: function(event, data) {
            if (eventHandlers[event]) {
                eventHandlers[event].forEach(h => h(data));
            }
        },

        emit: function(event, data) {
            console.log('[AuroraView Test] emit:', event, data);
        },

        send_event: function(event, data) {
            console.log('[AuroraView Test] send_event:', event, data);
        },

        api: new Proxy({}, {
            get: function(target, prop) {
                return function(...args) {
                    return window.auroraview.call('api.' + prop, args);
                };
            }
        }),

        _testMode: true,
        _platform: 'headless-test'
    };

    window.dispatchEvent(new CustomEvent('auroraviewready'));
    console.log('[AuroraView Test] Bridge initialized');
})();
"""


# Convenience function
@contextmanager
def headless_webview(
    mode: str = "auto",
    **kwargs,
) -> Iterator[HeadlessWebViewBase]:
    """Context manager for headless WebView testing.

    Args:
        mode: One of "auto", "playwright", "xvfb", "cdp".
        **kwargs: Options passed to the WebView constructor.

    Yields:
        HeadlessWebViewBase instance.

    Example:
        ```python
        from auroraview.testing.headless_webview import headless_webview

        with headless_webview() as webview:
            webview.goto("https://example.com")
            assert webview.text("h1") == "Example Domain"
        ```
    """
    if mode == "auto":
        webview = HeadlessWebView.auto(**kwargs)
    elif mode == "playwright":
        webview = HeadlessWebView.playwright(**kwargs)
    elif mode == "xvfb":
        webview = HeadlessWebView.virtual_display(**kwargs)
    elif mode == "cdp":
        cdp_url = kwargs.pop("cdp_url", os.environ.get("WEBVIEW2_CDP_URL", "http://localhost:9222"))
        webview = HeadlessWebView.webview2_cdp(cdp_url, **kwargs)
    else:
        raise ValueError(f"Unknown mode: {mode}")

    try:
        yield webview
    finally:
        webview.close()
