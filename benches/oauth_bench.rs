// OAuthStore performance benchmarks
//
// Run with: `cargo bench --bench oauth_bench`
//
// This benchmark suite measures the performance of OAuthStore operations:
// - register_client (write operation)
// - validate_client (read operation)
// - issue_code (write operation)
// - exchange_code (read + write operation)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;

// Import OAuthStore from auroraview-mcp crate
// Note: this requires `criterion` to be added as a dev-dependency

fn bench_register_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("oauth_register_client");
    group.throughput(Throughput::Elements(1));

    group.bench_function("register", |b| {
        b.iter(|| {
            // Create a new store for each iteration to avoid state buildup
            let store = Arc::new(auroraview_mcp::oauth::OAuthStore::new());
            let (_, _) = store.register_client(
                "Test Client".to_string(),
                vec!["http://localhost:8080/callback".to_string()],
                "mcp:tools".to_string(),
            );
        })
    });

    group.finish();
}

fn bench_validate_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("oauth_validate_client");
    group.throughput(Throughput::Elements(1));

    // Pre-populate a store
    let store = Arc::new(auroraview_mcp::oauth::OAuthStore::new());
    let (client, secret) = store.register_client(
        "Test Client".to_string(),
        vec!["http://localhost:8080/callback".to_string()],
        "mcp:tools".to_string(),
    );

    group.bench_function("validate_valid", |b| {
        b.iter(|| {
            let result = store.validate_client(&client.client_id, &secret);
            assert!(result.is_some());
        })
    });

    group.bench_function("validate_invalid", |b| {
        b.iter(|| {
            let result = store.validate_client("invalid_id", "invalid_secret");
            assert!(result.is_none());
        })
    });

    group.finish();
}

fn bench_issue_and_exchange_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("oauth_code_flow");
    group.throughput(Throughput::Elements(1));

    let store = Arc::new(auroraview_mcp::oauth::OAuthStore::new());
    let (client, _) = store.register_client(
        "Test Client".to_string(),
        vec!["http://localhost:8080/callback".to_string()],
        "mcp:tools".to_string(),
    );

    let code_verifier = "test_verifier_12345678901234567890123456789012";
    let code_challenge = base64_url::encode(&sha2::Sha256::digest(code_verifier.as_bytes()));

    group.bench_function("issue_code", |b| {
        b.iter(|| {
            let code = store.issue_code(
                client.client_id.clone(),
                "http://localhost:8080/callback".to_string(),
                code_challenge.clone(),
                "mcp:tools".to_string(),
            );
            assert!(!code.is_empty());
        })
    });

    // Pre-issue a code for exchange benchmark
    let code = store.issue_code(
        client.client_id.clone(),
        "http://localhost:8080/callback".to_string(),
        code_challenge.clone(),
        "mcp:tools".to_string(),
    );

    group.bench_function("exchange_code", |b| {
        b.iter(|| {
            let result = store.exchange_code(
                &code,
                &client.client_id,
                "http://localhost:8080/callback",
                code_verifier,
            );
            assert!(result.is_some());
        })
    });

    group.finish();
}

fn bench_concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("oauth_concurrent_reads");
    
    // Pre-populate store with many clients
    let store = Arc::new(auroraview_mcp::oauth::OAuthStore::new());
    for i in 0..100 {
        store.register_client(
            format!("Client {}", i),
            vec!["http://localhost:8080/callback".to_string()],
            "mcp:tools".to_string(),
        );
    }

    group.bench_function("validate_100_clients", |b| {
        b.iter(|| {
            // Randomly pick a client to validate
            let client_id = format!("client_{}", 42);
            let _ = store.validate_client(&client_id, "some_secret");
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_register_client,
    bench_validate_client,
    bench_issue_and_exchange_code,
    bench_concurrent_reads
);

criterion_main!(benches);
