//! Tests for PluginRouter

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auroraview_plugin_core::{
    PluginError, PluginHandler, PluginRequest, PluginResult, PluginRouter, ScopeConfig,
};
use serde_json::{json, Value};

// ── Minimal mock handler ──────────────────────────────────────────────────────

struct EchoPlugin;

impl PluginHandler for EchoPlugin {
    fn name(&self) -> &str {
        "echo"
    }

    fn handle(&self, command: &str, args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "echo" => Ok(args),
            "fail" => Err(PluginError::shell_error("intentional failure")),
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec!["echo", "fail"]
    }
}

struct CounterPlugin {
    name: String,
}

impl PluginHandler for CounterPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, _args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "ping" => Ok(json!({"pong": true})),
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec!["ping"]
    }
}

// ── Router creation ───────────────────────────────────────────────────────────

#[test]
fn router_new_empty() {
    let router = PluginRouter::new();
    assert!(router.plugin_names().is_empty());
}

#[test]
fn router_default_is_same_as_new() {
    let r1 = PluginRouter::new();
    let r2 = PluginRouter::default();
    assert_eq!(r1.plugin_names().len(), r2.plugin_names().len());
}

// ── Register / unregister ─────────────────────────────────────────────────────

#[test]
fn router_register_plugin() {
    let mut router = PluginRouter::new();
    router.register("echo", Arc::new(EchoPlugin));
    assert!(router.has_plugin("echo"));
}

#[test]
fn router_has_plugin_false_for_unknown() {
    let router = PluginRouter::new();
    assert!(!router.has_plugin("nonexistent"));
}

#[test]
fn router_unregister_removes_plugin() {
    let mut router = PluginRouter::new();
    router.register("echo", Arc::new(EchoPlugin));
    let removed = router.unregister("echo");
    assert!(removed.is_some());
    assert!(!router.has_plugin("echo"));
}

#[test]
fn router_unregister_unknown_returns_none() {
    let mut router = PluginRouter::new();
    let removed = router.unregister("ghost");
    assert!(removed.is_none());
}

#[test]
fn router_plugin_names_contains_registered() {
    let mut router = PluginRouter::new();
    router.register("alpha", Arc::new(EchoPlugin));
    router.register(
        "beta",
        Arc::new(CounterPlugin {
            name: "beta".into(),
        }),
    );
    let names = router.plugin_names();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

// ── handle: plugin not enabled ────────────────────────────────────────────────

#[test]
fn router_disabled_plugin_returns_error_response() {
    // Register but DON'T enable in scope
    let scope = ScopeConfig::default(); // no plugins enabled
    let mut router = PluginRouter::with_scope(scope);
    router.register("echo", Arc::new(EchoPlugin));

    let req = PluginRequest::new("echo", "echo", json!({"x": 1}));
    let resp = router.handle(req);
    assert!(!resp.success);
    assert!(resp
        .code
        .as_deref()
        .map(|c| c == "PLUGIN_DISABLED")
        .unwrap_or(false));
}

// ── handle: plugin not found ──────────────────────────────────────────────────

#[test]
fn router_plugin_not_registered_returns_error() {
    let mut router = PluginRouter::new();
    // "echo" is in enabled set (default new scope doesn't have it, so add it)
    router.scope_mut().enable_plugin("echo");

    let req = PluginRequest::new("echo", "echo", json!({}));
    let resp = router.handle(req);
    // plugin enabled but not registered
    assert!(!resp.success);
    assert_eq!(resp.code.as_deref(), Some("PLUGIN_NOT_FOUND"));
}

// ── handle: success ───────────────────────────────────────────────────────────

#[test]
fn router_handle_success() {
    let mut router = PluginRouter::new();
    router.scope_mut().enable_plugin("echo");
    router.register("echo", Arc::new(EchoPlugin));

    let args = json!({"msg": "hello"});
    let req = PluginRequest::new("echo", "echo", args.clone());
    let resp = router.handle(req);
    assert!(resp.success);
    assert_eq!(resp.data, Some(args));
}

// ── handle: command error ─────────────────────────────────────────────────────

#[test]
fn router_handle_plugin_error() {
    let mut router = PluginRouter::new();
    router.scope_mut().enable_plugin("echo");
    router.register("echo", Arc::new(EchoPlugin));

    let req = PluginRequest::new("echo", "fail", json!({}));
    let resp = router.handle(req);
    assert!(!resp.success);
    assert!(resp.error.is_some());
}

// ── handle: request id is echoed ─────────────────────────────────────────────

#[test]
fn router_handle_echoes_request_id() {
    let mut router = PluginRouter::new();
    router.scope_mut().enable_plugin("echo");
    router.register("echo", Arc::new(EchoPlugin));

    let req = PluginRequest::new("echo", "echo", json!({})).with_id("req-007");
    let resp = router.handle(req);
    assert_eq!(resp.id, Some("req-007".to_string()));
}

// ── with_scope ────────────────────────────────────────────────────────────────

#[test]
fn router_with_scope_uses_custom_scope() {
    let scope = ScopeConfig::permissive();
    let router = PluginRouter::with_scope(scope);
    // permissive scope enables default plugins
    assert!(router.scope().is_plugin_enabled("fs"));
}

// ── set_scope ─────────────────────────────────────────────────────────────────

#[test]
fn router_set_scope_replaces_scope() {
    let mut router = PluginRouter::new();
    let new_scope = ScopeConfig::permissive();
    router.set_scope(new_scope);
    assert!(router.scope().is_plugin_enabled("fs"));
}

// ── Event callback ────────────────────────────────────────────────────────────

#[test]
fn router_event_callback_set_and_emit() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let router = PluginRouter::new();
    router.set_event_callback(Arc::new(move |_event, _data| {
        counter2.fetch_add(1, Ordering::SeqCst);
    }));

    router.emit_event("test_event", json!({"key": "value"}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn router_clear_event_callback() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let router = PluginRouter::new();
    router.set_event_callback(Arc::new(move |_, _| {
        counter2.fetch_add(1, Ordering::SeqCst);
    }));
    router.clear_event_callback();
    router.emit_event("test_event", json!({}));
    // counter should remain 0 after clear
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn router_emit_without_callback_no_panic() {
    let router = PluginRouter::new();
    router.emit_event("no_callback", json!(null));
    // Should not panic
}

// ── event_callback_ref ────────────────────────────────────────────────────────

#[test]
fn router_event_callback_ref_initially_none() {
    let router = PluginRouter::new();
    let cb_ref = router.event_callback_ref();
    assert!(cb_ref.read().is_none());
}
