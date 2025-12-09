"""Test API injection timing.

This script verifies that API methods are properly injected
even when bind_api is called before load_url.
"""

from __future__ import annotations

import os

import pytest

# Skip UI tests in CI - these require WebView runtime and display
pytestmark = pytest.mark.skipif(
    os.environ.get("CI") == "true",
    reason="WebView creation requires display environment, skipped in CI",
)


@pytest.mark.qt
def test_api_injection_timing():
    """Test that API methods are injected after page load."""
    from auroraview import AuroraView, QtWebView

    # Create a simple API
    class TestAPI:
        def get_data(self) -> dict:
            return {"status": "ok", "message": "API is working!"}

        def echo(self, params: dict) -> dict:
            return params

    # Create WebView
    webview = QtWebView(title="API Injection Test", width=800, height=600, dev_tools=True)

    # Create API and bind it BEFORE loading URL
    api = TestAPI()
    AuroraView(parent=webview, api=api, _view=webview, _keep_alive_root=webview)

    print("[Test] API bound before loading URL")

    # Load HTML with test script
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>API Injection Test</title>
        <style>
            body {
                font-family: Arial, sans-serif;
                padding: 20px;
                background: #1e1e1e;
                color: #fff;
            }
            .status {
                padding: 10px;
                margin: 10px 0;
                border-radius: 4px;
            }
            .success { background: #2d5016; }
            .error { background: #5a1d1d; }
            button {
                padding: 10px 20px;
                margin: 5px;
                cursor: pointer;
            }
        </style>
    </head>
    <body>
        <h1>API Injection Timing Test</h1>
        <div id="status"></div>
        <button onclick="testAPI()">Test API</button>

        <script>
            console.log('[Test] Page loaded');

            // Wait for page to fully load
            window.addEventListener('load', function() {
                console.log('[Test] Window load event fired');

                // Check if API is available
                setTimeout(function() {
                    const status = document.getElementById('status');

                    console.log('[Test] Checking API availability...');
                    console.log('[Test] window.auroraview:', window.auroraview);
                    console.log('[Test] window.auroraview.api:', window.auroraview.api);

                    if (window.auroraview && window.auroraview.api) {
                        console.log('[Test] API object found');
                        console.log('[Test] API methods:', Object.keys(window.auroraview.api));

                        if (typeof window.auroraview.api.get_data === 'function') {
                            status.innerHTML = '<div class="status success">✓ API methods are available!</div>';
                            console.log('[Test] ✓ get_data is a function');
                        } else {
                            status.innerHTML = '<div class="status error">✗ get_data is not a function</div>';
                            console.error('[Test] ✗ get_data is not a function');
                        }
                    } else {
                        status.innerHTML = '<div class="status error">✗ API not found</div>';
                        console.error('[Test] ✗ API not found');
                    }
                }, 1000);
            });

            async function testAPI() {
                const status = document.getElementById('status');
                try {
                    console.log('[Test] Calling API...');
                    const result = await window.auroraview.api.get_data();
                    console.log('[Test] API result:', result);
                    status.innerHTML = '<div class="status success">✓ API call successful: ' +
                                      JSON.stringify(result) + '</div>';
                } catch (error) {
                    console.error('[Test] API call failed:', error);
                    status.innerHTML = '<div class="status error">✗ API call failed: ' +
                                      error.message + '</div>';
                }
            }
        </script>
    </body>
    </html>
    """

    webview.load_html(html)
    print("[Test] HTML loaded")
    print("[Test] Opening WebView - check console for API availability")

    webview.show()


if __name__ == "__main__":
    test_api_injection_timing()
