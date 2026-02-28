"""Telemetry tools for AuroraView MCP Server.

Provides AI-accessible performance metrics and diagnostics for WebView instances.
Metrics are collected automatically when WebView runs with debug=True (default).
"""

from __future__ import annotations

from typing import Any

from auroraview_mcp.server import get_connection_manager, mcp


@mcp.tool()
async def get_telemetry(webview_id: str | None = None) -> dict[str, Any]:
    """Get telemetry metrics for WebView instances.

    Returns performance counters, timing histograms, and health indicators
    collected automatically by the AuroraView telemetry system. Use this
    to understand WebView performance, IPC latency, JS evaluation times,
    and error rates.

    Args:
        webview_id: Specific WebView ID to query. If None, returns all instances.

    Returns:
        Telemetry data containing:
        - instances: List of WebView telemetry snapshots, each with:
            - webview_id: Instance identifier
            - uptime_s: Seconds since creation
            - counters: {emit_count, eval_js_count, navigation_count, ipc_call_count, error_count}
            - histograms: {load_time_avg_ms, eval_js_avg_ms, eval_js_p95_ms, ipc_latency_avg_ms, ipc_latency_p95_ms}
            - last_url: Last navigated URL
            - last_error: Last error type
            - otel_available: Whether OpenTelemetry export is active
    """
    manager = get_connection_manager()
    bridge_error: str | None = None

    # Try to get telemetry from the connected WebView's Python backend

    if manager.is_connected and manager.current_page is not None:
        try:
            conn = await manager.get_page_connection()

            script = """
            (() => {
                if (window.auroraview && window.auroraview.call) {
                    return window.auroraview.call('api.get_all_telemetry', {
                        include_logs: false,
                        log_since: 0,
                    });
                }
                return null;
            })()
            """

            result = await conn.evaluate(script)
            if result is not None:
                instances = []
                if isinstance(result, dict):
                    maybe_instances = result.get("instances")
                    if isinstance(maybe_instances, list):
                        instances = maybe_instances
                elif isinstance(result, list):
                    instances = result
                else:
                    instances = [result]

                if webview_id:
                    instances = [s for s in instances if s.get("webview_id") == webview_id]
                return {"instances": instances}
        except Exception as exc:
            # Continue with local-module fallback if bridge call is unavailable
            bridge_error = str(exc)



    # Fallback: try to import from the Python telemetry module directly
    # (works when MCP server runs in the same process)
    try:
        from auroraview.telemetry import get_all_snapshots, get_snapshot

        if webview_id:
            snapshot = get_snapshot(webview_id)
            return {"instances": [snapshot] if snapshot else []}
        else:
            return {"instances": get_all_snapshots()}
    except ImportError:
        note = "Telemetry module not available. Ensure auroraview is installed with telemetry-python feature."
        if bridge_error:
            note = f"{note} Bridge error: {bridge_error}"
        return {
            "instances": [],
            "note": note,
        }



@mcp.tool()
async def get_performance_summary() -> dict[str, Any]:
    """Get a high-level performance summary of all WebView instances.

    Returns a condensed overview useful for quick health checks, including
    total instances, aggregate counters, and any performance warnings.

    Returns:
        Performance summary:
        - total_instances: Number of active WebView instances
        - total_emit_count: Total events emitted across all instances
        - total_eval_js_count: Total JS evaluations
        - total_ipc_calls: Total IPC calls
        - total_errors: Total errors
        - warnings: List of performance warnings (e.g., high latency)
        - instances: Brief per-instance summary
    """
    try:
        from auroraview.telemetry import get_all_snapshots

        snapshots = get_all_snapshots()
    except ImportError:
        return {"error": "Telemetry module not available"}

    if not snapshots:
        return {
            "total_instances": 0,
            "note": "No active WebView instances with telemetry enabled.",
        }

    warnings = []
    total_emit = 0
    total_eval = 0
    total_ipc = 0
    total_errors = 0
    instances_brief = []

    for s in snapshots:
        counters = s.get("counters", {})
        histograms = s.get("histograms", {})

        total_emit += counters.get("emit_count", 0)
        total_eval += counters.get("eval_js_count", 0)
        total_ipc += counters.get("ipc_call_count", 0)
        total_errors += counters.get("error_count", 0)

        # Check for performance issues
        ipc_p95 = histograms.get("ipc_latency_p95_ms")
        if ipc_p95 is not None and ipc_p95 > 50:
            warnings.append(
                f"{s['webview_id']}: IPC latency p95={ipc_p95}ms (>50ms threshold)"
            )

        eval_p95 = histograms.get("eval_js_p95_ms")
        if eval_p95 is not None and eval_p95 > 100:
            warnings.append(
                f"{s['webview_id']}: JS eval p95={eval_p95}ms (>100ms threshold)"
            )

        if counters.get("error_count", 0) > 0:
            warnings.append(
                f"{s['webview_id']}: {counters['error_count']} error(s), last: {s.get('last_error')}"
            )

        instances_brief.append(
            {
                "webview_id": s["webview_id"],
                "uptime_s": s.get("uptime_s"),
                "emit_count": counters.get("emit_count", 0),
                "ipc_latency_avg_ms": histograms.get("ipc_latency_avg_ms"),
                "error_count": counters.get("error_count", 0),
            }
        )

    return {
        "total_instances": len(snapshots),
        "total_emit_count": total_emit,
        "total_eval_js_count": total_eval,
        "total_ipc_calls": total_ipc,
        "total_errors": total_errors,
        "warnings": warnings,
        "instances": instances_brief,
    }
