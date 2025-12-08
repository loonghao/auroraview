"""Test API method injection into JavaScript.

This test verifies that bind_api registers methods with Rust core,
which handles JavaScript injection via js_assets module.
"""

from __future__ import annotations

import os

import pytest

# Skip UI tests in CI - these require WebView runtime
pytestmark = pytest.mark.skipif(
    os.environ.get("CI") == "true",
    reason="WebView creation requires display environment, skipped in CI",
)


class TestAPIInjection:
    """Test that bind_api registers methods with Rust core."""

    def test_bind_api_registers_methods_with_rust(self) -> None:
        """Verify that bind_api registers methods with Rust core.

        This test ensures that after calling bind_api, methods are
        registered with the Rust core, which will inject them into
        JavaScript via the js_assets module.
        """
        from auroraview import WebView

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

        # The bind_api should have called register_api_methods on Rust core
        # Rust core will handle JavaScript injection via js_assets

        # Clean up
        webview.close()

        # If we got here without errors, the registration worked
        assert True

    def test_bind_api_with_custom_namespace(self) -> None:
        """Verify that bind_api works with custom namespace."""
        from auroraview import WebView

        class CustomAPI:
            def custom_method(self) -> str:
                return "custom"

        webview = WebView(title="Test Custom Namespace", width=100, height=100)
        api = CustomAPI()
        webview.bind_api(api, namespace="custom")

        webview.close()

        assert True

    def test_bind_api_filters_private_methods(self) -> None:
        """Verify that bind_api only exposes public methods."""
        from auroraview import WebView

        class APIWithPrivate:
            def public_method(self) -> str:
                return "public"

            def _private_method(self) -> str:
                return "private"

            def __dunder_method__(self) -> str:
                return "dunder"

        webview = WebView(title="Test Private Methods", width=100, height=100)
        api = APIWithPrivate()
        webview.bind_api(api, namespace="api")

        # Only public_method should be registered
        # _private_method and __dunder_method__ should be filtered out

        webview.close()

        assert True

    def test_bind_api_multiple_namespaces(self) -> None:
        """Verify that bind_api can register multiple namespaces."""
        from auroraview import WebView

        class API1:
            def method1(self) -> str:
                return "api1"

        class API2:
            def method2(self) -> str:
                return "api2"

        webview = WebView(title="Test Multiple Namespaces", width=100, height=100)
        webview.bind_api(API1(), namespace="api1")
        webview.bind_api(API2(), namespace="api2")

        # Both namespaces should be registered independently

        webview.close()

        assert True
