# Maya 集成

AuroraView 通过 Qt/PySide 集成与 Maya 无缝配合。

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Maya | 2020 | 2024+ |
| Python | 3.7 | 3.10+ |
| Qt | PySide2/Qt5 | PySide6/Qt6 |

## 快速开始

### 基础用法

```python
from auroraview import QtWebView
import maya.OpenMayaUI as omui
from qtpy import QtWidgets
import shiboken2

def maya_main_window():
    ptr = omui.MQtUtil.mainWindow()
    return shiboken2.wrapInstance(int(ptr), QtWidgets.QWidget)

# 创建 WebView
webview = QtWebView(
    parent=maya_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

### 可停靠面板

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt

def create_dockable_webview():
    main_window = maya_main_window()
    
    # 创建 dock widget
    dock = QDockWidget("AuroraView 工具", main_window)
    dock.setAllowedAreas(Qt.AllDockWidgetAreas)
    
    # 创建 WebView
    webview = QtWebView(parent=dock)
    webview.load_url("http://localhost:3000")
    
    # 设置内容
    dock.setWidget(webview)
    main_window.addDockWidget(Qt.RightDockWidgetArea, dock)
    
    webview.show()
    return dock, webview

dock, webview = create_dockable_webview()
```

## API 通信

### Python API 类

```python
from auroraview import QtWebView
import maya.cmds as cmds

class MayaAPI:
    def get_selection(self):
        """获取当前选择的对象"""
        return cmds.ls(selection=True)
    
    def create_cube(self, name="cube", size=1.0):
        """创建一个立方体"""
        cube = cmds.polyCube(name=name, width=size, height=size, depth=size)
        return cube[0]
    
    def set_transform(self, obj, tx=0, ty=0, tz=0):
        """设置对象变换"""
        cmds.setAttr(f"{obj}.translateX", tx)
        cmds.setAttr(f"{obj}.translateY", ty)
        cmds.setAttr(f"{obj}.translateZ", tz)

webview = QtWebView(
    parent=maya_main_window(),
    api=MayaAPI()
)
```

### JavaScript 调用

```javascript
// 获取选择
const selection = await auroraview.api.get_selection();
console.log('选中的对象:', selection);

// 创建立方体
const cube = await auroraview.api.create_cube('myCube', 2.0);

// 设置位置
await auroraview.api.set_transform(cube, 5, 0, 0);
```

## 线程调度器

Maya 要求某些操作必须在主线程执行：

```python
from auroraview.utils import run_on_main_thread, ensure_main_thread

@ensure_main_thread
def safe_create_node(node_type):
    """此函数始终在主线程运行"""
    import maya.cmds as cmds
    return cmds.createNode(node_type)

# 可以从任何线程安全调用
node = safe_create_node("transform")
```

## 故障排除

### WebView 不显示

**原因**: 父窗口引用丢失。

**解决方案**: 保持对 webview 和 dock widget 的引用。

### 主线程错误

**原因**: 从后台线程调用 Maya API。

**解决方案**: 使用 `@ensure_main_thread` 装饰器。

## 另请参阅

- [Qt 集成指南](../guide/qt-integration.md)
- [线程调度器](../guide/thread-dispatcher.md)
