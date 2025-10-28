# 🔴 真实的线程问题分析和解决方案

## 问题 1：点击按钮时 Maya 卡住

### 根本原因

当前的实现中：

```
JavaScript 事件 (WebView 后台线程)
    ↓
Python 回调函数 (仍在 WebView 事件循环中)
    ↓
尝试调用 Maya API (需要在主线程)
    ↓
死锁 - Maya 卡住
```

**关键问题：** JavaScript 回调是在 WebView 的事件循环中执行的，而事件循环运行在后台线程中。

### 解决方案

需要在 Rust 层面实现 **非阻塞的事件处理**：

1. **WebView 事件循环不应该阻塞** - 应该定期处理事件，而不是一直运行
2. **JavaScript 回调应该异步处理** - 回调应该被放入队列，而不是立即执行
3. **使用消息队列** - 在后台线程和主线程之间传递消息

## 问题 2：关闭 WebView 时 Maya 退出

### 根本原因

```python
self._show_thread = threading.Thread(target=_run_webview, daemon=True)
```

当后台线程是 daemon 线程时：
- 如果后台线程异常退出，可能导致整个进程崩溃
- 关闭 WebView 时，daemon 线程立即终止，可能导致资源泄漏
- Python 进程可能在清理时出现问题

### 解决方案

```python
self._show_thread = threading.Thread(target=_run_webview, daemon=False)
```

使用非 daemon 线程：
- 线程正常退出时，进程继续运行
- 关闭 WebView 时，线程有时间进行清理
- 更稳定的资源管理

## 🎯 完整的修复方案

### 步骤 1：修改 Python 层 - 使用非 daemon 线程

```python
# python/auroraview/webview.py

# 改变这一行：
self._show_thread = threading.Thread(target=_run_webview, daemon=True)

# 为：
self._show_thread = threading.Thread(target=_run_webview, daemon=False)
```

### 步骤 2：修改 Rust 层 - 实现异步事件处理

**问题：** 当前的 `core.show()` 是阻塞的，会一直运行事件循环。

**解决方案：** 需要实现一个 **非阻塞的事件处理机制**。

#### 选项 A：使用消息队列（推荐）

在 Rust 中实现：

```rust
// src/webview/mod.rs

pub struct AuroraView {
    // ... existing fields ...
    event_queue: Arc<Mutex<VecDeque<Event>>>,
}

impl AuroraView {
    /// Process pending events without blocking
    pub fn process_events(&self) -> usize {
        let mut queue = self.event_queue.lock().unwrap();
        let count = queue.len();
        
        while let Some(event) = queue.pop_front() {
            // Handle event
            self.handle_event(event);
        }
        
        count
    }
}
```

#### 选项 B：使用非阻塞事件循环

在 Rust 中实现：

```rust
pub fn run_event_loop_non_blocking(&mut self) -> bool {
    // Process one event and return immediately
    // Return true if there are more events to process
    // Return false if the window is closed
}
```

### 步骤 3：修改 Python 层 - 集成事件处理

```python
# python/auroraview/webview.py

def show_async(self) -> None:
    """Show WebView in background thread with proper event handling."""
    
    def _run_webview():
        try:
            core = _CoreWebView(...)
            core.load_html(self._stored_html)
            
            # 使用非阻塞事件循环
            while core.is_running():
                # 处理一个事件
                core.process_events()
                # 让出 CPU 时间
                time.sleep(0.01)
        finally:
            self._is_running = False
    
    self._show_thread = threading.Thread(target=_run_webview, daemon=False)
    self._show_thread.start()
```

## 📊 对比：阻塞 vs 非阻塞

| 特性 | 阻塞事件循环 | 非阻塞事件循环 |
|------|-----------|------------|
| Maya 响应性 | ❌ 卡住 | ✓ 响应 |
| 按钮点击 | ❌ 卡住 | ✓ 正常 |
| 关闭 WebView | ❌ 可能崩溃 | ✓ 正常 |
| 资源清理 | ❌ 不完整 | ✓ 完整 |
| 线程管理 | ❌ daemon=True | ✓ daemon=False |

## 🔧 实现步骤

### 第一步：修改 Python 代码（立即可做）

```bash
# 修改 python/auroraview/webview.py 第 153 行
# daemon=True → daemon=False
```

### 第二步：修改 Rust 代码（需要编译）

1. 在 `src/webview/mod.rs` 中添加 `process_events()` 方法
2. 修改 `show()` 方法使用非阻塞事件循环
3. 编译：`maturin develop --release`

### 第三步：测试

```python
# examples/maya_non_blocking_test.py
webview = WebView(...)
webview.load_html(html)
webview.show_async()  # 现在应该不会卡住
```

## ⚠️ 关键注意事项

1. **不要使用 daemon=True** - 会导致 Maya 退出
2. **不要在后台线程调用 Maya API** - 使用消息队列
3. **不要阻塞事件循环** - 使用非阻塞处理
4. **正确关闭 WebView** - 调用 `close()` 方法

## 📚 参考

- Python threading: https://docs.python.org/3/library/threading.html
- Daemon threads: https://docs.python.org/3/library/threading.html#daemon-threads
- Event loops: https://en.wikipedia.org/wiki/Event_loop
- Non-blocking I/O: https://en.wikipedia.org/wiki/Non-blocking_I/O

---

**下一步：** 实现 Rust 层的非阻塞事件处理机制。

