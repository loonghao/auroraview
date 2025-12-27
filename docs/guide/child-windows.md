# Child Window System

AuroraView provides a unified child window system that allows examples and applications to run either standalone or as child windows of a parent application (like Gallery).

## Overview

The child window system enables:

- **Dual-mode execution**: Examples can run independently or as sub-windows
- **Automatic mode detection**: Via environment variables
- **Parent-child communication**: Full IPC support between windows
- **Seamless integration**: No code changes needed for basic usage

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Gallery (Parent)                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐     ┌─────────────────┐                    │
│  │ ChildWindowManager │◄──►│   IPC Server    │                    │
│  └────────┬────────┘     └────────┬────────┘                    │
│           │                       │                              │
│           │  launch_example()     │  TCP Socket                  │
│           ▼                       ▼                              │
├───────────┴───────────────────────┴─────────────────────────────┤
│                    Environment Variables                         │
│  AURORAVIEW_PARENT_ID, AURORAVIEW_PARENT_PORT, etc.             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Example 1  │  │  Example 2  │  │  Example 3  │              │
│  │ (Child Mode)│  │ (Child Mode)│  │ (Child Mode)│              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                   ┌──────▼──────┐                                │
│                   │ ParentBridge │                                │
│                   │  (IPC Client)│                                │
│                   └─────────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Basic Usage with ChildContext

The simplest way to create a child-aware application:

```python
from auroraview import ChildContext

with ChildContext() as ctx:
    webview = ctx.create_webview(
        title="My Example",
        html="<h1>Hello World</h1>",
        width=800,
        height=600
    )
    
    # Check if running as child window
    if ctx.is_child:
        print(f"Running as child of: {ctx.parent_id}")
        # Send message to parent
        ctx.emit_to_parent("hello", {"message": "Hi from child!"})
    else:
        print("Running standalone")
    
    webview.show()
```

### Mode Detection Functions

```python
from auroraview import is_child_mode, get_parent_id, get_child_id

# Check if running as child window
if is_child_mode():
    print(f"Parent ID: {get_parent_id()}")
    print(f"Child ID: {get_child_id()}")
else:
    print("Running standalone")
```

## Environment Variables

When launched as a child window, these environment variables are set:

| Variable | Description |
|----------|-------------|
| `AURORAVIEW_PARENT_ID` | Parent window identifier |
| `AURORAVIEW_PARENT_PORT` | IPC communication port |
| `AURORAVIEW_CHILD_ID` | Unique child window ID |
| `AURORAVIEW_EXAMPLE_NAME` | Name of the example being run |

## API Reference

### ChildContext

Context manager for child-aware WebView creation.

```python
class ChildContext:
    def __init__(self):
        """Initialize child context with automatic mode detection."""
        
    @property
    def is_child(self) -> bool:
        """Check if running in child mode."""
        
    @property
    def parent_id(self) -> Optional[str]:
        """Get parent window ID if in child mode."""
        
    @property
    def child_id(self) -> Optional[str]:
        """Get this window's child ID if in child mode."""
        
    def create_webview(self, **kwargs) -> WebView:
        """Create a WebView with appropriate settings for current mode."""
        
    def emit_to_parent(self, event: str, data: Any) -> bool:
        """Send event to parent window (only works in child mode)."""
        
    def on_parent_message(self, handler: Callable[[str, Any], None]):
        """Register handler for messages from parent."""
```

### ChildInfo

Information about a child window.

```python
@dataclass
class ChildInfo:
    child_id: str          # Unique child identifier
    example_name: str      # Name of the example
    process_id: int        # OS process ID
    port: int              # IPC port
    started_at: float      # Start timestamp
```

### Helper Functions

```python
def is_child_mode() -> bool:
    """Check if running as a child window."""
    
def get_parent_id() -> Optional[str]:
    """Get parent window ID, or None if standalone."""
    
def get_child_id() -> Optional[str]:
    """Get this window's child ID, or None if standalone."""
    
def run_example(example_path: str, **kwargs) -> Optional[str]:
    """Launch an example as a child window. Returns child_id."""
```

## Parent-Child Communication

### From Child to Parent

```python
# In child window
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    # Send event to parent
    ctx.emit_to_parent("status_update", {
        "progress": 50,
        "message": "Processing..."
    })
```

### From Parent to Child

```python
# In parent (e.g., Gallery)
from gallery.backend.child_manager import get_manager

manager = get_manager()

# Send message to specific child
manager.send_to_child(child_id, "parent:command", {
    "action": "refresh"
})

# Broadcast to all children
manager.broadcast("parent:notification", {
    "message": "Settings changed"
})
```

### Handling Messages

```python
# In child window
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    @ctx.on_parent_message
    def handle_parent_message(event: str, data: dict):
        if event == "parent:command":
            if data.get("action") == "refresh":
                # Handle refresh command
                pass
```

## Gallery Integration

### JavaScript API

When running examples from Gallery, use these APIs:

```javascript
// Launch example as child window
const childId = await auroraview.api.launch_example_as_child("child_window_demo");

// Get all active child windows
const children = await auroraview.api.get_children();
// Returns: [{ child_id, example_name, process_id, port, started_at }, ...]

// Send message to child
await auroraview.api.send_to_child(childId, "parent:message", { data: "hello" });

// Broadcast to all children
await auroraview.api.broadcast_to_children("parent:notification", { message: "Hi all" });

// Close specific child
await auroraview.api.close_child(childId);

// Close all children
await auroraview.api.close_all_children();
```

### Listening for Child Events

```javascript
// In Gallery frontend
auroraview.on('child:connected', (data) => {
    console.log('Child connected:', data.child_id, data.example_name);
});

auroraview.on('child:disconnected', (data) => {
    console.log('Child disconnected:', data.child_id);
});

auroraview.on('child:message', (data) => {
    console.log('Message from child:', data.child_id, data.event, data.data);
});
```

## Complete Example

Here's a complete example that works both standalone and as a child window:

```python
"""Child-aware example that adapts to its execution context."""
from auroraview import ChildContext

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Child Window Demo</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .mode { padding: 10px; border-radius: 5px; margin-bottom: 20px; }
        .standalone { background: #e3f2fd; }
        .child { background: #e8f5e9; }
        button { padding: 10px 20px; margin: 5px; cursor: pointer; }
    </style>
</head>
<body>
    <div id="mode" class="mode"></div>
    <div id="messages"></div>
    <button onclick="sendToParent()">Send to Parent</button>
    
    <script>
        const isChild = window.AURORAVIEW_IS_CHILD || false;
        const modeDiv = document.getElementById('mode');
        
        if (isChild) {
            modeDiv.className = 'mode child';
            modeDiv.innerHTML = '<h2>Running as Child Window</h2>';
        } else {
            modeDiv.className = 'mode standalone';
            modeDiv.innerHTML = '<h2>Running Standalone</h2>';
        }
        
        function sendToParent() {
            if (isChild && window.auroraview) {
                auroraview.api.notify_parent({
                    event: 'button_clicked',
                    data: { timestamp: Date.now() }
                });
            }
        }
        
        // Listen for parent messages
        if (window.auroraview) {
            auroraview.on('parent:message', (data) => {
                const div = document.getElementById('messages');
                div.innerHTML += `<p>From parent: ${JSON.stringify(data)}</p>`;
            });
        }
    </script>
</body>
</html>
"""

def main():
    with ChildContext() as ctx:
        webview = ctx.create_webview(
            title="Child Window Demo",
            html=HTML,
            width=600,
            height=400
        )
        
        # Inject mode information
        webview.eval_js(f"window.AURORAVIEW_IS_CHILD = {str(ctx.is_child).lower()};")
        
        # Handle messages from parent
        if ctx.is_child:
            @ctx.on_parent_message
            def on_parent_msg(event, data):
                webview.emit(event, data)
        
        # API for child to notify parent
        @webview.bind_call("api.notify_parent")
        def notify_parent(event: str, data: dict):
            if ctx.is_child:
                ctx.emit_to_parent(event, data)
                return {"ok": True}
            return {"ok": False, "reason": "Not in child mode"}
        
        webview.show()

if __name__ == "__main__":
    main()
```

## Best Practices

### 1. Always Use ChildContext

```python
# Good: Uses context manager
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    webview.show()

# Avoid: Manual mode detection
if os.environ.get("AURORAVIEW_PARENT_ID"):
    # Manual setup...
```

### 2. Graceful Degradation

Design your app to work in both modes:

```python
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    # Features that only work in child mode
    if ctx.is_child:
        ctx.emit_to_parent("ready", {"version": "1.0"})
    
    # Core functionality works in both modes
    @webview.bind_call("api.process")
    def process(data):
        return do_processing(data)
    
    webview.show()
```

### 3. Clean Shutdown

```python
with ChildContext() as ctx:
    webview = ctx.create_webview(...)
    
    @webview.on_close
    def on_close():
        if ctx.is_child:
            ctx.emit_to_parent("closing", {"child_id": ctx.child_id})
    
    webview.show()
```

## Comparison with Rust Child Windows

| Feature | Python Child System | Rust `child_window.rs` |
|---------|---------------------|------------------------|
| Purpose | Python examples as sub-windows | JS `window.open()` handling |
| Communication | Full IPC | None |
| Configuration | Full WebView options | URL/size only |
| API Binding | Supported | Not supported |
| Mode Detection | Automatic | N/A |

The two systems are **complementary**, not replacements for each other.
