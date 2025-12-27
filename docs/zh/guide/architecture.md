# 架构概述

AuroraView 采用模块化、后端无关的架构设计，支持多种窗口集成模式。

## 设计原则

1. **模块化**: 核心逻辑与平台特定实现清晰分离
2. **可扩展**: 易于添加新的后端和平台支持
3. **类型安全**: 利用 Rust 类型系统确保可靠性
4. **API 一致性**: 不同后端使用统一 API
5. **高性能**: 尽可能使用零成本抽象

## 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                     Python API 层                           │
│  (WebView, QtWebView)                                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   PyO3 绑定层                               │
│  (AuroraView - Python 接口的 Rust 类)                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  后端抽象层                                  │
│  (WebViewBackend trait)                                     │
└─────────────────────────────────────────────────────────────┘
                            │
                ┌───────────┴───────────┐
                ▼                       ▼
┌───────────────────────┐   ┌───────────────────────┐
│   原生后端            │   │    Qt 后端            │
│  (平台特定)           │   │  (Qt 集成)            │
└───────────────────────┘   └───────────────────────┘
                │                       │
                ▼                       ▼
┌───────────────────────┐   ┌───────────────────────┐
│   Wry WebView         │   │  Qt WebEngine         │
│  (WebView2/WebKit)    │   │  (QWebEngineView)     │
└───────────────────────┘   └───────────────────────┘
```

## 核心组件

### Rust 核心 (`src/`)

Rust 核心提供：

- **WebView 管理**: 窗口创建、生命周期和事件
- **IPC 系统**: Python 和 JavaScript 之间的双向通信
- **插件系统**: 高性能原生插件
- **自定义协议**: 安全的本地资源加载

```
src/
├── lib.rs                      # PyO3 模块入口
├── ipc/                        # IPC 系统
│   ├── mod.rs
│   ├── handler.rs              # IPC 消息处理器
│   ├── message_queue.rs        # 线程安全消息队列
│   └── ...
├── utils/                      # 工具函数
│   └── mod.rs
└── webview/                    # WebView 实现
    ├── mod.rs                  # 模块导出
    ├── aurora_view.rs          # Python 接口类 (PyO3)
    ├── config.rs               # 配置结构
    ├── backend/                # 后端实现
    │   ├── mod.rs              # 后端 trait 定义
    │   ├── native.rs           # 原生后端 (Windows HWND)
    │   └── qt.rs               # Qt 后端
    ├── event_loop.rs           # 事件循环处理
    ├── message_pump.rs         # Windows 消息泵
    ├── protocol.rs             # 自定义协议处理器
    ├── standalone.rs           # 独立窗口模式
    └── webview_inner.rs        # 核心 WebView 逻辑
```

### Python 绑定 (`python/auroraview/`)

通过 PyO3 提供的 Python 绑定：

- **WebView API**: 高级 Python 接口
- **事件系统**: Node.js 风格的 EventEmitter
- **Qt 集成**: 用于 DCC 应用的 QtWebView
- **类型安全**: 完整的类型提示和运行时验证

## 后端系统

### 后端 Trait

`WebViewBackend` trait 定义了通用接口：

```rust
pub trait WebViewBackend {
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn webview(&self) -> Arc<Mutex<WryWebView>>;
    fn message_queue(&self) -> Arc<MessageQueue>;
    fn window(&self) -> Option<&tao::window::Window>;
    fn process_events(&self) -> bool;
    fn run_event_loop_blocking(&mut self);
    
    // 通用操作的默认实现
    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn emit(&mut self, event_name: &str, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>>;
}
```

### 原生后端

`NativeBackend` 使用平台特定 API 进行窗口嵌入：

| 平台 | 技术 | 状态 |
|------|------|------|
| **Windows** | WebView2 (HWND) | ✅ 支持 |
| **macOS** | WKWebView (NSView) | ✅ 支持 |
| **Linux** | WebKitGTK | ✅ 支持 |

**Windows 模式**:
- `Child`: WS_CHILD 样式（需要同线程父窗口）
- `Owner`: GWLP_HWNDPARENT（跨线程安全）

## 集成模式

AuroraView 提供三种集成模式适用于不同场景：

### 1. 桌面模式

创建独立窗口，拥有自己的事件循环。最适合独立应用。

```python
from auroraview import WebView

webview = WebView(title="My App", width=800, height=600)
webview.show()  # 阻塞调用，拥有事件循环
```

**使用场景**:
- 独立工具
- 桌面应用
- 测试和开发

### 2. 原生模式 (HWND)

通过 HWND 将 WebView 嵌入非 Qt 应用。完全支持特效，不依赖 Qt。

```python
from auroraview import WebView

# 从非 Qt 应用获取父窗口 HWND（Blender、Unreal 等）
parent_hwnd = get_app_window_handle()

webview = WebView.create(
    title="My Tool",
    width=650,
    height=500,
    parent=parent_hwnd,
    mode="owner",
)
webview.load_html("<h1>Hello from Native Mode!</h1>")
webview.show()
```

**使用场景**:
- Blender 集成（非 Qt）
- Unreal Engine 集成
- 其他非 Qt DCC 应用
- 任何应用中的浮动工具窗口

**主要特性**:
- ✅ 完全支持窗口特效（点击穿透、模糊、Mica）
- ✅ 非阻塞 - 宿主应用保持响应
- ✅ 不依赖 Qt

### 3. Qt 模式

将 WebView 作为 Qt widget 子窗口嵌入。最适合需要停靠功能的 Qt DCC 应用。

```python
from auroraview import QtWebView
import hou  # 或 maya.OpenMayaUI 等

# 获取 DCC 主窗口
main_window = hou.qt.mainWindow()

# 创建嵌入式 WebView
qt_webview = QtWebView(
    parent=main_window,
    width=650,
    height=500,
)
qt_webview.load_html("<h1>Hello from Qt Mode!</h1>")
qt_webview.show()
```

**使用场景**:
- Maya、Houdini、Nuke、3ds Max 集成
- 可停靠面板
- 基于 Qt 的 DCC 应用

**主要特性**:
- ✅ 无缝 Qt 集成
- ✅ QDockWidget 支持
- ✅ 使用 DCC 的 Qt 消息泵
- ⚠️ 窗口特效支持有限

### 3. 打包模式

打包为独立可执行文件时：

```
app.exe (Rust)
    ├── 解压资源和 Python 运行时
    ├── 创建 WebView
    ├── 加载前端（从 overlay）
    ├── 启动 Python 后端进程
    │       └── 作为 API 服务器运行 (JSON-RPC over stdin/stdout)
    └── 事件循环 (Rust 主线程)
```

与开发模式的主要区别：
- Rust 是主进程（而非 Python）
- Python 作为子进程运行
- 通过 stdin/stdout 的 JSON-RPC 通信
- 更好的进程隔离和错误处理

## 插件架构

AuroraView 包含内置 Rust 插件：

| 插件 | 描述 |
|------|------|
| **Process** | 运行外部进程并流式输出 |
| **File System** | 原生文件操作 |
| **Dialog** | 原生文件/文件夹对话框 |
| **Shell** | 执行命令、打开 URL |
| **Clipboard** | 系统剪贴板访问 |

## 线程安全

### 原生后端

- WebView 和 EventLoop 在 Windows 上不是 `Send`
- 设计为单线程使用（UI 线程）
- 消息队列提供线程安全通信

### DCC 集成模式

- WebView 在 DCC 主 UI 线程创建
- 无独立事件循环（无线程问题）
- 内部 EventTimer 处理消息
- 线程安全消息队列用于跨线程通信

## 性能特性

### 内存占用

| 组件 | 内存使用 |
|------|----------|
| Rust 核心 | ~5 MB |
| 系统 WebView | ~20-30 MB |
| **总计** | **~30 MB** |

对比：
- Electron: ~150 MB
- Qt WebEngine: ~100 MB

### 启动时间

- 冷启动: ~300ms
- 热启动: ~100ms
