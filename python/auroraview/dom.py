"""DOM Element manipulation module for AuroraView.

This module provides a Pythonic interface for DOM manipulation,
inspired by PyWebView's Element API and designed for future
Steel Browser integration.

Example:
    >>> # Basic usage
    >>> element = webview.dom("#my-button")
    >>> element.click()
    >>> element.set_text("Clicked!")

    >>> # Form handling
    >>> form = webview.dom("#login-form")
    >>> form.query("#username").set_value("admin")
    >>> form.query("#password").set_value("secret")
    >>> form.query("button[type=submit]").click()

    >>> # Batch operations
    >>> for item in webview.dom_all(".list-item"):
    ...     item.add_class("processed")
"""

from __future__ import annotations

import json
import logging
from typing import TYPE_CHECKING, Dict, Optional

if TYPE_CHECKING:
    from .webview import WebView

logger = logging.getLogger(__name__)

__all__ = ["Element", "ElementCollection"]


class Element:
    """DOM Element wrapper with Steel-compatible API.

    Provides a high-level interface for DOM manipulation using
    JavaScript evaluation under the hood.

    Attributes:
        selector: CSS selector used to identify this element.
    """

    __slots__ = ("_webview", "_selector")

    def __init__(self, webview: "WebView", selector: str) -> None:
        """Initialize Element wrapper.

        Args:
            webview: Parent WebView instance.
            selector: CSS selector for the element.
        """
        self._webview = webview
        self._selector = selector

    @property
    def selector(self) -> str:
        """Get the CSS selector for this element."""
        return self._selector

    def _eval(self, script: str) -> None:
        """Execute JavaScript on this element.

        Args:
            script: JavaScript code where `el` refers to the element.
        """
        full_script = f"""
        (function() {{
            const el = document.querySelector("{self._escape_selector()}");
            if (el) {{ {script} }}
        }})();
        """
        self._webview.eval_js(full_script)

    def _escape_selector(self) -> str:
        """Escape selector for JavaScript string."""
        return self._selector.replace("\\", "\\\\").replace('"', '\\"')

    # === Text & Content ===

    def get_text(self) -> None:
        """Get element's text content (async via bridge)."""
        self._eval("window.__auroraview_result = el.textContent;")

    def set_text(self, text: str) -> None:
        """Set element's text content.

        Args:
            text: New text content.
        """
        escaped = json.dumps(text)
        self._eval(f"el.textContent = {escaped};")

    def get_html(self) -> None:
        """Get element's innerHTML (async via bridge)."""
        self._eval("window.__auroraview_result = el.innerHTML;")

    def set_html(self, html: str) -> None:
        """Set element's innerHTML.

        Args:
            html: New HTML content.
        """
        escaped = json.dumps(html)
        self._eval(f"el.innerHTML = {escaped};")

    def get_outer_html(self) -> None:
        """Get element's outerHTML (async via bridge)."""
        self._eval("window.__auroraview_result = el.outerHTML;")

    # === Attributes ===

    def get_attribute(self, name: str) -> None:
        """Get attribute value (async via bridge).

        Args:
            name: Attribute name.
        """
        escaped = json.dumps(name)
        self._eval(f"window.__auroraview_result = el.getAttribute({escaped});")

    def set_attribute(self, name: str, value: str) -> None:
        """Set attribute value.

        Args:
            name: Attribute name.
            value: Attribute value.
        """
        name_escaped = json.dumps(name)
        value_escaped = json.dumps(value)
        self._eval(f"el.setAttribute({name_escaped}, {value_escaped});")

    def remove_attribute(self, name: str) -> None:
        """Remove attribute.

        Args:
            name: Attribute name to remove.
        """
        escaped = json.dumps(name)
        self._eval(f"el.removeAttribute({escaped});")

    def has_attribute(self, name: str) -> None:
        """Check if attribute exists (async via bridge).

        Args:
            name: Attribute name.
        """
        escaped = json.dumps(name)
        self._eval(f"window.__auroraview_result = el.hasAttribute({escaped});")

    # === Classes ===

    def add_class(self, *class_names: str) -> None:
        """Add CSS class(es).

        Args:
            *class_names: One or more class names to add.
        """
        classes = ", ".join(json.dumps(c) for c in class_names)
        self._eval(f"el.classList.add({classes});")

    def remove_class(self, *class_names: str) -> None:
        """Remove CSS class(es).

        Args:
            *class_names: One or more class names to remove.
        """
        classes = ", ".join(json.dumps(c) for c in class_names)
        self._eval(f"el.classList.remove({classes});")

    def toggle_class(self, class_name: str, force: Optional[bool] = None) -> None:
        """Toggle CSS class.

        Args:
            class_name: Class name to toggle.
            force: If provided, forces add (True) or remove (False).
        """
        escaped = json.dumps(class_name)
        if force is None:
            self._eval(f"el.classList.toggle({escaped});")
        else:
            self._eval(f"el.classList.toggle({escaped}, {str(force).lower()});")

    def has_class(self, class_name: str) -> None:
        """Check if element has class (async via bridge).

        Args:
            class_name: Class name to check.
        """
        escaped = json.dumps(class_name)
        self._eval(f"window.__auroraview_result = el.classList.contains({escaped});")

    # === Styles ===

    def get_style(self, property_name: str) -> None:
        """Get computed style property (async via bridge).

        Args:
            property_name: CSS property name.
        """
        escaped = json.dumps(property_name)
        self._eval(
            f"window.__auroraview_result = getComputedStyle(el).getPropertyValue({escaped});"
        )

    def set_style(self, property_name: str, value: str) -> None:
        """Set inline style property.

        Args:
            property_name: CSS property name.
            value: CSS property value.
        """
        prop_escaped = json.dumps(property_name)
        val_escaped = json.dumps(value)
        self._eval(f"el.style.setProperty({prop_escaped}, {val_escaped});")

    def set_styles(self, styles: Dict[str, str]) -> None:
        """Set multiple inline styles.

        Args:
            styles: Dictionary of CSS property-value pairs.
        """
        for prop, val in styles.items():
            self.set_style(prop, val)

    # === Visibility ===

    def show(self, display: str = "block") -> None:
        """Show element.

        Args:
            display: Display value to use (default: "block").
        """
        escaped = json.dumps(display)
        self._eval(f"el.style.display = {escaped};")

    def hide(self) -> None:
        """Hide element (display: none)."""
        self._eval("el.style.display = 'none';")

    def is_visible(self) -> None:
        """Check if element is visible (async via bridge)."""
        self._eval(
            "window.__auroraview_result = !!(el.offsetWidth || el.offsetHeight || "
            "el.getClientRects().length);"
        )

    # === Basic Query ===

    def query(self, selector: str) -> "Element":
        """Query single child element.

        Args:
            selector: CSS selector for child element.

        Returns:
            Element wrapper for the child.
        """
        # Combine selectors for nested query
        combined = f"{self._selector} {selector}"
        return Element(self._webview, combined)

    def query_all(self, selector: str) -> "ElementCollection":
        """Query all matching child elements.

        Args:
            selector: CSS selector for child elements.

        Returns:
            ElementCollection for iteration.
        """
        combined = f"{self._selector} {selector}"
        return ElementCollection(self._webview, combined)

    def exists(self) -> None:
        """Check if element exists in DOM (async via bridge)."""
        self._eval("window.__auroraview_result = true;")
        # If element doesn't exist, the callback won't be executed
        # and result will remain undefined

    def count(self) -> None:
        """Count matching elements (async via bridge)."""
        selector_escaped = json.dumps(self._selector)
        script = f"""
        (function() {{
            window.__auroraview_result = document.querySelectorAll({selector_escaped}).length;
        }})();
        """
        self._webview.eval_js(script)

    # === Element Info ===

    def get_tag_name(self) -> None:
        """Get element's tag name (async via bridge)."""
        self._eval("window.__auroraview_result = el.tagName.toLowerCase();")

    def get_bounding_rect(self) -> None:
        """Get element's bounding rectangle (async via bridge)."""
        self._eval(
            "const r = el.getBoundingClientRect(); "
            "window.__auroraview_result = {x: r.x, y: r.y, width: r.width, "
            "height: r.height, top: r.top, right: r.right, bottom: r.bottom, left: r.left};"
        )

    def __repr__(self) -> str:
        """Return string representation."""
        return f"Element({self._selector!r})"

    # === Form Values (Phase 2) ===

    def get_value(self) -> None:
        """Get input/textarea/select value (async via bridge)."""
        self._eval("window.__auroraview_result = el.value;")

    def set_value(self, value: str) -> None:
        """Set input/textarea/select value.

        Args:
            value: New value to set.
        """
        escaped = json.dumps(value)
        self._eval(f"el.value = {escaped}; el.dispatchEvent(new Event('input', {{bubbles: true}}));")

    def get_checked(self) -> None:
        """Get checkbox/radio checked state (async via bridge)."""
        self._eval("window.__auroraview_result = el.checked;")

    def set_checked(self, checked: bool) -> None:
        """Set checkbox/radio checked state.

        Args:
            checked: Whether to check the element.
        """
        self._eval(f"el.checked = {str(checked).lower()}; el.dispatchEvent(new Event('change', {{bubbles: true}}));")

    def is_disabled(self) -> None:
        """Check if form element is disabled (async via bridge)."""
        self._eval("window.__auroraview_result = el.disabled;")

    def set_disabled(self, disabled: bool) -> None:
        """Set disabled state.

        Args:
            disabled: Whether to disable the element.
        """
        self._eval(f"el.disabled = {str(disabled).lower()};")

    # === Select/Dropdown ===

    def get_selected_options(self) -> None:
        """Get selected option(s) from <select> (async via bridge)."""
        self._eval(
            "window.__auroraview_result = Array.from(el.selectedOptions).map(o => "
            "({value: o.value, text: o.text, index: o.index}));"
        )

    def select_option(self, value: str) -> None:
        """Select option by value.

        Args:
            value: Option value to select.
        """
        escaped = json.dumps(value)
        self._eval(f"el.value = {escaped}; el.dispatchEvent(new Event('change', {{bubbles: true}}));")

    def select_option_by_text(self, text: str) -> None:
        """Select option by visible text.

        Args:
            text: Option text to select.
        """
        escaped = json.dumps(text)
        self._eval(
            f"const opt = Array.from(el.options).find(o => o.text === {escaped}); "
            f"if (opt) {{ el.value = opt.value; el.dispatchEvent(new Event('change', {{bubbles: true}})); }}"
        )

    def select_option_by_index(self, index: int) -> None:
        """Select option by index.

        Args:
            index: Zero-based option index.
        """
        self._eval(f"el.selectedIndex = {index}; el.dispatchEvent(new Event('change', {{bubbles: true}}));")

    # === User Interactions ===

    def click(self) -> None:
        """Simulate click event."""
        self._eval("el.click();")

    def double_click(self) -> None:
        """Simulate double-click event."""
        self._eval("el.dispatchEvent(new MouseEvent('dblclick', {bubbles: true, cancelable: true}));")

    def focus(self) -> None:
        """Focus the element."""
        self._eval("el.focus();")

    def blur(self) -> None:
        """Remove focus from element."""
        self._eval("el.blur();")

    def scroll_into_view(self, behavior: str = "smooth", block: str = "center") -> None:
        """Scroll element into viewport.

        Args:
            behavior: Scroll behavior ("smooth" or "instant").
            block: Vertical alignment ("start", "center", "end", "nearest").
        """
        behavior_escaped = json.dumps(behavior)
        block_escaped = json.dumps(block)
        self._eval(f"el.scrollIntoView({{behavior: {behavior_escaped}, block: {block_escaped}}});")

    def hover(self) -> None:
        """Simulate mouse hover (mouseenter event)."""
        self._eval("el.dispatchEvent(new MouseEvent('mouseenter', {bubbles: true}));")

    # === Type & Submit ===

    def type_text(self, text: str, clear_first: bool = False) -> None:
        """Type text into input (simulates keystrokes).

        Args:
            text: Text to type.
            clear_first: Whether to clear existing value first.
        """
        escaped = json.dumps(text)
        if clear_first:
            self._eval(
                f"el.value = ''; el.value = {escaped}; "
                f"el.dispatchEvent(new Event('input', {{bubbles: true}}));"
            )
        else:
            self._eval(
                f"el.value += {escaped}; "
                f"el.dispatchEvent(new Event('input', {{bubbles: true}}));"
            )

    def clear(self) -> None:
        """Clear input/textarea content."""
        self._eval("el.value = ''; el.dispatchEvent(new Event('input', {bubbles: true}));")

    def submit(self) -> None:
        """Submit the parent form."""
        self._eval(
            "const form = el.closest('form'); "
            "if (form) { form.dispatchEvent(new Event('submit', {bubbles: true, cancelable: true})); }"
        )

    # === Traversal (Phase 3) ===

    def parent(self) -> "Element":
        """Get parent element.

        Returns:
            Element wrapper for parent (uses :has() selector).
        """
        # Use a unique approach: create selector for parent
        return Element(self._webview, f":has(> {self._selector})")

    def closest(self, ancestor_selector: str) -> "Element":
        """Find closest ancestor matching selector.

        Args:
            ancestor_selector: CSS selector for ancestor.

        Returns:
            Element wrapper for the ancestor.
        """
        # Combined selector approach
        return Element(self._webview, f"{ancestor_selector}:has({self._selector})")

    def first_child(self) -> "Element":
        """Get first child element.

        Returns:
            Element wrapper for first child.
        """
        return Element(self._webview, f"{self._selector} > :first-child")

    def last_child(self) -> "Element":
        """Get last child element.

        Returns:
            Element wrapper for last child.
        """
        return Element(self._webview, f"{self._selector} > :last-child")

    def nth_child(self, n: int) -> "Element":
        """Get nth child element (1-based).

        Args:
            n: Child index (1-based, like CSS).

        Returns:
            Element wrapper for nth child.
        """
        return Element(self._webview, f"{self._selector} > :nth-child({n})")

    def next_sibling(self) -> "Element":
        """Get next sibling element.

        Returns:
            Element wrapper for next sibling.
        """
        return Element(self._webview, f"{self._selector} + *")

    def prev_sibling(self) -> "Element":
        """Get previous sibling element.

        Note: CSS has limited support for previous siblings.
        This uses :has() which may not work in all browsers.

        Returns:
            Element wrapper for previous sibling.
        """
        return Element(self._webview, f":has(+ {self._selector})")

    def children(self) -> "ElementCollection":
        """Get all direct child elements.

        Returns:
            ElementCollection for child elements.
        """
        return ElementCollection(self._webview, f"{self._selector} > *")

    def siblings(self) -> "ElementCollection":
        """Get all sibling elements.

        Returns:
            ElementCollection for sibling elements.

        Note: This returns siblings but may include the element itself
        in some edge cases due to CSS selector limitations.
        """
        # Use parent > all children approach
        return ElementCollection(self._webview, f":has(> {self._selector}) > *")

    # === DOM Manipulation ===

    def append_html(self, html: str) -> None:
        """Append HTML content inside element.

        Args:
            html: HTML string to append.
        """
        escaped = json.dumps(html)
        self._eval(f"el.insertAdjacentHTML('beforeend', {escaped});")

    def prepend_html(self, html: str) -> None:
        """Prepend HTML content inside element.

        Args:
            html: HTML string to prepend.
        """
        escaped = json.dumps(html)
        self._eval(f"el.insertAdjacentHTML('afterbegin', {escaped});")

    def insert_before(self, html: str) -> None:
        """Insert HTML before element.

        Args:
            html: HTML string to insert.
        """
        escaped = json.dumps(html)
        self._eval(f"el.insertAdjacentHTML('beforebegin', {escaped});")

    def insert_after(self, html: str) -> None:
        """Insert HTML after element.

        Args:
            html: HTML string to insert.
        """
        escaped = json.dumps(html)
        self._eval(f"el.insertAdjacentHTML('afterend', {escaped});")

    def remove(self) -> None:
        """Remove element from DOM."""
        self._eval("el.remove();")

    def replace_with(self, html: str) -> None:
        """Replace element with new HTML.

        Args:
            html: HTML string to replace with.
        """
        escaped = json.dumps(html)
        self._eval(f"el.outerHTML = {escaped};")

    def empty(self) -> None:
        """Remove all child elements."""
        self._eval("el.innerHTML = '';")


class ElementCollection:
    """Collection of DOM Elements for batch operations.

    Provides iteration and batch operations on multiple elements
    matching a selector.

    Example:
        >>> items = webview.dom_all(".list-item")
        >>> for item in items:
        ...     item.add_class("processed")
    """

    __slots__ = ("_webview", "_selector")

    def __init__(self, webview: "WebView", selector: str) -> None:
        """Initialize ElementCollection.

        Args:
            webview: Parent WebView instance.
            selector: CSS selector for elements.
        """
        self._webview = webview
        self._selector = selector

    @property
    def selector(self) -> str:
        """Get the CSS selector for this collection."""
        return self._selector

    def _escape_selector(self) -> str:
        """Escape selector for JavaScript string."""
        return self._selector.replace("\\", "\\\\").replace('"', '\\"')

    def first(self) -> Element:
        """Get first matching element.

        Returns:
            Element wrapper for the first match.
        """
        return Element(self._webview, self._selector)

    def nth(self, index: int) -> Element:
        """Get element at specific index.

        Args:
            index: Zero-based index.

        Returns:
            Element wrapper using :nth-child selector.
        """
        return Element(self._webview, f"{self._selector}:nth-child({index + 1})")

    def count(self) -> None:
        """Count matching elements (async via bridge)."""
        selector_escaped = json.dumps(self._selector)
        script = f"""
        (function() {{
            window.__auroraview_result = document.querySelectorAll({selector_escaped}).length;
        }})();
        """
        self._webview.eval_js(script)

    # === Batch Operations ===

    def add_class(self, *class_names: str) -> None:
        """Add CSS class(es) to all matching elements.

        Args:
            *class_names: One or more class names to add.
        """
        classes = ", ".join(json.dumps(c) for c in class_names)
        selector_escaped = json.dumps(self._selector)
        script = f"""
        (function() {{
            document.querySelectorAll({selector_escaped}).forEach(el => {{
                el.classList.add({classes});
            }});
        }})();
        """
        self._webview.eval_js(script)

    def remove_class(self, *class_names: str) -> None:
        """Remove CSS class(es) from all matching elements.

        Args:
            *class_names: One or more class names to remove.
        """
        classes = ", ".join(json.dumps(c) for c in class_names)
        selector_escaped = json.dumps(self._selector)
        script = f"""
        (function() {{
            document.querySelectorAll({selector_escaped}).forEach(el => {{
                el.classList.remove({classes});
            }});
        }})();
        """
        self._webview.eval_js(script)

    def set_style(self, property_name: str, value: str) -> None:
        """Set inline style on all matching elements.

        Args:
            property_name: CSS property name.
            value: CSS property value.
        """
        selector_escaped = json.dumps(self._selector)
        prop_escaped = json.dumps(property_name)
        val_escaped = json.dumps(value)
        script = f"""
        (function() {{
            document.querySelectorAll({selector_escaped}).forEach(el => {{
                el.style.setProperty({prop_escaped}, {val_escaped});
            }});
        }})();
        """
        self._webview.eval_js(script)

    def hide(self) -> None:
        """Hide all matching elements."""
        self.set_style("display", "none")

    def show(self, display: str = "block") -> None:
        """Show all matching elements.

        Args:
            display: Display value to use (default: "block").
        """
        self.set_style("display", display)

    def __repr__(self) -> str:
        """Return string representation."""
        return f"ElementCollection({self._selector!r})"
