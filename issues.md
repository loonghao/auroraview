# AuroraView 架构问题分析

## 核心问题：多线程 + 多模式 + 跨语言 = 复杂度爆炸

当前架构试图同时解决太多问题，导致：
1. **初始化时序耦合严重**
2. **事件驱动机制不统一**
3. **调试路径过长**

---

## 问题 1：MCP Server 与事件循环的时序耦合

### 现象
MCP 工具调用卡住，只有点击前端才会触发执行。

### 根因
```
时序问题：
1. Python: _start_mcp_server()     → 创建 MCP + Dispatcher
2. Python: show_blocking()          → 进入 Rust
3. Rust:   run_blocking()           → 设置 event_loop_proxy  ← 太晚了！
4. Rust:   事件循环开始
5. MCP请求到来 → Dispatcher.push() → wake_event_loop() 
   → 如果 proxy 未设置，消息排队但无法唤醒事件循环
```

### 设计缺陷
- **MCP Server 创建** 和 **事件循环启动** 是两个独立操作
- 它们之间没有同步机制，导致竞态条件
- Dispatcher 持有 MessageQueue 引用，但 proxy 延迟设置

---

## 问题 2：Embedded vs Standalone 双模式架构

### 现象
- Embedded 模式需要 Timer drain 队列
- Standalone 模式依赖 Rust 事件循环
- 两种模式的代码路径完全不同，但共享相同的底层机制

### 设计缺陷
```python
# webview.py 中的分支逻辑
if is_embedded:
    self._core.show()           # 非阻塞
    self._auto_timer.start()    # Timer drain
    self._start_mcp_server()    # MCP 在 timer 之后
else:
    self._start_mcp_server()    # MCP 在事件循环之前！
    self.show_blocking()        # 阻塞
```

- **不一致的 MCP 启动时机**：embedded 模式在 timer 后，standalone 在事件循环前
- **不同的事件驱动源**：Timer vs Rust event loop
- **EventTimer 的复杂 readiness gate**：需要判断 `_show_thread.is_alive()` 等状态

---

## 问题 3：MessageQueue + Dispatcher 的过度设计

### 当前设计
```
MCP Request (Tokio) 
    → MessageQueueDispatcher.dispatch_with_handler()
    → MessageQueue.push(McpToolCall{...})
    → wake_event_loop() 发送 UserEvent::ProcessMessages
    → 事件循环收到事件
    → MainEventsCleared 中处理消息
    → execute_mcp_tool() 在主线程执行 Python handler
    → oneshot channel 返回结果
```

### 设计缺陷
- **7 步调用链**：任何一步出错都会导致卡住
- **隐式依赖**：依赖 event_loop_proxy 被设置、事件循环在运行、锁没有死锁
- **调试困难**：需要在多个文件中添加日志才能定位问题

---

## 问题 4：跨语言边界过多

### 当前调用链
```
JavaScript (前端) 
    ↓ IPC
Python (auroraview.WebView) 
    ↓ PyO3
Rust (auroraview-core) 
    ↓ MessageQueue
Rust (event_loop) 
    ↓ PyO3
Python (tool handler)
    ↓ 
返回结果
```

### 设计缺陷
- **5 次语言边界跨越**
- **每次跨越都有潜在的序列化/反序列化开销**
- **GIL 获取/释放的时机难以预测**
- **错误传播路径过长**

---

## 问题 5：ControlFlow::Poll 的 Windows 兼容性问题

### 现象
`ControlFlow::Poll` 在 Windows 上可能不会持续触发 `MainEventsCleared`。

### 根因
- tao/wry 的 Windows 实现依赖 Win32 消息泵
- 没有 Windows 消息时，`PeekMessage` 返回空，事件循环可能空转
- `MainEventsCleared` 的触发依赖于有消息被处理

---

## 建议的架构改进

### 方案 A：简化 MCP 执行模式

**直接在 Tokio 线程执行工具**（对于不需要 UI 操作的工具）：
```rust
// 默认 direct_execution=true
// 只有明确标记需要主线程的工具才走 Dispatcher
tool.with_handler(|args| {
    // 直接在 Tokio 执行，无需 MessageQueue
})
```

### 方案 B：统一事件驱动机制

**无论 Embedded 还是 Standalone，都使用 Timer 驱动**：
```python
def show(self):
    self._core.show()  # 总是非阻塞
    self._start_timer()  # 总是启动 timer
    self._start_mcp_server()  # 在 timer 之后
    if wait:
        self._wait_for_close()  # 阻塞等待关闭
```

### 方案 C：延迟 MCP 启动

**在事件循环启动后再创建 MCP Server**：
```rust
// 在 run_blocking() 中
let proxy = event_loop.create_proxy();
message_queue.set_event_loop_proxy(proxy);

// 发送事件通知 Python 可以启动 MCP 了
send_event("event_loop_ready");
```

### 方案 D：MCP Sidecar 进程（推荐）

**将 MCP Server 完全独立为子进程**：
```
┌─────────────────┐     IPC      ┌─────────────────┐
│   Main Process  │◄────────────►│  MCP Sidecar    │
│  (WebView + UI) │   (JSON-RPC) │  (独立进程)     │
└─────────────────┘              └─────────────────┘
```

优点：
- **完全解耦**：MCP 生命周期独立于 WebView
- **无 GIL 问题**：独立进程有自己的 Python 解释器
- **易于调试**：可以单独启动/停止 MCP 进程
- **更好的隔离**：MCP 崩溃不影响主进程

---

## 短期修复建议（不改架构）

### 修复 1：在事件循环启动后设置 Dispatcher

```rust
// src/webview/core/main.rs
fn show(&self) {
    // 1. 启动事件循环
    run_blocking(...);

    // 问题：show() 是阻塞的，无法在之后执行代码
}
```

需要改为：
```rust
// 在 run_blocking 内部，事件循环启动后发送就绪事件
Event::NewEvents(StartCause::Init) => {
    // 事件循环已启动，通知 Python
    emit_event("webview:ready");
}
```

Python 侧：
```python
@view.on("webview:ready")
def on_ready():
    self._start_mcp_server()  # 在事件循环就绪后启动
```

### 修复 2：增加主动轮询

在 `MainEventsCleared` 中无条件处理 MessageQueue，不依赖 `UserEvent::ProcessMessages`：

```rust
Event::MainEventsCleared => {
    // 每次 MainEventsCleared 都处理队列，不等待 wake
    let count = message_queue.process_all(...);
    if count > 0 {
        tracing::info!("Processed {} messages", count);
    }
}
```

### 修复 3：Windows 定时器唤醒

使用 Windows 定时器强制触发事件循环：
```rust
// 每 16ms 发送一个 UserEvent 确保事件循环迭代
std::thread::spawn(|| {
    loop {
        std::thread::sleep(Duration::from_millis(16));
        proxy.send_event(UserEvent::Tick);
    }
});
```

---

## 问题优先级

| 问题 | 严重程度 | 修复难度 | 建议 |
|------|----------|----------|------|
| MCP 时序耦合 | 高 | 中 | 短期：修复 1 或 2 |
| 双模式复杂度 | 中 | 高 | 长期：方案 B |
| MessageQueue 过度设计 | 中 | 高 | 长期：方案 A |
| 跨语言边界过多 | 低 | 高 | 长期：方案 D |
| Windows 兼容性 | 高 | 低 | 短期：修复 3 |

---

## 结论

当前架构的核心问题是**试图用单一机制（MessageQueue + Dispatcher）解决所有跨线程问题**，但这个机制本身有太多隐式依赖和时序要求。

**短期**：建议实施修复 2（主动轮询）和修复 3（定时器唤醒）
**长期**：建议考虑方案 D（MCP Sidecar 进程）或方案 B（统一事件驱动）

