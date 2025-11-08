"""Tests for EventTimer functionality."""

import time
from unittest.mock import MagicMock

import pytest

from auroraview.event_timer import EventTimer


class MockWebView:
    """Mock WebView for testing."""

    def __init__(self):
        self._should_close = False
        self._process_events_called = 0
        self._is_valid = True
        self._core = MagicMock()
        self._core.is_window_valid.return_value = True

    def process_events(self):
        """Mock process_events method."""
        self._process_events_called += 1
        return self._should_close

    def trigger_close(self):
        """Simulate window close."""
        self._should_close = True

    def invalidate_window(self):
        """Simulate window becoming invalid."""
        self._is_valid = False
        self._core.is_window_valid.return_value = False


class TestEventTimer:
    """Test EventTimer class."""

    def test_init(self):
        """Test EventTimer initialization."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=16, check_window_validity=True)

        assert timer._webview is webview
        assert timer._interval_ms == 16
        assert timer._check_validity is True
        assert timer._running is False
        assert timer._timer_impl is None

    def test_start_stop(self):
        """Test starting and stopping timer."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        # Start timer
        timer.start()
        assert timer.is_running is True
        assert timer._timer_impl is not None

        # Wait a bit
        time.sleep(0.05)

        # Stop timer
        timer.stop()
        assert timer.is_running is False

    def test_start_already_running(self):
        """Test starting timer when already running."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        timer.start()
        with pytest.raises(RuntimeError, match="already running"):
            timer.start()

        timer.stop()

    def test_stop_not_running(self):
        """Test stopping timer when not running."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        # Should not raise error
        timer.stop()

    def test_on_close_callback(self):
        """Test on_close callback registration and execution."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        close_called = [False]

        @timer.on_close
        def handle_close():
            close_called[0] = True

        timer.start()

        # Trigger close
        webview.trigger_close()

        # Wait for callback
        time.sleep(0.05)

        assert close_called[0] is True
        assert timer.is_running is False

    def test_on_tick_callback(self):
        """Test on_tick callback registration and execution."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        tick_count = [0]

        @timer.on_tick
        def handle_tick():
            tick_count[0] += 1

        timer.start()

        # Wait for a few ticks
        time.sleep(0.05)

        timer.stop()

        # Should have been called multiple times
        assert tick_count[0] > 0

    def test_process_events_called(self):
        """Test that process_events is called periodically."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        timer.start()

        # Wait for a few ticks
        time.sleep(0.05)

        timer.stop()

        # process_events should have been called multiple times
        assert webview._process_events_called > 0

    def test_window_validity_check(self):
        """Test window validity checking."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10, check_window_validity=True)

        close_called = [False]

        @timer.on_close
        def handle_close():
            close_called[0] = True

        timer.start()

        # Invalidate window
        webview.invalidate_window()

        # Wait for detection
        time.sleep(0.05)

        assert close_called[0] is True
        assert timer.is_running is False

    def test_window_validity_check_disabled(self):
        """Test that window validity check can be disabled."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10, check_window_validity=False)

        close_called = [False]

        @timer.on_close
        def handle_close():
            close_called[0] = True

        timer.start()

        # Invalidate window
        webview.invalidate_window()

        # Wait a bit
        time.sleep(0.05)

        # Should not have triggered close (validity check disabled)
        # But we need to stop the timer manually
        timer.stop()

    def test_interval_property(self):
        """Test interval_ms property."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=16)

        assert timer.interval_ms == 16

        # Change interval
        timer.interval_ms = 33
        assert timer.interval_ms == 33

    def test_interval_property_invalid(self):
        """Test setting invalid interval."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=16)

        with pytest.raises(ValueError, match="must be positive"):
            timer.interval_ms = 0

        with pytest.raises(ValueError, match="must be positive"):
            timer.interval_ms = -1

    def test_context_manager(self):
        """Test using EventTimer as context manager."""
        webview = MockWebView()

        with EventTimer(webview, interval_ms=10) as timer:
            assert timer.is_running is True

            # Wait a bit
            time.sleep(0.05)

        # Timer should be stopped after exiting context
        assert timer.is_running is False

    def test_multiple_close_callbacks(self):
        """Test multiple close callbacks."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        close_count = [0]

        @timer.on_close
        def handle_close_1():
            close_count[0] += 1

        @timer.on_close
        def handle_close_2():
            close_count[0] += 1

        timer.start()

        # Trigger close
        webview.trigger_close()

        # Wait for callbacks
        time.sleep(0.05)

        # Both callbacks should have been called
        assert close_count[0] == 2

    def test_multiple_tick_callbacks(self):
        """Test multiple tick callbacks."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        tick_count_1 = [0]
        tick_count_2 = [0]

        @timer.on_tick
        def handle_tick_1():
            tick_count_1[0] += 1

        @timer.on_tick
        def handle_tick_2():
            tick_count_2[0] += 1

        timer.start()

        # Wait for a few ticks
        time.sleep(0.05)

        timer.stop()

        # Both callbacks should have been called
        assert tick_count_1[0] > 0
        assert tick_count_2[0] > 0
        assert tick_count_1[0] == tick_count_2[0]

    def test_callback_exception_handling(self):
        """Test that exceptions in callbacks don't crash the timer."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        @timer.on_tick
        def handle_tick():
            raise RuntimeError("Test error")

        timer.start()

        # Wait a bit - timer should keep running despite exceptions
        time.sleep(0.05)

        assert timer.is_running is True

        timer.stop()

    def test_repr(self):
        """Test string representation."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=16)

        repr_str = repr(timer)
        assert "EventTimer" in repr_str
        assert "16ms" in repr_str
        assert "stopped" in repr_str

        timer.start()
        repr_str = repr(timer)
        assert "running" in repr_str

        timer.stop()

    def test_timer_backend_selection(self):
        """Test that timer selects appropriate backend."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=16)

        timer.start()

        # Should have selected a backend
        assert timer._timer_type in ["qt", "maya", "blender", "houdini", "thread"]
        assert timer._timer_impl is not None

        timer.stop()

    def test_tick_count_increments(self):
        """Test that tick count increments over time."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        initial_count = timer._tick_count

        timer.start()
        time.sleep(0.05)
        timer.stop()

        # Tick count should have increased
        assert timer._tick_count > initial_count

    def test_close_callback_stops_timer(self):
        """Test that close callback can stop the timer."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        @timer.on_close
        def handle_close():
            # Timer should auto-stop on close
            pass

        timer.start()
        webview.trigger_close()

        # Wait for close detection
        time.sleep(0.05)

        # Timer should be stopped
        assert not timer.is_running

    def test_multiple_start_stop_cycles(self):
        """Test multiple start/stop cycles."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        for _ in range(3):
            timer.start()
            assert timer.is_running
            time.sleep(0.02)
            timer.stop()
            assert not timer.is_running
            time.sleep(0.01)

    def test_interval_change_while_stopped(self):
        """Test changing interval while timer is stopped."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=16)

        timer.interval_ms = 33
        assert timer.interval_ms == 33

        # Should be able to start with new interval
        timer.start()
        assert timer.is_running
        timer.stop()

    def test_callback_execution_order(self):
        """Test that callbacks execute in registration order."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        execution_order = []

        @timer.on_tick
        def callback1():
            execution_order.append(1)

        @timer.on_tick
        def callback2():
            execution_order.append(2)

        @timer.on_tick
        def callback3():
            execution_order.append(3)

        timer.start()
        time.sleep(0.03)
        timer.stop()

        # Should have executed in order
        assert execution_order == [1, 2, 3] or execution_order[:3] == [1, 2, 3]

    def test_timer_with_zero_callbacks(self):
        """Test timer works without any callbacks."""
        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        # Should work fine without callbacks
        timer.start()
        time.sleep(0.03)
        timer.stop()

        assert webview._process_events_called > 0

    def test_check_validity_flag(self):
        """Test that check_window_validity flag is respected."""
        webview = MockWebView()

        # With validity check enabled
        timer1 = EventTimer(webview, interval_ms=10, check_window_validity=True)
        assert timer1._check_validity is True

        # With validity check disabled
        timer2 = EventTimer(webview, interval_ms=10, check_window_validity=False)
        assert timer2._check_validity is False
