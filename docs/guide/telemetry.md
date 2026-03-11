# Telemetry

AuroraView Telemetry provides OpenTelemetry-based observability including logging, tracing, and metrics collection. It is designed to monitor WebView performance, app lifecycle, and error tracking across all AuroraView applications.

## Overview

| Feature | Description |
|---------|-------------|
| Logging | Structured log collection via `tracing` -> OpenTelemetry bridge |
| Traces | Distributed tracing for WebView operations (creation, navigation, IPC) |
| Metrics | Performance counters (load time, IPC latency, memory, JS eval time) |
| OTLP Export | Optional export to any OpenTelemetry-compatible backend |
| Sentry Export | Optional Rust-native Sentry SDK export (no Python runtime dependency) |


## Quick Start

### Python

```python
from auroraview.telemetry import init, shutdown, TelemetryConfig, WebViewMetrics

# Initialize with defaults (stdout, info level)
init()

# Or with custom config
config = TelemetryConfig(log_level="debug", service_name="my-app")
init(config)

# Record metrics
metrics = WebViewMetrics()
metrics.webview_created("main-window")
metrics.record_load_time("main-window", 180.0)
metrics.record_ipc_latency("main-window", "js_to_rust", 5.2)

# Cleanup
shutdown()
```

### Python with OTLP Export (Jaeger / Grafana)

```python
from auroraview.telemetry import init, TelemetryConfig

config = TelemetryConfig(
    otlp_endpoint="http://localhost:4317",
    log_level="debug",
    traces_enabled=True,
    metrics_enabled=True,
)
init(config)
```

### Rust

```rust
use auroraview_telemetry::{Telemetry, TelemetryConfig};

// Initialize with default settings (stdout, enabled by default)
let _guard = Telemetry::init(TelemetryConfig::default()).unwrap();

// Use tracing macros as normal
tracing::info!(webview_id = "main", "WebView created");

// Record metrics
auroraview_telemetry::metrics_api::record_webview_load_time("main", 250.0);
```

### Rust with OTLP Export

```rust
use auroraview_telemetry::{Telemetry, TelemetryConfig};

let config = TelemetryConfig::with_otlp("http://localhost:4317");
let _guard = Telemetry::init(config).unwrap();
```

### Rust with Sentry Export

```rust
use auroraview_telemetry::{Telemetry, TelemetryConfig};

let config = TelemetryConfig {
    sentry_dsn: Some("https://public@example.com/1".to_string()),
    sentry_environment: Some("production".to_string()),
    sentry_release: Some("auroraview@0.4.9".to_string()),
    sentry_sample_rate: 1.0,
    sentry_traces_sample_rate: 0.2,
    ..TelemetryConfig::default()
};

let _guard = Telemetry::init(config).unwrap();
let _captured = Telemetry::capture_sentry_message("webview init failed", "error");
```


## Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `true` | Enable/disable telemetry globally |
| `service_name` | `str` | `"auroraview"` | Service name in telemetry data |
| `service_version` | `str` | crate version | Service version |
| `log_level` | `str` | `"info"` | Log level filter (e.g., `"debug"`, `"auroraview=debug,warn"`) |
| `log_to_stdout` | `bool` | `true` | Output logs to stdout |
| `log_json` | `bool` | `false` | Use JSON format for stdout logs |
| `otlp_endpoint` | `str \| None` | `None` | OTLP gRPC endpoint (e.g., `"http://localhost:4317"`) |
| `metrics_enabled` | `bool` | `true` | Enable metrics collection |
| `metrics_interval_secs` | `int` | `60` | Metrics export interval |
| `traces_enabled` | `bool` | `true` | Enable distributed tracing |
| `trace_sample_ratio` | `float` | `1.0` | Trace sampling ratio (0.0-1.0) |
| `sentry_dsn` | `str \| None` | `None` | Sentry DSN for Rust-native export |
| `sentry_environment` | `str \| None` | `None` | Sentry environment tag |
| `sentry_release` | `str \| None` | `None` | Sentry release version |
| `sentry_sample_rate` | `float` | `1.0` | Sentry event sampling ratio (0.0-1.0) |
| `sentry_traces_sample_rate` | `float` | `0.0` | Sentry transaction sampling ratio (0.0-1.0) |


## Built-in Metrics

| Metric | Type | Unit | Description |
|--------|------|------|-------------|
| `auroraview.webview.count` | UpDownCounter | - | Active WebView instances |
| `auroraview.webview.load_time` | Histogram | ms | Page load time |
| `auroraview.ipc.latency` | Histogram | ms | IPC round-trip latency |
| `auroraview.ipc.messages` | Counter | - | Total IPC messages |
| `auroraview.js.eval_count` | Counter | - | JS evaluations |
| `auroraview.js.eval_duration` | Histogram | ms | JS eval duration |
| `auroraview.errors` | Counter | - | Total errors |
| `auroraview.navigation.count` | Counter | - | Navigation events |
| `auroraview.events.emitted` | Counter | - | Events emitted (Python -> JS) |
| `auroraview.memory.bytes` | Gauge | bytes | Memory usage |

## Python API

### Module Functions

```python
from auroraview.telemetry import (
    init, shutdown, is_enabled, enable, disable,
    record_load_time, record_ipc_message, record_error, capture_sentry_message,
)


# Initialize / shutdown
init()                  # default config
init(config)            # custom config
shutdown()              # flush & shutdown

# Runtime control
is_enabled()            # -> bool
enable()
disable()

# Convenience metric recording
record_load_time("webview-id", 250.0)
record_ipc_message("webview-id", "js_to_rust", 5.2)
record_error("webview-id", "timeout")
capture_sentry_message("webview failed", level="error")

```

### WebViewMetrics

```python
from auroraview.telemetry import WebViewMetrics

metrics = WebViewMetrics()

# Track WebView lifecycle
metrics.webview_created("main-window")
metrics.record_load_time("main-window", 180.0)
metrics.record_ipc_message("main-window", "js_to_rust")
metrics.record_ipc_latency("main-window", "js_to_rust", 5.2)
metrics.record_js_eval("main-window", 12.5)
metrics.record_navigation("main-window", "https://example.com")
metrics.record_event_emit("main-window", "data_update")
metrics.record_memory("main-window", 1024 * 1024)

# Track errors
metrics.record_error("main-window", "timeout")

# Cleanup
metrics.webview_destroyed("main-window")
```

### TelemetryConfig

```python
from auroraview.telemetry import TelemetryConfig

# Default config
config = TelemetryConfig()

# Custom config
config = TelemetryConfig(
    enabled=True,
    service_name="my-app",
    log_level="debug",
    otlp_endpoint="http://localhost:4317",
    metrics_interval_secs=30,
    trace_sample_ratio=0.5,
    sentry_dsn="https://public@example.com/1",
    sentry_environment="development",
    sentry_release="auroraview@0.4.9",
    sentry_sample_rate=1.0,
    sentry_traces_sample_rate=0.2,
)


# Testing config (stdout, debug level)
config = TelemetryConfig.for_testing()

# Modify after creation
config.log_level = "warn"
config.otlp_endpoint = None
```

## Rust Span Extensions

Add AuroraView-specific attributes to tracing spans:

```rust
use auroraview_telemetry::SpanExt;

let span = tracing::info_span!("webview_navigate");
let _guard = span.enter();
span.set_webview_id("main-window");
span.set_operation("navigate");
span.set_app_name("my-gallery-app");
```

## Runtime Control

### Python

```python
from auroraview.telemetry import is_enabled, enable, disable

disable()        # pause collection
assert not is_enabled()

enable()         # resume collection
assert is_enabled()
```

### Rust

```rust
use auroraview_telemetry::Telemetry;

Telemetry::disable();
Telemetry::enable();
if Telemetry::is_enabled() {
    // ...
}
```

## Architecture

```
Python / Rust Application Code
    |
    v
+-----------------------------+
|  tracing macros             |  (info!, debug!, span!)
|  (tracing crate)            |
+-------------+---------------+
              |
              v
+-----------------------------+
|  tracing-opentelemetry      |  Bridge layer
|  (OpenTelemetryLayer)       |
+-------------+---------------+
              |
       +------+------+
       v              v
+----------+  +----------+
|  Traces  |  |  Metrics |
|  (OTLP)  |  |  (OTLP)  |
+----------+  +----------+
       |              |
       v              v
   Jaeger         Prometheus
   Zipkin         Grafana
   etc.           etc.
```

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `otlp` | Yes | Enable OTLP gRPC export via tonic |
| `python` | No | Enable Python bindings via PyO3 |
| `sentry` | No | Enable Rust-native Sentry SDK integration |
