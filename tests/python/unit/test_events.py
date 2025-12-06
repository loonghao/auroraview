"""Tests for window events module."""

import pytest


class TestWindowEvent:
    """Tests for WindowEvent enum."""

    def test_window_event_import(self):
        """Test that WindowEvent can be imported."""
        from auroraview.core.events import WindowEvent

        assert WindowEvent is not None

    def test_window_event_values(self):
        """Test WindowEvent enum values."""
        from auroraview.core.events import WindowEvent

        # Page lifecycle events
        assert WindowEvent.LOADED.value == "loaded"
        assert WindowEvent.LOAD_STARTED.value == "load_started"
        assert WindowEvent.LOAD_FINISHED.value == "load_finished"

        # Window visibility events
        assert WindowEvent.SHOWN.value == "shown"
        assert WindowEvent.HIDDEN.value == "hidden"

        # Window close events
        assert WindowEvent.CLOSING.value == "closing"
        assert WindowEvent.CLOSED.value == "closed"

        # Window state events
        assert WindowEvent.FOCUSED.value == "focused"
        assert WindowEvent.BLURRED.value == "blurred"
        assert WindowEvent.MINIMIZED.value == "minimized"
        assert WindowEvent.MAXIMIZED.value == "maximized"
        assert WindowEvent.RESTORED.value == "restored"

        # Window geometry events
        assert WindowEvent.RESIZED.value == "resized"
        assert WindowEvent.MOVED.value == "moved"

        # Navigation events
        assert WindowEvent.NAVIGATION_STARTED.value == "navigation_started"
        assert WindowEvent.NAVIGATION_FINISHED.value == "navigation_finished"

    def test_window_event_str(self):
        """Test WindowEvent string conversion."""
        from auroraview.core.events import WindowEvent

        assert str(WindowEvent.LOADED) == "loaded"
        assert str(WindowEvent.CLOSING) == "closing"

    def test_window_event_is_string_enum(self):
        """Test that WindowEvent is a string enum."""
        from auroraview.core.events import WindowEvent

        # Should be usable as a string
        assert WindowEvent.LOADED == "loaded"
        assert WindowEvent.CLOSED == "closed"


class TestWindowEventData:
    """Tests for WindowEventData class."""

    def test_window_event_data_import(self):
        """Test that WindowEventData can be imported."""
        from auroraview.core.events import WindowEventData

        assert WindowEventData is not None

    def test_window_event_data_empty(self):
        """Test WindowEventData with no data."""
        from auroraview.core.events import WindowEventData

        data = WindowEventData()
        assert data.url is None
        assert data.width is None
        assert data.height is None
        assert data.x is None
        assert data.y is None
        assert data.focused is None

    def test_window_event_data_with_values(self):
        """Test WindowEventData with values."""
        from auroraview.core.events import WindowEventData

        data = WindowEventData(
            {
                "url": "https://example.com",
                "width": 800,
                "height": 600,
                "x": 100,
                "y": 50,
                "focused": True,
            }
        )

        assert data.url == "https://example.com"
        assert data.width == 800
        assert data.height == 600
        assert data.x == 100
        assert data.y == 50
        assert data.focused is True

    def test_window_event_data_get(self):
        """Test WindowEventData.get() method."""
        from auroraview.core.events import WindowEventData

        data = WindowEventData({"key": "value"})
        assert data.get("key") == "value"
        assert data.get("missing") is None
        assert data.get("missing", "default") == "default"

    def test_window_event_data_getitem(self):
        """Test WindowEventData[] access."""
        from auroraview.core.events import WindowEventData

        data = WindowEventData({"key": "value"})
        assert data["key"] == "value"

        with pytest.raises(KeyError):
            _ = data["missing"]

    def test_window_event_data_contains(self):
        """Test 'in' operator for WindowEventData."""
        from auroraview.core.events import WindowEventData

        data = WindowEventData({"key": "value"})
        assert "key" in data
        assert "missing" not in data

    def test_window_event_data_repr(self):
        """Test WindowEventData repr."""
        from auroraview.core.events import WindowEventData

        data = WindowEventData({"key": "value"})
        assert "WindowEventData" in repr(data)
        assert "key" in repr(data)


class TestEventExports:
    """Tests for event exports from main module."""

    def test_window_event_exported(self):
        """Test that WindowEvent is exported from main module."""
        from auroraview import WindowEvent

        assert WindowEvent is not None
        assert WindowEvent.LOADED == "loaded"

    def test_window_event_data_exported(self):
        """Test that WindowEventData is exported from main module."""
        from auroraview import WindowEventData

        assert WindowEventData is not None

    def test_event_handler_exported(self):
        """Test that EventHandler is exported from main module."""
        from auroraview import EventHandler

        assert EventHandler is not None


class TestWebViewEventMethods:
    """Tests for WebView event convenience methods."""

    def test_webview_has_on_loaded(self):
        """Test that WebView has on_loaded method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_loaded")

    def test_webview_has_on_shown(self):
        """Test that WebView has on_shown method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_shown")

    def test_webview_has_on_closing(self):
        """Test that WebView has on_closing method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_closing")

    def test_webview_has_on_closed(self):
        """Test that WebView has on_closed method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_closed")

    def test_webview_has_on_resized(self):
        """Test that WebView has on_resized method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_resized")

    def test_webview_has_on_moved(self):
        """Test that WebView has on_moved method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_moved")

    def test_webview_has_on_focused(self):
        """Test that WebView has on_focused method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_focused")

    def test_webview_has_on_blurred(self):
        """Test that WebView has on_blurred method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_blurred")

    def test_webview_has_on_minimized(self):
        """Test that WebView has on_minimized method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_minimized")

    def test_webview_has_on_maximized(self):
        """Test that WebView has on_maximized method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_maximized")

    def test_webview_has_on_restored(self):
        """Test that WebView has on_restored method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "on_restored")


class TestWebViewControlMethods:
    """Tests for WebView window control methods."""

    def test_webview_has_move(self):
        """Test that WebView has move method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "move")

    def test_webview_has_resize(self):
        """Test that WebView has resize method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "resize")

    def test_webview_has_minimize(self):
        """Test that WebView has minimize method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "minimize")

    def test_webview_has_maximize(self):
        """Test that WebView has maximize method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "maximize")

    def test_webview_has_restore(self):
        """Test that WebView has restore method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "restore")

    def test_webview_has_toggle_fullscreen(self):
        """Test that WebView has toggle_fullscreen method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "toggle_fullscreen")

    def test_webview_has_set_always_on_top(self):
        """Test that WebView has set_always_on_top method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "set_always_on_top")

    def test_webview_has_hide(self):
        """Test that WebView has hide method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "hide")

    def test_webview_has_focus(self):
        """Test that WebView has focus method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "focus")

    def test_webview_has_get_current_url(self):
        """Test that WebView has get_current_url method."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "get_current_url")


class TestWebViewProperties:
    """Tests for WebView window properties."""

    def test_webview_has_width_property(self):
        """Test that WebView has width property."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "width")

    def test_webview_has_height_property(self):
        """Test that WebView has height property."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "height")

    def test_webview_has_x_property(self):
        """Test that WebView has x property."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "x")

    def test_webview_has_y_property(self):
        """Test that WebView has y property."""
        from auroraview.core.webview import WebView

        assert hasattr(WebView, "y")
