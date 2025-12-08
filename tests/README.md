# AuroraView Testing Framework

A comprehensive testing framework for AuroraView WebView applications with multiple backend support.

## Overview

AuroraView provides a unified testing framework with multiple backends for different testing scenarios:

- **HeadlessWebView** - Unified API for headless testing
- **Playwright Backend** - Recommended for CI/CD (cross-platform)
- **Xvfb Backend** - Real WebView testing on Linux
- **WebView2 CDP Backend** - Real WebView2 testing on Windows

## Quick Start

### Installation

```bash
# Install test dependencies
pip install pytest playwright
playwright install chromium
```

### Basic Test Example

```python
from auroraview.testing import HeadlessWebView

def test_button_click():
    """Test clicking a button."""
    with HeadlessWebView.playwright() as webview:
        webview.load_html("""
            <button id="btn" onclick="document.body.innerHTML='Clicked!'">Click</button>
        """)
        webview.click("#btn")
        assert "Clicked" in webview.content()
```

## HeadlessWebView API

### Creating Instances

```python
from auroraview.testing import HeadlessWebView

# Auto-detect best backend (recommended)
with HeadlessWebView.auto() as webview:
    webview.goto("https://example.com")

# Explicitly use Playwright
with HeadlessWebView.playwright() as webview:
    webview.goto("https://example.com")

# Use Xvfb on Linux
with HeadlessWebView.virtual_display() as webview:
    webview.goto("https://example.com")

# Connect to WebView2 via CDP (Windows)
with HeadlessWebView.webview2_cdp("http://localhost:9222") as webview:
    webview.goto("https://example.com")
```

### Navigation

```python
webview.goto("https://example.com")
webview.load_html("<h1>Hello</h1>")
webview.load_url("https://example.com")
webview.reload()
webview.go_back()
webview.go_forward()
```

### Element Interaction

```python
webview.click("#button")
webview.fill("#input", "text")
webview.type("#input", "text", delay=100)
webview.check("#checkbox")
webview.uncheck("#checkbox")
webview.select_option("#select", "value")
webview.hover("#element")
webview.focus("#element")
```

### Element Queries

```python
text = webview.text("#element")
html = webview.inner_html("#element")
value = webview.input_value("#input")
attr = webview.get_attribute("#element", "href")
visible = webview.is_visible("#element")
enabled = webview.is_enabled("#button")
checked = webview.is_checked("#checkbox")
exists = webview.exists("#element")
count = webview.count(".items")
```

### Waiting

```python
webview.wait_for_selector("#element", timeout=5000)
webview.wait_for_selector("#element", state="visible")
webview.wait_for_selector("#element", state="hidden")
webview.wait_for_load_state("load")
webview.wait_for_load_state("domcontentloaded")
```

### Screenshots

```python
webview.screenshot("screenshot.png")
webview.screenshot("full.png", full_page=True)
bytes_data = webview.screenshot()  # Returns bytes
```

### JavaScript Execution

```python
result = webview.evaluate("1 + 1")
webview.evaluate("document.title = 'New Title'")
```

### Page Information

```python
title = webview.title()
url = webview.url()
content = webview.content()
```

## Pytest Integration

### Using Fixtures

```python
import pytest

def test_with_fixture(headless_webview):
    """Test using the headless_webview fixture."""
    headless_webview.load_html("<h1>Test</h1>")
    assert headless_webview.text("h1") == "Test"

def test_playwright_specific(playwright_webview):
    """Test using Playwright-specific fixture."""
    playwright_webview.goto("https://example.com")
    playwright_webview.screenshot("test.png")
```

### Custom Configuration

```python
from auroraview.testing import HeadlessWebView, HeadlessOptions

def test_custom_options():
    options = HeadlessOptions(
        headless=True,
        width=1920,
        height=1080,
        timeout=10000,
        slow_mo=100  # Slow down for debugging
    )
    with HeadlessWebView.playwright(options=options) as webview:
        webview.goto("https://example.com")
```

## Running Tests

```bash
# Run all Python tests
uv run pytest tests/python/ -v

# Run headless WebView tests
uv run pytest tests/python/integration/test_headless_webview.py -v

# Run with coverage
uv run pytest tests/python/ --cov=auroraview

# Using nox
uvx nox -s pytest          # Without Qt
uvx nox -s pytest-qt       # With Qt
uvx nox -s pytest-all      # All tests
```

## CI/CD Configuration

### GitHub Actions

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      
      - name: Install dependencies
        run: |
          pip install -e ".[dev]"
          pip install playwright
          playwright install chromium
      
      - name: Run tests
        run: pytest tests/ -v
```

### Linux with Xvfb

```yaml
- name: Run tests with Xvfb
  run: |
    sudo apt-get install -y xvfb
    xvfb-run pytest tests/ -v
```

## Test Markers

```python
@pytest.mark.webview      # WebView tests
@pytest.mark.playwright   # Playwright-specific tests
@pytest.mark.xvfb         # Xvfb-specific tests (Linux only)
@pytest.mark.slow         # Slow running tests
@pytest.mark.integration  # Integration tests
```

## Backend Comparison

| Feature | Playwright | Xvfb | WebView2 CDP |
|---------|------------|------|--------------|
| Platform | All | Linux | Windows |
| Real WebView | No | Yes | Yes |
| Headless | Yes | Yes | Partial |
| Screenshots | Yes | Yes | Yes |
| Network Interception | Yes | No | Yes |
| CI/CD Ready | Yes | Yes | Limited |
| Speed | Fast | Medium | Medium |

## Troubleshooting

### Playwright not installed

```bash
pip install playwright
playwright install chromium
```

### Xvfb not available (Linux)

```bash
sudo apt-get install xvfb
xvfb-run pytest tests/ -v
```

### WebView2 CDP connection failed

Ensure WebView2 is running with remote debugging enabled:
```
--remote-debugging-port=9222
```
