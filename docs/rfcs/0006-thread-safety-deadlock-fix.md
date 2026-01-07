# RFC 0006: Thread Safety Deadlock Fix

## 问题背景

在 DCC 环境（Maya/Houdini/Unreal）中，经常出现"IDE 通过 MCP 协议推送消息，但前端和 MCP 服务线程不一致导致线程不安全卡死"的问题。

### 根本原因

在 `NativeBackend::process_events()` 中，当处理 Python 相关消息时（`McpToolCall`、`PythonCallbackDeferred`），代码在**持有 WebView mutex 锁**的情况下直接执行 Python handler。

这会导致重入死锁：
1. MCP 请求到达 → 入队 `McpToolCall`
2. DCC 主线程调用 `process_events()` → 拿到 WebView 锁 → 执行 Python handler
3. Python handler 内部调用 `webview.eval_js()` / `webview.emit()` / `webview.process_events()`
4. 这些调用再次尝试获取 WebView 锁 → **同线程重入死锁**

## 解决方案

### 核心策略：临时释放锁执行 Python

修改 `NativeBackend::process_events()` 的消息处理逻辑：

- **WebView 操作**：持锁执行（必须在 WebView 线程）
- **Python 执行**：临时释放锁 → 执行 Python → 重新加锁继续

### 具体修改

#### 1. 统一的锁管理模式

```rust
// 之前：批量处理，持锁执行所有消息
message_queue.process_batch(limit, |message| {
    // 在锁内执行所有消息，包括 Python
});

// 现在：逐个处理，遇到 Python 临时释放锁
let mut webview_guard = self.webview.lock()?;
while let Some(message) = message_queue.pop() {
    match message {
        // WebView 操作：持锁执行
        WebViewMessage::EvalJs(script) => {
            webview_guard.evaluate_script(&script)?;
        }
        // Python 操作：释放锁执行
        WebViewMessage::McpToolCall { .. } => {
            drop(webview_guard.take());
            execute_mcp_tool(...);
            webview_guard = self.webview.lock()?; // 重新加锁
        }
    }
}
```

#### 2. 扩展到所有 Python 回调

不仅修复 `McpToolCall`，同时修复 `PythonCallbackDeferred`：

```rust
WebViewMessage::PythonCallbackDeferred { callback_id, event_name, data } => {
    // 释放锁
    drop(webview_guard.take());
    
    // 执行 Python 回调
    execute_python_callback(callback_id, &event_name, data, &self.ipc_handler, "NativeBackend");
    
    // 重新加锁
    webview_guard = self.webview.lock()?;
}
```

#### 3. 移除旧的延迟执行模式

之前的 `deferred_callbacks` 收集模式会破坏消息顺序，已完全移除：

```rust
// 移除：
let mut deferred_callbacks = Vec::new();
// ... 收集到数组
// ... 在锁外批量执行

// 改为：立即执行（但临时释放锁）
```

## 架构优势

### 1. 消除重入死锁
- **问题**：Python handler 调用 `webview.emit()` → 重入 `process_events()` → 尝试获取已持有的锁
- **解决**：执行 Python 前释放锁，避免重入冲突

### 2. 保持消息顺序
- **问题**：旧的 `deferred_callbacks` 模式会打乱消息执行顺序
- **解决**：逐个处理消息，保证严格的 FIFO 顺序

### 3. 线程安全保证
- **WebView 操作**：仍在正确线程（主线程/UI线程）执行
- **Python 执行**：在主线程执行，但不持有 WebView 锁

### 4. 错误隔离
- 单个 Python handler 出错不影响后续消息处理
- 锁重新获取失败有明确错误处理

## 测试验证

### 高风险场景
1. **DCC 环境**：Maya/Houdini 中 MCP tool 调用 `webview.emit()`
2. **事件回调**：Python 事件处理器中调用 `webview.eval_js()`
3. **嵌套调用**：`process_events()` 内部再次调用 `process_events()`

### 验证方法
```python
# 之前会死锁的代码
def mcp_tool_handler():
    # 这会重入 process_events() 并尝试获取已持有的锁
    webview.emit("progress", {"step": 1})
    webview.eval_js("console.log('step 1')")
    return {"result": "success"}

# 现在应该正常工作
```

## 性能影响

### 锁操作开销
- **增加**：每个 Python 消息需要额外的 unlock/lock 操作
- **减少**：消除死锁后，整体响应性显著提升

### 内存使用
- **减少**：移除 `deferred_callbacks` Vec，减少内存分配
- **优化**：逐个处理消息，内存使用更平稳

## 向后兼容性

- ✅ **API 兼容**：Python 层 API 无变化
- ✅ **行为兼容**：消息处理行为更可靠
- ✅ **性能改进**：消除死锁，整体性能提升

## 后续改进

### 1. 超时机制
为 Python handler 添加执行超时，避免长时间阻塞主线程：

```rust
// 未来可以添加
let timeout = Duration::from_secs(30);
tokio::time::timeout(timeout, execute_python_handler()).await?;
```

### 2. 异步执行
对于耗时的 Python 操作，考虑异步执行 + 回调模式：

```rust
// 未来可以支持
WebViewMessage::McpToolCallAsync { ... } => {
    // 在后台线程执行，完成后通过消息队列返回结果
}
```

### 3. 错误恢复
增强锁重新获取失败的恢复机制：

```rust
// 当前：记录错误，继续处理
// 未来：可以尝试重新初始化 WebView
```

## 总结

这个修复解决了 AuroraView 在 DCC 环境中最常见的稳定性问题。通过"临时释放锁执行 Python"的策略，我们：

1. **消除了重入死锁**：Python handler 可以安全调用 WebView API
2. **保持了消息顺序**：严格 FIFO 处理，行为可预测  
3. **提升了稳定性**：特别是在 Maya/Houdini/Unreal 等 DCC 环境
4. **简化了代码**：移除复杂的延迟执行逻辑

这是 AuroraView 架构稳定性的重要里程碑，为后续功能开发奠定了坚实基础。