# Headless WebView 测试

AuroraView 提供多种策略在无头/CI 环境中运行 WebView 测试。

## 概述

| 模式 | 平台 | 用例 | 优点 | 缺点 |
|------|------|------|------|------|
| **Playwright** | 全平台 | UI 测试, CI | 快速、可靠、跨平台 | 非真实 WebView |
| **Xvfb** | Linux | 集成测试 | 真实 WebView | 仅 Linux |
| **WebView2 CDP** | Windows | WebView2 测试 | 真实 WebView2 | 仅 Windows |

## 快速开始

### Playwright 模式（推荐）

```python
from auroraview.testing.headless_webview import HeadlessWebView

with HeadlessWebView.playwright() as webview:
    webview.goto("https://example.com")
    webview.click("#button")
    assert webview.text("#result") == "Success"
```

### 自动检测模式

```python
from auroraview.testing.headless_webview import HeadlessWebView

# 自动为当前环境选择最佳模式
with HeadlessWebView.auto() as webview:
    webview.goto("https://example.com")
```

## 安装

### Playwright（全平台）

```bash
pip install playwright
playwright install chromium
```

### Xvfb（Linux）

```bash
# Debian/Ubuntu
sudo apt-get install xvfb

# Fedora/CentOS
sudo dnf install xorg-x11-server-Xvfb

# Arch Linux
sudo pacman -S xorg-server-xvfb
```

## Pytest 集成

### 使用 Fixtures

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

### 可用 Fixtures

| Fixture | 描述 |
|---------|------|
| `headless_webview` | 自动检测模式 |
| `playwright_webview` | Playwright 模式 |
| `xvfb_webview` | Xvfb 模式 (Linux) |
| `webview2_cdp_webview` | WebView2 CDP 模式 |
| `playwright_browser` | 会话级浏览器 |
| `playwright_page` | 函数级页面 |

## CI/CD 配置

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

      # Linux: 安装 Xvfb 和 WebKitGTK
      - name: Install Linux dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            xvfb \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev

      # 安装 Playwright
      - name: Install Playwright
        run: |
          pip install playwright
          playwright install chromium

      # 安装包
      - name: Install package
        run: pip install -e ".[test]"

      # 使用 Playwright 运行测试（全平台）
      - name: Run Playwright tests
        run: pytest tests/ -m playwright -v

      # 使用 Xvfb 运行测试（仅 Linux）
      - name: Run Xvfb tests (Linux)
        if: runner.os == 'Linux'
        run: xvfb-run pytest tests/ -m xvfb -v
```

### 使用 xvfb-run

在 Linux CI 上，使用 `xvfb-run` 在虚拟显示器中运行测试：

```bash
# 运行所有测试
xvfb-run pytest tests/

# 运行特定测试文件
xvfb-run pytest tests/test_webview.py

# 使用自定义显示设置
xvfb-run --server-args="-screen 0 1920x1080x24" pytest tests/
```

## WebView2 CDP 测试

### 启动带 CDP 的 WebView2

在启动 WebView2 应用程序之前设置环境变量：

```powershell
# PowerShell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
```

```bash
# Bash
export WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
```

### 连接到 WebView2

```python
from auroraview.testing.headless_webview import HeadlessWebView

# 连接到运行中的 WebView2 实例
with HeadlessWebView.webview2_cdp("http://localhost:9222") as webview:
    webview.goto("https://example.com")
```

### 环境变量

设置 `WEBVIEW2_CDP_URL` 以自动连接：

```bash
export WEBVIEW2_CDP_URL="http://localhost:9222"
```

```python
# 自动检测将使用 WebView2 CDP
with HeadlessWebView.auto() as webview:
    webview.goto("https://example.com")
```

## 高级用法

### 自定义选项

```python
from auroraview.testing.headless_webview import HeadlessWebView, HeadlessOptions

options = HeadlessOptions(
    timeout=60.0,
    width=1920,
    height=1080,
    inject_bridge=True,
    screenshot_on_failure=True,
    screenshot_dir="test-artifacts",
    slow_mo=100,  # 减慢速度以便调试
)

webview = HeadlessWebView.playwright(
    timeout=options.timeout,
    width=options.width,
    height=options.height,
)
```

### 直接访问 Playwright

```python
from auroraview.testing.headless_webview import HeadlessWebView

with HeadlessWebView.playwright() as webview:
    # 访问底层 Playwright 页面
    page = webview.page

    # 使用完整的 Playwright API
    page.locator("#button").click()
    page.wait_for_load_state("networkidle")
    page.screenshot(path="screenshot.png")
```

### 测试 AuroraView Bridge

```python
with HeadlessWebView.playwright(inject_bridge=True) as webview:
    webview.load_html("<div id='app'></div>")

    # 测试 bridge 是否注入
    result = webview.evaluate("window.auroraview._testMode")
    assert result == True

    # 测试 bridge API
    result = webview.evaluate("typeof window.auroraview.call")
    assert result == "function"
```

## 故障排除

### Playwright 问题

```bash
# 重新安装浏览器
playwright install --force chromium

# 检查安装
playwright --version
```

### Xvfb 问题

```bash
# 检查 Xvfb 是否运行
ps aux | grep Xvfb

# 手动启动 Xvfb
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99

# 运行测试
pytest tests/
```

### WebView2 CDP 问题

```powershell
# 检查 CDP 端口是否打开
Test-NetConnection -ComputerName localhost -Port 9222

# 列出可用目标
curl http://localhost:9222/json
```

## 参考资料

- [Tauri WebDriver CI 指南](https://tauri.app/develop/tests/webdriver/ci/)
- [Playwright 文档](https://playwright.dev/python/)
- [Xvfb 手册](https://www.x.org/releases/X11R7.6/doc/man/man1/Xvfb.1.xhtml)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
