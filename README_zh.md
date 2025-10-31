# AuroraView

中文文档 | [English](./README.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.7+-blue.svg)](https://www.python.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/loonghao/auroraview)
[![CI](https://github.com/loonghao/auroraview/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/auroraview/actions)

一个为DCC（数字内容创作）软件设计的超快速、轻量级WebView框架，使用Rust构建并提供Python绑定。完美支持Maya、3ds Max、Houdini、Blender等。

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

## 快速开始

### 安装

```bash
pip install auroraview
```

### 基础用法

```python
from auroraview import WebView

# 创建WebView实例
webview = WebView(
    title="我的应用",
    width=800,
    height=600,
    url="http://localhost:3000"
)

# 显示窗口
webview.show()
```

### 双向通信

```python
# Python → JavaScript
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})

# JavaScript → Python
@webview.on("export_scene")
def handle_export(data):
    print(f"导出到: {data['path']}")
    # 你的DCC导出逻辑
```

## [DOCS] 文档

-  [项目综述](./docs/SUMMARY.md)
-  [当前进展](./docs/CURRENT_STATUS.md)
-  [技术设计](./docs/TECHNICAL_DESIGN.md)
-  [DCC 集成指南](./docs/DCC_INTEGRATION_GUIDE.md)
-  [项目优势](./docs/PROJECT_ADVANTAGES.md)
-  [与 PyWebView 的对比](./docs/COMPARISON_WITH_PYWEBVIEW.md)
-  [路线图](./docs/ROADMAP.md)

##  DCC软件支持

| DCC软件 | 状态 | Python版本 | 示例 |
|---------|------|-----------|------|
| Maya | [OK] 已支持 | 3.7+ | [示例](./examples/maya/) |
| 3ds Max | [OK] 已支持 | 3.7+ | - |
| Houdini | [OK] 已支持 | 3.7+ | [示例](./examples/houdini/) |
| Blender | [OK] 已支持 | 3.7+ | [示例](./examples/blender/) |
| Photoshop | [CONSTRUCTION] 计划中 | 3.7+ | - |
| Unreal Engine | [CONSTRUCTION] 计划中 | 3.7+ | - |

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

```bash
# Rust测试
cargo test

# Python测试
pytest tests/
```

## [PACKAGE] 项目结构

```
auroraview/
├── src/                    # Rust核心库
├── python/                 # Python绑定
├── examples/               # DCC集成示例
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

---

**注意**: 本项目正在积极开发中。v1.0.0发布前API可能会有变化。

