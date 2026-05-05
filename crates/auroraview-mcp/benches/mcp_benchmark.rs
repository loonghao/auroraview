use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::sync::Arc;

// Type imports (benchmarks are part of the crate, can use internal modules)
use auroraview_mcp::agui::{AguiBus, AguiEvent};
use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;

fn bench_mcp_server_config_default(c: &mut Criterion) {
    c.bench_function("mcp_server_config_default", |b| {
        b.iter(|| black_box(McpServerConfig::default()))
    });
}

fn bench_mcp_server_config_with_port(c: &mut Criterion) {
    c.bench_function("mcp_server_config_with_port", |b| {
        b.iter(|| black_box(McpServerConfig::default().with_port(8080)))
    });
}

fn bench_mcp_runner_with_capacity(c: &mut Criterion) {
    c.bench_function("mcp_runner_with_capacity", |b| {
        b.iter(|| black_box(McpRunner::with_capacity(8080, 16)))
    });
}

fn bench_mcp_runner_with_mdns_port(c: &mut Criterion) {
    c.bench_function("mcp_runner_with_mdns_port", |b| {
        b.iter(|| black_box(McpRunner::with_mdns_port(8080)))
    });
}

fn bench_agui_bus_new(c: &mut Criterion) {
    c.bench_function("agui_bus_new", |b| b.iter(|| black_box(AguiBus::new())));
}

fn bench_agui_bus_emit_without_subscribers(c: &mut Criterion) {
    c.bench_function("agui_bus_emit_without_subscribers", |b| {
        let bus = AguiBus::new();
        let event = Arc::new(AguiEvent::RunStarted {
            run_id: "bench-run".to_string(),
            thread_id: "bench-thread".to_string(),
        });
        b.iter(|| {
            // Use black_box to prevent over-optimization
            // Cloning Arc is cheap (atomic increment), measures emit() performance
            bus.emit(black_box(Arc::clone(&event)))
        })
    });
}

fn bench_agui_bus_emit_with_subscribers(c: &mut Criterion) {
    let mut group = c.benchmark_group("agui_bus_emit_with_subscribers");
    for num_subscribers in [1, 10, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_subscribers),
            &num_subscribers,
            |b, &num| {
                // Setup: create bus and subscribers OUTSIDE the measured iteration
                let bus = AguiBus::new();
                // Create subscribers
                for _ in 0..num {
                    let _rx = bus.subscribe();
                }
                let event = Arc::new(AguiEvent::RunStarted {
                    run_id: "bench-run".to_string(),
                    thread_id: "bench-thread".to_string(),
                });
                // Only measure the emit call itself, use black_box to prevent over-optimization
                // Cloning Arc is cheap (atomic increment), measures emit() performance
                b.iter(|| bus.emit(black_box(Arc::clone(&event))))
            },
        );
    }
    group.finish();
}

fn bench_agui_bus_subscribe(c: &mut Criterion) {
    c.bench_function("agui_bus_subscribe", |b| {
        let bus = AguiBus::new();
        b.iter(|| black_box(bus.subscribe()))
    });
}

fn bench_agui_bus_receiver_count(c: &mut Criterion) {
    c.bench_function("agui_bus_receiver_count", |b| {
        let bus = AguiBus::new();
        let _rx = bus.subscribe();
        b.iter(|| black_box(bus.receiver_count()))
    });
}

fn bench_agui_event_to_sse_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("agui_event_to_sse_line");
    // Test different AguiEvent variants
    let events: Vec<(&str, AguiEvent)> = vec![
        (
            "RunStarted",
            AguiEvent::RunStarted {
                run_id: "run-123".to_string(),
                thread_id: "thread-456".to_string(),
            },
        ),
        (
            "RunError",
            AguiEvent::RunError {
                run_id: "run-123".to_string(),
                message: "Something went wrong".to_string(),
                code: Some("ERR_001".to_string()),
            },
        ),
        (
            "ToolCallStart",
            AguiEvent::ToolCallStart {
                run_id: "run-123".to_string(),
                tool_call_id: "call-789".to_string(),
                tool_name: "screenshot".to_string(),
            },
        ),
        (
            "StateDelta",
            AguiEvent::StateDelta {
                run_id: "run-123".to_string(),
                delta: vec![
                    serde_json::json!({"op": "replace", "path": "/status", "value": "running"}),
                    serde_json::json!({"op": "add", "path": "/steps/-", "value": {"name": "init"}}),
                ],
            },
        ),
    ];
    for (name, event) in events {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &event,
            |b, e| b.iter(|| black_box(e.to_sse_line())),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_mcp_server_config_default,
    bench_mcp_server_config_with_port,
    bench_mcp_runner_with_capacity,
    bench_mcp_runner_with_mdns_port,
    bench_agui_bus_new,
    bench_agui_bus_emit_without_subscribers,
    bench_agui_bus_emit_with_subscribers,
    bench_agui_bus_subscribe,
    bench_agui_bus_receiver_count,
    bench_agui_event_to_sse_line
);
criterion_main!(benches);
