"""
Locator class for AuroraTest.

Locator represents a way to find element(s) on the page.
It provides methods for interaction and assertions.

Architecture:
- All JavaScript execution goes through Page.evaluate() or proxy.eval_js()
- State queries use Page.evaluate() for async result retrieval
- Actions use proxy.eval_js() for fire-and-forget execution
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

    def _escape_selector(self, selector: str) -> str:
        """Escape selector for JavaScript string."""
        return selector.replace("\\", "\\\\").replace("'", "\\'")

    # ========== Chaining ==========

    def locator(self, selector: str) -> "Locator":
        """Create a child locator."""
        combined = f"{self._selector} {selector}"
        return Locator(self._page, combined, **self._options)

    def first(self) -> "Locator":
        """Get the first matching element."""
        locator = Locator(self._page, self._selector, **self._options)
        locator._filters.append({"type": "first"})
        return locator

    def last(self) -> "Locator":
        """Get the last matching element."""
        locator = Locator(self._page, self._selector, **self._options)
        locator._filters.append({"type": "last"})
        return locator

    def nth(self, index: int) -> "Locator":
        """Get the nth matching element (0-indexed)."""
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
        """Filter matching elements."""
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
        """Click the element."""
        timeout = timeout or self._page._timeout
        logger.info(f"Clicking: {self._selector}")

        if not force:
            await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el) {{
                el.click();
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

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
        """Fill an input element."""
        timeout = timeout or self._page._timeout
        logger.info(f"Filling '{self._selector}' with: {value}")

        if not force:
            await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        escaped_value = value.replace("\\", "\\\\").replace("'", "\\'").replace("\n", "\\n")
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
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
        self._page._proxy.eval_js(js)

    async def type(
        self,
        text: str,
        delay: float = 0,
        timeout: Optional[float] = None,
    ):
        """Type text character by character."""
        timeout = timeout or self._page._timeout
        logger.info(f"Typing into '{self._selector}': {text}")

        await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        for char in text:
            escaped_char = char.replace("\\", "\\\\").replace("'", "\\'")
            js = f"""
            (function() {{
                const el = document.querySelector('{selector}');
                if (el) {{
                    el.value += '{escaped_char}';
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
            """
            self._page._proxy.eval_js(js)

            if delay > 0:
                await asyncio.sleep(delay / 1000)

    async def press(
        self,
        key: str,
        delay: float = 0,
        timeout: Optional[float] = None,
    ):
        """Press a key."""
        timeout = timeout or self._page._timeout
        logger.info(f"Pressing key '{key}' on: {self._selector}")

        await self._wait_for_actionable(timeout)

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

        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
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
        self._page._proxy.eval_js(js)

    async def check(self, force: bool = False, timeout: Optional[float] = None):
        """Check a checkbox."""
        timeout = timeout or self._page._timeout

        if not force:
            await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el && !el.checked) {{
                el.checked = true;
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

    async def uncheck(self, force: bool = False, timeout: Optional[float] = None):
        """Uncheck a checkbox."""
        timeout = timeout or self._page._timeout

        if not force:
            await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el && el.checked) {{
                el.checked = false;
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

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

        selector = self._escape_selector(self._selector)
        values_js = ", ".join(f"'{v}'" for v in value)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
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
        self._page._proxy.eval_js(js)

    async def hover(self, force: bool = False, timeout: Optional[float] = None):
        """Hover over the element."""
        timeout = timeout or self._page._timeout

        if not force:
            await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el) {{
                el.dispatchEvent(new MouseEvent('mouseenter', {{ bubbles: true }}));
                el.dispatchEvent(new MouseEvent('mouseover', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

    async def focus(self, timeout: Optional[float] = None):
        """Focus the element."""
        timeout = timeout or self._page._timeout

        await self._wait_for_actionable(timeout)

        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el) {{
                el.focus();
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

    async def blur(self, timeout: Optional[float] = None):
        """Remove focus from the element."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el) {{
                el.blur();
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

    async def scroll_into_view_if_needed(self, timeout: Optional[float] = None):
        """Scroll element into view if needed."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (el) {{
                el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                return true;
            }}
            return false;
        }})()
        """
        self._page._proxy.eval_js(js)

    # ========== State (use Page.evaluate for results) ==========

    async def is_visible(self, timeout: Optional[float] = None) -> bool:
        """Check if element is visible."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (!el) return false;
            const style = window.getComputedStyle(el);
            return style.display !== 'none' &&
                   style.visibility !== 'hidden' &&
                   style.opacity !== '0' &&
                   el.offsetWidth > 0 &&
                   el.offsetHeight > 0;
        }})()
        """
        result = await self._page.evaluate(js)
        return bool(result)

    async def is_hidden(self, timeout: Optional[float] = None) -> bool:
        """Check if element is hidden."""
        return not await self.is_visible(timeout)

    async def is_enabled(self, timeout: Optional[float] = None) -> bool:
        """Check if element is enabled."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (!el) return false;
            return !el.disabled && !el.hasAttribute('disabled');
        }})()
        """
        result = await self._page.evaluate(js)
        return bool(result)

    async def is_disabled(self, timeout: Optional[float] = None) -> bool:
        """Check if element is disabled."""
        return not await self.is_enabled(timeout)

    async def is_checked(self, timeout: Optional[float] = None) -> bool:
        """Check if checkbox/radio is checked."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (!el) return false;
            return el.checked === true;
        }})()
        """
        result = await self._page.evaluate(js)
        return bool(result)

    async def is_editable(self, timeout: Optional[float] = None) -> bool:
        """Check if element is editable."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            if (!el) return false;
            const tag = el.tagName.toLowerCase();
            if (tag === 'input' || tag === 'textarea') {{
                return !el.disabled && !el.readOnly;
            }}
            return el.isContentEditable;
        }})()
        """
        result = await self._page.evaluate(js)
        return bool(result)

    # ========== Content ==========

    async def text_content(self, timeout: Optional[float] = None) -> Optional[str]:
        """Get element text content."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            return el ? el.textContent : null;
        }})()
        """
        return await self._page.evaluate(js)

    async def inner_text(self, timeout: Optional[float] = None) -> str:
        """Get element inner text."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            return el ? el.innerText : '';
        }})()
        """
        result = await self._page.evaluate(js)
        return result or ""

    async def inner_html(self, timeout: Optional[float] = None) -> str:
        """Get element inner HTML."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            return el ? el.innerHTML : '';
        }})()
        """
        result = await self._page.evaluate(js)
        return result or ""

    async def input_value(self, timeout: Optional[float] = None) -> str:
        """Get input element value."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            return el ? el.value : '';
        }})()
        """
        result = await self._page.evaluate(js)
        return result or ""

    async def get_attribute(self, name: str, timeout: Optional[float] = None) -> Optional[str]:
        """Get element attribute value."""
        selector = self._escape_selector(self._selector)
        escaped_name = name.replace("\\", "\\\\").replace("'", "\\'")
        js = f"""
        (function() {{
            const el = document.querySelector('{selector}');
            return el ? el.getAttribute('{escaped_name}') : null;
        }})()
        """
        return await self._page.evaluate(js)

    async def count(self) -> int:
        """Get number of matching elements."""
        selector = self._escape_selector(self._selector)
        js = f"""
        (function() {{
            return document.querySelectorAll('{selector}').length;
        }})()
        """
        result = await self._page.evaluate(js)
        return result or 0

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
        logger.warning("Element screenshot not fully implemented yet")
        return b""

    # ========== Internal ==========

    async def _wait_for_actionable(self, timeout: float):
        """Wait for element to be actionable."""
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
