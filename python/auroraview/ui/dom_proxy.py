"""DOM Proxy classes for Pythonic DOM manipulation.

This module provides proxy classes that enable intuitive DOM manipulation
with Pythonic syntax like:

    element.style['background-color'] = 'red'
    element.classes.toggle('active')
    element.attributes['data-id'] = '123'

Example:
    >>> from auroraview.integration.qt import QtWebView
    >>>
    >>> webview = QtWebView(parent=maya_main_window())
    >>> webview.load_html('<div id="box" class="container">Hello</div>')
    >>>
    >>> # Get element with enhanced API
    >>> box = webview.dom("#box")
    >>>
    >>> # Style manipulation (pywebview-style)
    >>> box.style['background-color'] = 'blue'
    >>> box.style['font-size'] = '16px'
    >>>
    >>> # Class manipulation
    >>> box.classes.add('active', 'highlighted')
    >>> box.classes.toggle('visible')
    >>> 'active' in box.classes  # Check if class exists
    >>>
    >>> # Attribute manipulation
    >>> box.attributes['data-id'] = '123'
    >>> box.attributes['title'] = 'My Box'
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from auroraview.ui.dom import Element

logger = logging.getLogger(__name__)

__all__ = ["StyleProxy", "ClassListProxy", "AttributeProxy"]


class StyleProxy:
    """Proxy for element.style manipulation with dict-like syntax.

    Example:
        element.style['background-color'] = 'red'
        element.style['font-size'] = '16px'

    Note: Getting style values is not synchronous due to WebView architecture.
    Use element.get_style() for reading computed styles.
    """

    __slots__ = ("_element",)

    def __init__(self, element: "Element") -> None:
        self._element = element

    def __setitem__(self, prop: str, value: str) -> None:
        """Set a CSS style property.

        Args:
            prop: CSS property name (e.g., 'background-color', 'fontSize')
            value: CSS value (e.g., 'red', '16px')
        """
        self._element.set_style(prop, value)

    def __delitem__(self, prop: str) -> None:
        """Remove a CSS style property.

        Args:
            prop: CSS property name to remove
        """
        self._element.set_style(prop, "")

    def update(self, styles: dict) -> None:
        """Update multiple styles at once.

        Args:
            styles: Dictionary of CSS properties and values

        Example:
            >>> element.style.update({
            ...     'background-color': 'blue',
            ...     'font-size': '14px',
            ...     'padding': '10px'
            ... })
        """
        self._element.set_styles(styles)

    def __repr__(self) -> str:
        return f"StyleProxy({self._element.selector!r})"


class ClassListProxy:
    """Proxy for element.classList manipulation with set-like syntax.

    Example:
        element.classes.add('active')
        element.classes.remove('hidden')
        element.classes.toggle('visible')
        'active' in element.classes
    """

    __slots__ = ("_element",)

    def __init__(self, element: "Element") -> None:
        self._element = element

    def add(self, *class_names: str) -> None:
        """Add one or more classes.

        Args:
            *class_names: Class names to add
        """
        self._element.add_class(*class_names)

    def remove(self, *class_names: str) -> None:
        """Remove one or more classes.

        Args:
            *class_names: Class names to remove
        """
        self._element.remove_class(*class_names)

    def toggle(self, class_name: str, force: Optional[bool] = None) -> None:
        """Toggle a class.

        Args:
            class_name: Class name to toggle
            force: If True, add the class; if False, remove it
        """
        self._element.toggle_class(class_name, force)

    def replace(self, old_class: str, new_class: str) -> None:
        """Replace one class with another.

        Args:
            old_class: Class to remove
            new_class: Class to add
        """
        self._element.remove_class(old_class)
        self._element.add_class(new_class)

    def __contains__(self, class_name: str) -> None:
        """Check if element has a class (async - stores result).

        Note: Due to WebView architecture, this triggers an async check.
        Result is stored in window.__auroraview_result.
        """
        self._element.has_class(class_name)

    def __repr__(self) -> str:
        return f"ClassListProxy({self._element.selector!r})"


class AttributeProxy:
    """Proxy for element attribute manipulation with dict-like syntax.

    Example:
        element.attributes['data-id'] = '123'
        del element.attributes['disabled']
    """

    __slots__ = ("_element",)

    def __init__(self, element: "Element") -> None:
        self._element = element

    def __setitem__(self, name: str, value: str) -> None:
        """Set an attribute.

        Args:
            name: Attribute name
            value: Attribute value
        """
        self._element.set_attribute(name, value)

    def __delitem__(self, name: str) -> None:
        """Remove an attribute.

        Args:
            name: Attribute name to remove
        """
        self._element.remove_attribute(name)

    def __contains__(self, name: str) -> None:
        """Check if attribute exists (async - stores result).

        Note: Due to WebView architecture, this triggers an async check.
        Result is stored in window.__auroraview_result.
        """
        self._element.has_attribute(name)

    def get(self, name: str, default: Optional[str] = None) -> None:
        """Get an attribute value (async - stores result).

        Note: Due to WebView architecture, this triggers an async get.
        Result is stored in window.__auroraview_result.
        """
        self._element.get_attribute(name)

    def update(self, attrs: dict) -> None:
        """Set multiple attributes at once.

        Args:
            attrs: Dictionary of attribute names and values
        """
        for name, value in attrs.items():
            self._element.set_attribute(name, value)

    def __repr__(self) -> str:
        return f"AttributeProxy({self._element.selector!r})"


class DataProxy:
    """Proxy for element.dataset manipulation with dict-like syntax.

    Provides access to data-* attributes:
        element.data['id'] = '123'  # Sets data-id="123"
        element.data['user-name'] = 'John'  # Sets data-user-name="John"
    """

    __slots__ = ("_element",)

    def __init__(self, element: "Element") -> None:
        self._element = element

    def __setitem__(self, name: str, value: str) -> None:
        """Set a data attribute.

        Args:
            name: Data attribute name (without 'data-' prefix)
            value: Attribute value
        """
        self._element.set_attribute(f"data-{name}", value)

    def __delitem__(self, name: str) -> None:
        """Remove a data attribute.

        Args:
            name: Data attribute name (without 'data-' prefix)
        """
        self._element.remove_attribute(f"data-{name}")

    def __repr__(self) -> str:
        return f"DataProxy({self._element.selector!r})"
