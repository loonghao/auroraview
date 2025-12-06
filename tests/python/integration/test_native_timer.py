"""Tests for NativeTimer (Rust implementation)."""

import sys

import pytest


class TestNativeTimer:
    """Test NativeTimer class (Rust implementation)."""

    def test_import_native_timer(self):
        """Test that NativeTimer can be imported."""
        try:
            from auroraview._auroraview import NativeTimer

            assert NativeTimer is not None
        except ImportError as e:
            pytest.skip(f"NativeTimer not available: {e}")

    def test_timer_creation(self):
        """Test creating a NativeTimer."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)
        assert timer is not None
        assert timer.interval_ms() == 16
        assert not timer.is_running()
        assert timer.tick_count() == 0

    def test_timer_creation_with_different_intervals(self):
        """Test creating timers with different intervals."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        intervals = [1, 16, 33, 100, 1000]
        for interval in intervals:
            timer = NativeTimer(interval)
            assert timer.interval_ms() == interval
            assert not timer.is_running()

    def test_timer_backend(self):
        """Test timer backend property."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)
        backend = timer.backend()
        assert backend is not None
        # Backend should be ThreadBased by default
        assert str(backend) == "ThreadBased"

    def test_timer_repr(self):
        """Test timer string representation."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)
        repr_str = repr(timer)
        assert "NativeTimer" in repr_str
        assert "interval_ms=16" in repr_str
        assert "running=false" in repr_str.lower()

    def test_timer_to_dict(self):
        """Test timer to_dict method."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)
        timer_dict = timer.to_dict()

        assert isinstance(timer_dict, dict)
        assert timer_dict["interval_ms"] == 16
        assert timer_dict["running"] is False
        assert timer_dict["tick_count"] == 0
        assert "backend" in timer_dict

    def test_timer_stop_when_not_running(self):
        """Test stopping timer when not running."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)
        # Should not raise error
        timer.stop()
        assert not timer.is_running()

    def test_timer_callback(self):
        """Test setting timer callback."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)

        callback_called = [False]

        def callback():
            callback_called[0] = True

        # Should not raise error
        timer.set_callback(callback)

    @pytest.mark.skipif(sys.platform != "win32", reason="Windows-only test")
    def test_windows_timer_start_invalid_hwnd(self):
        """Test starting Windows timer with invalid HWND."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)

        # Invalid HWND should fail
        with pytest.raises(RuntimeError):
            timer.start_windows(0)

    @pytest.mark.skipif(sys.platform != "win32", reason="Windows-only test")
    def test_windows_timer_process_messages(self):
        """Test processing Windows timer messages."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)

        # Should return 0 when not running
        count = timer.process_messages()
        assert count == 0

    def test_timer_context_manager(self):
        """Test using NativeTimer as context manager."""
        try:
            from auroraview._auroraview import NativeTimer
        except ImportError:
            pytest.skip("NativeTimer not available")

        timer = NativeTimer(16)

        with timer as t:
            assert t is timer

        # Timer should be stopped after exiting context
        assert not timer.is_running()
