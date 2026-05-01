//! Benchmarks for `auroraview-ue`.
//!
//! Run with: `cargo bench -p auroraview-ue`

use auroraview_ue::{UeIntegration, UeWebViewConfig, UeEmbedMode};
use criterion::{black_box, Criterion};

/// Benchmark `UeIntegration::new()` (initialization).
fn bench_integration_new(c: &mut Criterion) {
    c.bench_function("ue_integration_new", |b| {
        b.iter(|| {
            let _integration = UeIntegration::new(UeWebViewConfig::default());
            black_box(_integration);
        });
    });
}

/// Benchmark `UeIntegration::create_webview()` (placeholder).
fn bench_create_webview(c: &mut Criterion) {
    let integration = UeIntegration::new(UeWebViewConfig::default());
    
    c.bench_function("ue_create_webview", |b| {
        b.iter(|| {
            let _result = integration.create_webview("https://example.com");
            black_box(_result);
        });
    });
}

/// Benchmark `UeGameThreadExecutor::new()` (channel creation).
fn bench_executor_new(c: &mut Criterion) {
    c.bench_function("ue_executor_new", |b| {
        b.iter(|| {
            let (_executor, _rx) = auroraview_ue::UeGameThreadExecutor::new();
            black_box((_executor, _rx));
        });
    });
}

/// Benchmark `UeWebViewConfig::default()` (config creation).
fn bench_config_default(c: &mut Criterion) {
    c.bench_function("ue_config_default", |b| {
        b.iter(|| {
            let _config = UeWebViewConfig::default();
            black_box(_config);
        });
    });
}

/// Benchmark `UeWebViewConfig` with custom values.
fn bench_config_custom(c: &mut Criterion) {
    c.bench_function("ue_config_custom", |b| {
        b.iter(|| {
            let config = UeWebViewConfig {
                initial_size: (1920, 1080),
                embed_mode: UeEmbedMode::NativeChildWindow,
                dev_tools: true,
                init_script: Some("console.log('hello')".to_string()),
            };
            black_box(config);
        });
    });
}

/// Benchmark `SlateWidgetHandle` creation.
fn bench_slate_handle(c: &mut Criterion) {
    c.bench_function("ue_slate_handle", |b| {
        b.iter(|| {
            let _handle = auroraview_ue::SlateWidgetHandle::null();
            black_box(_handle);
        });
    });
}

criterion_group!(
    benches,
    bench_integration_new,
    bench_create_webview,
    bench_executor_new,
    bench_config_default,
    bench_config_custom,
    bench_slate_handle
);

criterion_main!(benches);
