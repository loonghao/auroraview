# AuroraView

中文文档 | [English](./README.md)

[![PyPI 版本](https://img.shields.io/pypi/v/auroraview.svg)](https://pypi.org/project/auroraview/)
[![Python 版本](https://img.shields.io/pypi/pyversions/auroraview.svg)](https://pypi.org/project/auroraview/)
[![下载量](https://static.pepy.tech/badge/auroraview)](https://pepy.tech/project/auroraview)
[![Codecov](https://codecov.io/gh/loonghao/auroraview/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/auroraview)
[![PR Checks](https://github.com/loonghao/auroraview/actions/workflows/pr-checks.yml/badge.svg)](https://github.com/loonghao/auroraview/actions/workflows/pr-checks.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![平台](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/loonghao/auroraview)
[![CI](https://github.com/loonghao/auroraview/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/ci.yml)
[![Build Wheels](https://github.com/loonghao/auroraview/actions/workflows/build-wheels.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/build-wheels.yml)
[![Release](https://github.com/loonghao/auroraview/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/release.yml)
[![CodeQL](https://github.com/loonghao/auroraview/actions/workflows/codeql.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/codeql.yml)
[![Security Audit](https://github.com/loonghao/auroraview/actions/workflows/security-audit.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/security-audit.yml)
[![Latest Release](https://img.shields.io/github/v/release/loonghao/auroraview?display_name=tag)](https://github.com/loonghao/auroraview/releases)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen.svg)](https://pre-commit.com/)

[![GitHub Stars](https://img.shields.io/github/stars/loonghao/auroraview?style=social)](https://github.com/loonghao/auroraview/stargazers)
[![GitHub Downloads](https://img.shields.io/github/downloads/loonghao/auroraview/total)](https://github.com/loonghao/auroraview/releases)
[![Last Commit](https://img.shields.io/github/last-commit/loonghao/auroraview)](https://github.com/loonghao/auroraview/commits/main)
[![Commit Activity](https://img.shields.io/github/commit-activity/m/loonghao/auroraview)](https://github.com/loonghao/auroraview/graphs/commit-activity)
[![Open Issues](https://img.shields.io/github/issues/loonghao/auroraview)](https://github.com/loonghao/auroraview/issues)
[![Open PRs](https://img.shields.io/github/issues-pr/loonghao/auroraview)](https://github.com/loonghao/auroraview/pulls)
[![Contributors](https://img.shields.io/github/contributors/loonghao/auroraview)](https://github.com/loonghao/auroraview/graphs/contributors)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
[![release-please](https://img.shields.io/badge/release--please-enabled-blue)](https://github.com/googleapis/release-please)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-025E8C?logo=dependabot)](./.github/dependabot.yml)
[![Code Style: ruff](https://img.shields.io/badge/code%20style-ruff-000000.svg)](https://docs.astral.sh/ruff/)
[![Type Checked: mypy](https://img.shields.io/badge/type%20checked-mypy-2A6DB0.svg)](http://mypy-lang.org/)

[行为准则](./CODE_OF_CONDUCT.md) • [安全策略](./SECURITY.md) • [问题追踪](https://github.com/loonghao/auroraview/issues)


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

### 核心功能
- [OK] **原生 WebView 集成**: 使用系统 WebView (WebView2/WKWebView/WebKitGTK)，占用空间最小
- [OK] **双向通信**: Python ↔ JavaScript IPC，支持 async/await
- [OK] **自定义协议处理器**: 从 DCC 项目加载资源 (`auroraview://`、自定义协议)
- [OK] **事件系统**: Node.js 风格 EventEmitter，支持 `on()`、`once()`、`off()`、`emit()`
- [OK] **多窗口支持**: WindowManager 管理多窗口，支持跨窗口事件通信
- [OK] **线程安全**: Rust 保证的内存安全和并发操作

### 存储与数据
- [OK] **localStorage/sessionStorage**: 完整的 Web 存储 CRUD 操作
- [OK] **Cookie 管理**: set/get/delete/clear cookies
- [OK] **浏览数据清理**: 通过 `clear_browsing_data()` 清理缓存、Cookie、历史

### 窗口与导航
- [OK] **文件对话框**: open_file、save_file、select_folder、select_folders
- [OK] **消息对话框**: confirm、alert、error、ok_cancel
- [OK] **导航控制**: go_back、go_forward、reload、stop、can_go_back/forward
- [OK] **窗口事件**: on_window_show/hide/focus/blur/resize、on_fullscreen_changed

### DCC 集成
- [OK] **生命周期管理**: 父 DCC 应用关闭时自动清理
- [OK] **Qt 后端**: QtWebView 无缝集成基于 Qt 的 DCC
- [OK] **WebView2 预热**: 预初始化 WebView2 加速 DCC 启动
- [OK] **性能监控**: get_performance_metrics()、get_ipc_stats()

### 安全
- [OK] **CSP 配置**: 内容安全策略支持
- [OK] **CORS 控制**: 跨域资源共享管理
- [OK] **权限系统**: 细粒度权限控制

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

AuroraView 提供三种主要集成模式以适应不同的使用场景：

| 模式 | 类 | 适用场景 | 停靠支持 |
|------|-----|----------|----------|
| **Qt 原生** | `QtWebView` | Maya, Houdini, Nuke, 3ds Max | ✅ QDockWidget |
| **HWND** | `AuroraView` | Unreal Engine, 非 Qt 应用 | ✅ 通过 HWND API |
| **独立** | `run_standalone` | 桌面应用程序 | N/A |

#### 1. Qt 原生模式 (QtWebView)

**最适合基于 Qt 的 DCC 应用程序** - Maya, Houdini, Nuke, 3ds Max。

此模式创建真正的 Qt 控件，可以停靠、嵌入布局，并由 Qt 的父子系统管理。

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDialog, QVBoxLayout

# 创建可停靠对话框
dialog = QDialog(maya_main_window())
layout = QVBoxLayout(dialog)

# 创建嵌入式 WebView 作为 Qt 控件
webview = QtWebView(
    parent=dialog,
    width=800,
    height=600
)
layout.addWidget(webview)

# 加载内容
webview.load_url("http://localhost:3000")

# 显示对话框 - WebView 会随父窗口自动关闭
dialog.show()
webview.show()
```

**主要特性：**
- ✅ 支持 `QDockWidget` 可停靠面板
- ✅ 自动生命周期管理（随父窗口关闭）
- ✅ 原生 Qt 事件集成
- ✅ 支持所有 Qt 布局管理器

#### 2. HWND 模式 (AuroraView)

**最适合 Unreal Engine 和非 Qt 应用程序**，需要直接访问窗口句柄。

```python
from auroraview import AuroraView

# 创建独立 WebView
webview = AuroraView(url="http://localhost:3000")
webview.show()

# 获取 HWND 用于外部集成
hwnd = webview.get_hwnd()
if hwnd:
    # Unreal Engine 集成
    import unreal
    unreal.parent_external_window_to_slate(hwnd)
```

**主要特性：**
- ✅ 通过 `get_hwnd()` 直接访问 HWND
- ✅ 适用于任何接受 HWND 的应用程序
- ✅ 无需 Qt 依赖
- ✅ 完全控制窗口定位

#### 3. 独立模式

**最适合桌面应用程序** - 一行代码启动独立应用。

```python
from auroraview import run_standalone

# 启动独立应用（阻塞直到关闭）
run_standalone(
    title="我的应用",
    url="https://example.com",
    width=1024,
    height=768
)
```

**主要特性：**
- ✅ 最简单的 API - 一个函数调用
- ✅ 自动事件循环管理
- ✅ 无需父窗口

**回调反注册（EventTimer）**：
```python
from auroraview import EventTimer

timer = EventTimer(webview, interval_ms=16)

def _on_close(): ...

timer.on_close(_on_close)
# 之后如需移除：
timer.off_close(_on_close)  # 也支持：off_tick(handler)
```

**共享状态（借鉴 PyWebView）**：

AuroraView 提供 Python 和 JavaScript 之间的自动双向状态同步：

```python
from auroraview import WebView

webview = WebView.create("我的应用", width=800, height=600)

# 访问共享状态（类字典接口）
webview.state["user"] = "Alice"
webview.state["theme"] = "dark"
webview.state["count"] = 0

# 跟踪状态变化
@webview.state.on_change
def on_state_change(key: str, value, old_value):
    print(f"状态变化: {key} = {value} (原值 {old_value})")

# 在 JavaScript 中：
# window.auroraview.state.user = "Bob";  // 同步到 Python
# console.log(window.auroraview.state.theme);  // "dark"
```

**命令系统（借鉴 Tauri）**：

将 Python 函数注册为可从 JavaScript 调用的 RPC 风格命令：

```python
from auroraview import WebView

webview = WebView.create("我的应用", width=800, height=600)

# 使用装饰器注册命令
@webview.command
def greet(name: str) -> str:
    return f"你好, {name}!"

@webview.command("add_numbers")
def add(x: int, y: int) -> int:
    return x + y

# 在 JavaScript 中：
# const msg = await auroraview.invoke("greet", {name: "World"});
# const sum = await auroraview.invoke("add_numbers", {x: 1, y: 2});
```

**Channel 流式传输**：

使用 Channel 从 Python 向 JavaScript 流式传输大数据：

```python
from auroraview import WebView

webview = WebView.create("我的应用", width=800, height=600)

# 创建用于流式传输的 channel
with webview.create_channel() as channel:
    for i in range(100):
        channel.send({"progress": i, "data": f"chunk_{i}"})

# 在 JavaScript 中：
# const channel = auroraview.channel("channel_id");
# channel.onMessage((data) => console.log("收到:", data));
# channel.onClose(() => console.log("流传输完成"));
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

### 窗口事件系统

AuroraView 提供完整的窗口事件系统，用于跟踪窗口生命周期：

```python
from auroraview import WebView
from auroraview.core.events import WindowEvent, WindowEventData

webview = WebView(title="我的应用", width=800, height=600)

# 使用装饰器注册窗口事件处理器
@webview.on_shown
def on_shown(data: WindowEventData):
    print("窗口已显示")

@webview.on_focused
def on_focused(data: WindowEventData):
    print("窗口获得焦点")

@webview.on_blurred
def on_blurred(data: WindowEventData):
    print("窗口失去焦点")

@webview.on_resized
def on_resized(data: WindowEventData):
    print(f"窗口大小调整为 {data.width}x{data.height}")

@webview.on_moved
def on_moved(data: WindowEventData):
    print(f"窗口移动到 ({data.x}, {data.y})")

@webview.on_closing
def on_closing(data: WindowEventData):
    print("窗口正在关闭...")
    return True  # 返回 True 允许关闭，False 取消关闭

# 窗口控制方法
webview.resize(1024, 768)
webview.move(100, 100)
webview.minimize()
webview.maximize()
webview.restore()
webview.toggle_fullscreen()
webview.focus()
webview.hide()

# 只读窗口属性
print(f"大小: {webview.width}x{webview.height}")
print(f"位置: ({webview.x}, {webview.y})")
```

### 高级功能

#### 自定义协议处理器

AuroraView 提供强大的自定义协议系统，解决本地资源加载的 CORS 限制问题。

**内置 `auroraview://` 协议**

用于加载本地资源（CSS、JS、图片等），无 CORS 限制：

```python
from auroraview import WebView

webview = WebView(
    title="我的应用",
    asset_root="C:/projects/my_app/assets"  # 配置资源根目录
)

# HTML 中使用 auroraview:// 协议
webview.load_html("""
<html>
    <head>
        <link rel="stylesheet" href="auroraview://css/style.css">
    </head>
    <body>
        <img src="auroraview://images/logo.png">
        <script src="auroraview://js/app.js"></script>
    </body>
</html>
""")
```

**路径映射**:
- `auroraview://css/style.css` → `{asset_root}/css/style.css`
- `auroraview://images/logo.png` → `{asset_root}/images/logo.png`

**自定义协议注册**

为 DCC 特定资源创建自定义协议：

```python
from auroraview import WebView

webview = WebView(title="Maya 工具")

# 注册自定义协议处理器
def handle_maya_protocol(uri: str) -> dict:
    """处理 maya:// 协议请求"""
    # 从 URI 提取路径: maya://thumbnails/character.jpg
    path = uri.replace("maya://", "")

    # 加载 Maya 项目资源
    full_path = f"C:/maya_projects/current/{path}"

    try:
        with open(full_path, "rb") as f:
            return {
                "data": f.read(),
                "mime_type": "image/jpeg",
                "status": 200
            }
    except FileNotFoundError:
        return {
            "data": b"Not Found",
            "mime_type": "text/plain",
            "status": 404
        }

# 注册协议
webview.register_protocol("maya", handle_maya_protocol)

# HTML 中使用
webview.load_html("""
<html>
    <body>
        <h1>Maya 资源浏览器</h1>
        <img src="maya://thumbnails/character.jpg">
        <video src="maya://previews/animation.mp4"></video>
    </body>
</html>
""")
```

**高级示例：FBX 文件加载**

```python
def handle_fbx_protocol(uri: str) -> dict:
    """加载 FBX 模型文件"""
    path = uri.replace("fbx://", "")
    full_path = f"C:/models/{path}"

    try:
        with open(full_path, "rb") as f:
            return {
                "data": f.read(),
                "mime_type": "application/octet-stream",
                "status": 200
            }
    except Exception as e:
        return {
            "data": str(e).encode(),
            "mime_type": "text/plain",
            "status": 500
        }

webview.register_protocol("fbx", handle_fbx_protocol)

# JavaScript 中使用
webview.load_html("""
<script>
    fetch('fbx://characters/hero.fbx')
        .then(r => r.arrayBuffer())
        .then(data => {
            // 处理 FBX 数据
            console.log('FBX 文件大小:', data.byteLength);
        });
</script>
""")
```

**优势**:
- ✅ **无 CORS 限制** - 解决 `file://` 协议的跨域问题
- ✅ **简洁 URL** - `maya://logo.png` vs `file:///C:/long/path/logo.png`
- ✅ **安全控制** - 限制访问范围，防止目录遍历攻击
- ✅ **灵活扩展** - 支持从内存、数据库、网络加载资源
- ✅ **跨平台** - 路径处理统一，Windows/macOS/Linux 一致

#### 自定义协议最佳实践

##### 平台特定 URL 格式

`auroraview://` 协议在不同平台使用不同的 URL 格式：

| 平台 | URL 格式 | 示例 |
|------|----------|------|
| **Windows** | `https://auroraview.localhost/path` | `https://auroraview.localhost/index.html` |
| **macOS** | `auroraview://path` | `auroraview://index.html` |
| **Linux** | `auroraview://path` | `auroraview://index.html` |

> **注意**: 在 Windows 上，wry（底层 WebView 库）会将自定义协议映射为 HTTP/HTTPS 格式。
> 我们使用 `.localhost` 作为主机名以确保安全性。

##### 为什么 `.localhost` 是安全的

`.localhost` 顶级域名提供了强大的安全保障：

1. **IANA 保留域名** - `.localhost` 是保留顶级域（RFC 6761），任何人都无法注册
2. **仅限本地** - 浏览器将 `.localhost` 视为本地地址 (127.0.0.1)
3. **DNS 前拦截** - 我们的协议处理器在 DNS 解析之前拦截请求
4. **无网络流量** - 请求永远不会离开本地机器

##### 本地资源加载方式对比

| 方式 | 安全性 | 推荐程度 |
|------|--------|----------|
| `auroraview://` + `asset_root` | ✅ **高** - 访问限制在指定目录 | **推荐** |
| `allow_file_protocol=True` | ⚠️ 低 - 可访问系统任意文件 | 谨慎使用 |
| HTTP 服务器 | ✅ 高 - 可控访问 | 适合开发环境 |

**推荐方式（使用 `asset_root` 配合相对路径）：**

<table>
<tr><th>WebView.create()</th><th>run_standalone()</th></tr>
<tr>
<td>

```python
from auroraview import WebView

# 安全：只能访问 assets/ 目录下的文件
webview = WebView.create(
    title="我的应用",
    asset_root="./assets",
)

# 使用相对路径 - 会解析到 asset_root 目录
html = """
<html>
<body>
    <img src="./images/logo.png">
    <img src="./images/animation.gif">
</body>
</html>
"""
webview.load_html(html)
```

</td>
<td>

```python
from auroraview import run_standalone

# 安全：只能访问 assets/ 目录下的文件
# 使用相对路径 - 会解析到 asset_root 目录
html = """
<html>
<body>
    <img src="./images/logo.png">
    <img src="./images/animation.gif">
</body>
</html>
"""

run_standalone(
    title="我的应用",
    html=html,
    asset_root="./assets",
)
```

</td>
</tr>
</table>

**不推荐方式（使用 `file://` 协议）：**

<table>
<tr><th>WebView.create()</th><th>run_standalone()</th></tr>
<tr>
<td>

```python
from auroraview import WebView
from auroraview import path_to_file_url

# ⚠️ 警告：允许访问任意文件
gif_url = path_to_file_url("C:/path/to/animation.gif")

webview = WebView.create(
    title="我的应用",
    allow_file_protocol=True,
)

html = f'<img src="{gif_url}">'
webview.load_html(html)
```

</td>
<td>

```python
from auroraview import run_standalone
from auroraview import path_to_file_url

# ⚠️ 警告：允许访问任意文件
gif_url = path_to_file_url("C:/path/to/animation.gif")

html = f'<img src="{gif_url}">'

run_standalone(
    title="我的应用",
    html=html,
    allow_file_protocol=True,
)
```

</td>
</tr>
</table>

> **注意**：`path_to_file_url()` 辅助函数将本地路径转换为正确的 `file:///` URL。
> 例如：`C:\images\logo.gif` → `file:///C:/images/logo.gif`

完整示例请参考 [examples/custom_protocol_example.py](./examples/custom_protocol_example.py) 和 [examples/local_assets_example.py](./examples/local_assets_example.py)。

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

