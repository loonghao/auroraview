# Testing Framework

AuroraView provides a comprehensive testing framework with multiple approaches for testing WebView applications.

## Overview

The testing framework includes:

| Component | Purpose | Use Case |
|-----------|---------|----------|
| **Decorators** | Conditional test execution | Skip tests based on environment |
| **Generators** | Random test data | Fuzz testing, edge cases |
| **Property Testing** | Hypothesis strategies | Property-based testing |
| **Snapshot Testing** | Regression detection | UI/output stability |
| **Headless WebView** | Browser automation | E2E testing |
| **Midscene.js** | AI-powered testing | Natural language UI automation |

## Quick Start

### Test Decorators

```python
from auroraview.testing import (
    requires_qt,
    requires_windows,
    slow_test,
    integration_test,
    with_timeout,
)

@requires_qt
def test_qt_webview():
    """Only runs if Qt is available."""
    from auroraview.integration.qt import QtWebView
    # Test Qt functionality

@requires_windows
@slow_test
def test_webview2_performance():
    """Only runs on Windows, marked as slow."""
    # Performance test

@integration_test
@with_timeout(60)
def test_full_workflow():
    """Integration test with 60s timeout."""
    # End-to-end test
```

### Test Data Generators

```python
from auroraview.testing import (
    random_html,
    random_html_page,
    random_js_value,
    random_event_name,
    random_selector,
    generate_test_dataset,
)

# Generate random HTML
html = random_html("div", content="Hello", attrs={"class": "test"})
# <div class="test">Hello</div>

# Generate complete HTML page
page = random_html_page(title="Test", body_content="<h1>Hello</h1>")

# Generate random JSON-serializable values
value = random_js_value()  # string, number, bool, array, object, or null

# Generate random event names
event = random_event_name(prefix="user", namespace="api")
# "api:user_abc123"

# Generate test datasets
dataset = generate_test_dataset(count=10, data_type="events")
```

### Snapshot Testing

```python
from auroraview.testing import SnapshotTest

def test_component_output(tmp_path):
    snapshot = SnapshotTest(tmp_path / "snapshots")

    # Test HTML output
    html = render_component()
    snapshot.assert_match_html(html, "component.html")

    # Test JSON output
    data = get_api_response()
    snapshot.assert_match_json(data, "response.json")
```

### Property-Based Testing

```python
from hypothesis import given
from auroraview.testing.property_testing import (
    html_elements,
    js_values,
    event_names,
    property_test,
)

@property_test(max_examples=100)
@given(value=js_values())
def test_json_roundtrip(value):
    """Test that values survive JSON serialization."""
    import json
    assert json.loads(json.dumps(value)) == value

@given(html=html_elements(max_depth=2))
def test_html_parsing(html):
    """Test HTML parsing with random inputs."""
    assert "<" in html and ">" in html
```

## Decorators Reference

### Environment Checks

| Decorator | Description |
|-----------|-------------|
| `@requires_qt` | Skip if Qt (PySide6/PySide2) not available |
| `@requires_cdp(url)` | Skip if CDP endpoint not available |
| `@requires_gallery` | Skip if packed gallery not available |
| `@requires_playwright` | Skip if Playwright not installed |
| `@requires_webview2` | Skip if WebView2 runtime not available |
| `@requires_windows` | Skip if not on Windows |
| `@requires_linux` | Skip if not on Linux |
| `@requires_macos` | Skip if not on macOS |
| `@requires_env(var, value)` | Skip if env var not set/matched |

### Test Categories

| Decorator | Description |
|-----------|-------------|
| `@slow_test` | Mark as slow (may be skipped in quick runs) |
| `@integration_test` | Mark as integration test |
| `@unit_test` | Mark as unit test |
| `@smoke_test` | Mark as smoke test (quick sanity check) |
| `@flaky_test(reruns=3)` | Mark as flaky with auto-retry |

### Test Setup

| Decorator | Description |
|-----------|-------------|
| `@with_timeout(seconds)` | Set test timeout |
| `@parametrize_examples(ids)` | Parametrize with example IDs |
| `@serial_test` | Run serially (not in parallel) |
| `@skip_if(condition, reason)` | Skip if condition is True |
| `@xfail_if(condition, reason)` | Expected failure if condition |

## Generators Reference

### HTML Generators

```python
from auroraview.testing import (
    random_string,
    random_html,
    random_html_page,
    random_form_html,
)

# Random string
s = random_string(length=10, charset="abc123")

# Random HTML element
html = random_html(
    tag="div",
    content="Hello",
    attrs={"class": "container", "id": "main"},
    children=["<span>Child 1</span>", "<span>Child 2</span>"],
)

# Complete HTML page
page = random_html_page(
    title="Test Page",
    body_content="<h1>Hello</h1>",
    styles="body { margin: 0; }",
    scripts="console.log('loaded');",
)

# HTML form
form = random_form_html(
    fields=[
        {"name": "email", "type": "email", "label": "Email"},
        {"name": "password", "type": "password", "label": "Password"},
    ],
    action="/login",
    method="post",
)
```

### JavaScript Value Generators

```python
from auroraview.testing import (
    random_js_value,
    random_event_payload,
    random_api_method,
    random_api_params,
)

# Random JSON-serializable value
value = random_js_value(value_type="object", max_depth=3)

# Random event payload
payload = random_event_payload(event_type="click")
# {"timestamp": 1234567890, "type": "click", "x": 100, "y": 200, ...}

# Random API method
method = random_api_method(namespace="api")
# "api.get_user"

# Random API parameters
params = random_api_params(param_count=3, as_dict=True)
# {"key1": "value1", "key2": 123, "key3": true}
```

### Selector Generators

```python
from auroraview.testing import random_selector, random_xpath

# CSS selectors
id_sel = random_selector("id")      # "#abc123"
class_sel = random_selector("class")  # ".xyz789"
tag_sel = random_selector("tag")    # "div"
attr_sel = random_selector("attr")  # '[data-id="abc"]'

# XPath expressions
xpath = random_xpath("button")
# "//button[@id='abc123']"
```

### URL Generators

```python
from auroraview.testing import random_url, random_file_url

# HTTP/HTTPS URLs
url = random_url(scheme="https", domain="example.com")
# "https://example.com/path/to/resource"

# File URLs
file_url = random_file_url(extension="html", directory="/tmp/test")
# "file:///tmp/test/abc123.html"
```

## Snapshot Testing

### Basic Usage

```python
from auroraview.testing import SnapshotTest

snapshot = SnapshotTest("tests/snapshots")

# Text comparison
snapshot.assert_match(output, "output.txt")

# JSON comparison (sorted keys, formatted)
snapshot.assert_match_json(data, "data.json")

# HTML comparison (normalized whitespace)
snapshot.assert_match_html(html, "page.html")

# Hash comparison (for large content)
snapshot.assert_hash_match(large_content, "large_file")
```

### Updating Snapshots

```bash
# Update all snapshots
UPDATE_SNAPSHOTS=1 pytest tests/

# Or use pytest flag (if configured)
pytest tests/ --update-snapshots
```

### Pytest Integration

```python
# conftest.py
import pytest
from auroraview.testing import pytest_snapshot_fixture

@pytest.fixture
def snapshot(request):
    return pytest_snapshot_fixture(request)

# test_example.py
def test_output(snapshot):
    output = generate_output()
    snapshot.assert_match(output, "expected_output.txt")
```

### Screenshot Snapshots

```python
from auroraview.testing import ScreenshotSnapshot

screenshot = ScreenshotSnapshot("tests/screenshots", threshold=0.01)

# Compare screenshots (requires Pillow for pixel comparison)
screenshot.assert_screenshot_match(png_data, "homepage.png")
```

## Property-Based Testing

Property-based testing generates random inputs to find edge cases.

### Installation

```bash
pip install hypothesis
```

### Strategies

```python
from auroraview.testing.property_testing import (
    html_tags,
    html_elements,
    js_primitives,
    js_values,
    event_names,
    api_methods,
    css_selectors,
    urls,
)

# Use with @given decorator
from hypothesis import given

@given(tag=html_tags())
def test_tag_is_valid(tag):
    assert tag in ["div", "span", "p", ...]

@given(value=js_values(max_depth=2))
def test_json_serializable(value):
    import json
    json.dumps(value)  # Should not raise

@given(selector=css_selectors())
def test_selector_format(selector):
    assert selector.startswith(("#", ".", "[")) or selector.isalpha()
```

### Custom Settings

```python
from auroraview.testing.property_testing import property_test

@property_test(max_examples=200, deadline=None)
@given(html=html_elements())
def test_html_parsing(html):
    # Test with more examples, no timeout
    parse_html(html)
```

## Midscene.js AI-Powered Testing

[Midscene.js](https://midscenejs.com/) is an AI-powered UI automation SDK by ByteDance that enables natural language-driven testing.

### Installation

```bash
# Playwright is required for Midscene integration
pip install playwright
playwright install chromium

# Set up AI model API key
export OPENAI_API_KEY=your-api-key
# Or use other models (Qwen, Gemini, etc.)
```

### Basic Usage

```python
from auroraview.testing import MidsceneAgent, MidsceneConfig
from playwright.async_api import async_playwright

async def test_with_ai():
    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()
        await page.goto("https://example.com")

        # Create AI agent
        async with MidsceneAgent(page) as agent:
            # Natural language actions
            await agent.ai_act('click the login button')
            await agent.ai_act('type "test@example.com" in the email field')
            await agent.ai_act('type "password123" in the password field')
            await agent.ai_act('click submit')

            # AI-powered assertions
            await agent.ai_assert('the dashboard is visible')
            await agent.ai_assert('welcome message contains "test@example.com"')

            # Extract data with AI
            user_info = await agent.ai_query({
                'name': 'string, user display name',
                'email': 'string, user email address',
                'role': 'string, user role'
            })

        await browser.close()
```

### Core Methods

| Method | Description | Example |
|--------|-------------|---------|
| `ai_act(instruction)` | Execute natural language action | `await agent.ai_act('click the blue button')` |
| `ai_query(demand)` | Extract structured data | `await agent.ai_query('string[], product names')` |
| `ai_assert(condition)` | Verify with natural language | `await agent.ai_assert('form is submitted')` |
| `ai_wait_for(condition)` | Wait for condition | `await agent.ai_wait_for('loading spinner disappears')` |
| `ai_locate(description)` | Find element by description | `await agent.ai_locate('the submit button')` |

### Configuration

```python
from auroraview.testing import MidsceneConfig

config = MidsceneConfig(
    # Model settings
    model_name="gpt-4o",           # or "qwen-vl-plus", "gemini-1.5-flash"
    model_family="openai",          # auto-detected from model_name
    api_key="your-api-key",         # or use OPENAI_API_KEY env var
    base_url=None,                  # custom API endpoint

    # Behavior
    timeout=60000,                  # 60 seconds
    cacheable=True,                 # cache AI responses
    debug=False,                    # verbose logging

    # Context options
    dom_included=False,             # include DOM info
)
```

### Pytest Integration

```python
# conftest.py
import pytest
from auroraview.testing import MidscenePlaywrightFixture

@pytest.fixture
async def ai(page):
    """AI-powered testing fixture."""
    fixture = MidscenePlaywrightFixture(page)
    yield fixture
    fixture.close()

# test_example.py
async def test_login_flow(ai):
    await ai.act('click login button')
    await ai.act('enter "user@example.com" in email field')
    await ai.act('enter "password" in password field')
    await ai.act('click submit')
    await ai.assert_('dashboard is visible')
```

### Data Extraction Examples

```python
# Extract list of strings
products = await agent.ai_query('string[], all product names on the page')

# Extract structured data
items = await agent.ai_query({
    'title': 'string, product title',
    'price': 'number, price in dollars',
    'inStock': 'boolean, availability'
})

# Extract with DOM context (for non-visible attributes)
links = await agent.ai_query(
    '{text: string, href: string}[], all navigation links',
    dom_included=True
)
```

### Supported AI Models

| Provider | Model | Environment Variables |
|----------|-------|----------------------|
| OpenAI | gpt-4o, gpt-4o-mini | `OPENAI_API_KEY` |
| Qwen | qwen-vl-plus, qwen-vl-max | `MIDSCENE_MODEL_API_KEY`, `MIDSCENE_MODEL_BASE_URL` |
| Gemini | gemini-1.5-flash, gemini-1.5-pro | `MIDSCENE_MODEL_API_KEY` |
| Claude | claude-3-5-sonnet | `MIDSCENE_MODEL_API_KEY` |

## Best Practices

### 1. Use Appropriate Test Categories

```python
@smoke_test
def test_import():
    """Quick sanity check."""
    import auroraview
    assert auroraview is not None

@unit_test
def test_function():
    """Isolated unit test."""
    result = my_function(1, 2)
    assert result == 3

@integration_test
def test_workflow():
    """Full workflow test."""
    # Multiple components working together
```

### 2. Handle Environment Dependencies

```python
@requires_qt
@requires_windows
def test_qt_on_windows():
    """Only runs if Qt available AND on Windows."""
    pass

@requires_env("CI")
def test_ci_only():
    """Only runs in CI environment."""
    pass
```

### 3. Use Generators for Edge Cases

```python
def test_html_rendering():
    """Test with various HTML inputs."""
    for _ in range(100):
        html = random_html_page()
        result = render(html)
        assert result is not None
```

### 4. Combine with Snapshots

```python
def test_api_response(snapshot):
    """Ensure API response format is stable."""
    response = api.get_data()
    snapshot.assert_match_json(response, "api_response.json")
```

### 5. Property Testing for Robustness

```python
@given(value=js_values())
def test_serialization_roundtrip(value):
    """Ensure serialization is reversible."""
    serialized = serialize(value)
    deserialized = deserialize(serialized)
    assert deserialized == value
```

## Browser Automation

AuroraView provides a unified browser automation abstraction layer for testing.

### Automation API

```python
from auroraview import WebView
from auroraview.utils.automation import Automation

# Create WebView with automation
webview = WebView(title="Test", width=800, height=600)

# Create automation instance
auto = Automation(webview)

# Navigate and interact
await auto.goto("https://example.com")
await auto.click("#submit-button")
await auto.fill("#email", "test@example.com")

# Extract data
title = await auto.get_text("h1")
links = await auto.query_selector_all("a")
```

### Local WebView Backend

```python
from auroraview.utils.automation import LocalWebViewBackend

# Use local WebView for automation
backend = LocalWebViewBackend(webview)

# Execute JavaScript
result = await backend.evaluate("document.title")

# Take screenshot
screenshot = await backend.screenshot()
```

## Example Demos

AuroraView includes several example demos for testing:

| Demo | Description |
|------|-------------|
| `automation_demo.py` | Browser automation abstraction layer |
| `midscene_demo.py` | AI-powered testing with Midscene.js |
| `dom_batch_demo.py` | Batch DOM operations |
| `event_timer_demo.py` | Event and timer testing |
| `channel_streaming_demo.py` | IPC channel streaming |
| `command_registry_demo.py` | Command registry patterns |

Run demos with:

```bash
python examples/automation_demo.py
python examples/midscene_demo.py
```

## References

- [pytest Documentation](https://docs.pytest.org/)
- [Hypothesis Documentation](https://hypothesis.readthedocs.io/)
- [Snapshot Testing Best Practices](https://jestjs.io/docs/snapshot-testing)
- [Property-Based Testing Introduction](https://hypothesis.works/articles/what-is-hypothesis/)
- [Midscene.js Documentation](https://midscenejs.com/)
