# -*- coding: utf-8 -*-
"""Tests for auroraview.telemetry module."""


class TestTelemetryImport:
    """Test that telemetry module can be imported."""

    def test_import_module(self):
        from auroraview import telemetry

        assert telemetry is not None

    def test_import_public_api(self):
        from auroraview.telemetry import (
            TelemetryConfig,
            WebViewMetrics,
            disable,
            enable,
            init,
            is_enabled,
            record_error,
            record_ipc_message,
            record_load_time,
            shutdown,
        )

        assert init is not None
        assert shutdown is not None
        assert is_enabled is not None
        assert enable is not None
        assert disable is not None
        assert record_load_time is not None
        assert record_ipc_message is not None
        assert record_error is not None
        assert TelemetryConfig is not None
        assert WebViewMetrics is not None


class TestTelemetryConfig:
    """Test TelemetryConfig class."""

    def test_default_config(self):
        from auroraview.telemetry import TelemetryConfig

        config = TelemetryConfig()
        assert config.enabled is True
        assert config.service_name == "auroraview"
        assert config.log_level == "info"
        assert config.log_to_stdout is True
        assert config.log_json is False
        assert config.otlp_endpoint is None
        assert config.metrics_enabled is True
        assert config.metrics_interval_secs == 60
        assert config.traces_enabled is True
        assert abs(config.trace_sample_ratio - 1.0) < 1e-9

    def test_custom_config(self):
        from auroraview.telemetry import TelemetryConfig

        config = TelemetryConfig(
            enabled=True,
            service_name="my-app",
            log_level="debug",
            otlp_endpoint="http://localhost:4317",
            metrics_interval_secs=30,
            trace_sample_ratio=0.5,
        )
        assert config.service_name == "my-app"
        assert config.log_level == "debug"
        assert config.otlp_endpoint == "http://localhost:4317"
        assert config.metrics_interval_secs == 30
        assert abs(config.trace_sample_ratio - 0.5) < 1e-9

    def test_for_testing(self):
        from auroraview.telemetry import TelemetryConfig

        config = TelemetryConfig.for_testing()
        assert config.enabled is True
        assert config.service_name == "auroraview-test"
        assert config.log_level == "debug"
        assert config.metrics_interval_secs == 5

    def test_config_repr(self):
        from auroraview.telemetry import TelemetryConfig

        config = TelemetryConfig()
        r = repr(config)
        assert "TelemetryConfig" in r
        assert "auroraview" in r

    def test_config_setters(self):
        from auroraview.telemetry import TelemetryConfig

        config = TelemetryConfig()
        config.service_name = "changed"
        assert config.service_name == "changed"
        config.log_level = "warn"
        assert config.log_level == "warn"
        config.otlp_endpoint = "http://otel:4317"
        assert config.otlp_endpoint == "http://otel:4317"
        config.enabled = False
        assert config.enabled is False


class TestWebViewMetrics:
    """Test WebViewMetrics class."""

    def test_create_metrics(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        assert metrics is not None

    def test_webview_lifecycle(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.webview_created("test-window")
        metrics.webview_destroyed("test-window")

    def test_record_load_time(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_load_time("test-window", 250.0)

    def test_record_ipc(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_ipc_message("test-window", "js_to_rust")
        metrics.record_ipc_latency("test-window", "js_to_rust", 5.2)

    def test_record_js_eval(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_js_eval("test-window", 12.5)

    def test_record_error(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_error("test-window", "timeout")

    def test_record_navigation(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_navigation("test-window", "https://example.com")

    def test_record_event_emit(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_event_emit("test-window", "data_update")

    def test_record_memory(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        metrics.record_memory("test-window", 1024 * 1024)

    def test_repr(self):
        from auroraview.telemetry import WebViewMetrics

        metrics = WebViewMetrics()
        assert "WebViewMetrics" in repr(metrics)


class TestTelemetryFunctions:
    """Test module-level telemetry functions."""

    def test_convenience_record_load_time(self):
        from auroraview.telemetry import record_load_time

        record_load_time("test", 100.0)

    def test_convenience_record_ipc_message(self):
        from auroraview.telemetry import record_ipc_message

        record_ipc_message("test", "js_to_rust", 3.5)

    def test_convenience_record_error(self):
        from auroraview.telemetry import record_error

        record_error("test", "connection_timeout")
