"""Unit tests for multi-tab browser demo.

Tests the tab management functionality without requiring a GUI.
"""

from __future__ import annotations

import threading
import uuid
from dataclasses import dataclass, field
from typing import Dict, List, Optional

import pytest


@dataclass
class Tab:
    """Represents a browser tab."""

    id: str
    title: str = "New Tab"
    url: str = ""
    favicon: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False


@dataclass
class BrowserState:
    """Manages browser state across all tabs."""

    tabs: Dict[str, Tab] = field(default_factory=dict)
    active_tab_id: Optional[str] = None
    tab_order: List[str] = field(default_factory=list)
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def add_tab(self, tab: Tab) -> None:
        """Add a new tab."""
        with self._lock:
            self.tabs[tab.id] = tab
            self.tab_order.append(tab.id)

    def remove_tab(self, tab_id: str) -> Optional[str]:
        """Remove a tab and return the next active tab ID."""
        with self._lock:
            if tab_id not in self.tabs:
                return self.active_tab_id

            idx = self.tab_order.index(tab_id)
            del self.tabs[tab_id]
            self.tab_order.remove(tab_id)

            if not self.tab_order:
                return None

            new_idx = min(idx, len(self.tab_order) - 1)
            return self.tab_order[new_idx]

    def get_tab(self, tab_id: str) -> Optional[Tab]:
        """Get a tab by ID."""
        with self._lock:
            return self.tabs.get(tab_id)

    def update_tab(self, tab_id: str, **kwargs) -> None:
        """Update tab properties."""
        with self._lock:
            if tab_id in self.tabs:
                for key, value in kwargs.items():
                    if hasattr(self.tabs[tab_id], key):
                        setattr(self.tabs[tab_id], key, value)

    def get_tabs_info(self) -> List[dict]:
        """Get all tabs info for UI."""
        with self._lock:
            return [
                {
                    "id": tab_id,
                    "title": self.tabs[tab_id].title,
                    "url": self.tabs[tab_id].url,
                    "favicon": self.tabs[tab_id].favicon,
                    "is_loading": self.tabs[tab_id].is_loading,
                    "is_active": tab_id == self.active_tab_id,
                }
                for tab_id in self.tab_order
                if tab_id in self.tabs
            ]


class TestTab:
    """Tests for Tab dataclass."""

    def test_tab_defaults(self):
        """Test Tab default values."""
        tab = Tab(id="test-1")
        assert tab.id == "test-1"
        assert tab.title == "New Tab"
        assert tab.url == ""
        assert tab.favicon == ""
        assert tab.is_loading is False
        assert tab.can_go_back is False
        assert tab.can_go_forward is False

    def test_tab_with_values(self):
        """Test Tab with custom values."""
        tab = Tab(
            id="tab-123",
            title="GitHub",
            url="https://github.com",
            favicon="🐙",
            is_loading=True,
            can_go_back=True,
            can_go_forward=False,
        )
        assert tab.id == "tab-123"
        assert tab.title == "GitHub"
        assert tab.url == "https://github.com"
        assert tab.favicon == "🐙"
        assert tab.is_loading is True
        assert tab.can_go_back is True
        assert tab.can_go_forward is False


class TestBrowserState:
    """Tests for BrowserState class."""

    @pytest.fixture
    def state(self):
        """Create a fresh BrowserState for each test."""
        return BrowserState()

    def test_initial_state(self, state):
        """Test initial state is empty."""
        assert len(state.tabs) == 0
        assert len(state.tab_order) == 0
        assert state.active_tab_id is None

    def test_add_tab(self, state):
        """Test adding a tab."""
        tab = Tab(id="tab-1", title="Test Tab")
        state.add_tab(tab)

        assert "tab-1" in state.tabs
        assert state.tabs["tab-1"].title == "Test Tab"
        assert state.tab_order == ["tab-1"]

    def test_add_multiple_tabs(self, state):
        """Test adding multiple tabs."""
        tab1 = Tab(id="tab-1", title="Tab 1")
        tab2 = Tab(id="tab-2", title="Tab 2")
        tab3 = Tab(id="tab-3", title="Tab 3")

        state.add_tab(tab1)
        state.add_tab(tab2)
        state.add_tab(tab3)

        assert len(state.tabs) == 3
        assert state.tab_order == ["tab-1", "tab-2", "tab-3"]

    def test_remove_tab(self, state):
        """Test removing a tab."""
        tab1 = Tab(id="tab-1")
        tab2 = Tab(id="tab-2")
        state.add_tab(tab1)
        state.add_tab(tab2)
        state.active_tab_id = "tab-1"

        next_id = state.remove_tab("tab-1")

        assert "tab-1" not in state.tabs
        assert state.tab_order == ["tab-2"]
        assert next_id == "tab-2"

    def test_remove_nonexistent_tab(self, state):
        """Test removing a tab that doesn't exist."""
        state.active_tab_id = "existing"
        result = state.remove_tab("nonexistent")
        assert result == "existing"

    def test_remove_last_tab(self, state):
        """Test removing the last tab."""
        tab = Tab(id="tab-1")
        state.add_tab(tab)
        state.active_tab_id = "tab-1"

        next_id = state.remove_tab("tab-1")

        assert next_id is None
        assert len(state.tabs) == 0
        assert len(state.tab_order) == 0

    def test_remove_middle_tab(self, state):
        """Test removing a tab from the middle."""
        for i in range(3):
            state.add_tab(Tab(id=f"tab-{i}"))
        state.active_tab_id = "tab-1"

        next_id = state.remove_tab("tab-1")

        assert next_id == "tab-2"
        assert state.tab_order == ["tab-0", "tab-2"]

    def test_get_tab(self, state):
        """Test getting a tab by ID."""
        tab = Tab(id="tab-1", title="Test")
        state.add_tab(tab)

        result = state.get_tab("tab-1")
        assert result is not None
        assert result.title == "Test"

    def test_get_nonexistent_tab(self, state):
        """Test getting a tab that doesn't exist."""
        result = state.get_tab("nonexistent")
        assert result is None

    def test_update_tab(self, state):
        """Test updating tab properties."""
        tab = Tab(id="tab-1", title="Original")
        state.add_tab(tab)

        state.update_tab("tab-1", title="Updated", url="https://example.com")

        updated = state.get_tab("tab-1")
        assert updated.title == "Updated"
        assert updated.url == "https://example.com"

    def test_update_nonexistent_tab(self, state):
        """Test updating a tab that doesn't exist (should not raise)."""
        state.update_tab("nonexistent", title="Test")
        # Should not raise an exception

    def test_get_tabs_info(self, state):
        """Test getting tabs info for UI."""
        tab1 = Tab(id="tab-1", title="Tab 1", url="https://example.com")
        tab2 = Tab(id="tab-2", title="Tab 2", url="https://test.com")
        state.add_tab(tab1)
        state.add_tab(tab2)
        state.active_tab_id = "tab-1"

        info = state.get_tabs_info()

        assert len(info) == 2
        assert info[0]["id"] == "tab-1"
        assert info[0]["title"] == "Tab 1"
        assert info[0]["is_active"] is True
        assert info[1]["id"] == "tab-2"
        assert info[1]["is_active"] is False

    def test_thread_safety(self, state):
        """Test that operations are thread-safe."""
        results = []
        errors = []

        def add_tabs():
            try:
                for i in range(100):
                    tab = Tab(id=f"thread-{threading.current_thread().name}-{i}")
                    state.add_tab(tab)
            except Exception as e:
                errors.append(e)

        threads = [
            threading.Thread(target=add_tabs, name=f"T{i}") for i in range(5)
        ]

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(errors) == 0
        assert len(state.tabs) == 500


class TestTabNavigation:
    """Tests for tab navigation scenarios."""

    @pytest.fixture
    def state_with_tabs(self):
        """Create a state with multiple tabs."""
        state = BrowserState()
        for i in range(5):
            tab = Tab(id=f"tab-{i}", title=f"Tab {i}")
            state.add_tab(tab)
        state.active_tab_id = "tab-2"
        return state

    def test_close_first_tab(self, state_with_tabs):
        """Test closing the first tab."""
        next_id = state_with_tabs.remove_tab("tab-0")
        assert next_id == "tab-1"

    def test_close_last_tab(self, state_with_tabs):
        """Test closing the last tab."""
        next_id = state_with_tabs.remove_tab("tab-4")
        assert next_id == "tab-3"

    def test_close_active_tab(self, state_with_tabs):
        """Test closing the active tab."""
        next_id = state_with_tabs.remove_tab("tab-2")
        assert next_id == "tab-3"

    def test_tab_order_preserved(self, state_with_tabs):
        """Test that tab order is preserved after operations."""
        state_with_tabs.remove_tab("tab-1")
        state_with_tabs.remove_tab("tab-3")

        assert state_with_tabs.tab_order == ["tab-0", "tab-2", "tab-4"]


class TestUrlParsing:
    """Tests for URL parsing utilities."""

    def test_get_domain_title(self):
        """Test extracting domain as title."""
        from urllib.parse import urlparse

        def get_domain_title(url: str) -> str:
            try:
                parsed = urlparse(url)
                domain = parsed.netloc or parsed.path
                if domain.startswith("www."):
                    domain = domain[4:]
                return domain.split(".")[0].capitalize()
            except Exception:
                return "New Tab"

        assert get_domain_title("https://github.com") == "Github"
        assert get_domain_title("https://www.google.com") == "Google"
        assert get_domain_title("https://developer.mozilla.org") == "Developer"
        assert get_domain_title("") == "New Tab"
        assert get_domain_title("invalid") == "Invalid"
