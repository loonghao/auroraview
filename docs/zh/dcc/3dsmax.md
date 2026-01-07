# 3ds Max 集成

AuroraView 通过 Qt 与 3ds Max 集成。

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| 3ds Max | 2020 | 2024+ |
| Python | 3.7 | 3.10+ |

## 快速开始

```python
from auroraview import QtWebView
from qtpy import QtWidgets
import MaxPlus

def max_main_window():
    return QtWidgets.QWidget.find(MaxPlus.GetQMaxMainWindow())

webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000"
)
webview.show()
```

## API 通信

```python
from auroraview import QtWebView
import MaxPlus

class MaxAPI:
    def get_selection(self):
        """获取选中的对象"""
        return [n.Name for n in MaxPlus.SelectionManager.Nodes]
    
    def create_box(self, name, length=10, width=10, height=10):
        """创建一个盒子"""
        box = MaxPlus.Factory.CreateGeomObject(MaxPlus.ClassIds.Box)
        node = MaxPlus.Factory.CreateNode(box, name)
        return node.Name

webview = QtWebView(
    parent=max_main_window(),
    api=MaxAPI()
)
```

## 线程安全

AuroraView 为 3ds Max 集成提供自动线程安全。3ds Max 要求所有 `pymxs` 操作在主线程运行。

### 使用 `dcc_mode` 自动线程安全

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
