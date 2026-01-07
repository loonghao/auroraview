# 3ds Max 集成

AuroraView 通过 Qt 与 3ds Max 集成。

::: info 3ds Max 2020+
MaxPlus 从 3ds Max 2020 开始已弃用。本指南使用 `pymxs`，这是 3ds Max 2020 及更高版本推荐的 API。
:::

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| 3ds Max | 2020 | 2024+ |
| Python | 3.9 | 3.11+ |
| Qt | PySide2/Qt5 | PySide2/Qt5 |

## 快速开始

```python
from auroraview import QtWebView
from qtpy import QtWidgets
from pymxs import runtime as rt

def max_main_window():
    """获取 3ds Max 主窗口作为 QWidget"""
    hwnd = rt.windows.getMAXHWND()
    return QtWidgets.QWidget.find(hwnd)

# AuroraView 自动检测 3ds Max 并启用线程安全
webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000"
)
webview.show()
```

::: tip 自动线程安全
在 3ds Max 中运行时，AuroraView 自动启用线程安全（`dcc_mode="auto"`）。无需额外配置！
:::

## API 通信

```python
from auroraview import QtWebView
from pymxs import runtime as rt

class MaxAPI:
    def get_selection(self):
        """获取选中的对象"""
        sel = list(rt.selection)
        return [str(obj.name) for obj in sel]

    def create_box(self, name="Box001", size=10.0):
        """创建一个盒子"""
        box = rt.Box(name=name, length=size, width=size, height=size)
        return str(box.name)

webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000"
)
webview.bind_api(MaxAPI())
```

## 线程安全

AuroraView 为 3ds Max 集成提供**自动**线程安全。由于 3ds Max 内部使用 Qt，AuroraView 利用 Qt 的事件循环（`QTimer.singleShot`）在主线程调度回调。

::: tip 零配置
由于 `dcc_mode="auto"` 是默认值，AuroraView 会自动检测 3ds Max 并启用线程安全。无需任何配置！
:::

### 自动线程安全（默认）

```python
from auroraview import QtWebView
from pymxs import runtime as rt
from qtpy import QtWidgets

def max_main_window():
    hwnd = rt.windows.getMAXHWND()
    return QtWidgets.QWidget.find(hwnd)

# 所有回调自动在 3ds Max 主线程运行
webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000",
    dcc_mode=True,  # 启用自动线程安全
)

@webview.on("create_box")
def handle_create(data):
    # 自动在 3ds Max 主线程运行！
    name = data.get("name", "Box001")
    size = data.get("size", 10.0)
    box = rt.Box(name=name, length=size, width=size, height=size)
    return {"ok": True, "name": str(box.name)}

@webview.on("get_selection")
def handle_selection(data):
    sel = list(rt.selection)
    return {"selection": [str(obj.name) for obj in sel], "count": len(sel)}

webview.show()
```

### 使用装饰器手动线程安全

```python
from auroraview import QtWebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async
from pymxs import runtime as rt

webview = QtWebView(parent=max_main_window(), url="http://localhost:3000")

@webview.on("render_scene")
@dcc_thread_safe  # 阻塞直到渲染完成
def handle_render(data):
    output_path = data.get("path", "C:/temp/render.png")
    rt.render(outputFile=output_path)
    return {"ok": True, "path": output_path}

@webview.on("refresh_viewport")
@dcc_thread_safe_async  # 即发即忘
def handle_refresh(data):
    rt.redrawViews()

webview.show()
```

### 直接使用 `run_on_main_thread`

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync
from pymxs import runtime as rt

# 即发即忘
def clear_selection():
    rt.clearSelection()

run_on_main_thread(clear_selection)

# 阻塞并返回值
def get_max_file_path():
    return rt.maxFilePath + rt.maxFileName

file_path = run_on_main_thread_sync(get_max_file_path)
print(f"当前文件: {file_path}")
```

## 另请参阅

- [Qt 集成指南](../guide/qt-integration.md)
- [线程调度器](../guide/thread-dispatcher.md)
