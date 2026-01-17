# -*- coding: utf-8 -*-
"""Unit tests for TabContainer."""

from __future__ import annotations

from unittest.mock import MagicMock, patch


class TestTabState:
    """Tests for TabState dataclass."""

    def test_default_values(self):
        """Test default values of TabState."""
        from auroraview.browser.tab_container import TabState

        tab = TabState(id="test_id")

        assert tab.id == "test_id"
        assert tab.title == "New Tab"
        assert tab.url == ""
        assert tab.favicon == ""
        assert tab.is_loading is False
        assert tab.can_go_back is False
        assert tab.can_go_forward is False
        assert tab.webview_id is None
        assert tab.metadata == {}

    def test_to_dict(self):
        """Test TabState serialization."""
        from auroraview.browser.tab_container import TabState

        tab = TabState(
            id="tab_123",
            title="Test Tab",
            url="https://example.com",
            favicon="data:image/png;base64,...",
            is_loading=True,
            can_go_back=True,
            can_go_forward=False,
            metadata={"custom": "data"},
        )

        result = tab.to_dict()

        assert result == {
            "id": "tab_123",
            "title": "Test Tab",
            "url": "https://example.com",
            "favicon": "data:image/png;base64,...",
            "isLoading": True,
            "canGoBack": True,
            "canGoForward": False,
            "metadata": {"custom": "data"},
        }


class TestTabContainer:
    """Tests for TabContainer."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        WindowManager._instance = None

    def test_create_tab(self):
        """Test creating a tab."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            tab = container.create_tab(
                url="https://example.com",
                title="Test Tab",
                load_immediately=False,
            )

            assert tab.id.startswith("tab_")
            assert tab.url == "https://example.com"
            assert tab.title == "Test Tab"
            assert container.get_tab_count() == 1

    def test_create_tab_with_default_url(self):
        """Test creating tab with default URL."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer(default_url="https://default.com")
            tab = container.create_tab(load_immediately=False)

            assert tab.url == "https://default.com"

    def test_close_tab(self):
        """Test closing a tab."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            tab1 = container.create_tab(load_immediately=False)
            tab2 = container.create_tab(load_immediately=False)

            assert container.get_tab_count() == 2

            container.close_tab(tab1.id)

            assert container.get_tab_count() == 1
            assert container.get_tab(tab1.id) is None
            assert container.get_tab(tab2.id) is not None

    def test_activate_tab(self):
        """Test activating a tab."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            tab1 = container.create_tab(load_immediately=False)
            tab2 = container.create_tab(load_immediately=False)

            # tab2 should be active (last created with activate=True)
            assert container.get_active_tab_id() == tab2.id

            # Activate tab1
            result = container.activate_tab(tab1.id)

            assert result is True
            assert container.get_active_tab_id() == tab1.id

    def test_activate_nonexistent_tab(self):
        """Test activating a nonexistent tab."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            result = container.activate_tab("nonexistent")

            assert result is False

    def test_navigate(self):
        """Test navigating a tab."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            tab = container.create_tab(load_immediately=False)

            result = container.navigate("https://new-url.com")

            assert result is True
            assert tab.url == "https://new-url.com"
            assert tab.is_loading is True

    def test_get_all_tabs(self):
        """Test getting all tabs in order."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            container.create_tab(title="Tab 1", load_immediately=False)
            container.create_tab(title="Tab 2", load_immediately=False)
            container.create_tab(title="Tab 3", load_immediately=False)

            tabs = container.get_all_tabs()

            assert len(tabs) == 3
            assert tabs[0].title == "Tab 1"
            assert tabs[1].title == "Tab 2"
            assert tabs[2].title == "Tab 3"

    def test_update_tab(self):
        """Test updating tab properties."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            tab = container.create_tab(load_immediately=False)

            result = container.update_tab(
                tab.id,
                title="Updated Title",
                favicon="new-favicon.png",
            )

            assert result is True
            assert tab.title == "Updated Title"
            assert tab.favicon == "new-favicon.png"

    def test_on_tabs_update_callback(self):
        """Test tabs update callback."""
        from auroraview.browser.tab_container import TabContainer

        callback = MagicMock()

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer(on_tabs_update=callback)
            container.create_tab(load_immediately=False)

            assert callback.called
            tabs = callback.call_args[0][0]
            assert len(tabs) == 1

    def test_on_tab_change_callback(self):
        """Test tab change callback."""
        from auroraview.browser.tab_container import TabContainer

        callback = MagicMock()

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer(on_tab_change=callback)
            tab1 = container.create_tab(load_immediately=False)
            container.create_tab(load_immediately=False)  # Create second tab

            callback.reset_mock()
            container.activate_tab(tab1.id)

            callback.assert_called_once()
            assert callback.call_args[0][0].id == tab1.id

    def test_close_all(self):
        """Test closing all tabs."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            container.create_tab(load_immediately=False)
            container.create_tab(load_immediately=False)
            container.create_tab(load_immediately=False)

            assert container.get_tab_count() == 3

            container.close_all()

            assert container.get_tab_count() == 0

    def test_active_tab_after_close(self):
        """Test active tab selection after closing active tab."""
        from auroraview.browser.tab_container import TabContainer

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container = TabContainer()
            container.create_tab(load_immediately=False)
            tab2 = container.create_tab(load_immediately=False)
            tab3 = container.create_tab(load_immediately=False)

            # tab3 is active
            container.close_tab(tab3.id)

            # Should switch to last remaining tab
            assert container.get_active_tab_id() == tab2.id
