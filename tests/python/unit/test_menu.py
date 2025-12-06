"""Unit tests for the menu module."""

from auroraview.ui.menu import Menu, MenuBar, MenuItem, MenuItemType


class TestMenuItem:
    """Tests for MenuItem class."""

    def test_action_item(self):
        """Test creating an action menu item."""
        item = MenuItem.action("New", "file.new", "Ctrl+N")
        assert item.label == "New"
        assert item.action_id == "file.new"
        assert item.accelerator == "Ctrl+N"
        assert item.item_type == MenuItemType.ACTION
        assert item.enabled is True
        assert item.checked is False

    def test_action_item_no_accelerator(self):
        """Test action item without accelerator."""
        item = MenuItem.action("Exit", "file.exit")
        assert item.label == "Exit"
        assert item.action_id == "file.exit"
        assert item.accelerator is None

    def test_checkbox_item(self):
        """Test creating a checkbox menu item."""
        item = MenuItem.checkbox("Show Sidebar", "view.sidebar", checked=True)
        assert item.label == "Show Sidebar"
        assert item.action_id == "view.sidebar"
        assert item.item_type == MenuItemType.CHECKBOX
        assert item.checked is True

    def test_separator(self):
        """Test creating a separator."""
        item = MenuItem.separator()
        assert item.item_type == MenuItemType.SEPARATOR
        assert item.label == ""

    def test_submenu(self):
        """Test creating a submenu."""
        children = [
            MenuItem.action("Item 1", "sub.1"),
            MenuItem.action("Item 2", "sub.2"),
        ]
        item = MenuItem.submenu("Submenu", children)
        assert item.label == "Submenu"
        assert item.item_type == MenuItemType.SUBMENU
        assert len(item.children) == 2

    def test_to_dict(self):
        """Test converting menu item to dictionary."""
        item = MenuItem.action("New", "file.new", "Ctrl+N")
        d = item.to_dict()
        assert d["label"] == "New"
        assert d["action_id"] == "file.new"
        assert d["accelerator"] == "Ctrl+N"
        assert d["item_type"] == "action"
        assert d["enabled"] is True


class TestMenu:
    """Tests for Menu class."""

    def test_create_menu(self):
        """Test creating a menu."""
        menu = Menu("File")
        assert menu.label == "File"
        assert len(menu.items) == 0
        assert menu.enabled is True

    def test_add_item(self):
        """Test adding items to menu."""
        menu = Menu("File")
        menu.add_item(MenuItem.action("New", "file.new"))
        assert len(menu.items) == 1

    def test_add_items(self):
        """Test adding multiple items."""
        menu = Menu("File")
        menu.add_items(
            [
                MenuItem.action("New", "file.new"),
                MenuItem.action("Open", "file.open"),
            ]
        )
        assert len(menu.items) == 2

    def test_add_separator(self):
        """Test adding separator."""
        menu = Menu("File")
        menu.add_item(MenuItem.action("New", "file.new"))
        menu.add_separator()
        menu.add_item(MenuItem.action("Exit", "file.exit"))
        assert len(menu.items) == 3
        assert menu.items[1].item_type == MenuItemType.SEPARATOR

    def test_to_dict(self):
        """Test converting menu to dictionary."""
        menu = Menu("File")
        menu.add_item(MenuItem.action("New", "file.new"))
        d = menu.to_dict()
        assert d["label"] == "File"
        assert len(d["items"]) == 1


class TestMenuBar:
    """Tests for MenuBar class."""

    def test_create_menu_bar(self):
        """Test creating a menu bar."""
        bar = MenuBar()
        assert len(bar.menus) == 0

    def test_add_menu(self):
        """Test adding menu to bar."""
        bar = MenuBar()
        bar.add_menu(Menu("File"))
        assert len(bar.menus) == 1

    def test_add_menus(self):
        """Test adding multiple menus."""
        bar = MenuBar()
        bar.add_menus([Menu("File"), Menu("Edit")])
        assert len(bar.menus) == 2

    def test_with_standard_menus(self):
        """Test creating standard menus."""
        bar = MenuBar.with_standard_menus("TestApp")
        assert len(bar.menus) == 4  # File, Edit, View, Help
        assert bar.menus[0].label == "&File"
        assert bar.menus[1].label == "&Edit"
        assert bar.menus[2].label == "&View"
        assert bar.menus[3].label == "&Help"

    def test_to_dict(self):
        """Test converting menu bar to dictionary."""
        bar = MenuBar()
        bar.add_menu(Menu("File"))
        d = bar.to_dict()
        assert "menus" in d
        assert len(d["menus"]) == 1
