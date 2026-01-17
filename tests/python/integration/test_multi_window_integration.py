# -*- coding: utf-8 -*-
"""Integration tests for multi-window management.

These tests verify the WindowManager and ReadyEvents work correctly
across multiple WebView instances in realistic scenarios.
"""

from __future__ import annotations

import threading
import time
from typing import Any, List
from unittest.mock import MagicMock, patch


class TestWindowManagerIntegration:
    """Integration tests for WindowManager with multiple windows."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        WindowManager._instance = None

    def test_multiple_windows_lifecycle(self):
        """Test complete lifecycle of multiple windows."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Simulate creating multiple windows
        windows = []
        for i in range(5):
            webview = MagicMock()
            webview.title = f"Window {i}"
            uid = wm.register(webview)
            windows.append((uid, webview))

        # Verify all registered
        assert wm.count() == 5
        assert len(wm.get_all()) == 5

        # Close windows in random order
        close_order = [2, 0, 4, 1, 3]
        for idx in close_order:
            uid, _ = windows[idx]
            wm.unregister(uid)

        assert wm.count() == 0

    def test_active_window_tracking(self):
        """Test active window changes correctly during operations."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        active_changes: List[str] = []

        def on_change():
            active_changes.append(wm.get_active_id() or "none")

        wm.on_change(on_change)

        # Create windows
        wv1 = MagicMock()
        wv2 = MagicMock()
        wv3 = MagicMock()

        uid1 = wm.register(wv1)
        uid2 = wm.register(wv2)
        uid3 = wm.register(wv3)

        # First window should be active
        assert wm.get_active_id() == uid1

        # Switch active
        wm.set_active(uid2)
        assert wm.get_active_id() == uid2

        # Close active window, should switch to another
        wm.unregister(uid2)
        assert wm.get_active_id() in (uid1, uid3)

        # Close all
        wm.close_all()

    def test_broadcast_to_all_windows(self):
        """Test event broadcasting to all windows."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Create windows with emit tracking
        windows = []
        for _ in range(3):
            wv = MagicMock()
            wv.emit = MagicMock()
            wm.register(wv)
            windows.append(wv)

        # Broadcast event
        event_data = {"action": "refresh", "timestamp": 12345}
        count = wm.broadcast("global:refresh", event_data)

        assert count == 3
        for wv in windows:
            wv.emit.assert_called_with("global:refresh", event_data)

    def test_broadcast_with_failing_window(self):
        """Test broadcast continues when one window fails."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Create windows, one will fail
        wv1 = MagicMock()
        wv2 = MagicMock()
        wv2.emit.side_effect = RuntimeError("Window closed")
        wv3 = MagicMock()

        wm.register(wv1)
        wm.register(wv2)
        wm.register(wv3)

        # Should still broadcast to other windows
        count = wm.broadcast("test:event", {})
        assert count == 2

    def test_find_by_title(self):
        """Test finding windows by title."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Create windows with different titles
        wv1 = MagicMock()
        wv1.title = "Main Window"
        wv2 = MagicMock()
        wv2.title = "Settings"
        wv3 = MagicMock()
        wv3.title = "About"

        wm.register(wv1)
        wm.register(wv2)
        wm.register(wv3)

        assert wm.find_by_title("Settings") is wv2
        assert wm.find_by_title("Main Window") is wv1
        assert wm.find_by_title("Nonexistent") is None

    def test_concurrent_registration(self):
        """Test thread-safe concurrent registration."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        registered_ids: List[str] = []
        lock = threading.Lock()

        def register_window(idx: int):
            wv = MagicMock()
            wv.title = f"Window {idx}"
            uid = wm.register(wv)
            with lock:
                registered_ids.append(uid)

        # Register concurrently
        threads = [threading.Thread(target=register_window, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # All should be registered
        assert wm.count() == 10
        assert len(set(registered_ids)) == 10

    def test_weak_reference_cleanup(self):
        """Test windows are removed when garbage collected."""
        import gc

        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Create window and keep only weak ref
        class FakeWebView:
            def __init__(self):
                self.title = "Test"

        wv = FakeWebView()
        uid = wm.register(wv)
        assert wm.count() == 1

        # Delete strong reference
        del wv
        gc.collect()

        # Should be cleaned up
        assert wm.get(uid) is None


class TestReadyEventsIntegration:
    """Integration tests for ReadyEvents with WebView lifecycle."""

    def test_lifecycle_sequence(self):
        """Test events fire in correct sequence."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)
        sequence: List[str] = []

        def wait_and_record(name: str, event_method):
            result = event_method(timeout=1.0)
            if result:
                sequence.append(name)

        # Simulate lifecycle in background
        def lifecycle():
            time.sleep(0.05)
            events.set_created()
            time.sleep(0.05)
            events.set_shown()
            time.sleep(0.05)
            events.set_loaded()
            time.sleep(0.05)
            events.set_bridge_ready()

        lifecycle_thread = threading.Thread(target=lifecycle)
        lifecycle_thread.start()

        # Wait for events in sequence
        events.wait_created(timeout=1.0)
        sequence.append("created")
        events.wait_shown(timeout=1.0)
        sequence.append("shown")
        events.wait_loaded(timeout=1.0)
        sequence.append("loaded")
        events.wait_bridge_ready(timeout=1.0)
        sequence.append("bridge_ready")

        lifecycle_thread.join()

        assert sequence == ["created", "shown", "loaded", "bridge_ready"]

    def test_wait_all_with_partial_events(self):
        """Test wait_all times out if some events missing."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        # Only set some events
        events.set_created()
        events.set_shown()
        # Missing: loaded, bridge_ready

        result = events.wait_all(timeout=0.2)
        assert result is False

    def test_status_reflects_current_state(self):
        """Test status() accurately reflects event states."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        # Initial state
        status = events.status()
        assert all(v is False for v in status.values())

        # Set some events
        events.set_created()
        events.set_bridge_ready()

        status = events.status()
        assert status["created"] is True
        assert status["shown"] is False
        assert status["loaded"] is False
        assert status["bridge_ready"] is True


class TestTabContainerIntegration:
    """Integration tests for TabContainer with WindowManager."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        WindowManager._instance = None

    def test_tab_container_basic_operations(self):
        """Test basic TabContainer operations."""
        from auroraview.browser.tab_container import TabContainer

        updates: List[Any] = []
        changes: List[Any] = []

        container = TabContainer(
            on_tabs_update=lambda tabs: updates.append(tabs),
            on_tab_change=lambda tab: changes.append(tab),
            default_url="about:blank",
        )

        # Create tabs (without actually loading WebViews)
        with patch("auroraview.browser.tab_container.get_window_manager"):
            tab1 = container.create_tab(
                url="https://example.com",
                title="Example",
                load_immediately=False,
            )
            tab2 = container.create_tab(
                url="https://github.com",
                title="GitHub",
                load_immediately=False,
            )

        assert container.get_tab_count() == 2
        assert container.get_active_tab_id() == tab1.id

        # Switch tabs
        container.activate_tab(tab2.id)
        assert container.get_active_tab_id() == tab2.id

        # Close tab
        container.close_tab(tab1.id)
        assert container.get_tab_count() == 1
        assert container.get_active_tab_id() == tab2.id

    def test_tab_state_serialization(self):
        """Test TabState serialization."""
        from auroraview.browser.tab_container import TabState

        tab = TabState(
            id="tab_123",
            title="Test Tab",
            url="https://example.com",
            favicon="https://example.com/favicon.ico",
            is_loading=True,
            can_go_back=True,
            can_go_forward=False,
            metadata={"custom": "data"},
        )

        data = tab.to_dict()

        assert data["id"] == "tab_123"
        assert data["title"] == "Test Tab"
        assert data["url"] == "https://example.com"
        assert data["isLoading"] is True
        assert data["canGoBack"] is True
        assert data["canGoForward"] is False
        assert data["metadata"] == {"custom": "data"}

    def test_multiple_containers_independent(self):
        """Test multiple TabContainers are independent."""
        from auroraview.browser.tab_container import TabContainer

        container1 = TabContainer(default_url="about:blank")
        container2 = TabContainer(default_url="about:blank")

        with patch("auroraview.browser.tab_container.get_window_manager"):
            container1.create_tab(title="Tab 1", load_immediately=False)
            container1.create_tab(title="Tab 2", load_immediately=False)
            container2.create_tab(title="Tab A", load_immediately=False)

        assert container1.get_tab_count() == 2
        assert container2.get_tab_count() == 1


class TestBrowserIntegration:
    """Integration tests for Browser API."""

    def test_browser_tab_operations(self):
        """Test Browser tab operations."""
        from auroraview.browser.browser import Browser

        browser = Browser(
            title="Test Browser",
            width=800,
            height=600,
            default_url="about:blank",
        )

        # Create tabs
        tab1 = browser.new_tab("https://example.com", "Example")
        tab2 = browser.new_tab("https://github.com", "GitHub")

        assert len(browser.get_tabs()) == 2
        assert browser.get_active_tab()["id"] == tab1["id"]

        # Switch tab
        browser.activate_tab(tab2["id"])
        assert browser.get_active_tab()["id"] == tab2["id"]

        # Navigate
        browser.navigate("https://google.com")
        assert browser.get_active_tab()["url"] == "https://google.com"

        # Close tab
        browser.close_tab(tab1["id"])
        assert len(browser.get_tabs()) == 1

    def test_browser_on_ready_callback(self):
        """Test Browser on_ready callback."""
        from auroraview.browser.browser import Browser

        browser = Browser()
        ready_called = [False]

        def on_ready(b):
            ready_called[0] = True
            assert b is browser

        browser.on_ready(on_ready)

        # Without running, callback should not be called
        assert ready_called[0] is False


class TestCrossPanelCommunication:
    """Integration tests for cross-panel/window communication."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        WindowManager._instance = None

    def test_broadcast_state_sync(self):
        """Test state synchronization via broadcast."""
        from auroraview.core.window_manager import broadcast_event, get_window_manager

        wm = get_window_manager()
        received_events: List[Any] = []

        # Create windows that track received events
        for i in range(3):
            wv = MagicMock()

            def make_handler(idx):
                def handler(event, data):
                    received_events.append((idx, event, data))

                return handler

            wv.emit = make_handler(i)
            wm.register(wv)

        # Broadcast state update
        state_data = {
            "selected_item": "asset_001",
            "timestamp": 1234567890,
        }
        count = broadcast_event("state:selection_changed", state_data)

        assert count == 3
        assert len(received_events) == 3

        # All windows should receive same data
        for _idx, event, data in received_events:
            assert event == "state:selection_changed"
            assert data == state_data

    def test_targeted_window_communication(self):
        """Test sending event to specific window."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Create windows
        wv1 = MagicMock()
        wv2 = MagicMock()
        wv3 = MagicMock()

        wm.register(wv1, uid="panel_inspector")
        wm.register(wv2, uid="panel_outliner")
        wm.register(wv3, uid="panel_viewport")

        # Send to specific window
        target = wm.get("panel_inspector")
        assert target is wv1

        target.emit("specific:event", {"data": "value"})

        # Only wv1 should receive
        wv1.emit.assert_called_once()
        wv2.emit.assert_not_called()
        wv3.emit.assert_not_called()

    def test_window_groups(self):
        """Test grouping windows for targeted broadcasts."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()

        # Create windows with metadata-like IDs
        tool_windows = []
        panel_windows = []

        for i in range(2):
            wv = MagicMock()
            uid = wm.register(wv, uid=f"tool_{i}")
            tool_windows.append((uid, wv))

        for i in range(3):
            wv = MagicMock()
            uid = wm.register(wv, uid=f"panel_{i}")
            panel_windows.append((uid, wv))

        # Broadcast only to tools
        for uid, _wv in tool_windows:
            target = wm.get(uid)
            if target:
                target.emit("tools:refresh", {})

        # Verify only tool windows received
        for _, wv in tool_windows:
            wv.emit.assert_called_once()

        for _, wv in panel_windows:
            wv.emit.assert_not_called()
