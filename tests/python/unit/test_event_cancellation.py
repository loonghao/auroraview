# -*- coding: utf-8 -*-
"""Tests for event cancellation mechanism."""

from unittest.mock import MagicMock

from auroraview.core.event_emitter import EventEmitter


class TestEventCancellation:
    """Tests for cancellable events."""

    def test_emit_cancellable_no_handlers(self):
        """Test emit_cancellable returns False when no handlers registered.

        Note: The implementation returns False (not cancelled semantically means
        the event can proceed) when there are no handlers. This is because
        `not cancellable` evaluates to False when cancellable=True.
        """
        emitter = EventEmitter()
        result = emitter.emit_cancellable("test_event", {"data": "value"})
        # Implementation returns `not cancellable` (False) when no listeners
        assert result is False

    def test_emit_cancellable_handler_returns_none(self):
        """Test emit_cancellable with handler that returns None."""
        emitter = EventEmitter()
        handler = MagicMock(return_value=None)
        emitter.on("test_event", handler)

        result = emitter.emit_cancellable("test_event", {"data": "value"})

        assert result is True  # None means not cancelled
        handler.assert_called_once_with({"data": "value"})

    def test_emit_cancellable_handler_returns_true(self):
        """Test emit_cancellable with handler that returns True."""
        emitter = EventEmitter()
        handler = MagicMock(return_value=True)
        emitter.on("test_event", handler)

        result = emitter.emit_cancellable("test_event", {"data": "value"})

        assert result is True  # True means not cancelled
        handler.assert_called_once()

    def test_emit_cancellable_handler_returns_false(self):
        """Test emit_cancellable with handler that returns False."""
        emitter = EventEmitter()
        handler = MagicMock(return_value=False)
        emitter.on("test_event", handler)

        result = emitter.emit_cancellable("test_event", {"data": "value"})

        assert result is False  # False means cancelled
        handler.assert_called_once()

    def test_emit_cancellable_multiple_handlers_all_allow(self):
        """Test emit_cancellable with multiple handlers all allowing."""
        emitter = EventEmitter()
        handler1 = MagicMock(return_value=True)
        handler2 = MagicMock(return_value=None)
        handler3 = MagicMock(return_value=True)

        emitter.on("test_event", handler1)
        emitter.on("test_event", handler2)
        emitter.on("test_event", handler3)

        result = emitter.emit_cancellable("test_event", "data")

        assert result is True
        handler1.assert_called_once()
        handler2.assert_called_once()
        handler3.assert_called_once()

    def test_emit_cancellable_multiple_handlers_one_cancels(self):
        """Test emit_cancellable with one handler cancelling."""
        emitter = EventEmitter()
        handler1 = MagicMock(return_value=True)
        handler2 = MagicMock(return_value=False)  # This cancels
        handler3 = MagicMock(return_value=True)

        emitter.on("test_event", handler1)
        emitter.on("test_event", handler2)
        emitter.on("test_event", handler3)

        result = emitter.emit_cancellable("test_event", "data")

        assert result is False  # Event was cancelled
        # All handlers should still be called
        handler1.assert_called_once()
        handler2.assert_called_once()
        handler3.assert_called_once()

    def test_emit_cancellable_first_handler_cancels(self):
        """Test emit_cancellable when first handler cancels."""
        emitter = EventEmitter()
        handler1 = MagicMock(return_value=False)
        handler2 = MagicMock(return_value=True)

        emitter.on("test_event", handler1)
        emitter.on("test_event", handler2)

        result = emitter.emit_cancellable("test_event", "data")

        assert result is False
        # Both handlers should still be called
        handler1.assert_called_once()
        handler2.assert_called_once()

    def test_emit_cancellable_without_data(self):
        """Test emit_cancellable without event data."""
        emitter = EventEmitter()
        handler = MagicMock(return_value=True)
        emitter.on("test_event", handler)

        result = emitter.emit_cancellable("test_event")

        assert result is True
        handler.assert_called_once_with()

    def test_emit_vs_emit_cancellable(self):
        """Test difference between emit and emit_cancellable."""
        emitter = EventEmitter()
        handler = MagicMock(return_value=False)
        emitter.on("test_event", handler)

        # Regular emit ignores return value
        result1 = emitter.emit("test_event", "data")
        assert result1 is True  # Returns True if handlers were called

        # Cancellable emit checks return value
        result2 = emitter.emit_cancellable("test_event", "data")
        assert result2 is False  # Returns False because handler returned False

    def test_emit_cancellable_with_exception(self):
        """Test emit_cancellable handles exceptions gracefully."""
        emitter = EventEmitter()

        def failing_handler(data):
            raise ValueError("Test error")

        handler2 = MagicMock(return_value=True)

        emitter.on("test_event", failing_handler)
        emitter.on("test_event", handler2)

        # Should not raise, should continue to next handler
        result = emitter.emit_cancellable("test_event", "data")

        assert result is True
        handler2.assert_called_once()

    def test_emit_cancellable_once_handler(self):
        """Test emit_cancellable with once handler."""
        emitter = EventEmitter()
        handler = MagicMock(return_value=False)
        emitter.once("test_event", handler)

        # First call - handler cancels and is removed
        result1 = emitter.emit_cancellable("test_event", "data1")
        assert result1 is False
        handler.assert_called_once_with("data1")

        # Second call - no handlers, returns False (implementation detail)
        result2 = emitter.emit_cancellable("test_event", "data2")
        assert result2 is False  # No handlers = returns `not cancellable` = False
        # Handler should not be called again
        handler.assert_called_once()


class TestClosingEventCancellation:
    """Tests for window closing event cancellation pattern."""

    def test_closing_event_allowed(self):
        """Test closing event when all handlers allow."""
        emitter = EventEmitter()

        def on_closing(data):
            # Do cleanup
            return True  # Allow closing

        emitter.on("closing", on_closing)

        result = emitter.emit_cancellable("closing", {"reason": "user_request"})
        assert result is True  # Window can close

    def test_closing_event_prevented(self):
        """Test closing event when handler prevents it."""
        emitter = EventEmitter()

        def on_closing(data):
            # Check for unsaved changes
            has_unsaved = True
            if has_unsaved:
                return False  # Prevent closing
            return True

        emitter.on("closing", on_closing)

        result = emitter.emit_cancellable("closing", {"reason": "user_request"})
        assert result is False  # Window should not close

    def test_closing_event_multiple_checks(self):
        """Test closing event with multiple validation handlers."""
        emitter = EventEmitter()

        def check_unsaved_changes(data):
            return True  # No unsaved changes

        def check_running_tasks(data):
            return False  # Tasks still running, prevent close

        def check_confirmation(data):
            return True  # User confirmed

        emitter.on("closing", check_unsaved_changes)
        emitter.on("closing", check_running_tasks)
        emitter.on("closing", check_confirmation)

        result = emitter.emit_cancellable("closing", {"reason": "user_request"})
        assert result is False  # One handler prevented closing
