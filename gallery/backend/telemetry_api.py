"""Telemetry API handlers for AuroraView Gallery.

Exposes auto-collected WebView performance metrics and framework logs
to the frontend TelemetryPanel.
"""

from __future__ import annotations

import sys

from auroraview import WebView


def register_telemetry_apis(view: WebView) -> None:
    """Register telemetry query APIs with the WebView."""

    @view.bind_call("api.get_telemetry_snapshot")
    def get_telemetry_snapshot(include_logs: bool = False, log_since: int = 0) -> dict:
        """Get telemetry snapshot for this WebView instance."""
        snapshot = view.get_telemetry_snapshot(
            include_logs=include_logs,
            log_since=log_since,
        )
        if snapshot is None:
            return {"ok": False, "error": "Telemetry not enabled (debug=False?)"}
        return {"ok": True, **snapshot}

    @view.bind_call("api.get_all_telemetry")
    def get_all_telemetry(include_logs: bool = False, log_since: int = 0) -> dict:
        """Get telemetry snapshots for all active WebView instances."""
        try:
            from auroraview.telemetry import get_all_snapshots

            snapshots = get_all_snapshots(
                include_logs=include_logs,
                log_since=log_since,
            )
            return {"ok": True, "instances": snapshots, "count": len(snapshots)}
        except ImportError:
            return {"ok": False, "error": "Telemetry module not available"}

    print("[Python] Telemetry APIs registered", file=sys.stderr)
