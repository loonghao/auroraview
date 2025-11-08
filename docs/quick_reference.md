# Quick Reference: Cross-Platform Lifecycle Management

## 🚀 Quick Start

### For Python Users

```python
from auroraview import AuroraView

# Embedded mode (DCC integration)
view = AuroraView.create_embedded(
    parent_hwnd=maya_window_handle,
    width=800,
    height=600,
    url="https://example.com"
)

# Process events in timer callback
def on_timer():
    if view.process_events():  # Returns True when window should close
        print("Window closed")
        view.close()
        return False  # Stop timer
    return True  # Continue timer
```

### For Rust Developers

```rust
use auroraview::webview::{WebViewConfig, WebViewInner};
use auroraview::ipc::{IpcHandler, MessageQueue};
use std::sync::Arc;

// Create embedded webview
let config = WebViewConfig::builder()
    .title("My App")
    .width(800)
    .height(600)
    .url("https://example.com")
    .build();

let ipc_handler = Arc::new(IpcHandler::new());
let message_queue = Arc::new(MessageQueue::new());

let webview = WebViewInner::create_embedded(
    parent_hwnd,
    800,
    600,
    config,
    ipc_handler,
    message_queue,
)?;

// Process events
loop {
    if webview.process_events() {
        println!("Window closed");
        break;
    }
    std::thread::sleep(std::time::Duration::from_millis(16));
}
```

## 📚 Core Concepts

### Lifecycle States

```
Creating → Active → CloseRequested → Destroying → Destroyed
```

### Close Reasons

| Reason | Description |
|--------|-------------|
| `UserRequest` | User clicked close button |
| `AppRequest` | Application requested close |
| `ParentClosed` | Parent window closed (embedded mode) |
| `SystemShutdown` | System shutdown |
| `Error` | Error occurred |

## 🔧 Key APIs

### LifecycleManager

```rust
// Check current state
let state = lifecycle.state();

// Request close
lifecycle.request_close(CloseReason::UserRequest)?;

// Check for close signal (non-blocking)
if let Some(reason) = lifecycle.check_close_requested() {
    println!("Close requested: {:?}", reason);
}

// Register cleanup handler
lifecycle.register_cleanup(|| {
    println!("Cleanup executed");
});

// Execute all cleanup handlers
lifecycle.execute_cleanup();
```

### PlatformWindowManager

```rust
// Process platform events
let should_close = platform_manager.process_events();

// Check window validity
let is_valid = platform_manager.is_window_valid();

// Setup close handlers
platform_manager.setup_close_handlers(lifecycle);

// Cleanup
platform_manager.cleanup();
```

## 🎯 Common Patterns

### Pattern 1: Guaranteed Cleanup

```rust
use scopeguard::defer;

fn my_function() {
    defer! {
        println!("This always runs, even on panic!");
    }
    
    // Your code here
}
```

### Pattern 2: Event-Driven Close Detection

```rust
// Old way (polling)
loop {
    if !is_window_valid() {
        break;
    }
    thread::sleep(Duration::from_millis(100));
}

// New way (event-driven)
if lifecycle.check_close_requested().is_some() {
    // Handle close immediately
}
```

### Pattern 3: Platform-Specific Handling

```rust
#[cfg(target_os = "windows")]
let platform_manager = {
    use crate::webview::platform;
    let manager = platform::create_platform_manager(hwnd);
    manager.setup_close_handlers(lifecycle.clone());
    Some(manager)
};

#[cfg(target_os = "macos")]
let platform_manager = {
    use crate::webview::platform;
    let manager = platform::create_platform_manager(ns_window);
    manager.setup_close_handlers(lifecycle.clone());
    Some(manager)
};
```

## 🐛 Troubleshooting

### Issue: Window not closing

**Check:**
1. Is `process_events()` being called regularly?
2. Is the lifecycle state correct?
3. Are there any errors in the logs?

**Solution:**
```rust
// Enable debug logging
tracing::info!("Lifecycle state: {:?}", lifecycle.state());
tracing::info!("Window valid: {}", platform_manager.is_window_valid());
```

### Issue: Resource leaks

**Check:**
1. Is `Drop` being called?
2. Are cleanup handlers registered?

**Solution:**
```rust
// Register cleanup handler
lifecycle.register_cleanup(|| {
    println!("Cleanup handler called");
});

// Verify Drop is called
impl Drop for MyStruct {
    fn drop(&mut self) {
        println!("Drop called");
    }
}
```

### Issue: Thread safety errors

**Check:**
1. Are you storing raw pointers?
2. Is Send + Sync implemented correctly?

**Solution:**
```rust
// Store as u64 instead of raw pointer
pub struct WindowsWindowManager {
    hwnd_value: u64,  // ✅ Thread-safe
    // NOT: hwnd: HWND,  // ❌ Not thread-safe
}

// Implement Send + Sync
unsafe impl Send for WindowsWindowManager {}
unsafe impl Sync for WindowsWindowManager {}
```

## 📖 Further Reading

- [Lifecycle Management Guide](./lifecycle_management.md)
- [Improvements Summary](./improvements_summary.md)
- [Changelog](./CHANGELOG_lifecycle.md)

## 💡 Tips

1. **Always use `scopeguard`** for cleanup code
2. **Prefer event-driven** over polling
3. **Check lifecycle state** before operations
4. **Enable logging** for debugging
5. **Test on all platforms** before release

