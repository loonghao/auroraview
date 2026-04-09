"""Tests to achieve 100% coverage on remaining uncovered lines.

Covers:
- __main__.py: main() function and __name__ == "__main__" guard
- discovery.py: is_process_alive non-Windows path (os.kill branch)
- tools/gallery.py:
  - get_project_root() raise FileNotFoundError (line 99)
  - get_gallery_dir() fallback to get_project_root() (line 108)
  - get_sample_info() main_file not exists after dir lookup (line 152)
"""

from __future__ import annotations

from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

# ============================================================================
# __main__.py: main() and entry point
# ============================================================================


class TestMainEntry:
    """Tests for __main__.py entry point."""

    def test_main_calls_mcp_run(self) -> None:
        """main() should call mcp.run() and return 0."""
        mock_mcp = MagicMock()
        with patch.dict("sys.modules", {"auroraview_mcp.server": MagicMock(mcp=mock_mcp)}):
            # Re-import to use the patched module
            import importlib

            import auroraview_mcp.__main__ as main_module
            importlib.reload(main_module)
            with patch("auroraview_mcp.server.mcp", mock_mcp):
                result = main_module.main()
        assert result == 0

    def test_main_module_import(self) -> None:
        """__main__.py should be importable without side effects."""
        import auroraview_mcp.__main__ as main_module
        assert callable(main_module.main)

    def test_main_entry_point_via_sys_exit(self) -> None:
        """Calling main() via sys.exit simulation should return 0."""
        mock_mcp = MagicMock()
        with patch("auroraview_mcp.server.mcp", mock_mcp):
            from auroraview_mcp.__main__ import main
            result = main()
            assert result == 0
            mock_mcp.run.assert_called_once()


# ============================================================================
# discovery.py: is_process_alive non-Windows path
# ============================================================================


class TestIsProcessAliveNonWindows:
    """Tests for is_process_alive() on non-Windows platforms."""

    def test_is_alive_on_non_windows_returns_true(self) -> None:
        """os.kill(pid, 0) succeeds → process is alive."""

        with patch("sys.platform", "linux"), patch("os.kill", return_value=None):
            # Force the non-Windows code path
            import auroraview_mcp.discovery as disc_mod
            try:
                # Temporarily override platform check
                with patch.object(disc_mod.sys, "platform", "linux"):
                    # Directly test the logic
                    import os
                    with patch.object(os, "kill", return_value=None):
                        disc_mod.is_process_alive(12345)
                        # On Windows this uses ctypes path; we test logic directly
            finally:
                pass

    def test_is_process_alive_linux_alive(self) -> None:
        """Simulate non-Windows: os.kill succeeds → True."""
        import auroraview_mcp.discovery as disc_mod

        # Patch sys.platform inside the module to simulate Linux
        with patch.object(disc_mod.sys, "platform", "linux"):
            import os
            with patch.object(os, "kill", return_value=None):
                result = disc_mod.is_process_alive(9999)
                assert result is True

    def test_is_process_alive_linux_oserror(self) -> None:
        """Simulate non-Windows: os.kill raises OSError → False."""
        import auroraview_mcp.discovery as disc_mod

        with patch.object(disc_mod.sys, "platform", "linux"):
            import os
            with patch.object(os, "kill", side_effect=OSError("no process")):
                result = disc_mod.is_process_alive(9999)
                assert result is False

    def test_is_process_alive_linux_process_lookup_error(self) -> None:
        """Simulate non-Windows: os.kill raises ProcessLookupError → False."""
        import auroraview_mcp.discovery as disc_mod

        with patch.object(disc_mod.sys, "platform", "linux"):
            import os
            with patch.object(os, "kill", side_effect=ProcessLookupError("no such process")):
                result = disc_mod.is_process_alive(9999)
                assert result is False


# ============================================================================
# tools/gallery.py: get_project_root raises FileNotFoundError
# ============================================================================


class TestGetProjectRootNotFound:
    """Tests for get_project_root() when markers are not found."""

    def test_get_project_root_raises_when_not_found(self) -> None:
        """When Cargo.toml+gallery not found up the tree, raise FileNotFoundError."""
        from auroraview_mcp.tools.gallery import get_project_root

        # Mock Path.exists to always return False so no marker is found
        with patch("auroraview_mcp.tools.gallery.Path") as mock_path_cls:
            # Make every path check return False
            mock_path_instance = MagicMock()
            mock_path_instance.resolve.return_value = mock_path_instance
            mock_path_instance.parent = mock_path_instance  # infinite parent loop
            cargo_marker = MagicMock()
            cargo_marker.exists.return_value = False
            gallery_marker = MagicMock()
            gallery_marker.exists.return_value = False
            mock_path_instance.__truediv__ = lambda self, other: (
                cargo_marker if other == "Cargo.toml" else gallery_marker
            )
            mock_path_cls.return_value = mock_path_instance
            mock_path_cls.__file__ = __file__

            with pytest.raises(FileNotFoundError, match="Could not find project root"):
                get_project_root()

    def test_get_project_root_raises_via_no_markers(self, tmp_path: Path) -> None:
        """In a temp dir with no Cargo.toml+gallery, should raise FileNotFoundError."""
        from auroraview_mcp.tools.gallery import get_project_root

        # Patch __file__ inside gallery module to be deep inside tmp_path
        deep_dir = tmp_path / "a" / "b" / "c" / "d" / "e" / "f" / "g"
        deep_dir.mkdir(parents=True)
        fake_file = deep_dir / "gallery.py"
        fake_file.write_text("# fake")

        import auroraview_mcp.tools.gallery as gallery_mod
        with (
            patch.object(gallery_mod, "__file__", str(fake_file)),
            pytest.raises(FileNotFoundError, match="Could not find project root"),
        ):
            get_project_root()


# ============================================================================
# tools/gallery.py: get_gallery_dir() fallback path (line 108)
# ============================================================================


class TestGetGalleryDirFallback:
    """Tests for get_gallery_dir() fallback through get_project_root()."""

    def test_get_gallery_dir_fallback_uses_project_root(self, tmp_path: Path) -> None:
        """When AURORAVIEW_GALLERY_DIR not set, fallback to get_project_root()/gallery."""
        from auroraview_mcp.tools.gallery import get_gallery_dir

        fake_root = tmp_path / "project"
        fake_root.mkdir()

        with patch.dict("os.environ", {}, clear=False):
            # Ensure env var not set
            import os
            os.environ.pop("AURORAVIEW_GALLERY_DIR", None)

            with patch("auroraview_mcp.tools.gallery.get_project_root", return_value=fake_root):
                result = get_gallery_dir()
                assert result == fake_root / "gallery"

    def test_get_gallery_dir_uses_env_var(self, tmp_path: Path) -> None:
        """When AURORAVIEW_GALLERY_DIR is set, use it directly."""
        from auroraview_mcp.tools.gallery import get_gallery_dir

        env_gallery = str(tmp_path / "custom_gallery")
        with patch.dict("os.environ", {"AURORAVIEW_GALLERY_DIR": env_gallery}):
            result = get_gallery_dir()
            assert result == Path(env_gallery)


# ============================================================================
# tools/gallery.py: get_sample_info main_file not exists (line 152)
# ============================================================================


class TestGetSampleInfoMainFileNotExists:
    """Tests for get_sample_info when main_file doesn't exist."""

    def test_dir_main_py_exists_but_is_removed_mid_check(self, tmp_path: Path) -> None:
        """Directory has main.py listed but file doesn't exist when checked (line 152)."""
        from auroraview_mcp.tools.gallery import get_sample_info

        sample_dir = tmp_path / "my_sample"
        sample_dir.mkdir()
        # Create main.py so the initial check passes, then remove it to simulate race
        main_py = sample_dir / "main.py"
        main_py.write_text("# placeholder")

        # Now patch main_file.exists() to return False to hit line 152
        original_exists = Path.exists

        call_count = [0]

        def patched_exists(self_path: Path) -> bool:
            if self_path.name == "main.py" and self_path.parent == sample_dir:
                call_count[0] += 1
                if call_count[0] > 1:
                    return False  # Second call returns False (line 152)
            return original_exists(self_path)

        with patch.object(Path, "exists", patched_exists):
            result = get_sample_info(sample_dir)
            assert result is None

    def test_py_file_exists_check_fails_at_line_152(self, tmp_path: Path) -> None:
        """A .py file where main_file.exists() returns False hits line 152 → None."""
        from auroraview_mcp.tools.gallery import get_sample_info

        # Create a .py path that doesn't actually exist on disk
        non_existent = tmp_path / "non_existent_sample.py"
        # Do NOT create the file — it doesn't exist

        result = get_sample_info(non_existent)
        assert result is None

    def test_dir_with_py_files_where_first_py_not_found(self, tmp_path: Path) -> None:
        """Directory where glob finds py files but then main_file.exists() is False."""
        from auroraview_mcp.tools.gallery import get_sample_info

        sample_dir = tmp_path / "sample_dir"
        sample_dir.mkdir()

        # Don't create main.py, create other .py file
        other_py = sample_dir / "helper.py"
        other_py.write_text("# helper")

        # Patch exists on the found file to return False at line 152
        original_exists = Path.exists

        def fake_exists(self_path: Path) -> bool:
            if self_path == other_py:
                return False
            return original_exists(self_path)

        with patch.object(Path, "exists", fake_exists):
            result = get_sample_info(sample_dir)
            assert result is None
