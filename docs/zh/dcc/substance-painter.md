# Substance Painter 集成

AuroraView 通过 Python 脚本 API 与 Adobe Substance Painter 集成。

## 架构

```
┌─────────────────────────────────────────────┐
│           Substance Painter                  │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────┐ │
│  │  Qt 窗口    │ ◄──► │  AuroraView      │ │
│  │  容器       │      │  (WebView2)      │ │
│  └─────────────┘      └──────────────────┘ │
│         │                      │            │
│         │ Qt 父级              │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │   Substance Painter Python API      │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Substance Painter | 2022.1 | 2024.1+ |
| Python | 3.9 | 3.11+ |
| 操作系统 | Windows 10, macOS 11 | Windows 11, macOS 14+ |

## 安装

```bash
# 安装到 Substance Painter 的 Python 环境
pip install auroraview[qt]
```

## 快速开始

### 使用 QtWebView

```python
from auroraview import QtWebView
import substance_painter.ui as ui

# 获取 Substance Painter 主窗口
main_window = ui.get_main_window()

webview = QtWebView(
    parent=main_window,
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
import substance_painter.ui as ui

main_window = ui.get_main_window()

# 创建 dock widget
dock = QDockWidget("我的工具", main_window)

# 创建 WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# 设置内容
dock.setWidget(webview)
main_window.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## 线程安全

AuroraView 为 Substance Painter 集成提供**自动**线程安全。

::: tip 零配置
由于 `dcc_mode="auto"` 是默认值，AuroraView 会自动检测 Substance Painter 并启用线程安全。无需任何配置！
:::

### 自动线程安全（默认）

正常使用 AuroraView 即可 - 线程安全是自动的：

```python
from auroraview import QtWebView
import substance_painter.ui as ui
import substance_painter.project as project
import substance_painter.textureset as textureset

main_window = ui.get_main_window()

# 检测到 Substance Painter 时自动启用线程安全
webview = QtWebView(
    parent=main_window,
    url="http://localhost:3000",
    # dcc_mode="auto" 是默认值 - 无需指定！
)

@webview.on("get_project_info")
def handle_project_info(data):
    # 自动在主线程运行！
    if not project.is_open():
        return {"ok": False, "error": "未打开项目"}
    
    return {
        "ok": True,
        "name": project.name(),
        "file_path": project.file_path(),
        "texture_sets": [ts.name() for ts in textureset.all_texture_sets()]
    }

webview.show()
```

### 使用装饰器手动线程安全

```python
from auroraview import QtWebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async
import substance_painter.ui as ui
import substance_painter.project as project

webview = QtWebView(parent=ui.get_main_window(), url="http://localhost:3000")

@webview.on("save_project")
@dcc_thread_safe  # 阻塞直到保存完成
def handle_save(data):
    if project.is_open():
        project.save()
        return {"ok": True}
    return {"ok": False, "error": "未打开项目"}

@webview.on("refresh_ui")
@dcc_thread_safe_async  # 即发即忘
def handle_refresh(data):
    ui.get_main_window().update()

webview.show()
```

### 直接使用 `run_on_main_thread`

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync
import substance_painter.project as project

# 即发即忘
def close_project():
    if project.is_open():
        project.close()

run_on_main_thread(close_project)

# 阻塞并返回值
def get_project_name():
    if project.is_open():
        return project.name()
    return None

name = run_on_main_thread_sync(get_project_name)
print(f"当前项目: {name}")
```

## 开发状态

| 功能 | 状态 |
|------|------|
| 基础集成 | 🚧 进行中 |
| 图层管理 | 📋 计划中 |
| 导出自动化 | 📋 计划中 |
| 材质同步 | 📋 计划中 |

## 另请参阅

- [Substance Painter Python API](https://substance3d.adobe.com/documentation/spdoc/python-api-194216357.html)
- [Qt 集成指南](../guide/qt-integration)
- [DCC 概览](./index) - 所有 DCC 集成概览

