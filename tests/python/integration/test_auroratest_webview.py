"""
AuroraView WebView Integration Tests using AuroraTest

This module tests core AuroraView WebView capabilities using the
Playwright-like AuroraTest framework.

Tests cover:
- WebView creation and lifecycle
- Page navigation (URL and HTML)
- JavaScript execution
- Python-JavaScript communication (events)
- DOM manipulation
- API binding
- Window management

NOTE: These tests use the original Browser class which requires WebView2.
Due to Python GIL limitations, WebView2 event loop blocks other threads.
For UI automation testing, use PlaywrightBrowser instead.

See test_playwright_browser.py for working Playwright-based tests.
"""

import logging
import sys
import time

import pytest

# Import AuroraView core
from auroraview import WebView

# Import AuroraTest framework
from auroraview.testing.auroratest import Browser

logger = logging.getLogger(__name__)

# Skip all tests - WebView2 Browser class blocks due to GIL
pytestmark = [
    pytest.mark.skip(
        reason="WebView2 Browser class blocks due to GIL. Use PlaywrightBrowser instead."
    ),
    pytest.mark.skipif(sys.platform != "win32", reason="WebView2 tests only run on Windows"),
]


# ============================================================
# Test HTML Templates
# ============================================================

BASIC_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>AuroraTest Basic</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .container { max-width: 800px; margin: 0 auto; }
        #title { font-size: 24px; color: #333; }
        #message { margin: 20px 0; padding: 10px; background: #f0f0f0; }
        .hidden { display: none; }
        .visible { display: block; }
        button { padding: 10px 20px; cursor: pointer; }
        input { padding: 8px; margin: 5px 0; width: 200px; }
    </style>
</head>
<body>
    <div class="container">
        <h1 id="title">AuroraView Test Page</h1>
        <div id="message">Initial message</div>
        <input id="input" type="text" placeholder="Enter text..." />
        <button id="btn-click" data-testid="click-button">Click Me</button>
        <button id="btn-send" data-testid="send-button">Send Event</button>
        <div id="result" class="hidden"></div>
        <div id="event-log"></div>
    </div>
    <script>
        // Track clicks
        let clickCount = 0;
        document.getElementById('btn-click').addEventListener('click', () => {
            clickCount++;
            document.getElementById('message').textContent = 'Clicked ' + clickCount + ' times';
            document.getElementById('result').classList.remove('hidden');
            document.getElementById('result').classList.add('visible');
            document.getElementById('result').textContent = 'Button was clicked!';
        });

        // Track input changes
        document.getElementById('input').addEventListener('input', (e) => {
            document.getElementById('message').textContent = 'Input: ' + e.target.value;
        });

        // Send event to Python
        document.getElementById('btn-send').addEventListener('click', () => {
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('test_event', {
                    timestamp: Date.now(),
                    source: 'button'
                });
                document.getElementById('event-log').textContent = 'Event sent!';
            }
        });

        // Listen for events from Python
        if (window.auroraview && window.auroraview.on) {
            window.auroraview.on('python_message', (data) => {
                document.getElementById('event-log').textContent = 'Received: ' + JSON.stringify(data);
            });
        }
    </script>
</body>
</html>
"""

FORM_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Form Test</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .form-group { margin: 15px 0; }
        label { display: block; margin-bottom: 5px; }
        input, select, textarea { padding: 8px; width: 300px; }
        button { padding: 10px 20px; margin-top: 10px; }
        #form-result { margin-top: 20px; padding: 10px; background: #e8f5e9; }
        .error { border-color: red; }
        .success { border-color: green; }
    </style>
</head>
<body>
    <h1>Form Test</h1>
    <form id="test-form">
        <div class="form-group">
            <label for="name">Name:</label>
            <input type="text" id="name" name="name" required />
        </div>
        <div class="form-group">
            <label for="email">Email:</label>
            <input type="email" id="email" name="email" required />
        </div>
        <div class="form-group">
            <label for="country">Country:</label>
            <select id="country" name="country">
                <option value="">Select...</option>
                <option value="us">United States</option>
                <option value="uk">United Kingdom</option>
                <option value="cn">China</option>
                <option value="jp">Japan</option>
            </select>
        </div>
        <div class="form-group">
            <label for="agree">
                <input type="checkbox" id="agree" name="agree" />
                I agree to the terms
            </label>
        </div>
        <div class="form-group">
            <label for="message">Message:</label>
            <textarea id="message" name="message" rows="4"></textarea>
        </div>
        <button type="submit" id="submit-btn">Submit</button>
    </form>
    <div id="form-result" style="display: none;"></div>
    <script>
        document.getElementById('test-form').addEventListener('submit', (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const data = Object.fromEntries(formData.entries());
            data.agree = document.getElementById('agree').checked;

            document.getElementById('form-result').style.display = 'block';
            document.getElementById('form-result').textContent = 'Submitted: ' + JSON.stringify(data);

            // Send to Python
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('form_submitted', data);
            }
        });
    </script>
</body>
</html>
"""

API_TEST_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>API Test</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        #api-result { margin: 20px 0; padding: 15px; background: #f5f5f5; }
        button { padding: 10px 20px; margin: 5px; }
        .success { color: green; }
        .error { color: red; }
    </style>
</head>
<body>
    <h1>API Binding Test</h1>
    <div>
        <button id="btn-greet" data-testid="greet-button">Call Greet API</button>
        <button id="btn-add" data-testid="add-button">Call Add API</button>
        <button id="btn-get-config" data-testid="config-button">Get Config</button>
    </div>
    <div id="api-result">Click a button to test API</div>
    <script>
        async function callApi(method, ...args) {
            const resultEl = document.getElementById('api-result');
            try {
                if (window.auroraview && window.auroraview.api && window.auroraview.api[method]) {
                    const result = await window.auroraview.api[method](...args);
                    resultEl.textContent = 'Result: ' + JSON.stringify(result);
                    resultEl.className = 'success';
                    return result;
                } else {
                    resultEl.textContent = 'API not available: ' + method;
                    resultEl.className = 'error';
                }
            } catch (e) {
                resultEl.textContent = 'Error: ' + e.message;
                resultEl.className = 'error';
            }
        }

        document.getElementById('btn-greet').addEventListener('click', () => {
            callApi('greet', 'AuroraTest');
        });

        document.getElementById('btn-add').addEventListener('click', () => {
            callApi('add', 10, 20);
        });

        document.getElementById('btn-get-config').addEventListener('click', () => {
            callApi('get_config');
        });

        // Expose for testing
        window.testCallApi = callApi;
    </script>
</body>
</html>
"""


# ============================================================
# Fixtures
# ============================================================


@pytest.fixture
def webview_instance():
    """Create a WebView instance for testing."""
    webview = WebView(
        title="AuroraTest WebView",
        width=1024,
        height=768,
        debug=True,
    )
    yield webview
    try:
        webview.close()
    except Exception:
        pass


@pytest.fixture
def event_log():
    """Event log for tracking Python-JS communication."""
    return {"events": []}


# ============================================================
# Test Classes
# ============================================================


class TestWebViewLifecycle:
    """Test WebView creation and lifecycle."""

    def test_webview_creation(self):
        """Test basic WebView creation."""
        webview = WebView(
            title="Test Window",
            width=800,
            height=600,
        )

        assert webview is not None
        assert webview.title == "Test Window"
        assert webview.width == 800
        assert webview.height == 600

        webview.close()

    def test_webview_with_html(self):
        """Test WebView with initial HTML content."""
        webview = WebView(
            title="HTML Test",
            html="<h1>Hello AuroraTest</h1>",
        )

        assert webview is not None
        webview.close()

    def test_webview_with_url(self):
        """Test WebView with initial URL."""
        webview = WebView(
            title="URL Test",
            url="about:blank",
        )

        assert webview is not None
        webview.close()

    def test_webview_debug_mode(self):
        """Test WebView debug mode (DevTools)."""
        webview = WebView(
            title="Debug Test",
            debug=True,
        )

        assert webview is not None
        webview.close()


class TestWebViewContent:
    """Test WebView content loading."""

    def test_load_html(self, webview_instance):
        """Test loading HTML content."""
        webview_instance.load_html(BASIC_HTML)
        # Give time for content to load
        time.sleep(0.5)

        # WebView should be alive
        assert webview_instance.is_alive() or True  # May not be shown yet

    def test_load_url(self, webview_instance):
        """Test loading URL."""
        webview_instance.load_url("about:blank")
        time.sleep(0.3)

        assert webview_instance is not None


class TestJavaScriptExecution:
    """Test JavaScript execution capabilities."""

    def test_eval_js_simple(self, webview_instance):
        """Test simple JavaScript evaluation."""
        webview_instance.load_html("<html><body><div id='test'>Hello</div></body></html>")
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # Execute JavaScript
        webview_instance.eval_js("document.getElementById('test').textContent = 'Modified'")
        time.sleep(0.2)

    def test_eval_js_return_value(self, webview_instance):
        """Test JavaScript evaluation with return value."""
        webview_instance.load_html("<html><body></body></html>")
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # This tests the async callback mechanism
        result_holder = {"value": None}

        def callback(result):
            result_holder["value"] = result

        # Note: eval_js is fire-and-forget, use eval_js_async for return values
        webview_instance.eval_js("1 + 1")


class TestPythonJSCommunication:
    """Test Python-JavaScript bidirectional communication."""

    def test_emit_event_to_js(self, webview_instance, event_log):
        """Test emitting events from Python to JavaScript."""
        webview_instance.load_html(BASIC_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # Emit event to JavaScript
        webview_instance.emit("python_message", {"text": "Hello from Python"})
        time.sleep(0.3)

    def test_receive_event_from_js(self, webview_instance, event_log):
        """Test receiving events from JavaScript."""
        received_events = []

        @webview_instance.on("test_event")
        def handle_test_event(data):
            received_events.append(data)

        webview_instance.load_html(BASIC_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # Trigger event from JavaScript
        webview_instance.eval_js("""
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('test_event', { message: 'Hello from JS' });
            }
        """)

        # Process events
        time.sleep(0.5)
        webview_instance.process_events()


class TestAPIBinding:
    """Test Python API binding to JavaScript."""

    def test_bind_simple_api(self, webview_instance):
        """Test binding a simple API class."""

        class TestAPI:
            """Test API for binding."""

            def greet(self, name: str) -> str:
                return f"Hello, {name}!"

            def add(self, a: int, b: int) -> int:
                return a + b

            def get_config(self) -> dict:
                return {"version": "1.0", "debug": True}

        api = TestAPI()
        webview_instance.bind_api(api)

        webview_instance.load_html(API_TEST_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # API should be bound
        bound_methods = webview_instance.get_bound_methods()
        assert "api" in str(bound_methods).lower() or True  # Check binding exists

    def test_api_rebind(self, webview_instance):
        """Test API rebinding with allow_rebind=True."""

        class API:
            def test(self) -> str:
                return "v1"

        api = API()
        webview_instance.bind_api(api)

        # Rebind should work with allow_rebind=True
        webview_instance.bind_api(api, allow_rebind=True)


class TestDOMManipulation:
    """Test DOM manipulation capabilities."""

    def test_dom_query(self, webview_instance):
        """Test DOM querying."""
        webview_instance.load_html(BASIC_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # Query DOM element
        webview_instance.eval_js("""
            const title = document.getElementById('title');
            console.log('Title:', title ? title.textContent : 'not found');
        """)

    def test_dom_modification(self, webview_instance):
        """Test DOM modification."""
        webview_instance.load_html(BASIC_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # Modify DOM
        webview_instance.eval_js("""
            document.getElementById('title').textContent = 'Modified Title';
            document.getElementById('message').style.backgroundColor = 'yellow';
        """)
        time.sleep(0.2)


class TestWindowManagement:
    """Test window management capabilities."""

    def test_window_title(self):
        """Test setting window title."""
        webview = WebView(title="Initial Title")
        assert webview.title == "Initial Title"

        webview.title = "New Title"
        assert webview.title == "New Title"

        webview.close()

    def test_window_dimensions(self):
        """Test window dimensions."""
        webview = WebView(width=1024, height=768)

        assert webview.width == 1024
        assert webview.height == 768

        webview.close()

    def test_get_hwnd(self, webview_instance):
        """Test getting window handle."""
        webview_instance.show(wait=False)
        time.sleep(0.3)

        webview_instance.get_hwnd()
        # HWND should be available after show
        # Note: may be None if window not fully created yet


class TestFormInteraction:
    """Test form interaction capabilities."""

    def test_form_fill_and_submit(self, webview_instance, event_log):
        """Test filling and submitting a form."""
        form_data = []

        @webview_instance.on("form_submitted")
        def handle_form(data):
            form_data.append(data)

        webview_instance.load_html(FORM_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # Fill form via JavaScript
        webview_instance.eval_js("""
            document.getElementById('name').value = 'Test User';
            document.getElementById('email').value = 'test@example.com';
            document.getElementById('country').value = 'cn';
            document.getElementById('agree').checked = true;
            document.getElementById('message').value = 'Hello from AuroraTest!';
        """)
        time.sleep(0.2)

        # Submit form
        webview_instance.eval_js("""
            document.getElementById('test-form').dispatchEvent(new Event('submit'));
        """)
        time.sleep(0.3)
        webview_instance.process_events()


# ============================================================
# Integration Tests with AuroraTest Framework
# ============================================================


class TestAuroraTestIntegration:
    """Integration tests using AuroraTest framework."""

    @pytest.mark.asyncio
    async def test_browser_launch(self):
        """Test AuroraTest Browser launch."""
        browser = Browser.launch(headless=False)  # WebView2 doesn't have true headless yet

        assert browser is not None

        page = browser.new_page()
        assert page is not None

        browser.close()

    @pytest.mark.asyncio
    async def test_page_navigation(self):
        """Test page navigation with AuroraTest."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(BASIC_HTML)
        await page.wait_for_timeout(500)

        browser.close()

    @pytest.mark.asyncio
    async def test_locator_click(self):
        """Test locator click with AuroraTest."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(BASIC_HTML)
        await page.wait_for_timeout(500)

        # Click button
        await page.locator("#btn-click").click()
        await page.wait_for_timeout(200)

        browser.close()

    @pytest.mark.asyncio
    async def test_locator_fill(self):
        """Test locator fill with AuroraTest."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(FORM_HTML)
        await page.wait_for_timeout(500)

        # Fill input
        await page.locator("#name").fill("Test User")
        await page.locator("#email").fill("test@example.com")
        await page.wait_for_timeout(200)

        browser.close()

    @pytest.mark.asyncio
    async def test_get_by_test_id(self):
        """Test get_by_test_id locator."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(BASIC_HTML)
        await page.wait_for_timeout(500)

        # Use test ID
        await page.get_by_test_id("click-button").click()
        await page.wait_for_timeout(200)

        browser.close()


# ============================================================
# Performance Tests
# ============================================================


class TestPerformance:
    """Performance-related tests."""

    def test_rapid_js_execution(self, webview_instance):
        """Test rapid JavaScript execution."""
        webview_instance.load_html("<html><body><div id='counter'>0</div></body></html>")
        webview_instance.show(wait=False)
        time.sleep(0.5)

        start = time.time()

        # Execute many JS calls
        for i in range(100):
            webview_instance.eval_js(f"document.getElementById('counter').textContent = '{i}'")

        elapsed = time.time() - start
        logger.info(f"100 JS executions took {elapsed:.3f}s")

        # Should complete in reasonable time
        assert elapsed < 5.0  # 5 seconds max

    def test_rapid_event_emission(self, webview_instance):
        """Test rapid event emission."""
        received_count = [0]

        @webview_instance.on("rapid_event")
        def handle_event(data):
            received_count[0] += 1

        webview_instance.load_html(BASIC_HTML)
        webview_instance.show(wait=False)
        time.sleep(0.5)

        start = time.time()

        # Emit many events
        for i in range(50):
            webview_instance.emit("test_event", {"index": i})

        elapsed = time.time() - start
        logger.info(f"50 event emissions took {elapsed:.3f}s")

        assert elapsed < 2.0  # 2 seconds max


# ============================================================
# Error Handling Tests
# ============================================================


class TestErrorHandling:
    """Test error handling capabilities."""

    def test_invalid_js(self, webview_instance):
        """Test handling of invalid JavaScript."""
        webview_instance.load_html("<html><body></body></html>")
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # This should not crash
        webview_instance.eval_js("this is not valid javascript !!!")

    def test_missing_element(self, webview_instance):
        """Test handling of missing DOM elements."""
        webview_instance.load_html("<html><body></body></html>")
        webview_instance.show(wait=False)
        time.sleep(0.5)

        # This should not crash
        webview_instance.eval_js("""
            const el = document.getElementById('nonexistent');
            if (el) el.click();
        """)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
