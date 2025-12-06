# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView DOM Manipulation Mixin.

This module provides DOM manipulation methods for the WebView class.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from auroraview.ui.dom import Element, ElementCollection


class WebViewDOMMixin:
    """Mixin providing DOM manipulation methods.

    Provides methods for DOM access:
    - dom: Get a DOM Element by CSS selector
    - dom_all: Get all DOM Elements matching a selector
    - dom_by_id: Get a DOM Element by ID
    - dom_by_class: Get all DOM Elements by class name
    """

    def dom(self, selector: str) -> "Element":
        """Get a DOM Element by CSS selector.

        This provides a high-level interface for DOM manipulation
        with intuitive Pythonic syntax.

        Args:
            selector: CSS selector for the element.

        Returns:
            Element wrapper for DOM manipulation.

        Example:
            >>> # Basic element access
            >>> button = webview.dom("#submit-btn")
            >>> button.set_text("Submit")
            >>> button.add_class("active")

            >>> # Chained operations
            >>> webview.dom("#title").set_html("<b>Hello</b>")

            >>> # Nested queries
            >>> form = webview.dom("#login-form")
            >>> form.query("#username").set_value("admin")
        """
        from auroraview.ui.dom import Element

        return Element(self, selector)  # type: ignore[arg-type]

    def dom_all(self, selector: str) -> "ElementCollection":
        """Get all DOM Elements matching a CSS selector.

        Args:
            selector: CSS selector for elements.

        Returns:
            ElementCollection for batch operations.

        Example:
            >>> # Batch class operations
            >>> webview.dom_all(".item").add_class("processed")

            >>> # Access specific elements
            >>> items = webview.dom_all(".list-item")
            >>> items.first().set_text("First item")
            >>> items.nth(2).add_class("selected")
        """
        from auroraview.ui.dom import ElementCollection

        return ElementCollection(self, selector)  # type: ignore[arg-type]

    def dom_by_id(self, element_id: str) -> "Element":
        """Get a DOM Element by ID.

        Shortcut for dom("#id").

        Args:
            element_id: Element ID (without # prefix).

        Returns:
            Element wrapper for DOM manipulation.

        Example:
            >>> webview.dom_by_id("title").set_text("Hello")
        """
        from auroraview.ui.dom import Element

        return Element(self, f"#{element_id}")  # type: ignore[arg-type]

    def dom_by_class(self, class_name: str) -> "ElementCollection":
        """Get all DOM Elements by class name.

        Shortcut for dom_all(".class").

        Args:
            class_name: Class name (without . prefix).

        Returns:
            ElementCollection for batch operations.

        Example:
            >>> webview.dom_by_class("button").add_class("styled")
        """
        from auroraview.ui.dom import ElementCollection

        return ElementCollection(self, f".{class_name}")  # type: ignore[arg-type]
