"""Tests for MCP tools.

Note: MCP tools are decorated with @mcp.tool() which wraps them in FunctionTool
objects. These tests focus on the underlying logic and helper functions rather
than directly calling the decorated tools.
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

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


class TestDiscoveryModule:
    """Tests for discovery module functionality."""

    @pytest.mark.asyncio
    async def test_instance_discovery_class(self) -> None:
        """Test InstanceDiscovery class."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()
        assert discovery is not None
        assert hasattr(discovery, "discover")

    @pytest.mark.asyncio
    async def test_instance_discovery_with_no_instances(self) -> None:
        """Test discovery when no instances are running."""
        from auroraview_mcp.discovery import InstanceDiscovery

        discovery = InstanceDiscovery()

        # Mock httpx to return connection error (no instance)
        with patch("auroraview_mcp.discovery.httpx.AsyncClient") as mock_client:
            mock_instance = MagicMock()
            mock_instance.__aenter__ = AsyncMock(return_value=mock_instance)
            mock_instance.__aexit__ = AsyncMock(return_value=None)
            mock_instance.get = AsyncMock(side_effect=Exception("Connection refused"))
            mock_client.return_value = mock_instance

            instances = await discovery.discover([9999])
            assert instances == []


class TestConnectionModule:
    """Tests for connection module functionality."""

    def test_page_dataclass(self) -> None:
        """Test Page dataclass."""
        from auroraview_mcp.connection import Page

        page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test Page",
            ws_url="ws://localhost:9222/devtools/page/ABC123",
        )

        assert page.id == "ABC123"
        assert page.url == "http://localhost:8080"
        assert page.title == "Test Page"

    def test_page_to_dict(self) -> None:
        """Test Page.to_dict() method."""
        from auroraview_mcp.connection import Page

        page = Page(
            id="ABC123",
            url="http://localhost:8080",
            title="Test Page",
            ws_url="ws://localhost:9222/devtools/page/ABC123",
        )

        d = page.to_dict()
        assert d["id"] == "ABC123"
        assert d["url"] == "http://localhost:8080"
        assert d["title"] == "Test Page"

    def test_connection_manager_initial_state(self) -> None:
        """Test ConnectionManager initial state."""
        from auroraview_mcp.connection import ConnectionManager

        manager = ConnectionManager()
        assert manager.is_connected is False
        assert manager.current_page is None
        assert manager.current_port is None


class TestServerModule:
    """Tests for server module functionality."""

    def test_mcp_instance_exists(self) -> None:
        """Test that MCP server instance exists."""
        from auroraview_mcp.server import mcp

        assert mcp is not None
        assert mcp.name == "auroraview"

    def test_get_discovery_function(self) -> None:
        """Test get_discovery helper function."""
        from auroraview_mcp.discovery import InstanceDiscovery
        from auroraview_mcp.server import get_discovery

        discovery = get_discovery()
        assert isinstance(discovery, InstanceDiscovery)

    def test_get_connection_manager_function(self) -> None:
        """Test get_connection_manager helper function."""
        from auroraview_mcp.connection import ConnectionManager
        from auroraview_mcp.server import get_connection_manager

        manager = get_connection_manager()
        assert isinstance(manager, ConnectionManager)


class TestToolRegistration:
    """Tests for tool registration."""

    def test_discovery_tools_registered(self) -> None:
        """Test that discovery tools are registered."""

        # Check tools are registered (they become FunctionTool objects)
        from auroraview_mcp.tools import discovery

        assert hasattr(discovery, "discover_instances")
        assert hasattr(discovery, "connect")
        assert hasattr(discovery, "disconnect")

    def test_page_tools_registered(self) -> None:
        """Test that page tools are registered."""
        from auroraview_mcp.tools import page

        assert hasattr(page, "list_pages")
        assert hasattr(page, "select_page")
        assert hasattr(page, "get_page_info")

    def test_api_tools_registered(self) -> None:
        """Test that API tools are registered."""
        from auroraview_mcp.tools import api

        assert hasattr(api, "call_api")
        assert hasattr(api, "list_api_methods")
        assert hasattr(api, "emit_event")

    def test_ui_tools_registered(self) -> None:
        """Test that UI tools are registered."""
        from auroraview_mcp.tools import ui

        assert hasattr(ui, "take_screenshot")
        assert hasattr(ui, "click")
        assert hasattr(ui, "fill")
        assert hasattr(ui, "evaluate")

    def test_gallery_tools_registered(self) -> None:
        """Test that gallery tools are registered."""
        from auroraview_mcp.tools import gallery

        assert hasattr(gallery, "run_gallery")
        assert hasattr(gallery, "stop_gallery")
        assert hasattr(gallery, "get_gallery_status")
        assert hasattr(gallery, "get_samples")
        assert hasattr(gallery, "run_sample")
        assert hasattr(gallery, "stop_sample")

    def test_debug_tools_registered(self) -> None:
        """Test that debug tools are registered."""
        from auroraview_mcp.tools import debug

        assert hasattr(debug, "get_console_logs")
        assert hasattr(debug, "get_backend_status")


class TestHelperFunctions:
    """Tests for helper functions in tools modules."""

    def test_gallery_path_functions(self) -> None:
        """Test gallery path helper functions."""
        from auroraview_mcp.tools.gallery import (
            get_examples_dir,
            get_gallery_dir,
            get_project_root,
        )

        # Test with environment variables
        with (
            tempfile.TemporaryDirectory() as tmpdir,
            patch.dict(
                os.environ,
                {
                    "AURORAVIEW_PROJECT_ROOT": tmpdir,
                    "AURORAVIEW_GALLERY_DIR": tmpdir,
                    "AURORAVIEW_EXAMPLES_DIR": tmpdir,
                },
            ),
        ):
            assert get_project_root() == Path(tmpdir)
            assert get_gallery_dir() == Path(tmpdir)
            assert get_examples_dir() == Path(tmpdir)

    def test_scan_samples_function(self) -> None:
        """Test scan_samples function."""
        from auroraview_mcp.tools.gallery import scan_samples

        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            # Create test samples
            (examples_dir / "demo1.py").write_text('"""Demo 1"""\nprint(1)')
            (examples_dir / "demo2.py").write_text('"""Demo 2"""\nprint(2)')

            samples = scan_samples(examples_dir)

            assert len(samples) == 2
            names = [s["name"] for s in samples]
            assert "demo1" in names
            assert "demo2" in names
