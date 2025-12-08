"""
AuroraView API Binding Tests using AuroraTest

This module tests Python API binding to JavaScript including:
- Simple method binding
- Method with arguments
- Return values
- Async methods
- Error handling
- Namespace management

NOTE: These tests use the original Browser class which requires WebView2.
Due to Python GIL limitations, WebView2 event loop blocks other threads.
For UI automation testing, use PlaywrightBrowser instead.

See test_playwright_browser.py for working Playwright-based tests.
"""

import asyncio
import logging
import sys
import time
from typing import Any, Dict, List

import pytest

from auroraview import WebView
from auroraview.testing.auroratest import Browser

logger = logging.getLogger(__name__)

pytestmark = [
    pytest.mark.skip(
        reason="WebView2 Browser class blocks due to GIL. Use PlaywrightBrowser instead."
    ),
    pytest.mark.skipif(sys.platform != "win32", reason="WebView2 tests only run on Windows"),
]


# ============================================================
# Test API Classes
# ============================================================


class MathAPI:
    """Math operations API for testing."""

    def add(self, a: int, b: int) -> int:
        """Add two numbers."""
        return a + b

    def subtract(self, a: int, b: int) -> int:
        """Subtract two numbers."""
        return a - b

    def multiply(self, a: int, b: int) -> int:
        """Multiply two numbers."""
        return a * b

    def divide(self, a: float, b: float) -> float:
        """Divide two numbers."""
        if b == 0:
            raise ValueError("Division by zero")
        return a / b

    def power(self, base: float, exp: float) -> float:
        """Raise base to power."""
        return base**exp


class StringAPI:
    """String operations API for testing."""

    def upper(self, s: str) -> str:
        """Convert to uppercase."""
        return s.upper()

    def lower(self, s: str) -> str:
        """Convert to lowercase."""
        return s.lower()

    def reverse(self, s: str) -> str:
        """Reverse string."""
        return s[::-1]

    def concat(self, *args: str) -> str:
        """Concatenate strings."""
        return "".join(args)

    def split(self, s: str, delimiter: str = " ") -> List[str]:
        """Split string."""
        return s.split(delimiter)


class DataAPI:
    """Data operations API for testing."""

    def __init__(self):
        self._storage: Dict[str, Any] = {}

    def set(self, key: str, value: Any) -> bool:
        """Set a value."""
        self._storage[key] = value
        return True

    def get(self, key: str, default: Any = None) -> Any:
        """Get a value."""
        return self._storage.get(key, default)

    def delete(self, key: str) -> bool:
        """Delete a value."""
        if key in self._storage:
            del self._storage[key]
            return True
        return False

    def keys(self) -> List[str]:
        """Get all keys."""
        return list(self._storage.keys())

    def clear(self) -> int:
        """Clear all data, return count."""
        count = len(self._storage)
        self._storage.clear()
        return count

    def get_all(self) -> Dict[str, Any]:
        """Get all data."""
        return self._storage.copy()


class ConfigAPI:
    """Configuration API for testing."""

    def __init__(self):
        self._config = {
            "version": "1.0.0",
            "debug": True,
            "theme": "dark",
            "language": "en",
            "features": {"api_binding": True, "dom_batch": True, "signals": True},
        }

    def get_config(self) -> Dict[str, Any]:
        """Get full configuration."""
        return self._config.copy()

    def get_value(self, key: str) -> Any:
        """Get config value by key."""
        return self._config.get(key)

    def set_value(self, key: str, value: Any) -> bool:
        """Set config value."""
        self._config[key] = value
        return True

    def get_version(self) -> str:
        """Get version string."""
        return self._config["version"]

    def is_debug(self) -> bool:
        """Check if debug mode."""
        return self._config.get("debug", False)


class AsyncAPI:
    """Async operations API for testing."""

    async def async_add(self, a: int, b: int) -> int:
        """Async add operation."""
        await asyncio.sleep(0.1)
        return a + b

    async def async_fetch(self, url: str) -> Dict[str, Any]:
        """Simulate async fetch."""
        await asyncio.sleep(0.2)
        return {"url": url, "status": 200, "data": "mock data"}

    def sync_method(self) -> str:
        """Regular sync method."""
        return "sync result"


# ============================================================
# Test HTML
# ============================================================

API_TEST_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>API Binding Test</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; }
        .result { padding: 10px; margin: 5px 0; background: #f5f5f5; font-family: monospace; }
        .success { background: #e8f5e9; }
        .error { background: #ffebee; }
        button { padding: 8px 16px; margin: 5px; cursor: pointer; }
        input { padding: 8px; margin: 5px; }
    </style>
</head>
<body>
    <h1>API Binding Test</h1>

    <div class="section">
        <h2>Math API</h2>
        <input type="number" id="num-a" value="10" />
        <input type="number" id="num-b" value="5" />
        <button onclick="testMath('add')">Add</button>
        <button onclick="testMath('subtract')">Subtract</button>
        <button onclick="testMath('multiply')">Multiply</button>
        <button onclick="testMath('divide')">Divide</button>
        <div id="math-result" class="result">Result will appear here</div>
    </div>

    <div class="section">
        <h2>String API</h2>
        <input type="text" id="str-input" value="Hello World" />
        <button onclick="testString('upper')">Upper</button>
        <button onclick="testString('lower')">Lower</button>
        <button onclick="testString('reverse')">Reverse</button>
        <div id="string-result" class="result">Result will appear here</div>
    </div>

    <div class="section">
        <h2>Data API</h2>
        <input type="text" id="data-key" placeholder="Key" />
        <input type="text" id="data-value" placeholder="Value" />
        <button onclick="testData('set')">Set</button>
        <button onclick="testData('get')">Get</button>
        <button onclick="testData('delete')">Delete</button>
        <button onclick="testData('keys')">Keys</button>
        <button onclick="testData('get_all')">Get All</button>
        <div id="data-result" class="result">Result will appear here</div>
    </div>

    <div class="section">
        <h2>Config API</h2>
        <button onclick="testConfig('get_config')">Get Config</button>
        <button onclick="testConfig('get_version')">Get Version</button>
        <button onclick="testConfig('is_debug')">Is Debug</button>
        <div id="config-result" class="result">Result will appear here</div>
    </div>

    <div class="section">
        <h2>API Status</h2>
        <div id="api-status" class="result">Checking API availability...</div>
    </div>

    <script>
        // Check API availability
        function checkAPIs() {
            const status = document.getElementById('api-status');
            const apis = ['math', 'string', 'data', 'config'];
            const available = [];
            const missing = [];

            apis.forEach(api => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api[api]) {
                    available.push(api);
                } else {
                    missing.push(api);
                }
            });

            if (missing.length === 0) {
                status.textContent = 'All APIs available: ' + available.join(', ');
                status.className = 'result success';
            } else {
                status.textContent = 'Available: ' + available.join(', ') +
                    ' | Missing: ' + missing.join(', ');
                status.className = 'result error';
            }
        }

        // Test Math API
        async function testMath(operation) {
            const a = parseFloat(document.getElementById('num-a').value);
            const b = parseFloat(document.getElementById('num-b').value);
            const result = document.getElementById('math-result');

            try {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.math) {
                    const r = await window.auroraview.api.math[operation](a, b);
                    result.textContent = `${operation}(${a}, ${b}) = ${r}`;
                    result.className = 'result success';
                } else {
                    result.textContent = 'Math API not available';
                    result.className = 'result error';
                }
            } catch (e) {
                result.textContent = 'Error: ' + e.message;
                result.className = 'result error';
            }
        }

        // Test String API
        async function testString(operation) {
            const s = document.getElementById('str-input').value;
            const result = document.getElementById('string-result');

            try {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.string) {
                    const r = await window.auroraview.api.string[operation](s);
                    result.textContent = `${operation}("${s}") = "${r}"`;
                    result.className = 'result success';
                } else {
                    result.textContent = 'String API not available';
                    result.className = 'result error';
                }
            } catch (e) {
                result.textContent = 'Error: ' + e.message;
                result.className = 'result error';
            }
        }

        // Test Data API
        async function testData(operation) {
            const key = document.getElementById('data-key').value;
            const value = document.getElementById('data-value').value;
            const result = document.getElementById('data-result');

            try {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.data) {
                    let r;
                    switch (operation) {
                        case 'set':
                            r = await window.auroraview.api.data.set(key, value);
                            break;
                        case 'get':
                            r = await window.auroraview.api.data.get(key);
                            break;
                        case 'delete':
                            r = await window.auroraview.api.data.delete(key);
                            break;
                        case 'keys':
                            r = await window.auroraview.api.data.keys();
                            break;
                        case 'get_all':
                            r = await window.auroraview.api.data.get_all();
                            break;
                    }
                    result.textContent = `${operation}() = ${JSON.stringify(r)}`;
                    result.className = 'result success';
                } else {
                    result.textContent = 'Data API not available';
                    result.className = 'result error';
                }
            } catch (e) {
                result.textContent = 'Error: ' + e.message;
                result.className = 'result error';
            }
        }

        // Test Config API
        async function testConfig(operation) {
            const result = document.getElementById('config-result');

            try {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.config) {
                    const r = await window.auroraview.api.config[operation]();
                    result.textContent = `${operation}() = ${JSON.stringify(r)}`;
                    result.className = 'result success';
                } else {
                    result.textContent = 'Config API not available';
                    result.className = 'result error';
                }
            } catch (e) {
                result.textContent = 'Error: ' + e.message;
                result.className = 'result error';
            }
        }

        // Check APIs on load
        setTimeout(checkAPIs, 500);
    </script>
</body>
</html>
"""


# ============================================================
# Fixtures
# ============================================================


@pytest.fixture
def webview_with_apis():
    """Create WebView with multiple APIs bound."""
    webview = WebView(
        title="API Test",
        width=1024,
        height=768,
        debug=True,
    )

    # Bind APIs with different namespaces
    webview.bind_api(MathAPI(), namespace="math")
    webview.bind_api(StringAPI(), namespace="string")
    webview.bind_api(DataAPI(), namespace="data")
    webview.bind_api(ConfigAPI(), namespace="config")

    webview.load_html(API_TEST_HTML)
    webview.show(wait=False)
    time.sleep(0.5)

    # Re-register APIs after page load
    webview.bind_api(MathAPI(), namespace="math", allow_rebind=True)
    webview.bind_api(StringAPI(), namespace="string", allow_rebind=True)
    webview.bind_api(DataAPI(), namespace="data", allow_rebind=True)
    webview.bind_api(ConfigAPI(), namespace="config", allow_rebind=True)

    time.sleep(0.3)

    yield webview
    try:
        webview.close()
    except Exception:
        pass


# ============================================================
# API Binding Tests
# ============================================================


class TestAPIBinding:
    """Test basic API binding functionality."""

    def test_bind_single_api(self):
        """Test binding a single API."""
        webview = WebView(title="Single API Test")
        api = MathAPI()

        webview.bind_api(api, namespace="math")

        bound = webview.get_bound_methods()
        assert bound is not None

        webview.close()

    def test_bind_multiple_apis(self):
        """Test binding multiple APIs."""
        webview = WebView(title="Multiple API Test")

        webview.bind_api(MathAPI(), namespace="math")
        webview.bind_api(StringAPI(), namespace="string")
        webview.bind_api(DataAPI(), namespace="data")

        bound = webview.get_bound_methods()
        assert bound is not None

        webview.close()

    def test_bind_default_namespace(self):
        """Test binding with default namespace."""
        webview = WebView(title="Default Namespace Test")

        class SimpleAPI:
            def hello(self) -> str:
                return "Hello!"

        webview.bind_api(SimpleAPI())  # Default namespace is "api"

        bound = webview.get_bound_methods()
        assert bound is not None

        webview.close()

    def test_rebind_api(self):
        """Test rebinding API with allow_rebind."""
        webview = WebView(title="Rebind Test")
        api = MathAPI()

        webview.bind_api(api, namespace="math")

        # Should not raise with allow_rebind=True
        webview.bind_api(api, namespace="math", allow_rebind=True)

        webview.close()


# ============================================================
# API Method Tests
# ============================================================


class TestAPIMethods:
    """Test API method invocation."""

    def test_method_with_args(self, webview_with_apis):
        """Test calling method with arguments."""
        webview_with_apis.eval_js("""
            (async () => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.math) {
                    const result = await window.auroraview.api.math.add(5, 3);
                    document.getElementById('math-result').textContent = 'Result: ' + result;
                }
            })();
        """)
        time.sleep(0.3)

    def test_method_return_value(self, webview_with_apis):
        """Test method return value."""
        webview_with_apis.eval_js("""
            (async () => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.string) {
                    const result = await window.auroraview.api.string.upper('hello');
                    document.getElementById('string-result').textContent = 'Result: ' + result;
                }
            })();
        """)
        time.sleep(0.3)

    def test_method_with_complex_return(self, webview_with_apis):
        """Test method with complex return value."""
        webview_with_apis.eval_js("""
            (async () => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.config) {
                    const config = await window.auroraview.api.config.get_config();
                    document.getElementById('config-result').textContent =
                        'Config: ' + JSON.stringify(config);
                }
            })();
        """)
        time.sleep(0.3)

    def test_method_with_state(self, webview_with_apis):
        """Test method that modifies state."""
        webview_with_apis.eval_js("""
            (async () => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.data) {
                    await window.auroraview.api.data.set('key1', 'value1');
                    await window.auroraview.api.data.set('key2', 'value2');
                    const keys = await window.auroraview.api.data.keys();
                    document.getElementById('data-result').textContent =
                        'Keys: ' + JSON.stringify(keys);
                }
            })();
        """)
        time.sleep(0.3)


# ============================================================
# Error Handling Tests
# ============================================================


class TestAPIErrorHandling:
    """Test API error handling."""

    def test_division_by_zero(self, webview_with_apis):
        """Test handling of Python exception."""
        webview_with_apis.eval_js("""
            (async () => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.math) {
                    try {
                        const result = await window.auroraview.api.math.divide(10, 0);
                        document.getElementById('math-result').textContent = 'Result: ' + result;
                    } catch (e) {
                        document.getElementById('math-result').textContent = 'Error: ' + e.message;
                    }
                }
            })();
        """)
        time.sleep(0.3)

    def test_invalid_method(self, webview_with_apis):
        """Test calling non-existent method."""
        webview_with_apis.eval_js("""
            (async () => {
                if (window.auroraview && window.auroraview.api && window.auroraview.api.math) {
                    try {
                        const result = await window.auroraview.api.math.nonexistent();
                        document.getElementById('math-result').textContent = 'Result: ' + result;
                    } catch (e) {
                        document.getElementById('math-result').textContent = 'Error: ' + e.message;
                    }
                }
            })();
        """)
        time.sleep(0.3)


# ============================================================
# AuroraTest Integration
# ============================================================


class TestAuroraTestAPIIntegration:
    """Test API binding using AuroraTest framework."""

    @pytest.mark.asyncio
    async def test_api_availability_check(self):
        """Test checking API availability via AuroraTest."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(API_TEST_HTML)
        await page.wait_for_timeout(500)

        # Check API status element
        # Note: APIs won't be bound in this test since we're using Browser directly

        browser.close()


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
