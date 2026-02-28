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

_import_error = None
_telemetry_native_available = False
_telemetry_enabled = True


try:
    from auroraview._core import telemetry as _telemetry

    TelemetryConfig = _telemetry.TelemetryConfig
    WebViewMetrics = _telemetry.WebViewMetrics
    _telemetry_native_available = True
except ImportError as exc:
    _import_error = str(exc)

    _telemetry = None

    class TelemetryConfig(object):
        """Pure-Python fallback config when native telemetry is unavailable."""

        def __init__(
            self,
            enabled=True,
            service_name="auroraview",
            log_level="info",
            log_to_stdout=True,
            log_json=False,
            otlp_endpoint=None,
            metrics_enabled=True,
            metrics_interval_secs=60,
            traces_enabled=True,
            trace_sample_ratio=1.0,
        ):
            self.enabled = enabled
            self.service_name = service_name
            self.log_level = log_level
            self.log_to_stdout = log_to_stdout
            self.log_json = log_json
            self.otlp_endpoint = otlp_endpoint
            self.metrics_enabled = metrics_enabled
            self.metrics_interval_secs = metrics_interval_secs
            self.traces_enabled = traces_enabled
            self.trace_sample_ratio = trace_sample_ratio

        @classmethod
        def for_testing(cls):
            return cls(
                enabled=True,
                service_name="auroraview-test",
                log_level="debug",
                metrics_interval_secs=5,
            )

        def __repr__(self):
            return (
                "TelemetryConfig("
                "enabled={!r}, service_name={!r}, log_level={!r}, otlp_endpoint={!r}"
                ")"
            ).format(
                self.enabled,
                self.service_name,
                self.log_level,
                self.otlp_endpoint,
            )

    class WebViewMetrics(object):
        """Pure-Python no-op fallback metrics collector."""

        def webview_created(self, webview_id):
            del webview_id

        def webview_destroyed(self, webview_id):
            del webview_id

        def record_load_time(self, webview_id, duration_ms):
            del webview_id, duration_ms

        def record_ipc_message(self, webview_id, direction):
            del webview_id, direction

        def record_ipc_latency(self, webview_id, direction, latency_ms):
            del webview_id, direction, latency_ms

        def record_js_eval(self, webview_id, duration_ms):
            del webview_id, duration_ms

        def record_error(self, webview_id, error_type):
            del webview_id, error_type

        def record_navigation(self, webview_id, url):
            del webview_id, url

        def record_event_emit(self, webview_id, event_name):
            del webview_id, event_name

        def record_memory(self, webview_id, bytes_used):
            del webview_id, bytes_used

        def __repr__(self):
            return "WebViewMetrics(native_available=False)"


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
    global _telemetry_enabled

    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.init_telemetry(config)
        return

    if config is not None:
        _telemetry_enabled = bool(getattr(config, "enabled", True))
    else:
        _telemetry_enabled = True


def shutdown() -> None:
    """Shutdown the telemetry system, flushing all pending data."""
    global _telemetry_enabled

    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.shutdown_telemetry()
        return

    _telemetry_enabled = False


def is_enabled() -> bool:
    """Check if telemetry is currently enabled."""
    if _telemetry_native_available:
        assert _telemetry is not None
        return _telemetry.is_telemetry_enabled()

    return _telemetry_enabled


def enable() -> None:
    """Re-enable telemetry at runtime."""
    global _telemetry_enabled

    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.enable_telemetry()
        return

    _telemetry_enabled = True


def disable() -> None:
    """Disable telemetry at runtime (data stops being collected)."""
    global _telemetry_enabled

    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.disable_telemetry()
        return

    _telemetry_enabled = False


# =========================================================================
# Manual record helpers (for advanced usage)
# =========================================================================


def record_load_time(webview_id: str, duration_ms: float) -> None:
    """Record WebView page load time."""
    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.record_webview_load_time(webview_id, duration_ms)
        return

    del webview_id, duration_ms


def record_ipc_message(webview_id: str, direction: str, latency_ms: float) -> None:
    """Record an IPC message with latency."""
    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.record_ipc_message(webview_id, direction, latency_ms)
        return

    del webview_id, direction, latency_ms


def record_error(webview_id: str, error_type: str) -> None:
    """Record an error occurrence."""
    if _telemetry_native_available:
        assert _telemetry is not None
        _telemetry.record_telemetry_error(webview_id, error_type)
        return

    del webview_id, error_type


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
