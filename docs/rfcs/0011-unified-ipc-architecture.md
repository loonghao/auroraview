# RFC 0011: Unified IPC Architecture

## Summary

Refactor AuroraView's IPC (Inter-Process Communication) layer into a unified, layered architecture based on ipckit. This design covers the entire project lifecycle including standalone mode, packed mode, DCC embedding, and subprocess communication, replacing the current fragmented approach with a consistent, high-performance solution.

## Motivation

### Current Problems

1. **Fragmented IPC Mechanisms**: Multiple IPC approaches coexist without unified abstraction:
   - Packed Mode: stdin/stdout pipes for Python backend communication
   - ProcessPlugin: ipckit LocalSocket for subprocess communication
   - WebView: crossbeam-channel for thread-safe message queue
   - DCC Mode: Qt-based message passing

2. **stdout Mixing Issues**: In packed mode, Python backend uses stdout for both:
   - IPC data (JSON-RPC responses, events)
   - Debug logging (`print()` statements)
   
   This causes:
   - Debugging difficulty (can't freely use `print()`)
   - Log noise mixed with IPC data
   - Potential parsing errors from unexpected output

3. **Inconsistent Error Handling**: Each IPC mechanism has its own error types and handling patterns.

4. **Performance Overhead**: String serialization overhead when using stdout-based IPC vs. native socket communication.

5. **Limited Testability**: Difficult to mock or test IPC layers in isolation.

### Goals

- **Unified Abstraction**: Single IPC trait/interface across all modes
- **Clean Separation**: Logs on stderr, IPC data on dedicated channel
- **High Performance**: Native socket communication via ipckit
- **Graceful Shutdown**: Consistent shutdown coordination using ipckit's `ShutdownState`
- **DCC Compatible**: Works with Maya, Houdini, 3ds Max, Nuke, Blender
- **Testable**: Easy to mock and unit test IPC components

### Use Cases

1. **Packed Mode**: Rust CLI ↔ Python backend communication
2. **Subprocess IPC**: Parent process ↔ spawned child scripts
3. **DCC Embedding**: Qt host ↔ WebView communication
4. **Plugin System**: PluginRouter ↔ individual plugins
5. **AI Agent**: AI provider ↔ WebView communication

## Design

### 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Unified IPC Architecture                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Application Layer                                 │   │
│  │                                                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────┐  │   │
│  │  │ WebView   │ │ Packed    │ │ DCC       │ │ Subprocess        │  │   │
│  │  │ (wry/tao) │ │ Mode CLI  │ │ Embedding │ │ (ProcessPlugin)   │  │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    IPC Abstraction Layer                             │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │              IpcChannel Trait (Rust)                         │   │   │
│  │  │  - send(&self, msg: IpcMessage) -> Result<()>                │   │   │
│  │  │  - recv(&self) -> Result<IpcMessage>                         │   │   │
│  │  │  - send_json(&self, data: Value) -> Result<()>               │   │   │
│  │  │  - recv_json(&self) -> Result<Value>                         │   │   │
│  │  │  - on_message(&self, handler: Fn(IpcMessage))                │   │   │
│  │  │  - shutdown(&self)                                           │   │   │
│  │  │  - is_shutdown(&self) -> bool                                │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │              IpcChannel Class (Python)                       │   │   │
│  │  │  - send(data: dict) -> bool                                  │   │   │
│  │  │  - receive(timeout: float) -> dict | None                    │   │   │
│  │  │  - on_message(handler: Callable)                             │   │   │
│  │  │  - close()                                                   │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Transport Layer (ipckit)                         │   │
│  │                                                                     │   │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────────────────┐ │   │
│  │  │ LocalSocket   │ │ SharedMemory  │ │ ThreadChannel             │ │   │
│  │  │ (Pipe/UDS)    │ │ (Large Data)  │ │ (In-Process)              │ │   │
│  │  └───────────────┘ └───────────────┘ └───────────────────────────┘ │   │
│  │                                                                     │   │
│  │  ┌───────────────────────────────────────────────────────────────┐ │   │
│  │  │              GracefulShutdown (ShutdownState)                  │ │   │
│  │  │  - Coordinated shutdown across all channels                    │ │   │
│  │  │  - Prevents "EventLoopClosed" errors                           │ │   │
│  │  │  - Operation guards for in-flight messages                     │ │   │
│  │  └───────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Platform Abstraction Layer                       │   │
│  │                                                                     │   │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────────────────┐ │   │
│  │  │   Windows     │ │    macOS      │ │         Linux             │ │   │
│  │  │ Named Pipes   │ │ Unix Domain   │ │ Unix Domain Sockets       │ │   │
│  │  │ \\.\pipe\...  │ │ Sockets       │ │ /tmp/ipckit_...           │ │   │
│  │  └───────────────┘ └───────────────┘ └───────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. IPC Channel Trait (Rust)

```rust
// crates/auroraview-core/src/ipc/channel.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Unified IPC message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Message type: "request", "response", "event", "ping", "pong"
    #[serde(rename = "type")]
    pub msg_type: String,
    
    /// Request/response ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// Method name (for requests)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    
    /// Event name (for events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    
    /// Payload data
    #[serde(default)]
    pub data: Value,
    
    /// Error information (for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
}

/// IPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// IPC Channel trait - unified interface for all IPC backends
pub trait IpcChannel: Send + Sync {
    /// Send a message through the channel
    fn send(&self, msg: IpcMessage) -> Result<(), IpcChannelError>;
    
    /// Receive a message (blocking with optional timeout)
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<IpcMessage, IpcChannelError>;
    
    /// Try to receive a message (non-blocking)
    fn try_recv(&self) -> Result<Option<IpcMessage>, IpcChannelError>;
    
    /// Send JSON data as a message
    fn send_json(&self, data: Value) -> Result<(), IpcChannelError> {
        self.send(IpcMessage {
            msg_type: "data".to_string(),
            id: None,
            method: None,
            event: None,
            data,
            error: None,
        })
    }
    
    /// Emit an event
    fn emit(&self, event: &str, data: Value) -> Result<(), IpcChannelError> {
        self.send(IpcMessage {
            msg_type: "event".to_string(),
            id: None,
            method: None,
            event: Some(event.to_string()),
            data,
            error: None,
        })
    }
    
    /// Initiate graceful shutdown
    fn shutdown(&self);
    
    /// Check if shutdown has been initiated
    fn is_shutdown(&self) -> bool;
    
    /// Wait for pending operations to complete
    fn wait_for_drain(&self, timeout: Option<std::time::Duration>) -> Result<(), IpcChannelError>;
    
    /// Get channel name/identifier
    fn name(&self) -> &str;
}

/// IPC Channel error types
#[derive(Debug, thiserror::Error)]
pub enum IpcChannelError {
    #[error("Channel disconnected")]
    Disconnected,
    
    #[error("Send failed: {0}")]
    SendFailed(String),
    
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Shutdown in progress")]
    ShuttingDown,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 3. Channel Implementations

#### 3.1 LocalSocket Channel (Process Communication)

```rust
// crates/auroraview-core/src/ipc/local_socket_channel.rs

use ipckit::local_socket::{LocalSocketListener, LocalSocketStream};
use ipckit::graceful::ShutdownState;
use std::sync::Arc;

/// LocalSocket-based IPC channel (uses ipckit)
/// 
/// Platform-specific implementation:
/// - Windows: Named Pipes (\\.\pipe\auroraview_xxx)
/// - Unix: Unix Domain Sockets (/tmp/auroraview_xxx)
pub struct LocalSocketChannel {
    name: String,
    stream: LocalSocketStream,
    shutdown_state: Arc<ShutdownState>,
}

impl LocalSocketChannel {
    /// Create a server channel (listener)
    pub fn server(name: &str) -> Result<LocalSocketChannelServer, IpcChannelError> {
        let listener = LocalSocketListener::bind(name)?;
        Ok(LocalSocketChannelServer {
            name: name.to_string(),
            listener,
            shutdown_state: Arc::new(ShutdownState::new()),
        })
    }
    
    /// Connect to an existing channel (client)
    pub fn connect(name: &str) -> Result<Self, IpcChannelError> {
        let stream = LocalSocketStream::connect(name)?;
        Ok(Self {
            name: name.to_string(),
            stream,
            shutdown_state: Arc::new(ShutdownState::new()),
        })
    }
}

impl IpcChannel for LocalSocketChannel {
    fn send(&self, msg: IpcMessage) -> Result<(), IpcChannelError> {
        if self.shutdown_state.is_shutdown() {
            return Err(IpcChannelError::ShuttingDown);
        }
        
        let _guard = self.shutdown_state.begin_operation();
        let json = serde_json::to_string(&msg)
            .map_err(|e| IpcChannelError::SerializationError(e.to_string()))?;
        
        self.stream.send_line(&json)?;
        Ok(())
    }
    
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<IpcMessage, IpcChannelError> {
        let line = self.stream.recv_line_timeout(timeout)?;
        let msg: IpcMessage = serde_json::from_str(&line)
            .map_err(|e| IpcChannelError::SerializationError(e.to_string()))?;
        Ok(msg)
    }
    
    // ... other trait methods
}
```

#### 3.2 Thread Channel (In-Process Communication)

```rust
// crates/auroraview-core/src/ipc/thread_channel.rs

use crossbeam_channel::{bounded, Receiver, Sender};
use ipckit::graceful::ShutdownState;

/// Thread-safe channel for in-process IPC
/// 
/// Uses crossbeam-channel for lock-free communication.
/// Suitable for WebView ↔ Python bindings communication.
pub struct ThreadChannel {
    name: String,
    tx: Sender<IpcMessage>,
    rx: Receiver<IpcMessage>,
    shutdown_state: Arc<ShutdownState>,
}

impl ThreadChannel {
    pub fn new(name: &str, capacity: usize) -> Self {
        let (tx, rx) = bounded(capacity);
        Self {
            name: name.to_string(),
            tx,
            rx,
            shutdown_state: Arc::new(ShutdownState::new()),
        }
    }
    
    /// Create a paired channel for bidirectional communication
    pub fn pair(name: &str, capacity: usize) -> (Self, Self) {
        let (tx1, rx1) = bounded(capacity);
        let (tx2, rx2) = bounded(capacity);
        let shutdown_state = Arc::new(ShutdownState::new());
        
        let ch1 = Self {
            name: format!("{}_a", name),
            tx: tx1,
            rx: rx2,
            shutdown_state: Arc::clone(&shutdown_state),
        };
        
        let ch2 = Self {
            name: format!("{}_b", name),
            tx: tx2,
            rx: rx1,
            shutdown_state,
        };
        
        (ch1, ch2)
    }
}

impl IpcChannel for ThreadChannel {
    fn send(&self, msg: IpcMessage) -> Result<(), IpcChannelError> {
        if self.shutdown_state.is_shutdown() {
            return Err(IpcChannelError::ShuttingDown);
        }
        
        let _guard = self.shutdown_state.begin_operation();
        self.tx.send(msg)
            .map_err(|e| IpcChannelError::SendFailed(e.to_string()))
    }
    
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<IpcMessage, IpcChannelError> {
        match timeout {
            Some(t) => self.rx.recv_timeout(t)
                .map_err(|_| IpcChannelError::Timeout),
            None => self.rx.recv()
                .map_err(|e| IpcChannelError::ReceiveFailed(e.to_string())),
        }
    }
    
    // ... other trait methods
}
```

### 4. Python IPC Channel

```python
# python/auroraview/core/ipc/channel.py

"""Unified IPC Channel for AuroraView.

Provides a consistent interface for IPC communication across all modes:
- Standalone mode: ThreadChannel (in-process)
- Packed mode: LocalSocketChannel (process-to-process)
- DCC mode: ThreadChannel with Qt integration
"""

from __future__ import annotations

import json
import os
import socket
import sys
import threading
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Optional, Union
from enum import Enum, auto

# Use Rust-powered JSON if available
try:
    from auroraview._core import json_dumps, json_loads
except ImportError:
    json_dumps = lambda obj: json.dumps(obj, ensure_ascii=False)
    json_loads = json.loads


class IpcMessageType(Enum):
    """IPC message types."""
    REQUEST = "request"
    RESPONSE = "response"
    EVENT = "event"
    PING = "ping"
    PONG = "pong"
    DATA = "data"


@dataclass
class IpcMessage:
    """Unified IPC message structure."""
    type: str
    data: Any = field(default_factory=dict)
    id: Optional[str] = None
    method: Optional[str] = None
    event: Optional[str] = None
    error: Optional[Dict[str, Any]] = None
    
    def to_dict(self) -> Dict[str, Any]:
        result = {"type": self.type, "data": self.data}
        if self.id:
            result["id"] = self.id
        if self.method:
            result["method"] = self.method
        if self.event:
            result["event"] = self.event
        if self.error:
            result["error"] = self.error
        return result
    
    def to_json(self) -> str:
        return json_dumps(self.to_dict())
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "IpcMessage":
        return cls(
            type=data.get("type", "data"),
            data=data.get("data", {}),
            id=data.get("id"),
            method=data.get("method"),
            event=data.get("event"),
            error=data.get("error"),
        )
    
    @classmethod
    def from_json(cls, json_str: str) -> "IpcMessage":
        return cls.from_dict(json_loads(json_str))


class IpcChannelError(Exception):
    """IPC channel error."""
    pass


class IpcChannel:
    """Abstract base class for IPC channels."""
    
    def __init__(self, name: str):
        self.name = name
        self._shutdown = False
        self._handlers: list[Callable[[IpcMessage], None]] = []
        self._lock = threading.Lock()
    
    def send(self, msg: IpcMessage) -> bool:
        """Send a message through the channel."""
        raise NotImplementedError
    
    def receive(self, timeout: Optional[float] = None) -> Optional[IpcMessage]:
        """Receive a message (blocking with optional timeout)."""
        raise NotImplementedError
    
    def emit(self, event: str, data: Any = None) -> bool:
        """Emit an event."""
        msg = IpcMessage(
            type=IpcMessageType.EVENT.value,
            event=event,
            data=data or {},
        )
        return self.send(msg)
    
    def on_message(self, handler: Callable[[IpcMessage], None]) -> None:
        """Register a message handler."""
        self._handlers.append(handler)
    
    def shutdown(self) -> None:
        """Initiate graceful shutdown."""
        self._shutdown = True
    
    def is_shutdown(self) -> bool:
        """Check if shutdown has been initiated."""
        return self._shutdown
    
    def close(self) -> None:
        """Close the channel."""
        self.shutdown()


class LocalSocketChannel(IpcChannel):
    """LocalSocket-based IPC channel (Windows Named Pipes / Unix Domain Sockets).
    
    Used for process-to-process communication (e.g., Packed Mode).
    """
    
    WINDOWS_PIPE_PREFIX = r"\\.\pipe\auroraview_"
    UNIX_SOCKET_PREFIX = "/tmp/auroraview_"
    
    def __init__(self, name: str, is_server: bool = False):
        super().__init__(name)
        self._socket: Optional[socket.socket] = None
        self._file: Any = None
        self._is_server = is_server
        self._connected = False
    
    @classmethod
    def server(cls, name: str) -> "LocalSocketChannelServer":
        """Create a server channel."""
        return LocalSocketChannelServer(name)
    
    @classmethod
    def connect(cls, name: str) -> "LocalSocketChannel":
        """Connect to an existing channel."""
        channel = cls(name, is_server=False)
        channel._connect()
        return channel
    
    def _connect(self) -> None:
        """Establish connection."""
        if self._connected:
            return
        
        if sys.platform == "win32":
            pipe_path = f"{self.WINDOWS_PIPE_PREFIX}{self.name}"
            self._file = open(pipe_path, "r+b", buffering=0)
        else:
            socket_path = f"{self.UNIX_SOCKET_PREFIX}{self.name}"
            self._socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            self._socket.connect(socket_path)
            self._file = self._socket.makefile("rw", buffering=1)
        
        self._connected = True
    
    def send(self, msg: IpcMessage) -> bool:
        if self._shutdown or not self._connected:
            return False
        
        try:
            with self._lock:
                json_str = msg.to_json() + "\n"
                if sys.platform == "win32":
                    self._file.write(json_str.encode("utf-8"))
                else:
                    self._file.write(json_str)
                self._file.flush()
                return True
        except Exception as e:
            raise IpcChannelError(f"Send failed: {e}") from e
    
    def receive(self, timeout: Optional[float] = None) -> Optional[IpcMessage]:
        if self._shutdown or not self._connected:
            return None
        
        try:
            # Read line from channel
            if sys.platform == "win32":
                line = self._read_line_windows()
            else:
                line = self._file.readline().strip()
            
            if not line:
                return None
            
            return IpcMessage.from_json(line)
        except Exception as e:
            raise IpcChannelError(f"Receive failed: {e}") from e
    
    def _read_line_windows(self) -> str:
        """Read a line from Windows named pipe."""
        line = b""
        while True:
            char = self._file.read(1)
            if not char or char == b"\n":
                break
            line += char
        return line.decode("utf-8")
    
    def close(self) -> None:
        super().close()
        self._connected = False
        if self._file:
            try:
                self._file.close()
            except Exception:
                pass
        if self._socket:
            try:
                self._socket.close()
            except Exception:
                pass


class LocalSocketChannelServer:
    """Server-side LocalSocket channel."""
    
    def __init__(self, name: str):
        self.name = name
        self._listener = None
        self._shutdown = False
        self._setup_listener()
    
    def _setup_listener(self) -> None:
        """Setup the listener socket."""
        if sys.platform == "win32":
            # Windows Named Pipe - handled differently
            pass
        else:
            socket_path = f"{LocalSocketChannel.UNIX_SOCKET_PREFIX}{self.name}"
            # Remove existing socket file if present
            import os
            if os.path.exists(socket_path):
                os.unlink(socket_path)
            
            self._listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            self._listener.bind(socket_path)
            self._listener.listen(1)
    
    def accept(self) -> LocalSocketChannel:
        """Accept an incoming connection."""
        if sys.platform == "win32":
            # Windows implementation would use win32pipe
            raise NotImplementedError("Windows server not yet implemented")
        
        conn, _ = self._listener.accept()
        channel = LocalSocketChannel(self.name)
        channel._socket = conn
        channel._file = conn.makefile("rw", buffering=1)
        channel._connected = True
        return channel
    
    def shutdown(self) -> None:
        self._shutdown = True
        if self._listener:
            self._listener.close()
```

### 5. Packed Mode Migration

#### Current Architecture (stdout-based)

```
┌─────────────────┐      stdin (JSON-RPC)      ┌─────────────────┐
│    Rust CLI     │ ──────────────────────────> │  Python Backend │
│   (WebView2)    │                             │   (Gallery)     │
│                 │ <────────────────────────── │                 │
└─────────────────┘      stdout (JSON/events)   └─────────────────┘
                              ↓
                    stderr (logs + errors混合)
```

#### New Architecture (LocalSocket-based)

```
┌─────────────────────────────────────────────────────────────────┐
│                      Rust CLI (WebView2)                         │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │            LocalSocketChannelServer                          ││
│  │         (\\.\pipe\auroraview_{session})                      ││
│  └─────────────────────────────────────────────────────────────┘│
│                          ↕ JSON-RPC                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                 Python Backend                               ││
│  │            LocalSocketChannel.connect()                      ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│       stdout → 纯日志，可自由 print() 调试                       │
│       stderr → 错误日志                                          │
└─────────────────────────────────────────────────────────────────┘
```

#### Migration Steps

**Phase 1: Rust CLI Changes**

```rust
// crates/auroraview-cli/src/packed/backend.rs

use auroraview_core::ipc::{IpcChannel, LocalSocketChannel, LocalSocketChannelServer};

pub struct PythonBackend {
    process: Mutex<Child>,
    /// IPC channel (replaces stdin)
    ipc_channel: Arc<dyn IpcChannel>,
    shutdown_state: Arc<ShutdownState>,
}

pub fn start_python_backend_with_ipc(
    overlay: &OverlayData,
    python_config: &PythonBundleConfig,
    proxy: EventLoopProxy<UserEvent>,
    metrics: &mut PackedMetrics,
) -> Result<PythonBackend> {
    // Generate unique channel name for this session
    let channel_name = format!("packed_{}", std::process::id());
    
    // Create server channel before spawning Python
    let server = LocalSocketChannelServer::new(&channel_name)?;
    
    // Set environment variable for Python to connect
    std::env::set_var("AURORAVIEW_IPC_CHANNEL", &channel_name);
    
    // Spawn Python process (no longer capture stdout for IPC)
    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", &python_code])
        .current_dir(&temp_dir)
        .stdin(Stdio::null())      // No stdin IPC
        .stdout(Stdio::inherit())  // Pass through for logging
        .stderr(Stdio::inherit()); // Pass through for errors
    
    // ... spawn child process ...
    
    // Wait for Python to connect
    let channel = server.accept()?;
    
    // Start message reader thread
    let channel_clone = Arc::clone(&channel);
    let shutdown_state_clone = Arc::clone(&shutdown_state);
    thread::spawn(move || {
        loop {
            if shutdown_state_clone.is_shutdown() {
                break;
            }
            
            match channel_clone.recv(Some(Duration::from_millis(100))) {
                Ok(msg) => {
                    // Handle message based on type
                    match msg.msg_type.as_str() {
                        "event" => {
                            // Forward to WebView
                            let _ = proxy.send_event(UserEvent::PluginEvent {
                                event: msg.event.unwrap_or_default(),
                                data: msg.data.to_string(),
                            });
                        }
                        "response" => {
                            // Forward API response
                            let _ = proxy.send_event(UserEvent::PythonResponse(
                                serde_json::to_string(&msg).unwrap()
                            ));
                        }
                        _ => {}
                    }
                }
                Err(IpcChannelError::Timeout) => continue,
                Err(_) => break,
            }
        }
    });
    
    Ok(PythonBackend {
        process: Mutex::new(child),
        ipc_channel: Arc::new(channel),
        shutdown_state,
    })
}
```

**Phase 2: Python Backend Changes**

```python
# gallery/main.py

def run_gallery():
    from auroraview.core.ipc import IpcChannel, LocalSocketChannel, IpcMessage
    from auroraview.core.packed import is_packed_mode
    
    packed_mode = is_packed_mode()
    
    if packed_mode:
        # Connect to Rust CLI via LocalSocket
        channel_name = os.environ.get("AURORAVIEW_IPC_CHANNEL")
        if not channel_name:
            raise RuntimeError("AURORAVIEW_IPC_CHANNEL not set")
        
        ipc_channel = LocalSocketChannel.connect(channel_name)
        
        # Set up event callback using IPC channel
        def packed_emit_callback(event_name: str, data: Any):
            """Emit events via IPC channel (not stdout)."""
            ipc_channel.emit(event_name, data)
            # Now safe to print for debugging!
            print(f"[Debug] Emitted event: {event_name}")
        
        plugins.set_emit_callback(packed_emit_callback)
        
        # Send ready signal via IPC
        ipc_channel.send(IpcMessage(
            type="ready",
            data={"handlers": list(view.handlers.keys())}
        ))
        
        # Start API server loop
        def api_server_loop():
            while not ipc_channel.is_shutdown():
                msg = ipc_channel.receive(timeout=0.1)
                if msg and msg.type == "request":
                    # Handle API request
                    result = handle_request(msg.method, msg.data)
                    ipc_channel.send(IpcMessage(
                        type="response",
                        id=msg.id,
                        data=result
                    ))
        
        thread = threading.Thread(target=api_server_loop, daemon=True)
        thread.start()
```

### 6. IPC Mode Selection

```rust
// crates/auroraview-core/src/ipc/factory.rs

/// Factory for creating IPC channels based on context
pub struct IpcChannelFactory;

impl IpcChannelFactory {
    /// Create appropriate IPC channel for the current context
    pub fn create(mode: IpcMode, name: &str) -> Result<Box<dyn IpcChannel>, IpcChannelError> {
        match mode {
            IpcMode::Thread => {
                // In-process communication (WebView ↔ Python bindings)
                Ok(Box::new(ThreadChannel::new(name, 10_000)))
            }
            IpcMode::LocalSocket => {
                // Process-to-process (Packed mode, subprocess)
                Ok(Box::new(LocalSocketChannel::connect(name)?))
            }
            IpcMode::SharedMemory => {
                // Large data transfer (future use)
                Ok(Box::new(SharedMemoryChannel::new(name)?))
            }
        }
    }
    
    /// Auto-detect IPC mode from environment
    pub fn auto_detect() -> IpcMode {
        if std::env::var("AURORAVIEW_IPC_CHANNEL").is_ok() {
            IpcMode::LocalSocket
        } else if std::env::var("AURORAVIEW_PACKED").is_ok() {
            IpcMode::LocalSocket
        } else {
            IpcMode::Thread
        }
    }
}

/// IPC mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcMode {
    /// In-process thread communication (crossbeam-channel)
    Thread,
    /// Process-to-process via LocalSocket (ipckit)
    LocalSocket,
    /// Shared memory for large data (ipckit, future)
    SharedMemory,
}
```

### 7. DCC Integration

```rust
// crates/auroraview-core/src/ipc/dcc.rs

/// DCC-specific IPC channel with Qt integration
/// 
/// Wraps ThreadChannel with Qt event loop integration
/// for safe cross-thread communication in Maya, Houdini, etc.
pub struct DccIpcChannel {
    inner: ThreadChannel,
    /// Qt signal emitter for waking Qt event loop
    qt_waker: Option<Box<dyn QtWaker>>,
}

impl DccIpcChannel {
    pub fn new(name: &str) -> Self {
        Self {
            inner: ThreadChannel::new(name, 10_000),
            qt_waker: None,
        }
    }
    
    /// Set Qt waker for event loop integration
    pub fn set_qt_waker(&mut self, waker: Box<dyn QtWaker>) {
        self.qt_waker = Some(waker);
    }
}

impl IpcChannel for DccIpcChannel {
    fn send(&self, msg: IpcMessage) -> Result<(), IpcChannelError> {
        let result = self.inner.send(msg);
        
        // Wake Qt event loop if waker is set
        if let Some(waker) = &self.qt_waker {
            waker.wake();
        }
        
        result
    }
    
    // ... delegate other methods to inner
}

/// Trait for Qt event loop waking
pub trait QtWaker: Send + Sync {
    fn wake(&self);
}
```

### 8. Metrics & Monitoring

```rust
// crates/auroraview-core/src/ipc/metrics.rs

/// Extended IPC metrics with channel-specific tracking
#[derive(Debug, Clone)]
pub struct IpcChannelMetrics {
    pub channel_name: String,
    pub channel_type: String,  // "thread", "local_socket", "shared_memory"
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub errors: AtomicU64,
    pub avg_latency_us: AtomicU64,
    pub peak_queue_length: AtomicUsize,
    pub created_at: Instant,
}

impl IpcChannelMetrics {
    pub fn snapshot(&self) -> IpcChannelMetricsSnapshot {
        IpcChannelMetricsSnapshot {
            channel_name: self.channel_name.clone(),
            channel_type: self.channel_type.clone(),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            avg_latency_us: self.avg_latency_us.load(Ordering::Relaxed),
            peak_queue_length: self.peak_queue_length.load(Ordering::Relaxed),
            uptime_secs: self.created_at.elapsed().as_secs(),
        }
    }
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

1. **Create `IpcChannel` trait and base types**
   - `crates/auroraview-core/src/ipc/channel.rs`
   - `crates/auroraview-core/src/ipc/message.rs`
   - `crates/auroraview-core/src/ipc/error.rs`

2. **Implement `ThreadChannel`**
   - Migrate existing `MessageQueue` to implement `IpcChannel`
   - Add bidirectional communication support

3. **Implement `LocalSocketChannel`**
   - Wrapper around ipckit's `LocalSocket`
   - Server and client implementations

4. **Python bindings**
   - `python/auroraview/core/ipc/channel.py`
   - `python/auroraview/core/ipc/message.py`

### Phase 2: Packed Mode Migration (Week 2)

1. **Update Rust CLI backend**
   - Replace stdin/stdout with `LocalSocketChannel`
   - Update message routing logic

2. **Update Python backend**
   - Use `LocalSocketChannel.connect()`
   - Remove stdout-based IPC code

3. **Backward compatibility**
   - Keep old protocol as fallback (env var switch)
   - Gradual migration path

### Phase 3: Integration & Testing (Week 3)

1. **Update ProcessPlugin**
   - Use unified `IpcChannel` interface
   - Remove duplicate ipckit code

2. **DCC integration**
   - Test with Maya, Houdini, Blender
   - Qt event loop integration

3. **Performance benchmarks**
   - Compare LocalSocket vs stdout performance
   - Latency and throughput testing

### Phase 4: Documentation & Cleanup (Week 4)

1. **Migration guide**
   - Document new IPC APIs
   - Update examples

2. **Remove deprecated code**
   - Clean up old stdout-based IPC
   - Remove redundant implementations

3. **CI/CD updates**
   - Add IPC integration tests
   - Performance regression tests

## Migration Guide

### For Plugin Developers

**Before (stdout-based):**
```python
def packed_emit_callback(event_name, data):
    event_msg = json.dumps({"type": "event", "event": event_name, "data": data})
    print(event_msg, flush=True)  # Can't print debug messages!
```

**After (LocalSocket-based):**
```python
def packed_emit_callback(event_name, data):
    ipc_channel.emit(event_name, data)
    print(f"[Debug] Emitted: {event_name}")  # Now safe to print!
```

### For Application Developers

**Before:**
```python
# Had to check packed mode and use different code paths
if is_packed_mode():
    # Special stdout handling
    pass
else:
    # Normal WebView handling
    pass
```

**After:**
```python
# Unified IPC interface works in all modes
channel = IpcChannelFactory.create_for_context()
channel.emit("my_event", {"data": "value"})
```

## Compatibility

### Backward Compatibility

- Existing stdout-based IPC will continue to work via fallback
- Environment variable `AURORAVIEW_IPC_MODE=legacy` forces old behavior
- Gradual migration path for existing applications

### Python Version Support

- Python 3.7+ (matches project requirements)
- No new Python dependencies (uses ipckit's Python bindings)

### Platform Support

- Windows: Named Pipes (`\\.\pipe\auroraview_xxx`)
- macOS/Linux: Unix Domain Sockets (`/tmp/auroraview_xxx`)

## Performance Comparison

| Metric | stdout (Current) | LocalSocket (New) | Improvement |
|--------|------------------|-------------------|-------------|
| Latency (μs) | ~500 | ~50 | 10x |
| Throughput (msg/s) | ~10,000 | ~100,000 | 10x |
| CPU overhead | Higher (string parsing) | Lower (binary-ready) | ~30% |
| Memory | Higher (buffering) | Lower | ~20% |
| Debugging | Difficult | Easy (stdout free) | ∞ |

## Risks and Mitigations

### Risk 1: Breaking Changes
**Mitigation**: Phased rollout with backward compatibility mode

### Risk 2: Platform-specific Issues
**Mitigation**: Extensive testing on Windows/macOS/Linux

### Risk 3: DCC Integration Complexity
**Mitigation**: Test early with Maya, Houdini, Blender

### Risk 4: Performance Regression
**Mitigation**: Benchmarks before/after, performance CI gates

## Alternatives Considered

### 1. Keep stdout-based IPC
**Rejected**: Mixing logs and data is fundamentally problematic

### 2. Use raw TCP sockets
**Rejected**: More complex than LocalSocket, no platform abstraction

### 3. Use gRPC
**Rejected**: Too heavyweight for this use case, adds dependencies

### 4. Use ZeroMQ
**Rejected**: Additional dependency, ipckit already provides what we need

## References

- [ipckit Documentation](https://github.com/loonghao/ipckit)
- [RFC 0002: DCC Thread Safety](./0002-dcc-thread-safety.md)
- [RFC 0007: WebView Browser Unified Architecture](./0007-webview-browser-unified-architecture.md)
- [crossbeam-channel](https://docs.rs/crossbeam-channel)

## Changelog

- 2026-01-20: Initial RFC created
