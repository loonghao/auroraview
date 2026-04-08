"""Tests for MCP resource providers.

Tests the resource provider functions in auroraview_mcp.resources.providers.
"""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.discovery import Instance


class TestGetInstancesResource:
    """Tests for get_instances_resource."""

    @pytest.mark.asyncio
    async def test_returns_empty_list_when_no_instances(self) -> None:
        """Test resource returns empty JSON array when no instances found."""
        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[])

        with (
            patch("auroraview_mcp.resources.providers.get_discovery", return_value=mock_discovery),
        ):
            from auroraview_mcp.resources.providers import get_instances_resource

            fn = (
                get_instances_resource.fn
                if hasattr(get_instances_resource, "fn")
                else get_instances_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data == []

    @pytest.mark.asyncio
    async def test_returns_instances_as_json(self) -> None:
        """Test resource returns instances serialized as JSON."""
        inst = Instance(
            port=9222,
            browser="Chrome/120.0",
            ws_url="ws://localhost:9222/devtools/browser/xxx",
            dcc_type="maya",
            title="Maya 2025",
        )

        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[inst])

        with patch("auroraview_mcp.resources.providers.get_discovery", return_value=mock_discovery):
            from auroraview_mcp.resources.providers import get_instances_resource

            fn = (
                get_instances_resource.fn
                if hasattr(get_instances_resource, "fn")
                else get_instances_resource
            )
            result = await fn()

        data = json.loads(result)
        assert len(data) == 1
        assert data[0]["port"] == 9222
        assert data[0]["dcc_type"] == "maya"
        assert data[0]["title"] == "Maya 2025"

    @pytest.mark.asyncio
    async def test_returns_multiple_instances(self) -> None:
        """Test resource with multiple instances."""
        instances = [
            Instance(port=9222, dcc_type="maya"),
            Instance(port=9223, dcc_type="blender"),
            Instance(port=9224, dcc_type="houdini"),
        ]

        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=instances)

        with patch("auroraview_mcp.resources.providers.get_discovery", return_value=mock_discovery):
            from auroraview_mcp.resources.providers import get_instances_resource

            fn = (
                get_instances_resource.fn
                if hasattr(get_instances_resource, "fn")
                else get_instances_resource
            )
            result = await fn()

        data = json.loads(result)
        assert len(data) == 3
        ports = [d["port"] for d in data]
        assert 9222 in ports
        assert 9223 in ports
        assert 9224 in ports


class TestGetPageResource:
    """Tests for get_page_resource."""

    @pytest.mark.asyncio
    async def test_not_connected_returns_error(self) -> None:
        """Test resource returns error when not connected."""
        mock_manager = MagicMock()
        mock_manager.is_connected = False

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = (
                get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            )
            result = await fn(page_id="page1")

        data = json.loads(result)
        assert "error" in data
        assert data["error"] == "Not connected"

    @pytest.mark.asyncio
    async def test_page_found_returns_page_data(self) -> None:
        """Test resource returns page data when page found."""
        from auroraview_mcp.connection import Page

        page = Page(
            id="page1",
            url="http://localhost:8080",
            title="Test Page",
            ws_url="ws://localhost:9222/devtools/page/page1",
        )

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.get_pages = AsyncMock(return_value=[page])

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = (
                get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            )
            result = await fn(page_id="page1")

        data = json.loads(result)
        assert data["id"] == "page1"
        assert data["url"] == "http://localhost:8080"
        assert data["title"] == "Test Page"

    @pytest.mark.asyncio
    async def test_page_not_found_returns_error(self) -> None:
        """Test resource returns error when page not found."""
        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.get_pages = AsyncMock(return_value=[])

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = (
                get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            )
            result = await fn(page_id="nonexistent")

        data = json.loads(result)
        assert "error" in data
        assert "nonexistent" in data["error"]

    @pytest.mark.asyncio
    async def test_page_found_among_multiple(self) -> None:
        """Test finding correct page when multiple pages exist."""
        from auroraview_mcp.connection import Page

        pages = [
            Page(id="p1", url="http://localhost:8080", title="Page 1", ws_url="ws://localhost/p1"),
            Page(id="p2", url="http://localhost:8081", title="Page 2", ws_url="ws://localhost/p2"),
            Page(id="p3", url="http://localhost:8082", title="Page 3", ws_url="ws://localhost/p3"),
        ]

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.get_pages = AsyncMock(return_value=pages)

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = (
                get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            )
            result = await fn(page_id="p2")

        data = json.loads(result)
        assert data["id"] == "p2"
        assert data["title"] == "Page 2"


class TestGetSamplesResource:
    """Tests for get_samples_resource."""

    @pytest.mark.asyncio
    async def test_examples_dir_not_found(self) -> None:
        """Test resource returns error when examples dir not found."""
        with patch(
            "auroraview_mcp.resources.providers.get_examples_dir",
            side_effect=FileNotFoundError("not found"),
        ):
            from auroraview_mcp.resources.providers import get_samples_resource

            fn = (
                get_samples_resource.fn
                if hasattr(get_samples_resource, "fn")
                else get_samples_resource
            )
            result = await fn()

        data = json.loads(result)
        assert "error" in data
        assert "not found" in data["error"].lower()

    @pytest.mark.asyncio
    async def test_returns_samples_list(self) -> None:
        """Test resource returns sample list."""
        samples = [
            {"name": "demo1", "title": "Demo 1", "description": "First demo"},
            {"name": "demo2", "title": "Demo 2", "description": "Second demo"},
        ]

        with (
            patch(
                "auroraview_mcp.resources.providers.get_examples_dir",
                return_value=Path("/fake/examples"),
            ),
            patch("auroraview_mcp.resources.providers.scan_samples", return_value=samples),
        ):
            from auroraview_mcp.resources.providers import get_samples_resource

            fn = (
                get_samples_resource.fn
                if hasattr(get_samples_resource, "fn")
                else get_samples_resource
            )
            result = await fn()

        data = json.loads(result)
        assert len(data) == 2
        assert data[0]["name"] == "demo1"
        assert data[1]["name"] == "demo2"

    @pytest.mark.asyncio
    async def test_returns_empty_when_no_samples(self) -> None:
        """Test resource returns empty list when no samples."""
        with (
            patch(
                "auroraview_mcp.resources.providers.get_examples_dir",
                return_value=Path("/fake/examples"),
            ),
            patch("auroraview_mcp.resources.providers.scan_samples", return_value=[]),
        ):
            from auroraview_mcp.resources.providers import get_samples_resource

            fn = (
                get_samples_resource.fn
                if hasattr(get_samples_resource, "fn")
                else get_samples_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data == []


class TestGetSampleSourceResource:
    """Tests for get_sample_source_resource."""

    @pytest.mark.asyncio
    async def test_examples_dir_not_found(self) -> None:
        """Test resource returns error message when examples dir not found."""
        with patch(
            "auroraview_mcp.resources.providers.get_examples_dir",
            side_effect=FileNotFoundError("not found"),
        ):
            from auroraview_mcp.resources.providers import get_sample_source_resource

            fn = (
                get_sample_source_resource.fn
                if hasattr(get_sample_source_resource, "fn")
                else get_sample_source_resource
            )
            result = await fn(name="demo1")

        assert result.startswith("# Error:")
        assert "Examples directory not found" in result

    @pytest.mark.asyncio
    async def test_sample_py_file_found(self) -> None:
        """Test resource returns source when .py file found."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)
            demo_file = examples_dir / "demo1.py"
            demo_file.write_text('"""Demo 1"""\nprint("hello")', encoding="utf-8")

            with patch(
                "auroraview_mcp.resources.providers.get_examples_dir",
                return_value=examples_dir,
            ):
                from auroraview_mcp.resources.providers import get_sample_source_resource

                fn = (
                    get_sample_source_resource.fn
                    if hasattr(get_sample_source_resource, "fn")
                    else get_sample_source_resource
                )
                result = await fn(name="demo1")

        assert '"""Demo 1"""' in result
        assert 'print("hello")' in result

    @pytest.mark.asyncio
    async def test_sample_demo_suffix_file_found(self) -> None:
        """Test resource finds file with _demo suffix."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)
            demo_file = examples_dir / "my_tool_demo.py"
            demo_file.write_text('"""My Tool Demo"""\nprint("tool")', encoding="utf-8")

            with patch(
                "auroraview_mcp.resources.providers.get_examples_dir",
                return_value=examples_dir,
            ):
                from auroraview_mcp.resources.providers import get_sample_source_resource

                fn = (
                    get_sample_source_resource.fn
                    if hasattr(get_sample_source_resource, "fn")
                    else get_sample_source_resource
                )
                result = await fn(name="my_tool")

        assert '"""My Tool Demo"""' in result

    @pytest.mark.asyncio
    async def test_sample_as_directory(self) -> None:
        """Test resource finds source in sample directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)
            sample_dir = examples_dir / "my_sample"
            sample_dir.mkdir()
            main_file = sample_dir / "main.py"
            main_file.write_text(
                '"""My Sample"""\nprint("from dir")', encoding="utf-8"
            )

            sample_info = {
                "name": "my_sample",
                "main_file": str(main_file),
                "title": "My Sample",
            }

            with (
                patch(
                    "auroraview_mcp.resources.providers.get_examples_dir",
                    return_value=examples_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers.get_sample_info",
                    return_value=sample_info,
                ),
            ):
                from auroraview_mcp.resources.providers import get_sample_source_resource

                fn = (
                    get_sample_source_resource.fn
                    if hasattr(get_sample_source_resource, "fn")
                    else get_sample_source_resource
                )
                result = await fn(name="my_sample")

        assert '"""My Sample"""' in result

    @pytest.mark.asyncio
    async def test_sample_not_found(self) -> None:
        """Test resource returns error when sample not found."""
        with tempfile.TemporaryDirectory() as tmpdir:
            examples_dir = Path(tmpdir)

            with patch(
                "auroraview_mcp.resources.providers.get_examples_dir",
                return_value=examples_dir,
            ):
                from auroraview_mcp.resources.providers import get_sample_source_resource

                fn = (
                    get_sample_source_resource.fn
                    if hasattr(get_sample_source_resource, "fn")
                    else get_sample_source_resource
                )
                result = await fn(name="nonexistent")

        assert "# Error:" in result
        assert "nonexistent" in result


class TestGetLogsResource:
    """Tests for get_logs_resource."""

    @pytest.mark.asyncio
    async def test_not_connected_returns_error(self) -> None:
        """Test resource returns error when not connected."""
        mock_manager = MagicMock()
        mock_manager.is_connected = False

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = (
                get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            )
            result = await fn()

        data = json.loads(result)
        assert "error" in data
        assert data["error"] == "Not connected"

    @pytest.mark.asyncio
    async def test_no_page_returns_error(self) -> None:
        """Test resource returns error when no page selected."""
        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.current_page = None

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = (
                get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            )
            result = await fn()

        data = json.loads(result)
        assert "error" in data
        assert data["error"] == "No page selected"

    @pytest.mark.asyncio
    async def test_returns_logs(self) -> None:
        """Test resource returns console logs."""
        logs = [
            {"level": "info", "message": "Page loaded"},
            {"level": "warn", "message": "Deprecated API"},
        ]

        mock_conn = MagicMock()
        mock_conn.evaluate = AsyncMock(return_value=logs)

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.current_page = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_conn)

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = (
                get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            )
            result = await fn()

        data = json.loads(result)
        assert len(data) == 2
        assert data[0]["level"] == "info"
        assert data[1]["level"] == "warn"

    @pytest.mark.asyncio
    async def test_returns_empty_when_none(self) -> None:
        """Test resource returns empty array when logs is None."""
        mock_conn = MagicMock()
        mock_conn.evaluate = AsyncMock(return_value=None)

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.current_page = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_conn)

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = (
                get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data == []

    @pytest.mark.asyncio
    async def test_exception_returns_error(self) -> None:
        """Test resource returns error when exception occurs."""
        mock_conn = MagicMock()
        mock_conn.evaluate = AsyncMock(side_effect=RuntimeError("WebSocket closed"))

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.current_page = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_conn)

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = (
                get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            )
            result = await fn()

        data = json.loads(result)
        assert "error" in data
        assert "WebSocket closed" in data["error"]


class TestGetGalleryResource:
    """Tests for get_gallery_resource."""

    @pytest.mark.asyncio
    async def test_gallery_dir_not_found(self) -> None:
        """Test resource returns error when gallery dir not found."""
        with patch(
            "auroraview_mcp.resources.providers.get_gallery_dir",
            side_effect=FileNotFoundError("Gallery directory not found"),
        ):
            from auroraview_mcp.resources.providers import get_gallery_resource

            fn = (
                get_gallery_resource.fn
                if hasattr(get_gallery_resource, "fn")
                else get_gallery_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data["running"] is False
        assert "error" in data

    @pytest.mark.asyncio
    async def test_gallery_not_running(self) -> None:
        """Test resource when gallery is not running."""
        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)

            with (
                patch(
                    "auroraview_mcp.resources.providers.get_gallery_dir",
                    return_value=gallery_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers._process_manager"
                ) as mock_pm,
            ):
                mock_pm.get_gallery.return_value = None

                from auroraview_mcp.resources.providers import get_gallery_resource

                fn = (
                    get_gallery_resource.fn
                    if hasattr(get_gallery_resource, "fn")
                    else get_gallery_resource
                )
                result = await fn()

        data = json.loads(result)
        assert data["running"] is False
        assert "gallery_dir" in data

    @pytest.mark.asyncio
    async def test_gallery_running(self) -> None:
        """Test resource when gallery is running."""
        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)
            # Create dist/index.html
            dist_dir = gallery_dir / "dist"
            dist_dir.mkdir()
            (dist_dir / "index.html").write_text("<html/>")

            mock_proc = MagicMock()
            mock_proc.poll.return_value = None  # Running

            mock_proc_info = MagicMock()
            mock_proc_info.process = mock_proc
            mock_proc_info.pid = 12345
            mock_proc_info.port = 7890

            with (
                patch(
                    "auroraview_mcp.resources.providers.get_gallery_dir",
                    return_value=gallery_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers._process_manager"
                ) as mock_pm,
            ):
                mock_pm.get_gallery.return_value = mock_proc_info

                from auroraview_mcp.resources.providers import get_gallery_resource

                fn = (
                    get_gallery_resource.fn
                    if hasattr(get_gallery_resource, "fn")
                    else get_gallery_resource
                )
                result = await fn()

        data = json.loads(result)
        assert data["running"] is True
        assert data["pid"] == 12345
        assert data["port"] == 7890
        assert data["dist_exists"] is True

    @pytest.mark.asyncio
    async def test_gallery_terminated(self) -> None:
        """Test resource when gallery process has terminated."""
        with tempfile.TemporaryDirectory() as tmpdir:
            gallery_dir = Path(tmpdir)

            mock_proc = MagicMock()
            mock_proc.poll.return_value = 1  # Terminated

            mock_proc_info = MagicMock()
            mock_proc_info.process = mock_proc
            mock_proc_info.pid = 12345

            with (
                patch(
                    "auroraview_mcp.resources.providers.get_gallery_dir",
                    return_value=gallery_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers._process_manager"
                ) as mock_pm,
            ):
                mock_pm.get_gallery.return_value = mock_proc_info

                from auroraview_mcp.resources.providers import get_gallery_resource

                fn = (
                    get_gallery_resource.fn
                    if hasattr(get_gallery_resource, "fn")
                    else get_gallery_resource
                )
                result = await fn()

        data = json.loads(result)
        assert data["running"] is False


class TestGetProjectResource:
    """Tests for get_project_resource."""

    @pytest.mark.asyncio
    async def test_project_dirs_not_found(self) -> None:
        """Test resource returns error when project dirs not found."""
        with patch(
            "auroraview_mcp.resources.providers.get_project_root",
            side_effect=FileNotFoundError("Project root not found"),
        ):
            from auroraview_mcp.resources.providers import get_project_resource

            fn = (
                get_project_resource.fn
                if hasattr(get_project_resource, "fn")
                else get_project_resource
            )
            result = await fn()

        data = json.loads(result)
        assert "error" in data

    @pytest.mark.asyncio
    async def test_returns_project_info(self) -> None:
        """Test resource returns project information."""
        with tempfile.TemporaryDirectory() as tmpdir:
            root_dir = Path(tmpdir)
            gallery_dir = root_dir / "gallery"
            gallery_dir.mkdir()
            examples_dir = root_dir / "examples"
            examples_dir.mkdir()

            with (
                patch(
                    "auroraview_mcp.resources.providers.get_project_root",
                    return_value=root_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers.get_gallery_dir",
                    return_value=gallery_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers.get_examples_dir",
                    return_value=examples_dir,
                ),
                patch("auroraview_mcp.resources.providers.scan_samples", return_value=[]),
            ):
                from auroraview_mcp.resources.providers import get_project_resource

                fn = (
                    get_project_resource.fn
                    if hasattr(get_project_resource, "fn")
                    else get_project_resource
                )
                result = await fn()

        data = json.loads(result)
        assert "project_root" in data
        assert "gallery_dir" in data
        assert "examples_dir" in data
        assert "gallery_built" in data
        assert "sample_count" in data
        assert data["sample_count"] == 0

    @pytest.mark.asyncio
    async def test_reports_gallery_built(self) -> None:
        """Test resource correctly reports gallery build status."""
        with tempfile.TemporaryDirectory() as tmpdir:
            root_dir = Path(tmpdir)
            gallery_dir = root_dir / "gallery"
            gallery_dir.mkdir()
            dist_dir = gallery_dir / "dist"
            dist_dir.mkdir()
            (dist_dir / "index.html").write_text("<html/>")
            examples_dir = root_dir / "examples"
            examples_dir.mkdir()

            with (
                patch(
                    "auroraview_mcp.resources.providers.get_project_root",
                    return_value=root_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers.get_gallery_dir",
                    return_value=gallery_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers.get_examples_dir",
                    return_value=examples_dir,
                ),
                patch(
                    "auroraview_mcp.resources.providers.scan_samples",
                    return_value=[{"name": "s1"}, {"name": "s2"}],
                ),
            ):
                from auroraview_mcp.resources.providers import get_project_resource

                fn = (
                    get_project_resource.fn
                    if hasattr(get_project_resource, "fn")
                    else get_project_resource
                )
                result = await fn()

        data = json.loads(result)
        assert data["gallery_built"] is True
        assert data["sample_count"] == 2


class TestGetProcessesResource:
    """Tests for get_processes_resource."""

    @pytest.mark.asyncio
    async def test_returns_empty_list_when_no_processes(self) -> None:
        """Test resource returns empty list when no processes running."""
        with patch("auroraview_mcp.resources.providers._process_manager") as mock_pm:
            mock_pm.list_all.return_value = []

            from auroraview_mcp.resources.providers import get_processes_resource

            fn = (
                get_processes_resource.fn
                if hasattr(get_processes_resource, "fn")
                else get_processes_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data == []

    @pytest.mark.asyncio
    async def test_returns_running_processes(self) -> None:
        """Test resource returns running process info."""
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None  # Running

        mock_info = MagicMock()
        mock_info.pid = 1234
        mock_info.name = "demo_sample"
        mock_info.port = 9222
        mock_info.is_gallery = False
        mock_info.process = mock_proc

        with patch("auroraview_mcp.resources.providers._process_manager") as mock_pm:
            mock_pm.list_all.return_value = [mock_info]

            from auroraview_mcp.resources.providers import get_processes_resource

            fn = (
                get_processes_resource.fn
                if hasattr(get_processes_resource, "fn")
                else get_processes_resource
            )
            result = await fn()

        data = json.loads(result)
        assert len(data) == 1
        assert data[0]["pid"] == 1234
        assert data[0]["name"] == "demo_sample"
        assert data[0]["status"] == "running"
        assert data[0]["port"] == 9222
        assert data[0]["is_gallery"] is False

    @pytest.mark.asyncio
    async def test_returns_terminated_process_status(self) -> None:
        """Test resource correctly reports terminated process status."""
        mock_proc = MagicMock()
        mock_proc.poll.return_value = 1  # Terminated

        mock_info = MagicMock()
        mock_info.pid = 5678
        mock_info.name = "old_sample"
        mock_info.port = None
        mock_info.is_gallery = False
        mock_info.process = mock_proc

        with patch("auroraview_mcp.resources.providers._process_manager") as mock_pm:
            mock_pm.list_all.return_value = [mock_info]

            from auroraview_mcp.resources.providers import get_processes_resource

            fn = (
                get_processes_resource.fn
                if hasattr(get_processes_resource, "fn")
                else get_processes_resource
            )
            result = await fn()

        data = json.loads(result)
        assert len(data) == 1
        assert data[0]["status"] == "terminated"

    @pytest.mark.asyncio
    async def test_gallery_process_flagged(self) -> None:
        """Test resource correctly marks gallery process."""
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None

        mock_info = MagicMock()
        mock_info.pid = 9999
        mock_info.name = "gallery"
        mock_info.port = 7890
        mock_info.is_gallery = True
        mock_info.process = mock_proc

        with patch("auroraview_mcp.resources.providers._process_manager") as mock_pm:
            mock_pm.list_all.return_value = [mock_info]

            from auroraview_mcp.resources.providers import get_processes_resource

            fn = (
                get_processes_resource.fn
                if hasattr(get_processes_resource, "fn")
                else get_processes_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data[0]["is_gallery"] is True


class TestGetTelemetryResource:
    """Tests for get_telemetry_resource."""

    @pytest.mark.asyncio
    async def test_telemetry_module_not_available(self) -> None:
        """Test resource returns error when telemetry module not available."""
        with patch.dict("sys.modules", {"auroraview.telemetry": None}):
            from auroraview_mcp.resources.providers import get_telemetry_resource

            fn = (
                get_telemetry_resource.fn
                if hasattr(get_telemetry_resource, "fn")
                else get_telemetry_resource
            )
            result = await fn()

        data = json.loads(result)
        assert "error" in data

    @pytest.mark.asyncio
    async def test_telemetry_returns_snapshots(self) -> None:
        """Test resource returns telemetry snapshots when available."""
        snapshots = [
            {"webview_id": "wv1", "emit_count": 10, "eval_count": 5},
            {"webview_id": "wv2", "emit_count": 3, "eval_count": 8},
        ]

        mock_telemetry = MagicMock()
        mock_telemetry.get_all_snapshots = MagicMock(return_value=snapshots)

        with patch.dict("sys.modules", {"auroraview.telemetry": mock_telemetry}):
            from auroraview_mcp.resources.providers import get_telemetry_resource

            fn = (
                get_telemetry_resource.fn
                if hasattr(get_telemetry_resource, "fn")
                else get_telemetry_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data["count"] == 2
        assert len(data["instances"]) == 2

    @pytest.mark.asyncio
    async def test_telemetry_empty_snapshots(self) -> None:
        """Test resource returns zero count when no snapshots."""
        mock_telemetry = MagicMock()
        mock_telemetry.get_all_snapshots = MagicMock(return_value=[])

        with patch.dict("sys.modules", {"auroraview.telemetry": mock_telemetry}):
            from auroraview_mcp.resources.providers import get_telemetry_resource

            fn = (
                get_telemetry_resource.fn
                if hasattr(get_telemetry_resource, "fn")
                else get_telemetry_resource
            )
            result = await fn()

        data = json.loads(result)
        assert data["count"] == 0
        assert data["instances"] == []
