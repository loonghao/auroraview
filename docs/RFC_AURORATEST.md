# RFC: AuroraTest - Playwright-like Testing Framework for AuroraView

## 概述

为AuroraView添加类似Playwright的自动化测试能力，支持CI/CD环境下的无头测试。

**包名**: `auroratest` (内部模块: `auroraview.testing.auroratest`)
**未来独立PyPI包**: `auroraview-test`

## 背景

当前的测试框架（`auroraview.testing`）提供了基础的DOM操作和断言，但存在以下限制：

1. **伪Headless模式** - 当前使用`decorations=False`隐藏窗口装饰，但仍需要显示环境
2. **无截图能力** - 无法进行视觉回归测试
3. **无网络拦截** - 无法模拟API响应
4. **同步等待** - 使用`time.sleep()`而非智能等待

## 目标

### P0 (必须实现)
- [ ] 真正的Headless模式（无需显示环境）
- [ ] 截图功能（全页面/元素）
- [ ] 智能等待机制
- [ ] CI/CD集成支持

### P1 (应该实现)
- [ ] 网络请求拦截/模拟
- [ ] 多页面/多窗口支持
- [ ] Trace录制/回放
- [ ] 测试报告生成

### P2 (可以实现)
- [ ] 视频录制
- [ ] 移动端模拟
- [ ] 性能指标收集

## 技术方案

### 方案A: WebView2 Headless模式 (推荐)

WebView2 从 v1.0.1518.46 开始支持真正的Headless模式。

```rust
// Rust实现
use webview2_com::Microsoft::Web::WebView2::Win32::*;

pub struct HeadlessWebView {
    controller: ICoreWebView2Controller,
    webview: ICoreWebView2,
}

impl HeadlessWebView {
    pub fn new() -> Result<Self> {
        // 创建无窗口的WebView2
        let options = CoreWebView2EnvironmentOptions::default();
        // 设置headless模式
        options.set_additional_browser_arguments("--headless")?;
        
        // 创建环境
        let env = CreateCoreWebView2EnvironmentWithOptions(
            None, None, Some(&options), ...
        )?;
        
        // 创建无窗口控制器
        let controller = env.CreateCoreWebView2ControllerWithOptions(
            HWND(0), // 无父窗口
            ...
        )?;
        
        Ok(Self { controller, webview })
    }
    
    pub fn capture_screenshot(&self) -> Result<Vec<u8>> {
        // 使用CapturePreview API
        self.webview.CapturePreview(
            COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_PNG,
            stream,
            callback
        )
    }
}
```

### 方案B: CDP (Chrome DevTools Protocol)

WebView2支持CDP，可以用于高级自动化：

```python
# Python实现
class CDPSession:
    """Chrome DevTools Protocol session."""
    
    async def send(self, method: str, params: dict = None) -> dict:
        """Send CDP command."""
        pass
    
    async def screenshot(self, options: dict = None) -> bytes:
        """Capture screenshot via CDP."""
        return await self.send("Page.captureScreenshot", {
            "format": "png",
            "quality": 100,
            "fromSurface": True
        })
    
    async def set_viewport(self, width: int, height: int):
        """Set viewport size."""
        await self.send("Emulation.setDeviceMetricsOverride", {
            "width": width,
            "height": height,
            "deviceScaleFactor": 1,
            "mobile": False
        })
```

### 方案C: 混合方案 (推荐)

结合WebView2原生能力和CDP：

1. **截图/视口** - 使用WebView2原生API（更高效）
2. **网络拦截** - 使用CDP（更灵活）
3. **DOM操作** - 使用现有的JavaScript注入

## API设计

### 核心类

```python
from auroraview.testing.auroratest import Browser, Page, Locator, expect

class Browser:
    """浏览器实例，管理多个Page。"""
    
    @classmethod
    def launch(cls, headless: bool = True, **kwargs) -> "Browser":
        """启动浏览器实例。"""
        pass
    
    def new_page(self) -> "Page":
        """创建新页面。"""
        pass
    
    def close(self):
        """关闭浏览器。"""
        pass


class Page:
    """页面实例，提供Playwright风格的API。"""
    
    # === 导航 ===
    async def goto(self, url: str, wait_until: str = "load") -> Response:
        """导航到URL。"""
        pass
    
    async def reload(self, wait_until: str = "load") -> Response:
        """重新加载页面。"""
        pass
    
    # === 选择器 ===
    def locator(self, selector: str) -> "Locator":
        """创建定位器。"""
        pass
    
    def get_by_role(self, role: str, **kwargs) -> "Locator":
        """按角色查找元素。"""
        pass
    
    def get_by_text(self, text: str, exact: bool = False) -> "Locator":
        """按文本查找元素。"""
        pass
    
    def get_by_test_id(self, test_id: str) -> "Locator":
        """按data-testid查找元素。"""
        pass
    
    # === 截图 ===
    async def screenshot(
        self,
        path: str = None,
        full_page: bool = False,
        clip: dict = None
    ) -> bytes:
        """截取页面截图。"""
        pass
    
    # === 等待 ===
    async def wait_for_selector(
        self,
        selector: str,
        state: str = "visible",
        timeout: float = 30000
    ) -> "Locator":
        """等待选择器。"""
        pass
    
    async def wait_for_load_state(self, state: str = "load"):
        """等待加载状态。"""
        pass
    
    async def wait_for_url(self, url: str | re.Pattern):
        """等待URL变化。"""
        pass
    
    # === 网络 ===
    async def route(self, url: str | re.Pattern, handler: Callable):
        """拦截网络请求。"""
        pass
    
    async def unroute(self, url: str | re.Pattern):
        """取消拦截。"""
        pass


class Locator:
    """元素定位器，支持链式调用。"""
    
    # === 操作 ===
    async def click(self, **kwargs):
        """点击元素。"""
        pass
    
    async def fill(self, value: str):
        """填充输入框。"""
        pass
    
    async def type(self, text: str, delay: float = 0):
        """逐字输入。"""
        pass
    
    async def press(self, key: str):
        """按键。"""
        pass
    
    async def check(self):
        """勾选复选框。"""
        pass
    
    async def uncheck(self):
        """取消勾选。"""
        pass
    
    async def select_option(self, value: str | list):
        """选择下拉选项。"""
        pass
    
    async def hover(self):
        """悬停。"""
        pass
    
    async def focus(self):
        """聚焦。"""
        pass
    
    async def screenshot(self, path: str = None) -> bytes:
        """截取元素截图。"""
        pass
    
    # === 查询 ===
    async def text_content(self) -> str:
        """获取文本内容。"""
        pass
    
    async def inner_text(self) -> str:
        """获取内部文本。"""
        pass
    
    async def inner_html(self) -> str:
        """获取内部HTML。"""
        pass
    
    async def get_attribute(self, name: str) -> str:
        """获取属性值。"""
        pass
    
    async def input_value(self) -> str:
        """获取输入值。"""
        pass
    
    async def is_visible(self) -> bool:
        """是否可见。"""
        pass
    
    async def is_enabled(self) -> bool:
        """是否启用。"""
        pass
    
    async def is_checked(self) -> bool:
        """是否勾选。"""
        pass
    
    # === 链式定位 ===
    def first(self) -> "Locator":
        """第一个匹配。"""
        pass
    
    def last(self) -> "Locator":
        """最后一个匹配。"""
        pass
    
    def nth(self, index: int) -> "Locator":
        """第N个匹配。"""
        pass
    
    def filter(self, **kwargs) -> "Locator":
        """过滤。"""
        pass


# === 断言 ===
def expect(target: Locator | Page) -> LocatorAssertions | PageAssertions:
    """Playwright风格的断言。"""
    pass

class LocatorAssertions:
    async def to_be_visible(self, timeout: float = 5000):
        """断言可见。"""
        pass
    
    async def to_be_hidden(self, timeout: float = 5000):
        """断言隐藏。"""
        pass
    
    async def to_have_text(self, text: str | re.Pattern, timeout: float = 5000):
        """断言文本。"""
        pass
    
    async def to_have_value(self, value: str, timeout: float = 5000):
        """断言值。"""
        pass
    
    async def to_have_attribute(self, name: str, value: str, timeout: float = 5000):
        """断言属性。"""
        pass
    
    async def to_be_checked(self, timeout: float = 5000):
        """断言勾选。"""
        pass
    
    async def to_be_enabled(self, timeout: float = 5000):
        """断言启用。"""
        pass
    
    async def to_have_count(self, count: int, timeout: float = 5000):
        """断言数量。"""
        pass
```

### 使用示例

```python
import pytest
from auroraview.testing.auroratest import Browser, expect

@pytest.fixture
async def page():
    """创建测试页面。"""
    browser = Browser.launch(headless=True)
    page = browser.new_page()
    yield page
    browser.close()

async def test_login_form(page):
    """测试登录表单。"""
    # 导航
    await page.goto("https://auroraview.localhost/login.html")
    
    # 填写表单
    await page.locator("#email").fill("test@example.com")
    await page.locator("#password").fill("secret123")
    
    # 点击登录
    await page.get_by_role("button", name="Login").click()
    
    # 断言
    await expect(page.locator(".welcome-message")).to_have_text("Welcome, test@example.com")
    
    # 截图
    await page.screenshot(path="screenshots/login_success.png")

async def test_api_mocking(page):
    """测试API模拟。"""
    # 拦截API请求
    await page.route("**/api/users", lambda route: route.fulfill(
        status=200,
        content_type="application/json",
        body='[{"id": 1, "name": "Mock User"}]'
    ))
    
    await page.goto("https://auroraview.localhost/users.html")
    
    # 验证模拟数据显示
    await expect(page.locator(".user-name")).to_have_text("Mock User")

async def test_visual_regression(page):
    """视觉回归测试。"""
    await page.goto("https://auroraview.localhost/dashboard.html")
    
    # 全页面截图对比
    screenshot = await page.screenshot(full_page=True)
    
    # 使用pixelmatch或类似库进行对比
    assert_screenshots_match(screenshot, "baseline/dashboard.png", threshold=0.1)
```

### Pytest集成

```python
# conftest.py
import pytest
from auroraview.testing.auroratest import Browser

@pytest.fixture(scope="session")
def browser():
    """Session级别的浏览器实例。"""
    browser = Browser.launch(
        headless=True,
        # CI环境特定配置
        args=["--no-sandbox", "--disable-gpu"] if os.environ.get("CI") else []
    )
    yield browser
    browser.close()

@pytest.fixture
async def page(browser):
    """每个测试的新页面。"""
    page = browser.new_page()
    yield page
    await page.close()

# pytest.ini
[pytest]
markers =
    visual: Visual regression tests
    slow: Slow tests
    
asyncio_mode = auto
```

## 实现计划

### Phase 1: 核心功能 (2周)

1. **Rust: Headless WebView2**
   - 实现无窗口WebView2创建
   - 添加CapturePreview绑定
   - 支持设置viewport大小

2. **Python: 基础Page API**
   - `goto()`, `reload()`
   - `screenshot()`
   - `wait_for_selector()`

### Phase 2: 定位器系统 (1周)

1. **Locator类实现**
   - 基础选择器
   - 链式定位
   - 操作方法

2. **expect断言**
   - 基础断言
   - 自动重试机制

### Phase 3: 高级功能 (2周)

1. **网络拦截**
   - CDP集成
   - route/unroute API

2. **多页面支持**
   - Browser管理
   - 页面间通信

### Phase 4: CI/CD集成 (1周)

1. **GitHub Actions示例**
2. **测试报告生成**
3. **截图对比工具**

## 文件结构

```
python/auroraview/testing/
├── __init__.py              # 公开API
├── auroratest/              # Playwright-like testing framework
│   ├── __init__.py          # 公开API
│   ├── browser.py           # Browser类
│   ├── page.py              # Page类
│   ├── locator.py           # Locator类
│   ├── expect.py            # 断言
│   ├── network.py           # 网络拦截
│   └── fixtures.py          # Pytest fixtures
├── assertions.py            # Legacy assertions
├── dom_assertions.py        # DOM assertions
├── fixtures.py              # Legacy fixtures
├── headless.py              # HeadlessTestRunner
└── webview_bot.py           # WebViewBot

crates/auroraview-testing/   # Future Rust crate for native support
├── src/
│   ├── lib.rs
│   ├── headless.rs          # Headless WebView2
│   ├── screenshot.rs        # 截图实现
│   └── cdp.rs               # CDP支持
└── Cargo.toml
```

## 兼容性

- **Windows**: WebView2 (完全支持)
- **macOS**: WKWebView (部分支持，无真正headless)
- **Linux**: WebKitGTK (部分支持)

## 参考

- [Playwright Python API](https://playwright.dev/python/docs/api/class-playwright)
- [WebView2 CapturePreview](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2?view=webview2-1.0.2210.55#capturepreview)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
