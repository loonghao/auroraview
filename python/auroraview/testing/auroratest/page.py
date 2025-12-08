"""
Page class for AuroraTest.

Page represents a single WebView page and provides navigation,
interaction, and assertion methods.
"""

from __future__ import annotations

import asyncio
import logging
import time
from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, Pattern, Union

if TYPE_CHECKING:
    from auroraview import WebView

    from .browser import Browser
    from .locator import Locator
    from .network import Response, Route

logger = logging.getLogger(__name__)


class Page:
    """
    Page instance representing a WebView page.

    Provides Playwright-compatible API for navigation, interaction,
    screenshots, and assertions.

    Example:
        ```python
        page = browser.new_page()
        await page.goto("https://example.com")
        await page.locator("#search").fill("hello")
        await page.screenshot(path="screenshot.png")
        ```
    """

    def __init__(
        self,
        browser: "Browser",
        webview: "WebView",
        viewport: Optional[Dict[str, int]] = None,
        **kwargs
    ):
        """Initialize page."""
        self._browser = browser
        self._webview = webview
        self._viewport = viewport or {"width": 1280, "height": 720}
        self._closed = False
        self._routes: List[tuple] = []  # (pattern, handler)
        self._timeout = kwargs.get("timeout", 30000)

    @property
    def url(self) -> str:
        """Get current page URL."""
        # TODO: Get actual URL from WebView
        return ""

    # ========== Navigation ==========

    async def goto(
        self,
        url: str,
        timeout: Optional[float] = None,
        wait_until: str = "load"
    ) -> Optional["Response"]:
        """
        Navigate to a URL.

        Args:
            url: URL to navigate to.
            timeout: Navigation timeout in milliseconds.
            wait_until: When to consider navigation complete:
                - "load": Wait for load event
                - "domcontentloaded": Wait for DOMContentLoaded
                - "networkidle": Wait for network to be idle

        Returns:
            Response object or None.

        Example:
            ```python
            await page.goto("https://example.com")
            await page.goto("https://example.com", wait_until="networkidle")
            ```
        """
        timeout = timeout or self._timeout
        logger.info(f"Navigating to: {url}")

        self._webview.load_url(url)

        # Wait for page to load
        await self._wait_for_load_state(wait_until, timeout)

        return None  # TODO: Return Response object

    async def reload(
        self,
        timeout: Optional[float] = None,
        wait_until: str = "load"
    ) -> Optional["Response"]:
        """
        Reload the page.

        Args:
            timeout: Reload timeout in milliseconds.
            wait_until: When to consider reload complete.

        Returns:
            Response object or None.
        """
        timeout = timeout or self._timeout
        logger.info("Reloading page")

        self._webview.eval_js("window.location.reload()")
        await self._wait_for_load_state(wait_until, timeout)

        return None

    async def go_back(
        self,
        timeout: Optional[float] = None,
        wait_until: str = "load"
    ) -> Optional["Response"]:
        """Navigate back in history."""
        self._webview.eval_js("window.history.back()")
        await self._wait_for_load_state(wait_until, timeout or self._timeout)
        return None

    async def go_forward(
        self,
        timeout: Optional[float] = None,
        wait_until: str = "load"
    ) -> Optional["Response"]:
        """Navigate forward in history."""
        self._webview.eval_js("window.history.forward()")
        await self._wait_for_load_state(wait_until, timeout or self._timeout)
        return None

    async def _wait_for_load_state(self, state: str, timeout: float):
        """Wait for page load state."""
        # Simple implementation - wait a bit for page to load
        # TODO: Implement proper load state detection
        await asyncio.sleep(0.5)

    # ========== Content ==========

    async def content(self) -> str:
        """
        Get full HTML content of the page.

        Returns:
            HTML content as string.
        """
        # TODO: Implement via eval_js_async when available
        return ""

    async def title(self) -> str:
        """
        Get page title.

        Returns:
            Page title.
        """
        # TODO: Implement via eval_js_async
        return ""

    async def set_content(
        self,
        html: str,
        timeout: Optional[float] = None,
        wait_until: str = "load"
    ):
        """
        Set page HTML content.

        Args:
            html: HTML content to set.
            timeout: Timeout in milliseconds.
            wait_until: When to consider content loaded.
        """
        self._webview.load_html(html)
        await self._wait_for_load_state(wait_until, timeout or self._timeout)

    # ========== Locators ==========

    def locator(self, selector: str) -> "Locator":
        """
        Create a locator for the given selector.

        Args:
            selector: CSS selector or XPath.

        Returns:
            Locator instance.

        Example:
            ```python
            await page.locator("#submit").click()
            await page.locator("input[name='email']").fill("test@example.com")
            ```
        """
        from .locator import Locator
        return Locator(self, selector)

    def get_by_role(
        self,
        role: str,
        name: Optional[str] = None,
        exact: bool = False,
        **kwargs
    ) -> "Locator":
        """
        Locate element by ARIA role.

        Args:
            role: ARIA role (button, textbox, link, etc.)
            name: Accessible name to match.
            exact: Exact match for name.

        Returns:
            Locator instance.

        Example:
            ```python
            await page.get_by_role("button", name="Submit").click()
            ```
        """
        from .locator import Locator

        # Build selector based on role
        if name:
            if exact:
                selector = f'[role="{role}"][aria-label="{name}"], {role}:has-text("{name}")'
            else:
                selector = f'[role="{role}"], {role}'
        else:
            selector = f'[role="{role}"], {role}'

        return Locator(self, selector, role=role, name=name, exact=exact)

    def get_by_text(
        self,
        text: str,
        exact: bool = False
    ) -> "Locator":
        """
        Locate element by text content.

        Args:
            text: Text to match.
            exact: Exact match.

        Returns:
            Locator instance.

        Example:
            ```python
            await page.get_by_text("Welcome").click()
            ```
        """
        from .locator import Locator
        return Locator(self, f'text="{text}"', text=text, exact=exact)

    def get_by_label(self, text: str, exact: bool = False) -> "Locator":
        """Locate element by associated label text."""
        from .locator import Locator
        return Locator(self, f'label:has-text("{text}") + input, [aria-label="{text}"]')

    def get_by_placeholder(self, text: str, exact: bool = False) -> "Locator":
        """Locate element by placeholder text."""
        from .locator import Locator
        return Locator(self, f'[placeholder="{text}"]')

    def get_by_test_id(self, test_id: str) -> "Locator":
        """
        Locate element by data-testid attribute.

        Args:
            test_id: Value of data-testid attribute.

        Returns:
            Locator instance.

        Example:
            ```python
            await page.get_by_test_id("submit-button").click()
            ```
        """
        from .locator import Locator
        return Locator(self, f'[data-testid="{test_id}"]')

    # ========== Actions ==========

    async def click(
        self,
        selector: str,
        timeout: Optional[float] = None,
        **kwargs
    ):
        """
        Click an element.

        Args:
            selector: Element selector.
            timeout: Timeout in milliseconds.
        """
        await self.locator(selector).click(timeout=timeout, **kwargs)

    async def fill(
        self,
        selector: str,
        value: str,
        timeout: Optional[float] = None,
        **kwargs
    ):
        """
        Fill an input element.

        Args:
            selector: Element selector.
            value: Value to fill.
            timeout: Timeout in milliseconds.
        """
        await self.locator(selector).fill(value, timeout=timeout, **kwargs)

    async def type(
        self,
        selector: str,
        text: str,
        delay: float = 0,
        timeout: Optional[float] = None,
        **kwargs
    ):
        """
        Type text into an element character by character.

        Args:
            selector: Element selector.
            text: Text to type.
            delay: Delay between keystrokes in milliseconds.
            timeout: Timeout in milliseconds.
        """
        await self.locator(selector).type(text, delay=delay, timeout=timeout, **kwargs)

    async def press(
        self,
        selector: str,
        key: str,
        timeout: Optional[float] = None,
        **kwargs
    ):
        """
        Press a key on an element.

        Args:
            selector: Element selector.
            key: Key to press (e.g., "Enter", "Tab", "Escape").
            timeout: Timeout in milliseconds.
        """
        await self.locator(selector).press(key, timeout=timeout, **kwargs)

    async def check(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Check a checkbox."""
        await self.locator(selector).check(timeout=timeout, **kwargs)

    async def uncheck(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Uncheck a checkbox."""
        await self.locator(selector).uncheck(timeout=timeout, **kwargs)

    async def select_option(
        self,
        selector: str,
        value: Union[str, List[str]],
        timeout: Optional[float] = None,
        **kwargs
    ):
        """Select option(s) in a select element."""
        await self.locator(selector).select_option(value, timeout=timeout, **kwargs)

    async def hover(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Hover over an element."""
        await self.locator(selector).hover(timeout=timeout, **kwargs)

    async def focus(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Focus an element."""
        await self.locator(selector).focus(timeout=timeout, **kwargs)

    # ========== Waiting ==========

    async def wait_for_selector(
        self,
        selector: str,
        state: str = "visible",
        timeout: Optional[float] = None
    ) -> "Locator":
        """
        Wait for a selector to match an element.

        Args:
            selector: CSS selector.
            state: State to wait for:
                - "attached": Element is in DOM
                - "detached": Element is not in DOM
                - "visible": Element is visible
                - "hidden": Element is hidden
            timeout: Timeout in milliseconds.

        Returns:
            Locator for the element.

        Example:
            ```python
            await page.wait_for_selector(".loading", state="hidden")
            await page.wait_for_selector(".content", state="visible")
            ```
        """
        timeout = timeout or self._timeout
        locator = self.locator(selector)

        start = time.time()
        timeout_sec = timeout / 1000

        while time.time() - start < timeout_sec:
            try:
                if state == "visible":
                    if await locator.is_visible():
                        return locator
                elif state == "hidden":
                    if not await locator.is_visible():
                        return locator
                elif state == "attached":
                    if await locator.count() > 0:
                        return locator
                elif state == "detached":
                    if await locator.count() == 0:
                        return locator
            except Exception:
                pass

            await asyncio.sleep(0.1)

        raise TimeoutError(f"Timeout waiting for selector '{selector}' to be {state}")

    async def wait_for_load_state(
        self,
        state: str = "load",
        timeout: Optional[float] = None
    ):
        """
        Wait for page load state.

        Args:
            state: State to wait for:
                - "load": Wait for load event
                - "domcontentloaded": Wait for DOMContentLoaded
                - "networkidle": Wait for network to be idle
            timeout: Timeout in milliseconds.
        """
        await self._wait_for_load_state(state, timeout or self._timeout)

    async def wait_for_url(
        self,
        url: Union[str, Pattern],
        timeout: Optional[float] = None
    ):
        """
        Wait for URL to match.

        Args:
            url: URL string or regex pattern.
            timeout: Timeout in milliseconds.
        """
        # TODO: Implement URL watching
        await asyncio.sleep(0.5)

    async def wait_for_timeout(self, timeout: float):
        """
        Wait for specified time.

        Args:
            timeout: Time to wait in milliseconds.
        """
        await asyncio.sleep(timeout / 1000)

    async def wait_for_function(
        self,
        expression: str,
        timeout: Optional[float] = None,
        polling: float = 100
    ) -> Any:
        """
        Wait for a JavaScript function to return truthy value.

        Args:
            expression: JavaScript expression to evaluate.
            timeout: Timeout in milliseconds.
            polling: Polling interval in milliseconds.

        Returns:
            Return value of the expression.
        """
        # TODO: Implement with eval_js_async
        await asyncio.sleep(0.5)
        return None

    # ========== Screenshots ==========

    async def screenshot(
        self,
        path: Optional[str] = None,
        full_page: bool = False,
        clip: Optional[Dict[str, int]] = None,
        type: str = "png",
        quality: Optional[int] = None,
        scale: str = "device"
    ) -> bytes:
        """
        Take a screenshot of the page.

        Args:
            path: Path to save screenshot. If None, returns bytes.
            full_page: Capture full scrollable page.
            clip: Clip region: {"x": 0, "y": 0, "width": 100, "height": 100}
            type: Image type: "png" or "jpeg"
            quality: JPEG quality (0-100).
            scale: Scale: "css" or "device"

        Returns:
            Screenshot as bytes.

        Example:
            ```python
            # Save to file
            await page.screenshot(path="screenshot.png")

            # Get bytes
            screenshot_bytes = await page.screenshot()

            # Full page
            await page.screenshot(path="full.png", full_page=True)

            # Clip region
            await page.screenshot(path="header.png", clip={"x": 0, "y": 0, "width": 800, "height": 100})
            ```
        """
        logger.info(f"Taking screenshot (full_page={full_page}, path={path})")

        # Use WebView2's CapturePreview if available
        # For now, use JavaScript-based screenshot
        screenshot_data = await self._capture_screenshot(full_page, clip, type, quality)

        if path:
            Path(path).parent.mkdir(parents=True, exist_ok=True)
            Path(path).write_bytes(screenshot_data)
            logger.info(f"Screenshot saved to: {path}")

        return screenshot_data

    async def _capture_screenshot(
        self,
        full_page: bool,
        clip: Optional[Dict[str, int]],
        image_type: str,
        quality: Optional[int]
    ) -> bytes:
        """Capture screenshot using available method."""
        # Try WebView2 native capture first
        if hasattr(self._webview, '_core') and hasattr(self._webview._core, 'capture_screenshot'):
            try:
                return self._webview._core.capture_screenshot(
                    full_page=full_page,
                    format=image_type,
                    quality=quality or 100
                )
            except Exception as e:
                logger.warning(f"Native screenshot failed: {e}, falling back to JS")

        # Fallback: Use html2canvas via JavaScript
        # This is a placeholder - actual implementation would inject html2canvas
        logger.warning("Screenshot capture not fully implemented yet")
        return b""

    # ========== JavaScript ==========

    async def evaluate(
        self,
        expression: str,
        arg: Any = None
    ) -> Any:
        """
        Evaluate JavaScript expression.

        Args:
            expression: JavaScript expression or function.
            arg: Argument to pass to the function.

        Returns:
            Result of evaluation.

        Example:
            ```python
            result = await page.evaluate("document.title")
            result = await page.evaluate("(x) => x * 2", 5)
            ```
        """
        # TODO: Implement with eval_js_async
        self._webview.eval_js(expression)
        return None

    async def evaluate_handle(
        self,
        expression: str,
        arg: Any = None
    ):
        """Evaluate JavaScript and return handle to result."""
        # TODO: Implement
        pass

    # ========== Network ==========

    async def route(
        self,
        url: Union[str, Pattern],
        handler: Callable[["Route"], Any]
    ):
        """
        Intercept network requests matching URL pattern.

        Args:
            url: URL pattern (string or regex).
            handler: Handler function that receives Route object.

        Example:
            ```python
            async def mock_api(route):
                await route.fulfill(
                    status=200,
                    content_type="application/json",
                    body='{"data": "mocked"}'
                )

            await page.route("**/api/data", mock_api)
            ```
        """
        self._routes.append((url, handler))
        # TODO: Implement actual network interception

    async def unroute(
        self,
        url: Union[str, Pattern],
        handler: Optional[Callable] = None
    ):
        """
        Remove route handler.

        Args:
            url: URL pattern to remove.
            handler: Specific handler to remove (optional).
        """
        self._routes = [
            (u, h) for u, h in self._routes
            if u != url or (handler and h != handler)
        ]

    # ========== Viewport ==========

    async def set_viewport_size(self, viewport: Dict[str, int]):
        """
        Set viewport size.

        Args:
            viewport: {"width": 1280, "height": 720}
        """
        self._viewport = viewport
        # TODO: Resize WebView

    @property
    def viewport_size(self) -> Dict[str, int]:
        """Get current viewport size."""
        return self._viewport.copy()

    # ========== Lifecycle ==========

    def close(self):
        """Close the page."""
        if self._closed:
            return
        self._closed = True
        logger.info("Page closed")

    async def bring_to_front(self):
        """Bring page to front (focus)."""
        # TODO: Implement
        pass

    def is_closed(self) -> bool:
        """Check if page is closed."""
        return self._closed
