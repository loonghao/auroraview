"""Tests for auroraview.__init__ module exports and utilities."""


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
    """Tests for window utility functions.

    Note: These utilities require Rust core which may not be available in dev environment.
    Tests check that the exports exist (may be None if core not available).
    """

    def test_window_info_available(self):
        """Test that WindowInfo is exported (may be None if core not available)."""
        import auroraview

        assert hasattr(auroraview, "WindowInfo")

    def test_get_foreground_window_available(self):
        """Test that get_foreground_window is exported."""
        import auroraview

        assert hasattr(auroraview, "get_foreground_window")

    def test_find_windows_by_title_available(self):
        """Test that find_windows_by_title is exported."""
        import auroraview

        assert hasattr(auroraview, "find_windows_by_title")

    def test_get_all_windows_available(self):
        """Test that get_all_windows is exported."""
        import auroraview

        assert hasattr(auroraview, "get_all_windows")


class TestCliUtilities:
    """Tests for CLI utility functions.

    Note: These utilities require Rust core which may not be available in dev environment.
    """

    def test_normalize_url_available(self):
        """Test that normalize_url is exported."""
        import auroraview

        assert hasattr(auroraview, "normalize_url")

    def test_rewrite_html_for_custom_protocol_available(self):
        """Test that rewrite_html_for_custom_protocol is exported."""
        import auroraview

        assert hasattr(auroraview, "rewrite_html_for_custom_protocol")

    def test_run_standalone_available(self):
        """Test that run_standalone is exported."""
        import auroraview

        assert hasattr(auroraview, "run_standalone")


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
    """Extended tests for window utility functions.

    Note: These utilities require Rust core which may not be available in dev environment.
    """

    def test_find_window_by_exact_title_available(self):
        """Test that find_window_by_exact_title is exported."""
        import auroraview

        assert hasattr(auroraview, "find_window_by_exact_title")

    def test_close_window_by_hwnd_available(self):
        """Test that close_window_by_hwnd is exported."""
        import auroraview

        assert hasattr(auroraview, "close_window_by_hwnd")

    def test_destroy_window_by_hwnd_available(self):
        """Test that destroy_window_by_hwnd is exported."""
        import auroraview

        assert hasattr(auroraview, "destroy_window_by_hwnd")


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
