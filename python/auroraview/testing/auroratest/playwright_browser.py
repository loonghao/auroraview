"""
PlaywrightBrowser - Playwright-based testing for AuroraView.

Two modes of operation:
1. Native mode (default): Uses Playwright's built-in Chromium browser
   - Fast and reliable for testing frontend code
   - Injects AuroraView bridge API for compatibility testing

2. WebView2 mode: Connects to WebView2 via CDP (requires separate process)
   - For testing actual WebView2 integration
   - Requires WebView2 to be started in a separate process with --remote-debugging-port

Example (Native mode - recommended for most tests):
    ```python
    from auroraview.testing.auroratest import PlaywrightBrowser

    browser = PlaywrightBrowser.launch(headless=True)
    page = browser.new_page()

    page.goto("https://example.com")
    page.locator("#button").click()

    browser.close()
    ```

Example (WebView2 mode - for integration tests):
    ```python
    # Start WebView2 in separate process first with:
    # WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222

    browser = PlaywrightBrowser.connect_cdp("http://localhost:9222")
    page = browser.new_page()
    # ...
    ```
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import TYPE_CHECKING, Dict, Optional

if TYPE_CHECKING:
    from playwright.sync_api import Browser as PWBrowser
    from playwright.sync_api import BrowserContext as PWContext
    from playwright.sync_api import Page as PWPage
    from playwright.sync_api import Playwright

logger = logging.getLogger(__name__)

# AuroraView bridge script to inject into pages
AURORAVIEW_BRIDGE_SCRIPT = """
(function() {
    if (window.auroraview) return;

    const eventHandlers = {};
    let callId = 0;
    const pendingCalls = {};

    window.auroraview = {
        // Call Python method (mock - returns immediately in test mode)
        call: function(method, params) {
            return new Promise((resolve, reject) => {
                const id = ++callId;
                // In test mode, we just log and resolve
                console.log('[AuroraView Test] call:', method, params);
                resolve(undefined);
            });
        },

        // Subscribe to events
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

        // Unsubscribe from events
        off: function(event, handler) {
            if (eventHandlers[event]) {
                const idx = eventHandlers[event].indexOf(handler);
                if (idx >= 0) eventHandlers[event].splice(idx, 1);
            }
        },

        // Trigger event (for testing)
        trigger: function(event, data) {
            if (eventHandlers[event]) {
                eventHandlers[event].forEach(h => h(data));
            }
        },

        // API proxy (pywebview style)
        api: new Proxy({}, {
            get: function(target, prop) {
                return function(...args) {
                    return window.auroraview.call('api.' + prop, args);
                };
            }
        }),

        // Test mode flag
        _testMode: true,
        _platform: 'playwright-test'
    };

    // Dispatch ready event
    window.dispatchEvent(new CustomEvent('auroraviewready'));
    console.log('[AuroraView Test] Bridge initialized');
})();
"""


@dataclass
class PlaywrightBrowserOptions:
    """Options for PlaywrightBrowser launch."""

    headless: bool = True
    """Run browser in headless mode (hidden window)."""

    devtools: bool = False
    """Open DevTools automatically."""

    slow_mo: float = 0
    """Slow down operations by specified milliseconds."""

    timeout: float = 30000
    """Default timeout in milliseconds."""

    viewport: Optional[Dict[str, int]] = None
    """Default viewport size: {"width": 1280, "height": 720}."""

    inject_bridge: bool = True
    """Inject AuroraView bridge script into pages."""


class PlaywrightBrowser:
    """
    Browser for testing AuroraView applications using Playwright.

    This uses Playwright's native Chromium browser for fast, reliable testing.
    The AuroraView bridge API is injected into pages for compatibility testing.

    For actual WebView2 integration testing, use `connect_cdp()` method
    with a WebView2 instance running in a separate process.

    Example:
        ```python
        browser = PlaywrightBrowser.launch(headless=True)
        page = browser.new_page()

        page.goto("file:///path/to/app.html")
        page.locator("#button").click()

        # Test AuroraView API
        result = page.evaluate("window.auroraview._testMode")
        assert result == True

        browser.close()
        ```
    """

    def __init__(self, options: PlaywrightBrowserOptions):
        """Initialize browser with options."""
        self._options = options
        self._closed = False

        # Playwright instances
        self._playwright: Optional[Playwright] = None
        self._pw_browser: Optional[PWBrowser] = None
        self._pw_context: Optional[PWContext] = None

    @classmethod
    def launch(
        cls,
        headless: bool = True,
        devtools: bool = False,
        slow_mo: float = 0,
        timeout: float = 30000,
        viewport: Optional[Dict[str, int]] = None,
        inject_bridge: bool = True,
        **kwargs,
    ) -> "PlaywrightBrowser":
        """
        Launch a new browser instance using Playwright's Chromium.

        Args:
            headless: Run in headless mode (hidden window).
            devtools: Open DevTools automatically.
            slow_mo: Slow down operations by milliseconds.
            timeout: Default timeout in milliseconds.
            viewport: Default viewport size.
            inject_bridge: Inject AuroraView bridge script.

        Returns:
            PlaywrightBrowser instance with Playwright API.

        Example:
            ```python
            browser = PlaywrightBrowser.launch(headless=True)
            page = browser.new_page()
            page.goto("https://example.com")
            ```
        """
        options = PlaywrightBrowserOptions(
            headless=headless,
            devtools=devtools,
            slow_mo=slow_mo,
            timeout=timeout,
            viewport=viewport or {"width": 1280, "height": 720},
            inject_bridge=inject_bridge,
        )

        browser = cls(options)
        browser._start_native()
        return browser

    @classmethod
    def connect_cdp(
        cls,
        cdp_url: str,
        timeout: float = 30000,
        **kwargs,
    ) -> "PlaywrightBrowser":
        """
        Connect to an existing browser via CDP (Chrome DevTools Protocol).

        Use this to connect to a WebView2 instance running with
        --remote-debugging-port enabled.

        Args:
            cdp_url: CDP endpoint URL (e.g., "http://localhost:9222")
            timeout: Connection timeout in milliseconds.

        Returns:
            PlaywrightBrowser instance connected via CDP.

        Example:
            ```python
            # WebView2 must be started separately with:
            # WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222

            browser = PlaywrightBrowser.connect_cdp("http://localhost:9222")
            page = browser.new_page()
            ```
        """
        options = PlaywrightBrowserOptions(
            headless=True,
            timeout=timeout,
            inject_bridge=False,  # WebView2 has its own bridge
        )

        browser = cls(options)
        browser._connect_cdp(cdp_url)
        return browser

    def _start_native(self):
        """Start native Chromium browser via Playwright."""
        try:
            from playwright.sync_api import sync_playwright

            logger.info("Starting Playwright Chromium browser")

            self._playwright = sync_playwright().start()

            # Launch Chromium
            self._pw_browser = self._playwright.chromium.launch(
                headless=self._options.headless,
                devtools=self._options.devtools,
                slow_mo=self._options.slow_mo,
            )

            # Create context with viewport
            viewport = self._options.viewport or {"width": 1280, "height": 720}
            self._pw_context = self._pw_browser.new_context(
                viewport=viewport,
            )

            # Inject AuroraView bridge script into all pages
            if self._options.inject_bridge:
                self._pw_context.add_init_script(AURORAVIEW_BRIDGE_SCRIPT)
                logger.info("AuroraView bridge script will be injected into pages")

            logger.info("Playwright browser started successfully")

        except Exception as e:
            logger.error(f"Failed to start Playwright: {e}", exc_info=True)
            raise RuntimeError(f"Playwright launch failed: {e}") from e

    def _connect_cdp(self, cdp_url: str):
        """Connect to browser via CDP."""
        try:
            from playwright.sync_api import sync_playwright

            logger.info(f"Connecting to CDP endpoint: {cdp_url}")

            self._playwright = sync_playwright().start()

            # Connect via CDP
            self._pw_browser = self._playwright.chromium.connect_over_cdp(cdp_url)

            # Get existing context or create new one
            if self._pw_browser.contexts:
                self._pw_context = self._pw_browser.contexts[0]
            else:
                self._pw_context = self._pw_browser.new_context()

            logger.info("Connected to browser via CDP")

        except Exception as e:
            logger.error(f"Failed to connect via CDP: {e}", exc_info=True)
            raise RuntimeError(f"CDP connection failed: {e}") from e

    def new_page(self) -> "PWPage":
        """
        Create a new page.

        Returns:
            Playwright Page instance with full API.

        Example:
            ```python
            page = browser.new_page()
            page.goto("https://example.com")
            page.locator("#button").click()
            ```
        """
        if self._pw_context is None:
            raise RuntimeError("Browser not started. Call launch() first.")

        return self._pw_context.new_page()

    def new_context(self, **kwargs) -> "PWContext":
        """
        Create a new browser context.

        Args:
            **kwargs: Playwright context options.

        Returns:
            Playwright BrowserContext instance.
        """
        if self._pw_browser is None:
            raise RuntimeError("Browser not started.")

        context = self._pw_browser.new_context(**kwargs)

        # Inject bridge script if enabled
        if self._options.inject_bridge:
            context.add_init_script(AURORAVIEW_BRIDGE_SCRIPT)

        return context

    @property
    def playwright(self) -> "Playwright":
        """Get the Playwright instance."""
        if self._playwright is None:
            raise RuntimeError("Playwright not initialized.")
        return self._playwright

    @property
    def browser(self) -> "PWBrowser":
        """Get the Playwright Browser instance."""
        if self._pw_browser is None:
            raise RuntimeError("Browser not started.")
        return self._pw_browser

    @property
    def context(self) -> "PWContext":
        """Get the default Playwright BrowserContext."""
        if self._pw_context is None:
            raise RuntimeError("Browser not started.")
        return self._pw_context

    def close(self):
        """Close the browser and cleanup resources."""
        if self._closed:
            return

        logger.info("Closing PlaywrightBrowser")
        self._closed = True

        try:
            if self._pw_context:
                self._pw_context.close()
            if self._pw_browser:
                self._pw_browser.close()
            if self._playwright:
                self._playwright.stop()
        except Exception as e:
            logger.warning(f"Error closing Playwright: {e}")

        logger.info("PlaywrightBrowser closed")

    def __enter__(self) -> "PlaywrightBrowser":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False
