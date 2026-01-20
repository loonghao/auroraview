# RFC-0010: Event Loop Architecture Optimization

## Summary

This RFC proposes a comprehensive optimization of AuroraView's event loop architecture to improve thread safety, reduce lock contention, and enhance maintainability. The design is inspired by Tauri's event system and leverages established patterns from high-star Rust projects.

## Motivation

### Current Problems

1. **Lock Contention**: Heavy use of `Arc<Mutex<>>` causes potential deadlocks and performance bottlenecks
2. **Code Duplication**: Message processing logic is repeated in three places (`UserEvent::ProcessMessages`, `MainEventsCleared`, `poll_events_once`)
3. **Tight Coupling**: Event loop state management is tightly coupled with message processing
4. **Limited Observability**: No structured event tracing or metrics for debugging production issues
5. **Chinese Character Encoding Issues**: Message serialization in streaming scenarios may cause encoding problems

### Goals

- **Lock-free Hot Paths**: Minimize mutex usage in frequently accessed code paths
- **Clean Separation of Concerns**: Decouple state management from message processing
- **Tauri-inspired Event System**: Adopt patterns from Tauri's well-tested event architecture
- **Streaming Message Support**: Proper handling of streaming messages (e.g., AI chat responses)
- **Better Error Handling**: Structured error types with actionable information

## Design

### Phase 1: Short-term Optimizations (Completed)

Already implemented changes:

1. **AtomicBool for Exit Flag**
   - Changed `should_exit: Arc<Mutex<bool>>` to `Arc<AtomicBool>`
   - Lock-free exit signaling using `Ordering::SeqCst`

2. **Extracted Message Handler**
   - Created `process_webview_message()` function
   - Eliminated code duplication across three event handlers

### Phase 2: Channel-Based Architecture

Replace internal queue with high-performance channels:

```rust
// Current: Internal Mutex-based queue
pub struct MessageQueue {
    tx: Sender<WebViewMessage>,  // crossbeam-channel (already lock-free)
    rx: Receiver<WebViewMessage>,
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,  // Lock here
    // ...
}

// Proposed: Fully lock-free
pub struct MessageQueue {
    tx: Sender<WebViewMessage>,
    rx: Receiver<WebViewMessage>,
    event_loop_proxy: AtomicCell<Option<EventLoopProxy<UserEvent>>>,  // crossbeam::atomic
    // ...
}
```

**Recommended Channels (by performance)**:

| Library | Stars | Performance | Features |
|---------|-------|-------------|----------|
| kanal | 1.7k | Fastest (2-10x faster than std) | Sync + Async |
| crossbeam-channel | 7.4k | Very Fast | Mature, stable |
| flume | 2.5k | Fast | Sync + Async hybrid |

**Recommendation**: Continue with `crossbeam-channel` for stability, consider `kanal` for high-throughput scenarios.

### Phase 3: Tauri-Inspired Event System

Adopt Tauri's "try_lock + pending queue" pattern for event emission:

```rust
/// Event listeners with pending queue for contention handling
pub struct EventListeners {
    /// Primary listeners map
    inner: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    /// Pending operations when lock is contended
    pending: Arc<Mutex<Vec<PendingEvent>>>,
}

enum PendingEvent {
    Emit { event: String, data: serde_json::Value },
    Subscribe { event: String, handler: EventHandler },
    Unsubscribe { event: String, id: ListenerId },
}

impl EventListeners {
    pub fn emit(&self, event: &str, data: serde_json::Value) {
        // Try to acquire lock without blocking
        if let Ok(inner) = self.inner.try_lock() {
            self.do_emit(&inner, event, data);
            self.flush_pending(&inner);
        } else {
            // Lock contended - queue for later
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
                    // Handle other pending operations...
                }
            }
        }
    }
}
```

**Benefits**:
- Never blocks on event emission
- Automatically coalesces rapid events
- Prevents deadlocks in nested event handlers

### Phase 4: Message Processing Pipeline

Introduce a structured message processing pipeline:

```rust
/// Message processor trait for extensible message handling
pub trait MessageProcessor: Send + Sync {
    /// Process a single message, return true if handled
    fn process(&self, message: &WebViewMessage, ctx: &ProcessorContext) -> bool;

    /// Priority (higher = earlier in chain)
    fn priority(&self) -> i32 { 0 }
}

/// Context passed to processors
pub struct ProcessorContext<'a> {
    pub webview: &'a WryWebView,
    pub window: Option<&'a Window>,
    pub state: &'a EventLoopState,
}

/// Message processing chain
pub struct MessagePipeline {
    processors: Vec<Box<dyn MessageProcessor>>,
}

impl MessagePipeline {
    pub fn process(&self, message: &WebViewMessage, ctx: &ProcessorContext) {
        for processor in &self.processors {
            if processor.process(message, ctx) {
                return; // Message handled
            }
        }
        tracing::warn!("Unhandled message: {:?}", message);
    }
}
```

**Default Processors**:
1. `CloseProcessor` - Handles window close (highest priority)
2. `NavigationProcessor` - URL/HTML loading
3. `ScriptProcessor` - JavaScript evaluation
4. `EventProcessor` - Event emission to JS
5. `WindowProcessor` - Window visibility/state

### Phase 5: Streaming Message Support

For AI chat and other streaming scenarios:

```rust
/// Streaming-aware event emission
pub struct StreamingEmitter {
    /// Buffer for incomplete UTF-8 sequences
    utf8_buffer: Vec<u8>,
    /// Current message ID for correlation
    message_id: Option<String>,
}

impl StreamingEmitter {
    /// Emit text delta, handling partial UTF-8 sequences
    pub fn emit_text_delta(&mut self, delta: &[u8], queue: &MessageQueue) {
        // Append to buffer
        self.utf8_buffer.extend_from_slice(delta);
        
        // Find valid UTF-8 boundary
        let valid_len = self.find_valid_utf8_boundary();
        if valid_len == 0 {
            return; // Wait for more data
        }
        
        // Extract and emit valid portion
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
        // Find the last valid UTF-8 character boundary
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

### Phase 6: Structured Error Handling ✅ Done

Implemented `EventLoopError` enum in `src/webview/event_loop.rs`:

```rust
/// Event loop errors with context for debugging
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

/// Result type for event loop operations
pub type EventLoopResult<T> = Result<T, EventLoopError>;
```

Key features:
- Structured error types with actionable context
- Helper constructors for common error patterns
- Exported via `webview::EventLoopError` and `webview::EventLoopResult`

### Phase 7: Packed IPC Robustness (Done)

#### Observed Errors (Packed Runtime)

```
Invalid argument
[Backend:stderr] [AuroraView] Error in API server loop: [Errno 22] Invalid argument
[AuroraView] Call timed out: av_call_...
```

#### Implementation Status ✅

All items from this phase have been implemented:

1. **Python packed API server hardening** ✅
   - Implemented in `python/auroraview/core/packed.py`
   - On `EINVAL/EBADF`, emits `backend_error` and exits the loop
   - On invalid JSON or loop errors, emits `backend_error` to unblock UI state

2. **Rust ready-handshake resilience** ✅
   - Implemented in `crates/auroraview-cli/src/packed/backend.rs`
   - Ready timeout and failure handling
   - Backend error forwarding to frontend

3. **IPC health alignment** ✅
   - Heartbeat implemented: Python sends `backend_health` every 2 seconds
   - JS monitors heartbeat via `heartbeatTimeoutMs` config
   - `markBackendUnhealthy()` / `markBackendHealthy()` in `event_bridge.ts`
   - Fail-fast behavior when backend is unhealthy

4. **Stderr forwarding hygiene** ✅
   - Stderr messages are forwarded to WebView via `backend:stderr` event
   - Implemented in backend.rs stdout/stderr handling

5. **Frontend default handler** ✅
   - `backend_error` event automatically marks backend unavailable
   - `clearAllPendingCalls()` cancels all pending calls with structured error
   - `backendFailFast` config (default: true) rejects calls immediately when unhealthy

#### Configuration & Defaults ✅

Exposed via `window.__AURORAVIEW_CONFIG__` and `auroraview.setConfig()`:
- `callTimeoutMs`: per-call timeout (default 30000)
- `backendFailFast`: reject calls when backend unhealthy (default true)
- `heartbeatTimeoutMs`: backend health timeout (default 0, enabled after first health signal)

#### Protocol Versioning ✅

All health-related messages carry `schema_version: 1`:
- `backend_health` event (Python → JS)
- `__ping__` / `__pong__` (Rust ↔ Python)

## Implementation Plan

| Phase | Scope | Risk | Status | Notes |
|-------|-------|------|--------|-------|
| 1 | AtomicBool + DRY | Low | ✅ **Done** | `should_exit` 改为 AtomicBool，提取 `process_webview_message()` |
| 2 | Channel optimization | Low | ⏳ Deferred | 当前性能足够，无明确瓶颈 |
| 3 | Tauri event pattern | Medium | ⏳ Deferred | 需要性能基准测试验证需求 |
| 4 | Message pipeline | Medium | ⏳ Deferred | 当前架构满足需求 |
| 5 | Streaming support | Medium | ⏳ Deferred | UTF-8 问题已在 Phase 7 中解决 |
| 6 | Error handling | Low | ✅ **Done** | `EventLoopError` 枚举实现 |
| 7 | Packed IPC robustness | Medium | ✅ **Done** | 心跳、健康检查、错误处理 |

**Phase 2-5 延期原因**：
- 当前实现已满足性能需求，无明确瓶颈
- 核心优先级是稳定性和功能完善
- 这些优化可在未来版本中根据性能基准测试结果按需实施


## Migration Strategy

1. **Backward Compatibility**: All changes are internal; public API remains stable
2. **Feature Flags**: New implementations behind feature flags initially
3. **Gradual Rollout**: Test in Gallery app before enabling by default
4. **Metrics**: Add performance counters to compare before/after

## Testing Strategy

1. **Unit Tests**: Each processor and component tested in isolation
2. **Integration Tests**: Full message flow from Python to JS
3. **Stress Tests**: High-frequency message sending (1000+ msg/sec)
4. **Encoding Tests**: Unicode edge cases (CJK, emoji, surrogate pairs)
5. **DCC Integration**: Test in Maya/Houdini to verify no deadlocks

## Alternatives Considered

### 1. Actor Model (Actix)
- **Pros**: Clean separation, async-native
- **Cons**: Heavy dependency, learning curve, overkill for our use case
- **Decision**: Not adopted

### 2. Full tokio Runtime
- **Pros**: Industry standard for async Rust
- **Cons**: Conflicts with DCC event loops, complex integration
- **Decision**: Not adopted (keep sync-first with async wrappers)

### 3. Custom Event Loop
- **Pros**: Full control
- **Cons**: Maintenance burden, reinventing the wheel
- **Decision**: Not adopted (continue with tao/wry)

## References

- [Tauri Event System](https://github.com/tauri-apps/tauri/tree/dev/crates/tauri/src/event)
- [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam)
- [kanal](https://github.com/fereidani/kanal)
- [flume](https://github.com/zesterer/flume)

## Appendix: Streaming Message Encoding Issue

The Chinese character garbling issue in AI chat is likely caused by:

1. **Partial UTF-8 in Streaming**: DeepSeek API returns streaming chunks that may split UTF-8 sequences
2. **JSON Escaping**: Double escaping of Unicode characters in JSON serialization
3. **WebView Evaluation**: JavaScript `evaluate_script()` may have encoding issues with raw JSON

**Root Cause Analysis**:

```python
# Python side (correct)
data = {"delta": "你好"}  # UTF-8 string
json_str = json.dumps(data, ensure_ascii=False)  # '{"delta": "你好"}'

# Rust side (potential issue in poll_events_once)
let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
# This double-escapes valid JSON!
```

**Fix**: The `process_webview_message()` function now uses `data.to_string()` directly without additional escaping, which should resolve the encoding issue.
