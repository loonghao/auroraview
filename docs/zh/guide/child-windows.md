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

作为子窗口启动时，会设置以下环境变量：

| 变量 | 描述 |
|------|------|
| `AURORAVIEW_PARENT_ID` | 父窗口标识符 |
| `AURORAVIEW_PARENT_PORT` | IPC 通信端口 |
| `AURORAVIEW_CHILD_ID` | 唯一的子窗口 ID |
| `AURORAVIEW_EXAMPLE_NAME` | 正在运行的示例名称 |

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
