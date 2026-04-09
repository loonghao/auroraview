"""Extended tests for resource providers and Instance serialization.

Covers:
- get_sample_source_resource() file discovery paths
- get_logs_resource() edge cases including evaluate exceptions
- get_page_resource() page lookup edge cases
- get_instances_resource() serialization
- Instance.to_dict() complete field coverage
- InstanceDiscovery._instance_from_registry() default values
"""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from auroraview_mcp.discovery import Instance, InstanceDiscovery

# ---------------------------------------------------------------------------
# Instance.to_dict() complete field verification
# ---------------------------------------------------------------------------


class TestInstanceToDictComplete:
    """Verify all fields are serialized correctly in Instance.to_dict()."""

    def test_to_dict_all_fields_present(self) -> None:
        """to_dict() includes every declared field."""
        inst = Instance(
            port=9222,
            browser="Edg/120.0",
            ws_url="ws://127.0.0.1:9222/devtools/browser/abc",
            user_agent="Mozilla/5.0 (Windows NT 10.0)",
            protocol_version="1.3",
            pid=12345,
            title="My App",
            url="http://localhost:8080",
            dcc_type="maya",
            dcc_version="2025",
            panel_name="Maya Tool",
            window_id="win-abc",
            app_name="AuroraView",
            app_version="0.4.18",
            dcc_python_version="3.11",
            dock_area="left",
            start_time=1700000000.0,
            devtools_url="devtools://devtools/bundled/inspector.html",
            html_title="Page Title",
            is_loading=False,
            metadata={"custom_key": "custom_value"},
        )
        d = inst.to_dict()

        assert d["port"] == 9222
        assert d["browser"] == "Edg/120.0"
        assert d["ws_url"] == "ws://127.0.0.1:9222/devtools/browser/abc"
        assert d["user_agent"] == "Mozilla/5.0 (Windows NT 10.0)"
        assert d["protocol_version"] == "1.3"
        assert d["pid"] == 12345
        assert d["title"] == "My App"
        assert d["url"] == "http://localhost:8080"
        assert d["dcc_type"] == "maya"
        assert d["dcc_version"] == "2025"
        assert d["panel_name"] == "Maya Tool"
        assert d["window_id"] == "win-abc"
        assert d["app_name"] == "AuroraView"
        assert d["app_version"] == "0.4.18"
        assert d["dcc_python_version"] == "3.11"
        assert d["dock_area"] == "left"
        assert d["start_time"] == 1700000000.0
        assert d["devtools_url"] == "devtools://devtools/bundled/inspector.html"
        assert d["html_title"] == "Page Title"
        assert d["is_loading"] is False
        assert d["metadata"] == {"custom_key": "custom_value"}

    def test_to_dict_null_optional_fields(self) -> None:
        """Optional fields that are None serialize as None."""
        inst = Instance(port=9222)
        d = inst.to_dict()

        assert d["pid"] is None
        assert d["dcc_type"] is None
        assert d["dcc_version"] is None
        assert d["panel_name"] is None
        assert d["window_id"] is None
        assert d["app_name"] is None
        assert d["app_version"] is None
        assert d["dcc_python_version"] is None
        assert d["dock_area"] is None
        assert d["start_time"] is None
        assert d["devtools_url"] is None

    def test_to_dict_is_json_serializable(self) -> None:
        """to_dict() result can be JSON-serialized without error."""
        inst = Instance(
            port=9222,
            dcc_type="blender",
            metadata={"extra": [1, 2, 3]},
        )
        d = inst.to_dict()
        serialized = json.dumps(d)
        assert '"port": 9222' in serialized

    def test_to_dict_metadata_default_empty(self) -> None:
        """metadata defaults to empty dict."""
        inst = Instance(port=9222)
        assert inst.to_dict()["metadata"] == {}


# ---------------------------------------------------------------------------
# Instance.display_name() edge cases
# ---------------------------------------------------------------------------


class TestInstanceDisplayNameEdgeCases:
    """Additional edge cases for Instance.display_name()."""

    def test_dcc_with_version_in_brackets(self) -> None:
        """DCC type and version appear in brackets."""
        inst = Instance(port=9222, app_name="Tool", dcc_type="houdini", dcc_version="20.0")
        name = inst.display_name()
        assert "[houdini 20.0]" in name

    def test_all_fields_none_uses_port(self) -> None:
        """When no identifiable fields, falls back to port."""
        inst = Instance(port=9999)
        name = inst.display_name()
        assert "9999" in name
        assert "WebView" in name

    def test_app_name_takes_priority_over_title(self) -> None:
        """app_name is used before title when both present."""
        inst = Instance(port=9222, app_name="My App", title="Browser Tab")
        name = inst.display_name()
        assert "My App" in name
        # title should NOT appear separately as the primary identifier
        assert name.startswith("My App")

    def test_panel_name_appended_in_parens(self) -> None:
        """panel_name is shown in parentheses."""
        inst = Instance(port=9222, app_name="Tool", panel_name="Render Panel")
        name = inst.display_name()
        assert "(Render Panel)" in name

    def test_dcc_without_version_no_space(self) -> None:
        """DCC type without version shows just type in brackets."""
        inst = Instance(port=9222, dcc_type="nuke")
        name = inst.display_name()
        assert "[nuke]" in name
        assert "[nuke " not in name


# ---------------------------------------------------------------------------
# InstanceDiscovery._instance_from_registry() edge cases
# ---------------------------------------------------------------------------


class TestInstanceFromRegistry:
    """Tests for _instance_from_registry() default value handling."""

    def test_minimal_valid_data(self) -> None:
        """Only cdp_port provided; all others default."""
        discovery = InstanceDiscovery()
        data = {"cdp_port": 9222}
        inst = discovery._instance_from_registry(data)

        assert inst is not None
        assert inst.port == 9222
        assert inst.app_name == "AuroraView"  # default
        assert inst.title == ""
        assert inst.url == ""

    def test_returns_none_when_no_cdp_port(self) -> None:
        """Returns None when cdp_port missing."""
        discovery = InstanceDiscovery()
        assert discovery._instance_from_registry({}) is None
        assert discovery._instance_from_registry({"port": 9222}) is None  # wrong key

    def test_devtools_url_default_constructed(self) -> None:
        """devtools_url is constructed from port when not in data."""
        discovery = InstanceDiscovery()
        inst = discovery._instance_from_registry({"cdp_port": 9224})

        assert inst is not None
        assert "9224" in inst.devtools_url

    def test_ws_url_default_constructed(self) -> None:
        """ws_url is constructed from port when not in data."""
        discovery = InstanceDiscovery()
        inst = discovery._instance_from_registry({"cdp_port": 9225})

        assert inst is not None
        assert "9225" in inst.ws_url

    def test_all_optional_fields_populated(self) -> None:
        """All optional fields read from data dict."""
        discovery = InstanceDiscovery()
        data = {
            "cdp_port": 9222,
            "pid": 5000,
            "window_id": "w-1",
            "app_name": "Custom App",
            "app_version": "1.2.3",
            "dcc_type": "blender",
            "dcc_version": "3.6",
            "panel_name": "3D View",
            "dock_area": "right",
            "start_time": 1700000000.0,
            "devtools_url": "devtools://custom",
            "html_title": "HTML Title",
            "is_loading": True,
            "metadata": {"foo": "bar"},
        }
        inst = discovery._instance_from_registry(data)

        assert inst is not None
        assert inst.pid == 5000
        assert inst.window_id == "w-1"
        assert inst.app_name == "Custom App"
        assert inst.app_version == "1.2.3"
        assert inst.dcc_type == "blender"
        assert inst.dcc_version == "3.6"
        assert inst.panel_name == "3D View"
        assert inst.dock_area == "right"
        assert inst.start_time == 1700000000.0
        assert inst.devtools_url == "devtools://custom"
        assert inst.html_title == "HTML Title"
        assert inst.is_loading is True
        assert inst.metadata == {"foo": "bar"}


# ---------------------------------------------------------------------------
# get_sample_source_resource() edge cases
# ---------------------------------------------------------------------------


class TestGetSampleSourceResource:
    """Tests for get_sample_source_resource() provider."""

    @pytest.mark.asyncio
    async def test_sample_as_py_file(self) -> None:
        """Finds sample as direct .py file in examples dir."""
        with tempfile.TemporaryDirectory() as tmp:
            examples_dir = Path(tmp)
            sample_file = examples_dir / "my_sample.py"
            sample_file.write_text("# sample content", encoding="utf-8")

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
                result = await fn("my_sample")

        assert "# sample content" in result

    @pytest.mark.asyncio
    async def test_sample_with_demo_suffix(self) -> None:
        """Finds sample with _demo suffix."""
        with tempfile.TemporaryDirectory() as tmp:
            examples_dir = Path(tmp)
            (examples_dir / "hello_demo.py").write_text("# demo file", encoding="utf-8")

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
                result = await fn("hello")

        assert "# demo file" in result

    @pytest.mark.asyncio
    async def test_sample_not_found_returns_error_comment(self) -> None:
        """Returns error comment when sample not found."""
        with tempfile.TemporaryDirectory() as tmp:
            examples_dir = Path(tmp)

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
                result = await fn("nonexistent_sample")

        assert "Error" in result
        assert "nonexistent_sample" in result

    @pytest.mark.asyncio
    async def test_examples_dir_not_found_returns_error(self) -> None:
        """Returns error comment when examples dir not found."""
        with patch(
            "auroraview_mcp.resources.providers.get_examples_dir",
            side_effect=FileNotFoundError("Examples not found"),
        ):
            from auroraview_mcp.resources.providers import get_sample_source_resource

            fn = (
                get_sample_source_resource.fn
                if hasattr(get_sample_source_resource, "fn")
                else get_sample_source_resource
            )
            result = await fn("any_sample")

        assert "Error" in result

    @pytest.mark.asyncio
    async def test_sample_with_example_suffix(self) -> None:
        """Finds sample with _example suffix."""
        with tempfile.TemporaryDirectory() as tmp:
            examples_dir = Path(tmp)
            (examples_dir / "tutorial_example.py").write_text("# example code", encoding="utf-8")

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
                result = await fn("tutorial")

        assert "# example code" in result


# ---------------------------------------------------------------------------
# get_logs_resource() edge cases
# ---------------------------------------------------------------------------


class TestGetLogsResourceEdgeCases:
    """Extended tests for get_logs_resource() provider."""

    @pytest.mark.asyncio
    async def test_not_connected_returns_error_json(self) -> None:
        """Returns JSON error when not connected."""
        mock_manager = MagicMock()
        mock_manager.is_connected = False

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            result = await fn()

        data = json.loads(result)
        assert "error" in data

    @pytest.mark.asyncio
    async def test_no_page_selected_returns_error_json(self) -> None:
        """Returns JSON error when no page selected."""
        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.current_page = None

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            result = await fn()

        data = json.loads(result)
        assert "error" in data

    @pytest.mark.asyncio
    async def test_evaluate_exception_returns_error_json(self) -> None:
        """Returns JSON error when evaluate raises exception."""
        mock_conn = MagicMock()
        mock_conn.evaluate = AsyncMock(side_effect=RuntimeError("CDP disconnected"))

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.current_page = MagicMock()
        mock_manager.get_page_connection = AsyncMock(return_value=mock_conn)

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_logs_resource

            fn = get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            result = await fn()

        data = json.loads(result)
        assert "error" in data

    @pytest.mark.asyncio
    async def test_logs_returned_as_json_array(self) -> None:
        """Returns JSON array of log entries."""
        logs = [
            {"level": "log", "text": "Hello"},
            {"level": "error", "text": "Oops"},
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

            fn = get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            result = await fn()

        parsed = json.loads(result)
        assert isinstance(parsed, list)
        assert len(parsed) == 2

    @pytest.mark.asyncio
    async def test_non_list_evaluate_result_returns_empty_array(self) -> None:
        """Non-list evaluate result returns empty JSON array."""
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

            fn = get_logs_resource.fn if hasattr(get_logs_resource, "fn") else get_logs_resource
            result = await fn()

        parsed = json.loads(result)
        assert parsed == []


# ---------------------------------------------------------------------------
# get_page_resource() edge cases
# ---------------------------------------------------------------------------


class TestGetPageResourceEdgeCases:
    """Extended tests for get_page_resource() provider."""

    @pytest.mark.asyncio
    async def test_not_connected_returns_error_json(self) -> None:
        """Returns JSON error when not connected."""
        mock_manager = MagicMock()
        mock_manager.is_connected = False

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            result = await fn("nonexistent-id")

        data = json.loads(result)
        assert "error" in data

    @pytest.mark.asyncio
    async def test_page_id_not_found_returns_error(self) -> None:
        """Returns error JSON when page_id not in pages list."""
        from auroraview_mcp.connection import Page

        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.get_pages = AsyncMock(
            return_value=[Page(id="p1", url="http://localhost", title="P1", ws_url="")]
        )

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            result = await fn("missing-page-id")

        data = json.loads(result)
        assert "error" in data
        assert "missing-page-id" in data["error"]

    @pytest.mark.asyncio
    async def test_page_found_returns_page_dict(self) -> None:
        """Returns page dict when page_id matches."""
        from auroraview_mcp.connection import Page

        page = Page(
            id="page-abc",
            url="http://localhost:8080",
            title="Test App",
            ws_url="ws://...",
        )
        mock_manager = MagicMock()
        mock_manager.is_connected = True
        mock_manager.get_pages = AsyncMock(return_value=[page])

        with patch(
            "auroraview_mcp.resources.providers.get_connection_manager",
            return_value=mock_manager,
        ):
            from auroraview_mcp.resources.providers import get_page_resource

            fn = get_page_resource.fn if hasattr(get_page_resource, "fn") else get_page_resource
            result = await fn("page-abc")

        data = json.loads(result)
        assert data["id"] == "page-abc"
        assert data["url"] == "http://localhost:8080"
        assert data["title"] == "Test App"


# ---------------------------------------------------------------------------
# get_instances_resource() serialization
# ---------------------------------------------------------------------------


class TestGetInstancesResource:
    """Tests for get_instances_resource() provider."""

    @pytest.mark.asyncio
    async def test_returns_json_array(self) -> None:
        """Returns valid JSON array of instance dicts."""
        inst = Instance(
            port=9222,
            browser="Edg/120.0",
            dcc_type="maya",
        )
        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[inst])

        with patch(
            "auroraview_mcp.resources.providers.get_discovery",
            return_value=mock_discovery,
        ):
            from auroraview_mcp.resources.providers import get_instances_resource

            fn = (
                get_instances_resource.fn
                if hasattr(get_instances_resource, "fn")
                else get_instances_resource
            )
            result = await fn()

        data = json.loads(result)
        assert isinstance(data, list)
        assert len(data) == 1
        assert data[0]["port"] == 9222
        assert data[0]["dcc_type"] == "maya"

    @pytest.mark.asyncio
    async def test_empty_discovery_returns_empty_array(self) -> None:
        """Returns empty JSON array when no instances discovered."""
        mock_discovery = MagicMock()
        mock_discovery.discover = AsyncMock(return_value=[])

        with patch(
            "auroraview_mcp.resources.providers.get_discovery",
            return_value=mock_discovery,
        ):
            from auroraview_mcp.resources.providers import get_instances_resource

            fn = (
                get_instances_resource.fn
                if hasattr(get_instances_resource, "fn")
                else get_instances_resource
            )
            result = await fn()

        assert json.loads(result) == []
