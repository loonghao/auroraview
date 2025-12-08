# Headless WebView Testing Guide

AuroraView provides multiple strategies for running WebView tests in headless/CI environments.

## Overview

| Mode | Platform | Use Case | Pros | Cons |
|------|----------|----------|------|------|
| **Playwright** | All | UI testing, CI | Fast, reliable, cross-platform | Not real WebView |
| **Xvfb** | Linux | Integration testing | Real WebView | Linux only |
| **WebView2 CDP** | Windows | WebView2 testing | Real WebView2 | Windows only |

## Quick Start

### Playwright Mode (Recommended)

```python
from auroraview.testing.headless_webview import HeadlessWebView

with HeadlessWebView.playwright() as webview:
    webview.goto("https://example.com")
    webview.click("#button")
    assert webview.text("#result") == "Success"
```

### Auto-Detection Mode

```python
from auroraview.testing.headless_webview import HeadlessWebView

# Automatically selects the best mode for current environment
with HeadlessWebView.auto() as webview:
    webview.goto("https://example.com")
```

## Installation

### Playwright (All Platforms)

```bash
pip install playwright
playwright install chromium
```

### Xvfb (Linux)

```bash
# Debian/Ubuntu
sudo apt-get install xvfb

# Fedora/CentOS
sudo dnf install xorg-x11-server-Xvfb

# Arch Linux
sudo pacman -S xorg-server-xvfb
```

## Pytest Integration

### Using Fixtures

```python
# conftest.py
pytest_plugins = ["auroraview.testing.pytest_plugin"]

# test_example.py
import pytest

@pytest.mark.webview
def test_button_click(headless_webview):
    headless_webview.load_html("<button id='btn'>Click</button>")
    headless_webview.click("#btn")

@pytest.mark.playwright
def test_with_playwright(playwright_webview):
    playwright_webview.goto("https://example.com")
    assert playwright_webview.text("h1") == "Example Domain"
```

### Available Fixtures

| Fixture | Description |
|---------|-------------|
| `headless_webview` | Auto-detected mode |
| `playwright_webview` | Playwright mode |
| `xvfb_webview` | Xvfb mode (Linux) |
| `webview2_cdp_webview` | WebView2 CDP mode |
| `playwright_browser` | Session-scoped browser |
| `playwright_page` | Function-scoped page |

## CI/CD Configuration

### GitHub Actions

```yaml
name: WebView Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      # Linux: Install Xvfb and WebKitGTK
      - name: Install Linux dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            xvfb \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev

      # Install Playwright
      - name: Install Playwright
        run: |
          pip install playwright
          playwright install chromium

      # Install package
      - name: Install package
        run: pip install -e ".[test]"

      # Run tests with Playwright (all platforms)
      - name: Run Playwright tests
        run: pytest tests/ -m playwright -v

      # Run tests with Xvfb (Linux only)
      - name: Run Xvfb tests (Linux)
        if: runner.os == 'Linux'
        run: xvfb-run pytest tests/ -m xvfb -v
```

### Using xvfb-run

On Linux CI, use `xvfb-run` to run tests in a virtual display:

```bash
# Run all tests
xvfb-run pytest tests/

# Run specific test file
xvfb-run pytest tests/test_webview.py

# With custom display settings
xvfb-run --server-args="-screen 0 1920x1080x24" pytest tests/
```

## WebView2 CDP Testing

### Starting WebView2 with CDP

Set the environment variable before starting your WebView2 application:

```powershell
# PowerShell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
```

```bash
# Bash
export WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
```

### Connecting to WebView2

```python
from auroraview.testing.headless_webview import HeadlessWebView

# Connect to running WebView2 instance
with HeadlessWebView.webview2_cdp("http://localhost:9222") as webview:
    webview.goto("https://example.com")
```

### Environment Variable

Set `WEBVIEW2_CDP_URL` to auto-connect:

```bash
export WEBVIEW2_CDP_URL="http://localhost:9222"
```

```python
# Auto-detection will use WebView2 CDP
with HeadlessWebView.auto() as webview:
    webview.goto("https://example.com")
```

## Advanced Usage

### Custom Options

```python
from auroraview.testing.headless_webview import HeadlessWebView, HeadlessOptions

options = HeadlessOptions(
    timeout=60.0,
    width=1920,
    height=1080,
    inject_bridge=True,
    screenshot_on_failure=True,
    screenshot_dir="test-artifacts",
    slow_mo=100,  # Slow down for debugging
)

webview = HeadlessWebView.playwright(
    timeout=options.timeout,
    width=options.width,
    height=options.height,
)
```

### Direct Playwright Access

```python
from auroraview.testing.headless_webview import HeadlessWebView

with HeadlessWebView.playwright() as webview:
    # Access underlying Playwright page
    page = webview.page

    # Use full Playwright API
    page.locator("#button").click()
    page.wait_for_load_state("networkidle")
    page.screenshot(path="screenshot.png")
```

### Testing AuroraView Bridge

```python
with HeadlessWebView.playwright(inject_bridge=True) as webview:
    webview.load_html("<div id='app'></div>")

    # Test bridge is injected
    result = webview.evaluate("window.auroraview._testMode")
    assert result == True

    # Test bridge API
    result = webview.evaluate("typeof window.auroraview.call")
    assert result == "function"
```

## Troubleshooting

### Playwright Issues

```bash
# Reinstall browsers
playwright install --force chromium

# Check installation
playwright --version
```

### Xvfb Issues

```bash
# Check if Xvfb is running
ps aux | grep Xvfb

# Start Xvfb manually
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99

# Run tests
pytest tests/
```

### WebView2 CDP Issues

```powershell
# Check if CDP port is open
Test-NetConnection -ComputerName localhost -Port 9222

# List available targets
curl http://localhost:9222/json
```

## References

- [Tauri WebDriver CI Guide](https://tauri.app/develop/tests/webdriver/ci/)
- [Playwright Documentation](https://playwright.dev/python/)
- [Xvfb Manual](https://www.x.org/releases/X11R7.6/doc/man/man1/Xvfb.1.xhtml)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
