"""Native menu bar support for AuroraView.

This module provides cross-platform menu bar functionality with keyboard shortcuts.

Example:
    >>> from auroraview.menu import MenuBar, Menu, MenuItem
    >>>
    >>> # Create a file menu
    >>> file_menu = Menu("File").add_items([
    ...     MenuItem.action("New", "file.new", "Ctrl+N"),
    ...     MenuItem.action("Open", "file.open", "Ctrl+O"),
    ...     MenuItem.separator(),
    ...     MenuItem.action("Exit", "file.exit", "Alt+F4"),
    ... ])
    >>>
    >>> # Create menu bar
    >>> menu_bar = MenuBar().add_menu(file_menu)
    >>>
    >>> # Create WebView with menu
    >>> webview = WebView.create("My App", menu=menu_bar)
    >>>
    >>> # Handle menu actions
    >>> @webview.on("menu_action")
    >>> def on_menu(data):
    ...     print(f"Menu action: {data['action_id']}")
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional


class MenuItemType(Enum):
    """Menu item type."""

    ACTION = "action"
    CHECKBOX = "checkbox"
    RADIO = "radio"
    SEPARATOR = "separator"
    SUBMENU = "submenu"


@dataclass
class MenuItem:
    """A single menu item.

    Attributes:
        label: Item label (use & for mnemonic, e.g., "&File")
        action_id: Action identifier for event handling
        item_type: Type of menu item
        accelerator: Keyboard shortcut (e.g., "Ctrl+N")
        enabled: Whether item is enabled
        checked: Whether item is checked (for checkbox/radio)
        children: Submenu items
    """

    label: str = ""
    action_id: Optional[str] = None
    item_type: MenuItemType = MenuItemType.ACTION
    accelerator: Optional[str] = None
    enabled: bool = True
    checked: bool = False
    children: List["MenuItem"] = field(default_factory=list)

    @classmethod
    def action(
        cls,
        label: str,
        action_id: str,
        accelerator: Optional[str] = None,
        enabled: bool = True,
    ) -> "MenuItem":
        """Create an action menu item."""
        return cls(
            label=label,
            action_id=action_id,
            item_type=MenuItemType.ACTION,
            accelerator=accelerator,
            enabled=enabled,
        )

    @classmethod
    def checkbox(
        cls,
        label: str,
        action_id: str,
        checked: bool = False,
        accelerator: Optional[str] = None,
        enabled: bool = True,
    ) -> "MenuItem":
        """Create a checkbox menu item."""
        return cls(
            label=label,
            action_id=action_id,
            item_type=MenuItemType.CHECKBOX,
            accelerator=accelerator,
            enabled=enabled,
            checked=checked,
        )

    @classmethod
    def separator(cls) -> "MenuItem":
        """Create a separator."""
        return cls(item_type=MenuItemType.SEPARATOR)

    @classmethod
    def submenu(cls, label: str, children: List["MenuItem"]) -> "MenuItem":
        """Create a submenu."""
        return cls(
            label=label,
            item_type=MenuItemType.SUBMENU,
            children=children,
        )

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        result = {
            "label": self.label,
            "item_type": self.item_type.value,
            "enabled": self.enabled,
        }
        if self.action_id:
            result["action_id"] = self.action_id
        if self.accelerator:
            result["accelerator"] = self.accelerator
        if self.item_type in (MenuItemType.CHECKBOX, MenuItemType.RADIO):
            result["checked"] = self.checked
        if self.children:
            result["children"] = [c.to_dict() for c in self.children]
        return result


@dataclass
class Menu:
    """A menu (dropdown from menu bar).

    Attributes:
        label: Menu label (e.g., "File", "Edit")
        items: Menu items
        enabled: Whether menu is enabled
    """

    label: str
    items: List[MenuItem] = field(default_factory=list)
    enabled: bool = True

    def add_item(self, item: MenuItem) -> "Menu":
        """Add an item to the menu."""
        self.items.append(item)
        return self

    def add_items(self, items: List[MenuItem]) -> "Menu":
        """Add multiple items to the menu."""
        self.items.extend(items)
        return self

    def add_separator(self) -> "Menu":
        """Add a separator."""
        return self.add_item(MenuItem.separator())

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "label": self.label,
            "items": [item.to_dict() for item in self.items],
            "enabled": self.enabled,
        }


@dataclass
class MenuBar:
    """Menu bar (top-level menu container).

    Attributes:
        menus: Top-level menus
    """

    menus: List[Menu] = field(default_factory=list)

    def add_menu(self, menu: Menu) -> "MenuBar":
        """Add a menu to the bar."""
        self.menus.append(menu)
        return self

    def add_menus(self, menus: List[Menu]) -> "MenuBar":
        """Add multiple menus."""
        self.menus.extend(menus)
        return self

    @classmethod
    def with_standard_menus(cls, app_name: str = "AuroraView") -> "MenuBar":
        """Create a menu bar with standard File, Edit, View, Help menus."""
        bar = cls()

        # File menu
        file_menu = Menu("&File")
        file_menu.add_items(
            [
                MenuItem.action("&New", "file.new", "Ctrl+N"),
                MenuItem.action("&Open...", "file.open", "Ctrl+O"),
                MenuItem.action("&Save", "file.save", "Ctrl+S"),
                MenuItem.action("Save &As...", "file.save_as", "Ctrl+Shift+S"),
                MenuItem.separator(),
                MenuItem.action("E&xit", "file.exit", "Alt+F4"),
            ]
        )
        bar.add_menu(file_menu)

        # Edit menu
        edit_menu = Menu("&Edit")
        edit_menu.add_items(
            [
                MenuItem.action("&Undo", "edit.undo", "Ctrl+Z"),
                MenuItem.action("&Redo", "edit.redo", "Ctrl+Y"),
                MenuItem.separator(),
                MenuItem.action("Cu&t", "edit.cut", "Ctrl+X"),
                MenuItem.action("&Copy", "edit.copy", "Ctrl+C"),
                MenuItem.action("&Paste", "edit.paste", "Ctrl+V"),
                MenuItem.separator(),
                MenuItem.action("Select &All", "edit.select_all", "Ctrl+A"),
            ]
        )
        bar.add_menu(edit_menu)

        # View menu
        view_menu = Menu("&View")
        view_menu.add_items(
            [
                MenuItem.checkbox("Show &Toolbar", "view.toolbar", checked=True),
                MenuItem.checkbox("Show &Sidebar", "view.sidebar", checked=True),
                MenuItem.separator(),
                MenuItem.action("&Zoom In", "view.zoom_in", "Ctrl++"),
                MenuItem.action("Zoom &Out", "view.zoom_out", "Ctrl+-"),
                MenuItem.action("&Reset Zoom", "view.zoom_reset", "Ctrl+0"),
            ]
        )
        bar.add_menu(view_menu)

        # Help menu
        help_menu = Menu("&Help")
        help_menu.add_items(
            [
                MenuItem.action("&Documentation", "help.docs", "F1"),
                MenuItem.action("&Check for Updates", "help.updates"),
                MenuItem.separator(),
                MenuItem.action(f"&About {app_name}", "help.about"),
            ]
        )
        bar.add_menu(help_menu)

        return bar

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "menus": [menu.to_dict() for menu in self.menus],
        }
