# 双向通信

AuroraView 提供完整的 IPC（进程间通信）系统，实现 Python 和 JavaScript 之间的双向通信。

## 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                         JavaScript 层                            │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  事件桥接 (初始化脚本)                                      │ │
│  │  - 拦截 window.dispatchEvent()                             │ │
│  │  - 转发 CustomEvent 到 window.ipc.postMessage()            │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↓ ↑
                    window.ipc.postMessage()
                    window.dispatchEvent()
                              ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│                          Rust IPC 层                             │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  IpcHandler (src/ipc/handler.rs)                           │ │
│  │  - 接收来自 JavaScript 的消息                               │ │
│  │  - 通过 PyO3 调用 Python 回调                               │ │
│  │  - 线程安全的回调存储 (DashMap)                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  MessageQueue (src/ipc/message_queue.rs)                   │ │
│  │  - 队列化 Python 到 JavaScript 的消息                       │ │
│  │  - 无锁 MPMC 通道 (crossbeam-channel)                       │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↓ ↑
                         PyO3 绑定
                              ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│                         Python 层                                │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  WebView.on(event_name, callback)                          │ │
│  │  WebView.emit(event_name, data)                            │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 通信 API 概览

| 方向 | JavaScript API | Python API | 使用场景 |
|------|---------------|------------|----------|
| JS → Python | `auroraview.call(method, params)` | `@webview.bind_call` | 带返回值的 RPC |
| JS → Python | `auroraview.send_event(event, data)` | `@webview.on(event)` | 即发即忘事件 |
| Python → JS | - | `webview.emit(event, data)` | 推送通知 |
| 仅 JS | `auroraview.on(event, handler)` | - | 接收 Python 事件 |

## Python → JavaScript

### 发送事件

```python
# Python 端
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})
webview.emit("selection_changed", {"items": ["mesh1", "mesh2"]})
```

```javascript
// JavaScript 端
auroraview.on('update_data', (data) => {
    console.log('Frame:', data.frame);
    console.log('Objects:', data.objects);
});

auroraview.on('selection_changed', (data) => {
    highlightItems(data.items);
});
```

## JavaScript → Python

### 事件（即发即忘）

```javascript
// JavaScript 端
auroraview.send_event('export_scene', {
    path: '/path/to/export.fbx',
    format: 'fbx'
});
```

```python
# Python 端
@webview.on("export_scene")
def handle_export(data):
    print(f"导出到: {data['path']}")
    # 你的导出逻辑
```

### RPC 调用（带返回值）

```javascript
// JavaScript 端
const hierarchy = await auroraview.call('api.get_hierarchy', { root: 'scene' });
console.log('场景层级:', hierarchy);

const result = await auroraview.call('api.rename_object', {
    old_name: 'cube1',
    new_name: 'hero_cube'
});
```

```python
# Python 端
@webview.bind_call("api.get_hierarchy")
def get_hierarchy(root=None):
    return {"children": ["group1", "mesh_cube"], "count": 2}

@webview.bind_call("api.rename_object")
def rename_object(old_name, new_name):
    # 执行重命名
    return {"ok": True, "old": old_name, "new": new_name}
```

## API 对象模式

将 Python 方法暴露给 JavaScript 的最简单方式：

```python
from auroraview import AuroraView

class MyAPI:
    def get_data(self) -> dict:
        """从 JS 调用: await auroraview.api.get_data()"""
        return {"items": [1, 2, 3], "count": 3}

    def save_file(self, path: str = "", content: str = "") -> dict:
        """从 JS 调用: await auroraview.api.save_file({path: "...", content: "..."})"""
        with open(path, "w") as f:
            f.write(content)
        return {"ok": True, "path": path}

# 创建带 API 自动绑定的 WebView
view = AuroraView(url="http://localhost:3000", api=MyAPI())
view.show()
```

```javascript
// JavaScript 端
const data = await auroraview.api.get_data();
console.log(data.items);  // [1, 2, 3]

const result = await auroraview.api.save_file({
    path: "/tmp/test.txt",
    content: "Hello"
});
```

## 性能优化

### 无锁数据结构

- **DashMap**: 无锁并发 HashMap 用于回调存储
- **crossbeam-channel**: 无锁 MPMC 通道用于消息队列

### 批量处理

消息批量处理以减少开销：

```rust
let messages = message_queue.drain();  // 获取所有待处理消息
for message in messages {
    // 处理每条消息
}
```

### 线程安全

**Python GIL 管理**:
```rust
Python::with_gil(|py| {
    let callback = self.callbacks.get(&event_name)?;
    let args = PyTuple::new_bound(py, &[data_py]);
    callback.call1(py, args)?;
});
```

## 常见错误

::: danger 不要使用这些
```javascript
// 错误: trigger() 仅在 JS 本地，不会到达 Python
auroraview.trigger('my_event', data);

// 错误: dispatchEvent 是浏览器 API，不会到达 Python
window.dispatchEvent(new CustomEvent('my_event', {detail: data}));
```
:::

::: tip 正确用法
```javascript
// 正确: 使用 send_event() 进行即发即忘
auroraview.send_event('my_event', data);

// 正确: 使用 call() 进行请求-响应
const result = await auroraview.call('api.my_method', data);
```
:::

## 快速参考

### 注册事件处理器 (Python)

```python
@webview.on("event_name")
def handler(data):
    print(data)
```

### 发送事件 (Python)

```python
webview.emit("event_name", {"key": "value"})
```

### 发送事件 (JavaScript)

```javascript
auroraview.send_event('event_name', { key: 'value' });
```

### 监听事件 (JavaScript)

```javascript
auroraview.on('event_name', (data) => {
    console.log(data);
});
```

## 父子窗口通信

对于示例作为父应用程序（如 Gallery）子窗口运行的多窗口场景，AuroraView 提供了专用的 IPC 系统。

### 子窗口到父窗口

```python
from auroraview import ChildContext

with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    # 向父窗口发送事件
    if ctx.is_child:
        ctx.emit_to_parent("status_update", {
            "progress": 50,
            "message": "处理中..."
        })
```

### 父窗口到子窗口

```python
from gallery.backend.child_manager import get_manager

manager = get_manager()

# 发送到特定子窗口
manager.send_to_child(child_id, "parent:command", {"action": "refresh"})

# 广播到所有子窗口
manager.broadcast("parent:notification", {"message": "设置已更改"})
```

### 处理父窗口消息

```python
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    @ctx.on_parent_message
    def handle_parent_message(event: str, data: dict):
        if event == "parent:command":
            # 处理命令
            pass
```

[完整子窗口指南 →](/zh/guide/child-windows)
