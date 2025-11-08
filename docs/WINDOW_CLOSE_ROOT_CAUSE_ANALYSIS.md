# Window Close Button Root Cause Analysis

## 🎯 Executive Summary

**Problem**: The window close button (X button) does not work in DCC embedded mode, but programmatic `close()` works fine.

**Root Cause**: We are **detecting** WM_CLOSE correctly and **signaling** the lifecycle manager, but we are **NOT calling `DestroyWindow()`** to actually destroy the window. We're only dispatching the message and returning a flag to Python.

**Impact**: Window appears frozen when user clicks X button. The window handle remains valid, and the window stays visible.

**Solution**: Call `DestroyWindow()` immediately when WM_CLOSE is detected, similar to how we handle WM_SYSCOMMAND/SC_CLOSE.

---

## 🔍 Detailed Analysis

### 1. How Windows WM_CLOSE Works (Microsoft Documentation)

From [Microsoft WM_CLOSE documentation](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-close):

> **By default, the DefWindowProc function calls the DestroyWindow function to destroy the window.**

**Standard WM_CLOSE handling:**
```c
case WM_CLOSE:
    DestroyWindow(hWindow);  // ← This is what DefWindowProc does
    break;
```

**Key insight**: If you process WM_CLOSE yourself, **you must call `DestroyWindow()`**. Just dispatching the message is not enough if you've already handled it.

### 2. Current Implementation Issues

#### Issue #1: Inconsistent Handling in `message_pump.rs`

**Lines 61-90 in `src/webview/message_pump.rs`:**

```rust
// WM_SYSCOMMAND/SC_CLOSE: We call DestroyWindow ✅
if (msg.message == WM_SYSCOMMAND && ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE) {
    tracing::info!("[process_messages_for_hwnd] Close intent detected -> DestroyWindow");
    let _ = DestroyWindow(hwnd);  // ✅ CORRECT
    let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
    should_close = true;
    continue;
} 
// WM_CLOSE: We only dispatch, don't destroy ❌
else if msg.message == WM_CLOSE {
    tracing::debug!("[process_messages_for_hwnd] WM_CLOSE received");
    should_close = true;
    
    // PROBLEM: We dispatch to DefWindowProc, but we've already "handled" it
    // by setting should_close=true and continuing
    let _ = TranslateMessage(&msg);
    DispatchMessageW(&msg);  // ❌ This won't call DestroyWindow because we handled it
    continue;
}
```

**Why this doesn't work:**
1. We set `should_close = true` (marking it as "handled")
2. We dispatch to `DefWindowProc` via `DispatchMessageW`
3. But `DefWindowProc` sees we already handled it (we returned from our message loop)
4. Window never gets destroyed

#### Issue #2: Same Problem in `platform/windows.rs`

**Lines 73-87 in `src/webview/platform/windows.rs`:**

```rust
if self.is_close_message(&msg) {
    info!("[WindowsWindowManager] Close message detected: 0x{:04X}", msg.message);
    
    // Notify lifecycle manager
    if let Some(lifecycle) = self.lifecycle.lock().as_ref() {
        let reason = self.determine_close_reason(&msg);
        let _ = lifecycle.request_close(reason);  // ✅ Signal sent
    }
    
    should_close = true;  // ✅ Flag set
    
    // Still dispatch the message for proper cleanup
    let _ = TranslateMessage(&msg);
    DispatchMessageW(&msg);  // ❌ But window not destroyed
    continue;
}
```

**The problem**: We're doing everything right EXCEPT actually destroying the window.

### 3. Why Programmatic `close()` Works

When Python calls `outliner.close()`:

```python
def close(self):
    self._stop_event_processing()  # Stop timer
    self.cleanup_callbacks()        # Remove Maya callbacks
    self.webview.close()            # ← This calls Rust Drop
```

The Rust `Drop` implementation:

```rust
impl Drop for WebViewInner {
    fn drop(&mut self) {
        // ... cleanup code ...
        // The webview's Drop will eventually destroy the window
    }
}
```

**Why it works**: The `wry` webview's internal Drop implementation properly destroys the window because it's a controlled shutdown.

### 4. Comparison with Other Projects

#### Qt WebEngineView (from pixel-nexus.com article)

Qt handles this automatically:

```python
class MayaBrowser(QtWidgets.QMainWindow):
    def __init__(self, parent=None):
        super(MayaBrowser, self).__init__(parent)
        # Qt automatically handles WM_CLOSE → closeEvent → deleteLater
        self.setAttribute(QtCore.Qt.WA_DeleteOnClose)  # ← Key attribute
```

**How Qt does it:**
1. WM_CLOSE → Qt's event system
2. Triggers `closeEvent()`
3. If accepted, calls `deleteLater()`
4. Window destroyed in next event loop iteration

#### Tauri/wry (from GitHub research)

Tauri uses `tao` event loop which has a proper `CloseRequested` event:

```rust
// Tauri's approach (standalone mode)
event_loop.run(move |event, _, control_flow| {
    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            // Proper cleanup
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    }
});
```

**Why this works**: The event loop owns the window and handles destruction properly.

**Our problem**: In embedded mode, we don't have an event loop, so we must manually call `DestroyWindow()`.

---

## 💡 Solution

### Option 1: Call DestroyWindow Immediately (Recommended)

**Modify `src/webview/platform/windows.rs` lines 73-87:**

```rust
if self.is_close_message(&msg) {
    info!("[WindowsWindowManager] Close message detected: 0x{:04X}", msg.message);
    
    // Notify lifecycle manager FIRST
    if let Some(lifecycle) = self.lifecycle.lock().as_ref() {
        let reason = self.determine_close_reason(&msg);
        let _ = lifecycle.request_close(reason);
    }
    
    should_close = true;
    
    // ✅ FIX: Actually destroy the window
    if msg.message == WM_CLOSE || 
       (msg.message == WM_SYSCOMMAND && ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE) {
        let hwnd = HWND(msg.hwnd.0);
        let _ = DestroyWindow(hwnd);
        info!("[WindowsWindowManager] Window destroyed via DestroyWindow");
    } else {
        // For WM_DESTROY, WM_QUIT, just dispatch
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    
    continue;
}
```

**Pros:**
- ✅ Immediate window destruction (user sees instant response)
- ✅ Matches behavior of WM_SYSCOMMAND/SC_CLOSE path
- ✅ Simple, minimal code change
- ✅ Lifecycle signal sent before destruction

**Cons:**
- ⚠️ Window destroyed before Python cleanup completes
- ⚠️ Might cause issues if Python tries to access window after

### Option 2: Two-Phase Destruction (Safer)

**Phase 1**: Hide window immediately
**Phase 2**: Destroy after Python cleanup

```rust
if self.is_close_message(&msg) {
    info!("[WindowsWindowManager] Close message detected");
    
    // Notify lifecycle manager
    if let Some(lifecycle) = self.lifecycle.lock().as_ref() {
        let reason = self.determine_close_reason(&msg);
        let _ = lifecycle.request_close(reason);
    }
    
    should_close = true;
    
    // ✅ Phase 1: Hide window immediately (instant user feedback)
    let hwnd = HWND(msg.hwnd.0);
    unsafe {
        ShowWindow(hwnd, SW_HIDE);
        info!("[WindowsWindowManager] Window hidden (will destroy after cleanup)");
    }
    
    // ✅ Phase 2: Destroy will happen in Drop or explicit close()
    continue;
}
```

**Pros:**
- ✅ Instant visual feedback (window disappears)
- ✅ Safe cleanup order (Python → Rust → Window)
- ✅ No risk of accessing destroyed window

**Cons:**
- ⚠️ Window handle still valid (uses memory)
- ⚠️ Requires explicit DestroyWindow in Drop

---

## 📊 Comparison Table

| Approach | User Experience | Safety | Complexity | Recommendation |
|----------|----------------|--------|------------|----------------|
| **Current** | ❌ Frozen window | ✅ Safe | ✅ Simple | ❌ Broken |
| **Option 1** | ✅ Instant close | ⚠️ Moderate | ✅ Simple | ✅ **Recommended** |
| **Option 2** | ✅ Instant hide | ✅ Very safe | ⚠️ Moderate | ✅ Alternative |

---

## 🎯 Recommended Implementation

**Use Option 1** with one safety improvement:

```rust
if self.is_close_message(&msg) {
    info!("[WindowsWindowManager] Close message detected: 0x{:04X}", msg.message);
    
    let hwnd = HWND(msg.hwnd.0);
    
    // Notify lifecycle manager FIRST (before destruction)
    if let Some(lifecycle) = self.lifecycle.lock().as_ref() {
        let reason = self.determine_close_reason(&msg);
        let _ = lifecycle.request_close(reason);
    }
    
    should_close = true;
    
    // Destroy window immediately for WM_CLOSE and WM_SYSCOMMAND/SC_CLOSE
    match msg.message {
        WM_CLOSE => {
            unsafe {
                if DestroyWindow(hwnd).is_ok() {
                    info!("[WindowsWindowManager] ✅ Window destroyed successfully");
                } else {
                    warn!("[WindowsWindowManager] ⚠️ DestroyWindow failed");
                }
            }
        }
        WM_SYSCOMMAND if ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE => {
            unsafe {
                let _ = DestroyWindow(hwnd);
                info!("[WindowsWindowManager] ✅ Window destroyed (SC_CLOSE)");
            }
        }
        _ => {
            // For WM_DESTROY, WM_QUIT, just dispatch
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
    continue;
}
```

---

## 🔧 Files to Modify

1. **`src/webview/platform/windows.rs`** - Lines 73-87
2. **`src/webview/message_pump.rs`** - Lines 72-90 (for consistency)

---

## ✅ Expected Behavior After Fix

1. User clicks X button
2. WM_CLOSE message received
3. Lifecycle manager notified (close signal sent)
4. `DestroyWindow()` called immediately
5. Window disappears instantly
6. Python receives `should_close=true` from `process_events()`
7. Python calls `outliner.close()` for cleanup
8. Rust Drop runs (webview cleanup)
9. Complete!

---

**Next Steps**: Implement Option 1 and test in Maya.

