# Houdini 集成

AuroraView 通过 PySide2/Qt5 与 Houdini 集成。

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Houdini | 18.5 | 20.0+ |
| Python | 3.7 | 3.10+ |
| Qt | PySide2/Qt5 | PySide2/Qt5 |

## 快速开始

### 基础用法

```python
from auroraview import QtWebView
import hou

def houdini_main_window():
    return hou.qt.mainWindow()

# 创建 WebView
webview = QtWebView(
    parent=houdini_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

### Python Panel

```python
from auroraview import QtWebView
import hou

def onCreateInterface():
    """Houdini Python Panel 入口点"""
    webview = QtWebView()
    webview.load_url("http://localhost:3000")
    return webview
```

## API 通信

```python
from auroraview import QtWebView
import hou

class HoudiniAPI:
    def get_selected_nodes(self):
        """获取选中的节点"""
        return [n.path() for n in hou.selectedNodes()]
    
    def create_node(self, parent_path, node_type, name=None):
        """创建节点"""
        parent = hou.node(parent_path)
        node = parent.createNode(node_type, name)
        return node.path()

webview = QtWebView(
    parent=houdini_main_window(),
    api=HoudiniAPI()
)
```

## 线程安全

AuroraView 为 Houdini 集成提供**自动**线程安全。Houdini 要求所有 `hou` 操作在主线程运行，AuroraView 通过 `hdefereval` 处理。

::: tip 零配置
由于 `dcc_mode="auto"` 是默认值，AuroraView 会自动检测 Houdini 并启用线程安全。无需任何配置！
:::

### 自动线程安全（默认）

正常使用 AuroraView 即可 - 线程安全是自动的：

```python
from auroraview import QtWebView
import hou

# 检测到 Houdini 时自动启用线程安全
webview = QtWebView(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    # dcc_mode="auto" 是默认值 - 无需指定！
)

@webview.on("create_node")
def handle_create(data):
    # 自动在 Houdini 主线程运行！
    parent_path = data.get("parent", "/obj")
    node_type = data.get("type", "geo")

    parent = hou.node(parent_path)
    new_node = parent.createNode(node_type)
    return {"ok": True, "path": new_node.path()}

webview.show()
```

### 使用装饰器手动线程安全

```python
from auroraview import QtWebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async
import hou

webview = QtWebView(parent=hou.qt.mainWindow(), url="http://localhost:3000")

@webview.on("cook_node")
@dcc_thread_safe  # 阻塞直到烹饪完成
def handle_cook(data):
    node_path = data.get("path")
    node = hou.node(node_path)
    if node:
        node.cook(force=True)
        return {"ok": True}
    return {"ok": False, "error": "节点未找到"}

@webview.on("update_display")
@dcc_thread_safe_async  # 即发即忘
def handle_update(data):
    hou.ui.triggerUpdate()

webview.show()
```

### 直接使用 `run_on_main_thread`

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync
import hou

# 即发即忘
def select_node(path):
    node = hou.node(path)
    if node:
        node.setSelected(True, clear_all_selected=True)

run_on_main_thread(select_node, "/obj/geo1")

# 阻塞并返回值
def get_hip_path():
    return hou.hipFile.path()

hip_path = run_on_main_thread_sync(get_hip_path)
print(f"当前 HIP 文件: {hip_path}")
```

## 另请参阅

- [Qt 集成指南](../guide/qt-integration.md)
- [线程调度器](../guide/thread-dispatcher.md)
