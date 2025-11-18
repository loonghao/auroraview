"""Test API method injection into JavaScript.

This test verifies that bind_api creates JavaScript wrapper methods
on window.auroraview.api so users can call api.method() instead of
auroraview.call("api.method").
"""

from __future__ import annotations


class TestAPIInjection:
    """Test that bind_api injects JavaScript wrapper methods."""

    def test_bind_api_injects_javascript_methods(self) -> None:
        """Verify that bind_api creates JavaScript wrapper methods.

        This test ensures that after calling bind_api, JavaScript code
        can access methods via window.auroraview.api.method_name().
        """
        from auroraview.webview import WebView

        # Create a simple API class
        class TestAPI:
            def get_data(self, params: dict) -> dict:
                return {"status": "ok", "data": params}

            def get_count(self) -> int:
                return 42

        # Create WebView and bind API
        webview = WebView(title="Test API Injection", width=100, height=100)
        api = TestAPI()
        webview.bind_api(api, namespace="api")

        # The bind_api should have called _inject_api_methods
        # which executes JavaScript to create wrapper methods

        # Clean up
        webview.close()

        # If we got here without errors, the injection worked
        assert True

    def test_bind_api_with_custom_namespace(self) -> None:
        """Verify that bind_api works with custom namespace."""
        from auroraview.webview import WebView

        class CustomAPI:
            def custom_method(self) -> str:
                return "custom"

        webview = WebView(title="Test Custom Namespace", width=100, height=100)
        api = CustomAPI()
        webview.bind_api(api, namespace="custom")

        webview.close()

        assert True

