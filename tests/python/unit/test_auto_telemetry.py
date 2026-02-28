# -*- coding: utf-8 -*-
"""Tests for auto-telemetry mixin and snapshot API."""

from __future__ import annotations

import logging
import time

import pytest

from auroraview.core.mixins.telemetry import (
    _LOG_RING_SIZE,
    WebViewTelemetryMixin,
    _collectors,
    _collectors_lock,
    _global_log_handler_lock,
    _install_log_handler,
    _TelemetryCollector,
    _TelemetryLogHandler,
    _uninstall_log_handler,
    get_all_snapshots,
    get_collector,
)


class TestTelemetryCollector:
    """Tests for _TelemetryCollector."""

    def test_creation(self):
        collector = _TelemetryCollector("test-wv-1")
        assert collector.webview_id == "test-wv-1"

    def test_record_emit(self):
        collector = _TelemetryCollector("test-wv-2")
        collector.record_emit("update")
        collector.record_emit("update")
        collector.record_emit("close")

        snap = collector.snapshot()
        assert snap["counters"]["emit_count"] == 3

    def test_record_eval_js(self):
        collector = _TelemetryCollector("test-wv-3")
        collector.record_eval_js(1.5)
        collector.record_eval_js(2.5)
        collector.record_eval_js(3.0)

        snap = collector.snapshot()
        assert snap["counters"]["eval_js_count"] == 3
        # avg = (1.5 + 2.5 + 3.0) / 3 = 2.33
        assert snap["histograms"]["eval_js_avg_ms"] == pytest.approx(2.33, abs=0.01)

    def test_record_navigation(self):
        collector = _TelemetryCollector("test-wv-4")
        collector.record_navigation("https://example.com")
        collector.record_navigation("https://example.com/page2")

        snap = collector.snapshot()
        assert snap["counters"]["navigation_count"] == 2
        assert snap["last_url"] == "https://example.com/page2"

    def test_record_ipc_call(self):
        collector = _TelemetryCollector("test-wv-5")
        collector.record_ipc_call("api.echo", 2.0)
        collector.record_ipc_call("api.list", 5.0)

        snap = collector.snapshot()
        assert snap["counters"]["ipc_call_count"] == 2
        assert snap["histograms"]["ipc_latency_avg_ms"] == pytest.approx(3.5, abs=0.01)

    def test_record_error(self):
        collector = _TelemetryCollector("test-wv-6")
        collector.record_error("RuntimeError")

        snap = collector.snapshot()
        assert snap["counters"]["error_count"] == 1
        assert snap["last_error"] == "RuntimeError"

    def test_snapshot_structure(self):
        collector = _TelemetryCollector("test-wv-7")
        snap = collector.snapshot()

        assert "webview_id" in snap
        assert "uptime_s" in snap
        assert "counters" in snap
        assert "histograms" in snap
        assert "last_url" in snap
        assert "last_error" in snap
        assert "otel_available" in snap
        assert "log_seq" in snap
        assert snap["log_seq"] == 0

        # Logs not included by default
        assert "logs" not in snap

        # All counters start at 0
        for v in snap["counters"].values():
            assert v == 0

    def test_uptime_increases(self):
        collector = _TelemetryCollector("test-wv-8")
        t1 = collector.snapshot()["uptime_s"]
        time.sleep(0.05)
        t2 = collector.snapshot()["uptime_s"]
        assert t2 > t1

    def test_p95_histogram(self):
        collector = _TelemetryCollector("test-wv-9")
        # 20 values: 1..20
        for i in range(1, 21):
            collector.record_eval_js(float(i))

        snap = collector.snapshot()
        # p95 of 1..20 -> idx = int(20 * 0.95) = 19 -> s[19] = 20
        assert snap["histograms"]["eval_js_p95_ms"] == 20.0

    def test_load_time(self):
        collector = _TelemetryCollector("test-wv-10")
        collector.record_load_time(150.0)
        collector.record_load_time(250.0)

        snap = collector.snapshot()
        assert snap["histograms"]["load_time_avg_ms"] == pytest.approx(200.0, abs=0.01)


class TestGlobalRegistry:
    """Tests for global collector registry."""

    def setup_method(self):
        # Clean up global registry before each test
        with _collectors_lock:
            _collectors.clear()

    def test_get_collector(self):
        collector = _TelemetryCollector("reg-1")
        with _collectors_lock:
            _collectors["reg-1"] = collector

        assert get_collector("reg-1") is collector
        assert get_collector("nonexistent") is None

    def test_get_all_snapshots(self):
        c1 = _TelemetryCollector("snap-1")
        c2 = _TelemetryCollector("snap-2")
        c1.record_emit("test")

        with _collectors_lock:
            _collectors["snap-1"] = c1
            _collectors["snap-2"] = c2

        snapshots = get_all_snapshots()
        assert len(snapshots) == 2
        ids = {s["webview_id"] for s in snapshots}
        assert ids == {"snap-1", "snap-2"}


class TestWebViewTelemetryMixin:
    """Tests for WebViewTelemetryMixin methods."""

    def setup_method(self):
        with _collectors_lock:
            _collectors.clear()

    def test_init_telemetry_debug_true(self):
        mixin = WebViewTelemetryMixin()
        mixin._window_id = "mixin-1"
        mixin._debug = True
        mixin._init_telemetry()

        assert mixin._telemetry_collector is not None
        assert get_collector("mixin-1") is not None

    def test_init_telemetry_debug_false(self):
        mixin = WebViewTelemetryMixin()
        mixin._window_id = "mixin-2"
        mixin._debug = False
        mixin._init_telemetry()

        assert mixin._telemetry_collector is None
        assert get_collector("mixin-2") is None

    def test_teardown_telemetry(self):
        mixin = WebViewTelemetryMixin()
        mixin._window_id = "mixin-3"
        mixin._debug = True
        mixin._init_telemetry()

        assert get_collector("mixin-3") is not None

        mixin._teardown_telemetry()
        assert get_collector("mixin-3") is None

    def test_hook_methods(self):
        mixin = WebViewTelemetryMixin()
        mixin._window_id = "mixin-4"
        mixin._debug = True
        mixin._init_telemetry()

        mixin._telemetry_on_emit("test_event")
        mixin._telemetry_on_eval_js(2.5)
        mixin._telemetry_on_navigate("https://test.com")
        mixin._telemetry_on_ipc_call("api.echo", 3.0)
        mixin._telemetry_on_error("TestError")

        snap = mixin.get_telemetry_snapshot()
        assert snap is not None
        assert snap["counters"]["emit_count"] == 1
        assert snap["counters"]["eval_js_count"] == 1
        assert snap["counters"]["navigation_count"] == 1
        assert snap["counters"]["ipc_call_count"] == 1
        assert snap["counters"]["error_count"] == 1

    def test_page_load_timing(self):
        mixin = WebViewTelemetryMixin()
        mixin._window_id = "mixin-5"
        mixin._debug = True
        mixin._init_telemetry()

        # Simulate navigation + page load
        mixin._telemetry_on_navigate("https://example.com")
        time.sleep(0.05)  # Simulate load time
        mixin._telemetry_on_page_loaded()

        snap = mixin.get_telemetry_snapshot()
        assert snap is not None
        assert snap["histograms"]["load_time_avg_ms"] is not None
        assert snap["histograms"]["load_time_avg_ms"] >= 40  # At least 40ms

    def test_get_telemetry_snapshot_disabled(self):
        mixin = WebViewTelemetryMixin()
        mixin._window_id = "mixin-6"
        mixin._debug = False
        mixin._init_telemetry()

        assert mixin.get_telemetry_snapshot() is None


class TestTelemetryModuleAPI:
    """Tests for auroraview.telemetry module-level functions."""

    def setup_method(self):
        with _collectors_lock:
            _collectors.clear()

    def test_get_all_snapshots_import(self):
        from auroraview.telemetry import get_all_snapshots

        # Should work even with no instances
        snapshots = get_all_snapshots()
        assert isinstance(snapshots, list)
        assert len(snapshots) == 0

    def test_get_snapshot_import(self):
        from auroraview.telemetry import get_snapshot

        assert get_snapshot("nonexistent") is None

    def test_get_snapshot_with_data(self):
        from auroraview.telemetry import get_snapshot

        collector = _TelemetryCollector("api-test-1")
        collector.record_emit("test")

        with _collectors_lock:
            _collectors["api-test-1"] = collector

        snap = get_snapshot("api-test-1")
        assert snap is not None
        assert snap["counters"]["emit_count"] == 1


class TestLogRingBuffer:
    """Tests for log ring buffer in _TelemetryCollector."""

    def test_record_log(self):
        collector = _TelemetryCollector("log-1")
        collector.record_log(
            level="INFO",
            logger_name="auroraview.test",
            message="hello",
            timestamp=1000.0,
        )

        assert collector._log_seq == 1
        assert len(collector._logs) == 1

    def test_log_seq_increases(self):
        collector = _TelemetryCollector("log-2")
        for i in range(5):
            collector.record_log(
                level="DEBUG",
                logger_name="auroraview.test",
                message="msg {}".format(i),
                timestamp=1000.0 + i,
            )

        assert collector._log_seq == 5
        assert len(collector._logs) == 5

    def test_ring_buffer_overflow(self):
        collector = _TelemetryCollector("log-3")
        for i in range(_LOG_RING_SIZE + 100):
            collector.record_log(
                level="INFO",
                logger_name="test",
                message="msg {}".format(i),
                timestamp=float(i),
            )

        assert len(collector._logs) == _LOG_RING_SIZE
        assert collector._log_seq == _LOG_RING_SIZE + 100

    def test_snapshot_with_logs(self):
        collector = _TelemetryCollector("log-4")
        collector.record_log(
            level="WARNING",
            logger_name="auroraview.core",
            message="test warning",
            timestamp=12345.678,
        )

        snap = collector.snapshot(include_logs=True)
        assert "logs" in snap
        assert len(snap["logs"]) == 1
        entry = snap["logs"][0]
        assert entry["seq"] == 1
        assert entry["level"] == "WARNING"
        assert entry["logger"] == "auroraview.core"
        assert entry["msg"] == "test warning"
        assert entry["ts"] == 12345.678

    def test_snapshot_without_logs(self):
        collector = _TelemetryCollector("log-5")
        collector.record_log(level="INFO", logger_name="test", message="msg", timestamp=0.0)

        snap = collector.snapshot(include_logs=False)
        assert "logs" not in snap
        assert snap["log_seq"] == 1

    def test_cursor_based_fetching(self):
        collector = _TelemetryCollector("log-6")
        for i in range(10):
            collector.record_log(
                level="INFO",
                logger_name="test",
                message="msg {}".format(i),
                timestamp=float(i),
            )

        # Get all logs
        snap1 = collector.snapshot(include_logs=True, log_since=0)
        assert len(snap1["logs"]) == 10

        # Get only logs after seq 5
        snap2 = collector.snapshot(include_logs=True, log_since=5)
        assert len(snap2["logs"]) == 5
        assert snap2["logs"][0]["seq"] == 6

        # Get only logs after the latest
        snap3 = collector.snapshot(include_logs=True, log_since=10)
        assert len(snap3["logs"]) == 0

    def test_cursor_with_ring_overflow(self):
        collector = _TelemetryCollector("log-7")
        total = _LOG_RING_SIZE + 50
        for i in range(total):
            collector.record_log(
                level="DEBUG",
                logger_name="test",
                message="msg {}".format(i),
                timestamp=float(i),
            )

        # Oldest entry seq = total - _LOG_RING_SIZE + 1 = 51
        snap = collector.snapshot(include_logs=True, log_since=0)
        assert len(snap["logs"]) == _LOG_RING_SIZE
        assert snap["logs"][0]["seq"] == 51

        # Get entries after seq 500
        snap2 = collector.snapshot(include_logs=True, log_since=500)
        expected = total - 500
        assert len(snap2["logs"]) == expected


class TestTelemetryLogHandler:
    """Tests for _TelemetryLogHandler."""

    def test_handler_writes_to_collector(self):
        collector = _TelemetryCollector("handler-1")
        with _collectors_lock:
            _collectors[collector.webview_id] = collector

        handler = _TelemetryLogHandler()
        handler.setFormatter(logging.Formatter("%(message)s"))

        record = logging.LogRecord(
            name="auroraview.test",
            level=logging.INFO,
            pathname="",
            lineno=0,
            msg="test message",
            args=None,
            exc_info=None,
        )
        handler.emit(record)

        assert collector._log_seq == 1
        assert len(collector._logs) == 1

        with _collectors_lock:
            _collectors.clear()

    def test_handler_does_not_raise(self):
        handler = _TelemetryLogHandler()

        record = logging.LogRecord(
            name="auroraview.test",
            level=logging.INFO,
            pathname="",
            lineno=0,
            msg="ok",
            args=None,
            exc_info=None,
        )
        handler.emit(record)


class TestLogHandlerInstallation:
    """Tests for global log handler install/uninstall."""

    def setup_method(self):
        _uninstall_log_handler()
        with _collectors_lock:
            _collectors.clear()

    def teardown_method(self):
        _uninstall_log_handler()
        with _collectors_lock:
            _collectors.clear()

    def test_install_and_uninstall(self):
        collector = _TelemetryCollector("install-1")
        _install_log_handler(collector)

        with _global_log_handler_lock:
            from auroraview.core.mixins.telemetry import (
                _global_log_handler as handler,
            )

            assert handler is not None

        _uninstall_log_handler()

        with _global_log_handler_lock:
            from auroraview.core.mixins.telemetry import (
                _global_log_handler as handler2,
            )

            assert handler2 is None

    def test_install_only_once(self):
        c1 = _TelemetryCollector("install-2a")
        c2 = _TelemetryCollector("install-2b")
        _install_log_handler(c1)

        with _global_log_handler_lock:
            from auroraview.core.mixins.telemetry import (
                _global_log_handler as handler1,
            )

        _install_log_handler(c2)

        with _global_log_handler_lock:
            from auroraview.core.mixins.telemetry import (
                _global_log_handler as handler2,
            )

            assert handler1 is handler2

        _uninstall_log_handler()

    def test_logging_captured(self):
        collector = _TelemetryCollector("install-3")
        with _collectors_lock:
            _collectors[collector.webview_id] = collector
        _install_log_handler(collector)

        test_logger = logging.getLogger("auroraview.test_capture")

        test_logger.setLevel(logging.DEBUG)
        test_logger.info("captured message")

        assert collector._log_seq >= 1
        snap = collector.snapshot(include_logs=True)
        messages = [entry["msg"] for entry in snap["logs"]]
        assert any("captured message" in m for m in messages)

        _uninstall_log_handler()
