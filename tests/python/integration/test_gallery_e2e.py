"""End-to-end tests for Gallery frontend API calls.

These tests use Playwright to verify the frontend correctly calls
the Python backend API with proper parameter formats.

This catches issues like:
- Incorrect parameter format (e.g., passing `pid` instead of `{pid}`)
- Missing required parameters
- Type mismatches between frontend and backend
"""

from __future__ import annotations

import sys

import pytest

# Check if playwright is available
try:
    from importlib.util import find_spec

    PLAYWRIGHT_AVAILABLE = find_spec("playwright") is not None
except ImportError:
    PLAYWRIGHT_AVAILABLE = False

# Skip all tests if Python < 3.8 or playwright not installed
pytestmark = [
    pytest.mark.skipif(sys.version_info < (3, 8), reason="Playwright requires Python 3.8+"),
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.integration,
    pytest.mark.e2e,
]


class TestAuroraViewAPIParameterFormat:
    """Test that frontend API calls use correct parameter formats.

    These tests verify the fix for the kill_process parameter issue where
    the frontend was passing `pid` directly instead of `{pid: number}`.
    """

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_api_call_with_object_params(self):
        """Test that API calls with object params work correctly."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            # Verify api.kill_process receives object params
            result = page.evaluate("""
                (function() {
                    let capturedParams = null;

                    // Override call to capture params
                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {
                        capturedParams = { method, params };
                        return originalCall.call(this, method, params);
                    };

                    // Call with object params (correct format)
                    window.auroraview.api.kill_process({ pid: 12345 });

                    return capturedParams;
                })()
            """)

            assert result["method"] == "api.kill_process"
            # Should be called with object, not array
            assert result["params"] == [{"pid": 12345}]

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_api_call_with_single_value_becomes_array(self):
        """Test that single value params become array (current behavior)."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            # When calling api.method(value), value becomes [value]
            result = page.evaluate("""
                (function() {
                    let capturedParams = null;

                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {
                        capturedParams = { method, params };
                        return originalCall.call(this, method, params);
                    };

                    // Call with single value (old incorrect format)
                    window.auroraview.api.some_method(12345);

                    return capturedParams;
                })()
            """)

            assert result["method"] == "api.some_method"
            # Single value becomes array element
            assert result["params"] == [12345]

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_api_call_with_multiple_args(self):
        """Test that multiple args become array."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate("""
                (function() {
                    let capturedParams = null;

                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {
                        capturedParams = { method, params };
                        return originalCall.call(this, method, params);
                    };

                    // Call with multiple args
                    window.auroraview.api.multi_arg("arg1", "arg2", 123);

                    return capturedParams;
                })()
            """)

            assert result["method"] == "api.multi_arg"
            assert result["params"] == ["arg1", "arg2", 123]


class TestGalleryAPIContract:
    """Test the Gallery API contract between frontend and backend."""

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_kill_process_param_format(self):
        """Test kill_process uses correct param format {pid: number}."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        # Simulate the Gallery's useAuroraView hook behavior
        gallery_hook_code = """
            // This simulates the fixed useAuroraView.ts killProcess function
            async function killProcess(pid) {
                if (!window.auroraview) {
                    throw new Error('AuroraView not ready');
                }
                // CORRECT: Pass object with pid property
                return window.auroraview.api.kill_process({ pid });
            }
        """

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            # Inject the hook code and capture the call
            result = page.evaluate(f"""
                (function() {{
                    let capturedCall = null;

                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {{
                        capturedCall = {{ method, params }};
                        return Promise.resolve();
                    }};

                    {gallery_hook_code}

                    // Call the function
                    killProcess(66964);

                    return capturedCall;
                }})()
            """)

            assert result["method"] == "api.kill_process"
            # Should be [{pid: 66964}] not [66964]
            assert result["params"] == [{"pid": 66964}]

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_open_url_param_format(self):
        """Test open_url uses correct param format {url: string}."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        gallery_hook_code = """
            async function openUrl(url) {
                if (!window.auroraview) {
                    throw new Error('AuroraView not ready');
                }
                // CORRECT: Pass object with url property
                return window.auroraview.api.open_url({ url });
            }
        """

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate(f"""
                (function() {{
                    let capturedCall = null;

                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {{
                        capturedCall = {{ method, params }};
                        return Promise.resolve();
                    }};

                    {gallery_hook_code}

                    openUrl("https://example.com");

                    return capturedCall;
                }})()
            """)

            assert result["method"] == "api.open_url"
            assert result["params"] == [{"url": "https://example.com"}]

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_get_source_param_format(self):
        """Test get_source uses correct param format {sample_id: string}."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        gallery_hook_code = """
            async function getSource(sampleId) {
                if (!window.auroraview) {
                    throw new Error('AuroraView not ready');
                }
                // CORRECT: Pass object with sample_id property
                return window.auroraview.api.get_source({ sample_id: sampleId });
            }
        """

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate(f"""
                (function() {{
                    let capturedCall = null;

                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {{
                        capturedCall = {{ method, params }};
                        return Promise.resolve();
                    }};

                    {gallery_hook_code}

                    getSource("simple_decorator");

                    return capturedCall;
                }})()
            """)

            assert result["method"] == "api.get_source"
            assert result["params"] == [{"sample_id": "simple_decorator"}]


class TestBridgeEventSystem:
    """Test the AuroraView bridge event system."""

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_event_subscription_and_trigger(self):
        """Test event subscription and triggering."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate("""
                (function() {
                    const events = [];

                    // Subscribe to events
                    window.auroraview.on('process:stdout', (data) => {
                        events.push({ type: 'stdout', data });
                    });

                    window.auroraview.on('process:exit', (data) => {
                        events.push({ type: 'exit', data });
                    });

                    // Trigger events (simulating backend)
                    window.auroraview.trigger('process:stdout', { pid: 123, data: 'hello' });
                    window.auroraview.trigger('process:exit', { pid: 123, code: 0 });

                    return events;
                })()
            """)

            assert len(result) == 2
            assert result[0] == {"type": "stdout", "data": {"pid": 123, "data": "hello"}}
            assert result[1] == {"type": "exit", "data": {"pid": 123, "code": 0}}

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_call_result_event(self):
        """Test __auroraview_call_result event handling."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            # Test that call results are properly handled
            result = page.evaluate("""
                (function() {
                    let callResolved = false;
                    let resolvedValue = null;

                    // Make a call that will be resolved by trigger
                    const callPromise = new Promise((resolve) => {
                        // Simulate the pending call
                        window._pendingResolve = resolve;
                    });

                    // Subscribe to call result
                    window.auroraview.on('__auroraview_call_result', (data) => {
                        if (window._pendingResolve && data.id === 'test_call_1') {
                            callResolved = true;
                            resolvedValue = data;
                            window._pendingResolve(data.result);
                        }
                    });

                    // Trigger call result (simulating backend response)
                    window.auroraview.trigger('__auroraview_call_result', {
                        id: 'test_call_1',
                        ok: true,
                        result: { success: true, pid: 12345 }
                    });

                    return { callResolved, resolvedValue };
                })()
            """)

            assert result["callResolved"] is True
            assert result["resolvedValue"]["ok"] is True
            assert result["resolvedValue"]["result"]["pid"] == 12345


class TestAPIProxyBehavior:
    """Test the window.auroraview.api Proxy behavior."""

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_api_proxy_method_call(self):
        """Test that api proxy correctly wraps method calls."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate("""
                (function() {
                    // Check api proxy exists
                    const hasApi = typeof window.auroraview.api === 'object';

                    // Check that accessing any property returns a function
                    const methodType = typeof window.auroraview.api.any_method;

                    // Check that calling returns a promise
                    const callResult = window.auroraview.api.test_method();
                    const isPromise = callResult instanceof Promise;

                    return { hasApi, methodType, isPromise };
                })()
            """)

            assert result["hasApi"] is True
            assert result["methodType"] == "function"
            assert result["isPromise"] is True

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_api_proxy_preserves_method_name(self):
        """Test that api proxy preserves the method name."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate("""
                (function() {
                    const calls = [];

                    const originalCall = window.auroraview.call;
                    window.auroraview.call = function(method, params) {
                        calls.push(method);
                        return originalCall.call(this, method, params);
                    };

                    // Call various methods
                    window.auroraview.api.get_samples();
                    window.auroraview.api.run_sample({ sample_id: 'test' });
                    window.auroraview.api.kill_process({ pid: 123 });

                    return calls;
                })()
            """)

            assert result == ["api.get_samples", "api.run_sample", "api.kill_process"]
