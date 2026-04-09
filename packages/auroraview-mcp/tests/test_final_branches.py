"""Final branch coverage tests for gallery, dcc, ui, connection, and discovery modules.

Covers previously untested code paths:
- gallery: get_sample_source dir-no-py, scan_samples subdir returns None,
  get_gallery_status FileNotFoundError, get_project_info FileNotFoundError,
  get_sample_info no-docstring / OSError / UnicodeDecodeError / bad-suffix removal
- dcc: all tools evaluate returns None (fallback path)
- ui: take_screenshot selector bounds=None (element not found)
- connection: Page.to_dict(), CDPConnection.disconnect when _ws=None
- discovery: _detect_dcc_type for all DCC keywords, _enrich_dcc_context branches
"""

from __future__ import annotations

import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

# ============================================================================
# gallery: get_sample_info edge cases
# ============================================================================


class TestGetSampleInfoEdgeCases:
    """Edge cases for get_sample_info."""

    def test_not_py_and_not_dir_returns_none(self, tmp_path: Path) -> None:
        """Non-.py file that is not a directory returns None."""
        from auroraview_mcp.tools.gallery import get_sample_info

        txt_file = tmp_path / "readme.txt"
        txt_file.write_text("hello")
        assert get_sample_info(txt_file) is None

    def test_py_file_no_docstring_uses_default_title(self, tmp_path: Path) -> None:
        """A .py file without a docstring uses the stem as title."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "my_tool.py"
        py_file.write_text("x = 1\n")
        info = get_sample_info(py_file)
        assert info is not None
        assert info["name"] == "my_tool"
        assert "My Tool" in info["title"]
        assert info["description"].startswith("Demo:")

    def test_py_file_demo_suffix_stripped(self, tmp_path: Path) -> None:
        """_demo suffix is removed from sample name."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "animation_demo.py"
        py_file.write_text("x = 1\n")
        info = get_sample_info(py_file)
        assert info is not None
        assert info["name"] == "animation"

    def test_py_file_example_suffix_stripped(self, tmp_path: Path) -> None:
        """_example suffix is removed from sample name."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "camera_example.py"
        py_file.write_text("x = 1\n")
        info = get_sample_info(py_file)
        assert info is not None
        assert info["name"] == "camera"

    def test_py_file_test_suffix_stripped(self, tmp_path: Path) -> None:
        """_test suffix is removed from sample name."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "render_test.py"
        py_file.write_text("x = 1\n")
        info = get_sample_info(py_file)
        assert info is not None
        assert info["name"] == "render"

    def test_dir_no_py_files_returns_none(self, tmp_path: Path) -> None:
        """Directory with no .py files returns None."""
        from auroraview_mcp.tools.gallery import get_sample_info

        subdir = tmp_path / "empty_sample"
        subdir.mkdir()
        assert get_sample_info(subdir) is None

    def test_py_file_docstring_with_dash_sep_title(self, tmp_path: Path) -> None:
        """Title extracted from 'Title - rest' docstring format."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "sample.py"
        py_file.write_text('"""My Sample - does cool things."""\n')
        info = get_sample_info(py_file)
        assert info is not None
        assert info["title"] == "My Sample"

    def test_py_file_os_error_on_read(self, tmp_path: Path) -> None:
        """OSError during read_text returns None."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "bad.py"
        py_file.write_text("x=1")
        with patch.object(Path, "read_text", side_effect=OSError("permission denied")):
            result = get_sample_info(py_file)
        assert result is None

    def test_py_file_unicode_decode_error(self, tmp_path: Path) -> None:
        """UnicodeDecodeError during read_text returns None."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "binary.py"
        py_file.write_bytes(b"\xff\xfe")
        # Write binary content that will fail utf-8 decode
        with patch.object(
            Path,
            "read_text",
            side_effect=UnicodeDecodeError("utf-8", b"", 0, 1, "invalid byte"),
        ):
            result = get_sample_info(py_file)
        assert result is None

    def test_py_file_docstring_multiple_desc_lines(self, tmp_path: Path) -> None:
        """Description extracted from multiple docstring lines (up to 2)."""
        from auroraview_mcp.tools.gallery import get_sample_info

        py_file = tmp_path / "rich.py"
        py_file.write_text(
            '"""Rich Sample\n\nThis is first desc line.\nThis is second desc line.\nExtra line.\n"""\n'
        )
        info = get_sample_info(py_file)
        assert info is not None
        assert "first desc" in info["description"]

    def test_dir_with_main_py_uses_main(self, tmp_path: Path) -> None:
        """Directory with main.py uses it as main file."""
        from auroraview_mcp.tools.gallery import get_sample_info

        subdir = tmp_path / "my_project"
        subdir.mkdir()
        main = subdir / "main.py"
        main.write_text('"""Main Sample"""\n')
        info = get_sample_info(subdir)
        assert info is not None
        assert info["name"] == "my_project"
        assert info["source_file"] == "main.py"


# ============================================================================
# gallery: scan_samples — subdir returns None
# ============================================================================


class TestScanSamplesSubdirNone:
    """scan_samples skips subdirs where get_sample_info returns None."""

    def test_scan_samples_skips_none_subdir(self, tmp_path: Path) -> None:
        """Subdirectory with no .py files is skipped."""
        from auroraview_mcp.tools.gallery import scan_samples

        # Create a subdir with no .py files
        empty_subdir = tmp_path / "empty_sub"
        empty_subdir.mkdir()
        # Also a valid .py file at top level
        (tmp_path / "valid.py").write_text("x=1\n")

        samples = scan_samples(tmp_path)
        # Only the valid.py should appear; empty_sub is skipped
        names = [s["name"] for s in samples]
        assert "valid" in names
        assert "empty_sub" not in names

    def test_scan_samples_nonexistent_dir_returns_empty(self, tmp_path: Path) -> None:
        """Non-existent directory returns empty list."""
        from auroraview_mcp.tools.gallery import scan_samples

        result = scan_samples(tmp_path / "does_not_exist")
        assert result == []

    def test_scan_samples_skips_dunder_py_files(self, tmp_path: Path) -> None:
        """Files starting with __ are skipped."""
        from auroraview_mcp.tools.gallery import scan_samples

        (tmp_path / "__init__.py").write_text("")
        (tmp_path / "__main__.py").write_text("")
        (tmp_path / "valid.py").write_text("x=1\n")

        samples = scan_samples(tmp_path)
        names = [s["name"] for s in samples]
        assert "__init__" not in names
        assert "__main__" not in names
        assert "valid" in names

    def test_scan_samples_skips_hidden_subdirs(self, tmp_path: Path) -> None:
        """Subdirectories starting with _ or . are skipped."""
        from auroraview_mcp.tools.gallery import scan_samples

        hidden = tmp_path / "_hidden"
        hidden.mkdir()
        (hidden / "main.py").write_text("x=1\n")

        dot_dir = tmp_path / ".dotdir"
        dot_dir.mkdir()
        (dot_dir / "main.py").write_text("x=1\n")

        samples = scan_samples(tmp_path)
        names = [s["name"] for s in samples]
        assert "_hidden" not in names
        assert ".dotdir" not in names


# ============================================================================
# gallery: get_gallery_status FileNotFoundError
# ============================================================================


class TestGetGalleryStatusFileNotFound:
    """get_gallery_status returns error dict when gallery dir not found."""

    @pytest.mark.asyncio
    async def test_gallery_status_dir_not_found(self) -> None:
        """Returns error dict when get_gallery_dir raises FileNotFoundError."""
        import auroraview_mcp.tools.gallery as gallery_mod

        with patch.object(
            gallery_mod,
            "get_gallery_dir",
            side_effect=FileNotFoundError("Gallery dir not found"),
        ):
            fn = (
                gallery_mod.get_gallery_status.fn
                if hasattr(gallery_mod.get_gallery_status, "fn")
                else gallery_mod.get_gallery_status
            )
            result = await fn()

        assert result["running"] is False
        assert "error" in result

    @pytest.mark.asyncio
    async def test_gallery_status_running_process(self) -> None:
        """Returns running=True when gallery process is alive."""
        import auroraview_mcp.tools.gallery as gallery_mod

        mock_process = MagicMock()
        mock_process.poll.return_value = None  # still running

        from auroraview_mcp.tools.gallery import ProcessInfo, ProcessManager

        pm = ProcessManager()
        proc_info = ProcessInfo(
            pid=1234, name="gallery", process=mock_process, port=9222, is_gallery=True
        )
        pm.add(proc_info)

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_path = Path(tmpdir)
            with (
                patch.object(gallery_mod, "get_gallery_dir", return_value=gallery_path),
                patch.object(gallery_mod, "_process_manager", pm),
            ):
                fn = (
                    gallery_mod.get_gallery_status.fn
                    if hasattr(gallery_mod.get_gallery_status, "fn")
                    else gallery_mod.get_gallery_status
                )
                result = await fn()

        assert result["running"] is True
        assert result["pid"] == 1234
        assert result["port"] == 9222

    @pytest.mark.asyncio
    async def test_gallery_status_terminated_process_cleaned_up(self) -> None:
        """Terminated gallery process is cleaned up; returns running=False."""
        import auroraview_mcp.tools.gallery as gallery_mod

        mock_process = MagicMock()
        mock_process.poll.return_value = 1  # terminated

        from auroraview_mcp.tools.gallery import ProcessInfo, ProcessManager

        pm = ProcessManager()
        proc_info = ProcessInfo(
            pid=9999, name="gallery", process=mock_process, port=9222, is_gallery=True
        )
        pm.add(proc_info)

        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_path = Path(tmpdir)
            with (
                patch.object(gallery_mod, "get_gallery_dir", return_value=gallery_path),
                patch.object(gallery_mod, "_process_manager", pm),
            ):
                fn = (
                    gallery_mod.get_gallery_status.fn
                    if hasattr(gallery_mod.get_gallery_status, "fn")
                    else gallery_mod.get_gallery_status
                )
                result = await fn()

        assert result["running"] is False
        # Process should have been removed
        assert pm.get(9999) is None


# ============================================================================
# gallery: get_project_info FileNotFoundError
# ============================================================================


class TestGetProjectInfoFileNotFound:
    """get_project_info returns error dict when project root not found."""

    @pytest.mark.asyncio
    async def test_project_info_root_not_found(self) -> None:
        """Returns error dict when get_project_root raises FileNotFoundError."""
        import auroraview_mcp.tools.gallery as gallery_mod

        with patch.object(
            gallery_mod,
            "get_project_root",
            side_effect=FileNotFoundError("No project root"),
        ):
            fn = (
                gallery_mod.get_project_info.fn
                if hasattr(gallery_mod.get_project_info, "fn")
                else gallery_mod.get_project_info
            )
            result = await fn()

        assert "error" in result

    @pytest.mark.asyncio
    async def test_project_info_success(self, tmp_path: Path) -> None:
        """Returns project info with sample_count when dirs exist."""
        import auroraview_mcp.tools.gallery as gallery_mod

        gallery_dir = tmp_path / "gallery"
        gallery_dir.mkdir()
        examples_dir = tmp_path / "examples"
        examples_dir.mkdir()
        (examples_dir / "demo.py").write_text("x=1\n")

        with (
            patch.object(gallery_mod, "get_project_root", return_value=tmp_path),
            patch.object(gallery_mod, "get_gallery_dir", return_value=gallery_dir),
            patch.object(gallery_mod, "get_examples_dir", return_value=examples_dir),
        ):
            fn = (
                gallery_mod.get_project_info.fn
                if hasattr(gallery_mod.get_project_info, "fn")
                else gallery_mod.get_project_info
            )
            result = await fn()

        assert "project_root" in result
        assert result["sample_count"] >= 1


# ============================================================================
# gallery: get_sample_source — dir with no main.py and no py files
# ============================================================================


class TestGetSampleSourceEdgeCases:
    """Edge cases for get_sample_source."""

    @pytest.mark.asyncio
    async def test_sample_source_dir_no_py_files(self, tmp_path: Path) -> None:
        """Dir found but no .py files → raises RuntimeError."""
        import auroraview_mcp.tools.gallery as gallery_mod

        subdir = tmp_path / "my_sample"
        subdir.mkdir()
        # No .py files in subdir

        with patch.object(gallery_mod, "get_examples_dir", return_value=tmp_path):
            fn = (
                gallery_mod.get_sample_source.fn
                if hasattr(gallery_mod.get_sample_source, "fn")
                else gallery_mod.get_sample_source
            )
            with pytest.raises(RuntimeError, match="Sample not found"):
                await fn("my_sample")

    @pytest.mark.asyncio
    async def test_sample_source_dir_uses_first_py_when_no_main(self, tmp_path: Path) -> None:
        """Dir without main.py uses first .py file."""
        import auroraview_mcp.tools.gallery as gallery_mod

        subdir = tmp_path / "sub_sample"
        subdir.mkdir()
        other_py = subdir / "other.py"
        other_py.write_text("# code\n")

        with patch.object(gallery_mod, "get_examples_dir", return_value=tmp_path):
            fn = (
                gallery_mod.get_sample_source.fn
                if hasattr(gallery_mod.get_sample_source, "fn")
                else gallery_mod.get_sample_source
            )
            result = await fn("sub_sample")

        assert "# code" in result

    @pytest.mark.asyncio
    async def test_sample_source_not_found_raises(self, tmp_path: Path) -> None:
        """Non-existent sample raises RuntimeError."""
        import auroraview_mcp.tools.gallery as gallery_mod

        with patch.object(gallery_mod, "get_examples_dir", return_value=tmp_path):
            fn = (
                gallery_mod.get_sample_source.fn
                if hasattr(gallery_mod.get_sample_source, "fn")
                else gallery_mod.get_sample_source
            )
            with pytest.raises(RuntimeError, match="Sample not found"):
                await fn("nonexistent_sample")

    @pytest.mark.asyncio
    async def test_sample_source_demo_suffix_found(self, tmp_path: Path) -> None:
        """Sample with _demo suffix is found."""
        import auroraview_mcp.tools.gallery as gallery_mod

        (tmp_path / "hello_demo.py").write_text("# hello demo\n")

        with patch.object(gallery_mod, "get_examples_dir", return_value=tmp_path):
            fn = (
                gallery_mod.get_sample_source.fn
                if hasattr(gallery_mod.get_sample_source, "fn")
                else gallery_mod.get_sample_source
            )
            result = await fn("hello")

        assert "hello demo" in result


# ============================================================================
# dcc: all tools evaluate returns None (fallback)
# ============================================================================


def _make_dcc_manager(evaluate_return=None):
    """Build a mock manager where get_page_connection evaluates to given value."""
    page_conn = MagicMock()
    page_conn.evaluate = AsyncMock(return_value=evaluate_return)
    manager = MagicMock()
    manager.get_page_connection = AsyncMock(return_value=page_conn)
    return manager


class TestDCCToolsNoneFallback:
    """Test fallback returns when page_conn.evaluate() returns None."""

    @pytest.mark.asyncio
    async def test_get_dcc_context_none_returns_fallback(self) -> None:
        """get_dcc_context falls back to error dict when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import get_dcc_context

            fn = get_dcc_context.fn if hasattr(get_dcc_context, "fn") else get_dcc_context
            result = await fn()

        assert "error" in result
        assert result["dcc_type"] is None

    @pytest.mark.asyncio
    async def test_execute_dcc_command_none_returns_fallback(self) -> None:
        """execute_dcc_command falls back when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import execute_dcc_command

            fn = (
                execute_dcc_command.fn
                if hasattr(execute_dcc_command, "fn")
                else execute_dcc_command
            )
            result = await fn("maya.cmds.ls")

        assert result["success"] is False
        assert "error" in result

    @pytest.mark.asyncio
    async def test_sync_selection_none_returns_fallback(self) -> None:
        """sync_selection falls back when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import sync_selection

            fn = sync_selection.fn if hasattr(sync_selection, "fn") else sync_selection
            result = await fn()

        assert result["synced"] is False
        assert "error" in result

    @pytest.mark.asyncio
    async def test_set_dcc_selection_none_returns_fallback(self) -> None:
        """set_dcc_selection falls back when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import set_dcc_selection

            fn = set_dcc_selection.fn if hasattr(set_dcc_selection, "fn") else set_dcc_selection
            result = await fn(["pCube1"])

        assert result["success"] is False
        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_dcc_scene_info_none_returns_fallback(self) -> None:
        """get_dcc_scene_info falls back when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import get_dcc_scene_info

            fn = get_dcc_scene_info.fn if hasattr(get_dcc_scene_info, "fn") else get_dcc_scene_info
            result = await fn()

        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_dcc_timeline_none_returns_fallback(self) -> None:
        """get_dcc_timeline falls back when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import get_dcc_timeline

            fn = get_dcc_timeline.fn if hasattr(get_dcc_timeline, "fn") else get_dcc_timeline
            result = await fn()

        assert "error" in result

    @pytest.mark.asyncio
    async def test_set_dcc_frame_none_returns_fallback(self) -> None:
        """set_dcc_frame falls back when evaluate returns None."""
        manager = _make_dcc_manager(evaluate_return=None)
        with patch("auroraview_mcp.tools.dcc.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.dcc import set_dcc_frame

            fn = set_dcc_frame.fn if hasattr(set_dcc_frame, "fn") else set_dcc_frame
            result = await fn(42)

        assert result["success"] is False
        assert "error" in result


# ============================================================================
# ui: take_screenshot selector bounds=None
# ============================================================================


class TestTakeScreenshotSelectorNotFound:
    """take_screenshot with selector that returns no bounds."""

    @pytest.mark.asyncio
    async def test_selector_element_not_found_skips_clip(self) -> None:
        """When element not found (bounds=None), screenshot proceeds without clip."""
        import base64

        page_conn = MagicMock()
        png_data = base64.b64encode(b"fake-png").decode()
        # First call (evaluate for bounds) returns None, second call (send) returns data
        page_conn.evaluate = AsyncMock(return_value=None)
        page_conn.send = AsyncMock(return_value={"data": png_data})

        manager = MagicMock()
        manager.is_connected = True
        manager.current_page = MagicMock()
        manager.get_page_connection = AsyncMock(return_value=page_conn)

        with patch("auroraview_mcp.tools.ui.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.ui import take_screenshot

            fn = take_screenshot.fn if hasattr(take_screenshot, "fn") else take_screenshot
            result = await fn(selector="#nonexistent")

        # Should have called send without clip param
        call_args = page_conn.send.call_args
        params = (
            call_args[0][1]
            if len(call_args[0]) > 1
            else call_args[1].get("params", call_args[0][1] if call_args[0] else {})
        )
        assert "clip" not in params
        assert result.startswith("data:image/png;base64,")


# ============================================================================
# connection: Page.to_dict() and CDPConnection.disconnect no-ws
# ============================================================================


class TestPageToDict:
    """Test Page.to_dict() returns all expected fields."""

    def test_page_to_dict_all_fields(self) -> None:
        """to_dict() includes all fields with correct values."""
        from auroraview_mcp.connection import Page

        page = Page(
            id="abc123",
            url="http://localhost:3000",
            title="My App",
            ws_url="ws://localhost:9222/devtools/page/abc123",
            type="page",
        )
        d = page.to_dict()
        assert d["id"] == "abc123"
        assert d["url"] == "http://localhost:3000"
        assert d["title"] == "My App"
        assert d["ws_url"] == "ws://localhost:9222/devtools/page/abc123"
        assert d["type"] == "page"

    def test_page_to_dict_default_type(self) -> None:
        """Default type is 'page'."""
        from auroraview_mcp.connection import Page

        page = Page(id="x", url="http://a.com", title="A", ws_url="ws://a")
        d = page.to_dict()
        assert d["type"] == "page"


class TestCDPConnectionDisconnectNoWs:
    """CDPConnection.disconnect() when _ws is None is a no-op."""

    @pytest.mark.asyncio
    async def test_disconnect_no_ws_is_noop(self) -> None:
        """disconnect() when _ws is None does not raise."""
        from auroraview_mcp.connection import CDPConnection

        conn = CDPConnection(port=9222, ws_url="ws://127.0.0.1:9222")
        conn._ws = None
        # Should not raise
        await conn.disconnect()
        assert conn._ws is None

    @pytest.mark.asyncio
    async def test_cdp_is_connected_false_when_ws_none(self) -> None:
        """is_connected is False when _ws is None."""
        from auroraview_mcp.connection import CDPConnection

        conn = CDPConnection(port=9222, ws_url="ws://127.0.0.1:9222")
        assert conn.is_connected is False


class TestPageConnectionDisconnectNoWs:
    """PageConnection.disconnect() when _ws is None is a no-op."""

    @pytest.mark.asyncio
    async def test_disconnect_no_ws_is_noop(self) -> None:
        """disconnect() when _ws is None does not raise."""
        from auroraview_mcp.connection import Page, PageConnection

        page = Page(id="p1", url="http://x.com", title="X", ws_url="ws://x")
        conn = PageConnection(page=page)
        conn._ws = None
        await conn.disconnect()
        assert conn._ws is None

    @pytest.mark.asyncio
    async def test_page_connection_is_connected_false_when_none(self) -> None:
        """is_connected is False when _ws is None."""
        from auroraview_mcp.connection import Page, PageConnection

        page = Page(id="p1", url="http://x.com", title="X", ws_url="ws://x")
        conn = PageConnection(page=page)
        assert conn.is_connected is False


# ============================================================================
# discovery: _detect_dcc_type for all keywords
# ============================================================================


class TestDetectDCCTypeKeywords:
    """_detect_dcc_type covers all DCC keyword groups."""

    def _detect(self, title: str = "", url: str = "") -> str | None:
        from auroraview_mcp.discovery import InstanceDiscovery

        d = InstanceDiscovery()
        return d._detect_dcc_type(title, url)

    def test_maya_from_title(self) -> None:
        assert self._detect(title="Autodesk Maya 2025") == "maya"

    def test_maya_from_url(self) -> None:
        assert self._detect(url="http://maya-tool.local") == "maya"

    def test_blender_from_title(self) -> None:
        assert self._detect(title="Blender 4.0") == "blender"

    def test_houdini_from_title(self) -> None:
        assert self._detect(title="Houdini 20") == "houdini"

    def test_sidefx_from_url(self) -> None:
        assert self._detect(url="http://sidefx.panel") == "houdini"

    def test_nuke_from_title(self) -> None:
        assert self._detect(title="Nuke Studio") == "nuke"

    def test_foundry_from_url(self) -> None:
        assert self._detect(url="http://foundry.local") == "nuke"

    def test_unreal_from_title(self) -> None:
        assert self._detect(title="Unreal Engine 5") == "unreal"

    def test_ue5_from_url(self) -> None:
        assert self._detect(url="http://ue5.panel") == "unreal"

    def test_3dsmax_from_title(self) -> None:
        assert self._detect(title="Autodesk 3ds Max 2025") == "3dsmax"

    def test_3dsmax_from_url(self) -> None:
        assert self._detect(url="http://3dsmax-panel.local") == "3dsmax"

    def test_unknown_returns_none(self) -> None:
        assert self._detect(title="Unknown Tool", url="http://unknown.local") is None

    def test_empty_returns_none(self) -> None:
        assert self._detect() is None


# ============================================================================
# discovery: _enrich_dcc_context edge cases
# ============================================================================


class TestEnrichDCCContextBranches:
    """_enrich_dcc_context paths: non-200, no pages, dcc detected, exception."""

    @pytest.mark.asyncio
    async def test_enrich_returns_same_instance_on_non_200(self) -> None:
        """Non-200 response from /json/list leaves instance unchanged."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type=None)
        discovery = InstanceDiscovery()

        mock_resp = MagicMock()
        mock_resp.status_code = 404

        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._enrich_dcc_context(inst)

        assert result.dcc_type is None

    @pytest.mark.asyncio
    async def test_enrich_detects_dcc_from_page_title(self) -> None:
        """200 response with maya page title sets dcc_type."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type=None)
        discovery = InstanceDiscovery()

        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = [{"title": "Autodesk Maya 2025", "url": "http://localhost"}]

        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._enrich_dcc_context(inst)

        assert result.dcc_type == "maya"

    @pytest.mark.asyncio
    async def test_enrich_stops_after_first_match(self) -> None:
        """Stops enriching after first DCC match is found."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type=None)
        discovery = InstanceDiscovery()

        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = [
            {"title": "Blender 4.0", "url": "http://localhost"},
            {"title": "Houdini 20", "url": "http://localhost2"},
        ]

        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._enrich_dcc_context(inst)

        # Only first match should be set
        assert result.dcc_type == "blender"

    @pytest.mark.asyncio
    async def test_enrich_exception_leaves_instance_unchanged(self) -> None:
        """Exception during enrichment leaves instance unchanged."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type=None)
        discovery = InstanceDiscovery()

        with patch(
            "auroraview_mcp.discovery.httpx.AsyncClient", side_effect=Exception("network error")
        ):
            result = await discovery._enrich_dcc_context(inst)

        assert result.dcc_type is None

    @pytest.mark.asyncio
    async def test_enrich_no_matching_page_leaves_none(self) -> None:
        """Pages with no DCC keywords leaves dcc_type as None."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type=None)
        discovery = InstanceDiscovery()

        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = [{"title": "Generic Web App", "url": "http://generic.local"}]

        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("auroraview_mcp.discovery.httpx.AsyncClient", return_value=mock_client):
            result = await discovery._enrich_dcc_context(inst)

        assert result.dcc_type is None


# ============================================================================
# discovery: discover_dcc_instances enrichment path
# ============================================================================


class TestDiscoverDCCInstancesEnrichment:
    """discover_dcc_instances enriches instances that lack dcc_type."""

    @pytest.mark.asyncio
    async def test_instances_with_dcc_type_not_enriched(self) -> None:
        """Instances that already have dcc_type skip enrichment."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type="maya")
        discovery = InstanceDiscovery()

        with (
            patch.object(discovery, "discover", return_value=[inst]),
            patch.object(discovery, "_enrich_dcc_context") as mock_enrich,
        ):
            result = await discovery.discover_dcc_instances()

        mock_enrich.assert_not_called()
        assert result[0].dcc_type == "maya"

    @pytest.mark.asyncio
    async def test_instances_without_dcc_type_enriched(self) -> None:
        """Instances without dcc_type are enriched."""
        from auroraview_mcp.discovery import Instance, InstanceDiscovery

        inst = Instance(port=9222, dcc_type=None)
        enriched_inst = Instance(port=9222, dcc_type="houdini")
        discovery = InstanceDiscovery()

        with (
            patch.object(discovery, "discover", return_value=[inst]),
            patch.object(
                discovery, "_enrich_dcc_context", return_value=enriched_inst
            ) as mock_enrich,
        ):
            result = await discovery.discover_dcc_instances()

        mock_enrich.assert_called_once_with(inst)
        assert result[0].dcc_type == "houdini"
