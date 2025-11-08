# Maya Integration Issues and Solutions

## 🔴 Current Problems

### Problem 1: WebView Blocks Maya Main Thread

**Symptom**: When running WebView in Maya, Maya's UI becomes unresponsive.

**Root Cause**: 
- Even though WebView runs in a background Python thread, the Rust event loop (`run_return()`) is blocking
- The event loop processes Windows messages synchronously
- This somehow interferes with Maya's Qt event loop

**Current Code Flow**:
```python
webview = WebView(title="Test")  # No parent_hwnd
webview.show()  # Calls show_window() -> run_event_loop_blocking()
```

```rust
// In standalone.rs
pub fn show_window() {
    // Creates event loop
    let event_loop = EventLoopBuilder::new().build();
    
    // Runs blocking event loop (even in background thread!)
    event_loop.run_return(|event, _, control_flow| {
        // Process events...
    });
}
```

### Problem 2: Maya Cannot Exit After Closing WebView

**Symptom**: After closing the WebView window, Maya's UI flickers/refreshes continuously and cannot exit normally.

**Root Cause**:
- Event loop cleanup is incomplete
- Window resources are not properly released
- Maya's Qt event loop is left in a confused state

**Evidence**:
```rust
// In aurora_view.rs:show_window()
webview_inner.run_event_loop_blocking();  // Blocks until window closes

// After event loop exits:
*inner_after = None;  // Drops WebViewInner

// But: Window messages may still be in flight
// Qt event loop may still reference the window
```

---

## ✅ Solution: Use Embedded Mode with Parent Window

### The Correct Approach

**Key Insight**: When you provide a `parent_hwnd`, WebView uses **embedded mode**, which:
1. Creates the window as a child/owner of the parent
2. **Does NOT run an event loop** (non-blocking!)
3. Relies on the parent's event loop to process messages

**Correct Code**:
```python
# Get Maya's main window HWND
import maya.OpenMayaUI as omui
import shiboken2
from PySide2 import QtWidgets

maya_main_window_ptr = omui.MQtUtil.mainWindow()
maya_main_window = shiboken2.wrapInstance(int(maya_main_window_ptr), QtWidgets.QWidget)
parent_hwnd = maya_main_window.winId()

# Create WebView with parent (EMBEDDED MODE)
webview = WebView(
    title="AI Chat",
    width=1200,
    height=800,
    parent_hwnd=parent_hwnd,  # THIS IS THE KEY!
    parent_mode="owner"  # Use "owner" for cross-thread safety
)

webview.show()  # Non-blocking! Returns immediately!
```

### Why This Works

1. **No Event Loop Blocking**:
   ```rust
   // In aurora_view.rs:show_embedded()
   fn show_embedded(&self) -> PyResult<()> {
       // Create window
       let webview = WebViewInner::create_embedded(...)?;
       
       // DON'T run event loop!
       // Parent's event loop handles our messages
       
       Ok(())  // Returns immediately!
   }
   ```

2. **Maya's Qt Event Loop Handles Everything**:
   - Maya's Qt event loop processes Windows messages for all windows
   - Our WebView window is a child/owner of Maya's window
   - Qt automatically processes our window's messages

3. **Clean Lifecycle**:
   - When Maya closes, parent window monitor detects it
   - WebView automatically closes
   - No resource leaks

---

## 🐛 Remaining Issue: Message Processing in Embedded Mode

### Problem

Even in embedded mode, there's a subtle issue:

**WebView messages (eval_js, emit) are not processed!**

**Why?**
```rust
// In embedded.rs:create_embedded()
let event_loop = EventLoopBuilder::new().build();  // Created
let message_queue = Arc::new(MessageQueue::new());  // Created

// But: Event loop is NEVER run!
// So: Messages in the queue are never processed!
```

**Impact**:
- `webview.eval_js(script)` doesn't work
- `webview.emit(event, data)` doesn't work
- JavaScript injection fails

### Solution Options

#### Option 1: Use Windows Message Pump (Recommended)

Create a background thread that:
1. Processes Windows messages for the WebView window
2. Processes WebView message queue
3. Runs periodically without blocking

```rust
// Pseudo-code
fn start_message_pump(hwnd: HWND, message_queue: Arc<MessageQueue>, webview: Arc<Mutex<WryWebView>>) {
    thread::spawn(move || {
        loop {
            // Process Windows messages
            process_messages_for_hwnd(hwnd);
            
            // Process WebView messages
            message_queue.process_all(|msg| {
                match msg {
                    WebViewMessage::EvalJs(script) => {
                        webview.lock().unwrap().evaluate_script(&script);
                    }
                    // ... other messages
                }
            });
            
            thread::sleep(Duration::from_millis(16));  // ~60 FPS
        }
    });
}
```

#### Option 2: Use Maya's Timer

Register a Qt timer in Maya that periodically processes messages:

```python
from PySide2 import QtCore

class WebViewMessageProcessor(QtCore.QObject):
    def __init__(self, webview):
        super().__init__()
        self.webview = webview
        self.timer = QtCore.QTimer()
        self.timer.timeout.connect(self.process_messages)
        self.timer.start(16)  # ~60 FPS
    
    def process_messages(self):
        # Call Rust function to process message queue
        self.webview._process_pending_messages()
```

#### Option 3: Hybrid Approach (Best)

Combine both:
1. Background thread for Windows messages (required for window responsiveness)
2. Expose `process_pending_messages()` method for manual processing
3. Optionally use Maya timer for automatic processing

---

## 📋 Implementation Plan

### Phase 1: Fix Message Processing (High Priority)

1. **Add message pump thread to embedded mode**:
   - Modify `embedded.rs:create_embedded()`
   - Start background thread that processes both Windows messages and WebView messages
   - Store thread handle in `WebViewInner`

2. **Ensure proper cleanup**:
   - Stop message pump thread when WebView is dropped
   - Clean up all resources

3. **Test in Maya**:
   - Verify `eval_js()` works
   - Verify `emit()` works
   - Verify JavaScript injection works

### Phase 2: Improve Lifecycle Management (Medium Priority)

1. **Better parent window monitoring**:
   - Current implementation checks every 500ms
   - Consider using Windows hooks for immediate notification

2. **Graceful shutdown**:
   - Ensure all threads are stopped
   - Ensure all resources are released
   - Prevent Maya from hanging

### Phase 3: Documentation and Examples (Medium Priority)

1. **Update examples**:
   - Create `examples/08_maya_integration_fixed.py` ✅ (Done)
   - Add Houdini example
   - Add 3ds Max example

2. **Update documentation**:
   - Explain embedded vs standalone mode
   - Explain when to use each mode
   - Provide troubleshooting guide

---

## 🎯 Quick Fix for Users (Temporary)

Until the message pump is implemented, users can work around the issue:

### Workaround 1: Manual Message Processing

```python
import threading
import time

def process_messages_loop(webview):
    while True:
        try:
            # This will be implemented in next version
            webview._process_pending_messages()
        except:
            pass
        time.sleep(0.016)  # ~60 FPS

webview = WebView(parent_hwnd=maya_hwnd, parent_mode="owner")
webview.show()

# Start manual message processing
thread = threading.Thread(target=process_messages_loop, args=(webview,), daemon=True)
thread.start()
```

### Workaround 2: Use Standalone Mode with Background Thread (Current)

This is what `examples/07_ai_chat_non_blocking.py` does:
- Creates WebView without parent
- Runs in background thread
- Works, but may still have minor blocking issues

---

## 📊 Comparison: Standalone vs Embedded Mode

| Feature | Standalone Mode | Embedded Mode |
|---------|----------------|---------------|
| **Parent Window** | None | Required |
| **Event Loop** | Runs own loop (blocking) | Uses parent's loop |
| **Blocking** | Yes (even in thread) | No |
| **Message Processing** | Automatic | **Needs message pump** ⚠️ |
| **Lifecycle** | Independent | Tied to parent |
| **Use Case** | Standalone tools | DCC integration |
| **Maya Compatibility** | ⚠️ May block | ✅ Non-blocking |
| **Current Status** | ✅ Works | ⚠️ Needs message pump |

---

## 🔧 Next Steps

1. **Implement message pump for embedded mode** (This PR)
2. **Test thoroughly in Maya 2024**
3. **Test in other DCCs** (Houdini, 3ds Max, Blender)
4. **Update documentation**
5. **Release new version**

---

## 📝 References

- [Tao Event Loop Documentation](https://docs.rs/tao/latest/tao/event_loop/)
- [Wry WebView Documentation](https://docs.rs/wry/latest/wry/)
- [Windows Message Pump](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues)
- [Maya Python API](https://help.autodesk.com/view/MAYAUL/2024/ENU/)

