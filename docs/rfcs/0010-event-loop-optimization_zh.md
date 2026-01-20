# RFC-0010: 事件循环架构优化

## 概述

本 RFC 提出对 AuroraView 事件循环架构的全面优化，以提高线程安全性、减少锁竞争并增强可维护性。设计灵感来自 Tauri 的事件系统，并借鉴了高 star Rust 项目的成熟模式。

## 动机

### 当前问题

1. **锁竞争**：大量使用 `Arc<Mutex<>>` 导致潜在的死锁和性能瓶颈
2. **代码重复**：消息处理逻辑在三个地方重复（`UserEvent::ProcessMessages`、`MainEventsCleared`、`poll_events_once`）
3. **紧耦合**：事件循环状态管理与消息处理紧密耦合
4. **可观测性不足**：缺乏结构化的事件追踪或指标用于调试生产问题
5. **中文字符编码问题**：流式场景中的消息序列化可能导致编码问题

### 目标

- **无锁热路径**：最小化频繁访问代码路径中的互斥锁使用
- **清晰的关注点分离**：将状态管理与消息处理解耦
- **Tauri 风格的事件系统**：采用 Tauri 经过充分测试的事件架构模式
- **流式消息支持**：正确处理流式消息（如 AI 聊天响应）
- **更好的错误处理**：具有可操作信息的结构化错误类型

## 设计

### 第一阶段：短期优化（已完成）

已实现的更改：

1. **退出标志使用 AtomicBool**
   - 将 `should_exit: Arc<Mutex<bool>>` 改为 `Arc<AtomicBool>`
   - 使用 `Ordering::SeqCst` 实现无锁退出信号

2. **提取消息处理器**
   - 创建 `process_webview_message()` 函数
   - 消除三个事件处理器中的代码重复

### 第二阶段：基于 Channel 的架构

用高性能 channel 替换内部队列：

```rust
// 当前：基于 Mutex 的内部队列
pub struct MessageQueue {
    tx: Sender<WebViewMessage>,  // crossbeam-channel（已经无锁）
    rx: Receiver<WebViewMessage>,
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,  // 这里有锁
    // ...
}

// 提议：完全无锁
pub struct MessageQueue {
    tx: Sender<WebViewMessage>,
    rx: Receiver<WebViewMessage>,
    event_loop_proxy: AtomicCell<Option<EventLoopProxy<UserEvent>>>,  // crossbeam::atomic
    // ...
}
```

**推荐的 Channel（按性能排序）**：

| 库 | Stars | 性能 | 特性 |
|---------|-------|-------------|----------|
| kanal | 1.7k | 最快（比 std 快 2-10 倍） | 同步 + 异步 |
| crossbeam-channel | 7.4k | 非常快 | 成熟、稳定 |
| flume | 2.5k | 快 | 同步 + 异步混合 |

**建议**：继续使用 `crossbeam-channel` 保持稳定性，高吞吐场景考虑 `kanal`。

### 第三阶段：Tauri 风格的事件系统

采用 Tauri 的 "try_lock + pending queue" 模式进行事件发射：

```rust
/// 带有待处理队列的事件监听器，用于处理竞争
pub struct EventListeners {
    /// 主监听器映射
    inner: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    /// 锁竞争时的待处理操作
    pending: Arc<Mutex<Vec<PendingEvent>>>,
}

enum PendingEvent {
    Emit { event: String, data: serde_json::Value },
    Subscribe { event: String, handler: EventHandler },
    Unsubscribe { event: String, id: ListenerId },
}

impl EventListeners {
    pub fn emit(&self, event: &str, data: serde_json::Value) {
        // 尝试获取锁而不阻塞
        if let Ok(inner) = self.inner.try_lock() {
            self.do_emit(&inner, event, data);
            self.flush_pending(&inner);
        } else {
            // 锁竞争 - 排队稍后处理
            if let Ok(mut pending) = self.pending.lock() {
                pending.push(PendingEvent::Emit {
                    event: event.to_string(),
                    data,
                });
            }
        }
    }

    fn flush_pending(&self, inner: &HashMap<String, Vec<EventHandler>>) {
        if let Ok(mut pending) = self.pending.try_lock() {
            for op in pending.drain(..) {
                match op {
                    PendingEvent::Emit { event, data } => {
                        self.do_emit(inner, &event, data);
                    }
                    // 处理其他待处理操作...
                }
            }
        }
    }
}
```

**优势**：
- 事件发射从不阻塞
- 自动合并快速事件
- 防止嵌套事件处理器中的死锁

### 第四阶段：消息处理管道

引入结构化的消息处理管道：

```rust
/// 可扩展消息处理的消息处理器 trait
pub trait MessageProcessor: Send + Sync {
    /// 处理单个消息，返回 true 如果已处理
    fn process(&self, message: &WebViewMessage, ctx: &ProcessorContext) -> bool;

    /// 优先级（越高 = 越早处理）
    fn priority(&self) -> i32 { 0 }
}

/// 传递给处理器的上下文
pub struct ProcessorContext<'a> {
    pub webview: &'a WryWebView,
    pub window: Option<&'a Window>,
    pub state: &'a EventLoopState,
}

/// 消息处理链
pub struct MessagePipeline {
    processors: Vec<Box<dyn MessageProcessor>>,
}

impl MessagePipeline {
    pub fn process(&self, message: &WebViewMessage, ctx: &ProcessorContext) {
        for processor in &self.processors {
            if processor.process(message, ctx) {
                return; // 消息已处理
            }
        }
        tracing::warn!("未处理的消息: {:?}", message);
    }
}
```

**默认处理器**：
1. `CloseProcessor` - 处理窗口关闭（最高优先级）
2. `NavigationProcessor` - URL/HTML 加载
3. `ScriptProcessor` - JavaScript 执行
4. `EventProcessor` - 向 JS 发射事件
5. `WindowProcessor` - 窗口可见性/状态

### 第五阶段：流式消息支持

用于 AI 聊天和其他流式场景：

```rust
/// 流式感知的事件发射器
pub struct StreamingEmitter {
    /// 不完整 UTF-8 序列的缓冲区
    utf8_buffer: Vec<u8>,
    /// 用于关联的当前消息 ID
    message_id: Option<String>,
}

impl StreamingEmitter {
    /// 发射文本增量，处理部分 UTF-8 序列
    pub fn emit_text_delta(&mut self, delta: &[u8], queue: &MessageQueue) {
        // 追加到缓冲区
        self.utf8_buffer.extend_from_slice(delta);
        
        // 查找有效的 UTF-8 边界
        let valid_len = self.find_valid_utf8_boundary();
        if valid_len == 0 {
            return; // 等待更多数据
        }
        
        // 提取并发射有效部分
        let valid_bytes: Vec<u8> = self.utf8_buffer.drain(..valid_len).collect();
        if let Ok(text) = String::from_utf8(valid_bytes) {
            queue.push(WebViewMessage::EmitEvent {
                event_name: "agui:text_message_content".to_string(),
                data: serde_json::json!({
                    "message_id": self.message_id,
                    "delta": text,
                }),
            });
        }
    }
    
    fn find_valid_utf8_boundary(&self) -> usize {
        // 查找最后一个有效的 UTF-8 字符边界
        for i in (1..=self.utf8_buffer.len().min(4)).rev() {
            let end = self.utf8_buffer.len() - i;
            if std::str::from_utf8(&self.utf8_buffer[..end]).is_ok() {
                return end;
            }
        }
        0
    }
}
```

### 第六阶段：结构化错误处理 ✅ 已完成

已在 `src/webview/event_loop.rs` 中实现 `EventLoopError` 枚举：

```rust
/// 带上下文的事件循环错误
#[derive(Debug, Error)]
pub enum EventLoopError {
    #[error("WebView lock failed: {context}")]
    WebViewLock { context: String },

    #[error("JavaScript execution failed: {script_preview}")]
    ScriptExecution {
        script_preview: String,
        message: String,
    },

    #[error("Message queue full: {queue_len} messages pending")]
    QueueFull { queue_len: usize },

    #[error("Encoding error: {message}")]
    Encoding { message: String },

    #[error("Event loop closed or unavailable")]
    EventLoopClosed,

    #[error("Window operation failed: {operation}")]
    WindowOperation { operation: String },

    #[error("Message processing timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}

/// 事件循环操作的 Result 类型
pub type EventLoopResult<T> = Result<T, EventLoopError>;
```

主要特性：
- 结构化错误类型，提供可操作的调试上下文
- 辅助构造函数用于常见错误模式
- 通过 `webview::EventLoopError` 和 `webview::EventLoopResult` 导出

### 第七阶段：Packed IPC 健壮性（已完成）

#### 观察到的错误（Packed 运行时）

```
Invalid argument
[Backend:stderr] [AuroraView] Error in API server loop: [Errno 22] Invalid argument
[AuroraView] Call timed out: av_call_...
```

#### 实施状态 ✅

本阶段所有项目已完成：

1. **Python packed API server 加固** ✅
   - 实现于 `python/auroraview/core/packed.py`
   - 遇到 `EINVAL/EBADF`，发送 `backend_error` 并退出循环
   - 无效 JSON 或 loop 异常时，发送 `backend_error` 以解除 UI 阻塞

2. **Rust ready 握手增强** ✅
   - 实现于 `crates/auroraview-cli/src/packed/backend.rs`
   - Ready 超时和失败处理
   - 后端错误转发到前端

3. **IPC 健康对齐** ✅
   - 心跳已实现：Python 每 2 秒发送 `backend_health`
   - JS 通过 `heartbeatTimeoutMs` 配置监控心跳
   - `event_bridge.ts` 中的 `markBackendUnhealthy()` / `markBackendHealthy()`
   - 后端不健康时的快速失败行为

4. **stderr 转发节流** ✅
   - stderr 消息通过 `backend:stderr` 事件转发到 WebView
   - 在 backend.rs 的 stdout/stderr 处理中实现

5. **前端默认错误处理器** ✅
   - `backend_error` 事件自动标记后端不可用
   - `clearAllPendingCalls()` 取消所有待处理调用并返回结构化错误
   - `backendFailFast` 配置（默认：true）在后端不健康时立即拒绝调用

#### 配置与默认值 ✅

通过 `window.__AURORAVIEW_CONFIG__` 和 `auroraview.setConfig()` 暴露配置：
- `callTimeoutMs`: 单次调用超时（默认 30000）
- `backendFailFast`: 后端不健康时快速失败（默认 true）
- `heartbeatTimeoutMs`: 后端健康超时（默认 0，收到首个健康信号后启用）

#### 协议版本 ✅

健康相关消息携带 `schema_version: 1`：
- `backend_health` 事件（Python → JS）
- `__ping__` / `__pong__`（Rust ↔ Python）

## 实施计划

| 阶段 | 范围 | 风险 | 状态 | 备注 |
|-------|-------|------|--------|-------|
| 1 | AtomicBool + DRY | 低 | ✅ **已完成** | `should_exit` 改为 AtomicBool，提取 `process_webview_message()` |
| 2 | Channel 优化 | 低 | ⏳ 延期 | 当前性能足够，无明确瓶颈 |
| 3 | Tauri 事件模式 | 中 | ⏳ 延期 | 需要性能基准测试验证需求 |
| 4 | 消息管道 | 中 | ⏳ 延期 | 当前架构满足需求 |
| 5 | 流式支持 | 中 | ⏳ 延期 | UTF-8 问题已在第七阶段中解决 |
| 6 | 错误处理 | 低 | ✅ **已完成** | `EventLoopError` 枚举实现 |
| 7 | Packed IPC 健壮性 | 中 | ✅ **已完成** | 心跳、健康检查、错误处理 |

**第 2-5 阶段延期原因**：
- 当前实现已满足性能需求，无明确瓶颈
- 核心优先级是稳定性和功能完善
- 这些优化可在未来版本中根据性能基准测试结果按需实施


## 迁移策略

1. **向后兼容**：所有更改都是内部的；公共 API 保持稳定
2. **Feature Flags**：新实现最初在 feature flags 后面
3. **渐进式推出**：在 Gallery 应用中测试后再默认启用
4. **指标**：添加性能计数器以比较前后差异

## 测试策略

1. **单元测试**：每个处理器和组件独立测试
2. **集成测试**：从 Python 到 JS 的完整消息流
3. **压力测试**：高频消息发送（1000+ msg/sec）
4. **编码测试**：Unicode 边界情况（CJK、emoji、代理对）
5. **DCC 集成**：在 Maya/Houdini 中测试以验证没有死锁

## 考虑过的替代方案

### 1. Actor 模型（Actix）
- **优点**：清晰分离、async 原生
- **缺点**：依赖较重、学习曲线、对我们的用例来说过度
- **决定**：不采用

### 2. 完整的 tokio 运行时
- **优点**：Rust 异步的行业标准
- **缺点**：与 DCC 事件循环冲突、集成复杂
- **决定**：不采用（保持同步优先，加异步包装）

### 3. 自定义事件循环
- **优点**：完全控制
- **缺点**：维护负担、重新发明轮子
- **决定**：不采用（继续使用 tao/wry）

## 参考资料

- [Tauri 事件系统](https://github.com/tauri-apps/tauri/tree/dev/crates/tauri/src/event)
- [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam)
- [kanal](https://github.com/fereidani/kanal)
- [flume](https://github.com/zesterer/flume)

## 附录：流式消息编码问题

AI 聊天中的中文字符乱码问题可能由以下原因导致：

1. **流式中的部分 UTF-8**：DeepSeek API 返回的流式块可能分割 UTF-8 序列
2. **JSON 转义**：JSON 序列化中 Unicode 字符的双重转义
3. **WebView 执行**：JavaScript `evaluate_script()` 在处理原始 JSON 时可能存在编码问题

**根本原因分析**：

```python
# Python 端（正确）
data = {"delta": "你好"}  # UTF-8 字符串
json_str = json.dumps(data, ensure_ascii=False)  # '{"delta": "你好"}'

# Rust 端（poll_events_once 中的潜在问题）
let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
# 这会双重转义有效的 JSON！
```

**修复**：`process_webview_message()` 函数现在直接使用 `data.to_string()` 而不进行额外转义，这应该能解决编码问题。
