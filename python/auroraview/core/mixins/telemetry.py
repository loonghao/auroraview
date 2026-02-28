# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Telemetry Mixin.

Provides automatic OpenTelemetry instrumentation for WebView operations.
When enabled, all lifecycle events, IPC calls, JS evaluations, and
navigation are automatically recorded as OTel metrics - no manual
``record_*()`` calls needed.

Also captures framework logs (Python logging) into a ring buffer for
real-time display in the TelemetryPanel / DevTools / MCP.
"""

from __future__ import annotations

import collections
import logging
import threading
import time
from typing import TYPE_CHECKING, Any, Dict, List, Optional, Tuple

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)

# Sentinel: telemetry Rust module available?
_telemetry = None
_AVAILABLE = False

try:
    from auroraview._core import telemetry as _telemetry

    _AVAILABLE = True
except ImportError:
    pass

# Maximum number of log entries kept per WebView instance
_LOG_RING_SIZE = 500
# Maximum number of metric samples kept per histogram
_METRIC_RING_SIZE = 2048


class _TelemetryLogHandler(logging.Handler):
    """Logging handler that fans out records to all active collectors."""

    def emit(self, record: logging.LogRecord) -> None:
        try:
            message = self.format(record)
            with _collectors_lock:
                collectors = list(_collectors.values())
            for collector in collectors:
                collector.record_log(
                    level=record.levelname,
                    logger_name=record.name,
                    message=message,
                    timestamp=record.created,
                )
        except Exception:
            # Keep logging pipeline resilient, but surface traceback in debug mode.
            if logging.raiseExceptions:
                self.handleError(record)


class _TelemetryCollector:
    """Internal collector that wraps Rust WebViewMetrics with an in-memory snapshot."""

    def __init__(self, webview_id: str) -> None:
        self.webview_id = webview_id
        self._lock = threading.Lock()

        # Rust OTel metrics (if available)
        self._metrics = None
        if _AVAILABLE:
            try:
                self._metrics = _telemetry.WebViewMetrics()
            except Exception:
                pass

        # In-memory counters for snapshot queries (DevTools / MCP)
        self._created_at = time.monotonic()
        self._counters: Dict[str, int] = {
            "emit_count": 0,
            "eval_js_count": 0,
            "navigation_count": 0,
            "ipc_call_count": 0,
            "error_count": 0,
        }
        self._histograms: Dict[str, collections.deque[float]] = {
            "load_times_ms": collections.deque(maxlen=_METRIC_RING_SIZE),
            "eval_js_times_ms": collections.deque(maxlen=_METRIC_RING_SIZE),
            "ipc_latencies_ms": collections.deque(maxlen=_METRIC_RING_SIZE),
        }

        self._last_url: Optional[str] = None
        self._last_error: Optional[str] = None

        # Log ring buffer: deque of (timestamp, level, logger_name, message)
        self._logs: collections.deque[Tuple[float, str, str, str]] = collections.deque(
            maxlen=_LOG_RING_SIZE,
        )
        # Monotonic sequence for cursor-based log fetching
        self._log_seq: int = 0

    # -----------------------------------------------------------------
    # Record methods (write to both Rust OTel and in-memory snapshot)
    # -----------------------------------------------------------------

    def webview_created(self) -> None:
        if self._metrics:
            self._metrics.webview_created(self.webview_id)

    def webview_destroyed(self) -> None:
        if self._metrics:
            self._metrics.webview_destroyed(self.webview_id)

    def record_load_time(self, duration_ms: float) -> None:
        with self._lock:
            self._histograms["load_times_ms"].append(duration_ms)
        if self._metrics:
            self._metrics.record_load_time(self.webview_id, duration_ms)

    def record_emit(self, event_name: str) -> None:
        with self._lock:
            self._counters["emit_count"] += 1
        if self._metrics:
            self._metrics.record_event_emit(self.webview_id, event_name)

    def record_eval_js(self, duration_ms: float) -> None:
        with self._lock:
            self._counters["eval_js_count"] += 1
            self._histograms["eval_js_times_ms"].append(duration_ms)
        if self._metrics:
            self._metrics.record_js_eval(self.webview_id, duration_ms)

    def record_navigation(self, url: str) -> None:
        with self._lock:
            self._counters["navigation_count"] += 1
            self._last_url = url
        if self._metrics:
            self._metrics.record_navigation(self.webview_id, url)

    def record_ipc_call(self, method: str, duration_ms: float) -> None:
        with self._lock:
            self._counters["ipc_call_count"] += 1
            self._histograms["ipc_latencies_ms"].append(duration_ms)
        if self._metrics:
            self._metrics.record_ipc_latency(self.webview_id, "js_to_python", duration_ms)
            self._metrics.record_ipc_message(self.webview_id, "js_to_python")

    def record_error(self, error_type: str) -> None:
        with self._lock:
            self._counters["error_count"] += 1
            self._last_error = error_type
        if self._metrics:
            self._metrics.record_error(self.webview_id, error_type)

    def record_memory(self, bytes_used: int) -> None:
        if self._metrics:
            self._metrics.record_memory(self.webview_id, bytes_used)

    def record_log(
        self,
        level: str,
        logger_name: str,
        message: str,
        timestamp: float,
    ) -> None:
        """Append a log entry to the ring buffer."""
        with self._lock:
            self._log_seq += 1
            self._logs.append((timestamp, level, logger_name, message))

    # -----------------------------------------------------------------
    # Snapshot (for DevTools / MCP consumption)
    # -----------------------------------------------------------------

    def snapshot(self, include_logs: bool = False, log_since: int = 0) -> Dict[str, Any]:
        """Return a JSON-serializable snapshot of current metrics.

        Args:
            include_logs: If True, include recent log entries in the snapshot.
            log_since: Only return logs with sequence > log_since (cursor).
        """
        with self._lock:
            uptime_s = time.monotonic() - self._created_at

            def _avg(vals: List[float]) -> Optional[float]:
                return round(sum(vals) / len(vals), 2) if vals else None

            def _p95(vals: List[float]) -> Optional[float]:
                if not vals:
                    return None
                s = sorted(vals)
                idx = int(len(s) * 0.95)
                return round(s[min(idx, len(s) - 1)], 2)

            load_times = list(self._histograms["load_times_ms"])
            eval_times = list(self._histograms["eval_js_times_ms"])
            ipc_latencies = list(self._histograms["ipc_latencies_ms"])

            result: Dict[str, Any] = {
                "webview_id": self.webview_id,
                "uptime_s": round(uptime_s, 2),
                "counters": dict(self._counters),
                "histograms": {
                    "load_time_avg_ms": _avg(load_times),
                    "load_time_p95_ms": _p95(load_times),
                    "eval_js_avg_ms": _avg(eval_times),
                    "eval_js_p95_ms": _p95(eval_times),
                    "ipc_latency_avg_ms": _avg(ipc_latencies),
                    "ipc_latency_p95_ms": _p95(ipc_latencies),
                },
                "last_url": self._last_url,
                "last_error": self._last_error,
                "otel_available": self._metrics is not None,
                "log_seq": self._log_seq,
            }

            if include_logs:
                # Return log entries as list of dicts
                # Apply cursor: only return entries added after log_since
                total = self._log_seq
                buf_len = len(self._logs)
                # The oldest entry in the ring has seq = total - buf_len + 1
                start_seq = total - buf_len + 1
                logs_out: List[Dict[str, Any]] = []
                for i, (ts, lvl, name, msg) in enumerate(self._logs):
                    entry_seq = start_seq + i
                    if entry_seq > log_since:
                        logs_out.append(
                            {"seq": entry_seq, "ts": ts, "level": lvl, "logger": name, "msg": msg}
                        )
                result["logs"] = logs_out

            return result


# Global registry: webview_id -> collector
_collectors: Dict[str, _TelemetryCollector] = {}
_collectors_lock = threading.Lock()

# Global log handler (attached to root logger once)
_global_log_handler: Optional[_TelemetryLogHandler] = None
_global_log_handler_lock = threading.Lock()


def get_collector(webview_id: str) -> Optional[_TelemetryCollector]:
    """Get the telemetry collector for a WebView instance."""
    with _collectors_lock:
        return _collectors.get(webview_id)


def get_all_snapshots(
    include_logs: bool = False,
    log_since: int = 0,
) -> List[Dict[str, Any]]:
    """Get snapshots of all active WebView instances."""
    with _collectors_lock:
        return [
            c.snapshot(include_logs=include_logs, log_since=log_since) for c in _collectors.values()
        ]


def _install_log_handler(collector: Optional[_TelemetryCollector] = None) -> None:
    """Attach a global telemetry logging handler to ``auroraview`` logger hierarchy.

    One handler is installed globally and fans out records to all active collectors.
    The ``collector`` argument is kept for backward compatibility and ignored.
    """
    del collector

    global _global_log_handler
    with _global_log_handler_lock:
        if _global_log_handler is not None:
            return  # already installed
        handler = _TelemetryLogHandler()
        handler.setFormatter(logging.Formatter("%(message)s"))
        # Capture all auroraview.* logs (including DEBUG when debug=True)
        handler.setLevel(logging.DEBUG)
        av_logger = logging.getLogger("auroraview")
        av_logger.addHandler(handler)
        _global_log_handler = handler


def _uninstall_log_handler() -> None:
    """Remove the telemetry log handler from the auroraview logger."""
    global _global_log_handler
    with _global_log_handler_lock:
        if _global_log_handler is None:
            return
        av_logger = logging.getLogger("auroraview")
        av_logger.removeHandler(_global_log_handler)
        _global_log_handler = None


class WebViewTelemetryMixin:
    """Mixin that auto-instruments WebView lifecycle with OpenTelemetry.

    When ``debug=True`` (default), this mixin automatically:
    - Records WebView create/destroy events
    - Times page loads
    - Counts and times eval_js calls
    - Counts emit() calls
    - Tracks navigation events
    - Times IPC (bind_call) handler invocations
    - Captures ``auroraview.*`` Python logs into a ring buffer
    - Exposes a snapshot for DevTools/MCP queries

    No manual ``telemetry.init()`` or ``record_*()`` needed.
    """

    # Type hints for attributes from main class
    _window_id: Optional[str]
    _debug: bool

    def _init_telemetry(self) -> None:
        """Initialize auto-telemetry for this WebView instance.

        Called automatically in ``__init__`` after WindowManager registration.
        """
        self._telemetry_collector: Optional[_TelemetryCollector] = None
        self._telemetry_nav_start: Optional[float] = None

        wid = getattr(self, "_window_id", None) or "unknown"
        debug = getattr(self, "_debug", True)

        if not debug:
            return

        collector = _TelemetryCollector(wid)
        self._telemetry_collector = collector

        with _collectors_lock:
            _collectors[wid] = collector

        collector.webview_created()

        # Record the initial URL/html passed to the constructor
        initial_url = getattr(self, "_stored_url", None)
        if initial_url:
            collector.record_navigation(initial_url)
            self._telemetry_nav_start = time.monotonic()

        # Install log handler (once globally) to capture auroraview.* logs
        _install_log_handler()

        logger.debug("Auto-telemetry enabled for WebView %s", wid)

    def _teardown_telemetry(self) -> None:
        """Cleanup telemetry on WebView close."""
        collector = getattr(self, "_telemetry_collector", None)
        if collector is None:
            return

        collector.webview_destroyed()

        with _collectors_lock:
            _collectors.pop(collector.webview_id, None)
            # If no more collectors, remove log handler
            if not _collectors:
                _uninstall_log_handler()

    # -----------------------------------------------------------------
    # Hook points (called by other mixins / WebView)
    # -----------------------------------------------------------------

    def _telemetry_on_emit(self, event_name: str) -> None:
        collector = getattr(self, "_telemetry_collector", None)
        if collector:
            collector.record_emit(event_name)

    def _telemetry_on_eval_js(self, duration_ms: float) -> None:
        collector = getattr(self, "_telemetry_collector", None)
        if collector:
            collector.record_eval_js(duration_ms)

    def _telemetry_on_navigate(self, url: str) -> None:
        collector = getattr(self, "_telemetry_collector", None)
        if collector:
            collector.record_navigation(url)
            self._telemetry_nav_start = time.monotonic()

    def _telemetry_on_page_loaded(self) -> None:
        collector = getattr(self, "_telemetry_collector", None)
        nav_start = getattr(self, "_telemetry_nav_start", None)
        if collector and nav_start is not None:
            duration_ms = (time.monotonic() - nav_start) * 1000.0
            collector.record_load_time(duration_ms)
            self._telemetry_nav_start = None

    def _telemetry_on_ipc_call(self, method: str, duration_ms: float) -> None:
        collector = getattr(self, "_telemetry_collector", None)
        if collector:
            collector.record_ipc_call(method, duration_ms)

    def _telemetry_on_error(self, error_type: str) -> None:
        collector = getattr(self, "_telemetry_collector", None)
        if collector:
            collector.record_error(error_type)

    # -----------------------------------------------------------------
    # Public query API
    # -----------------------------------------------------------------

    def get_telemetry_snapshot(
        self,
        include_logs: bool = False,
        log_since: int = 0,
    ) -> Optional[Dict[str, Any]]:
        """Get the current telemetry metrics snapshot.

        Args:
            include_logs: If True, include recent log entries.
            log_since: Only return logs with sequence > this cursor.

        Returns a dict with counters, histograms, uptime, logs, etc.
        Returns None if telemetry is not enabled.
        """
        collector = getattr(self, "_telemetry_collector", None)
        if collector:
            return collector.snapshot(include_logs=include_logs, log_since=log_since)
        return None
