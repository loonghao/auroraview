"""Test automatic event processing in WebView.

This module tests the new _auto_process_events mechanism that replaces
the old _post_eval_js_hook pattern.
"""

import os

import pytest

# Mark all tests as Qt tests and skip in CI (requires display)
pytestmark = [
    pytest.mark.qt,
    pytest.mark.unit,
    pytest.mark.skipif(
        os.environ.get("CI") == "true",
        reason="WebView tests require display environment, skipped in CI",
    ),
]


class TestWebViewAutoProcessEvents:
    """Test that WebView.emit() and eval_js() automatically process events."""

    def test_emit_calls_auto_process_events(self):
        """Test that WebView.emit() calls _auto_process_events by default."""
        try:
            from auroraview import WebView

            webview = WebView()

            process_called = []

            def mock_process():
                process_called.append(True)

            webview._auto_process_events = mock_process

            # Emit an event
            webview.emit("test_event", {"data": "test"})

            # Verify auto process was called
            assert len(process_called) == 1, "emit() should call _auto_process_events"

            webview.close()
        except ImportError:
            pytest.skip("Package not built yet")

    def test_emit_with_auto_process_false(self):
        """Test that WebView.emit() skips auto processing when auto_process=False."""
        try:
            from auroraview import WebView

            webview = WebView()

            process_called = []

            def mock_process():
                process_called.append(True)

            webview._auto_process_events = mock_process

            # Emit with auto_process=False
            webview.emit("test_event", {"data": "test"}, auto_process=False)

            # Verify auto process was NOT called
            assert len(process_called) == 0, (
                "emit(auto_process=False) should not call _auto_process_events"
            )

            webview.close()
        except ImportError:
            pytest.skip("Package not built yet")

    def test_eval_js_calls_auto_process_events(self):
        """Test that WebView.eval_js() calls _auto_process_events by default."""
        try:
            from auroraview import WebView

            webview = WebView()

            process_called = []

            def mock_process():
                process_called.append(True)

            webview._auto_process_events = mock_process

            # Execute JavaScript
            webview.eval_js("console.log('test')")

            # Verify auto process was called
            assert len(process_called) == 1, "eval_js() should call _auto_process_events"

            webview.close()
        except ImportError:
            pytest.skip("Package not built yet")

    def test_eval_js_with_auto_process_false(self):
        """Test that WebView.eval_js() skips auto processing when auto_process=False."""
        try:
            from auroraview import WebView

            webview = WebView()

            process_called = []

            def mock_process():
                process_called.append(True)

            webview._auto_process_events = mock_process

            # Execute with auto_process=False
            webview.eval_js("console.log('test')", auto_process=False)

            # Verify auto process was NOT called
            assert len(process_called) == 0, (
                "eval_js(auto_process=False) should not call _auto_process_events"
            )

            webview.close()
        except ImportError:
            pytest.skip("Package not built yet")

    def test_batch_operations(self):
        """Test batching multiple operations without auto processing."""
        try:
            from auroraview import WebView

            webview = WebView()

            process_called = []

            def mock_process():
                process_called.append(True)

            webview._auto_process_events = mock_process

            # Batch multiple operations
            webview.emit("event1", {"data": 1}, auto_process=False)
            webview.emit("event2", {"data": 2}, auto_process=False)
            webview.eval_js("console.log('test')", auto_process=False)

            # Verify auto process was NOT called
            assert len(process_called) == 0, "Batched operations should not auto process"

            # Manually call the auto process method
            webview._auto_process_events()

            webview.close()
        except ImportError:
            pytest.skip("Package not built yet")
