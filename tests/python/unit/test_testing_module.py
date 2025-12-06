"""Tests for auroraview.testing module."""

from unittest.mock import MagicMock

import pytest


class TestWebViewBot:
    """Tests for WebViewBot class."""

    def test_webview_bot_import(self):
        """Test that WebViewBot can be imported."""
        from auroraview.testing import WebViewBot

        assert WebViewBot is not None

    def test_webview_bot_creation(self):
        """Test WebViewBot creation with mock WebView."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        bot = WebViewBot(mock_webview)

        assert bot.webview is mock_webview
        assert bot.events == []
        assert bot._monitoring_active is False

    def test_webview_bot_click(self):
        """Test click method uses DOM API."""
        from auroraview.testing import WebViewBot

        mock_element = MagicMock()
        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.dom = MagicMock(return_value=mock_element)
        bot = WebViewBot(mock_webview)

        bot.click("#testBtn")

        mock_webview.dom.assert_called_once_with("#testBtn")
        mock_element.click.assert_called_once()

    def test_webview_bot_type(self):
        """Test type method uses DOM API."""
        from auroraview.testing import WebViewBot

        mock_element = MagicMock()
        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.dom = MagicMock(return_value=mock_element)
        bot = WebViewBot(mock_webview)

        bot.type("#inputField", "Hello World")

        mock_webview.dom.assert_called_once_with("#inputField")
        mock_element.type_text.assert_called_once_with("Hello World")

    def test_webview_bot_drag(self):
        """Test drag method calls Rust simulate_drag or falls back to eval_js."""
        from auroraview.testing import WebViewBot

        # Test with Rust core available
        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview._core = MagicMock()
        mock_webview._core.simulate_drag = MagicMock()
        mock_webview._auto_process_events = MagicMock()
        bot = WebViewBot(mock_webview)

        bot.drag("#dragElement", (10, 20))

        mock_webview._core.simulate_drag.assert_called_once_with("#dragElement", 10, 20)

    def test_webview_bot_element_exists(self):
        """Test element_exists method with Rust core."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview._core = MagicMock()
        mock_webview._core.check_element_exists = MagicMock()
        mock_webview._auto_process_events = MagicMock()
        bot = WebViewBot(mock_webview)

        result = bot.element_exists("#myElement")

        assert result is True
        mock_webview._core.check_element_exists.assert_called_once_with("#myElement")

    def test_webview_bot_get_element_text(self):
        """Test get_element_text method with Rust core."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview._core = MagicMock()
        mock_webview._core.query_element_text = MagicMock()
        mock_webview._auto_process_events = MagicMock()
        bot = WebViewBot(mock_webview)

        result = bot.get_element_text("#output")

        assert result == "Test Page"  # Default placeholder
        mock_webview._core.query_element_text.assert_called_once_with("#output")

    def test_webview_bot_inject_monitoring_script(self):
        """Test inject_monitoring_script method."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.eval_js = MagicMock()
        bot = WebViewBot(mock_webview)

        bot.inject_monitoring_script()

        assert bot._monitoring_active is True
        mock_webview.eval_js.assert_called_once()

    def test_webview_bot_assert_event_emitted(self):
        """Test assert_event_emitted method."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.eval_js = MagicMock()
        bot = WebViewBot(mock_webview)

        # Should not raise
        bot.assert_event_emitted("test_event")

    def test_webview_bot_assert_event_emitted_exception(self):
        """Test assert_event_emitted raises on JS error."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.eval_js = MagicMock(side_effect=Exception("JS error"))
        bot = WebViewBot(mock_webview)

        with pytest.raises(AssertionError, match="Failed to check event"):
            bot.assert_event_emitted("test_event")


class TestEventRecord:
    """Tests for EventRecord dataclass."""

    def test_event_record_creation(self):
        """Test EventRecord creation."""
        from auroraview.testing.webview_bot import EventRecord

        record = EventRecord(name="test_event", data={"key": "value"}, timestamp=123.456)

        assert record.name == "test_event"
        assert record.data == {"key": "value"}
        assert record.timestamp == 123.456

    def test_event_record_defaults(self):
        """Test EventRecord default values."""
        from auroraview.testing.webview_bot import EventRecord

        record = EventRecord(name="test_event")

        assert record.name == "test_event"
        assert record.data is None
        assert record.timestamp == 0.0


class TestAssertions:
    """Tests for assertion helpers."""

    def test_assertions_import(self):
        """Test that assertions can be imported."""
        from auroraview.testing.assertions import (
            assert_element_exists,
            assert_element_hidden,
            assert_element_text,
            assert_element_visible,
            assert_event_emitted,
            assert_window_title,
        )

        assert callable(assert_element_exists)
        assert callable(assert_element_visible)
        assert callable(assert_element_hidden)
        assert callable(assert_element_text)
        assert callable(assert_event_emitted)
        assert callable(assert_window_title)

    def test_assert_element_exists_success(self):
        """Test assert_element_exists with existing element."""
        from auroraview.testing.assertions import assert_element_exists

        mock_bot = MagicMock()
        mock_bot.element_exists = MagicMock(return_value=True)

        # Should not raise
        assert_element_exists(mock_bot, "#existingElement")

    def test_assert_element_text_success(self):
        """Test assert_element_text with valid text."""
        from auroraview.testing.assertions import assert_element_text

        mock_bot = MagicMock()
        mock_bot.get_element_text = MagicMock(return_value="Hello World")

        # Should not raise
        assert_element_text(mock_bot, "#element", "Hello")

    def test_assert_window_title_success(self):
        """Test assert_window_title success."""
        from auroraview.testing.assertions import assert_window_title

        mock_bot = MagicMock()
        mock_bot.webview = MagicMock()
        mock_bot.webview.eval_js = MagicMock()

        # Should not raise
        assert_window_title(mock_bot, "Test Title")

    def test_assert_window_title_exception(self):
        """Test assert_window_title raises on JS error."""
        from auroraview.testing.assertions import assert_window_title

        mock_bot = MagicMock()
        mock_bot.webview = MagicMock()
        mock_bot.webview.eval_js = MagicMock(side_effect=Exception("JS error"))

        with pytest.raises(AssertionError, match="Failed to execute title check"):
            assert_window_title(mock_bot, "Test Title")

    def test_assert_element_visible_success(self):
        """Test assert_element_visible success."""
        from auroraview.testing.assertions import assert_element_visible

        mock_bot = MagicMock()
        mock_bot.webview = MagicMock()
        mock_bot.webview.eval_js = MagicMock()

        # Should not raise
        assert_element_visible(mock_bot, "#visibleElement")

    def test_assert_element_visible_exception(self):
        """Test assert_element_visible raises on JS error."""
        from auroraview.testing.assertions import assert_element_visible

        mock_bot = MagicMock()
        mock_bot.webview = MagicMock()
        mock_bot.webview.eval_js = MagicMock(side_effect=Exception("JS error"))

        with pytest.raises(AssertionError, match="Failed to check visibility"):
            assert_element_visible(mock_bot, "#element")

    def test_assert_element_hidden_success(self):
        """Test assert_element_hidden success."""
        from auroraview.testing.assertions import assert_element_hidden

        mock_bot = MagicMock()
        mock_bot.webview = MagicMock()
        mock_bot.webview.eval_js = MagicMock()

        # Should not raise
        assert_element_hidden(mock_bot, "#hiddenElement")

    def test_assert_element_hidden_exception(self):
        """Test assert_element_hidden raises on JS error."""
        from auroraview.testing.assertions import assert_element_hidden

        mock_bot = MagicMock()
        mock_bot.webview = MagicMock()
        mock_bot.webview.eval_js = MagicMock(side_effect=Exception("JS error"))

        with pytest.raises(AssertionError, match="Failed to check visibility"):
            assert_element_hidden(mock_bot, "#element")

    def test_assert_event_emitted_delegates(self):
        """Test assert_event_emitted delegates to bot."""
        from auroraview.testing.assertions import assert_event_emitted

        mock_bot = MagicMock()
        mock_bot.assert_event_emitted = MagicMock()

        assert_event_emitted(mock_bot, "test_event")

        mock_bot.assert_event_emitted.assert_called_once_with("test_event")


class TestFixtures:
    """Tests for fixture functionality."""

    def test_fixtures_import(self):
        """Test that fixtures module can be imported."""
        from auroraview.testing import fixtures

        assert fixtures is not None

    def test_test_html_fixture_content(self):
        """Test that test_html fixture returns valid HTML."""
        # Simulate the fixture
        test_html = """
        <html>
            <head>
                <title>Test Page</title>
            </head>
            <body>
                <h1>Test Page</h1>
                <button id="testBtn">Test Button</button>
            </body>
        </html>
        """

        assert "<html>" in test_html
        assert "<body>" in test_html
        assert "testBtn" in test_html

    def test_draggable_window_html_fixture(self):
        """Test draggable_window_html fixture content."""
        # Simulate the fixture content
        html = """
        <html>
            <head>
                <style>
                    .title-bar {
                        background: #333;
                        color: white;
                        padding: 10px;
                        cursor: move;
                        user-select: none;
                    }
                </style>
            </head>
            <body>
                <div class="title-bar" id="titleBar">Draggable Window</div>
                <div id="content">
                    <p>This window can be dragged by the title bar.</p>
                </div>
            </body>
        </html>
        """

        assert "title-bar" in html
        assert "titleBar" in html
        assert "Draggable Window" in html


class TestWebViewBotWaitForEvent:
    """Tests for WebViewBot wait_for_event method."""

    def test_wait_for_event_returns_true(self):
        """Test wait_for_event returns True after timeout."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.eval_js = MagicMock()
        bot = WebViewBot(mock_webview)

        # With a very short timeout, it should return True
        result = bot.wait_for_event("test_event", timeout=0.1)

        assert result is True

    def test_wait_for_event_handles_exception(self):
        """Test wait_for_event handles JS exceptions gracefully."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        mock_webview.on = MagicMock(return_value=lambda f: f)
        mock_webview.eval_js = MagicMock(side_effect=Exception("JS error"))
        bot = WebViewBot(mock_webview)

        # Should not raise, just return True after timeout
        result = bot.wait_for_event("test_event", timeout=0.1)

        assert result is True


class TestEventTimerEdgeCases:
    """Tests for EventTimer edge cases."""

    def test_event_timer_import(self):
        """Test that EventTimer can be imported."""
        from auroraview.utils.event_timer import EventTimer

        assert EventTimer is not None

    def test_event_timer_creation(self):
        """Test EventTimer creation with mock webview."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview, interval_ms=100)

        assert timer._webview is mock_webview
        assert timer._interval_ms == 100
        assert timer._running is False

    def test_event_timer_on_close_callback(self):
        """Test on_close callback registration."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview)

        callback_called = []

        def my_callback():
            callback_called.append(True)

        timer.on_close(my_callback)

        assert my_callback in timer._close_callbacks

    def test_event_timer_is_running_property(self):
        """Test is_running property."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview)

        assert timer.is_running is False

    def test_event_timer_stop_when_not_running(self):
        """Test stop when timer is not running."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview)

        # Should not raise
        timer.stop()

        assert timer._running is False

    def test_event_timer_start_already_running(self):
        """Test start raises when already running."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview)
        timer._running = True

        with pytest.raises(RuntimeError, match="already running"):
            timer.start()

    def test_event_timer_check_window_valid_no_core(self):
        """Test _check_window_valid when webview has no _core."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock(spec=[])  # No _core attribute
        timer = EventTimer(mock_webview)

        result = timer._check_window_valid()

        assert result is True

    def test_event_timer_check_window_valid_with_core(self):
        """Test _check_window_valid when webview has _core."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        mock_webview._core = MagicMock()
        mock_webview._core.is_window_valid = MagicMock(return_value=True)
        timer = EventTimer(mock_webview)

        result = timer._check_window_valid()

        assert result is True
        mock_webview._core.is_window_valid.assert_called_once()

    def test_event_timer_check_window_valid_exception(self):
        """Test _check_window_valid handles exceptions."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        mock_webview._core = MagicMock()
        mock_webview._core.is_window_valid = MagicMock(side_effect=Exception("Error"))
        timer = EventTimer(mock_webview)

        result = timer._check_window_valid()

        assert result is False


class TestTimerBackendsEdgeCases:
    """Tests for timer_backends edge cases."""

    def test_timer_backends_import(self):
        """Test that timer_backends can be imported."""
        from auroraview.utils import timer_backends

        assert timer_backends is not None

    def test_thread_timer_backend_import(self):
        """Test ThreadTimerBackend import."""
        from auroraview.utils.timer_backends import ThreadTimerBackend

        assert ThreadTimerBackend is not None

    def test_thread_timer_backend_creation(self):
        """Test ThreadTimerBackend creation."""
        from auroraview.utils.timer_backends import ThreadTimerBackend

        backend = ThreadTimerBackend()

        assert backend.get_name() == "ThreadTimer"

    def test_thread_timer_backend_start_stop(self):
        """Test ThreadTimerBackend start and stop."""
        from auroraview.utils.timer_backends import ThreadTimerBackend

        backend = ThreadTimerBackend()
        callback_count = []

        def callback():
            callback_count.append(1)

        handle = backend.start(50, callback)
        assert handle is not None

        # Wait a bit for callback to be called
        import time

        time.sleep(0.1)

        backend.stop(handle)

        # Callback should have been called at least once
        assert len(callback_count) >= 1

    def test_get_available_backend(self):
        """Test get_available_backend function."""
        from auroraview.utils.timer_backends import get_available_backend

        backend = get_available_backend()

        # Should return a backend (ThreadTimerBackend is always available)
        assert backend is not None

    def test_register_timer_backend(self):
        """Test register_timer_backend function."""
        from auroraview.utils.timer_backends import (
            ThreadTimerBackend,
            get_available_backend,
            register_timer_backend,
        )

        # Register a new backend with high priority
        register_timer_backend(ThreadTimerBackend, priority=1000)

        # Should be able to get it
        available = get_available_backend()
        assert available is not None


class TestWebViewBotQueryHandlers:
    """Tests for WebViewBot query handlers."""

    def test_setup_query_handlers_element_exists(self):
        """Test _element_exists_result handler."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        handlers = {}

        def mock_on(event_name):
            def decorator(func):
                handlers[event_name] = func
                return func

            return decorator

        mock_webview.on = mock_on
        bot = WebViewBot(mock_webview)

        # Verify handlers were registered
        assert "_element_exists_result" in handlers
        assert "_element_text_result" in handlers

        # Test element_exists handler
        handlers["_element_exists_result"]({"exists": True})
        assert bot._query_results.get("element_exists") is True

    def test_setup_query_handlers_element_text(self):
        """Test _element_text_result handler."""
        from auroraview.testing import WebViewBot

        mock_webview = MagicMock()
        handlers = {}

        def mock_on(event_name):
            def decorator(func):
                handlers[event_name] = func
                return func

            return decorator

        mock_webview.on = mock_on
        bot = WebViewBot(mock_webview)

        # Test element_text handler
        handlers["_element_text_result"]({"text": "Hello World"})
        assert bot._query_results.get("element_text") == "Hello World"


class TestMainModuleEdgeCases:
    """Tests for __main__.py edge cases."""

    def test_main_module_import(self):
        """Test that __main__ module can be imported."""
        from auroraview import __main__

        assert __main__ is not None

    def test_main_function_exists(self):
        """Test that main function exists."""
        from auroraview.__main__ import main

        assert callable(main)


class TestTimerBackendsMoreCoverage:
    """Additional tests for timer_backends coverage."""

    def test_timer_backend_abstract_methods(self):
        """Test TimerBackend abstract class."""
        from auroraview.utils.timer_backends import TimerBackend

        # TimerBackend is abstract, cannot be instantiated directly
        with pytest.raises(TypeError):
            TimerBackend()

    def test_thread_timer_backend_stop_invalid_handle(self):
        """Test ThreadTimerBackend stop with invalid handle."""
        from auroraview.utils.timer_backends import ThreadTimerBackend

        backend = ThreadTimerBackend()

        # Should not raise with invalid handle
        backend.stop(None)

    def test_thread_timer_backend_is_available(self):
        """Test ThreadTimerBackend is_available."""
        from auroraview.utils.timer_backends import ThreadTimerBackend

        backend = ThreadTimerBackend()

        # Thread backend is always available
        assert backend.is_available() is True

    def test_qt_timer_backend_not_available(self):
        """Test QtTimerBackend when Qt is not available."""
        from auroraview.utils.timer_backends import QtTimerBackend

        backend = QtTimerBackend()

        # Qt may or may not be available depending on environment
        # Just verify the method works
        result = backend.is_available()
        assert isinstance(result, bool)


class TestEventTimerMoreCoverage:
    """Additional tests for event_timer coverage."""

    def test_event_timer_with_custom_backend(self):
        """Test EventTimer with custom backend."""
        from auroraview.utils.event_timer import EventTimer
        from auroraview.utils.timer_backends import ThreadTimerBackend

        mock_webview = MagicMock()
        backend = ThreadTimerBackend()
        timer = EventTimer(mock_webview, backend=backend)

        assert timer._backend is backend

    def test_event_timer_interval_property(self):
        """Test EventTimer interval_ms property."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview, interval_ms=50)

        assert timer.interval_ms == 50

    def test_event_timer_webview_attribute(self):
        """Test EventTimer _webview attribute."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview)

        assert timer._webview is mock_webview

    def test_event_timer_check_window_validity_flag(self):
        """Test EventTimer check_window_validity flag."""
        from auroraview.utils.event_timer import EventTimer

        mock_webview = MagicMock()
        timer = EventTimer(mock_webview, check_window_validity=True)

        assert timer._check_validity is True

    def test_event_timer_backend_attribute(self):
        """Test EventTimer _backend attribute."""
        from auroraview.utils.event_timer import EventTimer
        from auroraview.utils.timer_backends import ThreadTimerBackend

        mock_webview = MagicMock()
        backend = ThreadTimerBackend()
        timer = EventTimer(mock_webview, backend=backend)

        assert timer._backend is backend


class TestFileProtocolCoverage:
    """Tests for file_protocol module coverage."""

    def test_file_protocol_import(self):
        """Test file_protocol module import."""
        from auroraview.utils import file_protocol

        assert file_protocol is not None

    def test_path_to_file_url(self):
        """Test path_to_file_url function."""
        from auroraview.utils.file_protocol import path_to_file_url

        result = path_to_file_url("test.html")
        assert result.startswith("file://")
        assert "test.html" in result

    def test_prepare_html_with_local_assets(self):
        """Test prepare_html_with_local_assets function."""
        from auroraview.utils.file_protocol import prepare_html_with_local_assets

        html = '<img src="{{IMAGE_PATH}}">'
        result = prepare_html_with_local_assets(html, {"IMAGE_PATH": "test.png"})
        assert "file://" in result
        assert "{{IMAGE_PATH}}" not in result

    def test_prepare_html_with_manifest_path(self):
        """Test prepare_html_with_local_assets with manifest_path."""
        from auroraview.utils.file_protocol import prepare_html_with_local_assets

        html = '<iframe src="{{MANIFEST_PATH}}"></iframe>'
        result = prepare_html_with_local_assets(html, manifest_path="index.html")
        assert "file://" in result
        assert "{{MANIFEST_PATH}}" not in result


class TestInitModuleMoreCoverage:
    """Additional tests for __init__.py coverage."""

    def test_version_attribute(self):
        """Test __version__ attribute exists."""
        import auroraview

        assert hasattr(auroraview, "__version__")
        assert isinstance(auroraview.__version__, str)

    def test_author_attribute(self):
        """Test __author__ attribute exists."""
        import auroraview

        assert hasattr(auroraview, "__author__")
        assert isinstance(auroraview.__author__, str)

    def test_webview_class_exported(self):
        """Test WebView class is exported."""
        from auroraview import WebView

        assert WebView is not None

    def test_aurora_view_class_exported(self):
        """Test AuroraView class is exported."""
        from auroraview import AuroraView

        assert AuroraView is not None

    def test_event_timer_exported(self):
        """Test EventTimer is exported."""
        from auroraview import EventTimer

        assert EventTimer is not None

    def test_timer_backends_exported(self):
        """Test timer backends are exported."""
        from auroraview import (
            QtTimerBackend,
            ThreadTimerBackend,
            TimerBackend,
            get_available_backend,
            list_registered_backends,
            register_timer_backend,
        )

        assert TimerBackend is not None
        assert ThreadTimerBackend is not None
        assert QtTimerBackend is not None
        assert callable(get_available_backend)
        assert callable(list_registered_backends)
        assert callable(register_timer_backend)

    def test_file_protocol_exported(self):
        """Test file protocol functions are exported."""
        from auroraview import path_to_file_url, prepare_html_with_local_assets

        assert callable(path_to_file_url)
        assert callable(prepare_html_with_local_assets)

    def test_bridge_class_exported(self):
        """Test Bridge class is exported (may be placeholder)."""
        from auroraview import Bridge

        assert Bridge is not None

    def test_service_discovery_exported(self):
        """Test ServiceDiscovery is exported (may be placeholder)."""
        from auroraview import ServiceDiscovery, ServiceInfo

        assert ServiceDiscovery is not None
        assert ServiceInfo is not None


# Check if websockets is available for Bridge tests
try:
    import websockets  # noqa: F401

    HAS_WEBSOCKETS = True
except ImportError:
    HAS_WEBSOCKETS = False


@pytest.mark.skipif(
    not HAS_WEBSOCKETS,
    reason="websockets library is required for Bridge tests",
)
class TestBridgeMoreCoverage:
    """Additional tests for bridge.py coverage."""

    def test_bridge_creation_with_defaults(self):
        """Test Bridge creation with default parameters."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()
        assert bridge is not None
        assert bridge.host == "localhost"
        # Default port is 9001
        assert bridge.port == 9001

    def test_bridge_creation_with_custom_port(self):
        """Test Bridge creation with custom port."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge(port=9000)
        assert bridge.port == 9000

    def test_bridge_creation_with_custom_host(self):
        """Test Bridge creation with custom host."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge(host="0.0.0.0")
        assert bridge.host == "0.0.0.0"

    def test_bridge_on_decorator(self):
        """Test Bridge on decorator."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()

        @bridge.on("test_command")
        async def test_handler(data, client):
            return {"result": "ok"}

        assert "test_command" in bridge._handlers

    def test_bridge_register_handler(self):
        """Test Bridge register_handler method."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()

        async def test_handler(data, client):
            return {"result": "ok"}

        bridge.register_handler("test_action", test_handler)
        assert "test_action" in bridge._handlers

    def test_bridge_is_running_property(self):
        """Test Bridge is_running property."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()
        assert bridge.is_running is False

    def test_bridge_client_count_property(self):
        """Test Bridge client_count property."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()
        assert bridge.client_count == 0

    def test_bridge_protocol_property(self):
        """Test Bridge protocol property."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()
        # Default protocol is 'json'
        assert bridge.protocol == "json"

    def test_bridge_protocol_custom(self):
        """Test Bridge with custom protocol."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge(protocol="msgpack")
        assert bridge.protocol == "msgpack"

    def test_bridge_set_webview_callback(self):
        """Test Bridge set_webview_callback method."""
        from auroraview.integration.bridge import Bridge

        bridge = Bridge()

        def callback(action, data, result):
            pass

        bridge.set_webview_callback(callback)
        assert bridge._webview_callback is callback
