# 遥测

AuroraView Telemetry 提供基于 OpenTelemetry 的可观测性功能，包括日志、链路追踪和指标收集。它用于监控所有 AuroraView 应用的 WebView 性能、应用生命周期和错误追踪。

## 概述

| 功能 | 描述 |
|------|------|
| 日志 | 通过 `tracing` -> OpenTelemetry 桥接的结构化日志收集 |
| 链路追踪 | WebView 操作的分布式追踪（创建、导航、IPC） |
| 指标 | 性能计数器（加载时间、IPC 延迟、内存、JS 执行时间） |
| OTLP 导出 | 可选导出到任何兼容 OpenTelemetry 的后端 |

## 快速开始

### Rust

```rust
use auroraview_telemetry::{Telemetry, TelemetryConfig};

// 使用默认配置初始化（stdout 输出，默认启用）
let _guard = Telemetry::init(TelemetryConfig::default()).unwrap();

// 正常使用 tracing 宏
tracing::info!(webview_id = "main", "WebView created");

// 记录指标
auroraview_telemetry::metrics_api::record_webview_load_time("main", 250.0);
```

### 使用 OTLP 导出

```rust
use auroraview_telemetry::{Telemetry, TelemetryConfig};

let config = TelemetryConfig::with_otlp("http://localhost:4317");
let _guard = Telemetry::init(config).unwrap();
```

## 配置

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `enabled` | `bool` | `true` | 全局启用/禁用遥测 |
| `service_name` | `String` | `"auroraview"` | 遥测数据中的服务名 |
| `service_version` | `String` | crate 版本 | 服务版本 |
| `log_level` | `String` | `"info"` | 日志级别过滤（如 `"debug"`、`"auroraview=debug,warn"`） |
| `log_to_stdout` | `bool` | `true` | 输出日志到 stdout |
| `log_json` | `bool` | `false` | 使用 JSON 格式输出日志 |
| `otlp_endpoint` | `Option<String>` | `None` | OTLP gRPC 端点（如 `"http://localhost:4317"`） |
| `metrics_enabled` | `bool` | `true` | 启用指标收集 |
| `metrics_interval_secs` | `u64` | `60` | 指标导出间隔（秒） |
| `traces_enabled` | `bool` | `true` | 启用分布式追踪 |
| `trace_sample_ratio` | `f64` | `1.0` | 追踪采样率（0.0-1.0） |

## 内置指标

| 指标 | 类型 | 单位 | 描述 |
|------|------|------|------|
| `auroraview.webview.count` | UpDownCounter | - | 活跃的 WebView 实例数 |
| `auroraview.webview.load_time` | Histogram | ms | 页面加载时间 |
| `auroraview.ipc.latency` | Histogram | ms | IPC 往返延迟 |
| `auroraview.ipc.messages` | Counter | - | IPC 消息总数 |
| `auroraview.js.eval_count` | Counter | - | JS 执行次数 |
| `auroraview.js.eval_duration` | Histogram | ms | JS 执行时长 |
| `auroraview.errors` | Counter | - | 错误总数 |
| `auroraview.navigation.count` | Counter | - | 导航事件 |
| `auroraview.events.emitted` | Counter | - | 事件推送（Python -> JS） |
| `auroraview.memory.bytes` | Gauge | bytes | 内存使用量 |

## 使用 WebViewMetrics

```rust
use auroraview_telemetry::WebViewMetrics;

let metrics = WebViewMetrics::new();

// 跟踪 WebView 生命周期
metrics.webview_created("main-window");
metrics.record_load_time("main-window", 180.0);
metrics.record_ipc_message("main-window", "js_to_rust");
metrics.record_ipc_latency("main-window", "js_to_rust", 5.2);

// 跟踪错误
metrics.record_error("main-window", "timeout");

// 清理
metrics.webview_destroyed("main-window");
```

## Span 扩展

为 tracing span 添加 AuroraView 特定属性：

```rust
use auroraview_telemetry::SpanExt;

let span = tracing::info_span!("webview_navigate");
let _guard = span.enter();
span.set_webview_id("main-window");
span.set_operation("navigate");
span.set_app_name("my-gallery-app");
```

## 运行时控制

```rust
use auroraview_telemetry::Telemetry;

// 运行时禁用遥测
Telemetry::disable();

// 重新启用
Telemetry::enable();

// 检查状态
if Telemetry::is_enabled() {
    // ...
}
```

## 架构

```
应用代码
    │
    ▼
┌─────────────────────────────┐
│  tracing 宏                 │  (info!, debug!, span!)
│  (tracing crate)            │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│  tracing-opentelemetry      │  桥接层
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
  等          等
```

## Cargo Features

| Feature | 默认 | 描述 |
|---------|------|------|
| `otlp` | 是 | 通过 tonic 启用 OTLP gRPC 导出 |
