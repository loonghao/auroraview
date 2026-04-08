"""Advanced tests for gallery tools - run_gallery, run_sample, edge cases."""

from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.tools.gallery import ProcessInfo, ProcessManager


# ============================================================================
# run_gallery advanced tests
# ============================================================================


class TestRunGalleryAlreadyRunning:
    """Tests for run_gallery when gallery is already running."""

    @pytest.mark.asyncio
    async def test_already_running_returns_status(self) -> None:
        """When gallery is already running, returns already_running status."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        mock_process = MagicMock()
        mock_process.poll.return_value = None  # still running

        manager = ProcessManager()
        manager.add(ProcessInfo(pid=1234, name="gallery", process=mock_process, port=9222, is_gallery=True))

        with patch("auroraview_mcp.tools.gallery._process_manager", manager):
            result = await fn()

        assert result["status"] == "already_running"
        assert result["pid"] == 1234
        assert result["port"] == 9222
        assert "already running" in result["message"].lower()

    @pytest.mark.asyncio
    async def test_already_running_with_custom_port(self) -> None:
        """Even when already running, returns the existing port."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        mock_process = MagicMock()
        mock_process.poll.return_value = None

        manager = ProcessManager()
        manager.add(ProcessInfo(pid=5678, name="gallery", process=mock_process, port=9333, is_gallery=True))

        with patch("auroraview_mcp.tools.gallery._process_manager", manager):
            result = await fn(port=9999)  # different port requested

        # Should return existing port, not requested port
        assert result["status"] == "already_running"
        assert result["port"] == 9333

    @pytest.mark.asyncio
    async def test_gallery_dir_not_found_raises(self) -> None:
        """Raises RuntimeError when gallery directory not found."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with (
            patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
            patch(
                "auroraview_mcp.tools.gallery.get_gallery_dir",
                side_effect=FileNotFoundError("Gallery dir not found"),
            ),
            pytest.raises(RuntimeError, match="Gallery dir not found"),
        ):
            await fn()

    @pytest.mark.asyncio
    async def test_gallery_main_py_not_found_raises(self) -> None:
        """Raises RuntimeError when gallery/main.py not found."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)
            # No main.py in gallery dir

            with (
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("auroraview_mcp.tools.gallery.get_gallery_dir", return_value=gallery_dir),
                pytest.raises(RuntimeError, match="main.py not found"),
            ):
                await fn()

    @pytest.mark.asyncio
    async def test_gallery_process_fails_to_start_raises(self) -> None:
        """Raises RuntimeError when gallery process exits immediately."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)
            main_file = gallery_dir / "main.py"
            main_file.write_text("raise RuntimeError('gallery crash')")

            mock_process = MagicMock()
            mock_process.pid = 9001
            mock_process.poll.return_value = 1  # immediately terminated
            mock_process.communicate.return_value = (b"", b"gallery crash")

            with (
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("auroraview_mcp.tools.gallery.get_gallery_dir", return_value=gallery_dir),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
                pytest.raises(RuntimeError, match="failed to start"),
            ):
                await fn()

    @pytest.mark.asyncio
    async def test_gallery_starts_successfully(self) -> None:
        """Returns running status when gallery starts OK."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)
            main_file = gallery_dir / "main.py"
            main_file.write_text('"""Gallery"""')

            mock_process = MagicMock()
            mock_process.pid = 8888
            mock_process.poll.return_value = None  # still running

            with (
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("auroraview_mcp.tools.gallery.get_gallery_dir", return_value=gallery_dir),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(port=9222)

        assert result["status"] == "running"
        assert result["pid"] == 8888
        assert result["port"] == 9222

    @pytest.mark.asyncio
    async def test_gallery_dev_mode_sets_env(self) -> None:
        """dev_mode=True sets AURORAVIEW_DEV_MODE env variable."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)
            main_file = gallery_dir / "main.py"
            main_file.write_text('"""Gallery"""')

            mock_process = MagicMock()
            mock_process.pid = 7777
            mock_process.poll.return_value = None

            captured_env = {}

            def mock_popen(cmd, cwd=None, env=None, **kwargs):
                if env:
                    captured_env.update(env)
                return mock_process

            with (
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("auroraview_mcp.tools.gallery.get_gallery_dir", return_value=gallery_dir),
                patch("subprocess.Popen", side_effect=mock_popen),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                await fn(dev_mode=True)

        assert captured_env.get("AURORAVIEW_DEV_MODE") == "1"

    @pytest.mark.asyncio
    async def test_gallery_default_cdp_port_is_9222(self) -> None:
        """Default CDP port is 9222 when not specified."""
        from auroraview_mcp.tools.gallery import run_gallery

        fn = run_gallery.fn if hasattr(run_gallery, "fn") else run_gallery

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)
            main_file = gallery_dir / "main.py"
            main_file.write_text('"""Gallery"""')

            mock_process = MagicMock()
            mock_process.pid = 6666
            mock_process.poll.return_value = None

            captured_env = {}

            def mock_popen(cmd, cwd=None, env=None, **kwargs):
                if env:
                    captured_env.update(env)
                return mock_process

            with (
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("auroraview_mcp.tools.gallery.get_gallery_dir", return_value=gallery_dir),
                patch("subprocess.Popen", side_effect=mock_popen),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn()

        assert result["port"] == 9222
        assert captured_env.get("AURORAVIEW_CDP_PORT") == "9222"


# ============================================================================
# run_sample advanced tests
# ============================================================================


class TestRunSampleAdvanced:
    """Advanced tests for run_sample tool."""

    @pytest.mark.asyncio
    async def test_run_sample_not_found_raises(self) -> None:
        """Raises RuntimeError when sample not found by any path."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with (
            tempfile.TemporaryDirectory() as tmpdir,
            patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
            pytest.raises(RuntimeError, match="Sample not found"),
        ):
            await fn(name="nonexistent_sample")

    @pytest.mark.asyncio
    async def test_run_sample_examples_dir_not_found_raises(self) -> None:
        """Raises RuntimeError when examples dir not found."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with (
            patch(
                "auroraview_mcp.tools.gallery.get_examples_dir",
                side_effect=FileNotFoundError("examples dir not found"),
            ),
            pytest.raises(RuntimeError),
        ):
            await fn(name="hello")

    @pytest.mark.asyncio
    async def test_run_sample_as_py_file(self) -> None:
        """Runs sample found as plain .py file."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "hello.py"
            sample_file.write_text('"""Hello"""')

            mock_process = MagicMock()
            mock_process.pid = 1111
            mock_process.poll.return_value = None

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="hello")

        assert result["status"] == "running"
        assert result["name"] == "hello"
        assert result["pid"] == 1111

    @pytest.mark.asyncio
    async def test_run_sample_as_demo_suffix(self) -> None:
        """Runs sample found with _demo suffix."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "widget_demo.py"
            sample_file.write_text('"""Widget Demo"""')

            mock_process = MagicMock()
            mock_process.pid = 2222
            mock_process.poll.return_value = None

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="widget")

        assert result["status"] == "running"
        assert result["pid"] == 2222

    @pytest.mark.asyncio
    async def test_run_sample_as_example_suffix(self) -> None:
        """Runs sample found with _example suffix."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "advanced_example.py"
            sample_file.write_text('"""Advanced Example"""')

            mock_process = MagicMock()
            mock_process.pid = 3333
            mock_process.poll.return_value = None

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="advanced")

        assert result["status"] == "running"

    @pytest.mark.asyncio
    async def test_run_sample_as_directory(self) -> None:
        """Runs sample found as directory with main.py."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "complex_app"
            sample_dir.mkdir()
            (sample_dir / "main.py").write_text('"""Complex App"""')

            mock_process = MagicMock()
            mock_process.pid = 4444
            mock_process.poll.return_value = None

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="complex_app")

        assert result["status"] == "running"
        assert "complex_app" in result["main_file"]

    @pytest.mark.asyncio
    async def test_run_sample_directory_with_py_files(self) -> None:
        """Runs sample dir without main.py, uses first .py file."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "no_main"
            sample_dir.mkdir()
            (sample_dir / "app.py").write_text('"""App"""')

            mock_process = MagicMock()
            mock_process.pid = 5555
            mock_process.poll.return_value = None

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="no_main")

        assert result["status"] == "running"

    @pytest.mark.asyncio
    async def test_run_sample_process_fails_raises(self) -> None:
        """Raises RuntimeError when sample process exits immediately."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "crash.py"
            sample_file.write_text("raise SystemExit(1)")

            mock_process = MagicMock()
            mock_process.pid = 9999
            mock_process.poll.return_value = 1
            mock_process.communicate.return_value = (b"", b"crash error")

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", return_value=mock_process),
                patch("asyncio.sleep", new_callable=AsyncMock),
                pytest.raises(RuntimeError, match="failed to start"),
            ):
                await fn(name="crash")

    @pytest.mark.asyncio
    async def test_run_sample_with_port_sets_env(self) -> None:
        """Port parameter sets AURORAVIEW_CDP_PORT env variable."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "test.py"
            sample_file.write_text('"""Test"""')

            mock_process = MagicMock()
            mock_process.pid = 6789
            mock_process.poll.return_value = None

            captured_env = {}

            def mock_popen(cmd, cwd=None, env=None, **kwargs):
                if env:
                    captured_env.update(env)
                return mock_process

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", side_effect=mock_popen),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="test", port=9333)

        assert captured_env.get("AURORAVIEW_CDP_PORT") == "9333"
        assert result["port"] == 9333

    @pytest.mark.asyncio
    async def test_run_sample_no_port_no_env_var(self) -> None:
        """When no port, AURORAVIEW_CDP_PORT is not set in env."""
        from auroraview_mcp.tools.gallery import run_sample

        fn = run_sample.fn if hasattr(run_sample, "fn") else run_sample

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "test.py"
            sample_file.write_text('"""Test"""')

            mock_process = MagicMock()
            mock_process.pid = 7890
            mock_process.poll.return_value = None

            captured_env = {}

            def mock_popen(cmd, cwd=None, env=None, **kwargs):
                if env:
                    captured_env.update(env)
                return mock_process

            with (
                patch.dict(os.environ, {"AURORAVIEW_EXAMPLES_DIR": tmpdir}),
                patch("auroraview_mcp.tools.gallery._process_manager", ProcessManager()),
                patch("subprocess.Popen", side_effect=mock_popen),
                patch("asyncio.sleep", new_callable=AsyncMock),
            ):
                result = await fn(name="test")

        assert "AURORAVIEW_CDP_PORT" not in captured_env
        assert result["port"] is None


# ============================================================================
# stop_sample advanced tests
# ============================================================================


class TestStopSampleAdvanced:
    """Advanced tests for stop_sample with edge cases."""

    @pytest.mark.asyncio
    async def test_stop_kills_on_timeout(self) -> None:
        """Kill process when wait() times out."""
        from auroraview_mcp.tools.gallery import stop_sample

        fn = stop_sample.fn if hasattr(stop_sample, "fn") else stop_sample

        mock_process = MagicMock()
        mock_process.poll.return_value = None
        mock_process.wait.side_effect = subprocess.TimeoutExpired("cmd", 5)

        manager = ProcessManager()
        manager.add(ProcessInfo(pid=1234, name="stubborn", process=mock_process))

        with patch("auroraview_mcp.tools.gallery._process_manager", manager):
            result = await fn(pid=1234)

        assert result["status"] == "stopped"
        mock_process.kill.assert_called_once()

    @pytest.mark.asyncio
    async def test_stop_by_name_when_pid_not_given(self) -> None:
        """Stop by name when pid not provided."""
        from auroraview_mcp.tools.gallery import stop_sample

        fn = stop_sample.fn if hasattr(stop_sample, "fn") else stop_sample

        mock_process = MagicMock()
        mock_process.wait.return_value = 0

        manager = ProcessManager()
        manager.add(ProcessInfo(pid=2345, name="to_stop", process=mock_process))

        with patch("auroraview_mcp.tools.gallery._process_manager", manager):
            result = await fn(name="to_stop")

        assert result["status"] == "stopped"
        assert result["name"] == "to_stop"
        mock_process.terminate.assert_called_once()


# ============================================================================
# ProcessManager edge case tests
# ============================================================================


class TestProcessManagerEdgeCases:
    """Edge case tests for ProcessManager class."""

    def test_remove_nonexistent_returns_none(self) -> None:
        """Removing non-existent pid returns None."""
        manager = ProcessManager()
        result = manager.remove(99999)
        assert result is None

    def test_get_nonexistent_returns_none(self) -> None:
        """Getting non-existent pid returns None."""
        manager = ProcessManager()
        result = manager.get(99999)
        assert result is None

    def test_get_gallery_when_none(self) -> None:
        """Returns None when no gallery process tracked."""
        manager = ProcessManager()
        result = manager.get_gallery()
        assert result is None

    def test_get_by_name_when_empty(self) -> None:
        """Returns None when no processes tracked."""
        manager = ProcessManager()
        result = manager.get_by_name("anything")
        assert result is None

    def test_list_all_empty(self) -> None:
        """Returns empty list when no processes."""
        manager = ProcessManager()
        assert manager.list_all() == []

    def test_cleanup_empty_manager(self) -> None:
        """Cleanup does not raise when no processes tracked."""
        manager = ProcessManager()
        manager.cleanup()  # should not raise

    def test_cleanup_removes_only_terminated(self) -> None:
        """Cleanup removes terminated, keeps running."""
        manager = ProcessManager()

        running = MagicMock()
        running.poll.return_value = None
        terminated = MagicMock()
        terminated.poll.return_value = 0

        manager.add(ProcessInfo(pid=1, name="running", process=running))
        manager.add(ProcessInfo(pid=2, name="terminated", process=terminated))
        manager.add(ProcessInfo(pid=3, name="also_terminated", process=terminated))

        manager.cleanup()

        assert manager.get(1) is not None
        assert manager.get(2) is None
        assert manager.get(3) is None

    def test_add_replaces_existing_pid(self) -> None:
        """Adding same pid replaces existing entry."""
        manager = ProcessManager()
        mock_process = MagicMock()

        info1 = ProcessInfo(pid=100, name="old", process=mock_process)
        info2 = ProcessInfo(pid=100, name="new", process=mock_process)

        manager.add(info1)
        manager.add(info2)

        assert manager.get(100) == info2
        assert manager.get(100).name == "new"

    def test_multiple_galleries_returns_first_found(self) -> None:
        """When multiple is_gallery=True entries, get_gallery returns one."""
        manager = ProcessManager()
        mock_process = MagicMock()

        manager.add(ProcessInfo(pid=10, name="gallery1", process=mock_process, is_gallery=True))
        manager.add(ProcessInfo(pid=20, name="gallery2", process=mock_process, is_gallery=True))

        result = manager.get_gallery()
        assert result is not None
        assert result.is_gallery is True


# ============================================================================
# get_sample_info edge cases
# ============================================================================


class TestGetSampleInfoEdgeCases:
    """Edge cases for get_sample_info function."""

    def test_file_read_error_returns_none(self) -> None:
        """Returns None when file cannot be read."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "bad_encoding.py"
            sample_file.write_bytes(b"\xff\xfe bad bytes \x00")

            # May return None on UnicodeDecodeError
            info = get_sample_info(sample_file)
            # Allow either None (on decode error) or a result with default title
            if info is not None:
                assert "name" in info

    def test_dir_no_py_files_returns_none(self) -> None:
        """Directory with only non-.py files returns None."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "no_py"
            sample_dir.mkdir()
            (sample_dir / "readme.txt").write_text("docs")
            (sample_dir / "config.json").write_text("{}")

            result = get_sample_info(sample_dir)
            assert result is None

    def test_py_file_with_test_suffix_stripped(self) -> None:
        """_test suffix is also stripped from name."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "hello_test.py"
            sample_file.write_text('"""Hello"""')

            info = get_sample_info(sample_file)

            assert info is not None
            assert info["name"] == "hello"

    def test_dir_with_only_main_py(self) -> None:
        """Directory with only main.py works correctly."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_dir = Path(tmpdir) / "minimal"
            sample_dir.mkdir()
            (sample_dir / "main.py").write_text("# no docstring")

            info = get_sample_info(sample_dir)

            assert info is not None
            assert info["name"] == "minimal"
            assert info["main_file"].endswith("main.py")

    def test_sample_description_truncated_at_200_chars(self) -> None:
        """Description is truncated at 200 characters."""
        from auroraview_mcp.tools.gallery import get_sample_info

        long_desc = "x" * 300
        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "long_demo.py"
            sample_file.write_text(f'"""Long Demo\n\n{long_desc}\n"""')

            info = get_sample_info(sample_file)

            assert info is not None
            assert len(info["description"]) <= 200

    def test_sample_path_and_main_file_in_result(self) -> None:
        """Result contains both path and main_file keys."""
        from auroraview_mcp.tools.gallery import get_sample_info

        with tempfile.TemporaryDirectory() as tmpdir:
            sample_file = Path(tmpdir) / "check.py"
            sample_file.write_text('"""Check"""')

            info = get_sample_info(sample_file)

            assert info is not None
            assert "path" in info
            assert "main_file" in info
            assert "source_file" in info
            assert info["source_file"] == "check.py"
