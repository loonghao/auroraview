"""Tests for auroraview.__init__ module exports and utilities."""

import pytest


class TestModuleExports:
    """Tests for module exports."""

    def test_version_available(self):
        """Test that __version__ is available."""
        import auroraview

        assert hasattr(auroraview, "__version__")
        assert isinstance(auroraview.__version__, str)

    def test_author_available(self):
        """Test that __author__ is available."""
        import auroraview

        assert hasattr(auroraview, "__author__")
        assert isinstance(auroraview.__author__, str)

    def test_webview_exported(self):
        """Test that WebView is exported."""
        from auroraview import WebView

        assert WebView is not None

    def test_auroraview_exported(self):
        """Test that AuroraView is exported."""
        from auroraview import AuroraView

        assert AuroraView is not None

    def test_bridge_exported(self):
        """Test that Bridge is exported."""
        from auroraview import Bridge

        assert Bridge is not None

    def test_event_timer_exported(self):
        """Test that EventTimer is exported."""
        from auroraview import EventTimer

        assert EventTimer is not None

    def test_timer_backends_exported(self):
        """Test that timer backends are exported."""
        from auroraview import (
            QtTimerBackend,
            ThreadTimerBackend,
            TimerBackend,
            get_available_backend,
            list_registered_backends,
            register_timer_backend,
        )

        assert TimerBackend is not None
        assert QtTimerBackend is not None
        assert ThreadTimerBackend is not None
        assert callable(register_timer_backend)
        assert callable(get_available_backend)
        assert callable(list_registered_backends)

    def test_file_protocol_utilities_exported(self):
        """Test that file protocol utilities are exported."""
        from auroraview import path_to_file_url, prepare_html_with_local_assets

        assert callable(path_to_file_url)
        assert callable(prepare_html_with_local_assets)


class TestOnEventDecorator:
    """Tests for on_event decorator."""

    def test_on_event_decorator(self):
        """Test on_event decorator registers handlers."""
        import auroraview

        # Clear any existing handlers
        auroraview._EVENT_HANDLERS.clear()

        @auroraview.on_event("test_event")
        def my_handler(data):
            return data

        assert "test_event" in auroraview._EVENT_HANDLERS
        assert my_handler in auroraview._EVENT_HANDLERS["test_event"]

    def test_on_event_multiple_handlers(self):
        """Test on_event with multiple handlers for same event."""
        import auroraview

        auroraview._EVENT_HANDLERS.clear()

        @auroraview.on_event("multi_event")
        def handler1(data):
            pass

        @auroraview.on_event("multi_event")
        def handler2(data):
            pass

        assert len(auroraview._EVENT_HANDLERS["multi_event"]) == 2


class TestWindowUtilities:
    """Tests for window utility functions."""

    def test_window_info_available(self):
        """Test that WindowInfo is available."""
        from auroraview import WindowInfo

        assert WindowInfo is not None

    def test_get_foreground_window_available(self):
        """Test that get_foreground_window is available."""
        from auroraview import get_foreground_window

        assert get_foreground_window is not None

    def test_find_windows_by_title_available(self):
        """Test that find_windows_by_title is available."""
        from auroraview import find_windows_by_title

        assert find_windows_by_title is not None

    def test_get_all_windows_available(self):
        """Test that get_all_windows is available."""
        from auroraview import get_all_windows

        assert get_all_windows is not None


class TestCliUtilities:
    """Tests for CLI utility functions."""

    def test_normalize_url_available(self):
        """Test that normalize_url is available."""
        from auroraview import normalize_url

        assert normalize_url is not None

    def test_rewrite_html_for_custom_protocol_available(self):
        """Test that rewrite_html_for_custom_protocol is available."""
        from auroraview import rewrite_html_for_custom_protocol

        assert rewrite_html_for_custom_protocol is not None

    def test_run_standalone_available(self):
        """Test that run_standalone is available."""
        from auroraview import run_standalone

        assert run_standalone is not None


class TestBackwardCompatibility:
    """Tests for backward compatibility aliases."""

    def test_auroraview_qt_alias(self):
        """Test that AuroraViewQt is an alias for QtWebView."""
        from auroraview import AuroraViewQt, QtWebView

        assert AuroraViewQt is QtWebView


class TestAllExports:
    """Tests for __all__ exports."""

    def test_all_exports_accessible(self):
        """Test that all items in __all__ are accessible."""
        import auroraview

        for name in auroraview.__all__:
            assert hasattr(auroraview, name), f"Missing export: {name}"


class TestServiceDiscovery:
    """Tests for ServiceDiscovery exports."""

    def test_service_discovery_available(self):
        """Test that ServiceDiscovery is available."""
        from auroraview import ServiceDiscovery

        assert ServiceDiscovery is not None

    def test_service_info_available(self):
        """Test that ServiceInfo is available."""
        from auroraview import ServiceInfo

        assert ServiceInfo is not None


class TestWindowUtilitiesExtended:
    """Extended tests for window utility functions."""

    def test_find_window_by_exact_title_available(self):
        """Test that find_window_by_exact_title is available."""
        from auroraview import find_window_by_exact_title

        assert find_window_by_exact_title is not None

    def test_close_window_by_hwnd_available(self):
        """Test that close_window_by_hwnd is available."""
        from auroraview import close_window_by_hwnd

        assert close_window_by_hwnd is not None

    def test_destroy_window_by_hwnd_available(self):
        """Test that destroy_window_by_hwnd is available."""
        from auroraview import destroy_window_by_hwnd

        assert destroy_window_by_hwnd is not None


class TestOnEventDecoratorExtended:
    """Extended tests for on_event decorator."""

    def test_on_event_returns_original_function(self):
        """Test that on_event returns the original function."""
        import auroraview

        auroraview._EVENT_HANDLERS.clear()

        def original_handler(data):
            return data * 2

        decorated = auroraview.on_event("test_return")(original_handler)

        assert decorated is original_handler
        assert decorated(5) == 10

    def test_on_event_different_events(self):
        """Test on_event with different event names."""
        import auroraview

        auroraview._EVENT_HANDLERS.clear()

        @auroraview.on_event("event_a")
        def handler_a(data):
            pass

        @auroraview.on_event("event_b")
        def handler_b(data):
            pass

        assert "event_a" in auroraview._EVENT_HANDLERS
        assert "event_b" in auroraview._EVENT_HANDLERS
        assert len(auroraview._EVENT_HANDLERS) == 2
