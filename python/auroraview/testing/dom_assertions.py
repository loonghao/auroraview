"""DOM-based assertions for AuroraView testing.

This module provides assertion functions that use the DOM API for real
value verification, unlike the event-based assertions that only check
if JavaScript executed successfully.

Example:
    ```python
    from auroraview.testing import DomAssertions

    def test_form_submission(webview):
        assertions = DomAssertions(webview)

        # These assertions actually verify values!
        assertions.assert_text("#title", "Welcome")
        assertions.assert_value("#email", "test@example.com")
        assertions.assert_has_class(".button", "active")
        assertions.assert_visible("#modal")
    ```
"""

from __future__ import annotations

import time
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from ..dom import Element
    from ..webview import WebView


class DomAssertions:
    """DOM-based assertions that verify actual values.

    Unlike the event-based assertions, these use the DOM API to
    synchronously get values from the WebView and verify them.
    """

    def __init__(self, webview: WebView, timeout: float = 5.0):
        """Initialize DomAssertions.

        Args:
            webview: The WebView instance to assert against.
            timeout: Default timeout for wait operations in seconds.
        """
        self.webview = webview
        self.timeout = timeout

    def _dom(self, selector: str) -> Element:
        """Get a DOM element by selector."""
        return self.webview.dom(selector)

    # ========== Text Assertions ==========

    def assert_text(self, selector: str, expected: str, message: Optional[str] = None):
        """Assert that an element's text content matches expected value.

        Args:
            selector: CSS selector for the element.
            expected: Expected text content.
            message: Optional custom error message.

        Raises:
            AssertionError: If text doesn't match.
        """
        actual = self._dom(selector).get_text()
        if actual != expected:
            msg = (
                message or f"Text mismatch for '{selector}': expected '{expected}', got '{actual}'"
            )
            raise AssertionError(msg)

    def assert_text_contains(self, selector: str, substring: str, message: Optional[str] = None):
        """Assert that an element's text contains a substring.

        Args:
            selector: CSS selector for the element.
            substring: Expected substring.
            message: Optional custom error message.

        Raises:
            AssertionError: If substring not found.
        """
        actual = self._dom(selector).get_text()
        if substring not in actual:
            msg = message or f"Text '{actual}' does not contain '{substring}' for '{selector}'"
            raise AssertionError(msg)

    def assert_text_matches(self, selector: str, pattern: str, message: Optional[str] = None):
        """Assert that an element's text matches a regex pattern.

        Args:
            selector: CSS selector for the element.
            pattern: Regex pattern to match.
            message: Optional custom error message.

        Raises:
            AssertionError: If pattern doesn't match.
        """
        import re

        actual = self._dom(selector).get_text()
        if not re.search(pattern, actual):
            msg = message or f"Text '{actual}' does not match pattern '{pattern}' for '{selector}'"
            raise AssertionError(msg)

    # ========== Value Assertions ==========

    def assert_value(self, selector: str, expected: str, message: Optional[str] = None):
        """Assert that an input's value matches expected.

        Args:
            selector: CSS selector for the input element.
            expected: Expected input value.
            message: Optional custom error message.

        Raises:
            AssertionError: If value doesn't match.
        """
        actual = self._dom(selector).get_value()
        if actual != expected:
            msg = (
                message or f"Value mismatch for '{selector}': expected '{expected}', got '{actual}'"
            )
            raise AssertionError(msg)

    def assert_checked(self, selector: str, message: Optional[str] = None):
        """Assert that a checkbox/radio is checked.

        Args:
            selector: CSS selector for the checkbox/radio.
            message: Optional custom error message.

        Raises:
            AssertionError: If not checked.
        """
        checked = self._dom(selector).get_checked()
        if not checked:
            msg = message or f"Element '{selector}' is not checked"
            raise AssertionError(msg)

    def assert_not_checked(self, selector: str, message: Optional[str] = None):
        """Assert that a checkbox/radio is not checked.

        Args:
            selector: CSS selector for the checkbox/radio.
            message: Optional custom error message.

        Raises:
            AssertionError: If checked.
        """
        checked = self._dom(selector).get_checked()
        if checked:
            msg = message or f"Element '{selector}' is checked"
            raise AssertionError(msg)

    # ========== Attribute Assertions ==========

    def assert_attribute(
        self, selector: str, name: str, expected: str, message: Optional[str] = None
    ):
        """Assert that an element has an attribute with expected value.

        Args:
            selector: CSS selector for the element.
            name: Attribute name.
            expected: Expected attribute value.
            message: Optional custom error message.

        Raises:
            AssertionError: If attribute value doesn't match.
        """
        actual = self._dom(selector).get_attribute(name)
        if actual != expected:
            msg = message or (
                f"Attribute '{name}' mismatch for '{selector}': "
                f"expected '{expected}', got '{actual}'"
            )
            raise AssertionError(msg)

    def assert_has_attribute(self, selector: str, name: str, message: Optional[str] = None):
        """Assert that an element has a specific attribute.

        Args:
            selector: CSS selector for the element.
            name: Attribute name to check.
            message: Optional custom error message.

        Raises:
            AssertionError: If attribute doesn't exist.
        """
        has_attr = self._dom(selector).has_attribute(name)
        if not has_attr:
            msg = message or f"Element '{selector}' does not have attribute '{name}'"
            raise AssertionError(msg)

    # ========== Class Assertions ==========

    def assert_has_class(self, selector: str, class_name: str, message: Optional[str] = None):
        """Assert that an element has a specific CSS class.

        Args:
            selector: CSS selector for the element.
            class_name: Class name to check.
            message: Optional custom error message.

        Raises:
            AssertionError: If class not present.
        """
        has_class = self._dom(selector).has_class(class_name)
        if not has_class:
            msg = message or f"Element '{selector}' does not have class '{class_name}'"
            raise AssertionError(msg)

    def assert_not_has_class(self, selector: str, class_name: str, message: Optional[str] = None):
        """Assert that an element does not have a specific CSS class.

        Args:
            selector: CSS selector for the element.
            class_name: Class name to check.
            message: Optional custom error message.

        Raises:
            AssertionError: If class is present.
        """
        has_class = self._dom(selector).has_class(class_name)
        if has_class:
            msg = message or f"Element '{selector}' has class '{class_name}'"
            raise AssertionError(msg)

    # ========== Visibility Assertions ==========

    def assert_visible(self, selector: str, message: Optional[str] = None):
        """Assert that an element is visible.

        Args:
            selector: CSS selector for the element.
            message: Optional custom error message.

        Raises:
            AssertionError: If element is not visible.
        """
        visible = self._dom(selector).is_visible()
        if not visible:
            msg = message or f"Element '{selector}' is not visible"
            raise AssertionError(msg)

    def assert_hidden(self, selector: str, message: Optional[str] = None):
        """Assert that an element is hidden.

        Args:
            selector: CSS selector for the element.
            message: Optional custom error message.

        Raises:
            AssertionError: If element is visible.
        """
        visible = self._dom(selector).is_visible()
        if visible:
            msg = message or f"Element '{selector}' is visible"
            raise AssertionError(msg)

    def assert_disabled(self, selector: str, message: Optional[str] = None):
        """Assert that an element is disabled.

        Args:
            selector: CSS selector for the element.
            message: Optional custom error message.

        Raises:
            AssertionError: If element is not disabled.
        """
        disabled = self._dom(selector).is_disabled()
        if not disabled:
            msg = message or f"Element '{selector}' is not disabled"
            raise AssertionError(msg)

    def assert_enabled(self, selector: str, message: Optional[str] = None):
        """Assert that an element is enabled.

        Args:
            selector: CSS selector for the element.
            message: Optional custom error message.

        Raises:
            AssertionError: If element is disabled.
        """
        disabled = self._dom(selector).is_disabled()
        if disabled:
            msg = message or f"Element '{selector}' is disabled"
            raise AssertionError(msg)

    # ========== Wait Assertions ==========

    def wait_for_text(
        self,
        selector: str,
        expected: str,
        timeout: Optional[float] = None,
        message: Optional[str] = None,
    ):
        """Wait for an element's text to match expected value.

        Args:
            selector: CSS selector for the element.
            expected: Expected text content.
            timeout: Timeout in seconds (uses default if None).
            message: Optional custom error message.

        Raises:
            AssertionError: If timeout reached without match.
        """
        timeout = timeout or self.timeout
        start = time.time()
        last_value = None

        while time.time() - start < timeout:
            try:
                actual = self._dom(selector).get_text()
                last_value = actual
                if actual == expected:
                    return
            except Exception:
                pass
            time.sleep(0.1)

        msg = message or (
            f"Timeout waiting for text '{expected}' on '{selector}'. Last value: '{last_value}'"
        )
        raise AssertionError(msg)

    def wait_for_visible(
        self,
        selector: str,
        timeout: Optional[float] = None,
        message: Optional[str] = None,
    ):
        """Wait for an element to become visible.

        Args:
            selector: CSS selector for the element.
            timeout: Timeout in seconds (uses default if None).
            message: Optional custom error message.

        Raises:
            AssertionError: If timeout reached without becoming visible.
        """
        timeout = timeout or self.timeout
        start = time.time()

        while time.time() - start < timeout:
            try:
                if self._dom(selector).is_visible():
                    return
            except Exception:
                pass
            time.sleep(0.1)

        msg = message or f"Timeout waiting for '{selector}' to become visible"
        raise AssertionError(msg)

    def wait_for_hidden(
        self,
        selector: str,
        timeout: Optional[float] = None,
        message: Optional[str] = None,
    ):
        """Wait for an element to become hidden.

        Args:
            selector: CSS selector for the element.
            timeout: Timeout in seconds (uses default if None).
            message: Optional custom error message.

        Raises:
            AssertionError: If timeout reached without becoming hidden.
        """
        timeout = timeout or self.timeout
        start = time.time()

        while time.time() - start < timeout:
            try:
                if not self._dom(selector).is_visible():
                    return
            except Exception:
                pass
            time.sleep(0.1)

        msg = message or f"Timeout waiting for '{selector}' to become hidden"
        raise AssertionError(msg)

    # ========== Count Assertions ==========

    def assert_count(self, selector: str, expected: int, message: Optional[str] = None):
        """Assert the number of elements matching a selector.

        Args:
            selector: CSS selector for elements.
            expected: Expected count.
            message: Optional custom error message.

        Raises:
            AssertionError: If count doesn't match.
        """
        actual = self.webview.dom_all(selector).count()
        if actual != expected:
            msg = message or f"Count mismatch for '{selector}': expected {expected}, got {actual}"
            raise AssertionError(msg)

    def assert_exists(self, selector: str, message: Optional[str] = None):
        """Assert that at least one element matches the selector.

        Args:
            selector: CSS selector for elements.
            message: Optional custom error message.

        Raises:
            AssertionError: If no elements match.
        """
        count = self.webview.dom_all(selector).count()
        if count == 0:
            msg = message or f"No elements found matching '{selector}'"
            raise AssertionError(msg)

    def assert_not_exists(self, selector: str, message: Optional[str] = None):
        """Assert that no elements match the selector.

        Args:
            selector: CSS selector for elements.
            message: Optional custom error message.

        Raises:
            AssertionError: If any elements match.
        """
        count = self.webview.dom_all(selector).count()
        if count > 0:
            msg = message or f"Found {count} elements matching '{selector}', expected none"
            raise AssertionError(msg)
