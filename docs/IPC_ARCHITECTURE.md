# IPC Architecture

## Overview

AuroraView uses a modular IPC (Inter-Process Communication) architecture to enable bidirectional communication between JavaScript (WebView) and Python.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         JavaScript Layer                         │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Event Bridge (Initialization Script)                      │ │
│  │  - Intercepts window.dispatchEvent()                       │ │
│  │  - Forwards CustomEvent to window.ipc.postMessage()        │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↓ ↑
                    window.ipc.postMessage()
                    window.dispatchEvent()
                              ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│                          Rust IPC Layer                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  IpcHandler (src/ipc/handler.rs)                           │ │
│  │  - Receives messages from JavaScript                       │ │
│  │  - Invokes Python callbacks via PyO3                       │ │
│  │  - Thread-safe callback storage (DashMap)                  │ │
│  └────────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  MessageQueue (src/ipc/message_queue.rs)                   │ │
│  │  - Queues messages from Python to JavaScript               │ │
│  │  - Lock-free MPMC channel (crossbeam-channel)              │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↓ ↑
                         PyO3 Bindings
                              ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│                         Python Layer                             │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  WebView.on(event_name, callback)                          │ │
│  │  WebView.emit(event_name, data)                            │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Event Bridge (JavaScript)

**Location**: Injected via `with_initialization_script()` in `src/webview/embedded.rs` and `src/webview/standalone.rs`

**Purpose**: Intercepts JavaScript events and forwards them to Rust

**Implementation**:
```javascript
// Override dispatchEvent to intercept custom events
EventTarget.prototype.dispatchEvent = function(event) {
    if (event instanceof CustomEvent) {
        // Send to Rust via IPC
        window.ipc.postMessage(JSON.stringify({
            type: 'event',
            event: event.type,
            detail: event.detail || {}
        }));
    }
    return originalDispatchEvent.call(this, event);
};
```

### 2. IpcHandler (Rust)

**Location**: `src/ipc/handler.rs`

**Purpose**: Manages event callbacks and message routing

**Key Features**:
- Thread-safe callback storage using `DashMap`
- Automatic Python GIL management
- Error handling and logging

**API**:
```rust
pub fn register_callback(&self, event_name: String, callback: PyObject)
pub fn handle_message(&self, message: IpcMessage) -> Result<(), String>
```

### 3. MessageQueue (Rust)

**Location**: `src/ipc/message_queue.rs`

**Purpose**: Thread-safe message queue for Python → JavaScript communication

**Key Features**:
- Lock-free MPMC channel (`crossbeam-channel`)
- Non-blocking push/pop operations
- Automatic message batching

**API**:
```rust
pub fn push(&self, message: WebViewMessage)
pub fn pop(&self) -> Option<WebViewMessage>
pub fn drain(&self) -> Vec<WebViewMessage>
```

## Message Flow

### JavaScript → Python

1. **JavaScript**: Dispatch CustomEvent
   ```javascript
   window.dispatchEvent(new CustomEvent('my_event', { detail: { key: 'value' } }));
   ```

2. **Event Bridge**: Intercept and forward
   ```javascript
   window.ipc.postMessage(JSON.stringify({
       type: 'event',
       event: 'my_event',
       detail: { key: 'value' }
   }));
   ```

3. **Rust IpcHandler**: Parse and route
   ```rust
   let message = IpcMessage {
       event: "my_event".to_string(),
       data: json!({"key": "value"}),
       id: None,
   };
   ipc_handler.handle_message(message)?;
   ```

4. **Python Callback**: Execute
   ```python
   @webview.on("my_event")
   def handle_event(data):
       print(f"Received: {data}")  # {'key': 'value'}
   ```

### Python → JavaScript

1. **Python**: Emit event
   ```python
   webview.emit("update_data", {"frame": 120})
   ```

2. **Rust MessageQueue**: Queue message
   ```rust
   message_queue.push(WebViewMessage::EmitEvent {
       event_name: "update_data".to_string(),
       data: json!({"frame": 120}),
   });
   ```

3. **Event Loop**: Process queue
   ```rust
   for message in message_queue.drain() {
       match message {
           WebViewMessage::EmitEvent { event_name, data } => {
               let script = format!(
                   "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
                   event_name, data
               );
               webview.evaluate_script(&script)?;
           }
       }
   }
   ```

4. **JavaScript**: Receive event
   ```javascript
   window.addEventListener('update_data', (event) => {
       console.log(event.detail);  // {frame: 120}
   });
   ```

## Performance Optimizations

### 1. Lock-Free Data Structures

- **DashMap**: Concurrent HashMap without locks for callback storage
- **crossbeam-channel**: Lock-free MPMC channel for message queue

### 2. Batch Processing

Messages are processed in batches to reduce overhead:
```rust
let messages = message_queue.drain();  // Get all pending messages
for message in messages {
    // Process each message
}
```

### 3. Lazy Initialization

Event bridge script is injected only once during WebView creation.

## Thread Safety

### Python GIL Management

```rust
Python::with_gil(|py| {
    let callback = self.callbacks.get(&event_name)?;
    let args = PyTuple::new_bound(py, &[data_py]);
    callback.call1(py, args)?;
});
```

### Callback Storage

```rust
// Thread-safe concurrent HashMap
callbacks: Arc<DashMap<String, PyObject>>
```

### Message Queue

```rust
// Lock-free MPMC channel
queue: Sender<WebViewMessage>
receiver: Receiver<WebViewMessage>
```

## Error Handling

### JavaScript Errors

```javascript
try {
    window.ipc.postMessage(JSON.stringify(message));
} catch (e) {
    console.error('Failed to send event via IPC:', e);
}
```

### Rust Errors

```rust
match ipc_handler.handle_message(message) {
    Ok(_) => tracing::info!("Event handled successfully"),
    Err(e) => tracing::error!("Error handling event: {}", e),
}
```

### Python Errors

```python
try:
    callback(data)
except Exception as e:
    logger.error(f"Error in event handler: {e}")
```

## Migration Guide

### From Old Architecture

**Old** (Direct WebView IPC):
```rust
// src/webview/ipc.rs
webview.with_ipc_handler(|request| {
    // Handle IPC directly in WebView module
});
```

**New** (Modular IPC):
```rust
// src/ipc/handler.rs
let ipc_handler = Arc::new(IpcHandler::new());
webview_builder.with_ipc_handler(move |request| {
    ipc_handler.handle_message(message);
});
```

### Benefits

1. **Separation of Concerns**: IPC logic separated from WebView logic
2. **Reusability**: IPC components can be used in different contexts
3. **Testability**: Each component can be tested independently
4. **Maintainability**: Easier to understand and modify

## Quick Reference

### Register Event Handler (Python)

```python
@webview.on("event_name")
def handler(data):
    print(data)
```

### Emit Event (Python)

```python
webview.emit("event_name", {"key": "value"})
```

### Dispatch Event (JavaScript)

```javascript
window.dispatchEvent(new CustomEvent('event_name', {
    detail: { key: 'value' }
}));
```

### Listen for Event (JavaScript)

```javascript
window.addEventListener('event_name', (event) => {
    console.log(event.detail);
});
```

## Related Documentation

- `DCC_INTEGRATION_GUIDE.md` - How to integrate with DCC applications
- `TECHNICAL_DESIGN.md` - Overall technical design
- `ROADMAP.md` - Future improvements

