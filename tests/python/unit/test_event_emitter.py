# -*- coding: utf-8 -*-
"""Tests for EventEmitter base class."""

import logging
import threading
from unittest.mock import MagicMock, patch

import pytest

from auroraview.core.event_emitter import (
    EventEmitter,
    LoadEvent,
    NavigationEvent,
    WindowEvent,
    deprecated,
)


class TestEventEmitter:
    """Tests for EventEmitter class."""

    def test_init(self):
        """Test EventEmitter initialization."""
        emitter = EventEmitter()
        assert emitter._listeners == {}
        assert isinstance(emitter._lock, type(threading.RLock()))

    def test_on_with_handler(self):
        """Test on() with handler function."""
        emitter = EventEmitter()
        handler = MagicMock()

        unsub = emitter.on("test_event", handler)

        assert callable(unsub)
        assert emitter.listener_count("test_event") == 1

    def test_on_as_decorator(self):
        """Test on() as decorator."""
        emitter = EventEmitter()

        @emitter.on("test_event")
        def handler(data):
            pass

        assert emitter.listener_count("test_event") == 1

    def test_emit_calls_handler(self):
        """Test emit() calls registered handler."""
        emitter = EventEmitter()
        handler = MagicMock()
        emitter.on("test_event", handler)

        result = emitter.emit("test_event", {"key": "value"})

        assert result is True
        handler.assert_called_once_with({"key": "value"})

    def test_emit_without_data(self):
        """Test emit() without data calls handler with no args."""
        emitter = EventEmitter()
        handler = MagicMock()
        emitter.on("test_event", handler)

        emitter.emit("test_event")

        handler.assert_called_once_with()

    def test_emit_returns_false_no_handlers(self):
        """Test emit() returns False when no handlers registered."""
        emitter = EventEmitter()

        result = emitter.emit("nonexistent_event", {})

        assert result is False

    def test_emit_multiple_handlers(self):
        """Test emit() calls all registered handlers."""
        emitter = EventEmitter()
        handler1 = MagicMock()
        handler2 = MagicMock()
        emitter.on("test_event", handler1)
        emitter.on("test_event", handler2)

        emitter.emit("test_event", "data")

        handler1.assert_called_once_with("data")
        handler2.assert_called_once_with("data")

    def test_once_handler_called_once(self):
        """Test once() handler is only called once."""
        emitter = EventEmitter()
        handler = MagicMock()
        emitter.once("test_event", handler)

        emitter.emit("test_event", "first")
        emitter.emit("test_event", "second")

        handler.assert_called_once_with("first")

    def test_once_as_decorator(self):
        """Test once() as decorator."""
        emitter = EventEmitter()
        call_count = [0]

        @emitter.once("test_event")
        def handler(data):
            call_count[0] += 1

        emitter.emit("test_event", "first")
        emitter.emit("test_event", "second")

        assert call_count[0] == 1

    def test_off_removes_specific_handler(self):
        """Test off() removes specific handler."""
        emitter = EventEmitter()
        handler1 = MagicMock()
        handler2 = MagicMock()
        emitter.on("test_event", handler1)
        emitter.on("test_event", handler2)

        emitter.off("test_event", handler1)

        assert emitter.listener_count("test_event") == 1
        emitter.emit("test_event", "data")
        handler1.assert_not_called()
        handler2.assert_called_once()

    def test_off_removes_all_handlers(self):
        """Test off() removes all handlers when no handler specified."""
        emitter = EventEmitter()
        emitter.on("test_event", MagicMock())
        emitter.on("test_event", MagicMock())

        emitter.off("test_event")

        assert emitter.listener_count("test_event") == 0

    def test_off_nonexistent_event(self):
        """Test off() on nonexistent event does nothing."""
        emitter = EventEmitter()

        # Should not raise
        emitter.off("nonexistent")

    def test_unsubscribe_function(self):
        """Test unsubscribe function returned by on()."""
        emitter = EventEmitter()
        handler = MagicMock()
        unsub = emitter.on("test_event", handler)

        assert emitter.listener_count("test_event") == 1

        unsub()

        assert emitter.listener_count("test_event") == 0

    def test_unsubscribe_idempotent(self):
        """Test unsubscribe can be called multiple times safely."""
        emitter = EventEmitter()
        handler = MagicMock()
        unsub = emitter.on("test_event", handler)

        unsub()
        unsub()  # Should not raise

        assert emitter.listener_count("test_event") == 0

    def test_remove_all_listeners_specific_event(self):
        """Test remove_all_listeners() for specific event."""
        emitter = EventEmitter()
        emitter.on("event1", MagicMock())
        emitter.on("event2", MagicMock())

        emitter.remove_all_listeners("event1")

        assert emitter.listener_count("event1") == 0
        assert emitter.listener_count("event2") == 1

    def test_remove_all_listeners_all_events(self):
        """Test remove_all_listeners() removes all events."""
        emitter = EventEmitter()
        emitter.on("event1", MagicMock())
        emitter.on("event2", MagicMock())

        emitter.remove_all_listeners()

        assert emitter.listener_count("event1") == 0
        assert emitter.listener_count("event2") == 0

    def test_remove_all_listeners_nonexistent_event(self):
        """Test remove_all_listeners() on nonexistent event does nothing."""
        emitter = EventEmitter()

        # Should not raise
        emitter.remove_all_listeners("nonexistent")

    def test_listener_count(self):
        """Test listener_count() returns correct count."""
        emitter = EventEmitter()

        assert emitter.listener_count("test_event") == 0

        emitter.on("test_event", MagicMock())
        assert emitter.listener_count("test_event") == 1

        emitter.on("test_event", MagicMock())
        assert emitter.listener_count("test_event") == 2

    def test_event_names(self):
        """Test event_names() returns all event names."""
        emitter = EventEmitter()
        emitter.on("event1", MagicMock())
        emitter.on("event2", MagicMock())
        emitter.on("event3", MagicMock())

        names = emitter.event_names()

        assert set(names) == {"event1", "event2", "event3"}

    def test_event_names_empty(self):
        """Test event_names() returns empty list when no events."""
        emitter = EventEmitter()

        assert emitter.event_names() == []

    def test_handler_exception_logged(self):
        """Test handler exception is logged but doesn't stop other handlers."""
        emitter = EventEmitter()

        def bad_handler(data):
            raise ValueError("Test error")

        good_handler = MagicMock()

        emitter.on("test_event", bad_handler)
        emitter.on("test_event", good_handler)

        with patch.object(logging.getLogger("auroraview.core.event_emitter"), "exception"):
            emitter.emit("test_event", "data")

        # Good handler should still be called
        good_handler.assert_called_once_with("data")

    def test_thread_safety(self):
        """Test EventEmitter is thread-safe."""
        emitter = EventEmitter()
        results = []
        errors = []

        def add_listener(i):
            try:
                emitter.on(f"event_{i}", lambda d: results.append(d))
            except Exception as e:
                errors.append(e)

        threads = [threading.Thread(target=add_listener, args=(i,)) for i in range(100)]

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(errors) == 0
        assert len(emitter.event_names()) == 100


class TestNavigationEvent:
    """Tests for NavigationEvent dataclass."""

    def test_navigation_event_defaults(self):
        """Test NavigationEvent default values."""
        event = NavigationEvent(url="https://example.com")

        assert event.url == "https://example.com"
        assert event.event_type == "start"
        assert event.success is True
        assert event.error is None
        assert event.progress == 0
        assert event.status_code is None

    def test_navigation_event_start_repr(self):
        """Test NavigationEvent repr for start event."""
        event = NavigationEvent(url="https://example.com", event_type="start")

        repr_str = repr(event)
        assert "url='https://example.com'" in repr_str
        assert "type='start'" in repr_str

    def test_navigation_event_progress_repr(self):
        """Test NavigationEvent repr for progress event."""
        event = NavigationEvent(url="https://example.com", event_type="progress", progress=50)

        repr_str = repr(event)
        assert "type='progress'" in repr_str
        assert "progress=50" in repr_str

    def test_navigation_event_end_success_repr(self):
        """Test NavigationEvent repr for successful end event."""
        event = NavigationEvent(url="https://example.com", event_type="end", success=True)

        repr_str = repr(event)
        assert "type='end'" in repr_str
        assert "success=True" in repr_str

    def test_navigation_event_end_error_repr(self):
        """Test NavigationEvent repr for failed end event."""
        event = NavigationEvent(
            url="https://example.com", event_type="end", success=False, error="Connection failed"
        )

        repr_str = repr(event)
        assert "type='end'" in repr_str
        assert "error='Connection failed'" in repr_str


class TestWindowEvent:
    """Tests for WindowEvent dataclass."""

    def test_window_event_resize(self):
        """Test WindowEvent for resize."""
        event = WindowEvent(event_type="resize", width=800, height=600)

        assert event.event_type == "resize"
        assert event.width == 800
        assert event.height == 600

    def test_window_event_move(self):
        """Test WindowEvent for move."""
        event = WindowEvent(event_type="move", x=100, y=200)

        assert event.event_type == "move"
        assert event.x == 100
        assert event.y == 200

    def test_window_event_fullscreen(self):
        """Test WindowEvent for fullscreen."""
        event = WindowEvent(event_type="fullscreen", fullscreen=True)

        assert event.event_type == "fullscreen"
        assert event.fullscreen is True

    def test_window_event_defaults(self):
        """Test WindowEvent default values."""
        event = WindowEvent(event_type="show")

        assert event.width is None
        assert event.height is None
        assert event.x is None
        assert event.y is None
        assert event.fullscreen is None


class TestLoadEvent:
    """Tests for LoadEvent dataclass."""

    def test_load_event_defaults(self):
        """Test LoadEvent default values."""
        event = LoadEvent(url="https://example.com")

        assert event.url == "https://example.com"
        assert event.title is None
        assert event.ready is False

    def test_load_event_with_values(self):
        """Test LoadEvent with all values."""
        event = LoadEvent(url="https://example.com", title="Example", ready=True)

        assert event.url == "https://example.com"
        assert event.title == "Example"
        assert event.ready is True


class TestDeprecatedDecorator:
    """Tests for deprecated decorator."""

    def test_deprecated_warns(self):
        """Test deprecated decorator emits warning."""

        @deprecated("Use new_func instead")
        def old_func():
            return "result"

        with pytest.warns(DeprecationWarning, match="old_func is deprecated"):
            result = old_func()

        assert result == "result"

    def test_deprecated_preserves_return_value(self):
        """Test deprecated decorator preserves return value."""

        @deprecated("Use new_func instead")
        def old_func(x, y):
            return x + y

        with pytest.warns(DeprecationWarning):
            result = old_func(1, 2)

        assert result == 3

    def test_deprecated_updates_docstring(self):
        """Test deprecated decorator updates docstring."""

        @deprecated("Use new_func instead")
        def old_func():
            """Original docstring."""
            pass

        assert "DEPRECATED" in old_func.__doc__
        assert "Use new_func instead" in old_func.__doc__
        assert "Original docstring" in old_func.__doc__

    def test_deprecated_with_no_docstring(self):
        """Test deprecated decorator with function without docstring."""

        @deprecated("Use new_func instead")
        def old_func():
            pass

        assert "DEPRECATED" in old_func.__doc__
