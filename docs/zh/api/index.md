# API 参考

本节提供 AuroraView 的详细 API 文档。

## 核心类

### WebView

创建 Web UI 的基础 WebView 类。

```python
from auroraview import WebView

webview = WebView.create(
    title="我的应用",
    url="http://localhost:3000",
    width=1024,
    height=768
)
```

[完整 WebView API →](/api/webview)

### QtWebView

用于 DCC 集成的 Qt 组件包装器。

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=parent_widget,
    url="http://localhost:3000"
)
```

[完整 QtWebView API →](/api/qt-webview)

### AuroraView

带有 HWND 访问的高级包装器。

```python
from auroraview import AuroraView

webview = AuroraView(
    url="http://localhost:3000",
    api=MyAPI()
)
```

[完整 AuroraView API →](/api/auroraview)

## 便捷函数

### run_desktop

启动独立桌面应用程序：

```python
from auroraview import run_desktop

run_desktop(
    title="我的应用",
    url="http://localhost:3000",
    width=1024,
    height=768
)
```

### run_standalone

`run_desktop` 的别名：

```python
from auroraview import run_standalone

run_standalone(
    title="我的应用",
    html="<h1>Hello</h1>"
)
```

## 子窗口支持

### ChildContext

用于创建子窗口感知 WebView 的上下文管理器：

```python
from auroraview import ChildContext

with ChildContext() as ctx:
    webview = ctx.create_webview(
        title="我的示例",
        html="<h1>Hello</h1>"
    )
    
    if ctx.is_child:
        ctx.emit_to_parent("ready", {"status": "ok"})
    
    webview.show()
```

### 模式检测

```python
from auroraview import is_child_mode, get_parent_id, get_child_id

if is_child_mode():
    print(f"父窗口: {get_parent_id()}")
    print(f"子窗口 ID: {get_child_id()}")
```

[完整子窗口指南 →](/zh/guide/child-windows)

## 工具函数

### path_to_file_url

将本地路径转换为 file:// URL：

```python
from auroraview import path_to_file_url

url = path_to_file_url("C:/path/to/file.html")
# 返回: file:///C:/path/to/file.html
```

## 类型定义

### WindowEventData

```python
from auroraview.core.events import WindowEventData

@webview.on_resized
def on_resized(data: WindowEventData):
    print(f"尺寸: {data.width}x{data.height}")
    print(f"位置: ({data.x}, {data.y})")
```

### Signal

类 Qt 信号系统：

```python
from auroraview import Signal

class MyTool(WebView):
    selection_changed = Signal(list)
    progress_updated = Signal(int, str)
```

## JavaScript API

### auroraview 对象

在浏览器上下文中可用：

```javascript
// 调用 Python 方法
const result = await auroraview.call('api.method', { param: 'value' });

// 向 Python 发送事件
auroraview.send_event('event_name', { data: 'value' });

// 监听 Python 事件
auroraview.on('event_name', (data) => {
    console.log(data);
});

// 访问共享状态
auroraview.state.key = 'value';
```
