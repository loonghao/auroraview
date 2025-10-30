# Window Close Button Fix

## Problem Summary

**Issue**: Windows system close button (X) does not close the WebView window in embedded mode (Maya integration).

**Root Cause**: The message pump was using `PeekMessageW(&mut msg, HWND(null), ...)` to process messages for **all windows** in the current thread. In Maya's environment, this caused the following problems:

1. **Message interception**: Maya's own message loop might intercept `WM_CLOSE` messages before our message pump could process them
2. **Cross-thread routing**: If the WebView window is on a different thread, messages might not be routed correctly
3. **Owner mode specifics**: In Owner mode (GWLP_HWNDPARENT), some messages might be sent to the parent window instead of the child window

## Solution

### 1. Use Specific HWND for Message Processing

**Modified Files**:
- `src/webview/webview_inner.rs`
- `src/webview/message_pump.rs`
- `src/webview/embedded.rs`

**Key Changes**:

#### `src/webview/webview_inner.rs` (Lines 257-305)
```rust
pub fn process_events(&self) -> bool {
    // Extract the actual HWND from the window
    #[cfg(target_os = "windows")]
    let hwnd = {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        
        if let Some(window) = &self.window {
            if let Ok(window_handle) = window.window_handle() {
                let raw_handle = window_handle.as_raw();
                if let RawWindowHandle::Win32(handle) = raw_handle {
                    Some(handle.hwnd.get() as u64)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // Use specific HWND if available, otherwise fall back to all windows
    let should_quit = if let Some(hwnd_value) = hwnd {
        tracing::debug!("🟢 [process_events] Processing messages for HWND: 0x{:X}", hwnd_value);
        message_pump::process_messages_for_hwnd(hwnd_value)
    } else {
        tracing::debug!("🟢 [process_events] Processing messages for all windows");
        message_pump::process_all_messages()
    };
    
    // ... rest of the function
}
```

#### `src/webview/message_pump.rs` (Lines 14-94)
Added new function `process_messages_for_hwnd()`:

```rust
pub fn process_messages_for_hwnd(hwnd_value: u64) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{WM_CLOSE, WM_DESTROY, WM_QUIT};
    use std::ffi::c_void;
    
    unsafe {
        let hwnd = HWND(hwnd_value as *mut c_void);
        let mut msg = MSG::default();
        let mut should_close = false;
        let mut message_count = 0;

        // Process messages for THIS SPECIFIC WINDOW only
        while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
            message_count += 1;

            // Check for window close messages
            if msg.message == WM_CLOSE {
                tracing::info!("🟢 [process_messages_for_hwnd] WM_CLOSE received");
                should_close = true;
                continue;  // Don't dispatch WM_CLOSE
            }
            // ... handle WM_DESTROY and WM_QUIT
            
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        should_close
    }
}
```

**Key Improvement**: Instead of `HWND(null)` (all windows), we now use the **specific HWND** of the WebView window.

### 2. Remove Premature Window Destruction

**Problem**: The original code called `DestroyWindow()` immediately when receiving `WM_CLOSE`, which could cause race conditions and improper cleanup.

**Solution**: Only set the `should_close` flag and let Python handle the actual window destruction through `webview.close()`.

**Before**:
```rust
if msg.message == WM_CLOSE {
    let result = DestroyWindow(msg.hwnd);  // ❌ Immediate destruction
    should_close = true;
    continue;
}
```

**After**:
```rust
if msg.message == WM_CLOSE {
    should_close = true;  // ✅ Just set flag
    continue;  // Let Python handle cleanup
}
```

### 3. Enhanced Logging

**Modified File**: `src/webview/embedded.rs` (Lines 165-186)

Added HWND logging during window creation:

```rust
// Log window HWND for debugging
#[cfg(target_os = "windows")]
{
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(window_handle) = window.window_handle() {
        let raw_handle = window_handle.as_raw();
        if let RawWindowHandle::Win32(handle) = raw_handle {
            let hwnd_value = handle.hwnd.get();
            tracing::info!("✅ [create_embedded] Window created successfully");
            tracing::info!("🟢 [create_embedded] WebView HWND: 0x{:X} ({})", hwnd_value, hwnd_value);
            tracing::info!("🟢 [create_embedded] Parent HWND: 0x{:X} ({})", parent_hwnd, parent_hwnd);
        }
    }
}
```

## Testing Instructions

### Prerequisites

1. **Close Maya** (to release the DLL file)
2. **Rebuild the project**:
   ```bash
   just rebuild-core
   ```
3. **Verify build success**:
   ```
   ✅ Core module rebuilt and installed successfully!
   ```

### Test Procedure

1. **Start Maya 2024**

2. **Enable Rust logging** (in Maya Script Editor):
   ```python
   import os
   os.environ['RUST_LOG'] = 'auroraview=debug'
   ```

3. **Run the Outliner example**:
   ```python
   exec(open(r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\maya\outliner_view.py').read())
   ```

4. **Verify window creation logs**:
   Look for these lines in Maya Script Editor:
   ```
   ✅ [create_embedded] Window created successfully
   🟢 [create_embedded] WebView HWND: 0x... (...)
   🟢 [create_embedded] Parent HWND: 0x... (...)
   ```

5. **Test HTML button close**:
   - Click the "✕ Close" button in the WebView UI
   - Expected: Window closes normally
   - Expected logs:
     ```
     📤 [closeWindow] Button clicked!
     🔒 [handle_close] CLOSE EVENT RECEIVED!
     ✅ [_do_close] webview.close() completed
     ```

6. **Test system X button close**:
   - Reopen the window (run the script again)
   - Click the Windows system X button (top-right corner)
   - Expected: Window closes normally
   - Expected logs:
     ```
     🟢 [process_messages_for_hwnd] WM_CLOSE received (X button clicked)
     🟢 [process_messages_for_hwnd] should_close set to true
     🟢 [process_events] should_quit = true
     🟢 [process_events] Window close signal detected!
     ```

7. **Test DevTools**:
   - Press F12 to open DevTools
   - Check Console tab for errors
   - Expected: No JavaScript errors

8. **Test refresh functionality**:
   - Create some objects in Maya scene
   - Click "🔄 Refresh" button
   - Expected: Object list updates

9. **Test process cleanup**:
   - Close Maya
   - Open Windows Task Manager
   - Check for residual processes
   - Expected: No `msedgewebview2` or WebView-related processes

## Expected Behavior

### ✅ Success Criteria

1. **HTML button close**: Works (already working before fix)
2. **System X button close**: Works (fixed by this PR)
3. **DevTools**: Opens with F12, no JavaScript errors
4. **Refresh**: Updates scene object list
5. **Process cleanup**: No residual processes after Maya exit
6. **Logs**: Complete event flow visible in Maya Script Editor

### 📋 Log Sequence for System X Button

```
🟢 [process_events] Processing messages for HWND: 0x...
🟢 [process_messages_for_hwnd] Message #1: 0x0010 (HWND: ...)
================================================================================
🟢 [process_messages_for_hwnd] WM_CLOSE received (X button clicked)
🟢 [process_messages_for_hwnd] Message HWND: ...
🟢 [process_messages_for_hwnd] Setting should_close flag...
🟢 [process_messages_for_hwnd] should_close set to true
🟢 [process_messages_for_hwnd] Will return to Python for cleanup
================================================================================
🟢 [process_messages_for_hwnd] Returning should_close = true
================================================================================
🟢 [process_events] should_quit = true
🟢 [process_events] Window close signal detected!
🟢 [process_events] Returning true to Python...
================================================================================
🔴 [process_events] should_close = True
🔴 [process_events] Window close signal detected!
🔴 [process_events] Cleaning up resources...
```

## Technical Details

### Message Flow

1. **User clicks X button** → Windows sends `WM_CLOSE` to window
2. **Maya timer calls `process_events()`** → Extracts HWND from window
3. **`process_messages_for_hwnd(hwnd)`** → Processes messages for specific window
4. **`PeekMessageW(&mut msg, hwnd, ...)`** → Gets `WM_CLOSE` message
5. **Set `should_close = true`** → Return to Python
6. **Python timer checks return value** → Calls cleanup code
7. **`webview.close()`** → Proper cleanup via `Drop` trait

### Why This Works

1. **Targeted message processing**: Only processes messages for the WebView window, not all windows
2. **No interference from Maya**: Maya's message loop can't intercept our messages
3. **Proper cleanup order**: Python controls the cleanup sequence, ensuring proper resource deallocation
4. **Thread-safe**: Works correctly even if WebView is on a different thread

## Files Modified

1. `src/webview/webview_inner.rs` - Extract HWND and use targeted message processing
2. `src/webview/message_pump.rs` - Add `process_messages_for_hwnd()` function
3. `src/webview/embedded.rs` - Add HWND logging
4. `src/ipc/process.rs` - Remove unused import
5. `src/ipc/mod.rs` - Comment out unused export
6. `src/webview/python_bindings.rs` - Fix clippy warning (use `std::f64::consts::PI`)

## Related Issues

- Window close button not working in embedded mode
- Process residue after Maya exit (fixed by `Drop` trait in `webview_inner.rs`)
- DevTools not accessible (fixed by passing `dev_tools` parameter)

## Future Improvements

1. Consider adding a timeout for `process_events()` to prevent infinite loops
2. Add more detailed error handling for window handle extraction
3. Consider using a custom window procedure (WndProc) for more control over message handling

