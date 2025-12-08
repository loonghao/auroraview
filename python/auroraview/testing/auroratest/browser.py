"""
Browser and BrowserContext classes for AuroraTest.

Browser manages the WebView2 instance and provides page creation.
BrowserContext provides isolated browser sessions.

Architecture (inspired by Playwright):
- Browser runs WebView in a background thread
- All communication happens through WebViewProxy (thread-safe message queue)
- Page operations are async and use the proxy for cross-thread safety
"""

from __future__ import annotations

import logging
import queue
import threading
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Dict, List, Optional

if TYPE_CHECKING:
    from .page import Page

logger = logging.getLogger(__name__)


@dataclass
class BrowserOptions:
    """Options for browser launch."""

    headless: bool = True
    """Run browser in headless mode."""

    devtools: bool = False
    """Open DevTools automatically."""

    slow_mo: float = 0
    """Slow down operations by specified milliseconds."""

    timeout: float = 30000
    """Default timeout in milliseconds."""

    viewport: Optional[Dict[str, int]] = None
    """Default viewport size: {"width": 1280, "height": 720}."""

    user_data_dir: Optional[str] = None
    """Path to user data directory."""

    args: List[str] = field(default_factory=list)
    """Additional browser arguments."""

    ignore_https_errors: bool = False
    """Ignore HTTPS errors."""

    proxy: Optional[Dict[str, str]] = None
    """Proxy settings: {"server": "http://proxy:8080"}."""


class Browser:
    """
    Browser instance that manages WebView2 and provides page creation.

    Architecture:
    - WebView runs in a dedicated background thread
    - Communication uses WebViewProxy (thread-safe message queue)
    - All Page operations go through the proxy

    Example:
        ```python
        browser = Browser.launch(headless=True)
        page = browser.new_page()
        await page.goto("https://example.com")
        browser.close()
        ```
    """

    def __init__(self, options: BrowserOptions):
        """Initialize browser with options."""
        self._options = options
        self._contexts: List[BrowserContext] = []
        self._pages: List["Page"] = []
        self._closed = False

        # WebView instance (created in background thread)
        self._webview = None

        # Thread-safe proxy for cross-thread communication
        self._proxy = None

        # Background thread running WebView event loop
        self._thread: Optional[threading.Thread] = None

        # Synchronization primitives
        self._ready_event = threading.Event()
        self._error_queue: queue.Queue = queue.Queue()

    @classmethod
    def launch(
        cls,
        headless: bool = True,
        devtools: bool = False,
        slow_mo: float = 0,
        timeout: float = 30000,
        viewport: Optional[Dict[str, int]] = None,
        args: Optional[List[str]] = None,
        **kwargs,
    ) -> "Browser":
        """
        Launch a new browser instance.

        Args:
            headless: Run in headless mode (no visible window).
            devtools: Open DevTools automatically.
            slow_mo: Slow down operations by milliseconds.
            timeout: Default timeout in milliseconds.
            viewport: Default viewport size.
            args: Additional browser arguments.

        Returns:
            Browser instance.

        Example:
            ```python
            browser = Browser.launch(headless=True)
            ```
        """
        options = BrowserOptions(
            headless=headless,
            devtools=devtools,
            slow_mo=slow_mo,
            timeout=timeout,
            viewport=viewport or {"width": 1280, "height": 720},
            args=args or [],
            **kwargs,
        )

        browser = cls(options)
        browser._start()
        return browser

    def _start(self):
        """Start the browser (WebView2) in a background thread."""
        logger.info(f"Starting browser (headless={self._options.headless})")

        # Start WebView in background thread
        self._thread = threading.Thread(target=self._run_webview_thread, daemon=True)
        self._thread.start()

        # Wait for WebView to be ready (proxy available)
        timeout_sec = 15
        if not self._ready_event.wait(timeout=timeout_sec):
            # Check for errors
            try:
                error = self._error_queue.get_nowait()
                raise RuntimeError(f"Browser failed to start: {error}") from None
            except queue.Empty:
                raise RuntimeError(
                    f"Browser failed to start within {timeout_sec} seconds"
                ) from None

        # Check for startup errors
        try:
            error = self._error_queue.get_nowait()
            raise RuntimeError(f"Browser startup error: {error}")
        except queue.Empty:
            pass

        logger.info("Browser started successfully")

    def _run_webview_thread(self):
        """Run WebView in background thread (Playwright-style architecture)."""
        try:
            from auroraview._core import WebView as CoreWebView

            logger.info(
                "[Browser Thread] Creating WebView instance (headless=%s)", self._options.headless
            )

            viewport = self._options.viewport or {"width": 1280, "height": 720}

            # Create WebView in this thread (required by WebView2)
            # Pass headless=True to keep window hidden for automated testing
            webview = CoreWebView(
                title="AuroraTest Browser",
                width=viewport["width"],
                height=viewport["height"],
                dev_tools=self._options.devtools,
                decorations=not self._options.headless,
                resizable=True,
                headless=self._options.headless,  # Key: pass headless to Rust layer
            )

            # Store WebView reference
            self._webview = webview

            # Get thread-safe proxy for cross-thread communication
            # This is the key to Playwright-style architecture
            self._proxy = webview.get_proxy()
            logger.info("[Browser Thread] WebViewProxy obtained")

            # Signal that browser is ready
            self._ready_event.set()

            # Run the event loop (blocking)
            logger.info("[Browser Thread] Starting event loop")
            webview.show()
            logger.info("[Browser Thread] Event loop exited")

        except Exception as e:
            logger.error(f"[Browser Thread] Error: {e}", exc_info=True)
            self._error_queue.put(str(e))
            self._ready_event.set()  # Unblock the main thread

    def new_context(self, **kwargs) -> "BrowserContext":
        """
        Create a new browser context (isolated session).

        Args:
            **kwargs: Context options.

        Returns:
            BrowserContext instance.
        """
        context = BrowserContext(self, **kwargs)
        self._contexts.append(context)
        return context

    def new_page(self, **kwargs) -> "Page":
        """
        Create a new page in the default context.

        Args:
            **kwargs: Page options.

        Returns:
            Page instance.

        Example:
            ```python
            page = browser.new_page()
            await page.goto("https://example.com")
            ```
        """
        from .page import Page

        if self._proxy is None:
            raise RuntimeError("Browser not started. Call Browser.launch() first.")

        page = Page(self, self._proxy, **kwargs)
        self._pages.append(page)
        return page

    @property
    def proxy(self):
        """Get the thread-safe WebViewProxy."""
        return self._proxy

    @property
    def contexts(self) -> List["BrowserContext"]:
        """Get all browser contexts."""
        return self._contexts.copy()

    @property
    def pages(self) -> List["Page"]:
        """Get all pages across all contexts."""
        all_pages = self._pages.copy()
        for context in self._contexts:
            all_pages.extend(context.pages)
        return all_pages

    def close(self):
        """
        Close the browser and all pages.

        Example:
            ```python
            browser.close()
            ```
        """
        if self._closed:
            return

        logger.info("Closing browser")
        self._closed = True

        # Close all contexts
        for context in self._contexts:
            context.close()
        self._contexts.clear()

        # Close all pages
        for page in self._pages:
            page.close()
        self._pages.clear()

        # Send close message via proxy to properly shut down the event loop
        if self._proxy is not None:
            try:
                logger.info("Sending close message via proxy")
                self._proxy.close()
            except Exception as e:
                logger.warning(f"Error sending close message: {e}")

        # Wait for thread to finish
        if self._thread and self._thread.is_alive():
            logger.info("Waiting for browser thread to exit...")
            self._thread.join(timeout=5)
            if self._thread.is_alive():
                logger.warning("Browser thread did not exit cleanly")
            else:
                logger.info("Browser thread exited")

        logger.info("Browser closed")

    def __enter__(self) -> "Browser":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False


class BrowserContext:
    """
    Isolated browser context with separate cookies and storage.

    Example:
        ```python
        context = browser.new_context()
        page = context.new_page()
        # ... use page
        context.close()
        ```
    """

    def __init__(self, browser: Browser, **kwargs):
        """Initialize context."""
        self._browser = browser
        self._pages: List["Page"] = []
        self._closed = False
        self._options = kwargs

    def new_page(self, **kwargs) -> "Page":
        """
        Create a new page in this context.

        Returns:
            Page instance.
        """
        from .page import Page

        if self._browser._proxy is None:
            raise RuntimeError("Browser not started.")

        merged_options = {**self._options, **kwargs}
        page = Page(self._browser, self._browser._proxy, **merged_options)
        self._pages.append(page)
        return page

    @property
    def pages(self) -> List["Page"]:
        """Get all pages in this context."""
        return self._pages.copy()

    @property
    def browser(self) -> Browser:
        """Get the browser instance."""
        return self._browser

    async def add_cookies(self, cookies: List[Dict[str, Any]]):
        """Add cookies to this context."""
        # TODO: Implement via WebView2 cookie API
        pass

    async def clear_cookies(self):
        """Clear all cookies in this context."""
        # TODO: Implement via WebView2 cookie API
        pass

    async def storage_state(self, path: Optional[str] = None) -> Dict[str, Any]:
        """
        Get storage state (cookies, localStorage).

        Args:
            path: Optional path to save state as JSON.

        Returns:
            Storage state dict.
        """
        # TODO: Implement
        return {"cookies": [], "origins": []}

    def close(self):
        """Close this context and all its pages."""
        if self._closed:
            return

        self._closed = True

        for page in self._pages:
            page.close()
        self._pages.clear()

    def __enter__(self) -> "BrowserContext":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False
