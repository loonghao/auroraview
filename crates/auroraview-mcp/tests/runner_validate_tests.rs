//! Tests for `McpRunner` config validation on `start()` and `emit_agui_step()`.

use auroraview_mcp::{AguiEvent, McpRunner, McpServerConfig};
use rstest::rstest;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn free_port() -> u16 {
    use std::net::TcpListener;
    TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

fn config_no_mdns(port: u16) -> McpServerConfig {
    McpServerConfig::default()
        .with_port(port)
        .with_mdns(false)
}

// ---------------------------------------------------------------------------
// Validate-on-start: invalid port (0)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_port_zero_returns_invalid_config() {
    let config = McpServerConfig {
        port: 0,
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    let runner = McpRunner::new(config);
    let result = runner.start().await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Invalid configuration") || err.contains("port"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Validate-on-start: empty host
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_empty_host_returns_invalid_config() {
    let config = McpServerConfig {
        host: String::new(),
        port: free_port(),
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    let runner = McpRunner::new(config);
    let result = runner.start().await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Invalid configuration") || err.contains("host"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Validate-on-start: empty service_name
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_empty_service_name_returns_invalid_config() {
    let config = McpServerConfig {
        service_name: String::new(),
        port: free_port(),
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    let runner = McpRunner::new(config);
    let result = runner.start().await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Invalid configuration") || err.contains("service_name"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Validate-on-start: valid config succeeds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_valid_config_succeeds() {
    let runner = McpRunner::new(config_no_mdns(free_port()));
    runner.start().await.expect("valid config should start");
    runner.stop().await;
}

// ---------------------------------------------------------------------------
// Validate-on-start: whitespace-only host is invalid
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_whitespace_host_returns_invalid_config() {
    let config = McpServerConfig {
        host: "   ".to_string(),
        port: free_port(),
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    let runner = McpRunner::new(config);
    let result = runner.start().await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Validate-on-start: whitespace-only service_name is invalid
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_whitespace_service_name_returns_invalid_config() {
    let config = McpServerConfig {
        service_name: "  ".to_string(),
        port: free_port(),
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    let runner = McpRunner::new(config);
    let result = runner.start().await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// emit_agui_step: emits two events (StepStarted + StepFinished)
// ---------------------------------------------------------------------------

#[test]
fn emit_agui_step_sends_two_events() {
    let config = McpServerConfig::default().with_mdns(false);
    let runner = McpRunner::new(config);
    let mut rx = runner.agui_bus().subscribe();

    runner.agui_bus().emit(AguiEvent::StepStarted {
        run_id: "r1".into(),
        step_name: "export".into(),
        step_id: "s1".into(),
    });
    runner.agui_bus().emit(AguiEvent::StepFinished {
        run_id: "r1".into(),
        step_id: "s1".into(),
    });

    let ev1 = rx.try_recv().expect("StepStarted");
    let ev2 = rx.try_recv().expect("StepFinished");

    assert!(matches!(ev1, AguiEvent::StepStarted { .. }));
    assert!(matches!(ev2, AguiEvent::StepFinished { .. }));
}

#[test]
fn emit_agui_step_helper_step_started_fields() {
    let runner = McpRunner::new(McpServerConfig::default().with_mdns(false));
    let mut rx = runner.agui_bus().subscribe();

    runner.emit_agui_step("run-abc", "import_scene", "step-xyz");

    let ev1 = rx.try_recv().expect("first event");
    match ev1 {
        AguiEvent::StepStarted { run_id, step_name, step_id } => {
            assert_eq!(run_id, "run-abc");
            assert_eq!(step_name, "import_scene");
            assert_eq!(step_id, "step-xyz");
        }
        other => panic!("expected StepStarted, got {other:?}"),
    }
}

#[test]
fn emit_agui_step_helper_step_finished_fields() {
    let runner = McpRunner::new(McpServerConfig::default().with_mdns(false));
    let mut rx = runner.agui_bus().subscribe();

    runner.emit_agui_step("run-abc", "export_scene", "step-001");

    let _started = rx.try_recv().expect("StepStarted");
    let finished = rx.try_recv().expect("StepFinished");
    match finished {
        AguiEvent::StepFinished { run_id, step_id } => {
            assert_eq!(run_id, "run-abc");
            assert_eq!(step_id, "step-001");
        }
        other => panic!("expected StepFinished, got {other:?}"),
    }
}

#[test]
fn emit_agui_step_run_id_consistent() {
    let runner = McpRunner::new(McpServerConfig::default().with_mdns(false));
    let mut rx = runner.agui_bus().subscribe();

    runner.emit_agui_step("my-run", "do_thing", "step-9");

    let ev1 = rx.try_recv().expect("ev1");
    let ev2 = rx.try_recv().expect("ev2");

    assert_eq!(ev1.run_id(), "my-run");
    assert_eq!(ev2.run_id(), "my-run");
}

#[rstest]
#[case("run-1", "step_a", "s1")]
#[case("run-2", "step_b", "s2")]
#[case("run-3", "import",  "s3")]
fn emit_agui_step_parametrized(
    #[case] run_id: &str,
    #[case] step_name: &str,
    #[case] step_id: &str,
) {
    let runner = McpRunner::new(McpServerConfig::default().with_mdns(false));
    let mut rx = runner.agui_bus().subscribe();
    runner.emit_agui_step(run_id, step_name, step_id);
    let ev1 = rx.try_recv().expect("ev1");
    let ev2 = rx.try_recv().expect("ev2");
    assert!(matches!(ev1, AguiEvent::StepStarted { .. }));
    assert!(matches!(ev2, AguiEvent::StepFinished { .. }));
    assert_eq!(ev1.run_id(), run_id);
    assert_eq!(ev2.run_id(), run_id);
}

// ---------------------------------------------------------------------------
// McpServerConfig::validate() unit tests
// ---------------------------------------------------------------------------

#[test]
fn config_validate_valid() {
    let cfg = McpServerConfig::default();
    assert!(cfg.validate().is_ok());
}

#[test]
fn config_validate_port_zero() {
    let cfg = McpServerConfig { port: 0, ..McpServerConfig::default() };
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("port"), "expected 'port' in: {err}");
}

#[test]
fn config_validate_empty_host() {
    let cfg = McpServerConfig { host: String::new(), ..McpServerConfig::default() };
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("host"), "expected 'host' in: {err}");
}

#[test]
fn config_validate_empty_service_name() {
    let cfg = McpServerConfig { service_name: String::new(), ..McpServerConfig::default() };
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("service_name"), "expected 'service_name' in: {err}");
}

#[test]
fn config_is_valid_true_for_default() {
    assert!(McpServerConfig::default().is_valid());
}

#[test]
fn config_is_valid_false_for_port_zero() {
    let cfg = McpServerConfig { port: 0, ..McpServerConfig::default() };
    assert!(!cfg.is_valid());
}
