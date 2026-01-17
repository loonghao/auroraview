# -*- coding: utf-8 -*-
"""Integration tests for Browser API.

These tests verify the Browser class works correctly
with TabContainer and WindowManager in realistic scenarios.
"""

from __future__ import annotations

from typing import Any, List
from unittest.mock import MagicMock, patch


class TestBrowserTabManagement:
    """Integration tests for Browser tab management."""

    def test_new_tab_creates_correct_structure(self):
        """Test new_tab creates proper tab structure."""
        from auroraview.browser.browser import Browser

        browser = Browser(default_url="https://example.com")

        tab = browser.new_tab("https://github.com", "GitHub")

        assert "id" in tab
        assert tab["url"] == "https://github.com"
        assert tab["title"] == "GitHub"
        assert tab["loading"] is False
        assert tab["canGoBack"] is False
        assert tab["canGoForward"] is False

    def test_first_tab_is_active(self):
        """Test first tab is automatically activated."""
        from auroraview.browser.browser import Browser

        browser = Browser()

        tab1 = browser.new_tab("https://example.com")
        browser.new_tab("https://github.com")  # Create second tab

        assert browser.get_active_tab()["id"] == tab1["id"]

    def test_close_active_tab_activates_next(self):
        """Test closing active tab activates next available."""
        from auroraview.browser.browser import Browser

        browser = Browser()

        tab1 = browser.new_tab("https://example.com", "Tab 1")
        browser.new_tab("https://github.com", "Tab 2")
        browser.new_tab("https://google.com", "Tab 3")

        # Close active (tab1)
        browser.close_tab(tab1["id"])

        # Should activate another tab
        assert browser.get_active_tab() is not None
        assert len(browser.get_tabs()) == 2

    def test_close_last_tab(self):
        """Test closing the last tab."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        tab = browser.new_tab("https://example.com")

        browser.close_tab(tab["id"])

        assert len(browser.get_tabs()) == 0
        assert browser.get_active_tab() is None

    def test_navigate_updates_tab_url(self):
        """Test navigate updates the active tab URL."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser.new_tab("https://example.com", "Example")

        browser.navigate("https://github.com")

        active = browser.get_active_tab()
        assert active["url"] == "https://github.com"
        assert active["loading"] is True

    def test_navigate_specific_tab(self):
        """Test navigating a specific tab by ID."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser.new_tab("https://example.com", "Tab 1")
        tab2 = browser.new_tab("https://github.com", "Tab 2")

        browser.navigate("https://google.com", tab_id=tab2["id"])

        # tab2 should be updated
        tabs = browser.get_tabs()
        tab2_updated = next(t for t in tabs if t["id"] == tab2["id"])
        assert tab2_updated["url"] == "https://google.com"

    def test_activate_tab(self):
        """Test activating a tab."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser.new_tab("https://example.com", "Tab 1")
        tab2 = browser.new_tab("https://github.com", "Tab 2")

        browser.activate_tab(tab2["id"])

        assert browser.get_active_tab()["id"] == tab2["id"]


class TestBrowserDefaultUrl:
    """Tests for Browser default URL behavior."""

    def test_default_url_used_for_empty_new_tab(self):
        """Test default URL is used when no URL specified."""
        from auroraview.browser.browser import Browser

        browser = Browser(default_url="https://start.page.com")

        tab = browser.new_tab()

        assert tab["url"] == "https://start.page.com"

    def test_explicit_url_overrides_default(self):
        """Test explicit URL overrides default."""
        from auroraview.browser.browser import Browser

        browser = Browser(default_url="https://start.page.com")

        tab = browser.new_tab("https://custom.url.com")

        assert tab["url"] == "https://custom.url.com"


class TestBrowserCallbacks:
    """Tests for Browser callback system."""

    def test_on_ready_called_after_show(self):
        """Test on_ready callback is called after show."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        ready_results: List[Browser] = []

        def on_ready(b):
            ready_results.append(b)

        browser.on_ready(on_ready)

        # Simulate creating webview and marking as running
        with patch.object(browser, "_create_webview"):
            with patch.object(browser, "_webview", MagicMock()):
                browser._running = True
                browser._webview.show = MagicMock()

                # Manually trigger ready callbacks
                for cb in browser._on_ready_callbacks:
                    cb(browser)

        assert len(ready_results) == 1
        assert ready_results[0] is browser

    def test_on_ready_immediate_if_already_running(self):
        """Test on_ready callback runs immediately if browser already running."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser._running = True

        ready_called = [False]

        def on_ready(b):
            ready_called[0] = True

        browser.on_ready(on_ready)

        assert ready_called[0] is True


class TestBrowserApiSetup:
    """Tests for Browser API bindings."""

    def test_api_methods_bound(self):
        """Test all expected API methods are bound."""
        from auroraview.browser.browser import Browser

        browser = Browser()

        # Mock webview with bind_call tracking
        mock_webview = MagicMock()
        bound_methods: List[str] = []

        def mock_bind_call(name):
            def decorator(func):
                bound_methods.append(name)
                return func

            return decorator

        mock_webview.bind_call = mock_bind_call
        browser._webview = mock_webview

        browser._setup_api()

        expected_methods = [
            "browser.new_tab",
            "browser.close_tab",
            "browser.activate_tab",
            "browser.navigate",
            "browser.go_back",
            "browser.go_forward",
            "browser.reload",
            "browser.get_tabs",
            "browser.get_state",
        ]

        for method in expected_methods:
            assert method in bound_methods, f"Missing API binding: {method}"


class TestBrowserHtmlGeneration:
    """Tests for Browser HTML generation."""

    def test_html_contains_initial_tabs(self):
        """Test generated HTML includes initial tabs."""
        from auroraview.browser.browser import Browser

        browser = Browser(title="Test Browser")
        browser.new_tab("https://example.com", "Example")

        html = browser._get_browser_html()

        assert "Test Browser" in html
        assert "https://example.com" in html

    def test_html_has_required_elements(self):
        """Test generated HTML has all required UI elements."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        html = browser._get_browser_html()

        # Tab bar
        assert 'class="tab-bar"' in html
        assert 'id="tabBar"' in html

        # Navigation buttons
        assert 'id="backBtn"' in html
        assert 'id="forwardBtn"' in html
        assert 'id="reloadBtn"' in html

        # URL bar
        assert 'id="urlBar"' in html
        assert 'class="url-bar"' in html

        # Content frame
        assert 'id="contentFrame"' in html
        assert "<iframe" in html


class TestBrowserStateSync:
    """Tests for Browser state synchronization."""

    def test_sync_tabs_emits_event(self):
        """Test _sync_tabs_to_ui emits browser:tabs_update event."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser._webview = MagicMock()

        browser.new_tab("https://example.com", "Tab 1")
        browser.new_tab("https://github.com", "Tab 2")

        browser._sync_tabs_to_ui()

        browser._webview.emit.assert_called_with(
            "browser:tabs_update",
            {
                "tabs": browser._tabs,
                "activeTabId": browser._active_tab_id,
            },
        )

    def test_tab_operations_trigger_sync(self):
        """Test tab operations trigger UI sync."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser._webview = MagicMock()

        # new_tab should sync
        browser.new_tab("https://example.com")
        assert browser._webview.emit.call_count >= 1

        browser._webview.emit.reset_mock()

        # close_tab should sync
        tab = browser.new_tab("https://github.com")
        browser._webview.emit.reset_mock()
        browser.close_tab(tab["id"])
        assert browser._webview.emit.call_count >= 1


class TestBrowserWithTabContainer:
    """Tests for Browser integration with TabContainer."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        WindowManager._instance = None

    def test_tab_container_operations(self):
        """Test TabContainer operations."""
        from auroraview.browser.tab_container import TabContainer

        container = TabContainer(default_url="about:blank")

        with patch("auroraview.browser.tab_container.get_window_manager"):
            tab1 = container.create_tab(
                url="https://example.com", load_immediately=False
            )
            tab2 = container.create_tab(url="https://github.com", load_immediately=False)

        # Get all tabs
        all_tabs = container.get_all_tabs()
        assert len(all_tabs) == 2
        assert all_tabs[0].id == tab1.id
        assert all_tabs[1].id == tab2.id

        # Update tab
        container.update_tab(tab1.id, title="Updated Title")
        updated = container.get_tab(tab1.id)
        assert updated.title == "Updated Title"

    def test_tab_container_callbacks(self):
        """Test TabContainer callbacks fire correctly."""
        from auroraview.browser.tab_container import TabContainer

        tab_changes: List[Any] = []
        tabs_updates: List[int] = []

        container = TabContainer(
            on_tab_change=lambda t: tab_changes.append(t.id),
            on_tabs_update=lambda tabs: tabs_updates.append(len(tabs)),
            default_url="about:blank",
        )

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container.create_tab(url="https://example.com", load_immediately=False)

        # Should have received callbacks
        assert len(tab_changes) >= 1
        assert len(tabs_updates) >= 1

    def test_tab_navigation_without_webview(self):
        """Test navigation when WebView not loaded."""
        from auroraview.browser.tab_container import TabContainer

        container = TabContainer(default_url="about:blank")

        with patch("auroraview.browser.tab_container.get_window_manager"):
            tab = container.create_tab(
                url="https://example.com", load_immediately=False
            )

        # Navigate should update URL but not fail
        container.navigate("https://newurl.com")

        updated_tab = container.get_tab(tab.id)
        assert updated_tab.url == "https://newurl.com"


class TestBrowserEdgeCases:
    """Tests for Browser edge cases."""

    def test_operations_before_show(self):
        """Test tab operations work before show() is called."""
        from auroraview.browser.browser import Browser

        browser = Browser()

        # Should work without error
        tab1 = browser.new_tab("https://example.com")
        tab2 = browser.new_tab("https://github.com")
        browser.activate_tab(tab2["id"])
        browser.navigate("https://google.com")
        browser.close_tab(tab1["id"])

        assert len(browser.get_tabs()) == 1

    def test_close_nonexistent_tab(self):
        """Test closing nonexistent tab doesn't error."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser.new_tab("https://example.com")

        # Should not error
        browser.close_tab("nonexistent_tab_id")

        assert len(browser.get_tabs()) == 1

    def test_navigate_no_active_tab(self):
        """Test navigate with no active tab."""
        from auroraview.browser.browser import Browser

        browser = Browser()

        # Should not error
        browser.navigate("https://example.com")

    def test_empty_url_uses_default(self):
        """Test empty URL uses default."""
        from auroraview.browser.browser import Browser

        browser = Browser(default_url="https://default.com")

        tab = browser.new_tab("")

        assert tab["url"] == "https://default.com"


class TestBrowserNavigation:
    """Tests for Browser navigation methods."""

    def test_go_back_forward_reload(self):
        """Test navigation methods delegate to webview."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        browser._webview = MagicMock()

        browser.go_back()
        browser._webview.go_back.assert_called_once()

        browser.go_forward()
        browser._webview.go_forward.assert_called_once()

        browser.reload()
        browser._webview.reload.assert_called_once()

    def test_navigation_without_webview(self):
        """Test navigation methods handle missing webview."""
        from auroraview.browser.browser import Browser

        browser = Browser()

        # Should not error when _webview is None
        browser.go_back()
        browser.go_forward()
        browser.reload()
