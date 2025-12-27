# 快速开始

AuroraView 是一个为 DCC（数字内容创作）软件设计的轻量级 WebView 框架，使用 Rust 构建并提供 Python 绑定。

## 前置要求

- Python 3.7+
- Windows、macOS 或 Linux

## 安装

### Windows 和 macOS

**基础安装**（仅原生后端）：

```bash
pip install auroraview
```

**带 Qt 支持**（用于 Maya、Houdini、Nuke 等基于 Qt 的 DCC）：

```bash
pip install auroraview[qt]
```

### Linux

Linux 需要 webkit2gtk 系统依赖：

```bash
# 首先安装系统依赖
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev  # Debian/Ubuntu
# sudo dnf install gtk3-devel webkit2gtk3-devel      # Fedora/CentOS
# sudo pacman -S webkit2gtk                          # Arch Linux

# 从 PyPI 安装
pip install auroraview
```

## 快速开始

### 桌面应用

创建桌面应用的最简方式：

```python
from auroraview import run_desktop

run_desktop(
    title="我的应用",
    url="http://localhost:3000",
    width=1024,
    height=768
)
```

### 加载 HTML 内容

```python
from auroraview import WebView

html = """
<!DOCTYPE html>
<html>
<body>
    <h1>Hello from AuroraView!</h1>
    <button onclick="alert('Hello!')">点击我</button>
</body>
</html>
"""

webview = WebView.create("我的应用", html=html)
webview.show()
```

### DCC 集成

用于基于 Qt 的 DCC 应用（Maya、Houdini、Nuke）：

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=dcc_main_window(),  # 你的 DCC 主窗口
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

## 集成模式

AuroraView 提供三种集成模式：

| 模式 | 类 | 描述 | 适用场景 |
|------|-----|------|----------|
| **桌面模式** | `WebView` + `show()` | 独立窗口，拥有自己的事件循环 | 独立工具、桌面应用 |
| **原生模式 (HWND)** | `WebView` + `parent=hwnd` | 通过 HWND 嵌入，不依赖 Qt | Blender、Unreal Engine、非 Qt 应用 |
| **Qt 模式** | `QtWebView` | 作为 Qt widget 子窗口嵌入 | Maya、Houdini、Nuke、3ds Max |

## 下一步

- [架构概述](/zh/guide/architecture) - 了解 AuroraView 架构
- [Qt 集成](/zh/guide/qt-integration) - Qt WebView 集成指南
- [JavaScript 通信](/zh/guide/communication) - 前后端通信
- [打包指南](/zh/guide/packing) - 打包和分发应用
