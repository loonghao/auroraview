//! Unit tests for auroraview-testing data types (no CDP required)

use auroraview_testing::{
    ActionResult, InspectorConfig, InspectorError, RefId, RefInfo, ScrollDirection, Snapshot,
    SnapshotFormat, WaitCondition,
};
use rstest::rstest;
use std::time::Duration;

// ============================================================================
// InspectorError: Display
// ============================================================================

#[rstest]
fn error_display_connection() {
    let e = InspectorError::Connection("refused".to_string());
    assert!(e.to_string().contains("CDP connection error"));
    assert!(e.to_string().contains("refused"));
}

#[rstest]
fn error_display_command() {
    let e = InspectorError::Command("method not found".to_string());
    assert!(e.to_string().contains("CDP command failed"));
    assert!(e.to_string().contains("method not found"));
}

#[rstest]
fn error_display_websocket() {
    let e = InspectorError::WebSocket("protocol error".to_string());
    assert!(e.to_string().contains("WebSocket error"));
}

#[rstest]
fn error_display_timeout() {
    let e = InspectorError::Timeout("30s exceeded".to_string());
    assert!(e.to_string().contains("timed out"));
    assert!(e.to_string().contains("30s exceeded"));
}

#[rstest]
fn error_display_element_not_found() {
    let e = InspectorError::ElementNotFound("@99".to_string());
    assert!(e.to_string().contains("Element not found"));
    assert!(e.to_string().contains("@99"));
}

#[rstest]
fn error_display_invalid_ref() {
    let e = InspectorError::InvalidRef("@abc".to_string());
    assert!(e.to_string().contains("Invalid ref ID"));
}

#[rstest]
fn error_display_javascript() {
    let e = InspectorError::JavaScript("SyntaxError".to_string());
    assert!(e.to_string().contains("JavaScript error"));
}

#[rstest]
fn error_display_navigation() {
    let e = InspectorError::Navigation("net::ERR_FAILED".to_string());
    assert!(e.to_string().contains("Navigation error"));
}

#[rstest]
fn error_display_screenshot() {
    let e = InspectorError::Screenshot("no page attached".to_string());
    assert!(e.to_string().contains("Screenshot error"));
}

#[rstest]
fn error_display_serialization() {
    let e = InspectorError::Serialization("missing field".to_string());
    assert!(e.to_string().contains("Serialization error"));
}

#[rstest]
fn error_display_parse() {
    let e = InspectorError::Parse("invalid url".to_string());
    assert!(e.to_string().contains("Parse error"));
}

#[rstest]
fn error_display_session() {
    let e = InspectorError::Session("session closed".to_string());
    assert!(e.to_string().contains("Session error"));
}

#[rstest]
fn error_display_internal() {
    let e = InspectorError::Internal("unexpected state".to_string());
    assert!(e.to_string().contains("Internal error"));
}

// ============================================================================
// InspectorError: From conversions
// ============================================================================

#[rstest]
fn error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
    let e: InspectorError = json_err.into();
    assert!(e.to_string().contains("Serialization error"));
}

#[rstest]
fn error_from_url_parse() {
    let url_err = url::Url::parse("not a url").unwrap_err();
    let e: InspectorError = url_err.into();
    assert!(e.to_string().contains("Parse error"));
}

// ============================================================================
// InspectorError: Send + Sync
// ============================================================================

fn assert_send_sync<T: Send + Sync>() {}

#[rstest]
fn inspector_error_is_send_sync() {
    assert_send_sync::<InspectorError>();
}

// ============================================================================
// InspectorConfig: defaults and construction
// ============================================================================

#[rstest]
fn inspector_config_default() {
    let config = InspectorConfig::default();
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert!(!config.capture_screenshots);
    assert!(config.detect_changes);
}

#[rstest]
fn inspector_config_clone() {
    let config = InspectorConfig {
        timeout: Duration::from_secs(60),
        capture_screenshots: true,
        detect_changes: false,
    };
    let c = config.clone();
    assert_eq!(c.timeout, Duration::from_secs(60));
    assert!(c.capture_screenshots);
    assert!(!c.detect_changes);
}

#[rstest]
fn inspector_config_debug() {
    let config = InspectorConfig::default();
    let s = format!("{:?}", config);
    assert!(s.contains("InspectorConfig"));
}

// ============================================================================
// RefInfo: construction and builder
// ============================================================================

#[rstest]
fn ref_info_new() {
    let r = RefInfo::new("@1", "button", "Submit");
    assert_eq!(r.ref_id, "@1");
    assert_eq!(r.role, "button");
    assert_eq!(r.name, "Submit");
    assert!(r.description.is_empty());
    assert!(r.selector.is_empty());
    assert!(r.backend_node_id.is_none());
    assert!(r.bounds.is_none());
}

#[rstest]
fn ref_info_with_description() {
    let r = RefInfo::new("@2", "link", "Home").with_description("nav link");
    assert_eq!(r.description, "nav link");
}

#[rstest]
fn ref_info_with_selector() {
    let r = RefInfo::new("@3", "textbox", "Email").with_selector("#email-input");
    assert_eq!(r.selector, "#email-input");
}

#[rstest]
fn ref_info_with_backend_node_id() {
    let r = RefInfo::new("@4", "button", "OK").with_backend_node_id(12345);
    assert_eq!(r.backend_node_id, Some(12345));
}

#[rstest]
fn ref_info_with_bounds() {
    let r = RefInfo::new("@5", "button", "Click").with_bounds(10.0, 20.0, 100.0, 30.0);
    assert_eq!(r.bounds, Some((10.0, 20.0, 100.0, 30.0)));
}

#[rstest]
fn ref_info_display_with_description() {
    let r = RefInfo::new("@3", "textbox", "Search").with_description("search input");
    assert_eq!(r.to_string(), "@3 [textbox] \"Search\" - search input");
}

#[rstest]
fn ref_info_display_no_description() {
    let r = RefInfo::new("@1", "button", "OK");
    assert_eq!(r.to_string(), "@1 [button] \"OK\"");
}

#[rstest]
fn ref_info_clone() {
    let r = RefInfo::new("@1", "button", "Test").with_description("d");
    let c = r.clone();
    assert_eq!(c.ref_id, "@1");
    assert_eq!(c.description, "d");
}

#[rstest]
fn ref_info_debug() {
    let r = RefInfo::new("@1", "button", "Test");
    let s = format!("{:?}", r);
    assert!(s.contains("RefInfo"));
    assert!(s.contains("@1"));
}

// ============================================================================
// RefId: normalization and conversions
// ============================================================================

#[rstest]
#[case("@3", "@3")]
#[case("3", "@3")]
#[case("@10", "@10")]
#[case("10", "@10")]
fn ref_id_normalized(#[case] input: &str, #[case] expected: &str) {
    let id = RefId::from(input);
    assert_eq!(id.normalized(), expected);
}

#[rstest]
fn ref_id_from_u32() {
    let id = RefId::from(5u32);
    assert_eq!(id.normalized(), "@5");
    assert_eq!(id.numeric(), Some(5));
}

#[rstest]
fn ref_id_from_i32() {
    let id = RefId::from(7i32);
    assert_eq!(id.normalized(), "@7");
}

#[rstest]
fn ref_id_from_string() {
    let id = RefId::from("@9".to_string());
    assert_eq!(id.normalized(), "@9");
}

#[rstest]
fn ref_id_numeric_with_at_prefix() {
    let id = RefId::from("@42");
    assert_eq!(id.numeric(), Some(42));
}

#[rstest]
fn ref_id_numeric_without_prefix() {
    let id = RefId::from("15");
    assert_eq!(id.numeric(), Some(15));
}

#[rstest]
fn ref_id_numeric_invalid() {
    let id = RefId::from("abc");
    assert_eq!(id.numeric(), None);
}

// ============================================================================
// Snapshot: construction and operations
// ============================================================================

#[rstest]
fn snapshot_new() {
    let s = Snapshot::new("Title".to_string(), "http://a.com".to_string(), (1280, 720));
    assert_eq!(s.title, "Title");
    assert_eq!(s.url, "http://a.com");
    assert_eq!(s.viewport, (1280, 720));
    assert_eq!(s.ref_count(), 0);
    assert!(s.tree.is_empty());
}

#[rstest]
fn snapshot_ref_count() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    assert_eq!(s.ref_count(), 0);
    s.refs
        .insert("@1".to_string(), RefInfo::new("@1", "button", "B1"));
    s.refs
        .insert("@2".to_string(), RefInfo::new("@2", "link", "L1"));
    assert_eq!(s.ref_count(), 2);
}

#[rstest]
fn snapshot_get_ref_with_at() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs
        .insert("@5".to_string(), RefInfo::new("@5", "button", "Five"));
    assert!(s.get_ref("@5").is_some());
    assert_eq!(s.get_ref("@5").unwrap().name, "Five");
}

#[rstest]
fn snapshot_get_ref_without_at() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs
        .insert("@5".to_string(), RefInfo::new("@5", "button", "Five"));
    assert!(s.get_ref("5").is_some());
}

#[rstest]
fn snapshot_get_ref_missing() {
    let s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    assert!(s.get_ref("@99").is_none());
}

#[rstest]
fn snapshot_find_by_name() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs.insert(
        "@1".to_string(),
        RefInfo::new("@1", "button", "Submit Form"),
    );
    s.refs
        .insert("@2".to_string(), RefInfo::new("@2", "link", "Cancel"));
    let results = s.find("submit");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Submit Form");
}

#[rstest]
fn snapshot_find_case_insensitive() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs
        .insert("@1".to_string(), RefInfo::new("@1", "button", "SUBMIT"));
    let results = s.find("submit");
    assert_eq!(results.len(), 1);
}

#[rstest]
fn snapshot_find_by_description() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs.insert(
        "@1".to_string(),
        RefInfo::new("@1", "button", "X").with_description("primary action"),
    );
    let results = s.find("primary");
    assert_eq!(results.len(), 1);
}

#[rstest]
fn snapshot_find_no_match() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs
        .insert("@1".to_string(), RefInfo::new("@1", "button", "OK"));
    let results = s.find("nonexistent");
    assert!(results.is_empty());
}

#[rstest]
fn snapshot_to_text_contains_header() {
    let s = Snapshot::new(
        "My Page".to_string(),
        "http://x.com".to_string(),
        (1024, 768),
    );
    let text = s.to_text();
    assert!(text.contains("My Page"));
    assert!(text.contains("http://x.com"));
    assert!(text.contains("1024x768"));
}

#[rstest]
fn snapshot_to_json_valid() {
    let s = Snapshot::new("P".to_string(), "u".to_string(), (800, 600));
    let json = s.to_json();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["title"].as_str(), Some("P"));
    assert_eq!(v["url"].as_str(), Some("u"));
}

#[rstest]
fn snapshot_display_same_as_to_text() {
    let s = Snapshot::new("Disp".to_string(), "http://d.com".to_string(), (640, 480));
    assert_eq!(format!("{}", s), s.to_text());
}

#[rstest]
fn snapshot_clone() {
    let mut s = Snapshot::new("T".to_string(), "u".to_string(), (800, 600));
    s.refs
        .insert("@1".to_string(), RefInfo::new("@1", "button", "B"));
    let c = s.clone();
    assert_eq!(c.title, "T");
    assert_eq!(c.ref_count(), 1);
}

// ============================================================================
// ActionResult
// ============================================================================

#[rstest]
fn action_result_success() {
    let r = ActionResult::success("click @1");
    assert!(r.success);
    assert_eq!(r.action, "click @1");
    assert!(r.error.is_none());
}

#[rstest]
fn action_result_failure() {
    let r = ActionResult::failure("click @99", "not found");
    assert!(!r.success);
    assert_eq!(r.error.as_deref(), Some("not found"));
}

#[rstest]
fn action_result_with_changes() {
    let r = ActionResult::success("click @1")
        .with_change("url changed")
        .with_change("ref @2 appeared");
    assert_eq!(r.changes.len(), 2);
    assert_eq!(r.changes[0], "url changed");
}

#[rstest]
fn action_result_with_duration() {
    let r = ActionResult::success("goto").with_duration(150);
    assert_eq!(r.duration_ms, 150);
}

#[rstest]
fn action_result_with_before_after() {
    let r = ActionResult::success("fill @3")
        .with_before("empty")
        .with_after("hello");
    assert_eq!(r.before, "empty");
    assert_eq!(r.after, "hello");
}

#[rstest]
fn action_result_display_success() {
    let r = ActionResult::success("click @3").with_change("nav");
    let s = r.to_string();
    assert!(s.contains("✓"));
    assert!(s.contains("nav"));
}

#[rstest]
fn action_result_display_failure() {
    let r = ActionResult::failure("click @99", "not found");
    let s = r.to_string();
    assert!(s.contains("✗"));
    assert!(s.contains("not found"));
}

#[rstest]
fn action_result_clone() {
    let r = ActionResult::success("act").with_change("c");
    let c = r.clone();
    assert!(c.success);
    assert_eq!(c.changes.len(), 1);
}

// ============================================================================
// WaitCondition: parse
// ============================================================================

#[rstest]
fn wait_condition_parse_idle() {
    let w = WaitCondition::parse("idle").unwrap();
    assert!(matches!(w, WaitCondition::NetworkIdle));
}

#[rstest]
fn wait_condition_parse_loaded() {
    let w = WaitCondition::parse("loaded").unwrap();
    assert!(matches!(w, WaitCondition::DomContentLoaded));
}

#[rstest]
fn wait_condition_parse_text() {
    let w = WaitCondition::parse("text:Welcome").unwrap();
    assert!(matches!(w, WaitCondition::Text(s) if s == "Welcome"));
}

#[rstest]
fn wait_condition_parse_ref() {
    let w = WaitCondition::parse("ref:@5").unwrap();
    assert!(matches!(w, WaitCondition::Ref(_)));
}

#[rstest]
fn wait_condition_parse_url() {
    let w = WaitCondition::parse("url:*/dashboard").unwrap();
    assert!(matches!(w, WaitCondition::Url(s) if s == "*/dashboard"));
}

#[rstest]
fn wait_condition_parse_js() {
    let w = WaitCondition::parse("js:document.readyState === 'complete'").unwrap();
    assert!(matches!(w, WaitCondition::Js(s) if s == "document.readyState === 'complete'"));
}

#[rstest]
fn wait_condition_parse_default_text() {
    let w = WaitCondition::parse("some text without prefix").unwrap();
    assert!(matches!(w, WaitCondition::Text(_)));
}

#[rstest]
fn wait_condition_parse_trims_whitespace() {
    let w = WaitCondition::parse("  idle  ").unwrap();
    assert!(matches!(w, WaitCondition::NetworkIdle));
}

// ============================================================================
// ScrollDirection: parse
// ============================================================================

#[rstest]
#[case("up", true)]
#[case("down", true)]
#[case("left", true)]
#[case("right", true)]
#[case("UP", true)]
#[case("DOWN", true)]
#[case("diagonal", false)]
fn scroll_direction_parse(#[case] input: &str, #[case] expect_some: bool) {
    let result = ScrollDirection::parse(input);
    assert_eq!(result.is_some(), expect_some);
}

#[rstest]
fn scroll_direction_debug() {
    let s = format!("{:?}", ScrollDirection::Down);
    assert!(s.contains("Down"));
}

#[rstest]
fn scroll_direction_clone() {
    let d = ScrollDirection::Up;
    let c = d;
    assert!(matches!(c, ScrollDirection::Up));
}

// ============================================================================
// SnapshotFormat: default and debug
// ============================================================================

#[rstest]
fn snapshot_format_default_is_text() {
    let f = SnapshotFormat::default();
    assert!(matches!(f, SnapshotFormat::Text));
}

#[rstest]
fn snapshot_format_debug() {
    let s = format!("{:?}", SnapshotFormat::Json);
    assert!(s.contains("Json"));
}
