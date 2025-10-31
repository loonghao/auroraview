# AuroraView Testing Quick Reference

## Running Tests

### All Tests
```bash
just test
```

### Specific Test File
```bash
just test-file tests/test_basic.py
```

### With Marker
```bash
just test-marker ui
```

### Fast Tests Only
```bash
just test-fast
```

### With Coverage
```bash
just test-cov
```

## Test Markers

- `@pytest.mark.ui` - UI tests requiring display
- `@pytest.mark.slow` - Slow running tests
- `@pytest.mark.maya` - Maya-specific tests
- `@pytest.mark.headless` - Tests without display
- `@pytest.mark.integration` - Integration tests
- `@pytest.mark.unit` - Unit tests

## Common Fixtures

### webview
Basic WebView instance for testing.

### webview_bot
High-level automation API for WebView testing.

### test_html
Sample HTML for testing.

### webview_with_html
WebView pre-loaded with test HTML.

## WebViewBot Methods

- `click(selector)` - Click element
- `type(selector, text)` - Type text
- `drag(selector, offset)` - Drag element
- `wait_for_event(event, timeout)` - Wait for event
- `assert_event_emitted(event)` - Assert event was emitted
- `element_exists(selector)` - Check if element exists
- `get_element_text(selector)` - Get element text

