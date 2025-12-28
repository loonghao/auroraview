# 测试框架

AuroraView 提供了一个全面的测试框架，包含多种测试 WebView 应用的方法。

## 概述

测试框架包括：

| 组件 | 用途 | 使用场景 |
|------|------|----------|
| **装饰器** | 条件测试执行 | 根据环境跳过测试 |
| **生成器** | 随机测试数据 | 模糊测试、边界情况 |
| **属性测试** | Hypothesis 策略 | 基于属性的测试 |
| **快照测试** | 回归检测 | UI/输出稳定性 |
| **Headless WebView** | 浏览器自动化 | 端到端测试 |
| **Midscene.js** | AI 驱动测试 | 自然语言 UI 自动化 |

## 快速开始

### 测试装饰器

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
    """仅在 Qt 可用时运行。"""
    from auroraview.integration.qt import QtWebView
    # 测试 Qt 功能

@requires_windows
@slow_test
def test_webview2_performance():
    """仅在 Windows 上运行，标记为慢速测试。"""
    # 性能测试

@integration_test
@with_timeout(60)
def test_full_workflow():
    """集成测试，60秒超时。"""
    # 端到端测试
```

### 测试数据生成器

```python
from auroraview.testing import (
    random_html,
    random_html_page,
    random_js_value,
    random_event_name,
    random_selector,
    generate_test_dataset,
)

# 生成随机 HTML
html = random_html("div", content="Hello", attrs={"class": "test"})
# <div class="test">Hello</div>

# 生成完整 HTML 页面
page = random_html_page(title="Test", body_content="<h1>Hello</h1>")

# 生成随机 JSON 可序列化值
value = random_js_value()  # string, number, bool, array, object 或 null

# 生成随机事件名称
event = random_event_name(prefix="user", namespace="api")
# "api:user_abc123"

# 生成测试数据集
dataset = generate_test_dataset(count=10, data_type="events")
```

### 快照测试

```python
from auroraview.testing import SnapshotTest

def test_component_output(tmp_path):
    snapshot = SnapshotTest(tmp_path / "snapshots")

    # 测试 HTML 输出
    html = render_component()
    snapshot.assert_match_html(html, "component.html")

    # 测试 JSON 输出
    data = get_api_response()
    snapshot.assert_match_json(data, "response.json")
```

### 基于属性的测试

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
    """测试值能否经过 JSON 序列化。"""
    import json
    assert json.loads(json.dumps(value)) == value

@given(html=html_elements(max_depth=2))
def test_html_parsing(html):
    """使用随机输入测试 HTML 解析。"""
    assert "<" in html and ">" in html
```

## 装饰器参考

### 环境检查

| 装饰器 | 描述 |
|--------|------|
| `@requires_qt` | 如果 Qt (PySide6/PySide2) 不可用则跳过 |
| `@requires_cdp(url)` | 如果 CDP 端点不可用则跳过 |
| `@requires_gallery` | 如果打包的 gallery 不可用则跳过 |
| `@requires_playwright` | 如果 Playwright 未安装则跳过 |
| `@requires_webview2` | 如果 WebView2 运行时不可用则跳过 |
| `@requires_windows` | 如果不在 Windows 上则跳过 |
| `@requires_linux` | 如果不在 Linux 上则跳过 |
| `@requires_macos` | 如果不在 macOS 上则跳过 |
| `@requires_env(var, value)` | 如果环境变量未设置/不匹配则跳过 |

### 测试类别

| 装饰器 | 描述 |
|--------|------|
| `@slow_test` | 标记为慢速测试（快速运行时可能跳过） |
| `@integration_test` | 标记为集成测试 |
| `@unit_test` | 标记为单元测试 |
| `@smoke_test` | 标记为冒烟测试（快速健全性检查） |
| `@flaky_test(reruns=3)` | 标记为不稳定测试，自动重试 |

### 测试设置

| 装饰器 | 描述 |
|--------|------|
| `@with_timeout(seconds)` | 设置测试超时 |
| `@parametrize_examples(ids)` | 使用示例 ID 参数化 |
| `@serial_test` | 串行运行（不并行） |
| `@skip_if(condition, reason)` | 如果条件为 True 则跳过 |
| `@xfail_if(condition, reason)` | 如果条件为 True 则预期失败 |

## 生成器参考

### HTML 生成器

```python
from auroraview.testing import (
    random_string,
    random_html,
    random_html_page,
    random_form_html,
)

# 随机字符串
s = random_string(length=10, charset="abc123")

# 随机 HTML 元素
html = random_html(
    tag="div",
    content="Hello",
    attrs={"class": "container", "id": "main"},
    children=["<span>Child 1</span>", "<span>Child 2</span>"],
)

# 完整 HTML 页面
page = random_html_page(
    title="Test Page",
    body_content="<h1>Hello</h1>",
    styles="body { margin: 0; }",
    scripts="console.log('loaded');",
)

# HTML 表单
form = random_form_html(
    fields=[
        {"name": "email", "type": "email", "label": "邮箱"},
        {"name": "password", "type": "password", "label": "密码"},
    ],
    action="/login",
    method="post",
)
```

### JavaScript 值生成器

```python
from auroraview.testing import (
    random_js_value,
    random_event_payload,
    random_api_method,
    random_api_params,
)

# 随机 JSON 可序列化值
value = random_js_value(value_type="object", max_depth=3)

# 随机事件负载
payload = random_event_payload(event_type="click")
# {"timestamp": 1234567890, "type": "click", "x": 100, "y": 200, ...}

# 随机 API 方法
method = random_api_method(namespace="api")
# "api.get_user"

# 随机 API 参数
params = random_api_params(param_count=3, as_dict=True)
# {"key1": "value1", "key2": 123, "key3": true}
```

### 选择器生成器

```python
from auroraview.testing import random_selector, random_xpath

# CSS 选择器
id_sel = random_selector("id")      # "#abc123"
class_sel = random_selector("class")  # ".xyz789"
tag_sel = random_selector("tag")    # "div"
attr_sel = random_selector("attr")  # '[data-id="abc"]'

# XPath 表达式
xpath = random_xpath("button")
# "//button[@id='abc123']"
```

### URL 生成器

```python
from auroraview.testing import random_url, random_file_url

# HTTP/HTTPS URLs
url = random_url(scheme="https", domain="example.com")
# "https://example.com/path/to/resource"

# 文件 URLs
file_url = random_file_url(extension="html", directory="/tmp/test")
# "file:///tmp/test/abc123.html"
```

## 快照测试

### 基本用法

```python
from auroraview.testing import SnapshotTest

snapshot = SnapshotTest("tests/snapshots")

# 文本比较
snapshot.assert_match(output, "output.txt")

# JSON 比较（排序键，格式化）
snapshot.assert_match_json(data, "data.json")

# HTML 比较（规范化空白）
snapshot.assert_match_html(html, "page.html")

# 哈希比较（用于大型内容）
snapshot.assert_hash_match(large_content, "large_file")
```

### 更新快照

```bash
# 更新所有快照
UPDATE_SNAPSHOTS=1 pytest tests/

# 或使用 pytest 标志（如果已配置）
pytest tests/ --update-snapshots
```

### Pytest 集成

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

### 截图快照

```python
from auroraview.testing import ScreenshotSnapshot

screenshot = ScreenshotSnapshot("tests/screenshots", threshold=0.01)

# 比较截图（需要 Pillow 进行像素比较）
screenshot.assert_screenshot_match(png_data, "homepage.png")
```

## 基于属性的测试

基于属性的测试生成随机输入以发现边界情况。

### 安装

```bash
pip install hypothesis
```

### 策略

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

# 与 @given 装饰器一起使用
from hypothesis import given

@given(tag=html_tags())
def test_tag_is_valid(tag):
    assert tag in ["div", "span", "p", ...]

@given(value=js_values(max_depth=2))
def test_json_serializable(value):
    import json
    json.dumps(value)  # 不应抛出异常

@given(selector=css_selectors())
def test_selector_format(selector):
    assert selector.startswith(("#", ".", "[")) or selector.isalpha()
```

### 自定义设置

```python
from auroraview.testing.property_testing import property_test

@property_test(max_examples=200, deadline=None)
@given(html=html_elements())
def test_html_parsing(html):
    # 使用更多示例测试，无超时
    parse_html(html)
```

## Midscene.js AI 驱动测试

[Midscene.js](https://midscenejs.com/) 是字节跳动开源的 AI 驱动 UI 自动化 SDK，支持使用自然语言进行测试。

### 安装

```bash
# Midscene 集成需要 Playwright
pip install playwright
playwright install chromium

# 设置 AI 模型 API 密钥
export OPENAI_API_KEY=your-api-key
# 或使用其他模型（通义千问、Gemini 等）
```

### 基本用法

```python
from auroraview.testing import MidsceneAgent, MidsceneConfig
from playwright.async_api import async_playwright

async def test_with_ai():
    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()
        await page.goto("https://example.com")

        # 创建 AI 代理
        async with MidsceneAgent(page) as agent:
            # 自然语言操作
            await agent.ai_act('点击登录按钮')
            await agent.ai_act('在邮箱输入框中输入 "test@example.com"')
            await agent.ai_act('在密码输入框中输入 "password123"')
            await agent.ai_act('点击提交')

            # AI 驱动的断言
            await agent.ai_assert('仪表盘可见')
            await agent.ai_assert('欢迎消息包含 "test@example.com"')

            # 使用 AI 提取数据
            user_info = await agent.ai_query({
                'name': 'string, 用户显示名称',
                'email': 'string, 用户邮箱地址',
                'role': 'string, 用户角色'
            })

        await browser.close()
```

### 核心方法

| 方法 | 描述 | 示例 |
|------|------|------|
| `ai_act(instruction)` | 执行自然语言操作 | `await agent.ai_act('点击蓝色按钮')` |
| `ai_query(demand)` | 提取结构化数据 | `await agent.ai_query('string[], 产品名称')` |
| `ai_assert(condition)` | 自然语言验证 | `await agent.ai_assert('表单已提交')` |
| `ai_wait_for(condition)` | 等待条件 | `await agent.ai_wait_for('加载动画消失')` |
| `ai_locate(description)` | 按描述查找元素 | `await agent.ai_locate('提交按钮')` |

### 配置

```python
from auroraview.testing import MidsceneConfig

config = MidsceneConfig(
    # 模型设置
    model_name="gpt-4o",           # 或 "qwen-vl-plus", "gemini-1.5-flash"
    model_family="openai",          # 从 model_name 自动检测
    api_key="your-api-key",         # 或使用 OPENAI_API_KEY 环境变量
    base_url=None,                  # 自定义 API 端点

    # 行为
    timeout=60000,                  # 60 秒
    cacheable=True,                 # 缓存 AI 响应
    debug=False,                    # 详细日志

    # 上下文选项
    dom_included=False,             # 包含 DOM 信息
)
```

### Pytest 集成

```python
# conftest.py
import pytest
from auroraview.testing import MidscenePlaywrightFixture

@pytest.fixture
async def ai(page):
    """AI 驱动测试 fixture。"""
    fixture = MidscenePlaywrightFixture(page)
    yield fixture
    fixture.close()

# test_example.py
async def test_login_flow(ai):
    await ai.act('点击登录按钮')
    await ai.act('在邮箱输入框输入 "user@example.com"')
    await ai.act('在密码输入框输入 "password"')
    await ai.act('点击提交')
    await ai.assert_('仪表盘可见')
```

### 数据提取示例

```python
# 提取字符串列表
products = await agent.ai_query('string[], 页面上所有产品名称')

# 提取结构化数据
items = await agent.ai_query({
    'title': 'string, 产品标题',
    'price': 'number, 价格（美元）',
    'inStock': 'boolean, 库存状态'
})

# 使用 DOM 上下文提取（用于不可见属性）
links = await agent.ai_query(
    '{text: string, href: string}[], 所有导航链接',
    dom_included=True
)
```

### 支持的 AI 模型

| 提供商 | 模型 | 环境变量 |
|--------|------|----------|
| OpenAI | gpt-4o, gpt-4o-mini | `OPENAI_API_KEY` |
| 通义千问 | qwen-vl-plus, qwen-vl-max | `MIDSCENE_MODEL_API_KEY`, `MIDSCENE_MODEL_BASE_URL` |
| Gemini | gemini-1.5-flash, gemini-1.5-pro | `MIDSCENE_MODEL_API_KEY` |
| Claude | claude-3-5-sonnet | `MIDSCENE_MODEL_API_KEY` |

## 最佳实践

### 1. 使用适当的测试类别

```python
@smoke_test
def test_import():
    """快速健全性检查。"""
    import auroraview
    assert auroraview is not None

@unit_test
def test_function():
    """隔离的单元测试。"""
    result = my_function(1, 2)
    assert result == 3

@integration_test
def test_workflow():
    """完整工作流测试。"""
    # 多个组件协同工作
```

### 2. 处理环境依赖

```python
@requires_qt
@requires_windows
def test_qt_on_windows():
    """仅在 Qt 可用且在 Windows 上时运行。"""
    pass

@requires_env("CI")
def test_ci_only():
    """仅在 CI 环境中运行。"""
    pass
```

### 3. 使用生成器处理边界情况

```python
def test_html_rendering():
    """使用各种 HTML 输入进行测试。"""
    for _ in range(100):
        html = random_html_page()
        result = render(html)
        assert result is not None
```

### 4. 结合快照测试

```python
def test_api_response(snapshot):
    """确保 API 响应格式稳定。"""
    response = api.get_data()
    snapshot.assert_match_json(response, "api_response.json")
```

### 5. 属性测试提高健壮性

```python
@given(value=js_values())
def test_serialization_roundtrip(value):
    """确保序列化是可逆的。"""
    serialized = serialize(value)
    deserialized = deserialize(serialized)
    assert deserialized == value
```

## 浏览器自动化

AuroraView 提供了统一的浏览器自动化抽象层用于测试。

### Automation API

```python
from auroraview import WebView
from auroraview.utils.automation import Automation

# 创建带自动化的 WebView
webview = WebView(title="Test", width=800, height=600)

# 创建自动化实例
auto = Automation(webview)

# 导航和交互
await auto.goto("https://example.com")
await auto.click("#submit-button")
await auto.fill("#email", "test@example.com")

# 提取数据
title = await auto.get_text("h1")
links = await auto.query_selector_all("a")
```

### 本地 WebView 后端

```python
from auroraview.utils.automation import LocalWebViewBackend

# 使用本地 WebView 进行自动化
backend = LocalWebViewBackend(webview)

# 执行 JavaScript
result = await backend.evaluate("document.title")

# 截图
screenshot = await backend.screenshot()
```

## 示例演示

AuroraView 包含多个测试示例演示：

| 示例 | 描述 |
|------|------|
| `automation_demo.py` | 浏览器自动化抽象层 |
| `midscene_demo.py` | 使用 Midscene.js 的 AI 驱动测试 |
| `dom_batch_demo.py` | 批量 DOM 操作 |
| `event_timer_demo.py` | 事件和定时器测试 |
| `channel_streaming_demo.py` | IPC 通道流式传输 |
| `command_registry_demo.py` | 命令注册模式 |

运行示例：

```bash
python examples/automation_demo.py
python examples/midscene_demo.py
```

## 参考资料

- [pytest 文档](https://docs.pytest.org/)
- [Hypothesis 文档](https://hypothesis.readthedocs.io/)
- [快照测试最佳实践](https://jestjs.io/docs/snapshot-testing)
- [基于属性的测试介绍](https://hypothesis.works/articles/what-is-hypothesis/)
- [Midscene.js 文档](https://midscenejs.com/)
