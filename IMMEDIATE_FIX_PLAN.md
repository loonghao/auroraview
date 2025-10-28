# 🚀 立即修复计划

## 问题总结

1. **点击按钮时 Maya 卡住** - JavaScript 回调在 WebView 事件循环中执行
2. **关闭 WebView 时 Maya 退出** - daemon 线程导致进程异常终止

## ✅ 已完成的修复

### 修复 1：改变线程类型（已完成）

```python
# python/auroraview/webview.py 第 153 行
# 改为：
self._show_thread = threading.Thread(target=_run_webview, daemon=False)
```

**效果：** 防止 Maya 在关闭 WebView 时退出

## 🔧 需要完成的修复

### 修复 2：实现非阻塞事件处理（关键）

**问题：** 当前 `core.show()` 是阻塞的，会一直运行事件循环，导致 JavaScript 回调在后台线程中执行。

**解决方案：** 需要在 Rust 中实现一个 **非阻塞的事件处理机制**。

#### 方案 A：使用 Python 的 `time.sleep()` 让出 CPU（快速修复）

```python
# python/auroraview/webview.py

def show_async(self) -> None:
    """Show WebView in background thread."""
    
    def _run_webview():
        try:
            from ._core import WebView as _CoreWebView
            core = _CoreWebView(...)
            core.load_html(self._stored_html)
            
            # 不要直接调用 core.show()
            # 而是使用非阻塞的方式
            core.show_non_blocking()  # 需要在 Rust 中实现
            
        finally:
            self._is_running = False
    
    self._show_thread = threading.Thread(target=_run_webview, daemon=False)
    self._show_thread.start()
```

#### 方案 B：在 Rust 中添加 `show_non_blocking()` 方法

```rust
// src/webview/mod.rs

#[pymethods]
impl AuroraView {
    /// Show WebView in non-blocking mode (for DCC integration)
    fn show_non_blocking(&self) -> PyResult<()> {
        // 创建 WebView 但不运行阻塞的事件循环
        // 而是返回一个可以定期调用的处理函数
        
        // 这需要修改 Rust 代码的架构
        // 使用 pump_events() 而不是 run()
        
        Ok(())
    }
}
```

## 📋 实现步骤

### 第一步：快速测试当前修复（立即）

```bash
# 1. 编译
maturin develop --release

# 2. 在 Maya 中测试
# 运行 examples/maya_event_queue_integration.py
# 观察：
# - 点击按钮时 Maya 是否仍然卡住？
# - 关闭 WebView 时 Maya 是否退出？
```

### 第二步：如果仍然卡住，实现真正的修复

需要修改 Rust 代码以支持非阻塞事件处理。

## 🎯 根本原因分析

### 为什么点击按钮会卡住？

```
1. WebView 在后台线程运行
2. core.show() 是阻塞的 - 一直运行事件循环
3. 用户点击按钮
4. JavaScript 事件触发
5. Python 回调被调用（仍在 WebView 事件循环中）
6. 回调尝试调用 Maya API
7. Maya API 需要在主线程中执行
8. 死锁 - 后台线程等待主线程，主线程被阻塞
```

### 解决方案的关键

**不能让 JavaScript 回调在 WebView 事件循环中执行。**

相反，应该：

```
1. WebView 事件循环处理 UI 事件
2. JavaScript 回调被放入队列
3. 后台线程定期检查队列
4. 回调被异步执行（不在事件循环中）
5. 回调可以安全地发送消息到 Maya 主线程
```

## 💡 关键洞察

**当前的架构问题：**

```
WebView 事件循环（后台线程）
    ↓
JavaScript 事件
    ↓
Python 回调（同步执行）← 问题！
    ↓
尝试调用 Maya API
    ↓
死锁
```

**正确的架构：**

```
WebView 事件循环（后台线程）
    ↓
JavaScript 事件
    ↓
事件队列
    ↓
后台线程定期处理队列
    ↓
Python 回调（异步执行）
    ↓
消息队列
    ↓
Maya 主线程处理消息
    ↓
调用 Maya API
```

## 📊 修复优先级

| 优先级 | 任务 | 状态 |
|------|------|------|
| 1 | 改变线程类型 daemon=False | ✅ 完成 |
| 2 | 测试当前修复 | ⏳ 待做 |
| 3 | 实现非阻塞事件处理 | ⏳ 待做 |
| 4 | 实现异步回调队列 | ⏳ 待做 |
| 5 | 集成消息队列 | ⏳ 待做 |

## 🧪 测试计划

### 测试 1：基本功能

```python
# 在 Maya 中运行
webview = WebView(title="Test", width=400, height=300)
webview.load_html("<button onclick='alert(\"clicked\")'>Click Me</button>")
webview.show_async()

# 观察：
# - WebView 是否出现？
# - 点击按钮时 Maya 是否卡住？
# - 关闭 WebView 时 Maya 是否退出？
```

### 测试 2：事件队列

```python
# 在 Maya 中运行
from auroraview.dcc_event_queue import DCCEventQueue

event_queue = DCCEventQueue()
event_queue.register_callback("test", lambda: print("Event processed"))

# 从后台线程发送事件
event_queue.post_event("test")

# 在主线程处理
event_queue.process_events()
```

## 📚 参考资源

- tao event loop: https://github.com/tauri-apps/tao
- wry WebView: https://github.com/tauri-apps/wry
- Python threading: https://docs.python.org/3/library/threading.html

---

**下一步：** 在 Maya 中测试当前修复，观察是否仍然卡住。

