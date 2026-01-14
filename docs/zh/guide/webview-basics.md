# WebView 基础

本指南介绍使用 AuroraView WebView 的基本概念。

## 创建 WebView

### 基本创建

```python
from auroraview import WebView

webview = WebView.create(
    title="我的应用",
    url="http://localhost:3000",
    width=1024,
    height=768
)
webview.show()
```

### 使用 HTML 内容

```python
html = """
<!DOCTYPE html>
<html>
<body>
    <h1>Hello World!</h1>
</body>
</html>
"""

webview = WebView.create("我的应用", html=html)
webview.show()
```

## 窗口配置

### 大小和位置

```python
webview = WebView.create(
    title="我的应用",
    width=1024,
    height=768,
    resizable=True,
)
webview.show()

# 运行时控制
webview.resize(1280, 720)
webview.move(100, 100)
```

### 窗口样式

```python
webview = WebView.create(
    title="我的应用",
    frame=True,          # 显示窗口边框
    resizable=True,      # 允许调整大小
    always_on_top=False, # 正常 z 顺序
    transparent=False,   # 不透明背景
)
```

### 无边框窗口

```python
webview = WebView.create(
    title="浮动面板",
    frame=False,         # 无标题栏
    transparent=True,    # 透明背景
    always_on_top=True,  # 保持置顶
)
```

## 窗口事件

### 事件处理

```python
from auroraview.core.events import WindowEventData

@webview.on_shown
def on_shown(data: WindowEventData):
    print("窗口已显示")

@webview.on_focused
def on_focused(data: WindowEventData):
    print("窗口获得焦点")

@webview.on_resized
def on_resized(data: WindowEventData):
    print(f"窗口大小调整为 {data.width}x{data.height}")

@webview.on_closing
def on_closing(data: WindowEventData):
    # 返回 True 允许关闭，False 取消关闭
    return True
```

### 窗口控制

```python
webview.minimize()
webview.maximize()
webview.restore()
webview.toggle_fullscreen()
webview.focus()
webview.hide()
```

## 导航

### 加载内容

```python
# 加载 URL
webview.load_url("https://example.com")

# 加载 HTML
webview.load_html("<h1>你好</h1>")
```

### 导航控制

```python
webview.go_back()
webview.go_forward()
webview.reload()
webview.stop()

# 检查导航状态
if webview.can_go_back():
    webview.go_back()
```

## JavaScript 执行

### 执行 JavaScript

```python
# 即发即忘
webview.eval_js("console.log('来自 Python 的问候')")

# 带返回值
result = webview.eval_js("document.title")
```

### 注入脚本

```python
webview = WebView.create(
    title="我的应用",
    url="https://example.com",
)

# 页面加载后注入
webview.eval_js("""
    const btn = document.createElement('button');
    btn.textContent = '自定义按钮';
    document.body.appendChild(btn);
""")
```

## 自定义图标

```python
webview = WebView.create(
    title="我的应用",
    icon="path/to/icon.png"  # PNG, ICO, JPEG, BMP, GIF
)
```

**图标要求：**

| 属性 | 建议 |
|------|------|
| 格式 | PNG（推荐）、ICO、JPEG、BMP、GIF |
| 大小 | 32×32（任务栏）、64×64（Alt-Tab）、256×256（高 DPI）|
| 色深 | 32 位 RGBA 支持透明 |

## 调试模式

```python
webview = WebView.create(
    title="我的应用",
    debug=True,  # 启用 DevTools
)

# 以编程方式打开 DevTools
webview.open_devtools()
```

## 右键菜单

```python
# 禁用原生右键菜单（用于自定义菜单）
webview = WebView.create(
    title="我的应用",
    context_menu=False,
)
```
