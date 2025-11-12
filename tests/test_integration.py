"""Integration tests for auroraview package."""

import pytest


@pytest.mark.integration
class TestWebViewIntegration:
    """Integration tests for WebView."""

    def test_webview_creation_and_properties(self):
        """Test WebView creation and property access."""
        try:
            from auroraview import WebView

            webview = WebView(
                title="Integration Test", width=1024, height=768, debug=True, resizable=True
            )

            assert webview.title == "Integration Test"
            assert webview._width == 1024
            assert webview._height == 768
        except ImportError:
            pytest.skip("Package not built yet")

    def test_webview_event_system(self):
        """Test WebView event system integration."""
        try:
            from auroraview import WebView

            webview = WebView()
            events_received = []

            @webview.on("test_event")
            def handler(data):
                events_received.append(data)

            # Emit event
            webview.emit("test_event", {"message": "test"})

            # Handler should be registered
            assert "test_event" in webview._event_handlers
        except ImportError:
            pytest.skip("Package not built yet")

    def test_webview_multiple_events(self):
        """Test WebView with multiple events."""
        try:
            from auroraview import WebView

            webview = WebView()

            @webview.on("event1")
            def handler1(data):
                pass

            @webview.on("event2")
            def handler2(data):
                pass

            @webview.on("event3")
            def handler3(data):
                pass

            assert len(webview._event_handlers) == 3
            assert "event1" in webview._event_handlers
            assert "event2" in webview._event_handlers
            assert "event3" in webview._event_handlers
        except ImportError:
            pytest.skip("Package not built yet")

    def test_webview_context_manager_integration(self):
        """Test WebView context manager integration."""
        try:
            from auroraview import WebView

            with WebView(title="Context Test") as webview:
                assert webview is not None
                assert webview.title == "Context Test"

                @webview.on("test")
                def handler(data):
                    pass

                assert "test" in webview._event_handlers
        except ImportError:
            pytest.skip("Package not built yet")


@pytest.mark.integration
class TestDecoratorIntegration:
    """Integration tests for decorators."""

    def test_on_event_with_webview_integration(self):
        """Test @webview.on() decorator with WebView integration."""
        try:
            from auroraview import WebView

            webview = WebView()

            @webview.on("data_update")
            def handle_update(data):
                return data

            assert "data_update" in webview._event_handlers
        except ImportError:
            pytest.skip("Package not built yet")


@pytest.mark.integration
class TestPackageIntegration:
    """Integration tests for the entire package."""

    def test_full_workflow(self):
        """Test a full workflow with WebView."""
        try:
            from auroraview import WebView

            # Create WebView
            webview = WebView(
                title="Full Workflow Test", width=800, height=600, url="https://example.com"
            )

            # Register event handlers
            @webview.on("scene_update")
            def handle_scene_update(data):
                return {"status": "updated"}

            @webview.on("export_complete")
            def handle_export(data):
                return {"status": "exported"}

            # Emit events
            webview.emit("scene_update", {"objects": 5})
            webview.emit("export_complete", {"path": "/tmp/export"})

            # Verify handlers are registered
            assert "scene_update" in webview._event_handlers
            assert "export_complete" in webview._event_handlers
            assert len(webview._event_handlers) == 2
        except ImportError:
            pytest.skip("Package not built yet")

    def test_webview_repr_and_properties(self):
        """Test WebView repr and properties."""
        try:
            from auroraview import WebView

            webview = WebView(title="Test App", width=1024, height=768)

            repr_str = repr(webview)
            assert "Test App" in repr_str
            assert "1024" in repr_str
            assert "768" in repr_str

            # Test property access
            assert webview.title == "Test App"
        except ImportError:
            pytest.skip("Package not built yet")
