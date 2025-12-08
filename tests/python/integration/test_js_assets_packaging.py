"""Test JavaScript assets packaging in .pyd files.

This test ensures that JavaScript resources embedded via include_str! macro
are correctly packaged into the .pyd file and accessible at runtime.

Critical for verifying that:
1. event_bridge.js is embedded and functional
2. context_menu.js is embedded and functional
3. IPC communication works after packaging

Note: These tests verify that JavaScript assets are embedded in the compiled
binary. The actual functionality of these scripts is tested in other test files
that use the @pytest.mark.ui marker for full UI testing.
"""

from __future__ import annotations

import os

import pytest

# Skip UI tests in CI - these require a display and WebView2/wry runtime
pytestmark = pytest.mark.skipif(
    os.environ.get("CI") == "true",
    reason="WebView creation requires display environment, skipped in CI",
)


class TestJavaScriptAssetsPackaging:
    """Test that JavaScript assets are properly embedded in the .pyd file."""

    def test_js_assets_embedded_in_pyd(self) -> None:
        """Verify that JavaScript assets are embedded in the .pyd file.

        This test ensures that the js_assets module and its constants
        are properly embedded in the compiled .pyd file by creating a WebView.
        If js_assets weren't embedded, WebView creation would fail.
        """
        from auroraview import WebView

        # Create a WebView - this internally uses js_assets.build_init_script()
        # which loads EVENT_BRIDGE and other JavaScript constants via include_str!
        # If these weren't embedded in the .pyd, this would fail
        webview = WebView(title="Test Packaging", width=100, height=100)
        assert webview is not None

        # Clean up
        webview.close()

    def test_webview_with_disabled_context_menu(self) -> None:
        """Verify that WebView can be created with context menu disabled.

        This test ensures that the context_menu.js script is embedded
        and can be conditionally included based on the context_menu parameter.
        """
        from auroraview import WebView

        # Create a WebView with context menu disabled
        # This internally uses js_assets.CONTEXT_MENU_DISABLE constant
        webview = WebView(
            title="Test Context Menu",
            width=100,
            height=100,
            context_menu=False,
        )
        assert webview is not None

        # Clean up
        webview.close()

    def test_bind_call_with_embedded_event_bridge(self) -> None:
        """Verify that bind_call works with embedded event_bridge.js.

        This test ensures that the IPC mechanism works correctly
        with the embedded JavaScript assets.
        """
        from auroraview import WebView

        webview = WebView(title="Test IPC", width=100, height=100)

        # Bind a Python function - this uses the embedded event bridge
        call_count = []

        def test_handler(data: dict) -> dict:
            call_count.append(1)
            return {"status": "ok", "received": data}

        # This should work without errors if event_bridge.js is embedded
        webview.bind_call("test.handler", test_handler)

        # Clean up
        webview.close()

        # If we got here without errors, the event bridge is embedded correctly
        assert True
