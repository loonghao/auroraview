"""
Page class for AuroraTest.

Page represents a single WebView page and provides navigation,
interaction, and assertion methods.

Architecture (Playwright-style):
- Page uses WebViewProxy for all WebView operations
- All operations are async and thread-safe
- JavaScript execution uses callback-based async pattern
"""

from __future__ import annotations

import asyncio
import json
import logging
import time
from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, Pattern, Union

if TYPE_CHECKING:
    from .browser import Browser
    from .locator import Locator
    from .network import Response, Route

logger = logging.getLogger(__name__)


class Page:
    """
    Page instance representing a WebView page.

    Provides Playwright-compatible API for navigation, interaction,
    screenshots, and assertions.

    All operations use WebViewProxy for thread-safe cross-thread communication.

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
        proxy,  # WebViewProxy - thread-safe
        viewport: Optional[Dict[str, int]] = None,
        **kwargs,
    ):
        """Initialize page with WebViewProxy."""
        self._browser = browser
        self._proxy = proxy  # Thread-safe proxy for all operations
        self._viewport = viewport or {"width": 1280, "height": 720}
        self._closed = False
        self._routes: List[tuple] = []  # (pattern, handler)
        self._timeout = kwargs.get("timeout", 30000)

    @property
    def url(self) -> str:
        """Get current page URL."""
        # TODO: Implement URL tracking
        return ""

    # ========== Navigation ==========

    async def goto(
        self, url: str, timeout: Optional[float] = None, wait_until: str = "load"
    ) -> Optional["Response"]:
        """
        Navigate to a URL.

        Args:
            url: URL to navigate to.
            timeout: Navigation timeout in milliseconds.
            wait_until: When to consider navigation complete.

        Returns:
            Response object or None.
        """
        timeout = timeout or self._timeout
        logger.info(f"Navigating to: {url}")

        # Use proxy for thread-safe navigation
        self._proxy.load_url(url)

        # Wait for page to load
        await self._wait_for_load_state(wait_until, timeout)

        return None

    async def reload(
        self, timeout: Optional[float] = None, wait_until: str = "load"
    ) -> Optional["Response"]:
        """Reload the page."""
        timeout = timeout or self._timeout
        logger.info("Reloading page")

        self._proxy.reload()
        await self._wait_for_load_state(wait_until, timeout)

        return None

    async def go_back(
        self, timeout: Optional[float] = None, wait_until: str = "load"
    ) -> Optional["Response"]:
        """Navigate back in history."""
        self._proxy.eval_js("window.history.back()")
        await self._wait_for_load_state(wait_until, timeout or self._timeout)
        return None

    async def go_forward(
        self, timeout: Optional[float] = None, wait_until: str = "load"
    ) -> Optional["Response"]:
        """Navigate forward in history."""
        self._proxy.eval_js("window.history.forward()")
        await self._wait_for_load_state(wait_until, timeout or self._timeout)
        return None

    async def _wait_for_load_state(self, state: str, timeout: float):
        """Wait for page load state."""
        # Simple implementation - wait for content to be ready
        # TODO: Implement proper load state detection via events
        await asyncio.sleep(0.5)

    # ========== Content ==========

    async def content(self) -> str:
        """Get full HTML content of the page."""
        result = await self.evaluate("document.documentElement.outerHTML")
        return result or ""

    async def title(self) -> str:
        """Get page title."""
        result = await self.evaluate("document.title")
        return result or ""

    async def set_content(
        self, html: str, timeout: Optional[float] = None, wait_until: str = "load"
    ):
        """
        Set page HTML content.

        Args:
            html: HTML content to set.
            timeout: Timeout in milliseconds.
            wait_until: When to consider content loaded.
        """
        logger.info(f"Setting content ({len(html)} bytes)")

        # Use proxy for thread-safe HTML loading
        self._proxy.load_html(html)

        await self._wait_for_load_state(wait_until, timeout or self._timeout)

    # ========== Locators ==========

    def locator(self, selector: str) -> "Locator":
        """
        Create a locator for the given selector.

        Args:
            selector: CSS selector or XPath.

        Returns:
            Locator instance.
        """
        from .locator import Locator

        return Locator(self, selector)

    def get_by_role(
        self, role: str, name: Optional[str] = None, exact: bool = False, **kwargs
    ) -> "Locator":
        """Locate element by ARIA role."""
        from .locator import Locator

        if name:
            if exact:
                selector = f'[role="{role}"][aria-label="{name}"], {role}:has-text("{name}")'
            else:
                selector = f'[role="{role}"], {role}'
        else:
            selector = f'[role="{role}"], {role}'

        return Locator(self, selector, role=role, name=name, exact=exact)

    def get_by_text(self, text: str, exact: bool = False) -> "Locator":
        """Locate element by text content."""
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
        """Locate element by data-testid attribute."""
        from .locator import Locator

        return Locator(self, f'[data-testid="{test_id}"]')

    # ========== Actions ==========

    async def click(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Click an element."""
        await self.locator(selector).click(timeout=timeout, **kwargs)

    async def fill(self, selector: str, value: str, timeout: Optional[float] = None, **kwargs):
        """Fill an input element."""
        await self.locator(selector).fill(value, timeout=timeout, **kwargs)

    async def type(
        self, selector: str, text: str, delay: float = 0, timeout: Optional[float] = None, **kwargs
    ):
        """Type text into an element character by character."""
        await self.locator(selector).type(text, delay=delay, timeout=timeout, **kwargs)

    async def press(self, selector: str, key: str, timeout: Optional[float] = None, **kwargs):
        """Press a key on an element."""
        await self.locator(selector).press(key, timeout=timeout, **kwargs)

    async def check(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Check a checkbox."""
        await self.locator(selector).check(timeout=timeout, **kwargs)

    async def uncheck(self, selector: str, timeout: Optional[float] = None, **kwargs):
        """Uncheck a checkbox."""
        await self.locator(selector).uncheck(timeout=timeout, **kwargs)

    async def select_option(
        self, selector: str, value: Union[str, List[str]], timeout: Optional[float] = None, **kwargs
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
        self, selector: str, state: str = "visible", timeout: Optional[float] = None
    ) -> "Locator":
        """
        Wait for a selector to match an element.

        Args:
            selector: CSS selector.
            state: State to wait for (attached, detached, visible, hidden).
            timeout: Timeout in milliseconds.

        Returns:
            Locator for the element.
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

    async def wait_for_load_state(self, state: str = "load", timeout: Optional[float] = None):
        """Wait for page load state."""
        await self._wait_for_load_state(state, timeout or self._timeout)

    async def wait_for_url(self, url: Union[str, Pattern], timeout: Optional[float] = None):
        """Wait for URL to match."""
        # TODO: Implement URL watching
        await asyncio.sleep(0.5)

    async def wait_for_timeout(self, timeout: float):
        """Wait for specified time in milliseconds."""
        await asyncio.sleep(timeout / 1000)

    async def wait_for_function(
        self, expression: str, timeout: Optional[float] = None, polling: float = 100
    ) -> Any:
        """Wait for a JavaScript function to return truthy value."""
        timeout_sec = (timeout or self._timeout) / 1000
        polling_sec = polling / 1000
        start = time.time()

        while time.time() - start < timeout_sec:
            result = await self.evaluate(expression)
            if result:
                return result
            await asyncio.sleep(polling_sec)

        raise TimeoutError(f"Timeout waiting for function: {expression}")

    # ========== Screenshots ==========

    async def screenshot(
        self,
        path: Optional[str] = None,
        full_page: bool = False,
        clip: Optional[Dict[str, int]] = None,
        type: str = "png",
        quality: Optional[int] = None,
        scale: str = "device",
    ) -> bytes:
        """
        Take a screenshot of the page.

        Note: This uses html2canvas for client-side screenshot capture.
        WebView2 doesn't support native screenshot API.

        Args:
            path: Path to save screenshot.
            full_page: Capture full scrollable page.
            clip: Clip region.
            type: Image type (png or jpeg).
            quality: JPEG quality (0-100).
            scale: Scale (css or device).

        Returns:
            Screenshot as bytes.
        """
        logger.info(f"Taking screenshot (full_page={full_page}, path={path})")

        # Use html2canvas for screenshot capture
        import base64

        options = {
            "fullPage": full_page,
            "format": type,
            "quality": (quality or 92) / 100.0,
        }
        if clip:
            options["clip"] = clip

        # Execute screenshot capture via JavaScript
        js_code = f"""
        (function() {{
            return new Promise((resolve, reject) => {{
                if (window.auroraview && window.auroraview._screenshot) {{
                    window.auroraview._screenshot.capture({json.dumps(options)})
                        .then(result => resolve(result.data))
                        .catch(err => reject(err.message));
                }} else {{
                    reject('Screenshot module not loaded');
                }}
            }});
        }})()
        """

        try:
            result = await self.evaluate(js_code)
            if result:
                screenshot_data = base64.b64decode(result)
            else:
                screenshot_data = b""
        except Exception as e:
            logger.warning(f"Screenshot failed: {e}")
            screenshot_data = b""

        if path and screenshot_data:
            Path(path).parent.mkdir(parents=True, exist_ok=True)
            Path(path).write_bytes(screenshot_data)
            logger.info(f"Screenshot saved to: {path}")

        return screenshot_data

    # ========== JavaScript ==========

    async def evaluate(self, expression: str, arg: Any = None) -> Any:
        """
        Evaluate JavaScript expression and return result.

        Uses WebViewProxy.eval_js_async for cross-thread communication.

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
        # Build the script
        if arg is not None:
            arg_json = json.dumps(arg)
            if expression.startswith("(") or expression.startswith("function"):
                script = f"({expression})({arg_json})"
            else:
                script = f"(function(arg) {{ return {expression}; }})({arg_json})"
        else:
            script = expression

        # Wrap script to return JSON-serializable result
        wrapped_script = f"""
        (function() {{
            try {{
                var result = (function() {{ return {script}; }})();
                // Handle Promise results
                if (result && typeof result.then === 'function') {{
                    return result.then(function(value) {{
                        return JSON.stringify({{ok: true, value: value}});
                    }}).catch(function(err) {{
                        return JSON.stringify({{ok: false, error: err.message || String(err)}});
                    }});
                }}
                return JSON.stringify({{ok: true, value: result}});
            }} catch(e) {{
                return JSON.stringify({{ok: false, error: e.message || String(e)}});
            }}
        }})()
        """

        # Create future for async result
        loop = asyncio.get_event_loop()
        future = loop.create_future()

        def callback(result, error):
            """Handle JavaScript execution result."""
            if not future.done():
                if error:
                    future.set_result(json.dumps({"ok": False, "error": str(error)}))
                else:
                    future.set_result(result)

        try:
            # Execute via proxy's async method
            self._proxy.eval_js_async(wrapped_script, callback, int(self._timeout))

            # Wait for result with timeout
            timeout_sec = self._timeout / 1000
            try:
                result_json = await asyncio.wait_for(future, timeout=timeout_sec)

                if result_json:
                    result = json.loads(result_json)
                    if result.get("ok"):
                        return result.get("value")
                    else:
                        logger.warning(f"JavaScript error: {result.get('error')}")
                        return None
                return None

            except asyncio.TimeoutError:
                logger.warning(f"JavaScript evaluation timed out: {expression[:100]}...")
                return None

        except Exception as e:
            logger.warning(f"JavaScript evaluation failed: {e}")
            return None

    async def evaluate_handle(self, expression: str, arg: Any = None):
        """Evaluate JavaScript and return handle to result."""
        return await self.evaluate(expression, arg)

    # ========== Network ==========

    async def route(self, url: Union[str, Pattern], handler: Callable[["Route"], Any]):
        """
        Intercept network requests matching URL pattern.

        Args:
            url: URL pattern (string or regex).
            handler: Handler function that receives Route object.
        """
        self._routes.append((url, handler))

        pattern_str = url.pattern if isinstance(url, Pattern) else str(url)
        handler_id = f"route_{len(self._routes)}_{id(handler)}"

        # Register route in JavaScript
        js_code = f"""
        (function() {{
            if (window.auroraview && window.auroraview._network) {{
                window.auroraview._network.addRoute('{pattern_str}', function(ctx) {{
                    var payload = {{
                        type: 'route_request',
                        handler_id: '{handler_id}',
                        request: {{
                            url: ctx.request.url,
                            method: ctx.request.method,
                            headers: ctx.request.headers,
                            postData: ctx.request.postData
                        }}
                    }};
                    window.ipc.postMessage(JSON.stringify(payload));
                    return ctx.continue_();
                }});
            }}
        }})();
        """
        self._proxy.eval_js(js_code)

    async def unroute(self, url: Union[str, Pattern], handler: Optional[Callable] = None):
        """Remove route handler."""
        self._routes = [(u, h) for u, h in self._routes if u != url or (handler and h != handler)]

        pattern_str = url.pattern if isinstance(url, Pattern) else str(url)
        js_code = f"""
        (function() {{
            if (window.auroraview && window.auroraview._network) {{
                window.auroraview._network.removeRoute('{pattern_str}');
            }}
        }})();
        """
        self._proxy.eval_js(js_code)

    # ========== Viewport ==========

    async def set_viewport_size(self, viewport: Dict[str, int]):
        """Set viewport size."""
        self._viewport = viewport
        # TODO: Resize WebView via proxy

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
        pass

    def is_closed(self) -> bool:
        """Check if page is closed."""
        return self._closed
