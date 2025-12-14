//! DOM operations tests

use auroraview_core::dom::DomBatch;

#[test]
fn test_empty_batch() {
    let batch = DomBatch::new();
    assert!(batch.is_empty());
    assert_eq!(batch.to_js(), "(function(){})()");
}

#[test]
fn test_set_text() {
    let mut batch = DomBatch::new();
    batch.set_text("#title", "Hello World");
    assert_eq!(batch.len(), 1);
    let js = batch.to_js();
    assert!(js.contains("textContent"));
    assert!(js.contains("Hello World"));
}

#[test]
fn test_escape_string() {
    assert_eq!(DomBatch::escape_string("hello"), "hello");
    assert_eq!(DomBatch::escape_string("hello\"world"), "hello\\\"world");
    assert_eq!(DomBatch::escape_string("hello\nworld"), "hello\\nworld");
}
