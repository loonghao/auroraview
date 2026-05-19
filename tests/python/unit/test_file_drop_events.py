# -*- coding: utf-8 -*-
"""Tests for file drop events."""

import inspect

import pytest

from auroraview.core.events import WindowEvent, WindowEventData


class TestFileDropEvents:
    """Tests for file drop event types."""

    def test_file_drop_event_exists(self):
        """Test FILE_DROP event is defined."""
        assert WindowEvent.FILE_DROP.value == "file_drop"

    def test_file_drop_hover_event_exists(self):
        """Test FILE_DROP_HOVER event is defined."""
        assert WindowEvent.FILE_DROP_HOVER.value == "file_drop_hover"

    def test_file_drop_cancelled_event_exists(self):
        """Test FILE_DROP_CANCELLED event is defined."""
        assert WindowEvent.FILE_DROP_CANCELLED.value == "file_drop_cancelled"

    def test_file_paste_event_exists(self):
        """Test FILE_PASTE event is defined."""
        assert WindowEvent.FILE_PASTE.value == "file_paste"

    def test_file_drop_event_str(self):
        """Test file drop events string conversion."""
        assert str(WindowEvent.FILE_DROP) == "file_drop"
        assert str(WindowEvent.FILE_DROP_HOVER) == "file_drop_hover"
        assert str(WindowEvent.FILE_DROP_CANCELLED) == "file_drop_cancelled"
        assert str(WindowEvent.FILE_PASTE) == "file_paste"


class TestFileDropEventData:
    """Tests for file drop event data properties."""

    def test_files_property(self):
        """Test files property for file drop events."""
        files = [
            {"name": "test.txt", "size": 1024, "type": "text/plain", "lastModified": 1234567890},
            {"name": "image.png", "size": 2048, "type": "image/png", "lastModified": 1234567891},
        ]
        data = WindowEventData({"files": files})

        assert data.files is not None
        assert len(data.files) == 2
        assert data.files[0]["name"] == "test.txt"
        assert data.files[1]["type"] == "image/png"

    def test_files_property_empty(self):
        """Test files property when not present."""
        data = WindowEventData({})
        assert data.files is None

    def test_paths_property(self):
        """Test paths property for file drop events."""
        paths = ["/path/to/file1.txt", "/path/to/file2.png"]
        data = WindowEventData({"paths": paths})

        assert data.paths is not None
        assert len(data.paths) == 2
        assert data.paths[0] == "/path/to/file1.txt"

    def test_paths_property_empty(self):
        """Test paths property when not present."""
        data = WindowEventData({})
        assert data.paths is None

    def test_position_property(self):
        """Test position property for file drop events."""
        position = {"x": 100, "y": 200, "screenX": 500, "screenY": 600}
        data = WindowEventData({"position": position})

        assert data.position is not None
        assert data.position["x"] == 100
        assert data.position["y"] == 200
        assert data.position["screenX"] == 500
        assert data.position["screenY"] == 600

    def test_position_property_empty(self):
        """Test position property when not present."""
        data = WindowEventData({})
        assert data.position is None

    def test_hovering_property(self):
        """Test hovering property for file drop hover events."""
        data = WindowEventData({"hovering": True})
        assert data.hovering is True

        data = WindowEventData({"hovering": False})
        assert data.hovering is False

    def test_hovering_property_empty(self):
        """Test hovering property when not present."""
        data = WindowEventData({})
        assert data.hovering is None

    def test_reason_property(self):
        """Test reason property for file drop cancelled events."""
        data = WindowEventData({"reason": "left_window"})
        assert data.reason == "left_window"

        data = WindowEventData({"reason": "no_files"})
        assert data.reason == "no_files"

    def test_reason_property_empty(self):
        """Test reason property when not present."""
        data = WindowEventData({})
        assert data.reason is None

    def test_timestamp_property(self):
        """Test timestamp property for events."""
        data = WindowEventData({"timestamp": 1234567890123})
        assert data.timestamp == 1234567890123

    def test_timestamp_property_empty(self):
        """Test timestamp property when not present."""
        data = WindowEventData({})
        assert data.timestamp is None

    def test_complete_file_drop_event_data(self):
        """Test complete file drop event data with all properties."""
        event_data = {
            "files": [
                {
                    "name": "document.pdf",
                    "size": 4096,
                    "type": "application/pdf",
                    "lastModified": 1234567890,
                }
            ],
            "paths": ["/downloads/document.pdf"],
            "position": {"x": 150, "y": 250, "screenX": 650, "screenY": 750},
            "timestamp": 1234567890123,
        }
        data = WindowEventData(event_data)

        assert data.files is not None
        assert len(data.files) == 1
        assert data.files[0]["name"] == "document.pdf"
        assert data.paths == ["/downloads/document.pdf"]
        assert data.position["x"] == 150
        assert data.timestamp == 1234567890123

    def test_complete_file_drop_hover_event_data(self):
        """Test complete file drop hover event data."""
        event_data = {
            "hovering": True,
            "files": [{"name": "test.txt", "size": 100, "type": "text/plain", "lastModified": 0}],
            "position": {"x": 50, "y": 75, "screenX": 100, "screenY": 200},
        }
        data = WindowEventData(event_data)

        assert data.hovering is True
        assert data.files is not None
        assert data.position is not None

    def test_complete_file_drop_cancelled_event_data(self):
        """Test complete file drop cancelled event data."""
        event_data = {"hovering": False, "reason": "left_window"}
        data = WindowEventData(event_data)

        assert data.hovering is False
        assert data.reason == "left_window"

    def test_complete_file_paste_event_data(self):
        """Test complete file paste event data."""
        event_data = {
            "files": [
                {
                    "name": "clipboard_image.png",
                    "size": 8192,
                    "type": "image/png",
                    "lastModified": 0,
                }
            ],
            "timestamp": 1234567890123,
        }
        data = WindowEventData(event_data)

        assert data.files is not None
        assert data.files[0]["name"] == "clipboard_image.png"
        assert data.timestamp == 1234567890123


# ============================================================================
# RFC 0013: use_default_file_drop toggle
# ============================================================================


@pytest.fixture
def webview_create_signature():
    """Return ``inspect.Signature`` for ``WebView.create``."""
    from auroraview.core.webview import WebView

    return inspect.signature(WebView.create)


@pytest.fixture
def webview_init_signature():
    """Return ``inspect.Signature`` for ``WebView.__init__``."""
    from auroraview.core.webview import WebView

    return inspect.signature(WebView.__init__)


class TestFileDropToggleSignature:
    """RFC 0013 (revised): ensure the kwarg is wired from Python to the Rust
    layer.

    These tests are signature-only so they run on every platform without
    spawning a real WebView. They guard against:

    - The kwarg disappearing from the public Python API.
    - The default ever flipping away from ``None`` (which means "use the
      Rust-side default", currently ``False`` = install the wry handler).
    """

    def test_create_kwarg_exposed(self, webview_create_signature):
        params = webview_create_signature.parameters
        assert "use_default_file_drop" in params

    def test_create_kwarg_default_is_none(self, webview_create_signature):
        param = webview_create_signature.parameters["use_default_file_drop"]
        assert param.default is None

    def test_init_kwarg_exposed(self, webview_init_signature):
        params = webview_init_signature.parameters
        assert "use_default_file_drop" in params

    def test_init_kwarg_default_is_none(self, webview_init_signature):
        param = webview_init_signature.parameters["use_default_file_drop"]
        assert param.default is None


class TestFileDropToggleExplicitTrue:
    """Verify the kwarg is forwarded when an explicit ``True`` is provided."""

    def test_create_explicit_true_passes_through(self, monkeypatch):
        """``WebView.create(use_default_file_drop=True)`` reaches __init__."""
        from auroraview.core import webview as webview_module

        captured = {}

        def fake_init(self, **kwargs):
            captured.update(kwargs)
            self._auto_timer = None

        monkeypatch.setattr(webview_module.WebView, "__init__", fake_init)
        monkeypatch.setattr(webview_module.WebView, "show", lambda self, *_a, **_kw: None)
        # Avoid singleton pollution between tests.
        webview_module.WebView._singleton_registry = {}

        instance = webview_module.WebView.create(use_default_file_drop=True, auto_show=False)
        assert isinstance(instance, webview_module.WebView)
        assert captured.get("use_default_file_drop") is True

    def test_create_explicit_false_passes_through(self, monkeypatch):
        """``WebView.create(use_default_file_drop=False)`` reaches __init__."""
        from auroraview.core import webview as webview_module

        captured = {}

        def fake_init(self, **kwargs):
            captured.update(kwargs)
            self._auto_timer = None

        monkeypatch.setattr(webview_module.WebView, "__init__", fake_init)
        monkeypatch.setattr(webview_module.WebView, "show", lambda self, *_a, **_kw: None)
        webview_module.WebView._singleton_registry = {}

        webview_module.WebView.create(use_default_file_drop=False, auto_show=False)
        assert captured.get("use_default_file_drop") is False

    def test_create_omitted_defaults_to_none(self, monkeypatch):
        """Omitting the kwarg forwards ``None`` (Rust-side default = False,
        which under the revised RFC 0013 semantics means "install the wry
        handler and emit file_drop_* events")."""
        from auroraview.core import webview as webview_module

        captured = {}

        def fake_init(self, **kwargs):
            captured.update(kwargs)
            self._auto_timer = None

        monkeypatch.setattr(webview_module.WebView, "__init__", fake_init)
        monkeypatch.setattr(webview_module.WebView, "show", lambda self, *_a, **_kw: None)
        webview_module.WebView._singleton_registry = {}

        webview_module.WebView.create(auto_show=False)
        assert captured.get("use_default_file_drop") is None
