//! WebViewRegistry performance benchmarks
//!
//! Run with: `cargo bench --bench registry_bench`
//!
//! This benchmark suite measures the performance of WebViewRegistry operations
//! including register, get, update_url, remove, and list.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use auroraview_mcp::registry::WebViewRegistry;
use auroraview_mcp::types::WebViewConfig;

/// Benchmark WebViewRegistry::register
fn bench_register(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_register");
    let config = WebViewConfig::default();

    // Benchmark single register operation
    group.throughput(Throughput::Elements(1));
    group.bench_function("single", |b| {
        b.iter(|| {
            let reg = WebViewRegistry::new();
            let _id = reg.register(&config);
            criterion::black_box(());
        })
    });

    // Benchmark batch register (10 items)
    group.throughput(Throughput::Elements(10));
    group.bench_function("batch_10", |b| {
        b.iter(|| {
            let reg = WebViewRegistry::new();
            for _ in 0..10 {
                let _id = reg.register(&config);
            }
            criterion::black_box(());
        })
    });

    // Benchmark batch register (100 items)
    group.throughput(Throughput::Elements(100));
    group.bench_function("batch_100", |b| {
        b.iter(|| {
            let reg = WebViewRegistry::new();
            for _ in 0..100 {
                let _id = reg.register(&config);
            }
            criterion::black_box(());
        })
    });

    group.finish();
}

/// Benchmark WebViewRegistry::get
fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_get");

    // Prepare registry with different sizes
    let sizes = [1, 10, 100, 1000];

    for size in sizes {
        let reg = WebViewRegistry::new();
        let mut ids = Vec::new();
        for _ in 0..size {
            ids.push(reg.register(&WebViewConfig::default()));
        }

        group.throughput(Throughput::Elements(1));
        // Use first ID for consistent benchmarking
        let first_id = ids[0].clone();
        group.bench_with_input(BenchmarkId::new("size", size), &first_id, |b, id| {
            b.iter(|| {
                let _info = reg.get(id);
                criterion::black_box(());
            })
        });
    }

    group.finish();
}

/// Benchmark WebViewRegistry::update_url
fn bench_update_url(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_update_url");

    let sizes = [1, 10, 100];

    for size in sizes {
        let reg = WebViewRegistry::new();
        let mut ids = Vec::new();
        for _ in 0..size {
            ids.push(reg.register(&WebViewConfig::default()));
        }

        group.throughput(Throughput::Elements(1));
        // Use first ID for consistent benchmarking
        let first_id = ids[0].clone();
        group.bench_with_input(
            BenchmarkId::new("size", size),
            &first_id,
            |b, id| {
                b.iter(|| {
                    reg.update_url(id, "https://example.com");
                    criterion::black_box(());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark WebViewRegistry::remove
fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_remove");

    let sizes = [1, 10, 100];

    for size in sizes {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("prepare_size", size),
            &size,
            |b, &size| {
                b.iter_with_setup(
                    || {
                        let reg = WebViewRegistry::new();
                        let mut ids = Vec::new();
                        for _ in 0..size {
                            ids.push(reg.register(&WebViewConfig::default()));
                        }
                        (reg, ids)
                    },
                    |(reg, ids)| {
                        let id = &ids[0];
                        reg.remove(id);
                        criterion::black_box(());
                    },
                )
            },
        );
    }

    group.finish();
}

/// Benchmark WebViewRegistry::list
fn bench_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_list");

    let sizes = [0, 1, 10, 100, 1000];

    for size in sizes {
        let reg = WebViewRegistry::new();
        for _ in 0..size {
            reg.register(&WebViewConfig::default());
        }

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &reg, |b, reg| {
            b.iter(|| {
                let views = reg.list();
                criterion::black_box(views);
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_register,
    bench_get,
    bench_update_url,
    bench_remove,
    bench_list,
);
criterion_main!(benches);
