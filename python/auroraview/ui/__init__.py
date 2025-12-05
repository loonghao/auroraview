# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView UI Module.

This module contains UI-related functionality:
- DOM: DOM manipulation (Element, ElementCollection)
- Menu: Native menu bar support (MenuBar, Menu, MenuItem)

Example:
    >>> from auroraview.ui import Element, MenuBar, Menu
    >>> menu_bar = MenuBar.with_standard_menus("My App")
"""

from __future__ import annotations

from .dom import Element, ElementCollection
from .menu import Menu, MenuBar, MenuItem, MenuItemType

# Import submodules for attribute access
from . import dom as dom
from . import menu as menu

__all__ = [
    # DOM
    "Element",
    "ElementCollection",
    # Menu
    "MenuBar",
    "Menu",
    "MenuItem",
    "MenuItemType",
    # Submodules
    "dom",
    "menu",
]

