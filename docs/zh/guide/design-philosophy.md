# 设计思路

AuroraView 基于特定的设计原则构建，这些原则指导着其架构和 API 设计决策。理解这些原则将帮助你更好地使用这个框架。

## 核心原则

### 1. DCC 优先设计

与通用 WebView 框架不同，AuroraView 专为数字内容创作软件集成而设计。

**这意味着：**
- 为 Maya、Houdini、Nuke、3ds Max 提供 Qt widget 集成
- 为 Unreal Engine 和非 Qt 应用提供基于 HWND 的嵌入
- 尊重 DCC 应用模式的生命周期管理
- 不会冻结宿主应用的非阻塞操作

**受此影响的设计决策：**
- `QtWebView` 创建真正的 Qt widget，而不是原生窗口的包装
- 事件处理与 Qt 事件循环集成，而不是单独的线程
- 父窗口监控以实现自动清理

### 2. 零 Python 依赖

核心 `auroraview` 包除标准库外没有 Python 依赖。

**原因：**
- DCC 应用通常有受限的 Python 环境
- 避免与 DCC 内置包的版本冲突
- 最小化安装复杂性
- 单一 `.pyd` 文件分发

**例外：** `[qt]` 扩展安装 `QtPy` 用于 Qt 版本抽象。

### 3. Rust 实现性能和安全

核心使用 Rust 编写，通过 PyO3 绑定。

**优势：**
- 无垃圾回收暂停的内存安全
- ~5MB 包大小 vs Electron 的 ~120MB
- IPC 和事件处理的原生性能
- 线程安全的消息传递

### 4. 后端抽象

AuroraView 使用后端抽象层，允许不同的实现：

```
┌─────────────────────────────────────────┐
│           Python API 层                 │
│  (WebView, QtWebView, AuroraView)       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           后端抽象层                    │
│         (WebViewBackend trait)          │
└─────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌───────────────┐       ┌───────────────┐
│ 原生后端      │       │  Qt 后端      │
│ (Wry/WebView2)│       │ (Qt Widget)   │
└───────────────┘       └───────────────┘
```

**这使得：**
- 平台特定优化
- 未来添加新后端（CEF、WebKitGTK 等）
- 跨后端的一致 API

### 5. 约定优于配置

AuroraView 提供合理的默认值，同时允许自定义。

**示例：**

```python
# 最小配置 - 开箱即用
webview = WebView.create("我的应用", url="http://localhost:3000")
webview.show()

# 需要时完全自定义
webview = WebView.create(
    title="我的应用",
    url="http://localhost:3000",
    width=1024,
    height=768,
    resizable=True,
    frame=True,
    debug=True,
    context_menu=False,
    asset_root="./assets",
)
```

## API 设计模式

### 1. 多种 API 风格

AuroraView 支持不同用例的不同 API 风格：

| 模式 | 适用场景 | 复杂度 |
|------|----------|--------|
| `AuroraView` + `api=` | 快速原型 | 简单 |
| `QtWebView` 子类 | DCC 集成 | 中等 |
| `WebView` + `bind_call` | 高级控制 | 高级 |

**示例：简单 API 对象**
```python
class MyAPI:
    def get_data(self) -> dict:
        return {"items": [1, 2, 3]}

view = AuroraView(url="...", api=MyAPI())
```

**示例：Qt 风格类**
```python
class MyTool(QtWebView):
    selection_changed = Signal(list)
    
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.bind_api(self)
```

### 2. 显式优于隐式

API 方法清晰地表明其行为：

```python
# 阻塞和非阻塞的清晰区分
webview.show()           # 独立模式下阻塞
webview.show(wait=False) # 非阻塞

# 事件类型的清晰区分
auroraview.send_event()  # 即发即忘（JS → Python）
auroraview.call()        # 请求-响应（JS → Python）
webview.emit()           # 推送通知（Python → JS）
```

### 3. Qt 风格信号系统

对于熟悉 Qt 的开发者：

```python
from auroraview import Signal

class MyTool(WebView):
    # 信号定义
    selection_changed = Signal(list)
    progress_updated = Signal(int, str)
    
    def _on_selection(self, items):
        self.selection_changed.emit(items)
```

## JavaScript API 设计

### 统一命名空间

所有 JavaScript API 都在 `window.auroraview` 下：

```javascript
// RPC 调用
await auroraview.call('api.method', params);

// 事件
auroraview.send_event('event_name', data);
auroraview.on('event_name', handler);

// API 代理（pywebview 风格）
await auroraview.api.method(params);

// 共享状态
auroraview.state.key = value;
```

### 协议设计

IPC 协议遵循请求/响应模式：

**请求（JS → Python）：**
```json
{
  "type": "call",
  "id": "unique-id",
  "method": "api.get_data",
  "params": {"key": "value"}
}
```

**响应（Python → JS）：**
```json
{
  "type": "call_result",
  "id": "unique-id",
  "ok": true,
  "result": {"data": "..."}
}
```

## 安全考虑

### 自定义协议安全

`auroraview://` 协议使用 `.localhost` TLD 以确保安全：

1. **IANA 保留** - 任何人都无法注册
2. **仅限本地** - 被视为 127.0.0.1
3. **DNS 前拦截** - 请求在 DNS 之前被拦截
4. **无网络流量** - 永远不会离开本地机器

### 资源访问控制

```python
# 安全：只有 assets/ 目录可访问
webview = WebView.create(
    title="我的应用",
    asset_root="./assets",  # 受限访问
)

# 不太安全：完整文件系统访问
webview = WebView.create(
    title="我的应用",
    allow_file_protocol=True,  # 谨慎使用
)
```

## 性能理念

### 延迟初始化

WebView 仅在调用 `show()` 时创建：

```python
webview = WebView.create("我的应用")  # 尚未创建 WebView
webview.load_url("...")               # URL 已存储，未加载
webview.show()                        # WebView 创建并加载 URL
```

### 消息批处理

IPC 消息批量处理：

```rust
let messages = message_queue.drain();
for message in messages {
    // 处理每条消息
}
```

### 无锁数据结构

- **DashMap**：用于回调的并发 HashMap
- **crossbeam-channel**：用于消息队列的无锁 MPMC

## 可扩展性

### 插件架构

内置 Rust 插件提供原生性能：

| 插件 | 描述 |
|------|------|
| Process | 运行外部进程，支持流式输出 |
| File System | 原生文件操作 |
| Dialog | 原生文件/文件夹对话框 |
| Shell | 执行命令、打开 URL |
| Clipboard | 系统剪贴板访问 |

### 自定义协议

注册自定义协议处理器：

```python
def handle_maya_protocol(uri: str) -> dict:
    path = uri.replace("maya://", "")
    return {
        "data": load_maya_resource(path),
        "mime_type": "application/octet-stream",
        "status": 200
    }

webview.register_protocol("maya", handle_maya_protocol)
```

## 迁移路径

### 从 pywebview 迁移

AuroraView 提供 pywebview 兼容的 API：

```python
# pywebview 风格
class Api:
    def get_data(self):
        return {"items": [1, 2, 3]}

# 与 AuroraView 兼容
view = AuroraView(url="...", api=Api())
```

```javascript
// 相同的 JavaScript API
const data = await auroraview.api.get_data();
```

### 从 Electron 迁移

主要区别：
- 单进程（无独立的 main/renderer）
- Python 后端而非 Node.js
- 原生系统 WebView 而非捆绑的 Chromium
- ~5MB vs ~120MB 包大小
