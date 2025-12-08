"""
Expect assertions for AuroraTest.

Provides Playwright-compatible assertion API.
"""

from __future__ import annotations

import asyncio
import logging
import time
from typing import TYPE_CHECKING, List, Optional, Pattern, Union

if TYPE_CHECKING:
    from .locator import Locator
    from .page import Page

logger = logging.getLogger(__name__)


class LocatorAssertions:
    """
    Assertions for Locator objects.

    Example:
        ```python
        await expect(page.locator("#title")).to_have_text("Welcome")
        await expect(page.locator("#submit")).to_be_visible()
        await expect(page.locator("#loading")).not_to_be_visible()
        ```
    """

    def __init__(self, locator: "Locator", is_not: bool = False):
        """Initialize assertions."""
        self._locator = locator
        self._is_not = is_not
        self._timeout = 5000  # Default assertion timeout

    @property
    def not_(self) -> "LocatorAssertions":
        """Negate the assertion."""
        return LocatorAssertions(self._locator, is_not=True)

    def _check(self, condition: bool, message: str):
        """Check condition and raise if failed."""
        if self._is_not:
            condition = not condition
        if not condition:
            prefix = "Expected NOT " if self._is_not else "Expected "
            raise AssertionError(f"{prefix}{message}")

    # ========== Visibility ==========

    async def to_be_visible(self, timeout: Optional[float] = None):
        """
        Assert element is visible.

        Example:
            ```python
            await expect(page.locator("#modal")).to_be_visible()
            ```
        """
        timeout = timeout or self._timeout
        start = time.time()

        while time.time() - start < timeout / 1000:
            is_visible = await self._locator.is_visible()
            if is_visible != self._is_not:
                return
            await asyncio.sleep(0.1)

        self._check(False, f"'{self._locator._selector}' to be visible")

    async def to_be_hidden(self, timeout: Optional[float] = None):
        """Assert element is hidden."""
        timeout = timeout or self._timeout
        start = time.time()

        while time.time() - start < timeout / 1000:
            is_hidden = await self._locator.is_hidden()
            if is_hidden != self._is_not:
                return
            await asyncio.sleep(0.1)

        self._check(False, f"'{self._locator._selector}' to be hidden")

    async def to_be_attached(self, timeout: Optional[float] = None):
        """Assert element is attached to DOM."""
        timeout = timeout or self._timeout
        start = time.time()

        while time.time() - start < timeout / 1000:
            count = await self._locator.count()
            if (count > 0) != self._is_not:
                return
            await asyncio.sleep(0.1)

        self._check(False, f"'{self._locator._selector}' to be attached")

    async def to_be_detached(self, timeout: Optional[float] = None):
        """Assert element is detached from DOM."""
        timeout = timeout or self._timeout
        start = time.time()

        while time.time() - start < timeout / 1000:
            count = await self._locator.count()
            if (count == 0) != self._is_not:
                return
            await asyncio.sleep(0.1)

        self._check(False, f"'{self._locator._selector}' to be detached")

    # ========== State ==========

    async def to_be_enabled(self, timeout: Optional[float] = None):
        """Assert element is enabled."""
        timeout = timeout or self._timeout
        is_enabled = await self._locator.is_enabled(timeout)
        self._check(is_enabled, f"'{self._locator._selector}' to be enabled")

    async def to_be_disabled(self, timeout: Optional[float] = None):
        """Assert element is disabled."""
        timeout = timeout or self._timeout
        is_disabled = await self._locator.is_disabled(timeout)
        self._check(is_disabled, f"'{self._locator._selector}' to be disabled")

    async def to_be_editable(self, timeout: Optional[float] = None):
        """Assert element is editable."""
        timeout = timeout or self._timeout
        is_editable = await self._locator.is_editable(timeout)
        self._check(is_editable, f"'{self._locator._selector}' to be editable")

    async def to_be_checked(self, timeout: Optional[float] = None):
        """Assert checkbox/radio is checked."""
        timeout = timeout or self._timeout
        is_checked = await self._locator.is_checked(timeout)
        self._check(is_checked, f"'{self._locator._selector}' to be checked")

    async def to_be_focused(self, timeout: Optional[float] = None):
        """Assert element is focused."""
        # TODO: Implement focus check
        pass

    async def to_be_empty(self, timeout: Optional[float] = None):
        """Assert element is empty."""
        timeout = timeout or self._timeout
        text = await self._locator.text_content(timeout)
        is_empty = text is None or text.strip() == ""
        self._check(is_empty, f"'{self._locator._selector}' to be empty")

    # ========== Content ==========

    async def to_have_text(
        self,
        expected: Union[str, Pattern, List[Union[str, Pattern]]],
        use_inner_text: bool = False,
        ignore_case: bool = False,
        timeout: Optional[float] = None,
    ):
        """
        Assert element has specific text.

        Args:
            expected: Expected text (string, regex, or list).
            use_inner_text: Use innerText instead of textContent.
            ignore_case: Case-insensitive comparison.
            timeout: Timeout in milliseconds.

        Example:
            ```python
            await expect(page.locator("#title")).to_have_text("Welcome")
            await expect(page.locator("#title")).to_have_text(re.compile(r"Welcome.*"))
            ```
        """
        timeout = timeout or self._timeout

        if use_inner_text:
            actual = await self._locator.inner_text(timeout)
        else:
            actual = await self._locator.text_content(timeout) or ""

        if isinstance(expected, Pattern):
            matches = bool(expected.search(actual))
        elif isinstance(expected, list):
            matches = any(
                (e.search(actual) if isinstance(e, Pattern) else e in actual) for e in expected
            )
        else:
            if ignore_case:
                matches = expected.lower() in actual.lower()
            else:
                matches = expected in actual

        self._check(
            matches, f"'{self._locator._selector}' to have text '{expected}', got '{actual}'"
        )

    async def to_contain_text(
        self,
        expected: Union[str, Pattern, List[Union[str, Pattern]]],
        use_inner_text: bool = False,
        ignore_case: bool = False,
        timeout: Optional[float] = None,
    ):
        """Assert element contains specific text."""
        await self.to_have_text(
            expected,
            use_inner_text=use_inner_text,
            ignore_case=ignore_case,
            timeout=timeout,
        )

    async def to_have_value(
        self,
        value: Union[str, Pattern],
        timeout: Optional[float] = None,
    ):
        """
        Assert input element has specific value.

        Example:
            ```python
            await expect(page.locator("#email")).to_have_value("test@example.com")
            ```
        """
        timeout = timeout or self._timeout
        actual = await self._locator.input_value(timeout)

        if isinstance(value, Pattern):
            matches = bool(value.search(actual))
        else:
            matches = actual == value

        self._check(matches, f"'{self._locator._selector}' to have value '{value}', got '{actual}'")

    async def to_have_values(
        self,
        values: List[Union[str, Pattern]],
        timeout: Optional[float] = None,
    ):
        """Assert multi-select has specific values."""
        # TODO: Implement for multi-select
        pass

    # ========== Attributes ==========

    async def to_have_attribute(
        self,
        name: str,
        value: Optional[Union[str, Pattern]] = None,
        timeout: Optional[float] = None,
    ):
        """
        Assert element has specific attribute.

        Args:
            name: Attribute name.
            value: Expected value (optional, just checks existence if None).
            timeout: Timeout in milliseconds.

        Example:
            ```python
            await expect(page.locator("#link")).to_have_attribute("href", "/home")
            await expect(page.locator("#btn")).to_have_attribute("disabled")
            ```
        """
        timeout = timeout or self._timeout
        actual = await self._locator.get_attribute(name, timeout)

        if value is None:
            matches = actual is not None
        elif isinstance(value, Pattern):
            matches = actual is not None and bool(value.search(actual))
        else:
            matches = actual == value

        self._check(
            matches,
            f"'{self._locator._selector}' to have attribute '{name}'='{value}', got '{actual}'",
        )

    async def to_have_class(
        self,
        expected: Union[str, Pattern, List[Union[str, Pattern]]],
        timeout: Optional[float] = None,
    ):
        """
        Assert element has specific class(es).

        Example:
            ```python
            await expect(page.locator("#btn")).to_have_class("primary")
            await expect(page.locator("#btn")).to_have_class(["btn", "primary"])
            ```
        """
        timeout = timeout or self._timeout
        actual = await self._locator.get_attribute("class", timeout) or ""
        classes = actual.split()

        if isinstance(expected, Pattern):
            matches = bool(expected.search(actual))
        elif isinstance(expected, list):
            matches = all(
                (e.search(actual) if isinstance(e, Pattern) else e in classes) for e in expected
            )
        else:
            matches = expected in classes

        self._check(
            matches, f"'{self._locator._selector}' to have class '{expected}', got '{actual}'"
        )

    async def to_have_id(
        self,
        id: Union[str, Pattern],
        timeout: Optional[float] = None,
    ):
        """Assert element has specific id."""
        await self.to_have_attribute("id", id, timeout)

    async def to_have_css(
        self,
        name: str,
        value: Union[str, Pattern],
        timeout: Optional[float] = None,
    ):
        """Assert element has specific CSS property value."""
        # TODO: Implement CSS property check
        pass

    # ========== Count ==========

    async def to_have_count(
        self,
        count: int,
        timeout: Optional[float] = None,
    ):
        """
        Assert number of matching elements.

        Example:
            ```python
            await expect(page.locator(".item")).to_have_count(5)
            ```
        """
        timeout = timeout or self._timeout
        start = time.time()

        while time.time() - start < timeout / 1000:
            actual = await self._locator.count()
            if (actual == count) != self._is_not:
                return
            await asyncio.sleep(0.1)

        actual = await self._locator.count()
        self._check(
            actual == count, f"'{self._locator._selector}' to have count {count}, got {actual}"
        )


class PageAssertions:
    """
    Assertions for Page objects.

    Example:
        ```python
        await expect(page).to_have_title("Home")
        await expect(page).to_have_url("https://example.com")
        ```
    """

    def __init__(self, page: "Page", is_not: bool = False):
        """Initialize assertions."""
        self._page = page
        self._is_not = is_not
        self._timeout = 5000

    @property
    def not_(self) -> "PageAssertions":
        """Negate the assertion."""
        return PageAssertions(self._page, is_not=True)

    def _check(self, condition: bool, message: str):
        """Check condition and raise if failed."""
        if self._is_not:
            condition = not condition
        if not condition:
            prefix = "Expected NOT " if self._is_not else "Expected "
            raise AssertionError(f"{prefix}{message}")

    async def to_have_title(
        self,
        title: Union[str, Pattern],
        timeout: Optional[float] = None,
    ):
        """
        Assert page has specific title.

        Example:
            ```python
            await expect(page).to_have_title("Home Page")
            await expect(page).to_have_title(re.compile(r"Home.*"))
            ```
        """
        timeout = timeout or self._timeout
        actual = await self._page.title()

        if isinstance(title, Pattern):
            matches = bool(title.search(actual))
        else:
            matches = actual == title

        self._check(matches, f"page to have title '{title}', got '{actual}'")

    async def to_have_url(
        self,
        url: Union[str, Pattern],
        timeout: Optional[float] = None,
    ):
        """
        Assert page has specific URL.

        Example:
            ```python
            await expect(page).to_have_url("https://example.com/home")
            await expect(page).to_have_url(re.compile(r".*example.com.*"))
            ```
        """
        timeout = timeout or self._timeout
        actual = self._page.url

        if isinstance(url, Pattern):
            matches = bool(url.search(actual))
        else:
            matches = actual == url

        self._check(matches, f"page to have URL '{url}', got '{actual}'")


def expect(target: Union["Locator", "Page"]) -> Union[LocatorAssertions, PageAssertions]:
    """
    Create assertions for a Locator or Page.

    Args:
        target: Locator or Page to assert on.

    Returns:
        LocatorAssertions or PageAssertions.

    Example:
        ```python
        # Locator assertions
        await expect(page.locator("#title")).to_have_text("Welcome")
        await expect(page.locator("#submit")).to_be_visible()
        await expect(page.locator("#loading")).not_.to_be_visible()

        # Page assertions
        await expect(page).to_have_title("Home")
        await expect(page).to_have_url("https://example.com")
        ```
    """
    # Import here to avoid circular imports
    from .locator import Locator
    from .page import Page

    if isinstance(target, Locator):
        return LocatorAssertions(target)
    elif isinstance(target, Page):
        return PageAssertions(target)
    else:
        raise TypeError(f"expect() requires Locator or Page, got {type(target)}")
