# Testing Frameworks Guide

## Overview

This guide explains the testing frameworks we'll use in the AuroraView project and provides examples of how to use them effectively.

## Testing Frameworks

### 1. `rstest` - Parameterized Testing

**Purpose:** Run the same test with different input values

**Installation:**
```toml
[dev-dependencies]
rstest = "0.23"
```

**Example:**

```rust
use rstest::rstest;

#[rstest]
#[case("console.log('test')", true)]
#[case("", false)]
#[case("alert('hello')", true)]
fn test_eval_js_validation(#[case] script: &str, #[case] expected_valid: bool) {
    let is_valid = !script.is_empty();
    assert_eq!(is_valid, expected_valid);
}
```

**Benefits:**
- [OK] Reduces code duplication
- [OK] Makes test cases explicit and readable
- [OK] Easy to add new test cases

### 2. `test-case` - Table-Driven Tests

**Purpose:** Similar to `rstest` but with a different syntax

**Installation:**
```toml
[dev-dependencies]
test-case = "3.3"
```

**Example:**

```rust
use test_case::test_case;

#[test_case("http://example.com", true ; "valid http url")]
#[test_case("https://example.com", true ; "valid https url")]
#[test_case("invalid", false ; "invalid url")]
fn test_url_validation(url: &str, expected: bool) {
    let is_valid = url.starts_with("http://") || url.starts_with("https://");
    assert_eq!(is_valid, expected);
}
```

**Benefits:**
- [OK] Named test cases for better error messages
- [OK] Concise syntax
- [OK] Good for simple parameterized tests

### 3. `proptest` - Property-Based Testing

**Purpose:** Generate random test inputs to find edge cases

**Installation:**
```toml
[dev-dependencies]
proptest = "1.5"
```

**Example:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_message_queue_order(messages in prop::collection::vec(".*", 0..100)) {
        let queue = MessageQueue::new();
        
        // Push all messages
        for msg in &messages {
            queue.push(WebViewMessage::EvalJs(msg.clone()));
        }
        
        // Pop all messages and verify order
        for expected in &messages {
            if let Some(WebViewMessage::EvalJs(actual)) = queue.pop() {
                prop_assert_eq!(&actual, expected);
            }
        }
        
        // Queue should be empty
        prop_assert!(queue.is_empty());
    }
}
```

**Benefits:**
- [OK] Discovers edge cases you didn't think of
- [OK] Tests properties rather than specific values
- [OK] Shrinks failing inputs to minimal examples

### 4. `mockall` - Mocking Framework

**Purpose:** Create mock objects for testing in isolation

**Installation:**
```toml
[dev-dependencies]
mockall = "0.13"
```

**Example:**

```rust
use mockall::*;
use mockall::predicate::*;

#[automock]
trait WebViewOperations {
    fn evaluate_script(&self, script: &str) -> Result<(), String>;
}

#[test]
fn test_emit_event() {
    let mut mock = MockWebViewOperations::new();
    
    // Expect evaluate_script to be called once with specific script
    mock.expect_evaluate_script()
        .with(predicate::eq("window.dispatchEvent(...)"))
        .times(1)
        .returning(|_| Ok(()));
    
    // Test code that uses the mock
    let result = mock.evaluate_script("window.dispatchEvent(...)");
    assert!(result.is_ok());
}
```

**Benefits:**
- [OK] Test components in isolation
- [OK] Verify interactions between components
- [OK] Control behavior of dependencies

### 5. `serial_test` - Serial Test Execution

**Purpose:** Run tests serially instead of in parallel

**Installation:**
```toml
[dev-dependencies]
serial_test = "3.2"
```

**Example:**

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_global_state_1() {
    // This test modifies global state
    // It will run serially with other #[serial] tests
}

#[test]
#[serial]
fn test_global_state_2() {
    // This test also modifies global state
    // It will run after test_global_state_1
}
```

**Benefits:**
- [OK] Prevents race conditions in tests
- [OK] Useful for tests that modify global state
- [OK] Simple attribute-based API

## Test Organization Patterns

### Unit Tests

**Location:** `tests/unit/`

**Purpose:** Test individual functions and modules in isolation

**Example:**

```rust
// tests/unit/webview/message_queue_tests.rs

use auroraview_core::webview::{MessageQueue, WebViewMessage};

#[test]
fn test_message_queue_basic() {
    let queue = MessageQueue::new();
    assert!(queue.is_empty());
    
    queue.push(WebViewMessage::EvalJs("test".to_string()));
    assert!(!queue.is_empty());
    
    let msg = queue.pop();
    assert!(msg.is_some());
    assert!(queue.is_empty());
}
```

### Integration Tests

**Location:** `tests/integration/`

**Purpose:** Test multiple components working together

**Example:**

```rust
// tests/integration/standalone_mode_tests.rs

use auroraview_core::webview::{WebViewConfig, AuroraView};

#[test]
fn test_standalone_mode_creation() {
    let config = WebViewConfig {
        title: "Test".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    
    // Test that standalone mode can be created
    // (This would require more setup in a real test)
}
```

### Common Test Utilities

**Location:** `tests/common/`

**Purpose:** Shared test helpers and fixtures

**Example:**

```rust
// tests/common/mod.rs

pub fn create_test_config() -> WebViewConfig {
    WebViewConfig {
        title: "Test WebView".to_string(),
        width: 800,
        height: 600,
        url: Some("http://localhost:8080".to_string()),
        ..Default::default()
    }
}

pub fn create_test_message_queue() -> MessageQueue {
    let queue = MessageQueue::new();
    queue.push(WebViewMessage::EvalJs("console.log('test')".to_string()));
    queue
}
```

## Best Practices

### 1. Test Naming

```rust
// [OK] Good: Descriptive name that explains what is being tested
#[test]
fn test_message_queue_maintains_fifo_order() { }

// [ERROR] Bad: Vague name
#[test]
fn test_queue() { }
```

### 2. Arrange-Act-Assert Pattern

```rust
#[test]
fn test_emit_event() {
    // Arrange: Set up test data
    let queue = MessageQueue::new();
    let event_name = "test_event";
    let data = json!({"key": "value"});
    
    // Act: Perform the action
    queue.push(WebViewMessage::EmitEvent {
        event_name: event_name.to_string(),
        data,
    });
    
    // Assert: Verify the result
    assert_eq!(queue.len(), 1);
}
```

### 3. Use Fixtures for Common Setup

```rust
use rstest::*;

#[fixture]
fn test_queue() -> MessageQueue {
    MessageQueue::new()
}

#[rstest]
fn test_with_fixture(test_queue: MessageQueue) {
    assert!(test_queue.is_empty());
}
```

### 4. Test Error Cases

```rust
#[test]
#[should_panic(expected = "Invalid URL")]
fn test_invalid_url_panics() {
    load_url("not a url");
}

#[test]
fn test_invalid_url_returns_error() {
    let result = try_load_url("not a url");
    assert!(result.is_err());
}
```

### 5. Use Property-Based Testing for Edge Cases

```rust
proptest! {
    #[test]
    fn test_queue_handles_any_string(s in ".*") {
        let queue = MessageQueue::new();
        queue.push(WebViewMessage::EvalJs(s.clone()));
        
        if let Some(WebViewMessage::EvalJs(result)) = queue.pop() {
            prop_assert_eq!(result, s);
        }
    }
}
```

## Running Tests

### Run All Tests

```bash
just test
```

### Run Unit Tests Only

```bash
just test-unit
```

### Run Integration Tests Only

```bash
just test-integration
```

### Run Specific Test File

```bash
cargo test --test message_queue_tests
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Tests in Watch Mode

```bash
just test-watch
```

## Test Coverage

### Generate Coverage Report

```bash
just coverage-rust
```

### View Coverage in Browser

```bash
open target/tarpaulin/index.html
```

## Troubleshooting

### Tests Fail Due to Race Conditions

**Solution:** Use `#[serial]` attribute

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_with_global_state() {
    // Test code
}
```

### Tests Are Slow

**Solution:** Use `#[ignore]` for slow tests

```rust
#[test]
#[ignore]
fn slow_integration_test() {
    // Slow test code
}
```

Run ignored tests with:
```bash
cargo test -- --ignored
```

### Mock Expectations Not Met

**Solution:** Ensure mock is dropped before assertions

```rust
#[test]
fn test_with_mock() {
    {
        let mut mock = MockWebViewOperations::new();
        mock.expect_evaluate_script()
            .times(1)
            .returning(|_| Ok(()));
        
        // Use mock
        mock.evaluate_script("test");
    } // Mock is dropped here, expectations are verified
    
    // Additional assertions
}
```

## Resources

- [rstest documentation](https://docs.rs/rstest/)
- [proptest book](https://proptest-rs.github.io/proptest/)
- [mockall documentation](https://docs.rs/mockall/)
- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

---

**Last Updated:** 2025-10-29  
**Maintainer:** AuroraView Team

