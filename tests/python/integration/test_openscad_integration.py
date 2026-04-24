"""OpenSCAD integration tests for AuroraView.

Tests AuroraView functionality with OpenSCAD.
"""

import os
import subprocess

import pytest


@pytest.mark.openscad
def test_openscad_cli_available():
    """Test that OpenSCAD CLI is available."""
    try:
        result = subprocess.run(
            ["openscad", "--version"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode == 0
        assert "OpenSCAD" in result.stdout or "OpenSCAD" in result.stderr
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pytest.skip("OpenSCAD not available")


@pytest.mark.openscad
def test_auroraview_import_with_openscad():
    """Test that AuroraView can be imported when OpenSCAD is available."""
    try:
        import openscad  # noqa: F401

        from auroraview import WebView

        assert WebView is not None
    except ImportError:
        # OpenSCAD doesn't have a Python API, just check AuroraView
        from auroraview import WebView

        assert WebView is not None


@pytest.mark.openscad
def test_openscad_file_rendering():
    """Test rendering OpenSCAD file preview with AuroraView."""
    # Create a simple .scad file
    test_scad = "/tmp/test_model.scad"
    with open(test_scad, "w") as f:
        f.write("cube([10, 10, 10]);\n")
        f.write("translate([20, 0, 0]) sphere(r=5);\n")

    assert os.path.exists(test_scad)

    # Test that AuroraView can load the file URL
    from auroraview.utils.file_protocol import path_to_file_url

    file_url = path_to_file_url(test_scad)
    assert file_url.startswith("file://")

    # Cleanup
    os.remove(test_scad)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
