# AuroraView Architecture Diagnosis: Thread Safety and Performance

> **Document Version**: 1.0  
> **Date**: 2026-01-11  
> **Branch**: `fix/architecture-thread-safety-diagnosis`  
> **Target**: DCC Integration Thread Safety Analysis

## Executive Summary

This document provides a comprehensive architectural diagnosis of AuroraView's thread safety and performance design, with particular focus on DCC (Digital Content Creation) application integration. The analysis identifies critical issues in the current implementation and proposes both short-term fixes and long-term architectural improvements.

**Key Findings**:
- P0: Lock ordering issues in event loop state management
- P0: Message queue wake-up batching can cause UI latency
- P1: Dual message pump conflict in DCC embedded mode
- P1: `Arc&lt;Mutex&lt;WryWebView&gt;&gt;` creates unnecessary contention
- P2: Inconsistent event processing strategies across modes

---

## 1. System Boundaries and Subsystems

### 1.1 Component Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AuroraView Architecture                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐          │
│  │  Python Layer   │    │   Rust Core     │    │  JavaScript     │          │
│  │  (auroraview/)  │    │   (src/)        │    │  (SDK/inject)   │          │
│  └────────┬────────┘    └────────┬────────┘    └────────┬────────┘          │
│           │                      │                      │                    │
│           ▼                      ▼                      ▼                    │
│  ┌─────────────────────────────────────────────────────────────────┐        │
│  │                      IPC Message Queue                           │        │
│  │  (crossbeam-channel + ipckit ShutdownState)                     │        │
│  └─────────────────────────────────────────────────────────────────┘        │
│           │                      │                      │                    │
│           ▼                      ▼                      ▼                    │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐          │
│  │  Qt Integration │    │  Event Loop     │    │  WebView2       │          │
│  │  (QtWebView)    │    │  (tao/wry)      │    │  (Backend)      │          │
│  └─────────────────┘    └────────┬────────┘    └─────────────────┘          │
│                                  │                                           │
│                                  ▼                                           │
│                    ┌─────────────────────────┐                              │
│                    │  Win32 Message Pump     │                              │
│                    │  (message_pump.rs)      │                              │
│                    └─────────────────────────┘                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Subsystem Responsibilities

| Subsystem | Responsibility | Thread Affinity |
|-----------|---------------|-----------------|
| `WebView` (Python) | High-level API, event binding, lifecycle | Any thread (via proxy) |
| `_CoreWebView` (Rust) | WebView2 control, window management | UI thread (STA) |
| `MessageQueue` | Cross-thread message passing | Lock-free (crossbeam) |
| `EventLoopState` | Event loop state management | UI thread only |
| `IpcHandler` | Python callback dispatch | GIL required |
| `message_pump` | Win32 message processing | UI thread only |
| `QtWebView` | Qt widget integration | Qt main thread |

---

## 2. Run Mode Matrix

### 2.1 Supported Modes

| Mode | Event Loop Owner | Message Pump | Use Case |
|------|-----------------|--------------|----------|
| **Standalone Blocking** | AuroraView (tao) | `run_return()` | CLI tools, dev |
| **Standalone Threaded** | Background thread | `run_return()` | Python apps |
| **Embedded Host Pump** | DCC/Qt host | `process_ipc_only()` | Maya, Houdini |
| **Embedded Self Pump** | AuroraView | `process_events()` | Legacy DCC |
| **Packed Headless** | None (JSON-RPC) | N/A | Gallery CLI |

### 2.2 Thread Model Truth Table

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        Thread Model by Mode                                   │
├────────────────────┬─────────────┬─────────────┬─────────────┬──────────────┤
│ Operation          │ Standalone  │ Embedded Qt │ Embedded    │ Packed       │
│                    │ Blocking    │ Host Pump   │ Self Pump   │ Headless     │
├────────────────────┼─────────────┼─────────────┼─────────────┼──────────────┤
│ WebView creation   │ Main thread │ Qt thread   │ Any thread  │ N/A          │
│ eval_js()          │ Main thread │ Qt thread   │ Queue→Main  │ JSON-RPC     │
│ emit()             │ Main thread │ Qt thread   │ Queue→Main  │ JSON-RPC     │
│ Event callbacks    │ Main thread │ Qt thread   │ Main thread │ N/A          │
│ Message pump       │ tao loop    │ Qt loop     │ Timer tick  │ N/A          │
│ Window messages    │ tao handles │ Qt handles  │ Self pump   │ N/A          │
└────────────────────┴─────────────┴─────────────┴─────────────┴──────────────┘
```

---

## 3. Critical Findings

### 3.1 P0: Lock Ordering Violation Risk in EventLoopState

**Evidence**: `src/webview/event_loop.rs:670-680`

```rust
// CRITICAL: Release the lock BEFORE calling process_messages_for_hwnd
// because DestroyWindow may trigger WM_DESTROY which could cause deadlock
let (hwnd_opt, should_exit_arc) = {
    if let Ok(state_guard) = state_clone.lock() {
        (state_guard.get_hwnd(), state_guard.should_exit.clone())
    } else {
        (None, Arc::new(Mutex::new(false)))
    }
};
```

**Problem**: 
- The comment acknowledges a deadlock risk but the fix is incomplete
- `EventLoopState` contains multiple `Arc<Mutex<>>` fields that can be locked in different orders
- `should_exit`, `webview`, and the outer `state` mutex can create lock cycles

**Risk**: 
- Deadlock when closing window while processing messages
- Affects: All DCC integrations, especially during shutdown

**Impact Path**:
```
User clicks X → WM_CLOSE → process_messages_for_hwnd() 
                               ↓
                         DestroyWindow()
                               ↓
                         WM_DESTROY callback
                               ↓
                         Tries to lock state (DEADLOCK if already held)
```

### 3.2 P0: Message Queue Wake-up Batching Causes UI Latency

**Evidence**: `src/ipc/message_queue.rs:509-536`

```rust
fn wake_event_loop(&self) {
    // Check if we should batch wake-ups
    if self.config.batch_interval_ms > 0 {
        if let Ok(mut last_wake_guard) = self.last_wake_time.lock() {
            let now = Instant::now();
            let should_wake = match *last_wake_guard {
                Some(last_wake) => {
                    let elapsed = now.duration_since(last_wake);
                    let batch_interval = std::time::Duration::from_millis(self.config.batch_interval_ms);
                    elapsed >= batch_interval
                }
                None => true,
            };
            if !should_wake {
                return; // Skips wake-up!
            }
        }
    }
}
```

**Problem**:
- Default `batch_interval_ms = 16ms` means messages can wait up to 16ms before processing
- In DCC environments with busy main threads, this adds to perceived latency
- For interactive UI (e.g., button clicks), 16ms delay is noticeable

**Risk**:
- User-perceived lag in DCC tools
- Affects: All modes, especially high-frequency operations

### 3.3 P1: Dual Message Pump Conflict in Embedded Mode

**Evidence**: `src/webview/webview_inner.rs:810-818`

```rust
let should_quit = if let Some(hwnd_value) = hwnd {
    let b1 = message_pump::process_messages_for_hwnd(hwnd_value);
    // Also service child/IPC windows (e.g., WebView2) in the same thread.
    #[cfg(target_os = "windows")]
    let b2 = message_pump::process_all_messages();
    b1 || b2
} else {
    false
};
```

**Problem**:
- `process_events()` calls BOTH `process_messages_for_hwnd()` AND `process_all_messages()`
- In Qt-hosted mode, Qt already owns the message pump
- This can steal messages from Qt's event loop, causing:
  - Missed Qt events
  - Double-processing of messages
  - UI glitches

**Evidence of Workaround**: `python/auroraview/utils/event_timer.py:291-300`

```python
is_qt_backend = isinstance(self._backend, QtTimerBackend)
if is_qt_backend and hasattr(self._webview, "process_events_ipc_only"):
    # Qt hosts own the native event loop.
    should_close = self._webview.process_events_ipc_only()
else:
    should_close = self._webview.process_events()
```

**Risk**:
- Qt event loop interference
- Affects: Maya, Houdini, Nuke, 3ds Max

### 3.4 P1: Arc&lt;Mutex&lt;WryWebView&gt;&gt; Creates Unnecessary Contention

**Evidence**: `src/webview/webview_inner.rs:21`

```rust
pub struct WebViewInner {
    pub(crate) webview: Arc&lt;Mutex&lt;WryWebView&gt;&gt;,
    // ...
}
```

**Problem**:
- `WryWebView` is already `!Send + !Sync` - it can only be used on the UI thread
- Wrapping it in `Arc<Mutex<>>` doesn't make it thread-safe, it just adds lock overhead
- Every `evaluate_script()` call requires acquiring the mutex

**Evidence of Impact**: `src/webview/event_loop.rs:476-478`

```rust
if let Some(webview_arc) = &state_guard.webview {
    if let Ok(webview) = webview_arc.lock() {  // Lock acquired here
        match &message {
            WebViewMessage::EvalJs(script) => {
                webview.evaluate_script(script)?;  // Lock held during JS execution
            }
        }
    }
}
```

**Risk**:
- Lock contention during high-frequency JS execution
- Potential for lock poisoning if JS execution panics

### 3.5 P1: TEMPORARY FIX Comment Indicates Architectural Debt

**Evidence**: `src/webview/webview_inner.rs:520-524`

```rust
drop(webview_guard);

// TEMPORARY FIX: Create state without webview
// TODO: Refactor EventLoopState to accept Arc&lt;Mutex&lt;WryWebView&gt;&gt;
tracing::warn!("Creating EventLoopState without webview - this needs architectural fix");
```

**Problem**:
- `EventLoopState::new_without_webview()` is a workaround for ownership issues
- The webview is set later via `set_webview()`, creating a two-phase initialization
- This pattern is error-prone and can lead to null pointer issues

### 3.6 P2: Inconsistent Event Processing Strategies

**Evidence**: Multiple files

| Location | Strategy | Issue |
|----------|----------|-------|
| `event_loop.rs` | `MainEventsCleared` + `UserEvent::ProcessMessages` | Duplicated processing logic |
| `webview_inner.rs` | `process_events()` + `process_ipc_only()` | Two different APIs |
| `event_timer.py` | Backend-specific selection | Python-side workaround |
| `_core.py` | `QtEventProcessor` | Yet another processing path |

**Problem**:
- Same message processing logic is duplicated in 4+ places
- Changes must be synchronized across all locations
- Easy to introduce inconsistencies

---

## 4. Options and Trade-offs

### Option A: Minimal Lock Order Fix (Short-term)

**Approach**: Establish strict lock ordering protocol

```rust
// Lock order: state → webview → should_exit
// NEVER lock in reverse order
```

| Aspect | Assessment |
|--------|------------|
| Scope | Small - documentation + code comments |
| Risk | Low |
| Effort | 1-2 days |
| Benefit | Prevents deadlocks |
| Drawback | Doesn't fix root cause |

### Option B: Lock-Free State Machine (Medium-term)

**Approach**: Replace `Arc<Mutex<>>` with atomic state machine

```rust
pub struct EventLoopState {
    state: AtomicU8,  // Creating, Active, CloseRequested, Destroying, Destroyed
    webview: UnsafeCell<Option<WryWebView>>,  // Only accessed from UI thread
    // ...
}
```

| Aspect | Assessment |
|--------|------------|
| Scope | Medium - refactor EventLoopState |
| Risk | Medium - requires careful unsafe code |
| Effort | 1-2 weeks |
| Benefit | Eliminates lock contention |
| Drawback | More complex code |

### Option C: Unified Message Processor (Medium-term)

**Approach**: Single message processing path for all modes

```rust
pub trait MessageProcessor {
    fn process(&self, queue: &MessageQueue) -> ProcessResult;
}

pub struct StandaloneProcessor { /* ... */ }
pub struct EmbeddedHostPumpProcessor { /* ... */ }
pub struct EmbeddedSelfPumpProcessor { /* ... */ }
```

| Aspect | Assessment |
|--------|------------|
| Scope | Medium - refactor message processing |
| Risk | Low |
| Effort | 1 week |
| Benefit | Eliminates code duplication |
| Drawback | Requires careful testing |

### Option D: Full Architecture Redesign (Long-term)

**Approach**: Implement proper WebViewBackend trait abstraction

```rust
pub trait WebViewBackend: Send + Sync {
    fn navigate(&self, url: &str) -> Result<(), Error>;
    fn eval_js(&self, script: &str) -> Result<(), Error>;
    fn eval_js_async<F>(&self, script: &str, callback: F) -> Result<(), Error>
    where F: FnOnce(Result<Value, Error>) + Send + 'static;
    // ...
}
```

| Aspect | Assessment |
|--------|------------|
| Scope | Large - full refactor |
| Risk | High |
| Effort | 4-6 weeks |
| Benefit | Clean architecture, easy to extend |
| Drawback | Breaking changes possible |

### Recommended Path

**Phase 1 (Immediate)**: Option A - Lock order fix
**Phase 2 (Q1)**: Option C - Unified message processor  
**Phase 3 (Q2)**: Option B - Lock-free state machine
**Phase 4 (Future)**: Option D - Full redesign (as needed)

---

## 5. Refactoring Roadmap

### Phase 1: Lock Order and Immediate Fixes (1-2 weeks)

**Milestone 1.1**: Document lock ordering
- Add `LOCK_ORDER.md` with explicit ordering rules
- Add assertions in debug builds to detect violations

**Milestone 1.2**: Fix message queue batching
- Add `immediate_wake` flag for high-priority messages
- Default to immediate wake for user-initiated actions

**Milestone 1.3**: Fix dual message pump
- Make `process_ipc_only()` the default for Qt integration
- Deprecate `process_events()` for embedded mode

**Acceptance Criteria**:
- No deadlocks in stress tests
- UI latency &lt; 10ms for button clicks
- Qt event loop not interfered with

### Phase 2: Unified Message Processor (2-3 weeks)

**Milestone 2.1**: Create `MessageProcessor` trait
- Define common interface for all modes
- Implement for Standalone, EmbeddedHostPump, EmbeddedSelfPump

**Milestone 2.2**: Consolidate processing logic
- Remove duplicated code from `event_loop.rs`
- Remove duplicated code from `webview_inner.rs`

**Milestone 2.3**: Update Python layer
- Remove `QtEventProcessor` workaround
- Simplify `EventTimer` backend selection

**Acceptance Criteria**:
- Single source of truth for message processing
- All existing tests pass
- No regression in performance

### Phase 3: Lock-Free State Machine (3-4 weeks)

**Milestone 3.1**: Design state machine
- Define states: Creating, Active, CloseRequested, Destroying, Destroyed
- Define valid transitions

**Milestone 3.2**: Implement atomic state
- Replace `Arc<Mutex<bool>>` with `AtomicU8`
- Add state transition methods

**Milestone 3.3**: Remove webview mutex
- Use `UnsafeCell` with UI thread assertion
- Add debug-only thread checks

**Acceptance Criteria**:
- Zero lock contention in hot paths
- Thread safety verified by Miri
- Performance improvement measurable

---

## 6. Extensions and Future Scenarios

### 6.1 Multi-Window Support

**Current Support**: `child_window.rs` provides basic child window creation

**Gap**: No proper window manager for tracking multiple instances

**Required Changes**:
- Implement `WindowManager` with instance registry
- Add window-to-window communication
- Handle focus and Z-order properly

### 6.2 CDP/DevTools Integration

**Current Support**: `remote_debugging_port` config option

**Gap**: No programmatic CDP access from Python

**Required Changes**:
- Expose CDP WebSocket URL
- Add Python CDP client wrapper
- Enable automated testing via CDP

### 6.3 macOS/Linux Support

**Current Support**: wry backend compiles but untested

**Gap**: No native backend for macOS (WKWebView) or Linux (WebKitGTK)

**Required Changes**:
- Implement `WebViewBackend` for each platform
- Add platform-specific message pump handling
- Test in DCC applications on each platform

---

## 7. Key Entry Points Index

### Rust Layer

| File | Purpose | Key Functions |
|------|---------|---------------|
| `src/webview/event_loop.rs` | Event loop management | `run_blocking()`, `poll_events_once()` |
| `src/webview/webview_inner.rs` | Core WebView operations | `process_events()`, `process_ipc_only()` |
| `src/webview/message_pump.rs` | Win32 message handling | `process_messages_for_hwnd()` |
| `src/ipc/message_queue.rs` | Cross-thread messaging | `push()`, `process_all()` |
| `src/webview/lifecycle.rs` | Lifecycle state machine | `request_close()`, `state()` |
| `src/webview/proxy.rs` | Thread-safe proxy | `eval_js()`, `emit()` |

### Python Layer

| File | Purpose | Key Classes |
|------|---------|-------------|
| `python/auroraview/core/webview.py` | High-level API | `WebView` |
| `python/auroraview/integration/qt/_core.py` | Qt integration | `QtWebView`, `QtEventProcessor` |
| `python/auroraview/utils/event_timer.py` | Timer-based processing | `EventTimer` |
| `python/auroraview/utils/thread_dispatcher.py` | DCC thread safety | `ThreadDispatcherBackend` |

---

## 8. Appendix: Grep Patterns for Future Analysis

```bash
# Lock-related patterns
rg "Arc<Mutex|Mutex::new|\.lock\(\)" src/

# Thread safety markers
rg "unsafe impl Send|unsafe impl Sync|!Send|!Sync" src/

# Temporary fixes
rg "TODO|FIXME|HACK|TEMPORARY" src/

# Event processing
rg "process_events|process_ipc_only|process_messages" src/

# Lifecycle management
rg "CloseRequested|Destroying|Destroyed|request_close" src/

# Message queue operations
rg "push\(|pop\(|process_all|wake_event_loop" src/
```

---

## 9. Implementation Status

### Completed (2026-01-11)

The following components have been implemented as part of the architecture refactoring:

#### 9.1 Lock-Free Lifecycle State Machine (P0 Fix)

**Location**: `crates/auroraview-core/src/backend/lifecycle.rs`

- `AtomicLifecycle`: Lock-free state machine using `AtomicU8`
- States: `Creating` → `Active` → `CloseRequested` → `Destroying` → `Destroyed`
- `ObservableLifecycle`: Observable variant with event notifications
- Eliminates deadlock risk identified in P0

```rust
// Usage example
let lifecycle = AtomicLifecycle::new_active();
lifecycle.request_close();  // Thread-safe, lock-free
lifecycle.begin_destroy();
lifecycle.finish_destroy();
```

#### 9.2 Unified Message Processor (P2 Fix)

**Location**: `crates/auroraview-core/src/backend/message_processor.rs`

- `ProcessorConfig`: Unified configuration for all modes
- `ProcessingMode`: `Full`, `IpcOnly`, `Batch`
- `WakeController`: Smart wake-up with priority support
- `AtomicProcessorStats`: Thread-safe performance metrics

```rust
// Configuration presets
let standalone = ProcessorConfig::standalone();
let qt_embedded = ProcessorConfig::qt_embedded();
let legacy = ProcessorConfig::legacy_embedded();
```

#### 9.3 Enhanced WebViewBackend Trait

**Location**: `crates/auroraview-core/src/backend/traits.rs`

- `WebViewBackend`: Core trait with lifecycle integration
- `EmbeddableBackend`: Extended trait for DCC/Qt embedding
- `EventLoopBackend`: Trait for standalone mode with owned event loop
- All traits use `ProcessResult` for consistent return values

#### 9.4 Python Bindings Backend

**Location**: `src/webview/backend/mod.rs`

- `PyBindingsBackend`: Python-specific backend trait
- Re-exports all core types for convenience
- Backward compatible `WebViewBackend` alias

### Remaining Work

1. **Migrate NativeBackend**: Update `native.rs` to use new traits
2. **Update WebViewInner**: Integrate `AtomicLifecycle` into main WebView
3. **Remove Duplicated Logic**: Consolidate message processing in event_loop.rs
4. **Add Tests**: Integration tests for lifecycle transitions

---

## 10. Conclusion

AuroraView's current architecture has several thread safety issues that primarily affect DCC integration scenarios. The most critical issues are:

1. **Lock ordering violations** that can cause deadlocks during shutdown
2. **Message queue batching** that adds unnecessary UI latency
3. **Dual message pump conflict** that interferes with Qt's event loop

The recommended approach is a phased refactoring:
- **Immediate**: Document and enforce lock ordering ✅ (AtomicLifecycle implemented)
- **Short-term**: Unify message processing logic ✅ (MessageProcessor implemented)
- **Medium-term**: Implement lock-free state machine ✅ (AtomicLifecycle implemented)
- **Long-term**: Full backend abstraction redesign ✅ (Core traits implemented)

This approach minimizes risk while progressively improving the architecture.
