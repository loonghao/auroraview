# AuroraView

中文文档 | [English](./README.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.7+-blue.svg)](https://www.python.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/loonghao/auroraview)
[![CI](https://github.com/loonghao/auroraview/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/auroraview/actions)

一个为DCC（数字内容创作）软件设计的超快速、轻量级WebView框架，使用Rust构建并提供Python绑定。完美支持Maya、3ds Max、Houdini、Blender等。

> **⚠️ 开发状态**: 本项目正在积极开发中。API 可能在 v1.0.0 发布前发生变化。项目尚未在 Linux 和 macOS 平台上进行广泛测试。

## [TARGET] 概述

AuroraView 为专业DCC应用程序（如Maya、3ds Max、Houdini、Blender、Photoshop和Unreal Engine）提供现代化的Web UI解决方案。基于Rust的Wry库和PyO3绑定构建，提供原生性能和最小开销。

### 为什么选择 AuroraView？

- ** 轻量级**: 约5MB包体积，而Electron约120MB
- **[LIGHTNING] 快速**: 原生性能，内存占用<30MB
- **[LINK] 无缝集成**: 为所有主流DCC工具提供简单的Python API
- **[GLOBE] 现代Web技术栈**: 支持React、Vue或任何Web框架
- **[LOCK] 安全**: Rust的内存安全保证
- **[PACKAGE] 跨平台**: 支持Windows、macOS和Linux

## [ARCHITECTURE] 架构

```
┌─────────────────────────────────────────────────────────┐
│         DCC软件 (Maya/Max/Houdini等)                    │
└────────────────────┬────────────────────────────────────┘
                     │ Python API
                     ▼
┌─────────────────────────────────────────────────────────┐
│               auroraview (Python包)                     │
│                   PyO3绑定                               │
└────────────────────┬────────────────────────────────────┘
                     │ FFI
                     ▼
┌─────────────────────────────────────────────────────────┐
│           auroraview_core (Rust库)                      │
│                  Wry WebView引擎                         │
└────────────────────┬────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────┐
│              系统原生WebView                             │
│    Windows: WebView2 | macOS: WKWebView | Linux: WebKit│
└─────────────────────────────────────────────────────────┘
```
##  技术框架

- 核心栈：Rust 1.75+、PyO3 0.22（abi3）、Wry 0.47、Tao 0.30
- 引擎：Windows（WebView2）、macOS（WKWebView）、Linux（WebKitGTK）
- 打包：maturin + abi3 → 单个 wheel 兼容 CPython 3.73.12
- 事件循环：默认阻塞式 show()；后续提供非阻塞模式以适配宿主循环
- 延迟加载：在 show() 前设置的 URL/HTML 会保存并在创建时应用（最后写入生效）
- IPC：Python ↔ JavaScript 双向事件总线（基于 CustomEvent）
- 协议：自定义协议与资源加载（如 dcc://）
- 嵌入：支持父窗口句柄（HWND/NSView/WId）的 DCC 宿主嵌入（路线图）
- 安全：可选的开发者工具、CSP 钩子、远程 URL 白名单（规划中）
- 性能目标：本地 HTML 首屏 <150ms、基线内存 <50MB

### 技术细节
- Python API：`auroraview.WebView` 封装 Rust 核心并提供易用增强
- Rust 核心：使用 Arc<Mutex<...>> 的内部可变配置，安全支持 show() 前更新
- 生命周期：在 `show()` 时创建 WebView，并应用 URL/HTML（最后写入生效）
- JS 桥：Python 侧 `emit(event, data)`；JS 侧通过 `CustomEvent('py', {...})` 回传到 Python（IpcHandler）
- 日志：Rust 端 `tracing`；Python 端 `logging`
- 测试：pytest 冒烟 + cargo 测试；CI 构建三平台 wheel


## 特性

- [OK] **原生WebView集成**: 使用系统WebView，占用空间最小
- [OK] **双向通信**: Python ↔ JavaScript IPC
- [OK] **自定义协议处理器**: 从DCC项目加载资源
- [OK] **事件系统**: 响应式事件驱动架构
- [OK] **多窗口支持**: 创建多个WebView实例
- [OK] **线程安全**: 安全的并发操作
- [OK] **热重载**: 开发模式支持实时重载
- [OK] **生命周期管理**: 父DCC应用关闭时自动清理
- [OK] **第三方网站集成**: JavaScript注入支持外部网站
- [OK] **AI聊天集成**: 内置AI助手集成支持

## 快速开始

### 安装

#### Windows 和 macOS

```bash
pip install auroraview
```

#### Linux

由于 webkit2gtk 系统依赖，Linux wheels 不在 PyPI 上提供。请从 GitHub Releases 安装：

```bash
# 首先安装系统依赖
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev  # Debian/Ubuntu
# sudo dnf install gtk3-devel webkit2gtk3-devel      # Fedora/CentOS
# sudo pacman -S webkit2gtk                          # Arch Linux

# 从 GitHub Releases 下载并安装 wheel
pip install https://github.com/loonghao/auroraview/releases/latest/download/auroraview-{version}-cp37-abi3-linux_x86_64.whl
```

或从源码构建：
```bash
pip install auroraview --no-binary :all:
```

### 集成模式

AuroraView 支持两种集成模式以适应不同的使用场景:

#### 1. 原生后端 (默认)

使用平台特定的 API (Windows 上的 HWND) 进行窗口嵌入。最适合独立应用程序和最大兼容性。

**独立窗口:**
```python
from auroraview import WebView

# 方法 1: 直接加载 HTML 内容（推荐入门使用）
webview = WebView(
    title="我的应用",
    width=800,
    height=600
)
webview.load_html("""
    <!DOCTYPE html>
    <html>
    <body>
        <h1>你好，AuroraView！</h1>
        <p>这是一个简单的示例。</p>
    </body>
    </html>
""")
webview.show()  # 阻塞调用

# 方法 2: 从 URL 加载（确保服务器已启动！）
webview = WebView(
    title="我的应用",
    width=800,
    height=600
)
webview.load_url("http://localhost:3000")
webview.show()  # 阻塞调用
```

**嵌入到 DCC (例如 Maya):**
```python
from auroraview import WebView
import maya.OpenMayaUI as omui

# 获取 Maya 主窗口句柄
maya_hwnd = int(omui.MQtUtil.mainWindow())

# 创建嵌入式 WebView（自动定时器，无需手动 Qt 定时器）
webview = WebView.create(
    "Maya 工具",
    parent=maya_hwnd,

)
webview.show()  # 嵌入模式：非阻塞
```

#### 2. Qt 后端

作为 Qt widget 集成,与基于 Qt 的 DCC 无缝集成。需要 `pip install auroraview[qt]`。

> **DCC 集成说明**: 基于 Qt 的 DCC 应用（Maya、Houdini、Nuke、3ds Max）需要 QtPy 作为中间件层来处理不同 DCC 应用之间的 Qt 版本差异。安装 `[qt]` 扩展会自动安装 QtPy。

```python
from auroraview import QtWebView

# 创建 WebView 作为 Qt widget
webview = QtWebView(
    parent=maya_main_window(),  # 任何 QWidget (可选)
    title="我的工具",
    width=800,
    height=600
)

# 加载内容
webview.load_url("http://localhost:3000")
# 或加载 HTML
webview.load_html("<html><body><h1>你好,来自 Qt!</h1></body></html>")

# 显示 widget
webview.show()
```

**何时使用 Qt 后端:**
- [OK] 你的 DCC 已经加载了 Qt (Maya, Houdini, Nuke)
- [OK] 你想要无缝的 Qt widget 集成
- [OK] 你需要使用 Qt 布局和信号/槽

**何时使用原生后端:**
- [OK] 所有平台的最大兼容性
- [OK] 独立应用程序
- [OK] 没有 Qt 的 DCC (Blender, 3ds Max)
- [OK] 最小依赖

### 双向通信

两种后端都支持相同的事件 API:

```python
# Python → JavaScript
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})

# JavaScript → Python
@webview.on("export_scene")
def handle_export(data):
    print(f"导出到: {data['path']}")
    # 你的 DCC 导出逻辑

# 或直接注册回调
webview.register_callback("export_scene", handle_export)
```

**JavaScript 端:**
```javascript
// 监听来自 Python 的事件
window.auroraview.on('update_data', (data) => {
    console.log('帧:', data.frame);
    console.log('对象:', data.objects);
});

// 发送事件到 Python
window.auroraview.send_event('export_scene', {
    path: '/path/to/export.fbx'
});
```

### 高级功能

#### 生命周期管理

当父DCC应用关闭时自动关闭WebView:

```python
from auroraview import WebView

# 获取父窗口句柄 (Windows上的HWND)
parent_hwnd = get_maya_main_window_hwnd()  # 你的DCC特定函数

webview = WebView(
    title="我的工具",
    width=800,
    height=600,
    parent_hwnd=parent_hwnd,  # 监控这个父窗口
    parent_mode="owner"  # 使用owner模式以保证跨线程安全
)

webview.show()
# 当父窗口被销毁时，WebView会自动关闭
```

#### 第三方网站集成

向第三方网站注入JavaScript并建立双向通信:

```python
from auroraview import WebView

webview = WebView(title="AI聊天", width=1200, height=800, dev_tools=True)

# 注册事件处理器
@webview.on("get_scene_info")
def handle_get_scene_info(data):
    # 获取DCC场景数据
    selection = maya.cmds.ls(selection=True)
    webview.emit("scene_info_response", {"selection": selection})

@webview.on("execute_code")
def handle_execute_code(data):
    # 在DCC中执行AI生成的代码
    code = data.get("code", "")
    exec(code)
    webview.emit("execution_result", {"status": "success"})

# 加载第三方网站
webview.load_url("https://ai-chat-website.com")

# 注入自定义JavaScript
injection_script = """
(function() {
    // 向页面添加自定义按钮
    const btn = document.createElement('button');
    btn.textContent = '获取DCC选择';
    btn.onclick = () => {
        window.dispatchEvent(new CustomEvent('get_scene_info', {
            detail: { timestamp: Date.now() }
        }));
    };
    document.body.appendChild(btn);

    // 监听响应
    window.addEventListener('scene_info_response', (e) => {
        console.log('DCC选择:', e.detail);
    });
})();
"""

import time
time.sleep(1)  # 等待页面加载
webview.eval_js(injection_script)

webview.show()
```

详细指南请参阅 [第三方网站集成指南](./docs/THIRD_PARTY_INTEGRATION.md)。

## [DOCS] 文档

### 核心文档
-  [项目综述](./docs/SUMMARY.md)
-  [技术设计](./docs/TECHNICAL_DESIGN.md)
-  [DCC 集成指南](./docs/DCC_INTEGRATION_GUIDE.md)
-  [第三方网站集成指南](./docs/THIRD_PARTY_INTEGRATION.md)

### Maya 集成专题 ⭐
- **[Maya 集成解决方案](./docs/MAYA_SOLUTION.md)** - 推荐阅读！完整的 Maya 集成指南
- [Maya 集成问题分析](./docs/MAYA_INTEGRATION_ISSUES.md) - 技术细节和问题根源
- [当前状态说明](./docs/CURRENT_STATUS.md) - 已知限制和可用方案

### 重要提示：Maya 用户必读 🎯

如果你在 Maya 中使用 AuroraView，请根据你的需求选择合适的模式：

**场景 1: 只需要显示网页（推荐）**
- 使用 **Embedded 模式**
- 特点: 完全非阻塞，Maya 保持响应，自动生命周期管理
- 限制: JavaScript 注入暂不可用

**场景 2: 需要 JavaScript 注入和双向通信**
- 使用 **Standalone 模式**
- 特点: 所有功能可用，包括 `eval_js()` 和 `emit()`
- 限制: 可能有轻微阻塞，需要手动管理生命周期

详细说明请查看 [Maya 集成解决方案](./docs/MAYA_SOLUTION.md)。
-  [第三方网站集成指南](./docs/THIRD_PARTY_INTEGRATION.md) - **新!** JavaScript注入和AI聊天集成
-  [项目优势](./docs/PROJECT_ADVANTAGES.md)
-  [与 PyWebView 的对比](./docs/COMPARISON_WITH_PYWEBVIEW.md)
-  [路线图](./docs/ROADMAP.md)

##  DCC软件支持

| DCC软件 | 状态 | Python版本 | 示例 |
|---------|------|-----------|------|
| Maya | [OK] 已支持 | 3.7+ | [Maya Outliner 示例](https://github.com/loonghao/auroraview-maya-outliner) |
| 3ds Max | [OK] 已支持 | 3.7+ | - |
| Houdini | [OK] 已支持 | 3.7+ | - |
| Blender | [OK] 已支持 | 3.7+ | - |
| Photoshop | [CONSTRUCTION] 计划中 | 3.7+ | - |
| Unreal Engine | [CONSTRUCTION] 计划中 | 3.7+ | - |

> **📚 示例**: 查看完整的工作示例，请访问 [Maya Outliner 示例](https://github.com/loonghao/auroraview-maya-outliner) - 使用 AuroraView、Vue 3 和 TypeScript 构建的现代化 Maya Outliner。

## [TOOLS] 开发

### 前置要求

- Rust 1.75+
- Python 3.7+
- Node.js 18+ (用于示例)

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/loonghao/auroraview.git
cd auroraview

# 安装Rust依赖并构建
cargo build --release

# 以开发模式安装Python包
pip install -e .
```

### 运行测试

AuroraView 为 Qt 和非 Qt 环境提供了全面的测试覆盖。

**不带 Qt 依赖的测试**（测试错误处理）：
```bash
# 使用 nox（推荐）
uvx nox -s pytest

# 或直接使用 pytest
uv run pytest tests/test_qt_import_error.py -v
```

**带 Qt 依赖的测试**（测试实际 Qt 功能）：
```bash
# 使用 nox（推荐）
uvx nox -s pytest-qt

# 或直接使用 pytest
pip install auroraview[qt] pytest pytest-qt
pytest tests/test_qt_backend.py -v
```

**运行所有测试**：
```bash
uvx nox -s pytest-all
```

**测试结构**：

- `tests/test_qt_import_error.py` - 测试未安装 Qt 时的错误处理
  - 验证占位符类正常工作
  - 测试诊断变量（`_HAS_QT`、`_QT_IMPORT_ERROR`）
  - 确保显示有用的错误消息

- `tests/test_qt_backend.py` - 测试实际的 Qt 后端功能
  - 需要安装 Qt 依赖
  - 测试 QtWebView 实例化和方法
  - 测试事件处理和 JavaScript 集成
  - 验证与 AuroraViewQt 别名的向后兼容性

**可用的 Nox 会话**：

```bash
# 列出所有可用的测试会话
uvx nox -l

# 常用会话：
uvx nox -s pytest          # 不带 Qt 的测试
uvx nox -s pytest-qt       # 带 Qt 的测试
uvx nox -s pytest-all      # 运行所有测试
uvx nox -s lint            # 运行代码检查
uvx nox -s format          # 格式化代码
uvx nox -s coverage        # 生成覆盖率报告
```

## [PACKAGE] 项目结构

```
auroraview/
├── src/                    # Rust核心库
├── python/                 # Python绑定
├── tests/                  # 测试套件
├── docs/                   # 文档
└── benches/                # 性能基准测试
```

## [HANDSHAKE] 贡献

欢迎贡献！请阅读我们的[贡献指南](./CONTRIBUTING.md)了解详情。

## [DOCUMENT] 许可证

本项目采用MIT许可证 - 详见[LICENSE](./LICENSE)文件。

## [THANKS] 致谢

- [Wry](https://github.com/tauri-apps/wry) - 跨平台WebView库
- [PyO3](https://github.com/PyO3/pyo3) - Python的Rust绑定
- [Tauri](https://tauri.app/) - 灵感和生态系统

## [MAILBOX] 联系方式

- 作者: Hal Long
- 邮箱: hal.long@outlook.com
- GitHub: [@loonghao](https://github.com/loonghao)

