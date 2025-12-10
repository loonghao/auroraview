# -*- coding: utf-8 -*-
"""Tests for file drop events."""


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
                {"name": "document.pdf", "size": 4096, "type": "application/pdf", "lastModified": 1234567890}
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
            "files": [{"name": "clipboard_image.png", "size": 8192, "type": "image/png", "lastModified": 0}],
            "timestamp": 1234567890123,
        }
        data = WindowEventData(event_data)

        assert data.files is not None
        assert data.files[0]["name"] == "clipboard_image.png"
        assert data.timestamp == 1234567890123
