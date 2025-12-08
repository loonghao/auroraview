//! IPC performance benchmarks
//!
//! Run with: `cargo bench --bench ipc_bench`

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde_json::json;
use std::hint::black_box;

/// Benchmark JSON serialization (common IPC operation)
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    // Small payload
    let small_data = json!({"event": "click", "x": 100, "y": 200});
    group.throughput(Throughput::Elements(1));
    group.bench_function("small_payload", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&small_data)).unwrap();
            black_box(s)
        })
    });

    // Medium payload (typical DCC data)
    let medium_data = json!({
        "event": "selection_changed",
        "objects": (0..100).map(|i| format!("object_{}", i)).collect::<Vec<_>>(),
        "transform": {
            "position": [1.0, 2.0, 3.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        }
    });
    group.bench_function("medium_payload", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&medium_data)).unwrap();
            black_box(s)
        })
    });

    // Large payload (scene hierarchy)
    let large_data = json!({
        "event": "scene_update",
        "nodes": (0..1000).map(|i| json!({
            "id": i,
            "name": format!("node_{}", i),
            "parent": if i > 0 { Some(i / 10) } else { None },
            "transform": [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        })).collect::<Vec<_>>()
    });
    group.bench_function("large_payload", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&large_data)).unwrap();
            black_box(s)
        })
    });

    group.finish();
}

/// Benchmark JSON deserialization
fn bench_json_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_deserialization");

    let small_json = r#"{"event":"click","x":100,"y":200}"#;
    group.bench_function("small_payload", |b| {
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
    group.bench_function("medium_payload", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(black_box(&medium_json)).unwrap();
            black_box(v)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_serialization,
    bench_json_deserialization,
);

criterion_main!(benches);
