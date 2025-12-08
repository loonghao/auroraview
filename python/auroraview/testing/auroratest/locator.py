"""
Locator class for AuroraTest.

Locator represents a way to find element(s) on the page.
It provides methods for interaction and assertions.
"""

from __future__ import annotations

import asyncio
import logging
import time
from typing import TYPE_CHECKING, Any, Dict, List, Optional, Union

if TYPE_CHECKING:
    from .page import Page

logger = logging.getLogger(__name__)


class Locator:
    """
    Locator for finding and interacting with elements.

    Locators are strict by default - they will throw if multiple elements match.
    Use .first, .last, or .nth(n) to work with multiple matches.

    Example:
        ```python
        # Basic locator
        await page.locator("#submit").click()

        # Chained locators
        await page.locator(".form").locator("input").fill("hello")

        # With filters
        await page.locator("button").filter(has_text="Submit").click()
        ```
    """

    def __init__(self, page: "Page", selector: str, **kwargs):
        """Initialize locator."""
        self._page = page
        self._selector = selector
        self._options = kwargs
        self._filters: List[Dict[str, Any]] = []

    @property
    def page(self) -> "Page":
        """Get the page this locator belongs to."""
        return self._page

    # ========== Chaining ==========

    def locator(self, selector: str) -> "Locator":
        """
        Create a child locator.

        Args:
            selector: Child selector.

        Returns:
            New Locator instance.
        """
        combined = f"{self._selector} {selector}"
        return Locator(self._page, combined, **self._options)

    def first(self) -> "Locator":
        """
        Get the first matching element.

        Returns:
            Locator for first element.
        """
        locator = Locator(self._page, self._selector, **self._options)
        locator._filters.append({"type": "first"})
        return locator

    def last(self) -> "Locator":
        """
        Get the last matching element.

        Returns:
            Locator for last element.
        """
        locator = Locator(self._page, self._selector, **self._options)
        locator._filters.append({"type": "last"})
        return locator

    def nth(self, index: int) -> "Locator":
        """
        Get the nth matching element (0-indexed).

        Args:
            index: Element index.

        Returns:
            Locator for nth element.
        """
        locator = Locator(self._page, self._selector, **self._options)
        locator._filters.append({"type": "nth", "index": index})
        return locator

    def filter(
        self,
        has_text: Optional[str] = None,
        has_not_text: Optional[str] = None,
        has: Optional["Locator"] = None,
        has_not: Optional["Locator"] = None,
    ) -> "Locator":
        """
        Filter matching elements.

        Args:
            has_text: Filter by text content.
            has_not_text: Exclude by text content.
            has: Filter by child locator.
            has_not: Exclude by child locator.

        Returns:
            Filtered Locator.
        """
        locator = Locator(self._page, self._selector, **self._options)
        locator._filters = self._filters.copy()

        if has_text:
            locator._filters.append({"type": "has_text", "text": has_text})
        if has_not_text:
            locator._filters.append({"type": "has_not_text", "text": has_not_text})
        if has:
            locator._filters.append({"type": "has", "locator": has})
        if has_not:
            locator._filters.append({"type": "has_not", "locator": has_not})

        return locator

    # ========== Actions ==========

    async def click(
        self,
        button: str = "left",
        click_count: int = 1,
        delay: float = 0,
        force: bool = False,
        modifiers: Optional[List[str]] = None,
        position: Optional[Dict[str, int]] = None,
        timeout: Optional[float] = None,
        no_wait_after: bool = False,
    ):
        """
        Click the element.

        Args:
            button: Mouse button: "left", "right", "middle".
            click_count: Number of clicks.
            delay: Time between mousedown and mouseup in ms.
            force: Bypass actionability checks.
            modifiers: Modifier keys: ["Alt", "Control", "Meta", "Shift"].
            position: Click position relative to element.
            timeout: Timeout in milliseconds.
            no_wait_after: Don't wait for navigation.

        Example:
            ```python
            await page.locator("#submit").click()
            await page.locator("#menu").click(button="right")
            ```
        """
        timeout = timeout or self._page._timeout
        logger.info(f"Clicking: {self._selector}")

        # Wait for element to be actionable
        if not force:
            await self._wait_for_actionable(timeout)

        # Execute click via JavaScript
        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                el.click();
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

        # Apply slow_mo if configured
        if self._page._browser._options.slow_mo > 0:
            await asyncio.sleep(self._page._browser._options.slow_mo / 1000)

    async def dblclick(
        self, delay: float = 0, force: bool = False, timeout: Optional[float] = None, **kwargs
    ):
        """Double-click the element."""
        await self.click(click_count=2, delay=delay, force=force, timeout=timeout, **kwargs)

    async def fill(
        self,
        value: str,
        force: bool = False,
        timeout: Optional[float] = None,
    ):
        """
        Fill an input element. Clears existing value, then types the value.

        Args:
            value: Value to fill.
            force: Bypass actionability checks.
            timeout: Timeout in milliseconds.

        Example:
            ```python
            await page.locator("#email").fill("test@example.com")
            ```
        """
        timeout = timeout or self._page._timeout
        logger.info(f"Filling '{self._selector}' with: {value}")

        if not force:
            await self._wait_for_actionable(timeout)

        # Clear and set value via JavaScript
        escaped_value = value.replace("'", "\\'").replace("\n", "\\n")
        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                el.value = '';
                el.value = '{escaped_value}';
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def type(
        self,
        text: str,
        delay: float = 0,
        timeout: Optional[float] = None,
    ):
        """
        Type text character by character.

        Args:
            text: Text to type.
            delay: Delay between keystrokes in milliseconds.
            timeout: Timeout in milliseconds.
        """
        timeout = timeout or self._page._timeout
        logger.info(f"Typing into '{self._selector}': {text}")

        await self._wait_for_actionable(timeout)

        # Type character by character
        for char in text:
            escaped_char = char.replace("'", "\\'")
            js = f"""
            (function() {{
                const el = document.querySelector('{self._selector}');
                if (el) {{
                    el.value += '{escaped_char}';
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
            """
            self._page._webview.eval_js(js)

            if delay > 0:
                await asyncio.sleep(delay / 1000)

    async def press(
        self,
        key: str,
        delay: float = 0,
        timeout: Optional[float] = None,
    ):
        """
        Press a key (e.g., "Enter", "Control+c").

        Args:
            key: Key to press.
            delay: Time between keydown and keyup in ms.
            timeout: Timeout in milliseconds.

        Example:
            ```python
            await page.locator("#search").press("Enter")
            ```
        """
        timeout = timeout or self._page._timeout
        logger.info(f"Pressing key '{key}' on: {self._selector}")

        await self._wait_for_actionable(timeout)

        # Map key names to key codes
        key_map = {
            "Enter": 13,
            "Tab": 9,
            "Escape": 27,
            "Backspace": 8,
            "Delete": 46,
            "ArrowUp": 38,
            "ArrowDown": 40,
            "ArrowLeft": 37,
            "ArrowRight": 39,
        }

        key_code = key_map.get(key, ord(key[0]) if len(key) == 1 else 0)

        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                const event = new KeyboardEvent('keydown', {{
                    key: '{key}',
                    keyCode: {key_code},
                    bubbles: true
                }});
                el.dispatchEvent(event);
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def check(self, force: bool = False, timeout: Optional[float] = None):
        """Check a checkbox."""
        timeout = timeout or self._page._timeout

        if not force:
            await self._wait_for_actionable(timeout)

        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el && !el.checked) {{
                el.checked = true;
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def uncheck(self, force: bool = False, timeout: Optional[float] = None):
        """Uncheck a checkbox."""
        timeout = timeout or self._page._timeout

        if not force:
            await self._wait_for_actionable(timeout)

        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el && el.checked) {{
                el.checked = false;
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def select_option(
        self,
        value: Union[str, List[str]],
        timeout: Optional[float] = None,
    ):
        """Select option(s) in a select element."""
        timeout = timeout or self._page._timeout

        await self._wait_for_actionable(timeout)

        if isinstance(value, str):
            value = [value]

        values_js = ", ".join(f"'{v}'" for v in value)
        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                const values = [{values_js}];
                for (const opt of el.options) {{
                    opt.selected = values.includes(opt.value);
                }}
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def hover(self, force: bool = False, timeout: Optional[float] = None):
        """Hover over the element."""
        timeout = timeout or self._page._timeout

        if not force:
            await self._wait_for_actionable(timeout)

        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                el.dispatchEvent(new MouseEvent('mouseenter', {{ bubbles: true }}));
                el.dispatchEvent(new MouseEvent('mouseover', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def focus(self, timeout: Optional[float] = None):
        """Focus the element."""
        timeout = timeout or self._page._timeout

        await self._wait_for_actionable(timeout)

        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                el.focus();
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def blur(self, timeout: Optional[float] = None):
        """Remove focus from the element."""
        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                el.blur();
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    async def scroll_into_view_if_needed(self, timeout: Optional[float] = None):
        """Scroll element into view if needed."""
        js = f"""
        (function() {{
            const el = document.querySelector('{self._selector}');
            if (el) {{
                el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                return true;
            }}
            return false;
        }})()
        """
        self._page._webview.eval_js(js)

    # ========== State ==========

    async def is_visible(self, timeout: Optional[float] = None) -> bool:
        """Check if element is visible."""
        # TODO: Implement with eval_js_async
        return True

    async def is_hidden(self, timeout: Optional[float] = None) -> bool:
        """Check if element is hidden."""
        return not await self.is_visible(timeout)

    async def is_enabled(self, timeout: Optional[float] = None) -> bool:
        """Check if element is enabled."""
        # TODO: Implement with eval_js_async
        return True

    async def is_disabled(self, timeout: Optional[float] = None) -> bool:
        """Check if element is disabled."""
        return not await self.is_enabled(timeout)

    async def is_checked(self, timeout: Optional[float] = None) -> bool:
        """Check if checkbox/radio is checked."""
        # TODO: Implement with eval_js_async
        return False

    async def is_editable(self, timeout: Optional[float] = None) -> bool:
        """Check if element is editable."""
        # TODO: Implement with eval_js_async
        return True

    # ========== Content ==========

    async def text_content(self, timeout: Optional[float] = None) -> Optional[str]:
        """Get element text content."""
        # TODO: Implement with eval_js_async
        return None

    async def inner_text(self, timeout: Optional[float] = None) -> str:
        """Get element inner text."""
        # TODO: Implement with eval_js_async
        return ""

    async def inner_html(self, timeout: Optional[float] = None) -> str:
        """Get element inner HTML."""
        # TODO: Implement with eval_js_async
        return ""

    async def input_value(self, timeout: Optional[float] = None) -> str:
        """Get input element value."""
        # TODO: Implement with eval_js_async
        return ""

    async def get_attribute(self, name: str, timeout: Optional[float] = None) -> Optional[str]:
        """Get element attribute value."""
        # TODO: Implement with eval_js_async
        return None

    async def count(self) -> int:
        """Get number of matching elements."""
        # TODO: Implement with eval_js_async
        return 1

    async def all(self) -> List["Locator"]:
        """Get all matching elements as locators."""
        count = await self.count()
        return [self.nth(i) for i in range(count)]

    # ========== Screenshots ==========

    async def screenshot(
        self,
        path: Optional[str] = None,
        type: str = "png",
        quality: Optional[int] = None,
        timeout: Optional[float] = None,
    ) -> bytes:
        """Take a screenshot of the element."""
        # TODO: Implement element screenshot
        logger.warning("Element screenshot not fully implemented yet")
        return b""

    # ========== Internal ==========

    async def _wait_for_actionable(self, timeout: float):
        """Wait for element to be actionable (visible, enabled, stable)."""
        start = time.time()
        timeout_sec = timeout / 1000

        while time.time() - start < timeout_sec:
            if await self.is_visible() and await self.is_enabled():
                return
            await asyncio.sleep(0.1)

        raise TimeoutError(f"Element '{self._selector}' not actionable within {timeout}ms")

    def __repr__(self) -> str:
        """String representation."""
        return f"Locator({self._selector!r})"
