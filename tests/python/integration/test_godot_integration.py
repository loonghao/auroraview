"""Godot integration tests for AuroraView.

Tests AuroraView functionality with Godot engine integration.
"""

import pytest


@pytest.mark.godot
def test_godot_import_availability():
    """Test that AuroraView can be imported in a Godot project context."""
    from auroraview import WebView

    assert WebView is not None


@pytest.mark.godot
def test_godot_scene_file_protocol():
    """Test file protocol handling for Godot scene files."""
    from auroraview.utils.file_protocol import path_to_auroraview_url

    test_paths = [
        "/mnt/projects/godot/scenes/main.tscn",
        "/mnt/projects/godot/scripts/player.gd",
    ]

    for path in test_paths:
        url = path_to_auroraview_url(path)
        assert url.startswith("https://auroraview.localhost/file/")


@pytest.mark.godot
def test_godot_dcc_environment_detection():
    """Test DCC environment detection for Godot."""
    try:
        from auroraview.utils.thread_dispatcher import get_current_dcc_name

        # Without Godot running, may return None or a string
        dcc_name = get_current_dcc_name()
        if dcc_name is not None:
            assert isinstance(dcc_name, str)
    except (ImportError, AttributeError):
        pytest.skip("DCC environment detection not available")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
