"""Integration tests for timer functionality."""

import sys
import time
from unittest.mock import MagicMock

import pytest


class MockWebView:
    """Mock WebView for testing."""

    def __init__(self):
        self._should_close = False
        self._process_events_called = 0
        self._core = MagicMock()
        self._core.is_window_valid.return_value = True
        self._core.hwnd.return_value = 0x12345678  # Mock HWND

    def process_events(self):
        """Mock process_events method."""
        self._process_events_called += 1
        return self._should_close

    def trigger_close(self):
        """Simulate window close."""
        self._should_close = True


class TestTimerIntegration:
    """Integration tests for timer functionality."""

    def test_event_timer_with_thread_backend(self):
        """Test EventTimer with thread-based backend."""
        from auroraview.event_timer import EventTimer

        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        tick_count = [0]

        @timer.on_tick
        def handle_tick():
            tick_count[0] += 1

        timer.start()

        # Verify timer is using thread backend (fallback)
        # In test environment, Qt/Maya/etc are not available
        assert timer._timer_type == "thread"

        time.sleep(0.05)
        timer.stop()

        # Should have ticked multiple times
        assert tick_count[0] > 0

    def test_event_timer_performance(self):
        """Test EventTimer performance and timing accuracy."""
        from auroraview.event_timer import EventTimer

        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        tick_times = []

        @timer.on_tick
        def handle_tick():
            tick_times.append(time.time())

        timer.start()
        time.sleep(0.1)
        timer.stop()

        # Should have multiple ticks
        assert len(tick_times) >= 5

        # Check timing accuracy (allow some variance)
        if len(tick_times) >= 2:
            intervals = [
                (tick_times[i + 1] - tick_times[i]) * 1000 for i in range(len(tick_times) - 1)
            ]
            avg_interval = sum(intervals) / len(intervals)
            # Should be close to 10ms (allow 50% variance due to thread scheduling)
            assert 5 <= avg_interval <= 20

    def test_event_timer_cleanup_on_close(self):
        """Test that EventTimer properly cleans up on close."""
        from auroraview.event_timer import EventTimer

        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        cleanup_called = [False]

        @timer.on_close
        def handle_close():
            cleanup_called[0] = True

        timer.start()
        webview.trigger_close()

        # Wait for close detection
        time.sleep(0.05)

        # Cleanup should have been called
        assert cleanup_called[0]
        assert not timer.is_running

    def test_event_timer_error_recovery(self):
        """Test that EventTimer recovers from errors in callbacks."""
        from auroraview.event_timer import EventTimer

        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        error_count = [0]
        success_count = [0]

        @timer.on_tick
        def error_callback():
            error_count[0] += 1
            raise RuntimeError("Test error")

        @timer.on_tick
        def success_callback():
            success_count[0] += 1

        timer.start()
        time.sleep(0.05)
        timer.stop()

        # Both callbacks should have been called despite errors
        assert error_count[0] > 0
        assert success_count[0] > 0
        assert error_count[0] == success_count[0]

    @pytest.mark.skipif(sys.platform != "win32", reason="Windows-only test")
    def test_native_timer_availability(self):
        """Test that NativeTimer is available on Windows."""
        try:
            from auroraview._auroraview import NativeTimer

            timer = NativeTimer(16)
            assert timer is not None
            assert timer.interval_ms() == 16
        except ImportError:
            pytest.skip("NativeTimer not available")

    def test_timer_backend_fallback_chain(self):
        """Test that EventTimer tries backends in correct order."""
        from auroraview.event_timer import EventTimer

        webview = MockWebView()
        timer = EventTimer(webview, interval_ms=10)

        # In test environment, should fall back to thread backend
        timer.start()

        # Should have selected thread backend
        assert timer._timer_type == "thread"
        assert timer._timer_impl is not None

        timer.stop()

    def test_multiple_timers_simultaneously(self):
        """Test running multiple timers simultaneously."""
        from auroraview.event_timer import EventTimer

        webview1 = MockWebView()
        webview2 = MockWebView()

        timer1 = EventTimer(webview1, interval_ms=10)
        timer2 = EventTimer(webview2, interval_ms=15)

        tick_count1 = [0]
        tick_count2 = [0]

        @timer1.on_tick
        def handle_tick1():
            tick_count1[0] += 1

        @timer2.on_tick
        def handle_tick2():
            tick_count2[0] += 1

        timer1.start()
        timer2.start()

        time.sleep(0.1)

        timer1.stop()
        timer2.stop()

        # Both timers should have ticked
        assert tick_count1[0] > 0
        assert tick_count2[0] > 0

        # Timer1 should have ticked more (shorter interval)
        assert tick_count1[0] >= tick_count2[0]
