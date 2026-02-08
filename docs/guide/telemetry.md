# Telemetry

AuroraView Telemetry provides OpenTelemetry-based observability including logging, tracing, and metrics collection. It is designed to monitor WebView performance, app lifecycle, and error tracking across all AuroraView applications.

## Overview

| Feature | Description |
|---------|-------------|
| Logging | Structured log collection via `tracing` -> OpenTelemetry bridge |
| Traces | Distributed tracing for WebView operations (creation, navigation, IPC) |
| Metrics | Performance counters (load time, IPC latency, memory, JS eval time) |
| OTLP Export | Optional export to any OpenTelemetry-compatible backend |

## Quick Start

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

### With OTLP Export

```rust
use auroraview_telemetry::{Telemetry, TelemetryConfig};

let config = TelemetryConfig::with_otlp("http://localhost:4317");
let _guard = Telemetry::init(config).unwrap();
```

## Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `true` | Enable/disable telemetry globally |
| `service_name` | `String` | `"auroraview"` | Service name in telemetry data |
| `service_version` | `String` | crate version | Service version |
| `log_level` | `String` | `"info"` | Log level filter (e.g., `"debug"`, `"auroraview=debug,warn"`) |
| `log_to_stdout` | `bool` | `true` | Output logs to stdout |
| `log_json` | `bool` | `false` | Use JSON format for stdout logs |
| `otlp_endpoint` | `Option<String>` | `None` | OTLP gRPC endpoint (e.g., `"http://localhost:4317"`) |
| `metrics_enabled` | `bool` | `true` | Enable metrics collection |
| `metrics_interval_secs` | `u64` | `60` | Metrics export interval |
| `traces_enabled` | `bool` | `true` | Enable distributed tracing |
| `trace_sample_ratio` | `f64` | `1.0` | Trace sampling ratio (0.0-1.0) |

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

## Using WebViewMetrics

```rust
use auroraview_telemetry::WebViewMetrics;

let metrics = WebViewMetrics::new();

// Track WebView lifecycle
metrics.webview_created("main-window");
metrics.record_load_time("main-window", 180.0);
metrics.record_ipc_message("main-window", "js_to_rust");
metrics.record_ipc_latency("main-window", "js_to_rust", 5.2);

// Track errors
metrics.record_error("main-window", "timeout");

// Cleanup
metrics.webview_destroyed("main-window");
```

## Span Extensions

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

```rust
use auroraview_telemetry::Telemetry;

// Disable telemetry at runtime
Telemetry::disable();

// Re-enable
Telemetry::enable();

// Check status
if Telemetry::is_enabled() {
    // ...
}
```

## Architecture

```
Application Code
    │
    ▼
┌─────────────────────────────┐
│  tracing macros             │  (info!, debug!, span!)
│  (tracing crate)            │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│  tracing-opentelemetry      │  Bridge layer
│  (OpenTelemetryLayer)       │
└──────────┬──────────────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
┌────────┐  ┌──────────┐
│ Traces │  │ Metrics  │
│ (OTLP) │  │ (OTLP)   │
└────────┘  └──────────┘
    │             │
    ▼             ▼
  Jaeger      Prometheus
  Zipkin      Grafana
  etc.        etc.
```

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `otlp` | Yes | Enable OTLP gRPC export via tonic |
