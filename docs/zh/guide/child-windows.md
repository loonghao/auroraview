# 子窗口系统

AuroraView 提供了统一的子窗口系统，允许示例和应用程序以独立模式或作为父应用程序（如 Gallery）的子窗口运行。

## 概述

子窗口系统支持：

- **双模式执行**：示例可以独立运行或作为子窗口运行
- **自动模式检测**：通过环境变量实现
- **父子通信**：窗口之间完整的 IPC 支持
- **无缝集成**：基本用法无需修改代码

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         Gallery (父窗口)                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐     ┌─────────────────┐                    │
│  │ ChildWindowManager │◄──►│   IPC Server    │                    │
│  └────────┬────────┘     └────────┬────────┘                    │
│           │                       │                              │
│           │  launch_example()     │  TCP Socket                  │
│           ▼                       ▼                              │
├───────────┴───────────────────────┴─────────────────────────────┤
│                         环境变量                                  │
│  AURORAVIEW_PARENT_ID, AURORAVIEW_PARENT_PORT 等                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   示例 1    │  │   示例 2    │  │   示例 3    │              │
│  │  (子窗口)   │  │  (子窗口)   │  │  (子窗口)   │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                   ┌──────▼──────┐                                │
│                   │ ParentBridge │                                │
│                   │  (IPC 客户端)│                                │
│                   └─────────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

## 快速开始

### 使用 ChildContext 的基本用法

创建子窗口感知应用的最简单方式：

```python
from auroraview import ChildContext

with ChildContext() as ctx:
    webview = ctx.create_webview(
        title="我的示例",
        html="<h1>Hello World</h1>",
        width=800,
        height=600
    )
    
    # 检查是否作为子窗口运行
    if ctx.is_child:
        print(f"作为子窗口运行，父窗口: {ctx.parent_id}")
        # 向父窗口发送消息
        ctx.emit_to_parent("hello", {"message": "来自子窗口的问候！"})
    else:
        print("独立运行")
    
    webview.show()
```

### 模式检测函数

```python
from auroraview import is_child_mode, get_parent_id, get_child_id

# 检查是否作为子窗口运行
if is_child_mode():
    print(f"父窗口 ID: {get_parent_id()}")
    print(f"子窗口 ID: {get_child_id()}")
else:
    print("独立运行")
```

## 环境变量

作为子窗口启动时，会使用以下环境变量：

| 变量 | 必需 | 描述 |
|------|------|------|
| `AURORAVIEW_PARENT_ID` | 是 | 父窗口标识符。**存在此变量即代表进入子窗口模式。** |
| `AURORAVIEW_PARENT_PORT` | 是 | 父进程监听的 TCP 端口 |
| `AURORAVIEW_CHILD_ID` | 否 | 子窗口唯一 ID，子进程发送的每一帧都会带上 |
| `AURORAVIEW_EXAMPLE_NAME` | 否 | 正在运行的示例名称 |
| `AURORAVIEW_PARENT_HWND` | 否 | 父窗口原生句柄（Windows `HWND`，十进制或 `0x` 前缀十六进制）。无需传 `--parent-hwnd` 即可嵌入。 |
| `AURORAVIEW_CHILD_EXIT_ON_DISCONNECT` | 否 | `parent_ipc_child` 演示用：父进程断开时退出 |

Python 子进程（`auroraview.child`）与 Rust 子进程（`auroraview_core::parent_ipc`，
`auroraview run` 使用）读取的是同一套环境变量，宿主可以用完全相同的方式启动二者。

## 线路协议规范

本节是父子通道的**规范性、语言无关**定义。目前有三份实现：
`auroraview.child.ParentBridge`（Python）、`auroraview_core::parent_ipc`（Rust）
与 `examples/parent_ipc/parent_host.ps1`（PowerShell）。任何宿主——C#、C++、
PowerShell、Go——只要有一个 TCP socket 和 JSON 解析器就能实现它。

### 传输层

| 属性 | 取值 |
|------|------|
| 传输 | 回环 TCP |
| 地址 | `127.0.0.1:<AURORAVIEW_PARENT_PORT>` |
| 角色 | 父进程 **监听/accept**；子进程 **connect** |
| 分帧 | 换行分隔 JSON（NDJSON），每行一个对象 |
| 编码 | UTF-8，**不带 BOM** |
| 单帧上限 | 1 MiB |

### 分帧规则

- 每一帧是一个 JSON 对象，以单个 `\n`（`0x0A`）结尾。
- JSON 字符串中不可能出现裸 `0x0A`（会被转义为 `\n`），因此 `0x0A` 是无歧义的分隔符。
- **发送方不得**写 UTF-8 BOM；**接收方必须容忍**开头的 BOM：一些常用库
  （基于 `Encoding.UTF8` 的 PowerShell / .NET `StreamWriter`）会在第一帧前写入 BOM，
  直接拒绝会让通道在第一帧就失效。
- 接收方**应当**容忍 `\r\n`（去掉结尾的 `\r`）。
- 空行**必须**被跳过，不能当作流结束。
- 一帧可能被拆分到多个 TCP 段，多个帧也可能出现在同一个段里；接收方**必须**缓冲并按 `0x0A` 切分。

### 帧结构

```jsonc
{
  "type": "event",        // 缺省时视为 "event"
  "event": "child:ready", // 仅 event 帧
  "data": { },            // 载荷，任意 JSON 值
  "child_id": "child-1",  // 仅 child -> parent
  "protocol": 1,          // 仅 hello / hello_ack
  "accepted": true,       // 仅 hello_ack
  "parent_id": "gallery", // 仅 hello / hello_ack
  "example_name": "demo", // 仅 hello
  "code": "bad_json",     // 仅 error
  "message": "...",       // 仅 error
  "fatal": false          // 仅 error
}
```

`type` 在**接收时**可选，缺省为 `event`，因此既有的 Gallery 帧格式
（`{"event": ..., "data": ...}`）依然有效。

### 帧类型

| `type` | 方向 | 用途 |
|--------|------|------|
| `hello` | child -> parent | 宣告子进程及协议版本，开启握手 |
| `hello_ack` | parent -> child | 接受（`"accepted": true`）或拒绝子进程 |
| `event` | 双向 | 应用事件（`event` + `data`） |
| `error` | 双向 | 错误通知；`"fatal": true` 表示发送方即将关闭 |
| `ping` / `pong` | 双向 | 存活探测 |

未知的 `type` **必须**被忽略而不是当作错误，这样新版本对端才不会破坏旧版本。

### 握手

```text
parent                                     child
  |  bind 127.0.0.1:<port>                   |
  |  带 AURORAVIEW_* 环境变量启动子进程        |
  |<-----------------------------------------|  connect
  |<-----------------------------------------|  {"type":"hello","protocol":1,...}
  |<-----------------------------------------|  {"type":"event","event":"child:ready",...}
  |  {"type":"hello_ack","accepted":true} -> |
  |<-----------------------------------------|  {"type":"event","event":"child:hello",...}
```

1. 子进程连接并发送 `hello`。
2. 子进程**立即**发送 `child:ready`，不等待 ack。
3. 若父进程实现了该协议，则回复 `hello_ack`。
4. 生效协议版本为 `min(child.protocol, parent.protocol)`。

握手是**可选且向后兼容的**：

| 父进程行为 | 结果状态 | 含义 |
|------------|----------|------|
| 回复 `hello_ack` 且 `accepted: true` | `Acked` | 支持协议的父进程 |
| 不回复（如当前的 Gallery） | `Legacy` | 完全可用，只是没有协商 |
| 回复 `accepted: false` | `Rejected` | 子进程应回退为独立模式 |

子进程**不得**在启动时阻塞等待 `hello_ack`：不支持协议的父进程永远不会发送它，
等待会把这段超时加到每个子进程的首次绘制时间上。

### 保留事件

| 事件 | 方向 | 载荷 |
|------|------|------|
| `child:ready` | child -> parent | `{ child_id, example_name }` |
| `child:closing` | child -> parent | `{ child_id }` |
| `parent:command` | parent -> child | `{ command, args }` |

`parent:command` 支持：

| `command` | `args` | 效果 |
|-----------|--------|------|
| `close` | – | 关闭窗口并退出 |
| `eval` | `{ js }` | 在页面中执行 JavaScript |
| `emit` | `{ event, data }` | 通过 `window.auroraview.trigger()` 向页面投递事件 |

### 错误码

| `code` | 含义 | 致命 |
|--------|------|------|
| `unsupported_protocol` | 对端使用了本端无法服务的协议版本 | 是 |
| `bad_json` | 帧不是合法 JSON，或不是 JSON 对象 | 否 |
| `frame_too_large` | 帧超过 1 MiB 上限 | 否 |
| `unknown_type` | 无法识别的 `type` | 否 |
| `handler_error` | 事件处理器失败 | 否 |
| `internal_error` | 其他 | 否 |

非致命错误不会关闭通道：接收方丢弃出错的那一帧、上报它，然后继续读取。

### 断连语义

- 子进程优雅退出时先发送 `child:closing`，再关闭 socket。
- 任何一方断开 socket 都是合法的“我结束了”信号，而不是错误。
- 子进程通过 disconnect 回调上报断连；是否退出是**宿主的决定**（见 `--exit-on-parent-disconnect`）。
- 重连是可选的。开启后子进程会重试有限次数，重新握手并再次宣告 `child:ready`。

### 参考实现

| 角色 | 语言 | 路径 |
|------|------|------|
| 子进程 | Python | `python/auroraview/child.py` |
| 子进程 | Rust | `crates/auroraview-core/src/parent_ipc/` |
| 子进程（无界面演示） | Rust | `crates/auroraview-core/examples/parent_ipc_child.rs` |
| 父进程 | PowerShell | `examples/parent_ipc/parent_host.ps1` |
| 父进程 | Python | `gallery/backend/child_manager.py` |

PowerShell 父进程是跨语言证明：它手工拼 JSON、完成握手、往返事件并验证断连，
全程不涉及 Python。

```powershell
# 构建 Rust 子进程，然后跑完整对话
cargo build -p auroraview-core --example parent_ipc_child
./examples/parent_ipc/parent_host.ps1

# 同上，但改为断开 socket 而不是发送 `close`
./examples/parent_ipc/parent_host.ps1 -Mode Disconnect
```

## 嵌入宿主窗口

除了 IPC，宿主还可以把 AuroraView 窗口挂到自己的窗口上。这正是 Unity、Unreal、
Qt 与 PowerPoint 等宿主做进程外嵌入所需要的。

```bash
# 子窗口：WS_CHILD，裁剪到父窗口客户区
auroraview run --url https://example.com --parent-hwnd 0x001A0B3C

# 被拥有的窗口：独立顶层窗口，始终位于所有者之上并随之销毁
auroraview run --url https://example.com --owner-hwnd 0x001A0B3C

# 也可以让环境变量传递句柄
AURORAVIEW_PARENT_HWND=0x001A0B3C auroraview run --url https://example.com
```

| 参数 | 风格 | 适用场景 |
|------|------|----------|
| `--parent-hwnd <HWND>` | `WS_CHILD` | 嵌入 DCC 视口内部的面板 |
| `--owner-hwnd <HWND>` | 被拥有的顶层窗口 | 浮动工具窗口 |

句柄接受十进制或 `0x` 前缀的十六进制。两个参数互斥——Win32 窗口要么是子窗口，
要么是被拥有的顶层窗口——`--parent-hwnd` 优先，因为子窗口本就会随父窗口销毁。

两个参数在所有平台上都可接受，以便宿主脚本保持可移植；但挂到外部窗口只在
Windows 上有效，其他平台会记录日志并忽略。

## API 参考


### ChildContext

用于创建子窗口感知 WebView 的上下文管理器。

```python
class ChildContext:
    def __init__(self):
        """初始化子窗口上下文，自动检测模式。"""
        
    @property
    def is_child(self) -> bool:
        """检查是否在子窗口模式下运行。"""
        
    @property
    def parent_id(self) -> Optional[str]:
        """获取父窗口 ID（仅在子窗口模式下）。"""
        
    @property
    def child_id(self) -> Optional[str]:
        """获取当前窗口的子窗口 ID（仅在子窗口模式下）。"""
        
    def create_webview(self, **kwargs) -> WebView:
        """根据当前模式创建适当配置的 WebView。"""
        
    def emit_to_parent(self, event: str, data: Any) -> bool:
        """向父窗口发送事件（仅在子窗口模式下有效）。"""
        
    def on_parent_message(self, handler: Callable[[str, Any], None]):
        """注册父窗口消息处理器。"""
```

### ChildInfo

子窗口信息。

```python
@dataclass
class ChildInfo:
    child_id: str          # 唯一子窗口标识符
    example_name: str      # 示例名称
    process_id: int        # 操作系统进程 ID
    port: int              # IPC 端口
    started_at: float      # 启动时间戳
```

### 辅助函数

```python
def is_child_mode() -> bool:
    """检查是否作为子窗口运行。"""
    
def get_parent_id() -> Optional[str]:
    """获取父窗口 ID，独立运行时返回 None。"""
    
def get_child_id() -> Optional[str]:
    """获取当前窗口的子窗口 ID，独立运行时返回 None。"""
    
def run_example(example_path: str, **kwargs) -> Optional[str]:
    """将示例作为子窗口启动，返回 child_id。"""
```

## 父子通信

### 从子窗口到父窗口

```python
# 在子窗口中
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    # 向父窗口发送事件
    ctx.emit_to_parent("status_update", {
        "progress": 50,
        "message": "处理中..."
    })
```

### 从父窗口到子窗口

```python
# 在父窗口中（如 Gallery）
from gallery.backend.child_manager import get_manager

manager = get_manager()

# 向特定子窗口发送消息
manager.send_to_child(child_id, "parent:command", {
    "action": "refresh"
})

# 广播到所有子窗口
manager.broadcast("parent:notification", {
    "message": "设置已更改"
})
```

### 处理消息

```python
# 在子窗口中
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    @ctx.on_parent_message
    def handle_parent_message(event: str, data: dict):
        if event == "parent:command":
            if data.get("action") == "refresh":
                # 处理刷新命令
                pass
```

## Gallery 集成

### JavaScript API

从 Gallery 运行示例时，使用以下 API：

```javascript
// 将示例作为子窗口启动
const childId = await auroraview.api.launch_example_as_child("child_window_demo");

// 获取所有活动的子窗口
const children = await auroraview.api.get_children();
// 返回: [{ child_id, example_name, process_id, port, started_at }, ...]

// 向子窗口发送消息
await auroraview.api.send_to_child(childId, "parent:message", { data: "hello" });

// 广播到所有子窗口
await auroraview.api.broadcast_to_children("parent:notification", { message: "大家好" });

// 关闭特定子窗口
await auroraview.api.close_child(childId);

// 关闭所有子窗口
await auroraview.api.close_all_children();
```

### 监听子窗口事件

```javascript
// 在 Gallery 前端
auroraview.on('child:connected', (data) => {
    console.log('子窗口已连接:', data.child_id, data.example_name);
});

auroraview.on('child:disconnected', (data) => {
    console.log('子窗口已断开:', data.child_id);
});

auroraview.on('child:message', (data) => {
    console.log('来自子窗口的消息:', data.child_id, data.event, data.data);
});
```

## 完整示例

以下是一个在独立模式和子窗口模式下都能工作的完整示例：

```python
"""子窗口感知示例，根据执行上下文自动适应。"""
from auroraview import ChildContext

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>子窗口演示</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .mode { padding: 10px; border-radius: 5px; margin-bottom: 20px; }
        .standalone { background: #e3f2fd; }
        .child { background: #e8f5e9; }
        button { padding: 10px 20px; margin: 5px; cursor: pointer; }
    </style>
</head>
<body>
    <div id="mode" class="mode"></div>
    <div id="messages"></div>
    <button onclick="sendToParent()">发送到父窗口</button>
    
    <script>
        const isChild = window.AURORAVIEW_IS_CHILD || false;
        const modeDiv = document.getElementById('mode');
        
        if (isChild) {
            modeDiv.className = 'mode child';
            modeDiv.innerHTML = '<h2>作为子窗口运行</h2>';
        } else {
            modeDiv.className = 'mode standalone';
            modeDiv.innerHTML = '<h2>独立运行</h2>';
        }
        
        function sendToParent() {
            if (isChild && window.auroraview) {
                auroraview.api.notify_parent({
                    event: 'button_clicked',
                    data: { timestamp: Date.now() }
                });
            }
        }
        
        // 监听父窗口消息
        if (window.auroraview) {
            auroraview.on('parent:message', (data) => {
                const div = document.getElementById('messages');
                div.innerHTML += `<p>来自父窗口: ${JSON.stringify(data)}</p>`;
            });
        }
    </script>
</body>
</html>
"""

def main():
    with ChildContext() as ctx:
        webview = ctx.create_webview(
            title="子窗口演示",
            html=HTML,
            width=600,
            height=400
        )
        
        # 注入模式信息
        webview.eval_js(f"window.AURORAVIEW_IS_CHILD = {str(ctx.is_child).lower()};")
        
        # 处理来自父窗口的消息
        if ctx.is_child:
            @ctx.on_parent_message
            def on_parent_msg(event, data):
                webview.emit(event, data)
        
        # 子窗口通知父窗口的 API
        @webview.bind_call("api.notify_parent")
        def notify_parent(event: str, data: dict):
            if ctx.is_child:
                ctx.emit_to_parent(event, data)
                return {"ok": True}
            return {"ok": False, "reason": "不在子窗口模式"}
        
        webview.show()

if __name__ == "__main__":
    main()
```

## 最佳实践

### 1. 始终使用 ChildContext

```python
# 推荐：使用上下文管理器
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    webview.show()

# 避免：手动模式检测
if os.environ.get("AURORAVIEW_PARENT_ID"):
    # 手动设置...
```

### 2. 优雅降级

设计应用在两种模式下都能工作：

```python
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    # 仅在子窗口模式下可用的功能
    if ctx.is_child:
        ctx.emit_to_parent("ready", {"version": "1.0"})
    
    # 核心功能在两种模式下都能工作
    @webview.bind_call("api.process")
    def process(data):
        return do_processing(data)
    
    webview.show()
```

### 3. 清理关闭

```python
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    @webview.on_close
    def on_close():
        if ctx.is_child:
            ctx.emit_to_parent("closing", {"child_id": ctx.child_id})
    
    webview.show()
```

## 与 Rust 子窗口的对比

| 特性 | Python 子窗口系统 | Rust `child_window.rs` |
|------|-------------------|------------------------|
| 用途 | Python 示例作为子窗口 | JS `window.open()` 处理 |
| 通信 | 完整 IPC | 无 |
| 配置 | 完整 WebView 选项 | 仅 URL/尺寸 |
| API 绑定 | 支持 | 不支持 |
| 模式检测 | 自动 | 不适用 |

这两个系统是**互补**的，而不是相互替代的。
