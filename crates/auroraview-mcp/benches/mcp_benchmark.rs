use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;

fn bench_mcp_server_config_default(c: &mut Criterion) {
    c.bench_function("mcp_server_config_default", |b| {
        b.iter(|| {
            black_box(McpServerConfig::default())
        })
    });
}

fn bench_mcp_server_config_with_port(c: &mut Criterion) {
    c.bench_function("mcp_server_config_with_port", |b| {
        b.iter(|| {
            black_box(McpServerConfig::default().with_port(8080))
        })
    });
}

fn bench_mcp_runner_with_capacity(c: &mut Criterion) {
    c.bench_function("mcp_runner_with_capacity", |b| {
        b.iter(|| {
            black_box(McpRunner::with_capacity(8080, 16))
        })
    });
}

fn bench_mcp_runner_with_mdns_port(c: &mut Criterion) {
    c.bench_function("mcp_runner_with_mdns_port", |b| {
        b.iter(|| {
            black_box(McpRunner::with_mdns_port(8080))
        })
    });
}

criterion_group!(benches, bench_mcp_server_config_default, bench_mcp_server_config_with_port, bench_mcp_runner_with_capacity, bench_mcp_runner_with_mdns_port);
criterion_main!(benches);
