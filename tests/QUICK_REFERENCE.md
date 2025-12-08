# AuroraView Testing Quick Reference

## Running Tests

### All Tests
```bash
uvx nox -s pytest-all
```

### Without Qt
```bash
uvx nox -s pytest
```

### With Qt
```bash
uvx nox -s pytest-qt
```

### Specific Test File
```bash
uv run pytest tests/python/integration/test_headless_webview.py -v
```

### With Coverage
```bash
uvx nox -s coverage
```

## Test Markers

- `@pytest.mark.webview` - WebView tests
- `@pytest.mark.playwright` - Playwright-specific tests
- `@pytest.mark.xvfb` - Xvfb tests (Linux only)
- `@pytest.mark.slow` - Slow running tests
- `@pytest.mark.integration` - Integration tests
- `@pytest.mark.unit` - Unit tests

## HeadlessWebView Quick Reference

### Create Instance

```python
from auroraview.testing import HeadlessWebView

# Auto-detect best backend
with HeadlessWebView.auto() as webview:
    pass

# Playwright backend
with HeadlessWebView.playwright() as webview:
    pass

# Xvfb backend (Linux)
with HeadlessWebView.virtual_display() as webview:
    pass
```

### Common Methods

| Method | Description |
|--------|-------------|
| `goto(url)` | Navigate to URL |
| `load_html(html)` | Load HTML content |
| `click(selector)` | Click element |
| `fill(selector, text)` | Fill input |
| `text(selector)` | Get element text |
| `screenshot(path)` | Take screenshot |
| `wait_for_selector(sel)` | Wait for element |
| `is_visible(selector)` | Check visibility |
| `evaluate(js)` | Execute JavaScript |

### Pytest Fixtures

| Fixture | Description |
|---------|-------------|
| `headless_webview` | Auto-detected headless WebView |
| `playwright_webview` | Playwright-based WebView |
| `xvfb_webview` | Xvfb-based WebView (Linux) |

## CI/CD Quick Setup

### GitHub Actions

```yaml
- name: Install Playwright
  run: |
    pip install playwright
    playwright install chromium

- name: Run tests
  run: pytest tests/ -v
```

### Linux with Xvfb

```yaml
- name: Run tests
  run: |
    sudo apt-get install -y xvfb
    xvfb-run pytest tests/ -v
```
