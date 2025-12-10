//! IPC performance benchmarks
//!
//! Run with: `cargo bench --bench ipc_bench`
//!
//! This benchmark suite measures the performance of various IPC operations
//! including JSON serialization/deserialization, message queue throughput,
//! and string escaping for JavaScript injection.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use std::hint::black_box;

/// Benchmark JSON serialization (common IPC operation)
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    // Small payload (~50 bytes)
    let small_data = json!({"event": "click", "x": 100, "y": 200});
    group.throughput(Throughput::Elements(1));
    group.bench_function("small_50b", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&small_data)).unwrap();
            black_box(s)
        })
    });

    // Medium payload (~2KB, typical DCC data)
    let medium_data = json!({
        "event": "selection_changed",
        "objects": (0..100).map(|i| format!("object_{}", i)).collect::<Vec<_>>(),
        "transform": {
            "position": [1.0, 2.0, 3.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        }
    });
    group.bench_function("medium_2kb", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&medium_data)).unwrap();
            black_box(s)
        })
    });

    // Large payload (~100KB, scene hierarchy)
    let large_data = json!({
        "event": "scene_update",
        "nodes": (0..1000).map(|i| json!({
            "id": i,
            "name": format!("node_{}", i),
            "parent": if i > 0 { Some(i / 10) } else { None },
            "transform": [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        })).collect::<Vec<_>>()
    });
    group.bench_function("large_100kb", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&large_data)).unwrap();
            black_box(s)
        })
    });

    // Very large payload (~1MB, bulk data export)
    let very_large_data = json!({
        "event": "bulk_export",
        "vertices": (0..10000).map(|i| [i as f64 * 0.1, i as f64 * 0.2, i as f64 * 0.3]).collect::<Vec<_>>(),
        "indices": (0..30000).collect::<Vec<_>>()
    });
    group.bench_function("very_large_1mb", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&very_large_data)).unwrap();
            black_box(s)
        })
    });

    group.finish();
}

/// Benchmark JSON deserialization
fn bench_json_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_deserialization");

    let small_json = r#"{"event":"click","x":100,"y":200}"#;
    group.throughput(Throughput::Bytes(small_json.len() as u64));
    group.bench_function("small_50b", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(black_box(small_json)).unwrap();
            black_box(v)
        })
    });

    let medium_json = serde_json::to_string(&json!({
        "event": "selection_changed",
        "objects": (0..100).map(|i| format!("object_{}", i)).collect::<Vec<_>>()
    }))
    .unwrap();
    group.throughput(Throughput::Bytes(medium_json.len() as u64));
    group.bench_function("medium_2kb", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(black_box(&medium_json)).unwrap();
            black_box(v)
        })
    });

    let large_json = serde_json::to_string(&json!({
        "event": "scene_update",
        "nodes": (0..1000).map(|i| json!({
            "id": i,
            "name": format!("node_{}", i),
            "parent": if i > 0 { Some(i / 10) } else { None },
            "transform": [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        })).collect::<Vec<_>>()
    }))
    .unwrap();
    group.throughput(Throughput::Bytes(large_json.len() as u64));
    group.bench_function("large_100kb", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(black_box(&large_json)).unwrap();
            black_box(v)
        })
    });

    group.finish();
}

/// Benchmark string escaping for JavaScript injection
/// This is a critical path in IPC when sending events to WebView
fn bench_js_string_escape(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_string_escape");

    // Test various payload sizes
    let payloads = vec![
        ("small_50b", r#"{"event":"click","x":100}"#.to_string()),
        (
            "medium_with_special",
            r#"{"event":"log","message":"Line1\nLine2\tTab\\Path'Quote"}"#.to_string(),
        ),
        (
            "large_clean",
            serde_json::to_string(&json!({
                "data": (0..1000).map(|i| format!("item_{}", i)).collect::<Vec<_>>()
            }))
            .unwrap(),
        ),
        (
            "large_special_chars",
            serde_json::to_string(&json!({
                "logs": (0..100).map(|i| format!("Log line {}\n\twith tabs\\ and 'quotes'", i)).collect::<Vec<_>>()
            }))
            .unwrap(),
        ),
    ];

    for (name, payload) in payloads {
        group.throughput(Throughput::Bytes(payload.len() as u64));
        group.bench_with_input(BenchmarkId::new("replace_chain", name), &payload, |b, p| {
            b.iter(|| {
                // Current implementation: chained replace
                let escaped = p.replace('\\', "\\\\").replace('\'', "\\'");
                black_box(escaped)
            })
        });

        group.bench_with_input(BenchmarkId::new("single_pass", name), &payload, |b, p| {
            b.iter(|| {
                // Alternative: single pass escape
                let escaped = escape_for_js_single_pass(p);
                black_box(escaped)
            })
        });
    }

    group.finish();
}

/// Single-pass JavaScript string escaping (potential optimization)
fn escape_for_js_single_pass(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + s.len() / 10);
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\'' => result.push_str("\\'"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Benchmark message formatting for WebView injection
fn bench_event_script_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_script_generation");

    let event_name = "test_event";
    let payloads = vec![
        ("small", json!({"x": 100, "y": 200})),
        (
            "medium",
            json!({"items": (0..100).map(|i| format!("item_{}", i)).collect::<Vec<_>>()}),
        ),
        (
            "large",
            json!({"nodes": (0..1000).map(|i| json!({"id": i, "name": format!("node_{}", i)})).collect::<Vec<_>>()}),
        ),
    ];

    for (name, data) in payloads {
        let json_str = serde_json::to_string(&data).unwrap();
        group.throughput(Throughput::Bytes(json_str.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("format_dispatch_event", name),
            &json_str,
            |b, json| {
                b.iter(|| {
                    let escaped = json.replace('\\', "\\\\").replace('\'', "\\'");
                    let script = format!(
                        "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}));",
                        event_name, escaped
                    );
                    black_box(script)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_auroraview_trigger", name),
            &json_str,
            |b, json| {
                b.iter(|| {
                    let escaped = json.replace('\\', "\\\\").replace('\'', "\\'");
                    let script = format!(
                        "(function(){{if(window.auroraview&&window.auroraview.trigger){{window.auroraview.trigger('{}',JSON.parse('{}'));}}else{{console.error('[AuroraView] Event bridge not ready');}}}})();",
                        event_name, escaped
                    );
                    black_box(script)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark batch message processing simulation
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    // Simulate batch sizes
    let batch_sizes = [1, 10, 50, 100, 500];

    for size in batch_sizes {
        let messages: Vec<serde_json::Value> = (0..size)
            .map(|i| {
                json!({
                    "id": format!("msg_{}", i),
                    "event": "update",
                    "data": {"index": i, "value": i * 2}
                })
            })
            .collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("serialize_batch", size),
            &messages,
            |b, msgs| {
                b.iter(|| {
                    let serialized: Vec<String> = msgs
                        .iter()
                        .map(|m| serde_json::to_string(m).unwrap())
                        .collect();
                    black_box(serialized)
                })
            },
        );

        let serialized: Vec<String> = messages
            .iter()
            .map(|m| serde_json::to_string(m).unwrap())
            .collect();
        group.bench_with_input(
            BenchmarkId::new("escape_batch", size),
            &serialized,
            |b, msgs| {
                b.iter(|| {
                    let escaped: Vec<String> = msgs
                        .iter()
                        .map(|m| m.replace('\\', "\\\\").replace('\'', "\\'"))
                        .collect();
                    black_box(escaped)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark payload size thresholds for potential compression decisions
fn bench_payload_size_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_size_check");

    // Various payload sizes to test threshold checking
    let payloads: Vec<(usize, String)> = vec![100, 1024, 4096, 10240, 102400]
        .into_iter()
        .map(|size| {
            let data = "x".repeat(size);
            (size, data)
        })
        .collect();

    for (size, payload) in &payloads {
        group.bench_with_input(BenchmarkId::new("len_check", size), payload, |b, p| {
            b.iter(|| {
                let should_compress = p.len() > 4096;
                black_box(should_compress)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_json_serialization,
    bench_json_deserialization,
    bench_js_string_escape,
    bench_event_script_generation,
    bench_batch_processing,
    bench_payload_size_check,
);

criterion_main!(benches);
