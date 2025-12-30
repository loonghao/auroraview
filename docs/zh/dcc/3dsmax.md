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

## 线程调度器

```python
from auroraview.utils import ensure_main_thread

@ensure_main_thread
def safe_max_operation():
    """在主线程执行 3ds Max 操作"""
    import MaxPlus
    MaxPlus.ViewportManager.ForceRedraw()
```

## 另请参阅

- [Qt 集成指南](../guide/qt-integration.md)
- [线程调度器](../guide/thread-dispatcher.md)
