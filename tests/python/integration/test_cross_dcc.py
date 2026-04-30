"""Cross-DCC pipeline tests for AuroraView.

Tests that don't require actual DCC binaries but test
the cross-DCC functionality and pipeline integration.
"""

import pytest


def test_dcc_environment_detection():
    """Test DCC environment detection without actual DCC."""
    from auroraview.utils.thread_dispatcher import (
        BlenderDispatcherBackend,
        HoudiniDispatcherBackend,
        MayaDispatcherBackend,
        NukeDispatcherBackend,
    )

    # Test that backends can be instantiated
    backends = [
        MayaDispatcherBackend(),
        BlenderDispatcherBackend(),
        NukeDispatcherBackend(),
        HoudiniDispatcherBackend(),
    ]

    for backend in backends:
        assert backend is not None
        # Without DCC installed, should return False
        result = backend.is_available()
        assert isinstance(result, bool)


def test_cross_dcc_event_system():
    """Test event system works across DCC contexts."""
    from auroraview.core.event_emitter import EventEmitter

    emitter = EventEmitter()

    # Track events
    events_received = []

    def handler(payload):
        events_received.append(payload)

    emitter.on("cross_dcc_test", handler)
    emitter.emit("cross_dcc_test", {"data": "test"})

    assert len(events_received) == 1
    assert events_received[0]["data"] == "test"


def test_dcc_file_protocol():
    """Test file protocol handling for DCC assets."""
    from auroraview.utils.file_protocol import path_to_auroraview_url

    # Test various DCC asset paths
    test_paths = [
        "/mnt/projects/maya/scenes/main.ma",
        "/mnt/projects/blender/characters/hero.blend",
        "/mnt/projects/houdini/fx/explosion.hip",
        "C:\\Projects\\Maya\\scenes\\main.ma",
        "C:\\Projects\\Blender\\characters\\hero.blend",
    ]

    for path in test_paths:
        url = path_to_auroraview_url(path)
        assert url.startswith("https://auroraview.localhost/file/")


def test_dcc_config_loading():
    """Test loading DCC-specific configurations."""
    from auroraview.core.config import WebViewConfig, WindowConfig

    config = WebViewConfig(window=WindowConfig(title="test", width=800, height=600))

    # Test that config can be created with DCC-related settings
    assert config.window.title == "test"
    assert config.window.width == 800


def test_mcp_integration_structure():
    """Test MCP integration structure for DCC tools."""
    # Test that MCP-related modules can be imported
    try:
        from auroraview.mcp import MCPServer  # noqa: F401

        assert True
    except ImportError:
        # MCP module might be optional
        pytest.skip("MCP module not available")


def test_dcc_thread_safety():
    """Test thread safety utilities for DCC environments."""
    from auroraview.utils.thread_dispatcher import DCCThreadSafeWrapper

    # Create a mock WebView-like object for testing
    class MockWebView:
        def eval_js(self, code):
            pass

        def emit(self, event, data):
            pass

        def get_proxy(self):
            return self

    wrapper = DCCThreadSafeWrapper(MockWebView())

    # Test that wrapper has required methods
    assert hasattr(wrapper, "eval_js")
    assert hasattr(wrapper, "emit")
    assert hasattr(wrapper, "load_url")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
