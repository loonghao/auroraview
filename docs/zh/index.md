---
layout: home

hero:
  name: AuroraView
  text: DCC 软件的轻量级 WebView 框架
  tagline: 为 Maya、Houdini、Blender 等软件构建现代 Web UI，具有 Rust 级别的性能
  image:
    src: /logo.png
    alt: AuroraView
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/getting-started
    - theme: alt
      text: GitHub
      link: https://github.com/loonghao/auroraview

features:
  - icon: 🚀
    title: 轻量级
    details: 约 5MB 包大小，对比 Electron 的约 120MB。原生 Rust 性能，内存占用极低。
  - icon: 🎨
    title: DCC 优先设计
    details: 专为 Maya、Houdini、3ds Max、Blender、Photoshop 和 Unreal Engine 集成而构建。
  - icon: 🔗
    title: 无缝集成
    details: 简洁的 Python API，支持 Qt 小部件，可创建可停靠面板和原生 DCC 集成。
  - icon: 🌐
    title: 现代 Web 技术栈
    details: 使用 React、Vue 或任何 Web 框架。完整的 Python ↔ JavaScript 双向通信。
  - icon: 🔒
    title: 安全可靠
    details: Rust 的内存安全保证。线程安全操作和自动生命周期管理。
  - icon: 📦
    title: 简易打包
    details: 将应用打包成单个可执行文件，内嵌 Python 运行时，支持离线分发。
---

## 快速开始

### 安装

```bash
# 基础安装
pip install auroraview

# 带 Qt 支持（用于 Maya、Houdini、Nuke）
pip install auroraview[qt]
```

### 桌面应用

```python
from auroraview import run_desktop

run_desktop(
    title="我的应用",
    url="http://localhost:3000"
)
```

### Maya 集成

```python
from auroraview import QtWebView
import maya.OpenMayaUI as omui

webview = QtWebView(
    parent=maya_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

## 支持的 DCC 软件

| 软件 | 状态 | 集成模式 |
|------|------|----------|
| Maya | ✅ 已支持 | Qt 模式 |
| Houdini | ✅ 已支持 | Qt 模式 |
| 3ds Max | ✅ 已支持 | Qt 模式 |
| Blender | ✅ 已支持 | 桌面 / 原生模式 |
| Photoshop | 🚧 计划中 | - |
| Unreal Engine | 🚧 计划中 | 原生模式 (HWND) |
