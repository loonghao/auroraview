"""Tests for telemetry tools module (get_telemetry, get_performance_summary)."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock, patch

import pytest


def make_manager(page=True, connected=True, page_conn=None):
    manager = MagicMock()
    manager.is_connected = connected
    manager.current_page = MagicMock() if page else None
    _page_conn = page_conn or MagicMock()
    manager.get_page_connection = AsyncMock(return_value=_page_conn)
    return manager, _page_conn


def make_snapshot(webview_id="wv-1", uptime=120, errors=0, ipc_p95=None, eval_p95=None):
    return {
        "webview_id": webview_id,
        "uptime_s": uptime,
        "counters": {
            "emit_count": 10,
            "eval_js_count": 5,
            "navigation_count": 2,
            "ipc_call_count": 8,
            "error_count": errors,
        },
        "histograms": {
            "load_time_avg_ms": 120.0,
            "eval_js_avg_ms": 20.0,
            "eval_js_p95_ms": eval_p95,
            "ipc_latency_avg_ms": 5.0,
            "ipc_latency_p95_ms": ipc_p95,
        },
        "last_url": "http://localhost:8080/",
        "last_error": None,
        "otel_available": False,
    }


class TestGetTelemetry:
    """Tests for get_telemetry tool."""

    @pytest.mark.asyncio
    async def test_not_connected_returns_fallback_note(self) -> None:
        """When not connected, falls back to local import (ImportError → note)."""
        manager, _ = make_manager(connected=False, page=False)
        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        # falls through to ImportError path since auroraview.telemetry not installed
        assert "instances" in result
        assert isinstance(result["instances"], list)

    @pytest.mark.asyncio
    async def test_connected_bridge_returns_instances(self) -> None:
        """When connected and bridge returns data, returns instances list."""
        manager, page_conn = make_manager()
        snapshot = make_snapshot("wv-42")
        page_conn.evaluate = AsyncMock(return_value={"instances": [snapshot]})
        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        assert len(result["instances"]) == 1
        assert result["instances"][0]["webview_id"] == "wv-42"

    @pytest.mark.asyncio
    async def test_connected_bridge_returns_list_directly(self) -> None:
        """When bridge returns a list (not dict), wraps into instances."""
        manager, page_conn = make_manager()
        snapshots = [make_snapshot("wv-1"), make_snapshot("wv-2")]
        page_conn.evaluate = AsyncMock(return_value=snapshots)
        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        assert len(result["instances"]) == 2

    @pytest.mark.asyncio
    async def test_connected_filter_by_webview_id(self) -> None:
        """Filters instances by webview_id when provided."""
        manager, page_conn = make_manager()
        snapshots = [make_snapshot("wv-1"), make_snapshot("wv-2")]
        page_conn.evaluate = AsyncMock(return_value={"instances": snapshots})
        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn(webview_id="wv-1")

        assert len(result["instances"]) == 1
        assert result["instances"][0]["webview_id"] == "wv-1"

    @pytest.mark.asyncio
    async def test_connected_bridge_returns_none_falls_back(self) -> None:
        """When bridge returns None, falls back to local module."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(return_value=None)
        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        assert "instances" in result

    @pytest.mark.asyncio
    async def test_bridge_exception_falls_back(self) -> None:
        """When CDP connection raises, falls back gracefully."""
        manager, page_conn = make_manager()
        page_conn.evaluate = AsyncMock(side_effect=Exception("CDP error"))
        with patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        # Should not raise; returns instances (possibly empty with note)
        assert "instances" in result

    @pytest.mark.asyncio
    async def test_local_module_available(self) -> None:
        """When local module available, uses get_all_snapshots."""
        manager, _ = make_manager(connected=False, page=False)
        snapshot = make_snapshot("local-wv")
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[snapshot])
        with (
            patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager),
            patch.dict("sys.modules", {"auroraview.telemetry": mock_module}),
        ):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn()

        assert len(result["instances"]) == 1
        assert result["instances"][0]["webview_id"] == "local-wv"

    @pytest.mark.asyncio
    async def test_local_module_filter_by_id(self) -> None:
        """Uses get_snapshot when webview_id provided and local module available."""
        manager, _ = make_manager(connected=False, page=False)
        snapshot = make_snapshot("target-wv")
        mock_module = MagicMock()
        mock_module.get_snapshot = MagicMock(return_value=snapshot)
        with (
            patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager),
            patch.dict("sys.modules", {"auroraview.telemetry": mock_module}),
        ):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn(webview_id="target-wv")

        assert len(result["instances"]) == 1
        assert result["instances"][0]["webview_id"] == "target-wv"

    @pytest.mark.asyncio
    async def test_local_module_filter_not_found(self) -> None:
        """Returns empty when get_snapshot returns None."""
        manager, _ = make_manager(connected=False, page=False)
        mock_module = MagicMock()
        mock_module.get_snapshot = MagicMock(return_value=None)
        with (
            patch("auroraview_mcp.tools.telemetry.get_connection_manager", return_value=manager),
            patch.dict("sys.modules", {"auroraview.telemetry": mock_module}),
        ):
            from auroraview_mcp.tools.telemetry import get_telemetry

            fn = get_telemetry.fn if hasattr(get_telemetry, "fn") else get_telemetry
            result = await fn(webview_id="ghost")

        assert result["instances"] == []


class TestGetPerformanceSummary:
    """Tests for get_performance_summary tool."""

    @pytest.mark.asyncio
    async def test_module_not_available_returns_error(self) -> None:
        """Returns error dict when telemetry module not available."""
        with patch.dict("sys.modules", {"auroraview.telemetry": None}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert "error" in result

    @pytest.mark.asyncio
    async def test_no_snapshots(self) -> None:
        """Returns total_instances=0 note when no active instances."""
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert result["total_instances"] == 0
        assert "note" in result

    @pytest.mark.asyncio
    async def test_aggregates_counters(self) -> None:
        """Aggregates emit/eval/ipc/error counts across instances."""
        s1 = make_snapshot("wv-1", errors=0)
        s2 = make_snapshot("wv-2", errors=2)
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[s1, s2])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert result["total_instances"] == 2
        assert result["total_emit_count"] == 20  # 10 + 10
        assert result["total_eval_js_count"] == 10  # 5 + 5
        assert result["total_ipc_calls"] == 16  # 8 + 8
        assert result["total_errors"] == 2

    @pytest.mark.asyncio
    async def test_warnings_high_ipc_latency(self) -> None:
        """Generates warning when IPC p95 > 50ms."""
        s = make_snapshot("wv-slow", ipc_p95=75.0)
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[s])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert any("IPC latency" in w for w in result["warnings"])

    @pytest.mark.asyncio
    async def test_warnings_high_eval_latency(self) -> None:
        """Generates warning when JS eval p95 > 100ms."""
        s = make_snapshot("wv-slow", eval_p95=150.0)
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[s])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert any("eval p95" in w for w in result["warnings"])

    @pytest.mark.asyncio
    async def test_warnings_errors(self) -> None:
        """Generates warning when error_count > 0."""
        s = make_snapshot("wv-err", errors=3)
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[s])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert any("error" in w.lower() for w in result["warnings"])

    @pytest.mark.asyncio
    async def test_no_warnings_when_healthy(self) -> None:
        """No warnings when all metrics are within thresholds."""
        s = make_snapshot("wv-healthy", errors=0, ipc_p95=10.0, eval_p95=50.0)
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[s])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert result["warnings"] == []

    @pytest.mark.asyncio
    async def test_instances_brief_format(self) -> None:
        """Each instance in summary has expected brief fields."""
        s = make_snapshot("wv-brief")
        mock_module = MagicMock()
        mock_module.get_all_snapshots = MagicMock(return_value=[s])
        with patch.dict("sys.modules", {"auroraview.telemetry": mock_module}):
            from auroraview_mcp.tools.telemetry import get_performance_summary

            fn = (
                get_performance_summary.fn
                if hasattr(get_performance_summary, "fn")
                else get_performance_summary
            )
            result = await fn()

        assert len(result["instances"]) == 1
        brief = result["instances"][0]
        assert brief["webview_id"] == "wv-brief"
        assert "uptime_s" in brief
        assert "emit_count" in brief
        assert "error_count" in brief
