"""Tests for MCP tools."""

from __future__ import annotations

import pytest
from unittest.mock import AsyncMock, MagicMock, patch
from pathlib import Path
import tempfile
import os

from auroraview_mcp.tools.gallery import (
    ProcessInfo,
    ProcessManager,
    get_sample_info,
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

    def test_sample_with_main_py(self) -> None:
        """Test sample with main.py file."""
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


class TestDiscoveryTools:
    """Tests for discovery tools."""

    @pytest.mark.asyncio
    async def test_discover_instances_empty(self) -> None:
        """Test discover_instances with no instances."""
        from auroraview_mcp.tools.discovery import discover_instances

        with patch("auroraview_mcp.server._discovery") as mock_discovery:
            mock_discovery.discover = AsyncMock(return_value=[])

            result = await discover_instances()
            assert result == []

    @pytest.mark.asyncio
    async def test_connect_success(self) -> None:
        """Test successful connection."""
        from auroraview_mcp.tools.discovery import connect
        from auroraview_mcp.connection import Page

        mock_page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test",
            ws_url="ws://test",
        )

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.connect = AsyncMock()
            mock_manager.get_pages = AsyncMock(return_value=[mock_page])
            mock_manager.select_page = AsyncMock(return_value=mock_page)
            mock_manager.current_page = mock_page

            result = await connect(9222)

            assert result["status"] == "connected"
            assert result["port"] == 9222
            assert len(result["pages"]) == 1


class TestPageTools:
    """Tests for page tools."""

    @pytest.mark.asyncio
    async def test_list_pages_not_connected(self) -> None:
        """Test list_pages when not connected."""
        from auroraview_mcp.tools.page import list_pages

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = False

            with pytest.raises(RuntimeError, match="Not connected"):
                await list_pages()

    @pytest.mark.asyncio
    async def test_get_page_info_no_page(self) -> None:
        """Test get_page_info with no page selected."""
        from auroraview_mcp.tools.page import get_page_info

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = True
            mock_manager.current_page = None

            with pytest.raises(RuntimeError, match="No page selected"):
                await get_page_info()


class TestAPITools:
    """Tests for API tools."""

    @pytest.mark.asyncio
    async def test_call_api_not_connected(self) -> None:
        """Test call_api when not connected."""
        from auroraview_mcp.tools.api import call_api

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = False

            with pytest.raises(RuntimeError, match="Not connected"):
                await call_api("api.test")

    @pytest.mark.asyncio
    async def test_emit_event_not_connected(self) -> None:
        """Test emit_event when not connected."""
        from auroraview_mcp.tools.api import emit_event

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = False

            with pytest.raises(RuntimeError, match="Not connected"):
                await emit_event("test_event")


class TestUITools:
    """Tests for UI tools."""

    @pytest.mark.asyncio
    async def test_click_no_selector_or_uid(self) -> None:
        """Test click without selector or uid."""
        from auroraview_mcp.tools.ui import click

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = True
            mock_manager.current_page = MagicMock()

            with pytest.raises(ValueError, match="Either selector or uid"):
                await click()

    @pytest.mark.asyncio
    async def test_fill_not_connected(self) -> None:
        """Test fill when not connected."""
        from auroraview_mcp.tools.ui import fill

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = False

            with pytest.raises(RuntimeError, match="Not connected"):
                await fill("input", "test")


class TestDebugTools:
    """Tests for debug tools."""

    @pytest.mark.asyncio
    async def test_get_console_logs_not_connected(self) -> None:
        """Test get_console_logs when not connected."""
        from auroraview_mcp.tools.debug import get_console_logs

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = False

            with pytest.raises(RuntimeError, match="Not connected"):
                await get_console_logs()

    @pytest.mark.asyncio
    async def test_get_backend_status_not_connected(self) -> None:
        """Test get_backend_status when not connected."""
        from auroraview_mcp.tools.debug import get_backend_status

        with patch("auroraview_mcp.server._connection_manager") as mock_manager:
            mock_manager.is_connected = False

            with pytest.raises(RuntimeError, match="Not connected"):
                await get_backend_status()
