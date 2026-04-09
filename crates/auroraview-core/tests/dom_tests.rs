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

// ============================================================================
// Missing DomOp::op_to_js tests (Forms, Interactions, Input, DOM, Custom)
// ============================================================================

#[test]
fn op_to_js_set_checked() {
    let op = DomOp::SetChecked {
        selector: "#cb".to_string(),
        checked: true,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("checked"));
    assert!(js.contains("true"));
    assert!(js.contains("#cb"));
}

#[test]
fn op_to_js_set_checked_false() {
    let op = DomOp::SetChecked {
        selector: "#toggle".to_string(),
        checked: false,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("false"));
}

#[rstest]
#[case(true)]
#[case(false)]
fn op_to_js_set_checked_parametrized(#[case] checked: bool) {
    let op = DomOp::SetChecked {
        selector: "#chk".to_string(),
        checked,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("#chk"));
}

#[test]
fn op_to_js_set_disabled() {
    let op = DomOp::SetDisabled {
        selector: "#btn".to_string(),
        disabled: true,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("disabled"));
    assert!(js.contains("true"));
}

#[test]
fn op_to_js_set_disabled_enable() {
    let op = DomOp::SetDisabled {
        selector: "#submit".to_string(),
        disabled: false,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("false"));
}

#[test]
fn op_to_js_select_option() {
    let op = DomOp::SelectOption {
        selector: "#country".to_string(),
        value: "cn".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains(".value="));
    assert!(js.contains("cn"));
}

#[test]
fn op_to_js_double_click() {
    let op = DomOp::DoubleClick {
        selector: "#row".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("dblclick"));
    assert!(js.contains("#row"));
    assert!(js.contains("MouseEvent"));
}

#[test]
fn op_to_js_blur() {
    let op = DomOp::Blur {
        selector: "#input".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("blur()"));
}

#[test]
fn op_to_js_scroll_into_view_smooth() {
    let op = DomOp::ScrollIntoView {
        selector: "#target".to_string(),
        smooth: true,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("scrollIntoView"));
    assert!(js.contains("smooth"));
}

#[test]
fn op_to_js_scroll_into_view_auto() {
    let op = DomOp::ScrollIntoView {
        selector: "#target".to_string(),
        smooth: false,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("'auto'"));
}

#[rstest]
#[case(true)]
#[case(false)]
fn op_to_js_scroll_into_view_parametrized(#[case] smooth: bool) {
    let op = DomOp::ScrollIntoView {
        selector: "#el".to_string(),
        smooth,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("#el"));
}

#[test]
fn op_to_js_type_text_with_clear() {
    let op = DomOp::TypeText {
        selector: "#search".to_string(),
        text: "hello".to_string(),
        clear: true,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("value=''"));
    assert!(js.contains("hello"));
    assert!(js.contains("split('')"));
}

#[test]
fn op_to_js_type_text_no_clear() {
    let op = DomOp::TypeText {
        selector: "#search".to_string(),
        text: "append".to_string(),
        clear: false,
    };
    let js = DomBatch::op_to_js(&op);
    assert!(!js.contains("value=''"));
    assert!(js.contains("append"));
}

#[test]
fn op_to_js_clear() {
    let op = DomOp::Clear {
        selector: "#field".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("value=''"));
    assert!(js.contains("dispatchEvent"));
    assert!(js.contains("Event('input'"));
}

#[test]
fn op_to_js_submit_form() {
    let op = DomOp::Submit {
        selector: "form#login".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("submit()") || js.contains(".submit()"));
}

#[test]
fn op_to_js_submit_from_input() {
    let op = DomOp::Submit {
        selector: "#inline-submit".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("#inline-submit"));
}

#[test]
fn op_to_js_append_html() {
    let op = DomOp::AppendHtml {
        selector: "#list".to_string(),
        html: "<li>item</li>".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("insertAdjacentHTML"));
    assert!(js.contains("beforeend"));
    assert!(js.contains("<li>item</li>"));
}

#[test]
fn op_to_js_prepend_html() {
    let op = DomOp::PrependHtml {
        selector: "#list".to_string(),
        html: "<li>first</li>".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("insertAdjacentHTML"));
    assert!(js.contains("afterbegin"));
}

#[test]
fn op_to_js_remove() {
    let op = DomOp::Remove {
        selector: ".old".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains(".remove()"));
}

#[test]
fn op_to_js_empty() {
    let op = DomOp::Empty {
        selector: "#container".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    // Empty sets innerHTML to empty string
    assert!(js.contains("innerHTML") || js.contains("#container"));
}

#[test]
fn op_to_js_raw() {
    let op = DomOp::Raw {
        selector: "#custom".to_string(),
        script: "e.dataset.x = '1';".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("#custom"));
    assert!(js.contains("dataset.x"));
}

#[test]
fn op_to_js_raw_global() {
    let op = DomOp::RawGlobal {
        script: "console.log('global');".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert_eq!(js, "console.log('global');");
}

#[test]
fn op_to_js_raw_global_complex() {
    let op = DomOp::RawGlobal {
        script: "var x = 1 + 2;".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("var x"));
}

#[test]
fn op_to_js_set_styles_single() {
    let op = DomOp::SetStyles {
        selector: "#el".to_string(),
        styles: vec![("color".to_string(), "red".to_string())],
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("style['color']"));
    assert!(js.contains("red"));
}

#[test]
fn op_to_js_styles_multiple() {
    let op = DomOp::SetStyles {
        selector: "#el".to_string(),
        styles: vec![
            ("color".to_string(), "blue".to_string()),
            ("font-size".to_string(), "14px".to_string()),
            ("display".to_string(), "flex".to_string()),
        ],
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("style['color']"));
    assert!(js.contains("style['font-size']"));
    assert!(js.contains("style['display']"));
    assert!(js.contains("blue"));
    assert!(js.contains("14px"));
    assert!(js.contains("flex"));
}

#[test]
fn op_to_js_styles_empty() {
    let op = DomOp::SetStyles {
        selector: "#el".to_string(),
        styles: vec![],
    };
    let js = DomBatch::op_to_js(&op);
    // Empty styles produces querySelector with empty if block
    assert!(js.contains("querySelector('#el')"));
}

// ============================================================================
// Missing convenience method tests
// ============================================================================

#[test]
fn set_checked_convenience() {
    let mut batch = DomBatch::new();
    batch.set_checked("#agree", true);
    let js = batch.to_js();
    assert!(js.contains("checked"));
}

#[test]
fn set_checked_false_convenience() {
    let mut batch = DomBatch::new();
    batch.set_checked("#agree", false);
    let js = batch.to_js();
    assert!(js.contains("false"));
}

#[test]
fn set_disabled_convenience() {
    let mut batch = DomBatch::new();
    batch.set_disabled("#btn", true);
    let js = batch.to_js();
    assert!(js.contains("disabled"));
}

#[test]
fn set_disabled_false_convenience() {
    let mut batch = DomBatch::new();
    batch.set_disabled("#btn", false);
    let js = batch.to_js();
    assert!(js.contains("false"));
}

#[test]
fn double_click_convenience() {
    let mut batch = DomBatch::new();
    batch.double_click("#row");
    assert_eq!(batch.len(), 1);
}

#[test]
fn blur_convenience() {
    let mut batch = DomBatch::new();
    batch.blur("#input");
    assert_eq!(batch.len(), 1);
}

#[test]
fn scroll_into_view_smooth_convenience() {
    let mut batch = DomBatch::new();
    batch.scroll_into_view("#header", true);
    assert_eq!(batch.len(), 1);
}

#[test]
fn scroll_into_view_auto_convenience() {
    let mut batch = DomBatch::new();
    batch.scroll_into_view("#header", false);
    assert_eq!(batch.len(), 1);
}

#[test]
fn type_text_with_clear() {
    let mut batch = DomBatch::new();
    batch.type_text("#input", "hello", true);
    assert_eq!(batch.len(), 1);
}

#[test]
fn type_text_without_clear() {
    let mut batch = DomBatch::new();
    batch.type_text("#input", "world", false);
    assert_eq!(batch.len(), 1);
}

#[test]
fn clear_input_convenience() {
    let mut batch = DomBatch::new();
    batch.clear_input("#field");
    assert_eq!(batch.len(), 1);
}

#[test]
fn submit_convenience() {
    let mut batch = DomBatch::new();
    batch.submit("#my-form");
    assert_eq!(batch.len(), 1);
}

#[test]
fn append_html_convenience() {
    let mut batch = DomBatch::new();
    batch.append_html("#list", "<li>new</li>");
    assert_eq!(batch.len(), 1);
}

#[test]
fn prepend_html_convenience() {
    let mut batch = DomBatch::new();
    batch.prepend_html("#list", "<li>first</li>");
    assert_eq!(batch.len(), 1);
}

#[test]
fn remove_convenience() {
    let mut batch = DomBatch::new();
    batch.remove(".deprecated");
    assert_eq!(batch.len(), 1);
}

#[test]
fn empty_convenience() {
    let mut batch = DomBatch::new();
    batch.empty("#container");
    assert_eq!(batch.len(), 1);
}

#[test]
fn raw_convenience() {
    let mut batch = DomBatch::new();
    batch.raw("#el", "e.title = 'x';");
    assert_eq!(batch.len(), 1);
}

#[test]
fn raw_global_convenience() {
    let mut batch = DomBatch::new();
    batch.raw_global("window.global = 1;");
    assert_eq!(batch.len(), 1);
}

// ============================================================================
// DomOp PartialEq (derived)
// ============================================================================

#[test]
fn dom_op_equality_same_set_text() {
    let a = DomOp::SetText {
        selector: "#x".to_string(),
        text: "a".to_string(),
    };
    let b = DomOp::SetText {
        selector: "#x".to_string(),
        text: "a".to_string(),
    };
    assert_eq!(a, b);
}

#[test]
fn dom_op_inequality_different_selector() {
    let a = DomOp::SetText {
        selector: "#x".to_string(),
        text: "a".to_string(),
    };
    let b = DomOp::SetText {
        selector: "#y".to_string(),
        text: "a".to_string(),
    };
    assert_ne!(a, b);
}

#[test]
fn dom_op_inequality_different_variant() {
    let a = DomOp::Show {
        selector: "#x".to_string(),
    };
    let b = DomOp::Hide {
        selector: "#x".to_string(),
    };
    assert_ne!(a, b);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn set_text_with_special_chars() {
    let mut batch = DomBatch::new();
    batch.set_text("#el", "<script>alert(1)</script>");
    let js = batch.to_js();
    // Should escape the string properly for JS
    assert!(js.contains("textContent"));
}

#[test]
fn set_html_with_nested_quotes() {
    let mut batch = DomBatch::new();
    batch.set_html("#el", r#"{"key": "value"}"#);
    let js = batch.to_js();
    assert!(js.contains("innerHTML"));
}

#[test]
fn op_to_js_empty_selector() {
    let op = DomOp::Click {
        selector: "".to_string(),
    };
    let _js = DomBatch::op_to_js(&op); // Should not panic
}

#[test]
fn op_to_js_selector_with_dots_and_brackets() {
    let op = DomOp::Show {
        selector: "#id.class[attr]".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    assert!(js.contains("#id.class[attr]"));
}

#[test]
fn raw_global_does_not_wrap_in_iife() {
    let op = DomOp::RawGlobal {
        script: "let x = 1;".to_string(),
    };
    let js = DomBatch::op_to_js(&op);
    // RawGlobal should NOT be wrapped in IIFE
    assert!(!js.starts_with("(function(){"));
    assert_eq!(js, "let x = 1;");
}

#[test]
fn regular_op_wraps_in_iife_when_batched() {
    let mut batch = DomBatch::new();
    batch.raw_global("let x = 1;");
    let js = batch.to_js();
    // RawGlobal in a batch should be inside the IIFE
    assert!(js.starts_with("(function(){"));
    assert!(js.ends_with("})()"));
}
