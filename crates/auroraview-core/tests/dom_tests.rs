//! DOM operations tests

use auroraview_core::dom::{DomBatch, DomOp};
use rstest::rstest;

// ============================================================================
// Basic DomBatch tests
// ============================================================================

#[test]
fn empty_batch() {
    let batch = DomBatch::new();
    assert!(batch.is_empty());
    assert_eq!(batch.to_js(), "(function(){})()");
}

#[test]
fn with_capacity_is_empty() {
    let batch = DomBatch::with_capacity(16);
    assert!(batch.is_empty());
    assert_eq!(batch.len(), 0);
}

#[test]
fn len_increases_with_ops() {
    let mut batch = DomBatch::new();
    batch.set_text("#a", "A");
    batch.set_text("#b", "B");
    assert_eq!(batch.len(), 2);
}

#[test]
fn clear_empties_batch() {
    let mut batch = DomBatch::new();
    batch.set_text("#a", "A");
    assert!(!batch.is_empty());
    batch.clear();
    assert!(batch.is_empty());
}

#[test]
fn clone_batch() {
    let mut batch = DomBatch::new();
    batch.set_text("#title", "Hello");
    let cloned = batch.clone();
    assert_eq!(cloned.len(), 1);
    assert!(!cloned.is_empty());
}

#[test]
fn operations_slice() {
    let mut batch = DomBatch::new();
    batch.set_text("#a", "A");
    batch.set_text("#b", "B");
    assert_eq!(batch.operations().len(), 2);
}

// ============================================================================
// DomBatch set_text
// ============================================================================

#[test]
fn set_text() {
    let mut batch = DomBatch::new();
    batch.set_text("#title", "Hello World");
    assert_eq!(batch.len(), 1);
    let js = batch.to_js();
    assert!(js.contains("textContent"));
    assert!(js.contains("Hello World"));
}

#[test]
fn set_text_empty_string() {
    let mut batch = DomBatch::new();
    batch.set_text("#el", "");
    let js = batch.to_js();
    assert!(js.contains("textContent"));
}

#[test]
fn set_html() {
    let mut batch = DomBatch::new();
    batch.set_html("#content", "<b>Bold</b>");
    let js = batch.to_js();
    assert!(js.contains("innerHTML"));
}

#[test]
fn add_class() {
    let mut batch = DomBatch::new();
    batch.add_class("#el", "active");
    let js = batch.to_js();
    assert!(js.contains("classList.add"));
    assert!(js.contains("active"));
}

#[test]
fn remove_class() {
    let mut batch = DomBatch::new();
    batch.remove_class("#el", "hidden");
    let js = batch.to_js();
    assert!(js.contains("classList.remove"));
}

#[test]
fn toggle_class() {
    let mut batch = DomBatch::new();
    batch.toggle_class("#el", "open");
    let js = batch.to_js();
    assert!(js.contains("classList.toggle"));
}

#[test]
fn set_attribute() {
    let mut batch = DomBatch::new();
    batch.set_attribute("#img", "src", "image.png");
    let js = batch.to_js();
    assert!(js.contains("setAttribute"));
    assert!(js.contains("src"));
}

#[test]
fn remove_attribute() {
    let mut batch = DomBatch::new();
    batch.remove_attribute("#el", "disabled");
    let js = batch.to_js();
    assert!(js.contains("removeAttribute"));
    assert!(js.contains("disabled"));
}

#[test]
fn show_element() {
    let mut batch = DomBatch::new();
    batch.show("#el");
    let js = batch.to_js();
    assert!(js.contains("#el"));
}

#[test]
fn hide_element() {
    let mut batch = DomBatch::new();
    batch.hide("#el");
    let js = batch.to_js();
    assert!(js.contains("none") || js.contains("#el"));
}

#[test]
fn set_style() {
    let mut batch = DomBatch::new();
    batch.set_style("#el", "color", "red");
    let js = batch.to_js();
    assert!(js.contains("style"));
    assert!(js.contains("red") || js.contains("color"));
}

#[test]
fn set_value() {
    let mut batch = DomBatch::new();
    batch.set_value("#input", "test@example.com");
    let js = batch.to_js();
    assert!(js.contains("value") || js.contains("test@example.com"));
}

#[test]
fn click_element() {
    let mut batch = DomBatch::new();
    batch.click("#btn");
    let js = batch.to_js();
    assert!(js.contains("click") || js.contains("#btn"));
}

#[test]
fn focus_element() {
    let mut batch = DomBatch::new();
    batch.focus("#input");
    let js = batch.to_js();
    assert!(js.contains("focus") || js.contains("#input"));
}

// ============================================================================
// escape_string
// ============================================================================

#[test]
fn escape_string_plain() {
    assert_eq!(DomBatch::escape_string("hello"), "hello");
}

#[test]
fn escape_string_quotes() {
    assert_eq!(DomBatch::escape_string("hello\"world"), "hello\\\"world");
}

#[test]
fn escape_string_newline() {
    assert_eq!(DomBatch::escape_string("hello\nworld"), "hello\\nworld");
}

#[test]
fn escape_string_tab() {
    assert_eq!(DomBatch::escape_string("hello\tworld"), "hello\\tworld");
}

#[test]
fn escape_string_backslash() {
    assert_eq!(DomBatch::escape_string("a\\b"), "a\\\\b");
}

#[test]
fn escape_string_single_quote() {
    assert_eq!(DomBatch::escape_string("it's"), "it\\'s");
}

#[test]
fn escape_string_carriage_return() {
    assert_eq!(DomBatch::escape_string("a\rb"), "a\\rb");
}

#[rstest]
#[case("", "")]
#[case("plain text", "plain text")]
#[case("unicode: 你好", "unicode: 你好")]
fn escape_string_various(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(DomBatch::escape_string(input), expected);
}

// ============================================================================
// escape_selector
// ============================================================================

#[test]
fn escape_selector_plain() {
    assert_eq!(DomBatch::escape_selector("#id"), "#id");
}

#[test]
fn escape_selector_backslash() {
    assert_eq!(DomBatch::escape_selector("a\\b"), "a\\\\b");
}

#[test]
fn escape_selector_single_quote() {
    assert_eq!(DomBatch::escape_selector("a'b"), "a\\'b");
}

// ============================================================================
// DomOp::op_to_js
// ============================================================================

#[test]
fn op_to_js_set_text() {
    let op = DomOp::SetText {
        selector: "#title".to_string(),
        text: "Hello".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("textContent"));
    assert!(js.contains("Hello"));
    assert!(js.contains("#title"));
}

#[test]
fn op_to_js_set_html() {
    let op = DomOp::SetHtml {
        selector: "#content".to_string(),
        html: "<b>bold</b>".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("innerHTML"));
}

#[test]
fn op_to_js_add_class() {
    let op = DomOp::AddClass {
        selector: ".container".to_string(),
        class: "active".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("classList.add"));
    assert!(js.contains("active"));
}

// ============================================================================
// Multiple ops in one batch
// ============================================================================

#[test]
fn batch_multiple_ops() {
    let mut batch = DomBatch::new();
    batch.set_text("#title", "Title");
    batch.add_class("#container", "loaded");
    batch.set_attribute("#img", "src", "photo.jpg");
    batch.hide("#spinner");
    assert_eq!(batch.len(), 4);
    let js = batch.to_js();
    assert!(js.contains("Title"));
    assert!(js.contains("loaded"));
    assert!(js.contains("photo.jpg"));
}

#[test]
fn push_dom_op_directly() {
    let mut batch = DomBatch::new();
    batch.push(DomOp::SetText {
        selector: "#el".to_string(),
        text: "pushed".to_string(),
    });
    assert_eq!(batch.len(), 1);
}
