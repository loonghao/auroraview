"""Tests for Gallery tools."""

from __future__ import annotations

import os
import tempfile
from pathlib import Path
from unittest.mock import MagicMock, patch

from auroraview_mcp.tools.gallery import (
    ProcessInfo,
    ProcessManager,
    _process_manager,
    get_examples_dir,
    get_gallery_dir,
    get_project_root,
    get_sample_info,
    scan_samples,
)


class TestProcessManager:
    """Tests for ProcessManager class."""

    def test_add_process(self) -> None:
        """Test adding a process."""
        manager = ProcessManager()
        mock_process = MagicMock()
        mock_process.poll.return_value = None

        info = ProcessInfo(pid=1234, name="test", process=mock_process)
        manager.add(info)

        assert manager.get(1234) == info

    def test_remove_process(self) -> None:
        """Test removing a process."""
        manager = ProcessManager()
        mock_process = MagicMock()

        info = ProcessInfo(pid=1234, name="test", process=mock_process)
        manager.add(info)

        removed = manager.remove(1234)
        assert removed == info
        assert manager.get(1234) is None

    def test_get_by_name(self) -> None:
        """Test getting process by name."""
        manager = ProcessManager()
        mock_process = MagicMock()

        info = ProcessInfo(pid=1234, name="my_sample", process=mock_process)
        manager.add(info)

        found = manager.get_by_name("my_sample")
        assert found == info

        not_found = manager.get_by_name("other")
        assert not_found is None

    def test_get_gallery(self) -> None:
        """Test getting Gallery process."""
        manager = ProcessManager()
        mock_process = MagicMock()

        # Regular sample
        sample_info = ProcessInfo(pid=1234, name="sample", process=mock_process)
        manager.add(sample_info)

        # Gallery process
        gallery_info = ProcessInfo(pid=5678, name="gallery", process=mock_process, is_gallery=True)
        manager.add(gallery_info)

        found = manager.get_gallery()
        assert found == gallery_info
        assert found.is_gallery is True

    def test_list_all(self) -> None:
        """Test listing all processes."""
        manager = ProcessManager()
        mock_process1 = MagicMock()
        mock_process2 = MagicMock()

        info1 = ProcessInfo(pid=1234, name="sample1", process=mock_process1)
        info2 = ProcessInfo(pid=5678, name="sample2", process=mock_process2)
        manager.add(info1)
        manager.add(info2)

        all_processes = manager.list_all()
        assert len(all_processes) == 2
        assert info1 in all_processes
        assert info2 in all_processes

    def test_cleanup(self) -> None:
        """Test cleanup of terminated processes."""
        manager = ProcessManager()

        # Running process
        running = MagicMock()
        running.poll.return_value = None
        info1 = ProcessInfo(pid=1234, name="running", process=running)

        # Terminated process
        terminated = MagicMock()
        terminated.poll.return_value = 0
        info2 = ProcessInfo(pid=5678, name="terminated", process=terminated)

        manager.add(info1)
        manager.add(info2)

        manager.cleanup()

        assert manager.get(1234) is not None  # Still there
        assert manager.get(5678) is None  # Cleaned up


class TestGetSampleInfo:
    """Tests for get_sample_info function."""

    def test_sample_file_with_docstring(self) -> None:
        """Test sample file with docstring."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "test_demo.py"
            sample_file.write_text(
                '"""Test Sample\n\nThis is a test description.\n"""\nprint("hello")'
            )

            info = get_sample_info(sample_file)

            assert info is not None
            assert info["name"] == "test"
            assert info["title"] == "Test Sample"
            assert "test description" in info["description"]

    def test_sample_dir_with_main_py(self) -> None:
        """Test sample directory with main.py file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "test_sample"
            sample_dir.mkdir()

            main_file = sample_dir / "main.py"
            main_file.write_text('"""Test Sample\n\nThis is a test.\n"""\nprint("hello")')

            info = get_sample_info(sample_dir)

            assert info is not None
            assert info["name"] == "test_sample"
            assert info["title"] == "Test Sample"
            assert info["description"] == "This is a test."

    def test_sample_without_py_files(self) -> None:
        """Test sample without Python files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "empty_sample"
            sample_dir.mkdir()

            info = get_sample_info(sample_dir)
            assert info is None

    def test_sample_with_other_py_file(self) -> None:
        """Test sample with non-main.py file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "other_sample"
            sample_dir.mkdir()

            other_file = sample_dir / "app.py"
            other_file.write_text("# Simple app\nprint('app')")

            info = get_sample_info(sample_dir)

            assert info is not None
            assert info["name"] == "other_sample"

    def test_sample_with_title_separator(self) -> None:
        """Test sample with title containing separator."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "demo.py"
            sample_file.write_text(
                '"""My Demo - A demonstration app\n\nDetailed description here.\n"""'
            )

            info = get_sample_info(sample_file)

            assert info is not None
            assert info["title"] == "My Demo"


class TestScanSamples:
    """Tests for scan_samples function."""

    def test_scan_empty_dir(self) -> None:
        """Test scanning empty directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            samples = scan_samples(Path(tmpdir))
            assert samples == []

    def test_scan_with_py_files(self) -> None:
        """Test scanning directory with Python files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            # Create sample files
            (examples_dir / "demo1.py").write_text('"""Demo 1"""\nprint(1)')
            (examples_dir / "demo2.py").write_text('"""Demo 2"""\nprint(2)')
            (examples_dir / "__init__.py").write_text("")  # Should be skipped

            samples = scan_samples(examples_dir)

            assert len(samples) == 2
            names = [s["name"] for s in samples]
            assert "demo1" in names
            assert "demo2" in names

    def test_scan_with_subdirs(self) -> None:
        """Test scanning directory with subdirectories."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            # Create sample directory
            sample_dir = examples_dir / "my_sample"
            sample_dir.mkdir()
            (sample_dir / "main.py").write_text('"""My Sample"""\nprint("hi")')

            # Create hidden directory (should be skipped)
            hidden_dir = examples_dir / ".hidden"
            hidden_dir.mkdir()
            (hidden_dir / "main.py").write_text('"""Hidden"""\nprint("hidden")')

            samples = scan_samples(examples_dir)

            assert len(samples) == 1
            assert samples[0]["name"] == "my_sample"

    def test_scan_nonexistent_dir(self) -> None:
        """Test scanning non-existent directory."""
        samples = scan_samples(Path("/nonexistent/path"))
        assert samples == []


class TestPathFunctions:
    """Tests for path resolution functions."""

    def test_get_examples_dir_from_env(self) -> None:
        """Test get_examples_dir with environment variable."""
        with (
            tempfile.TemporaryDirectory() as tmpdir,
            patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
        ):
            result = get_examples_dir()
            assert result == Path(tmpdir)

    def test_get_gallery_dir_from_env(self) -> None:
        """Test get_gallery_dir with environment variable."""
        with (
            tempfile.TemporaryDirectory() as tmpdir,
            patch.dict(os.environ, {"AURORAVIEW_GALLERY_DIR": tmpdir}),
        ):
            result = get_gallery_dir()
            assert result == Path(tmpdir)

    def test_get_project_root_from_env(self) -> None:
        """Test get_project_root with environment variable."""
        with (
            tempfile.TemporaryDirectory() as tmpdir,
            patch.dict(os.environ, {"AURORAVIEW_PROJECT_ROOT": tmpdir}),
        ):
            result = get_project_root()
            assert result == Path(tmpdir)


class TestGalleryToolsInternal:
    """Tests for Gallery tool internal functions.

    Note: MCP tools are decorated with @mcp.tool() which wraps them
    in FunctionTool objects. These tests use the internal implementation
    directly or test via the MCP server context.
    """

    def test_process_manager_global_instance(self) -> None:
        """Test global process manager exists."""
        assert _process_manager is not None
        assert isinstance(_process_manager, ProcessManager)

    def test_scan_samples_integration(self) -> None:
        """Test scan_samples with real examples directory structure."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            # Create a realistic sample structure
            (examples_dir / "hello_demo.py").write_text(
                '"""Hello World Demo\n\nSimple hello world example.\n"""\nprint("Hello, World!")'
            )

            (examples_dir / "advanced_demo.py").write_text(
                '"""Advanced Demo - Complex features\n\n'
                'This demo shows advanced features.\n"""\n'
                'print("Advanced")'
            )

            # Create subdirectory sample
            subdir = examples_dir / "multi_file_sample"
            subdir.mkdir()
            (subdir / "main.py").write_text(
                '"""Multi-file Sample\n\nSample with multiple files.\n"""\n'
                "from helper import do_something"
            )
            (subdir / "helper.py").write_text("def do_something(): pass")

            samples = scan_samples(examples_dir)

            assert len(samples) == 3

            # Check hello demo
            hello = next(s for s in samples if s["name"] == "hello")
            assert hello["title"] == "Hello World Demo"
            assert "hello world" in hello["description"].lower()

            # Check advanced demo
            advanced = next(s for s in samples if s["name"] == "advanced")
            assert advanced["title"] == "Advanced Demo"

            # Check multi-file sample
            multi = next(s for s in samples if s["name"] == "multi_file_sample")
            assert multi["title"] == "Multi-file Sample"

    def test_get_sample_info_with_complex_docstring(self) -> None:
        """Test get_sample_info with complex docstring."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "complex_demo.py"
            sample_file.write_text('''"""Complex Demo - A feature-rich demonstration

This demo showcases multiple features including:
- Feature A
- Feature B
- Feature C

Features:
- Advanced rendering
- Real-time updates

Use Cases:
- Production workflows
- Testing scenarios
"""

import sys
print("Complex demo")
''')

            info = get_sample_info(sample_file)

            assert info is not None
            assert info["title"] == "Complex Demo"
            # Description should be extracted from content lines
            assert (
                "showcases" in info["description"].lower() or "demo" in info["description"].lower()
            )

    def test_process_info_dataclass(self) -> None:
        """Test ProcessInfo dataclass."""
        mock_process = MagicMock()

        info = ProcessInfo(
            pid=1234,
            name="test_sample",
            process=mock_process,
            port=9222,
            is_gallery=False,
        )

        assert info.pid == 1234
        assert info.name == "test_sample"
        assert info.process == mock_process
        assert info.port == 9222
        assert info.is_gallery is False

    def test_process_info_defaults(self) -> None:
        """Test ProcessInfo default values."""
        mock_process = MagicMock()

        info = ProcessInfo(pid=1, name="test", process=mock_process)

        assert info.port is None
        assert info.is_gallery is False

    def test_process_manager_isolation(self) -> None:
        """Test that ProcessManager instances are isolated."""
        manager1 = ProcessManager()
        manager2 = ProcessManager()

        mock_process = MagicMock()
        info = ProcessInfo(pid=1234, name="test", process=mock_process)

        manager1.add(info)

        assert manager1.get(1234) == info
        assert manager2.get(1234) is None

    def test_get_sample_info_no_docstring(self) -> None:
        """Test get_sample_info with file without docstring."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "no_doc_demo.py"
            sample_file.write_text('# No docstring\nprint("hello")')

            info = get_sample_info(sample_file)

            assert info is not None
            assert info["name"] == "no_doc"
            # Should use filename-derived title
            assert info["title"] == "No Doc"

    def test_get_sample_info_invalid_file(self) -> None:
        """Test get_sample_info with non-Python file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "readme.txt"
            sample_file.write_text("This is not a Python file")

            info = get_sample_info(sample_file)
            assert info is None

    def test_scan_samples_skips_init(self) -> None:
        """Test that scan_samples skips __init__.py files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            (examples_dir / "__init__.py").write_text('"""Init file"""')
            (examples_dir / "__pycache__").mkdir()
            (examples_dir / "demo.py").write_text('"""Demo"""')

            samples = scan_samples(examples_dir)

            assert len(samples) == 1
            assert samples[0]["name"] == "demo"

    def test_scan_samples_skips_hidden(self) -> None:
        """Test that scan_samples skips hidden directories."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            # Hidden directory
            hidden = examples_dir / ".hidden"
            hidden.mkdir()
            (hidden / "main.py").write_text('"""Hidden"""')

            # Underscore directory
            underscore = examples_dir / "_private"
            underscore.mkdir()
            (underscore / "main.py").write_text('"""Private"""')

            # Normal directory
            normal = examples_dir / "public"
            normal.mkdir()
            (normal / "main.py").write_text('"""Public"""')

            samples = scan_samples(examples_dir)

            assert len(samples) == 1
            assert samples[0]["name"] == "public"


class TestPathResolution:
    """Tests for path resolution edge cases."""

    def test_get_project_root_from_traversal(self) -> None:
        """Test get_project_root finds project via directory traversal."""
        # When AURORAVIEW_PROJECT_ROOT is not set, the function traverses
        # up from __file__ looking for Cargo.toml + gallery directory
        # This test verifies the function works in the actual project
        with patch.dict(os.environ, {}, clear=False):
            # Remove the env var if set
            env = os.environ.copy()
            env.pop("AURORAVIEW_PROJECT_ROOT", None)
            with patch.dict(os.environ, env):
                # Should find the project root via traversal
                try:
                    result = get_project_root()
                    # If found, should have Cargo.toml
                    assert (result / "Cargo.toml").exists()
                except FileNotFoundError:
                    # OK if not in project context
                    pass

    def test_get_examples_dir_fallback(self) -> None:
        """Test get_examples_dir falls back to project root."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Set project root but not examples dir
            env = os.environ.copy()
            env["AURORAVIEW_PROJECT_ROOT"] = tmpdir
            env.pop("AURORAVIEW_EXAMPLES_DIR", None)
            with patch.dict(os.environ, env, clear=True):
                result = get_examples_dir()
                assert result == Path(tmpdir) / "examples"
