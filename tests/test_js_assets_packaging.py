"""Test JavaScript assets packaging in .pyd files.

This test ensures that JavaScript resources embedded via include_str! macro
are correctly packaged into the .pyd file and accessible at runtime.

Critical for verifying that:
1. event_bridge.js is embedded and functional
2. context_menu.js is embedded and functional
3. IPC communication works after packaging
"""

from __future__ import annotations

import pytest


class TestJavaScriptAssetsPackaging:
    """Test that JavaScript assets are properly embedded in the .pyd file."""

    def test_js_assets_embedded_in_pyd(self) -> None:
        """Verify that JavaScript assets are embedded in the .pyd file.

        This test ensures that the js_assets module and its constants
        are properly embedded in the compiled .pyd file by creating a WebView.
        If js_assets weren't embedded, WebView creation would fail.
        """
        from auroraview.webview import WebView

        # Create a WebView - this internally uses js_assets.build_init_script()
        # which loads EVENT_BRIDGE and other JavaScript constants via include_str!
        # If these weren't embedded in the .pyd, this would fail
        webview = WebView(title="Test Packaging", width=100, height=100, headless=True)
        assert webview is not None

        # Clean up
        webview.close()

    @pytest.mark.ui
    def test_event_bridge_script_is_functional(self, webview, webview_bot, test_html) -> None:
        """Verify that event_bridge.js is embedded and window.auroraview API exists.

        This test ensures that the core JavaScript API is available after packaging.
        The event_bridge.js file is embedded at compile time via include_str! macro.
        """
        webview.load_html(test_html)
        webview_bot.inject_monitoring_script()
        webview_bot.wait_for_event("webview_ready", timeout=5)

        # Check that window.auroraview object exists
        result = webview.eval_js("typeof window.auroraview")
        assert result == "object", "window.auroraview should be an object"

        # Check that core API methods exist
        result = webview.eval_js("typeof window.auroraview.call")
        assert result == "function", "window.auroraview.call should be a function"

        result = webview.eval_js("typeof window.auroraview.send_event")
        assert result == "function", "window.auroraview.send_event should be a function"

        result = webview.eval_js("typeof window.auroraview.on")
        assert result == "function", "window.auroraview.on should be a function"

    @pytest.mark.ui
    def test_ipc_call_works_after_packaging(self, webview, webview_bot, test_html) -> None:
        """Verify that IPC communication works with embedded JavaScript.

        This test ensures that the event bridge script is functional and
        can communicate between JavaScript and Python after packaging.
        """
        webview.load_html(test_html)
        webview_bot.inject_monitoring_script()
        webview_bot.wait_for_event("webview_ready", timeout=5)

        # Bind a Python function
        call_count = []

        def test_handler(data: dict) -> dict:
            call_count.append(1)
            return {"status": "ok", "received": data}

        webview.bind_call("test.handler", test_handler)

        # Call from JavaScript using the embedded event bridge
        result = webview.eval_js(
            """
            (async () => {
                const result = await window.auroraview.call('test.handler', {
                    message: 'hello from js'
                });
                return result;
            })()
            """
        )

        assert len(call_count) == 1, "Handler should be called once"
        assert result["status"] == "ok"
        assert result["received"]["message"] == "hello from js"

    @pytest.mark.ui
    def test_context_menu_disable_script_is_embedded(self, webview, webview_bot, test_html) -> None:
        """Verify that context_menu.js is embedded when enabled in config.

        This test ensures that optional JavaScript features are correctly
        embedded based on WebViewConfig settings.
        """
        # Create a new WebView with context menu disabled
        from auroraview.webview import WebView

        webview_with_disabled_menu = WebView(
            title="Test Context Menu",
            width=800,
            height=600,
            disable_context_menu=True,
        )

        webview_with_disabled_menu.load_html(test_html)

        # Check that context menu is disabled
        # The script should prevent default on contextmenu event
        result = webview_with_disabled_menu.eval_js(
            """
            (() => {
                const event = new MouseEvent('contextmenu', {
                    bubbles: true,
                    cancelable: true
                });
                document.dispatchEvent(event);
                return event.defaultPrevented;
            })()
            """
        )

        webview_with_disabled_menu.close()

        assert result is True, "Context menu should be disabled"

    @pytest.mark.ui
    def test_event_listeners_work_after_packaging(self, webview, webview_bot, test_html) -> None:
        """Verify that event listeners work with embedded JavaScript.

        This test ensures that the event system in event_bridge.js
        is functional after packaging.
        """
        webview.load_html(test_html)
        webview_bot.inject_monitoring_script()
        webview_bot.wait_for_event("webview_ready", timeout=5)

        # Set up event listener in JavaScript
        webview.eval_js(
            """
            window.testEventReceived = false;
            window.auroraview.on('test_event', (detail) => {
                window.testEventReceived = true;
                window.testEventData = detail;
            });
            """
        )

        # Emit event from Python
        webview.emit("test_event", {"message": "test data"})

        # Give time for event to be processed
        webview_bot.wait(0.1)

        # Check that event was received
        result = webview.eval_js("window.testEventReceived")
        assert result is True, "Event should be received"

        result = webview.eval_js("window.testEventData.message")
        assert result == "test data", "Event data should match"

