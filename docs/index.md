---
layout: home

hero:
  name: AuroraView
  text: Lightweight WebView for DCC Software
  tagline: Build modern web-based UI for Maya, Houdini, Blender, and more with Rust performance
  image:
    src: /logo.png
    alt: AuroraView
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/loonghao/auroraview

features:
  - icon: 🚀
    title: Lightweight
    details: ~5MB package size vs ~120MB for Electron. Native Rust performance with minimal memory footprint.
  - icon: 🎨
    title: DCC-First Design
    details: Built specifically for Maya, Houdini, 3ds Max, Blender, Photoshop, and Unreal Engine integration.
  - icon: 🔗
    title: Seamless Integration
    details: Easy Python API with Qt widget support for dockable panels and native DCC integration.
  - icon: 🌐
    title: Modern Web Stack
    details: Use React, Vue, or any web framework. Full bidirectional Python ↔ JavaScript communication.
  - icon: 🔒
    title: Safe & Reliable
    details: Rust's memory safety guarantees. Thread-safe operations and automatic lifecycle management.
  - icon: 📦
    title: Easy Packaging
    details: Bundle your app into a single executable with embedded Python runtime for offline distribution.
---

## Quick Start

### Installation

```bash
# Basic installation
pip install auroraview

# With Qt support (for Maya, Houdini, Nuke)
pip install auroraview[qt]
```

### Desktop Application

```python
from auroraview import run_desktop

run_desktop(
    title="My App",
    url="http://localhost:3000"
)
```

### Maya Integration

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

## Supported DCC Software

| Software | Status | Integration Mode |
|----------|--------|------------------|
| Maya | ✅ Supported | Qt Mode |
| Houdini | ✅ Supported | Qt Mode |
| 3ds Max | ✅ Supported | Qt Mode |
| Blender | ✅ Supported | Desktop / Native Mode |
| Photoshop | 🚧 Planned | - |
| Unreal Engine | 🚧 Planned | Native Mode (HWND) |
