# AuroraView UE Integration

Unreal Engine integration for AuroraView — supports UE4/UE5 Python scripting, GameThread adapter, Slate UI integration, and GC-safe WebView embedding.

## Features

- **GameThread Executor** — ensures operations run on UE's GameThread
- **Slate UI Integration** — embed WebView as Slate widget
- **GC-safe WebView Embedding** — handles UE garbage collection
- **Python Bindings** — optional `pyo3` bindings for UE Python scripting

## Core Types

### `GameThreadId`

Wrapper for UE GameThread ID.

```rust
use auroraview_ue::GameThreadId;

let id = GameThreadId::current();
assert!(id.is_current());
```

### `UeGameThreadExecutor`

Ensures operations are run on UE's GameThread.

```rust
use auroraview_ue::UeGameThreadExecutor;

let (executor, rx) = UeGameThreadExecutor::new();

// Execute on GameThread (immediate if already on GameThread)
executor.execute(|| {
    println!("Running on GameThread");
});

// Execute with callback
executor.execute_with_callback(
    || Ok(()),
    |result| println!("Task completed: {:?}", result),
);
```

### `SlateWidgetHandle`

Opaque handle for Slate widgets.

```rust
use auroraview_ue::SlateWidgetHandle;

let handle = SlateWidgetHandle::null();
assert!(handle.is_null());

let handle = SlateWidgetHandle(123);
assert!(!handle.is_null());
```

### `UeWebViewConfig`

Configuration for UE WebView integration.

```rust
use auroraview_ue::{UeWebViewConfig, UeEmbedMode};

let config = UeWebViewConfig {
    initial_size: (1024, 768),
    embed_mode: UeEmbedMode::SlateWidget,
    dev_tools: true,
    init_script: Some("console.log('hello')".to_string()),
};
```

### `UeEmbedMode`

WebView embedding mode within UE Slate.

```rust
pub enum UeEmbedMode {
    SlateWidget,        // Embed as Slate widget (SWindow/SWidget)
    NativeChildWindow,  // Embed as child window (Win32 HWND parenting)
    FloatingWindow,    // Floating tool window (for non-Slate DCCs)
}
```

### `UeIntegration`

Manages WebView embedding within Unreal Engine's Slate UI system.

```rust
use auroraview_ue::UeIntegration;

let config = UeWebViewConfig::default();
let integration = UeIntegration::new(config);

// Set parent Slate widget
let handle = SlateWidgetHandle(123);
integration.set_parent_widget(handle);

// Get GameThread executor
let executor = integration.executor();

// Process pending GameThread tasks (call from GameThread each frame)
integration.process_tasks();
```

## Python Bindings

When compiled with `python-bindings` feature, the following Python API is available:

```python
from auroraview_ue import UeIntegration

# Create UE integration
ue = UeIntegration(width=1024, height=768, dev_tools=True)

# Process pending GameThread tasks (call from GameThread)
ue.process_tasks()

# Create a WebView (must be called from GameThread)
handle = ue.create_webview("https://example.com")
print(f"WebView handle: {handle}")
```

## Usage Example (Rust)

```rust
use auroraview_ue::{UeIntegration, UeWebViewConfig, UeEmbedMode};

fn init_ue_integration() {
    let config = UeWebViewConfig {
        initial_size: (1024, 768),
        embed_mode: UeEmbedMode::SlateWidget,
        dev_tools: true,
        init_script: Some("window.ueReady = true;".to_string()),
    };
    
    let integration = UeIntegration::new(config);
    
    // In UE, call `process_tasks()` each frame from GameThread
    // integration.process_tasks();
}
```

## Thread Safety

UE requires certain operations to run on specific threads:

- **GameThread** — UI operations, Slate widget creation
- **RenderThread** — rendering operations (avoid WebView operations here)

`UeGameThreadExecutor` ensures your code runs on the correct thread:

```rust
let executor = UeGameThreadExecutor::new();
    
// This will run on GameThread (immediate if already there)
executor.execute(|| {
    // UE UI operations here
});
```

## GC-safe Memory Management

UE uses garbage collection for UObjects. `UeIntegration` handles this safely:

- Uses `Arc<Mutex<Option<Receiver>>` for task receiver
- Handles UE object lifecycle correctly
- Avoids dangling pointers

## Testing

```bash
# Run unit tests
cargo test -p auroraview-ue --lib

# Run integration tests
cargo test -p auroraview-ue --test integration_test

# Run all tests
cargo test -p auroraview-ue

# Run clippy
cargo clippy -p auroraview-ue -- -D warnings
```

## Dependencies

- `auroraview-core` — core WebView management
- `auroraview-signals` — Qt-style signals/slots
- `tokio` — async runtime
- `serde` — serialization
- `crossbeam-channel` — channel for GameThread task dispatch
- `pyo3` (optional) — Python bindings

## Current Status

- [x] **GameThread executor** — implemented
- [x] **SlateWidgetHandle** — implemented
- [x] **UeWebViewConfig** — implemented
- [x] **UeIntegration** — implemented (placeholder for `create_webview()`)
- [x] **Python bindings** — implemented
- [ ] **Actual WebView embedding** — TODO (requires UE FFI)
- [ ] **Slate widget creation** — TODO (requires UE C++ integration)

## Next Steps

1. **Implement actual WebView embedding** — create Slate widget via FFI
2. **Get native handle** — retrieve HWND (Windows) from Slate widget
3. **Pass to WebView backend** — embed WebView into UE
4. **Add more tests** — test actual embedding (requires mock UE environment)

## License

Same as AuroraView project (MIT OR Apache-2.0).
