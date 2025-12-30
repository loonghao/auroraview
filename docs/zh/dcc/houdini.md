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

## 线程调度器

```python
from auroraview.utils import ensure_main_thread

@ensure_main_thread
def safe_cook_node(node_path):
    """在主线程上烹饪节点"""
    import hou
    node = hou.node(node_path)
    node.cook(force=True)
```

## 另请参阅

- [Qt 集成指南](../guide/qt-integration.md)
- [线程调度器](../guide/thread-dispatcher.md)
