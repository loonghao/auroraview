"""Unit tests for decorators module."""

import time

import pytest


@pytest.mark.unit
class TestOnEventDecorator:
    """Test on_event decorator."""

    def test_on_event_import(self):
        """Test that on_event can be imported."""
        from auroraview import on_event

        assert on_event is not None
        assert callable(on_event)

    def test_on_event_basic(self):
        """Test basic on_event decorator usage."""
        from auroraview import on_event

        @on_event("test_event")
        def handler(data):
            return data

        assert hasattr(handler, "_event_name")
        assert handler._event_name == "test_event"

    def test_on_event_with_webview(self):
        """Test on_event decorator with WebView."""
        try:
            from auroraview import WebView, on_event

            webview = WebView()

            @on_event("test_event", webview)
            def handler(data):
                return data

            assert "test_event" in webview._event_handlers
        except ImportError:
            pytest.skip("Package not built yet")

    def test_on_event_preserves_function(self):
        """Test that on_event preserves function behavior."""
        from auroraview import on_event

        @on_event("test_event")
        def handler(data):
            return data * 2

        result = handler(5)
        assert result == 10


@pytest.mark.unit
class TestThrottleDecorator:
    """Test throttle decorator."""

    def test_throttle_import(self):
        """Test that throttle can be imported."""
        from auroraview.decorators import throttle

        assert throttle is not None
        assert callable(throttle)

    def test_throttle_basic(self):
        """Test basic throttle functionality."""
        from auroraview.decorators import throttle

        call_count = [0]

        @throttle(0.1)
        def handler():
            call_count[0] += 1

        # Call multiple times rapidly
        for _ in range(5):
            handler()

        # Should only be called once due to throttling
        assert call_count[0] <= 2  # Allow some tolerance

    def test_throttle_respects_interval(self):
        """Test that throttle respects the interval."""
        from auroraview.decorators import throttle

        call_count = [0]

        @throttle(0.05)
        def handler():
            call_count[0] += 1

        # First call
        handler()
        assert call_count[0] == 1

        # Immediate second call (should be throttled)
        handler()
        assert call_count[0] == 1

        # Wait and call again
        time.sleep(0.06)
        handler()
        assert call_count[0] == 2


@pytest.mark.unit
class TestDebounceDecorator:
    """Test debounce decorator."""

    def test_debounce_import(self):
        """Test that debounce can be imported."""
        from auroraview.decorators import debounce

        assert debounce is not None
        assert callable(debounce)

    def test_debounce_basic(self):
        """Test basic debounce functionality."""
        from auroraview.decorators import debounce

        call_count = [0]

        @debounce(0.05)
        def handler():
            call_count[0] += 1

        # Call multiple times rapidly
        for _ in range(5):
            handler()

        # Wait for debounce to complete
        time.sleep(0.1)

        # Should only be called once
        assert call_count[0] == 1

    def test_debounce_cancels_previous(self):
        """Test that debounce cancels previous calls."""
        from auroraview.decorators import debounce

        call_count = [0]

        @debounce(0.05)
        def handler():
            call_count[0] += 1

        # First call
        handler()
        time.sleep(0.02)

        # Second call (cancels first)
        handler()
        time.sleep(0.02)

        # Third call (cancels second)
        handler()

        # Wait for debounce to complete
        time.sleep(0.1)

        # Should only be called once (the last one)
        assert call_count[0] == 1

    def test_debounce_with_arguments(self):
        """Test debounce with function arguments."""
        from auroraview.decorators import debounce

        results = []

        @debounce(0.05)
        def handler(value):
            results.append(value)

        handler(1)
        handler(2)
        handler(3)

        time.sleep(0.1)

        # Should have the last value
        assert len(results) == 1
        assert results[0] == 3
