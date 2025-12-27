# Bidirectional Communication

AuroraView provides a complete IPC (Inter-Process Communication) system for bidirectional communication between Python and JavaScript.

## Architecture Overview

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

## Communication API Overview

| Direction | JavaScript API | Python API | Use Case |
|-----------|---------------|------------|----------|
| JS → Python | `auroraview.call(method, params)` | `@webview.bind_call` | RPC with return value |
| JS → Python | `auroraview.send_event(event, data)` | `@webview.on(event)` | Fire-and-forget events |
| Python → JS | - | `webview.emit(event, data)` | Push notifications |
| JS only | `auroraview.on(event, handler)` | - | Receive Python events |

## Python → JavaScript

### Emitting Events

```python
# Python side
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})
webview.emit("selection_changed", {"items": ["mesh1", "mesh2"]})
```

```javascript
// JavaScript side
auroraview.on('update_data', (data) => {
    console.log('Frame:', data.frame);
    console.log('Objects:', data.objects);
});

auroraview.on('selection_changed', (data) => {
    highlightItems(data.items);
});
```

### Message Flow

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

3. **Event Loop**: Process queue and execute script
   ```rust
   let script = format!(
       "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
       event_name, data
   );
   webview.evaluate_script(&script)?;
   ```

4. **JavaScript**: Receive event
   ```javascript
   window.addEventListener('update_data', (event) => {
       console.log(event.detail);  // {frame: 120}
   });
   ```

## JavaScript → Python

### Events (Fire-and-Forget)

```javascript
// JavaScript side
auroraview.send_event('export_scene', {
    path: '/path/to/export.fbx',
    format: 'fbx'
});
```

```python
# Python side
@webview.on("export_scene")
def handle_export(data):
    print(f"Exporting to: {data['path']}")
    # Your export logic here
```

### RPC Calls (With Return Value)

```javascript
// JavaScript side
const hierarchy = await auroraview.call('api.get_hierarchy', { root: 'scene' });
console.log('Scene hierarchy:', hierarchy);

const result = await auroraview.call('api.rename_object', {
    old_name: 'cube1',
    new_name: 'hero_cube'
});
```

```python
# Python side
@webview.bind_call("api.get_hierarchy")
def get_hierarchy(root=None):
    return {"children": ["group1", "mesh_cube"], "count": 2}

@webview.bind_call("api.rename_object")
def rename_object(old_name, new_name):
    # Perform rename
    return {"ok": True, "old": old_name, "new": new_name}
```

### Message Flow

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

## API Object Pattern

The simplest way to expose Python methods to JavaScript:

```python
from auroraview import AuroraView

class MyAPI:
    def get_data(self) -> dict:
        """Called from JS: await auroraview.api.get_data()"""
        return {"items": [1, 2, 3], "count": 3}

    def save_file(self, path: str = "", content: str = "") -> dict:
        """Called from JS: await auroraview.api.save_file({path: "...", content: "..."})"""
        with open(path, "w") as f:
            f.write(content)
        return {"ok": True, "path": path}

# Create WebView with API auto-binding
view = AuroraView(url="http://localhost:3000", api=MyAPI())
view.show()
```

```javascript
// JavaScript side
const data = await auroraview.api.get_data();
console.log(data.items);  // [1, 2, 3]

const result = await auroraview.api.save_file({
    path: "/tmp/test.txt",
    content: "Hello"
});
```

## Performance Optimizations

### Lock-Free Data Structures

- **DashMap**: Concurrent HashMap without locks for callback storage
- **crossbeam-channel**: Lock-free MPMC channel for message queue

### Batch Processing

Messages are processed in batches to reduce overhead:

```rust
let messages = message_queue.drain();  // Get all pending messages
for message in messages {
    // Process each message
}
```

### Thread Safety

**Python GIL Management**:
```rust
Python::with_gil(|py| {
    let callback = self.callbacks.get(&event_name)?;
    let args = PyTuple::new_bound(py, &[data_py]);
    callback.call1(py, args)?;
});
```

**Callback Storage**:
```rust
// Thread-safe concurrent HashMap
callbacks: Arc<DashMap<String, PyObject>>
```

## Common Mistakes

::: danger Don't Use These
```javascript
// WRONG: trigger() is JS-local only, won't reach Python
auroraview.trigger('my_event', data);

// WRONG: dispatchEvent is browser API, won't reach Python
window.dispatchEvent(new CustomEvent('my_event', {detail: data}));
```
:::

::: tip Correct Usage
```javascript
// CORRECT: use send_event() for fire-and-forget
auroraview.send_event('my_event', data);

// CORRECT: use call() for request-response
const result = await auroraview.call('api.my_method', data);
```
:::

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
auroraview.send_event('event_name', { key: 'value' });
```

### Listen for Event (JavaScript)

```javascript
auroraview.on('event_name', (data) => {
    console.log(data);
});
```

## Parent-Child Window Communication

For multi-window scenarios where examples run as child windows of a parent application (like Gallery), AuroraView provides a dedicated IPC system.

### Child to Parent

```python
from auroraview import ChildContext

with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    # Send event to parent window
    if ctx.is_child:
        ctx.emit_to_parent("status_update", {
            "progress": 50,
            "message": "Processing..."
        })
```

### Parent to Child

```python
from gallery.backend.child_manager import get_manager

manager = get_manager()

# Send to specific child
manager.send_to_child(child_id, "parent:command", {"action": "refresh"})

# Broadcast to all children
manager.broadcast("parent:notification", {"message": "Settings changed"})
```

### Handling Parent Messages

```python
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    @ctx.on_parent_message
    def handle_parent_message(event: str, data: dict):
        if event == "parent:command":
            # Handle command
            pass
```

[Full Child Window Guide →](/guide/child-windows)
