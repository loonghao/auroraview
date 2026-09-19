# -*- coding: utf-8 -*-
"""Unit tests for ReadyEvents."""

from __future__ import annotations

import threading
import time
from unittest.mock import MagicMock

import pytest


class TestReadyEvents:
    """Tests for ReadyEvents lifecycle tracking."""

    def test_initial_state(self):
        """Test initial state of ReadyEvents."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        assert events.is_created() is False
        assert events.is_shown() is False
        assert events.is_loaded() is False
        assert events.is_bridge_ready() is False
        assert events.is_ready() is False

    def test_set_events(self):
        """Test setting individual events."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        events.set_created()
        assert events.is_created() is True

        events.set_shown()
        assert events.is_shown() is True

        events.set_loaded()
        assert events.is_loaded() is True

        events.set_bridge_ready()
        assert events.is_bridge_ready() is True

        assert events.is_ready() is True

    def test_wait_created(self):
        """Test waiting for created event."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        # Set event in background thread
        def set_event():
            time.sleep(0.1)
            events.set_created()

        thread = threading.Thread(target=set_event)
        thread.start()

        result = events.wait_created(timeout=1.0)
        assert result is True

        thread.join()

    def test_wait_timeout(self):
        """Test timeout behavior."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        result = events.wait_created(timeout=0.1)
        assert result is False

    def test_wait_all(self):
        """Test waiting for all events."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        # Set all events in background
        def set_all():
            time.sleep(0.05)
            events.set_created()
            time.sleep(0.05)
            events.set_shown()
            time.sleep(0.05)
            events.set_loaded()
            time.sleep(0.05)
            events.set_bridge_ready()

        thread = threading.Thread(target=set_all)
        thread.start()

        result = events.wait_all(timeout=1.0)
        assert result is True

        thread.join()

    def test_wait_all_timeout(self):
        """Test wait_all timeout."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        # Only set some events
        events.set_created()
        events.set_shown()

        result = events.wait_all(timeout=0.1)
        assert result is False

    def test_reset(self):
        """Test resetting all events."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        events.set_created()
        events.set_shown()
        events.set_loaded()
        events.set_bridge_ready()

        assert events.is_ready() is True

        events.reset()

        assert events.is_created() is False
        assert events.is_shown() is False
        assert events.is_loaded() is False
        assert events.is_bridge_ready() is False

    def test_status(self):
        """Test status dictionary."""
        from auroraview.core.ready_events import ReadyEvents

        webview = MagicMock()
        events = ReadyEvents(webview)

        events.set_created()
        events.set_loaded()

        status = events.status()

        assert status == {
            "created": True,
            "shown": False,
            "loaded": True,
            "bridge_ready": False,
        }


class TestDecorators:
    """Tests for require_* decorators."""

    def test_require_created(self):
        """Test require_created decorator."""
        from auroraview.core.ready_events import ReadyEvents, require_created

        class MockWebView:
            def __init__(self):
                self._ready_events = ReadyEvents(self)

            @require_created
            def test_method(self):
                return "success"

        webview = MockWebView()

        # Should timeout without event
        with pytest.raises(RuntimeError, match="failed to create"):
            webview._ready_events.created.clear()
            webview.test_method()

        # Should succeed with event
        webview._ready_events.set_created()
        result = webview.test_method()
        assert result == "success"

    def test_require_loaded(self):
        """Test require_loaded decorator."""
        from auroraview.core.ready_events import ReadyEvents, require_loaded

        class MockWebView:
            def __init__(self):
                self._ready_events = ReadyEvents(self)

            @require_loaded
            def test_method(self):
                return "loaded"

        webview = MockWebView()

        # Should timeout without event
        with pytest.raises(RuntimeError, match="failed to load"):
            webview.test_method()

        # Should succeed with event
        webview._ready_events.set_loaded()
        result = webview.test_method()
        assert result == "loaded"

    def test_require_shown(self):
        """Test require_shown decorator."""
        from auroraview.core.ready_events import ReadyEvents, require_shown

        class MockWebView:
            def __init__(self):
                self._ready_events = ReadyEvents(self)

            @require_shown
            def test_method(self):
                return "shown"

        webview = MockWebView()

        # Should timeout without event
        with pytest.raises(RuntimeError, match="failed to show"):
            webview.test_method()

        # Should succeed with event
        webview._ready_events.set_shown()
        result = webview.test_method()
        assert result == "shown"

    def test_require_bridge_ready(self):
        """Test require_bridge_ready decorator."""
        from auroraview.core.ready_events import ReadyEvents, require_bridge_ready

        class MockWebView:
            def __init__(self):
                self._ready_events = ReadyEvents(self)

            @require_bridge_ready
            def test_method(self):
                return "bridge ready"

        webview = MockWebView()

        # Should timeout without event
        with pytest.raises(RuntimeError, match="bridge failed"):
            webview.test_method()

        # Should succeed with event
        webview._ready_events.set_bridge_ready()
        result = webview.test_method()
        assert result == "bridge ready"

    def test_require_ready(self):
        """Test require_ready decorator (all events)."""
        from auroraview.core.ready_events import ReadyEvents, require_ready

        class MockWebView:
            def __init__(self):
                self._ready_events = ReadyEvents(self)

            @require_ready
            def test_method(self):
                return "all ready"

        webview = MockWebView()

        # Should timeout without all events
        with pytest.raises(RuntimeError, match="failed to become ready"):
            webview.test_method()

        # Should succeed with all events
        webview._ready_events.set_created()
        webview._ready_events.set_shown()
        webview._ready_events.set_loaded()
        webview._ready_events.set_bridge_ready()
        result = webview.test_method()
        assert result == "all ready"

    def test_decorator_without_ready_events(self):
        """Test decorators work when _ready_events is None."""
        from auroraview.core.ready_events import require_loaded

        class MockWebView:
            def __init__(self):
                self._ready_events = None

            @require_loaded
            def test_method(self):
                return "no events"

        webview = MockWebView()

        # Should succeed without _ready_events
        result = webview.test_method()
        assert result == "no events"


class TestReadyEventsTimeoutOverride:
    """`AURORAVIEW_READY_TIMEOUT` keeps automated runs from stalling.

    Without a display (or in a test that never shows a window) the `require_*`
    decorators can never be satisfied, so each call burns the full timeout. The
    variable lets a test suite shorten that wait instead of hanging on it.
    """

    def test_default_timeouts_are_unchanged(self, monkeypatch):
        """Removing the variable restores the historical 20s / 30s defaults."""
        monkeypatch.delenv("AURORAVIEW_READY_TIMEOUT", raising=False)
        from auroraview.core.ready_events import ready_all_timeout, ready_event_timeout

        assert ready_event_timeout() == 20.0
        assert ready_all_timeout() == 30.0

    def test_env_var_shortens_the_wait(self, monkeypatch):
        """The decorators honour a shortened timeout."""
        import time

        from auroraview.core.ready_events import ReadyEvents, require_created

        monkeypatch.setenv("AURORAVIEW_READY_TIMEOUT", "0.05")

        class MockWebView:
            def __init__(self):
                self._ready_events = ReadyEvents(self)

            @require_created
            def test_method(self):
                return "success"

        webview = MockWebView()
        webview._ready_events.created.clear()

        start = time.monotonic()
        with pytest.raises(RuntimeError, match="failed to create"):
            webview.test_method()
        elapsed = time.monotonic() - start

        # Must fail fast rather than blocking for the 20s default.
        assert elapsed < 5.0

    def test_invalid_env_var_falls_back_to_default(self, monkeypatch):
        """A malformed value must never disable the timeout."""
        monkeypatch.setenv("AURORAVIEW_READY_TIMEOUT", "not-a-number")
        from auroraview.core.ready_events import ready_event_timeout

        assert ready_event_timeout() == 20.0

    def test_negative_env_var_falls_back_to_default(self, monkeypatch):
        """A negative value must never disable the timeout."""
        monkeypatch.setenv("AURORAVIEW_READY_TIMEOUT", "-1")
        from auroraview.core.ready_events import ready_event_timeout

        assert ready_event_timeout() == 20.0

    def test_explicit_timeout_argument_still_wins(self, monkeypatch):
        """Callers passing an explicit timeout keep full control."""
        monkeypatch.setenv("AURORAVIEW_READY_TIMEOUT", "0.05")
        from auroraview.core.ready_events import ReadyEvents

        events = ReadyEvents(MagicMock())
        assert events.wait_created(timeout=0.01) is False

        events.set_created()
        assert events.wait_created(timeout=0.01) is True
