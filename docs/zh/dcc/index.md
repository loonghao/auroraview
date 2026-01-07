# DCC 集成

AuroraView 专为数字内容创作 (DCC) 软件集成而设计。

## 支持的软件

| DCC 软件 | 状态 | Python 版本 | 集成模式 |
|----------|------|-------------|----------|
| [Maya](./maya) | ✅ 已支持 | 3.7+ | Qt 模式 |
| [Houdini](./houdini) | ✅ 已支持 | 3.7+ | Qt 模式 |
| [3ds Max](./3dsmax) | ✅ 已支持 | 3.7+ | Qt 模式 |
| [Blender](./blender) | ✅ 已支持 | 3.7+ | 桌面 / 原生模式 |
| Nuke | ✅ 已支持 | 3.7+ | Qt 模式 |
| [Substance Painter](./substance-painter) | 🚧 进行中 | 3.9+ | Qt 模式 |
| [Unreal Engine](./unreal) | 🚧 进行中 | 3.9+ | 原生模式 (HWND) |
| [Photoshop](./photoshop) | 🚧 计划中 | 3.9+ | WebSocket |

## 集成模式

AuroraView 提供三种集成模式以适应不同场景：

| 模式 | 类 | 描述 | 适用场景 |
|------|-----|------|----------|
| **桌面** | `WebView` + `show()` | 独立窗口，拥有自己的事件循环 | 独立工具、桌面应用 |
| **原生 (HWND)** | `WebView` + `parent=hwnd` | 通过 HWND 嵌入，无 Qt 依赖 | Blender、Unreal Engine、非 Qt 应用 |
| **Qt** | `QtWebView` | 作为 Qt widget 子控件嵌入 | Maya、Houdini、Nuke、3ds Max |

### 桌面模式

**适用于：** 独立工具、开发调试、Blender（浮动窗口）

创建独立窗口，拥有自己的事件循环。

```python
from auroraview import run_desktop

run_desktop(
    title="我的工具",
    url="http://localhost:3000"
)
```

**主要特性：**
- ✅ 完整的窗口特效支持（点击穿透、模糊、云母效果）
- ✅ 无 DCC 依赖
- ✅ 拥有独立事件循环

### 原生模式 (HWND)

**适用于：** Blender、Unreal Engine、非 Qt 应用

通过 HWND 嵌入 WebView，无需 Qt 依赖。

```python
from auroraview import WebView

# 从非 Qt 应用获取父窗口 HWND
parent_hwnd = get_app_window_handle()

webview = WebView.create(
    title="我的工具",
    parent=parent_hwnd,
    mode="owner",
)
webview.show()

# 获取 HWND 用于外部集成
hwnd = webview.get_hwnd()
```

**主要特性：**
- ✅ 直接访问 HWND
- ✅ 完整的窗口特效支持
- ✅ 适用于任何接受 HWND 的应用
- ✅ 无需 Qt 依赖

### Qt 模式

**适用于：** Maya、Houdini、Nuke、3ds Max

创建真正的 Qt widget，可以被 Qt 的父子系统停靠和管理。

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=dcc_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

**主要特性：**
- ✅ 支持 `QDockWidget` 实现可停靠面板
- ✅ 自动生命周期管理
- ✅ 原生 Qt 事件集成
- ✅ 支持所有 Qt 布局管理器
- ⚠️ 窗口特效支持有限

## 安装

### 基础安装

```bash
pip install auroraview
```

### 包含 Qt 支持

对于基于 Qt 的 DCC（Maya、Houdini、Nuke）：

```bash
pip install auroraview[qt]
```

这将安装 QtPy 作为中间件来处理不同的 Qt 版本。

## 常用模式

### 获取主窗口

每个 DCC 都有自己获取主窗口的方式：

::: code-group

```python [Maya]
import maya.OpenMayaUI as omui
from qtpy import QtWidgets
import shiboken2

def maya_main_window():
    ptr = omui.MQtUtil.mainWindow()
    return shiboken2.wrapInstance(int(ptr), QtWidgets.QWidget)
```

```python [Houdini]
import hou

def houdini_main_window():
    return hou.qt.mainWindow()
```

```python [Nuke]
from qtpy import QtWidgets

def nuke_main_window():
    return QtWidgets.QApplication.activeWindow()
```

:::

### 可停靠面板

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget

# 创建 dock widget
dock = QDockWidget("我的工具", main_window)

# 创建 WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# 设置为 dock widget 内容
dock.setWidget(webview)
main_window.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## 生命周期管理

当父 DCC 关闭时，AuroraView 会自动处理清理：

```python
webview = QtWebView(
    parent=dcc_main_window(),  # 监视此父窗口
    url="http://localhost:3000"
)
# 当父窗口销毁时，WebView 自动关闭
```

手动控制：

```python
webview = AuroraView(
    url="http://localhost:3000",
    parent_hwnd=get_dcc_hwnd(),
    parent_mode="owner"
)
# WebView 跟随父窗口最小化/恢复/关闭
```
