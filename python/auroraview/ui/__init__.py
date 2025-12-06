# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView UI Module.

This module contains UI-related functionality:
- DOM: DOM manipulation (Element, ElementCollection, Proxy classes)
- Menu: Native menu bar support (MenuBar, Menu, MenuItem)

Example:
    >>> from auroraview.ui import Element, MenuBar, Menu
    >>> menu_bar = MenuBar.with_standard_menus("My App")
    >>>
    >>> # DOM manipulation with proxy syntax
    >>> element = webview.dom("#box")
    >>> element.style['background'] = 'blue'
    >>> element.classes.add('active')
"""

from __future__ import annotations

from .dom import Element, ElementCollection
from .dom_proxy import StyleProxy, ClassListProxy, AttributeProxy, DataProxy
from .menu import Menu, MenuBar, MenuItem, MenuItemType

# Import submodules for attribute access
from . import dom as dom
from . import dom_proxy as dom_proxy
from . import menu as menu

__all__ = [
    # DOM
    "Element",
    "ElementCollection",
    # DOM Proxies
    "StyleProxy",
    "ClassListProxy",
    "AttributeProxy",
    "DataProxy",
    # Menu
    "MenuBar",
    "Menu",
    "MenuItem",
    "MenuItemType",
    # Submodules
    "dom",
    "dom_proxy",
    "menu",
]
