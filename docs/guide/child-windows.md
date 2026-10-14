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

When launched as a child window, these environment variables are used:

| Variable | Required | Description |
|----------|----------|-------------|
| `AURORAVIEW_PARENT_ID` | yes | Parent window identifier. **Its presence is what puts the process in child mode.** |
| `AURORAVIEW_PARENT_PORT` | yes | TCP port the parent listens on |
| `AURORAVIEW_CHILD_ID` | no | Unique child window ID, echoed on every frame the child sends |
| `AURORAVIEW_EXAMPLE_NAME` | no | Name of the example being run |
| `AURORAVIEW_PARENT_HWND` | no | Native handle of the parent window (Windows `HWND`, decimal or `0x`-prefixed hex). Embeds the window without passing `--parent-hwnd`. |
| `AURORAVIEW_CHILD_EXIT_ON_DISCONNECT` | no | Used by the `parent_ipc_child` demo: exit when the parent drops the link |

The same variables are read by both the Python child (`auroraview.child`) and
the Rust child (`auroraview_core::parent_ipc`, used by `auroraview run`), so a
host launches either one identically.

## Wire Protocol Specification

This section is the **normative, language-agnostic** definition of the
parent/child channel. It is implemented by `auroraview.child.ParentBridge`
(Python), `auroraview_core::parent_ipc` (Rust) and
`examples/parent_ipc/parent_host.ps1` (PowerShell). Any host — C#, C++,
PowerShell, Go — can implement it with nothing but a TCP socket and a JSON
parser.

### Transport

| Property | Value |
|----------|-------|
| Transport | TCP over loopback |
| Address | `127.0.0.1:<AURORAVIEW_PARENT_PORT>` |
| Roles | Parent **binds/accepts**; child **connects** |
| Framing | Newline-delimited JSON (NDJSON), one object per line |
| Encoding | UTF-8, **no BOM** |
| Frame limit | 1 MiB per frame |

### Framing rules

- Every frame is a single JSON object terminated by one `\n` (`0x0A`).
- A JSON string can never contain a raw `0x0A` (it is escaped as `\n`), so `0x0A`
  is an unambiguous delimiter.
- **Senders MUST NOT** write a UTF-8 BOM. **Receivers MUST tolerate** a leading
  BOM: several mainstream libraries (PowerShell / .NET `StreamWriter` built on
  `Encoding.UTF8`) emit one before the first frame, and rejecting it kills the
  channel on frame one.
- Receivers SHOULD tolerate `\r\n` by stripping a trailing `\r`.
- Blank lines MUST be skipped, not treated as end-of-stream.
- Frames may be split across TCP segments, and several frames may arrive in one
  segment; receivers MUST buffer and split on `0x0A`.

### Frame schema

```jsonc
{
  "type": "event",        // defaults to "event" when absent
  "event": "child:ready", // event frames only
  "data": { },            // payload; any JSON value
  "child_id": "child-1",  // child -> parent only
  "protocol": 1,          // hello / hello_ack only
  "accepted": true,       // hello_ack only
  "parent_id": "gallery", // hello / hello_ack only
  "example_name": "demo", // hello only
  "code": "bad_json",     // error only
  "message": "...",       // error only
  "fatal": false          // error only
}
```

`type` is optional **on receive** and defaults to `event`, so the pre-existing
Gallery framing (`{"event": ..., "data": ...}`) remains valid.

### Frame types

| `type` | Direction | Purpose |
|--------|-----------|---------|
| `hello` | child -> parent | Announce the child and its protocol version; opens the handshake |
| `hello_ack` | parent -> child | Accept (`"accepted": true`) or refuse the child |
| `event` | both | Application event (`event` + `data`) |
| `error` | both | Failure notification; `"fatal": true` means the sender is closing |
| `ping` / `pong` | both | Liveness probe |

Unknown `type` values MUST be ignored rather than treated as errors, so a newer
peer does not break an older one.

### Handshake

```text
parent                                     child
  |  bind 127.0.0.1:<port>                   |
  |  spawn child with AURORAVIEW_* env       |
  |<-----------------------------------------|  connect
  |<-----------------------------------------|  {"type":"hello","protocol":1,...}
  |<-----------------------------------------|  {"type":"event","event":"child:ready",...}
  |  {"type":"hello_ack","accepted":true} -> |
  |<-----------------------------------------|  {"type":"event","event":"child:hello",...}
```

1. The child connects and sends `hello`.
2. The child **immediately** sends `child:ready`, without waiting for the ack.
3. The parent replies `hello_ack` if it implements the protocol.
4. The effective protocol version is `min(child.protocol, parent.protocol)`.

The handshake is **optional and backward compatible by design**:

| Parent behaviour | Resulting state | Meaning |
|------------------|-----------------|---------|
| Sends `hello_ack` with `accepted: true` | `Acked` | Protocol-aware parent |
| Sends nothing (e.g. Gallery today) | `Legacy` | Fully usable; no negotiation |
| Sends `accepted: false` | `Rejected` | Child should fall back to standalone |

A child MUST NOT block startup waiting for `hello_ack`: a pre-protocol parent
never sends one, and waiting adds that timeout to every child's time to first
paint.

### Reserved events

| Event | Direction | Payload |
|-------|-----------|---------|
| `child:ready` | child -> parent | `{ child_id, example_name }` |
| `child:closing` | child -> parent | `{ child_id }` |
| `parent:command` | parent -> child | `{ command, args }` |

`parent:command` supports:

| `command` | `args` | Effect |
|-----------|--------|--------|
| `close` | – | Close the window and exit |
| `eval` | `{ js }` | Evaluate JavaScript in the page |
| `emit` | `{ event, data }` | Deliver an event to the page via `window.auroraview.trigger()` |

### Error codes

| `code` | Meaning | Fatal |
|--------|---------|-------|
| `unsupported_protocol` | Peer speaks a version this side cannot serve | yes |
| `bad_json` | Frame was not valid JSON, or not a JSON object | no |
| `frame_too_large` | Frame exceeded the 1 MiB limit | no |
| `unknown_type` | Unrecognised `type` | no |
| `handler_error` | An event handler failed | no |
| `internal_error` | Anything else | no |

Non-fatal errors leave the channel open: the receiver drops the offending frame,
reports it, and keeps reading.

### Disconnect semantics

- A graceful child shutdown sends `child:closing`, then closes the socket.
- Either side dropping the socket is a legitimate "I am done" signal, not an error.
- The child reports the drop through its disconnect callback; whether to exit is
  a **host decision** (see `--exit-on-parent-disconnect`).
- Reconnection is opt-in. When enabled, the child retries a bounded number of
  times, re-runs the handshake, and re-announces `child:ready`.

### Reference implementations

| Role | Language | Path |
|------|----------|------|
| Child | Python | `python/auroraview/child.py` |
| Child | Rust | `crates/auroraview-core/src/parent_ipc/` |
| Child (headless demo) | Rust | `crates/auroraview-core/examples/parent_ipc_child.rs` |
| Parent | PowerShell | `examples/parent_ipc/parent_host.ps1` |
| Parent | Python | `gallery/backend/child_manager.py` |

The PowerShell parent is the cross-language proof: it hand-writes JSON, performs
the handshake, round-trips events, and exercises disconnect — with no Python
involved.

```powershell
# Build the Rust child, then run the dialogue
cargo build -p auroraview-core --example parent_ipc_child
./examples/parent_ipc/parent_host.ps1

# Same, but drop the socket instead of sending `close`
./examples/parent_ipc/parent_host.ps1 -Mode Disconnect
```

## Embedding in a Host Window

Beyond IPC, a host can attach the AuroraView window to its own window. This is
what out-of-process embedding needs for Unity, Unreal, Qt and PowerPoint hosts.

```bash
# Child window: WS_CHILD, clipped to the parent's client area
auroraview run --url https://example.com --parent-hwnd 0x001A0B3C

# Owned window: separate top level, stays above and dies with its owner
auroraview run --url https://example.com --owner-hwnd 0x001A0B3C

# Or let the environment carry the handle
AURORAVIEW_PARENT_HWND=0x001A0B3C auroraview run --url https://example.com
```

| Flag | Style | Use for |
|------|-------|---------|
| `--parent-hwnd <HWND>` | `WS_CHILD` | Panels embedded inside a DCC viewport |
| `--owner-hwnd <HWND>` | Owned top level | Floating tool windows |

Handles accept decimal or `0x`-prefixed hexadecimal. The two flags are mutually
exclusive — a Win32 window is either a child or an owned top level — and
`--parent-hwnd` wins, since a child window is already destroyed with its parent.

Both flags are accepted on every platform so host scripts stay portable, but
reparenting into a foreign window is Windows-only; elsewhere they are logged and
ignored.

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
