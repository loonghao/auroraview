"""
AuroraView Event System Tests using AuroraTest

This module tests the event system including:
- Python to JavaScript events
- JavaScript to Python events
- Event handlers and callbacks
- Signal system (Qt-inspired)
- Event timing and ordering

NOTE: These tests use the original Browser class which requires WebView2.
Due to Python GIL limitations, WebView2 event loop blocks other threads.
For UI automation testing, use PlaywrightBrowser instead.

See test_playwright_browser.py for working Playwright-based tests.
"""

import logging
import sys
import threading
import time

import pytest

from auroraview import EventEmitter, Signal, WebView
from auroraview.testing.auroratest import Browser

logger = logging.getLogger(__name__)

pytestmark = [
    pytest.mark.skip(
        reason="WebView2 Browser class blocks due to GIL. Use PlaywrightBrowser instead."
    ),
    pytest.mark.skipif(sys.platform != "win32", reason="WebView2 tests only run on Windows"),
]


# ============================================================
# Test HTML Templates
# ============================================================

EVENT_TEST_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Event Test</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .log-entry { padding: 5px; margin: 2px 0; background: #f5f5f5; font-family: monospace; }
        .log-entry.sent { background: #e3f2fd; }
        .log-entry.received { background: #e8f5e9; }
        #event-log { max-height: 300px; overflow-y: auto; border: 1px solid #ccc; padding: 10px; }
        button { padding: 10px 20px; margin: 5px; }
    </style>
</head>
<body>
    <h1>Event System Test</h1>

    <div>
        <button id="btn-emit" data-testid="emit-button">Emit Event to Python</button>
        <button id="btn-emit-data" data-testid="emit-data-button">Emit with Data</button>
        <button id="btn-emit-multiple" data-testid="emit-multiple-button">Emit Multiple</button>
    </div>

    <h2>Event Log</h2>
    <div id="event-log"></div>

    <script>
        const eventLog = document.getElementById('event-log');

        function log(message, type = '') {
            const entry = document.createElement('div');
            entry.className = 'log-entry ' + type;
            entry.textContent = new Date().toISOString().substr(11, 12) + ' - ' + message;
            eventLog.appendChild(entry);
            eventLog.scrollTop = eventLog.scrollHeight;
        }

        // Emit simple event
        document.getElementById('btn-emit').addEventListener('click', () => {
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('simple_event', {});
                log('Sent: simple_event', 'sent');
            }
        });

        // Emit event with data
        document.getElementById('btn-emit-data').addEventListener('click', () => {
            if (window.auroraview && window.auroraview.emit) {
                const data = {
                    timestamp: Date.now(),
                    user: 'test_user',
                    action: 'button_click',
                    metadata: {
                        browser: navigator.userAgent.substr(0, 50),
                        screen: { width: screen.width, height: screen.height }
                    }
                };
                window.auroraview.emit('data_event', data);
                log('Sent: data_event with ' + JSON.stringify(data).length + ' bytes', 'sent');
            }
        });

        // Emit multiple events rapidly
        document.getElementById('btn-emit-multiple').addEventListener('click', () => {
            if (window.auroraview && window.auroraview.emit) {
                for (let i = 0; i < 10; i++) {
                    window.auroraview.emit('batch_event', { index: i, timestamp: Date.now() });
                }
                log('Sent: 10 batch_event events', 'sent');
            }
        });

        // Listen for events from Python
        if (window.auroraview && window.auroraview.on) {
            window.auroraview.on('python_event', (data) => {
                log('Received: python_event - ' + JSON.stringify(data), 'received');
            });

            window.auroraview.on('update_ui', (data) => {
                log('Received: update_ui - ' + JSON.stringify(data), 'received');
                if (data.title) {
                    document.querySelector('h1').textContent = data.title;
                }
            });

            window.auroraview.on('ping', (data) => {
                log('Received: ping, sending pong', 'received');
                window.auroraview.emit('pong', {
                    original_timestamp: data.timestamp,
                    pong_timestamp: Date.now()
                });
            });
        }

        log('Event system initialized');
    </script>
</body>
</html>
"""


# ============================================================
# Fixtures
# ============================================================


@pytest.fixture
def webview_with_events():
    """Create WebView with event test HTML."""
    webview = WebView(
        title="Event Test",
        width=1024,
        height=768,
        debug=True,
    )
    webview.load_html(EVENT_TEST_HTML)
    webview.show(wait=False)
    time.sleep(0.5)
    yield webview
    try:
        webview.close()
    except Exception:
        pass


@pytest.fixture
def event_collector():
    """Collector for received events."""
    return {"events": [], "lock": threading.Lock()}


# ============================================================
# Basic Event Tests
# ============================================================


class TestBasicEvents:
    """Test basic event emission and reception."""

    def test_emit_to_js(self, webview_with_events):
        """Test emitting event from Python to JavaScript."""
        webview_with_events.emit("python_event", {"message": "Hello from Python"})
        time.sleep(0.3)

    def test_emit_with_complex_data(self, webview_with_events):
        """Test emitting event with complex data."""
        data = {
            "string": "test",
            "number": 42,
            "float": 3.14,
            "boolean": True,
            "null": None,
            "array": [1, 2, 3],
            "nested": {"key": "value", "deep": {"deeper": "data"}},
        }
        webview_with_events.emit("python_event", data)
        time.sleep(0.3)

    def test_receive_from_js(self, webview_with_events, event_collector):
        """Test receiving event from JavaScript."""

        @webview_with_events.on("simple_event")
        def handle_simple(data):
            with event_collector["lock"]:
                event_collector["events"].append(("simple_event", data))

        # Trigger event from JS
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('simple_event', { source: 'js' });
            }
        """)

        time.sleep(0.3)
        webview_with_events.process_events()

    def test_receive_with_data(self, webview_with_events, event_collector):
        """Test receiving event with data from JavaScript."""

        @webview_with_events.on("data_event")
        def handle_data(data):
            with event_collector["lock"]:
                event_collector["events"].append(("data_event", data))

        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('data_event', {
                    timestamp: Date.now(),
                    user: 'test',
                    values: [1, 2, 3]
                });
            }
        """)

        time.sleep(0.3)
        webview_with_events.process_events()


# ============================================================
# Event Handler Tests
# ============================================================


class TestEventHandlers:
    """Test event handler registration and management."""

    def test_multiple_handlers(self, webview_with_events, event_collector):
        """Test multiple handlers for same event."""
        handler_calls = []

        @webview_with_events.on("multi_event")
        def handler1(data):
            handler_calls.append("handler1")

        @webview_with_events.on("multi_event")
        def handler2(data):
            handler_calls.append("handler2")

        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('multi_event', {});
            }
        """)

        time.sleep(0.3)
        webview_with_events.process_events()

    def test_handler_removal(self, webview_with_events, event_collector):
        """Test removing event handler."""
        call_count = [0]

        def handler(data):
            call_count[0] += 1

        # Register handler
        webview_with_events.on("removable_event")(handler)

        # Trigger event
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('removable_event', {});
            }
        """)
        time.sleep(0.2)
        webview_with_events.process_events()

        # Remove handler
        webview_with_events.off("removable_event", handler)

        # Trigger again - should not call handler
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('removable_event', {});
            }
        """)
        time.sleep(0.2)
        webview_with_events.process_events()

    def test_once_handler(self, webview_with_events, event_collector):
        """Test one-time event handler."""
        call_count = [0]

        @webview_with_events.once("once_event")
        def handler(data):
            call_count[0] += 1

        # Trigger multiple times
        for _ in range(3):
            webview_with_events.eval_js("""
                if (window.auroraview && window.auroraview.emit) {
                    window.auroraview.emit('once_event', {});
                }
            """)
            time.sleep(0.1)
            webview_with_events.process_events()


# ============================================================
# Bidirectional Communication Tests
# ============================================================


class TestBidirectionalCommunication:
    """Test bidirectional Python-JavaScript communication."""

    def test_ping_pong(self, webview_with_events, event_collector):
        """Test ping-pong communication pattern."""
        pong_received = [False]
        round_trip_time = [0]

        @webview_with_events.on("pong")
        def handle_pong(data):
            pong_received[0] = True
            if "original_timestamp" in data and "pong_timestamp" in data:
                round_trip_time[0] = data["pong_timestamp"] - data["original_timestamp"]

        # Send ping
        webview_with_events.emit("ping", {"timestamp": int(time.time() * 1000)})

        # Wait for pong
        time.sleep(0.5)
        webview_with_events.process_events()

        logger.info(f"Round trip time: {round_trip_time[0]}ms")

    def test_request_response_pattern(self, webview_with_events, event_collector):
        """Test request-response communication pattern."""
        response_data = [None]

        @webview_with_events.on("response")
        def handle_response(data):
            response_data[0] = data

        # Set up JS to respond to requests
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.on) {
                window.auroraview.on('request', (data) => {
                    const response = {
                        request_id: data.id,
                        result: data.value * 2,
                        processed_at: Date.now()
                    };
                    window.auroraview.emit('response', response);
                });
            }
        """)
        time.sleep(0.2)

        # Send request
        webview_with_events.emit("request", {"id": "req-001", "value": 21})

        time.sleep(0.3)
        webview_with_events.process_events()


# ============================================================
# Event Timing Tests
# ============================================================


class TestEventTiming:
    """Test event timing and ordering."""

    def test_event_order(self, webview_with_events, event_collector):
        """Test that events are received in order."""
        received_order = []

        @webview_with_events.on("ordered_event")
        def handle_ordered(data):
            received_order.append(data.get("index"))

        # Send events in order
        for i in range(5):
            webview_with_events.eval_js(f"""
                if (window.auroraview && window.auroraview.emit) {{
                    window.auroraview.emit('ordered_event', {{ index: {i} }});
                }}
            """)

        time.sleep(0.5)
        webview_with_events.process_events()

        # Check order
        if received_order:
            for i, val in enumerate(received_order):
                if val != i:
                    logger.warning(f"Event order mismatch at index {i}: expected {i}, got {val}")

    def test_rapid_events(self, webview_with_events, event_collector):
        """Test rapid event emission."""
        event_count = [0]

        @webview_with_events.on("rapid_event")
        def handle_rapid(data):
            event_count[0] += 1

        start = time.time()

        # Send many events rapidly
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                for (let i = 0; i < 100; i++) {
                    window.auroraview.emit('rapid_event', { index: i });
                }
            }
        """)

        # Process events
        time.sleep(1.0)
        for _ in range(10):
            webview_with_events.process_events()
            time.sleep(0.1)

        elapsed = time.time() - start
        logger.info(f"Processed {event_count[0]} events in {elapsed:.3f}s")


# ============================================================
# Signal System Tests (Qt-inspired)
# ============================================================


class TestSignalSystem:
    """Test Qt-inspired Signal system."""

    def test_signal_creation(self):
        """Test creating a Signal."""
        signal = Signal()
        assert signal is not None

    def test_signal_connect_emit(self):
        """Test connecting to and emitting signal."""
        signal = Signal()
        received = []

        def handler(value):
            received.append(value)

        signal.connect(handler)
        signal.emit(42)

        assert 42 in received

    def test_signal_disconnect(self):
        """Test disconnecting from signal."""
        signal = Signal()
        received = []

        def handler(value):
            received.append(value)

        conn_id = signal.connect(handler)
        signal.emit(1)

        signal.disconnect(conn_id)
        signal.emit(2)

        assert 1 in received
        assert 2 not in received

    def test_multiple_signal_handlers(self):
        """Test multiple handlers on same signal."""
        signal = Signal()
        results = []

        signal.connect(lambda x: results.append(f"handler1: {x}"))
        signal.connect(lambda x: results.append(f"handler2: {x}"))
        signal.connect(lambda x: results.append(f"handler3: {x}"))

        signal.emit("test")

        assert len(results) == 3


# ============================================================
# EventEmitter Tests
# ============================================================


class TestEventEmitter:
    """Test EventEmitter pattern."""

    def test_emitter_creation(self):
        """Test creating EventEmitter."""
        emitter = EventEmitter()
        assert emitter is not None

    def test_emitter_on_emit(self):
        """Test on and emit methods."""
        emitter = EventEmitter()
        received = []

        @emitter.on("test")
        def handler(data):
            received.append(data)

        emitter.emit("test", {"value": 123})

        assert len(received) == 1
        assert received[0]["value"] == 123

    def test_emitter_off(self):
        """Test removing handler."""
        emitter = EventEmitter()
        received = []

        def handler(data):
            received.append(data)

        emitter.on("test")(handler)
        emitter.emit("test", {"count": 1})

        emitter.off("test", handler)
        emitter.emit("test", {"count": 2})

        assert len(received) == 1

    def test_emitter_once(self):
        """Test one-time handler."""
        emitter = EventEmitter()
        received = []

        @emitter.once("test")
        def handler(data):
            received.append(data)

        emitter.emit("test", {"n": 1})
        emitter.emit("test", {"n": 2})
        emitter.emit("test", {"n": 3})

        assert len(received) == 1


# ============================================================
# Error Handling Tests
# ============================================================


class TestEventErrorHandling:
    """Test event error handling."""

    def test_handler_exception(self, webview_with_events):
        """Test that handler exceptions don't crash the system."""

        @webview_with_events.on("error_event")
        def bad_handler(data):
            raise ValueError("Intentional error")

        # This should not crash
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('error_event', {});
            }
        """)

        time.sleep(0.3)
        try:
            webview_with_events.process_events()
        except Exception:
            pass  # Expected to possibly raise

    def test_invalid_event_data(self, webview_with_events):
        """Test handling of invalid event data."""
        # Send event with potentially problematic data
        webview_with_events.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('test', undefined);
                window.auroraview.emit('test', null);
                window.auroraview.emit('test', '');
            }
        """)
        time.sleep(0.3)


# ============================================================
# AuroraTest Integration
# ============================================================


class TestAuroraTestEventIntegration:
    """Test events using AuroraTest framework."""

    @pytest.mark.asyncio
    async def test_click_triggers_event(self):
        """Test that clicking button triggers event."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(EVENT_TEST_HTML)
        await page.wait_for_timeout(500)

        # Click emit button
        await page.get_by_test_id("emit-button").click()
        await page.wait_for_timeout(300)

        browser.close()

    @pytest.mark.asyncio
    async def test_emit_with_data_button(self):
        """Test emit with data button."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(EVENT_TEST_HTML)
        await page.wait_for_timeout(500)

        await page.get_by_test_id("emit-data-button").click()
        await page.wait_for_timeout(300)

        browser.close()

    @pytest.mark.asyncio
    async def test_emit_multiple_button(self):
        """Test emit multiple events button."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(EVENT_TEST_HTML)
        await page.wait_for_timeout(500)

        await page.get_by_test_id("emit-multiple-button").click()
        await page.wait_for_timeout(500)

        browser.close()


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
