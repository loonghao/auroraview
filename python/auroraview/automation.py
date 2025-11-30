"""Automation abstraction layer for Steel Browser compatibility.

This module provides a unified interface for browser automation that works
with both local AuroraView WebViews and remote Steel Browser instances.

The goal is to allow developers to write automation code once and run it
against either backend, enabling:

1. Local development: Use AuroraView's embedded WebView for fast iteration
2. Production/CI: Connect to Steel Browser for headless automation
3. DCC integration: Use AuroraView in Maya/Houdini with same automation API

Example:
    >>> # Local automation with AuroraView
    >>> from auroraview import WebView
    >>> from auroraview.automation import Automation
    >>>
    >>> webview = WebView.create("My App", url="http://localhost:3000")
    >>> auto = Automation.local(webview)
    >>> auto.dom("#login").type_text("admin")
    >>> auto.dom("#submit").click()

    >>> # Remote automation with Steel Browser (future)
    >>> auto = Automation.steel("http://steel.example.com:3000")
    >>> auto.scrape("https://example.com")
"""

from __future__ import annotations

import logging
import sys
from typing import TYPE_CHECKING, Any, Dict, Optional

# Protocol and runtime_checkable are available in typing from Python 3.8+
if sys.version_info >= (3, 8):
    from typing import Protocol, runtime_checkable
else:
    from typing_extensions import Protocol, runtime_checkable

if TYPE_CHECKING:
    from .dom import Element, ElementCollection
    from .webview import WebView

logger = logging.getLogger(__name__)

__all__ = [
    "BrowserBackend",
    "LocalWebViewBackend",
    "SteelBrowserBackend",
    "Automation",
]


@runtime_checkable
class BrowserBackend(Protocol):
    """Protocol for browser automation backends.

    This protocol defines the interface that both local WebView and
    remote Steel Browser backends must implement.
    """

    def dom(self, selector: str) -> "Element":
        """Get DOM element by CSS selector.

        Args:
            selector: CSS selector for the element.

        Returns:
            Element wrapper for DOM manipulation.
        """
        ...

    def dom_all(self, selector: str) -> "ElementCollection":
        """Get all DOM elements matching selector.

        Args:
            selector: CSS selector for elements.

        Returns:
            ElementCollection for batch operations.
        """
        ...

    def scrape(self, url: Optional[str] = None, **options: Any) -> Dict[str, Any]:
        """Scrape page content.

        Args:
            url: URL to scrape (uses current page if None).
            **options: Additional scraping options.

        Returns:
            Dict with html, text, url, title, etc.
        """
        ...

    def screenshot(self, **options: Any) -> bytes:
        """Take screenshot.

        Args:
            **options: Screenshot options (format, quality, etc.)

        Returns:
            Screenshot image data as bytes.
        """
        ...

    def pdf(self, **options: Any) -> bytes:
        """Generate PDF of page.

        Args:
            **options: PDF generation options.

        Returns:
            PDF data as bytes.
        """
        ...


class LocalWebViewBackend:
    """AuroraView local WebView backend.

    Implements the BrowserBackend protocol using a local WebView instance.
    This is the default backend for DCC integration scenarios.
    """

    def __init__(self, webview: "WebView") -> None:
        """Initialize with a WebView instance.

        Args:
            webview: AuroraView WebView instance.
        """
        self._webview = webview

    def dom(self, selector: str) -> "Element":
        """Get DOM element by CSS selector."""
        return self._webview.dom(selector)

    def dom_all(self, selector: str) -> "ElementCollection":
        """Get all DOM elements matching selector."""
        return self._webview.dom_all(selector)

    def scrape(self, url: Optional[str] = None, **options: Any) -> Dict[str, Any]:
        """Scrape current page content.

        Note: URL parameter is ignored for local backend.
        Use webview.load_url() to navigate first.
        """
        # Execute JavaScript to gather page content
        # Results are stored in window.__auroraview_result
        self._webview.eval_js(
            "window.__auroraview_scrape_result = {"
            "  html: document.documentElement.outerHTML,"
            "  text: document.body.innerText,"
            "  url: window.location.href,"
            "  title: document.title"
            "};"
        )
        # Note: Actual result retrieval requires bridge integration
        return {"status": "pending", "message": "Result available via bridge"}

    def screenshot(self, **options: Any) -> bytes:
        """Take screenshot of WebView.

        Note: Screenshot functionality requires platform-specific implementation.
        """
        raise NotImplementedError("Screenshot not yet implemented for local backend")

    def pdf(self, **options: Any) -> bytes:
        """Generate PDF of page.

        Note: PDF generation requires platform-specific implementation.
        """
        raise NotImplementedError("PDF generation not yet implemented for local backend")


class SteelBrowserBackend:
    """Steel Browser remote backend (placeholder for future integration).

    This class will implement the BrowserBackend protocol using
    Steel Browser's HTTP API for remote browser automation.

    Steel Browser API endpoints:
    - POST /v1/scrape - Scrape page content
    - POST /v1/screenshot - Take screenshot
    - POST /v1/pdf - Generate PDF
    - POST /v1/sessions - Create browser session

    See: https://github.com/steel-dev/steel-browser
    """

    def __init__(
        self,
        base_url: str = "http://localhost:3000",
        api_key: Optional[str] = None,
    ) -> None:
        """Initialize Steel Browser backend.

        Args:
            base_url: Steel Browser API URL.
            api_key: Optional API key for authentication.
        """
        self._base_url = base_url.rstrip("/")
        self._api_key = api_key
        self._session_id: Optional[str] = None

    def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Make HTTP request to Steel API.

        Note: Requires httpx or requests library.
        """
        raise NotImplementedError(
            "Steel Browser backend requires httpx library. Install with: pip install httpx"
        )

    def dom(self, selector: str) -> "Element":
        """Get DOM element (requires active session)."""
        raise NotImplementedError(
            "DOM manipulation via Steel requires an active session. "
            "Use Playwright/Puppeteer integration instead."
        )

    def dom_all(self, selector: str) -> "ElementCollection":
        """Get all DOM elements (requires active session)."""
        raise NotImplementedError(
            "DOM manipulation via Steel requires an active session. "
            "Use Playwright/Puppeteer integration instead."
        )

    def scrape(self, url: Optional[str] = None, **options: Any) -> Dict[str, Any]:
        """Scrape page content via Steel API.

        Args:
            url: URL to scrape.
            **options: Additional options (wait_for, timeout, etc.)
        """
        # POST /v1/scrape
        return self._request("POST", "/v1/scrape", {"url": url, **options})

    def screenshot(self, url: Optional[str] = None, **options: Any) -> bytes:
        """Take screenshot via Steel API.

        Args:
            url: URL to screenshot.
            **options: Options (full_page, format, etc.)
        """
        # POST /v1/screenshot
        result = self._request("POST", "/v1/screenshot", {"url": url, **options})
        # Return base64 decoded data
        import base64

        return base64.b64decode(result.get("data", ""))

    def pdf(self, url: Optional[str] = None, **options: Any) -> bytes:
        """Generate PDF via Steel API.

        Args:
            url: URL to convert to PDF.
            **options: PDF options (format, margin, etc.)
        """
        # POST /v1/pdf
        result = self._request("POST", "/v1/pdf", {"url": url, **options})
        import base64

        return base64.b64decode(result.get("data", ""))


class Automation:
    """Unified automation interface for local and remote backends.

    This class provides a consistent API for browser automation that works
    with both AuroraView (local) and Steel Browser (remote) backends.

    Example:
        >>> # Local automation
        >>> webview = WebView.create("App", url="http://localhost:3000")
        >>> auto = Automation.local(webview)
        >>> auto.dom("#button").click()

        >>> # Remote automation (Steel Browser)
        >>> auto = Automation.steel("http://steel.company.com:3000")
        >>> result = auto.scrape("https://example.com")
    """

    def __init__(self, backend: BrowserBackend) -> None:
        """Initialize with a backend.

        Args:
            backend: Browser backend instance.
        """
        self._backend = backend

    @classmethod
    def local(cls, webview: "WebView") -> "Automation":
        """Create automation with local WebView backend.

        Args:
            webview: AuroraView WebView instance.

        Returns:
            Automation instance with local backend.
        """
        return cls(LocalWebViewBackend(webview))

    @classmethod
    def steel(
        cls,
        base_url: str = "http://localhost:3000",
        api_key: Optional[str] = None,
    ) -> "Automation":
        """Create automation with Steel Browser backend.

        Args:
            base_url: Steel Browser API URL.
            api_key: Optional API key.

        Returns:
            Automation instance with Steel backend.
        """
        return cls(SteelBrowserBackend(base_url, api_key))

    def dom(self, selector: str) -> "Element":
        """Get DOM element by CSS selector.

        Args:
            selector: CSS selector for the element.

        Returns:
            Element wrapper for DOM manipulation.
        """
        return self._backend.dom(selector)

    def dom_all(self, selector: str) -> "ElementCollection":
        """Get all DOM elements matching selector.

        Args:
            selector: CSS selector for elements.

        Returns:
            ElementCollection for batch operations.
        """
        return self._backend.dom_all(selector)

    def scrape(self, url: Optional[str] = None, **options: Any) -> Dict[str, Any]:
        """Scrape page content.

        Args:
            url: URL to scrape (uses current page if None).
            **options: Additional scraping options.

        Returns:
            Dict with html, text, url, title, etc.
        """
        return self._backend.scrape(url, **options)

    def screenshot(self, **options: Any) -> bytes:
        """Take screenshot.

        Args:
            **options: Screenshot options.

        Returns:
            Screenshot image data.
        """
        return self._backend.screenshot(**options)

    def pdf(self, **options: Any) -> bytes:
        """Generate PDF.

        Args:
            **options: PDF options.

        Returns:
            PDF data.
        """
        return self._backend.pdf(**options)
