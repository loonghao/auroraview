# -*- coding: utf-8 -*-
"""Unit tests for WindowManager."""

from __future__ import annotations

from unittest.mock import MagicMock


class TestWindowManager:
    """Tests for WindowManager singleton and operations."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        # Reset singleton for clean tests
        WindowManager._instance = None

    def test_singleton(self):
        """WindowManager should be singleton."""
        from auroraview.core.window_manager import get_window_manager

        wm1 = get_window_manager()
        wm2 = get_window_manager()
        assert wm1 is wm2

    def test_register_unregister(self):
        """Test window registration and unregistration."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        webview = MagicMock()

        # Register
        uid = wm.register(webview)
        assert uid.startswith("wv_")
        assert wm.get(uid) is webview
        assert wm.count() == 1

        # Unregister
        result = wm.unregister(uid)
        assert result is True
        assert wm.get(uid) is None
        assert wm.count() == 0

    def test_register_with_custom_uid(self):
        """Test registration with custom UID."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        webview = MagicMock()

        uid = wm.register(webview, uid="my_custom_id")
        assert uid == "my_custom_id"
        assert wm.get("my_custom_id") is webview

    def test_active_window(self):
        """Test active window tracking."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()

        uid1 = wm.register(wv1)
        uid2 = wm.register(wv2)

        # First registered should be active
        assert wm.get_active() is wv1
        assert wm.get_active_id() == uid1

        # Change active
        wm.set_active(uid2)
        assert wm.get_active() is wv2
        assert wm.get_active_id() == uid2

    def test_active_window_after_close(self):
        """Test active window selection after closing active."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()

        uid1 = wm.register(wv1)
        wm.register(wv2)

        wm.set_active(uid1)
        wm.unregister(uid1)

        # Should switch to remaining window
        assert wm.get_active() is wv2

    def test_get_all(self):
        """Test getting all windows."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()
        wv3 = MagicMock()

        wm.register(wv1)
        wm.register(wv2)
        wm.register(wv3)

        all_windows = wm.get_all()
        assert len(all_windows) == 3
        assert wv1 in all_windows
        assert wv2 in all_windows
        assert wv3 in all_windows

    def test_get_all_ids(self):
        """Test getting all window IDs."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()

        uid1 = wm.register(wv1)
        uid2 = wm.register(wv2)

        all_ids = wm.get_all_ids()
        assert uid1 in all_ids
        assert uid2 in all_ids

    def test_has(self):
        """Test checking if window exists."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        webview = MagicMock()

        uid = wm.register(webview)
        assert wm.has(uid) is True
        assert wm.has("nonexistent") is False

    def test_on_change_callback(self):
        """Test change notification callbacks."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        callback = MagicMock()

        unsubscribe = wm.on_change(callback)
        webview = MagicMock()

        # Should trigger on register
        wm.register(webview)
        assert callback.call_count == 1

        # Should trigger on set_active
        uid2 = wm.register(MagicMock())
        wm.set_active(uid2)
        assert callback.call_count >= 2

        # Unsubscribe
        unsubscribe()
        wm.register(MagicMock())
        # Should not increase after unsubscribe

    def test_broadcast(self):
        """Test event broadcasting."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()
        wv3 = MagicMock()

        wm.register(wv1)
        wm.register(wv2)
        wm.register(wv3)

        count = wm.broadcast("test:event", {"data": 123})

        assert count == 3
        wv1.emit.assert_called_with("test:event", {"data": 123})
        wv2.emit.assert_called_with("test:event", {"data": 123})
        wv3.emit.assert_called_with("test:event", {"data": 123})

    def test_close_all(self):
        """Test closing all windows."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()

        wm.register(wv1)
        wm.register(wv2)

        count = wm.close_all()

        assert count == 2
        wv1.close.assert_called_once()
        wv2.close.assert_called_once()

    def test_find_by_title(self):
        """Test finding window by title."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wv1 = MagicMock()
        wv1.title = "Window 1"
        wv2 = MagicMock()
        wv2.title = "Window 2"

        wm.register(wv1)
        wm.register(wv2)

        found = wm.find_by_title("Window 2")
        assert found is wv2

        not_found = wm.find_by_title("Nonexistent")
        assert not_found is None

    def test_reset(self):
        """Test resetting WindowManager."""
        from auroraview.core.window_manager import get_window_manager

        wm = get_window_manager()
        wm.register(MagicMock())
        wm.register(MagicMock())

        wm.reset()

        assert wm.count() == 0
        assert wm.get_active() is None


class TestGlobalAccessors:
    """Tests for global accessor functions."""

    def setup_method(self):
        """Reset WindowManager before each test."""
        from auroraview.core.window_manager import WindowManager

        WindowManager._instance = None

    def test_get_windows(self):
        """Test get_windows() function."""
        from auroraview.core.window_manager import get_window_manager, get_windows

        wm = get_window_manager()
        wv1 = MagicMock()
        wv2 = MagicMock()

        wm.register(wv1)
        wm.register(wv2)

        windows = get_windows()
        assert len(windows) == 2

    def test_get_active_window(self):
        """Test get_active_window() function."""
        from auroraview.core.window_manager import get_active_window, get_window_manager

        wm = get_window_manager()
        webview = MagicMock()
        wm.register(webview)

        active = get_active_window()
        assert active is webview

    def test_broadcast_event(self):
        """Test broadcast_event() function."""
        from auroraview.core.window_manager import broadcast_event, get_window_manager

        wm = get_window_manager()
        webview = MagicMock()
        wm.register(webview)

        count = broadcast_event("test:event", {"key": "value"})

        assert count == 1
        webview.emit.assert_called_with("test:event", {"key": "value"})
