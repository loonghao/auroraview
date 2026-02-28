# -*- coding: utf-8 -*-
"""AuroraView Telemetry - OpenTelemetry-based observability for Python.

This module provides a Python interface to the Rust-based OpenTelemetry
telemetry system. It enables performance profiling, distributed tracing,
and metrics collection for WebView operations, signal threads, and IPC.

**Auto-instrumentation** (recommended)::

    # No manual setup needed! When debug=True (default), WebView
    # automatically collects metrics for all lifecycle operations.
    from auroraview import WebView

    webview = WebView(title="My App", debug=True)
    webview.show()

    # Query current metrics snapshot
    snapshot = webview.get_telemetry_snapshot()
    print(snapshot)
    # {"webview_id": "wv-1", "uptime_s": 5.3,
    #  "counters": {"emit_count": 10, "eval_js_count": 5, ...},
    #  "histograms": {"eval_js_avg_ms": 1.2, "ipc_latency_p95_ms": 4.8}}

    # Query all active WebView instances
    from auroraview.telemetry import get_all_snapshots
    all_metrics = get_all_snapshots()

**Manual control** (advanced)::

    from auroraview.telemetry import init, shutdown, TelemetryConfig

    # Initialize with OTLP export (Jaeger / Grafana)
    config = TelemetryConfig(
        otlp_endpoint="http://localhost:4317",
        log_level="debug",
        traces_enabled=True,
        metrics_enabled=True,
    )
    init(config)

    # ... run WebView ...

    shutdown()
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

__all__ = [
    "TelemetryConfig",
    "WebViewMetrics",
    "init",
    "shutdown",
    "is_enabled",
    "enable",
    "disable",
    "record_load_time",
    "record_ipc_message",
    "record_error",
    "get_all_snapshots",
    "get_snapshot",
]

_IMPORT_ERROR = None

try:
    from auroraview._core import telemetry as _telemetry

    TelemetryConfig = _telemetry.TelemetryConfig
    WebViewMetrics = _telemetry.WebViewMetrics
except ImportError as exc:
    _IMPORT_ERROR = str(exc)
    TelemetryConfig = None  # type: ignore[assignment,misc]
    WebViewMetrics = None  # type: ignore[assignment,misc]


def _check_available() -> None:
    if _IMPORT_ERROR is not None:
        raise RuntimeError(
            "Telemetry module is not available. "
            "Build with 'telemetry-python' feature enabled. "
            "Error: {}".format(_IMPORT_ERROR)
        )


# =========================================================================
# Manual init / shutdown (OTLP export, custom config)
# =========================================================================


def init(config: Optional["TelemetryConfig"] = None) -> None:  # noqa: F821
    """Initialize the telemetry system.

    Args:
        config: Optional configuration. Uses defaults if not provided.

    Raises:
        RuntimeError: If telemetry is already initialized or not available.

    Example::

        from auroraview.telemetry import init, TelemetryConfig

        # Default config
        init()

        # Custom config with OTLP export
        config = TelemetryConfig(
            otlp_endpoint="http://localhost:4317",
            log_level="debug",
        )
        init(config)
    """
    _check_available()
    _telemetry.init_telemetry(config)


def shutdown() -> None:
    """Shutdown the telemetry system, flushing all pending data.

    Example::

        from auroraview.telemetry import shutdown
        shutdown()
    """
    _check_available()
    _telemetry.shutdown_telemetry()


def is_enabled() -> bool:
    """Check if telemetry is currently enabled.

    Returns:
        True if telemetry is initialized and enabled.
    """
    _check_available()
    return _telemetry.is_telemetry_enabled()


def enable() -> None:
    """Re-enable telemetry at runtime."""
    _check_available()
    _telemetry.enable_telemetry()


def disable() -> None:
    """Disable telemetry at runtime (data stops being collected)."""
    _check_available()
    _telemetry.disable_telemetry()


# =========================================================================
# Manual record helpers (for advanced usage)
# =========================================================================


def record_load_time(webview_id: str, duration_ms: float) -> None:
    """Record WebView page load time.

    Args:
        webview_id: Identifier of the WebView instance.
        duration_ms: Load time in milliseconds.
    """
    _check_available()
    _telemetry.record_webview_load_time(webview_id, duration_ms)


def record_ipc_message(webview_id: str, direction: str, latency_ms: float) -> None:
    """Record an IPC message with latency.

    Args:
        webview_id: Identifier of the WebView instance.
        direction: Message direction (e.g. "js_to_rust", "rust_to_js").
        latency_ms: Round-trip latency in milliseconds.
    """
    _check_available()
    _telemetry.record_ipc_message(webview_id, direction, latency_ms)


def record_error(webview_id: str, error_type: str) -> None:
    """Record an error occurrence.

    Args:
        webview_id: Identifier of the WebView instance.
        error_type: Error classification string.
    """
    _check_available()
    _telemetry.record_telemetry_error(webview_id, error_type)


# =========================================================================
# Auto-telemetry snapshot queries
# =========================================================================


def get_all_snapshots(
    include_logs: bool = False,
    log_since: int = 0,
) -> List[Dict[str, Any]]:
    """Get telemetry snapshots for all active WebView instances.

    Args:
        include_logs: If True, include recent log entries in each snapshot.
        log_since: Only return logs with sequence > this cursor.

    Returns:
        List of telemetry snapshots.

    Example::

        from auroraview.telemetry import get_all_snapshots

        snapshots = get_all_snapshots()
        for s in snapshots:
            print(f"{s['webview_id']}: {s['counters']['emit_count']} events")
    """
    from auroraview.core.mixins.telemetry import get_all_snapshots as _get_all

    return _get_all(include_logs=include_logs, log_since=log_since)


def get_snapshot(
    webview_id: str,
    include_logs: bool = False,
    log_since: int = 0,
) -> Optional[Dict[str, Any]]:
    """Get telemetry snapshot for a specific WebView instance.

    Args:
        webview_id: The WebView window ID.
        include_logs: If True, include recent log entries.
        log_since: Only return logs with sequence > this cursor.

    Returns:
        Telemetry snapshot dict, or None if not found.
    """
    from auroraview.core.mixins.telemetry import get_collector

    collector = get_collector(webview_id)
    if collector:
        return collector.snapshot(include_logs=include_logs, log_since=log_since)
    return None
