# RFC 0011: 统一 IPC 架构

## 概述

将 AuroraView 的 IPC（进程间通信）层重构为基于 ipckit 的统一分层架构。此设计覆盖整个项目生命周期，包括独立模式、打包模式、DCC 嵌入和子进程通信，用一致的高性能解决方案替代当前分散的方法。

## 动机

### 当前问题

1. **IPC 机制分散**：多种 IPC 方式共存，缺乏统一抽象：
   - 打包模式：stdin/stdout 管道用于 Python 后端通信
   - ProcessPlugin：ipckit LocalSocket 用于子进程通信
   - WebView：crossbeam-channel 用于线程安全消息队列
   - DCC 模式：基于 Qt 的消息传递

2. **stdout 混用问题**：打包模式下，Python 后端同时使用 stdout 传输：
   - IPC 数据（JSON-RPC 响应、事件）
   - 调试日志（`print()` 语句）
   
   导致：
   - 调试困难（无法自由使用 `print()`）
   - 日志噪声与 IPC 数据混合
   - 意外输出可能导致解析错误

3. **错误处理不一致**：每种 IPC 机制有自己的错误类型和处理模式。

4. **性能开销**：基于 stdout 的 IPC 相比原生 socket 通信有字符串序列化开销。

5. **可测试性有限**：难以独立模拟或测试 IPC 层。

### 目标

- **统一抽象**：所有模式使用单一 IPC trait/接口
- **清晰分离**：日志走 stderr，IPC 数据走专用通道
- **高性能**：通过 ipckit 实现原生 socket 通信
- **优雅关闭**：使用 ipckit 的 `ShutdownState` 实现一致的关闭协调
- **DCC 兼容**：支持 Maya、Houdini、3ds Max、Nuke、Blender
- **可测试**：易于模拟和单元测试 IPC 组件

### 使用场景

1. **打包模式**：Rust CLI ↔ Python 后端通信
2. **子进程 IPC**：父进程 ↔ 启动的子脚本
3. **DCC 嵌入**：Qt 宿主 ↔ WebView 通信
4. **插件系统**：PluginRouter ↔ 各个插件
5. **AI 代理**：AI 提供者 ↔ WebView 通信

## 设计

### 1. 架构概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          统一 IPC 架构                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        应用层                                        │   │
│  │                                                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────┐  │   │
│  │  │ WebView   │ │ 打包模式  │ │ DCC       │ │ 子进程            │  │   │
│  │  │ (wry/tao) │ │ CLI       │ │ 嵌入      │ │ (ProcessPlugin)   │  │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        IPC 抽象层                                    │   │
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
│  │  │              IpcChannel 类 (Python)                          │   │   │
│  │  │  - send(data: dict) -> bool                                  │   │   │
│  │  │  - receive(timeout: float) -> dict | None                    │   │   │
│  │  │  - on_message(handler: Callable)                             │   │   │
│  │  │  - close()                                                   │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        传输层 (ipckit)                              │   │
│  │                                                                     │   │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────────────────┐ │   │
│  │  │ LocalSocket   │ │ SharedMemory  │ │ ThreadChannel             │ │   │
│  │  │ (管道/UDS)    │ │ (大数据)      │ │ (进程内)                  │ │   │
│  │  └───────────────┘ └───────────────┘ └───────────────────────────┘ │   │
│  │                                                                     │   │
│  │  ┌───────────────────────────────────────────────────────────────┐ │   │
│  │  │              优雅关闭 (ShutdownState)                          │ │   │
│  │  │  - 跨所有通道的协调关闭                                        │ │   │
│  │  │  - 防止 "EventLoopClosed" 错误                                 │ │   │
│  │  │  - 传输中消息的操作守护                                        │ │   │
│  │  └───────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        平台抽象层                                    │   │
│  │                                                                     │   │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────────────────┐ │   │
│  │  │   Windows     │ │    macOS      │ │         Linux             │ │   │
│  │  │ 命名管道      │ │ Unix 域       │ │ Unix 域套接字             │ │   │
│  │  │ \\.\pipe\...  │ │ 套接字        │ │ /tmp/ipckit_...           │ │   │
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

/// 统一的 IPC 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// 消息类型: "request", "response", "event", "ping", "pong"
    #[serde(rename = "type")]
    pub msg_type: String,
    
    /// 请求/响应 ID 用于关联
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// 方法名（用于请求）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    
    /// 事件名（用于事件）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    
    /// 负载数据
    #[serde(default)]
    pub data: Value,
    
    /// 错误信息（用于错误响应）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
}

/// IPC 错误结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// IPC Channel trait - 所有 IPC 后端的统一接口
pub trait IpcChannel: Send + Sync {
    /// 通过通道发送消息
    fn send(&self, msg: IpcMessage) -> Result<(), IpcChannelError>;
    
    /// 接收消息（阻塞，可选超时）
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<IpcMessage, IpcChannelError>;
    
    /// 尝试接收消息（非阻塞）
    fn try_recv(&self) -> Result<Option<IpcMessage>, IpcChannelError>;
    
    /// 发送 JSON 数据作为消息
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
    
    /// 发出事件
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
    
    /// 发起优雅关闭
    fn shutdown(&self);
    
    /// 检查是否已发起关闭
    fn is_shutdown(&self) -> bool;
    
    /// 等待挂起操作完成
    fn wait_for_drain(&self, timeout: Option<std::time::Duration>) -> Result<(), IpcChannelError>;
    
    /// 获取通道名称/标识符
    fn name(&self) -> &str;
}

/// IPC Channel 错误类型
#[derive(Debug, thiserror::Error)]
pub enum IpcChannelError {
    #[error("通道已断开")]
    Disconnected,
    
    #[error("发送失败: {0}")]
    SendFailed(String),
    
    #[error("接收失败: {0}")]
    ReceiveFailed(String),
    
    #[error("超时")]
    Timeout,
    
    #[error("正在关闭")]
    ShuttingDown,
    
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 3. 通道实现

#### 3.1 LocalSocket 通道（进程通信）

```rust
// crates/auroraview-core/src/ipc/local_socket_channel.rs

use ipckit::local_socket::{LocalSocketListener, LocalSocketStream};
use ipckit::graceful::ShutdownState;
use std::sync::Arc;

/// 基于 LocalSocket 的 IPC 通道（使用 ipckit）
/// 
/// 平台特定实现：
/// - Windows: 命名管道 (\\.\pipe\auroraview_xxx)
/// - Unix: Unix 域套接字 (/tmp/auroraview_xxx)
pub struct LocalSocketChannel {
    name: String,
    stream: LocalSocketStream,
    shutdown_state: Arc<ShutdownState>,
}

impl LocalSocketChannel {
    /// 创建服务端通道（监听器）
    pub fn server(name: &str) -> Result<LocalSocketChannelServer, IpcChannelError> {
        let listener = LocalSocketListener::bind(name)?;
        Ok(LocalSocketChannelServer {
            name: name.to_string(),
            listener,
            shutdown_state: Arc::new(ShutdownState::new()),
        })
    }
    
    /// 连接到现有通道（客户端）
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
    
    // ... 其他 trait 方法
}
```

#### 3.2 线程通道（进程内通信）

```rust
// crates/auroraview-core/src/ipc/thread_channel.rs

use crossbeam_channel::{bounded, Receiver, Sender};
use ipckit::graceful::ShutdownState;

/// 线程安全的进程内 IPC 通道
/// 
/// 使用 crossbeam-channel 实现无锁通信。
/// 适用于 WebView ↔ Python 绑定通信。
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
    
    /// 创建配对通道用于双向通信
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
    
    // ... 其他 trait 方法
}
```

### 4. Python IPC Channel

```python
# python/auroraview/core/ipc/channel.py

"""AuroraView 统一 IPC 通道。

提供跨所有模式的一致 IPC 通信接口：
- 独立模式：ThreadChannel（进程内）
- 打包模式：LocalSocketChannel（进程间）
- DCC 模式：ThreadChannel 与 Qt 集成
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

# 如果可用，使用 Rust 驱动的 JSON
try:
    from auroraview._core import json_dumps, json_loads
except ImportError:
    json_dumps = lambda obj: json.dumps(obj, ensure_ascii=False)
    json_loads = json.loads


class IpcMessageType(Enum):
    """IPC 消息类型。"""
    REQUEST = "request"
    RESPONSE = "response"
    EVENT = "event"
    PING = "ping"
    PONG = "pong"
    DATA = "data"


@dataclass
class IpcMessage:
    """统一的 IPC 消息结构。"""
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
    """IPC 通道错误。"""
    pass


class IpcChannel:
    """IPC 通道抽象基类。"""
    
    def __init__(self, name: str):
        self.name = name
        self._shutdown = False
        self._handlers: list[Callable[[IpcMessage], None]] = []
        self._lock = threading.Lock()
    
    def send(self, msg: IpcMessage) -> bool:
        """通过通道发送消息。"""
        raise NotImplementedError
    
    def receive(self, timeout: Optional[float] = None) -> Optional[IpcMessage]:
        """接收消息（阻塞，可选超时）。"""
        raise NotImplementedError
    
    def emit(self, event: str, data: Any = None) -> bool:
        """发出事件。"""
        msg = IpcMessage(
            type=IpcMessageType.EVENT.value,
            event=event,
            data=data or {},
        )
        return self.send(msg)
    
    def on_message(self, handler: Callable[[IpcMessage], None]) -> None:
        """注册消息处理器。"""
        self._handlers.append(handler)
    
    def shutdown(self) -> None:
        """发起优雅关闭。"""
        self._shutdown = True
    
    def is_shutdown(self) -> bool:
        """检查是否已发起关闭。"""
        return self._shutdown
    
    def close(self) -> None:
        """关闭通道。"""
        self.shutdown()


class LocalSocketChannel(IpcChannel):
    """基于 LocalSocket 的 IPC 通道（Windows 命名管道 / Unix 域套接字）。
    
    用于进程间通信（如打包模式）。
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
        """创建服务端通道。"""
        return LocalSocketChannelServer(name)
    
    @classmethod
    def connect(cls, name: str) -> "LocalSocketChannel":
        """连接到现有通道。"""
        channel = cls(name, is_server=False)
        channel._connect()
        return channel
    
    def _connect(self) -> None:
        """建立连接。"""
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
            raise IpcChannelError(f"发送失败: {e}") from e
    
    def receive(self, timeout: Optional[float] = None) -> Optional[IpcMessage]:
        if self._shutdown or not self._connected:
            return None
        
        try:
            # 从通道读取一行
            if sys.platform == "win32":
                line = self._read_line_windows()
            else:
                line = self._file.readline().strip()
            
            if not line:
                return None
            
            return IpcMessage.from_json(line)
        except Exception as e:
            raise IpcChannelError(f"接收失败: {e}") from e
    
    def _read_line_windows(self) -> str:
        """从 Windows 命名管道读取一行。"""
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
```

### 5. 打包模式迁移

#### 当前架构（基于 stdout）

```
┌─────────────────┐      stdin (JSON-RPC)      ┌─────────────────┐
│    Rust CLI     │ ──────────────────────────> │  Python 后端    │
│   (WebView2)    │                             │   (Gallery)     │
│                 │ <────────────────────────── │                 │
└─────────────────┘      stdout (JSON/事件)     └─────────────────┘
                              ↓
                    stderr (日志 + 错误混合)
```

#### 新架构（基于 LocalSocket）

```
┌─────────────────────────────────────────────────────────────────┐
│                      Rust CLI (WebView2)                         │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │            LocalSocketChannelServer                          ││
│  │         (\\.\pipe\auroraview_{session})                      ││
│  └─────────────────────────────────────────────────────────────┘│
│                          ↕ JSON-RPC                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                 Python 后端                                  ││
│  │            LocalSocketChannel.connect()                      ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│       stdout → 纯日志，可自由 print() 调试                       │
│       stderr → 错误日志                                          │
└─────────────────────────────────────────────────────────────────┘
```

#### 迁移步骤

**阶段 1：Rust CLI 改动**

```rust
// crates/auroraview-cli/src/packed/backend.rs

use auroraview_core::ipc::{IpcChannel, LocalSocketChannel, LocalSocketChannelServer};

pub struct PythonBackend {
    process: Mutex<Child>,
    /// IPC 通道（替代 stdin）
    ipc_channel: Arc<dyn IpcChannel>,
    shutdown_state: Arc<ShutdownState>,
}

pub fn start_python_backend_with_ipc(
    overlay: &OverlayData,
    python_config: &PythonBundleConfig,
    proxy: EventLoopProxy<UserEvent>,
    metrics: &mut PackedMetrics,
) -> Result<PythonBackend> {
    // 为此会话生成唯一通道名
    let channel_name = format!("packed_{}", std::process::id());
    
    // 在启动 Python 之前创建服务端通道
    let server = LocalSocketChannelServer::new(&channel_name)?;
    
    // 设置环境变量供 Python 连接
    std::env::set_var("AURORAVIEW_IPC_CHANNEL", &channel_name);
    
    // 启动 Python 进程（不再为 IPC 捕获 stdout）
    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", &python_code])
        .current_dir(&temp_dir)
        .stdin(Stdio::null())      // 无 stdin IPC
        .stdout(Stdio::inherit())  // 传递用于日志
        .stderr(Stdio::inherit()); // 传递用于错误
    
    // ... 启动子进程 ...
    
    // 等待 Python 连接
    let channel = server.accept()?;
    
    // 启动消息读取线程
    let channel_clone = Arc::clone(&channel);
    let shutdown_state_clone = Arc::clone(&shutdown_state);
    thread::spawn(move || {
        loop {
            if shutdown_state_clone.is_shutdown() {
                break;
            }
            
            match channel_clone.recv(Some(Duration::from_millis(100))) {
                Ok(msg) => {
                    // 根据类型处理消息
                    match msg.msg_type.as_str() {
                        "event" => {
                            // 转发到 WebView
                            let _ = proxy.send_event(UserEvent::PluginEvent {
                                event: msg.event.unwrap_or_default(),
                                data: msg.data.to_string(),
                            });
                        }
                        "response" => {
                            // 转发 API 响应
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

**阶段 2：Python 后端改动**

```python
# gallery/main.py

def run_gallery():
    from auroraview.core.ipc import IpcChannel, LocalSocketChannel, IpcMessage
    from auroraview.core.packed import is_packed_mode
    
    packed_mode = is_packed_mode()
    
    if packed_mode:
        # 通过 LocalSocket 连接到 Rust CLI
        channel_name = os.environ.get("AURORAVIEW_IPC_CHANNEL")
        if not channel_name:
            raise RuntimeError("AURORAVIEW_IPC_CHANNEL 未设置")
        
        ipc_channel = LocalSocketChannel.connect(channel_name)
        
        # 使用 IPC 通道设置事件回调
        def packed_emit_callback(event_name: str, data: Any):
            """通过 IPC 通道发出事件（不是 stdout）。"""
            ipc_channel.emit(event_name, data)
            # 现在可以安全地打印调试信息了！
            print(f"[Debug] 发出事件: {event_name}")
        
        plugins.set_emit_callback(packed_emit_callback)
        
        # 通过 IPC 发送就绪信号
        ipc_channel.send(IpcMessage(
            type="ready",
            data={"handlers": list(view.handlers.keys())}
        ))
        
        # 启动 API 服务器循环
        def api_server_loop():
            while not ipc_channel.is_shutdown():
                msg = ipc_channel.receive(timeout=0.1)
                if msg and msg.type == "request":
                    # 处理 API 请求
                    result = handle_request(msg.method, msg.data)
                    ipc_channel.send(IpcMessage(
                        type="response",
                        id=msg.id,
                        data=result
                    ))
        
        thread = threading.Thread(target=api_server_loop, daemon=True)
        thread.start()
```

### 6. IPC 模式选择

```rust
// crates/auroraview-core/src/ipc/factory.rs

/// 根据上下文创建 IPC 通道的工厂
pub struct IpcChannelFactory;

impl IpcChannelFactory {
    /// 为当前上下文创建适当的 IPC 通道
    pub fn create(mode: IpcMode, name: &str) -> Result<Box<dyn IpcChannel>, IpcChannelError> {
        match mode {
            IpcMode::Thread => {
                // 进程内通信（WebView ↔ Python 绑定）
                Ok(Box::new(ThreadChannel::new(name, 10_000)))
            }
            IpcMode::LocalSocket => {
                // 进程间（打包模式，子进程）
                Ok(Box::new(LocalSocketChannel::connect(name)?))
            }
            IpcMode::SharedMemory => {
                // 大数据传输（未来使用）
                Ok(Box::new(SharedMemoryChannel::new(name)?))
            }
        }
    }
    
    /// 从环境自动检测 IPC 模式
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

/// IPC 模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcMode {
    /// 进程内线程通信（crossbeam-channel）
    Thread,
    /// 进程间通过 LocalSocket（ipckit）
    LocalSocket,
    /// 大数据共享内存（ipckit，未来）
    SharedMemory,
}
```

### 7. DCC 集成

```rust
// crates/auroraview-core/src/ipc/dcc.rs

/// DCC 特定的 IPC 通道，带 Qt 集成
/// 
/// 包装 ThreadChannel，添加 Qt 事件循环集成
/// 用于 Maya、Houdini 等的安全跨线程通信
pub struct DccIpcChannel {
    inner: ThreadChannel,
    /// Qt 信号发射器用于唤醒 Qt 事件循环
    qt_waker: Option<Box<dyn QtWaker>>,
}

impl DccIpcChannel {
    pub fn new(name: &str) -> Self {
        Self {
            inner: ThreadChannel::new(name, 10_000),
            qt_waker: None,
        }
    }
    
    /// 设置 Qt 唤醒器用于事件循环集成
    pub fn set_qt_waker(&mut self, waker: Box<dyn QtWaker>) {
        self.qt_waker = Some(waker);
    }
}

impl IpcChannel for DccIpcChannel {
    fn send(&self, msg: IpcMessage) -> Result<(), IpcChannelError> {
        let result = self.inner.send(msg);
        
        // 如果设置了唤醒器，唤醒 Qt 事件循环
        if let Some(waker) = &self.qt_waker {
            waker.wake();
        }
        
        result
    }
    
    // ... 委托其他方法给 inner
}

/// Qt 事件循环唤醒 trait
pub trait QtWaker: Send + Sync {
    fn wake(&self);
}
```

### 8. 指标与监控

```rust
// crates/auroraview-core/src/ipc/metrics.rs

/// 扩展的 IPC 指标，带通道特定跟踪
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

## 实现计划

### 阶段 1：核心基础设施（第 1 周）

1. **创建 `IpcChannel` trait 和基础类型**
   - `crates/auroraview-core/src/ipc/channel.rs`
   - `crates/auroraview-core/src/ipc/message.rs`
   - `crates/auroraview-core/src/ipc/error.rs`

2. **实现 `ThreadChannel`**
   - 迁移现有 `MessageQueue` 以实现 `IpcChannel`
   - 添加双向通信支持

3. **实现 `LocalSocketChannel`**
   - 包装 ipckit 的 `LocalSocket`
   - 服务端和客户端实现

4. **Python 绑定**
   - `python/auroraview/core/ipc/channel.py`
   - `python/auroraview/core/ipc/message.py`

### 阶段 2：打包模式迁移（第 2 周）

1. **更新 Rust CLI 后端**
   - 用 `LocalSocketChannel` 替换 stdin/stdout
   - 更新消息路由逻辑

2. **更新 Python 后端**
   - 使用 `LocalSocketChannel.connect()`
   - 移除基于 stdout 的 IPC 代码

3. **向后兼容**
   - 保留旧协议作为后备（环境变量切换）
   - 渐进式迁移路径

### 阶段 3：集成与测试（第 3 周）

1. **更新 ProcessPlugin**
   - 使用统一的 `IpcChannel` 接口
   - 移除重复的 ipckit 代码

2. **DCC 集成**
   - 在 Maya、Houdini、Blender 中测试
   - Qt 事件循环集成

3. **性能基准测试**
   - 比较 LocalSocket vs stdout 性能
   - 延迟和吞吐量测试

### 阶段 4：文档与清理（第 4 周）

1. **迁移指南**
   - 记录新的 IPC API
   - 更新示例

2. **移除废弃代码**
   - 清理旧的基于 stdout 的 IPC
   - 移除冗余实现

3. **CI/CD 更新**
   - 添加 IPC 集成测试
   - 性能回归测试

## 迁移指南

### 对于插件开发者

**之前（基于 stdout）：**
```python
def packed_emit_callback(event_name, data):
    event_msg = json.dumps({"type": "event", "event": event_name, "data": data})
    print(event_msg, flush=True)  # 无法打印调试信息！
```

**之后（基于 LocalSocket）：**
```python
def packed_emit_callback(event_name, data):
    ipc_channel.emit(event_name, data)
    print(f"[Debug] 发出: {event_name}")  # 现在可以安全打印了！
```

### 对于应用开发者

**之前：**
```python
# 必须检查打包模式并使用不同的代码路径
if is_packed_mode():
    # 特殊的 stdout 处理
    pass
else:
    # 正常的 WebView 处理
    pass
```

**之后：**
```python
# 统一的 IPC 接口在所有模式下工作
channel = IpcChannelFactory.create_for_context()
channel.emit("my_event", {"data": "value"})
```

## 兼容性

### 向后兼容

- 现有基于 stdout 的 IPC 通过后备继续工作
- 环境变量 `AURORAVIEW_IPC_MODE=legacy` 强制使用旧行为
- 现有应用的渐进式迁移路径

### Python 版本支持

- Python 3.7+（符合项目要求）
- 无新 Python 依赖（使用 ipckit 的 Python 绑定）

### 平台支持

- Windows：命名管道（`\\.\pipe\auroraview_xxx`）
- macOS/Linux：Unix 域套接字（`/tmp/auroraview_xxx`）

## 性能比较

| 指标 | stdout（当前） | LocalSocket（新） | 改进 |
|------|----------------|-------------------|------|
| 延迟 (μs) | ~500 | ~50 | 10x |
| 吞吐量 (msg/s) | ~10,000 | ~100,000 | 10x |
| CPU 开销 | 较高（字符串解析） | 较低（二进制就绪） | ~30% |
| 内存 | 较高（缓冲） | 较低 | ~20% |
| 调试 | 困难 | 简单（stdout 释放） | ∞ |

## 风险与缓解

### 风险 1：破坏性变更
**缓解**：带向后兼容模式的分阶段推出

### 风险 2：平台特定问题
**缓解**：在 Windows/macOS/Linux 上进行广泛测试

### 风险 3：DCC 集成复杂性
**缓解**：提前在 Maya、Houdini、Blender 中测试

### 风险 4：性能回归
**缓解**：前后基准测试，性能 CI 关卡

## 考虑过的替代方案

### 1. 保持基于 stdout 的 IPC
**拒绝**：混合日志和数据是根本性问题

### 2. 使用原始 TCP 套接字
**拒绝**：比 LocalSocket 更复杂，无平台抽象

### 3. 使用 gRPC
**拒绝**：对此用例过于重量级，增加依赖

### 4. 使用 ZeroMQ
**拒绝**：额外依赖，ipckit 已提供所需功能

## 参考资料

- [ipckit 文档](https://github.com/loonghao/ipckit)
- [RFC 0002: DCC 线程安全](./0002-dcc-thread-safety.md)
- [RFC 0007: WebView 浏览器统一架构](./0007-webview-browser-unified-architecture.md)
- [crossbeam-channel](https://docs.rs/crossbeam-channel)

## 变更日志

- 2026-01-20: 初始 RFC 创建
