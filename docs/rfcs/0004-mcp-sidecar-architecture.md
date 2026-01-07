# RFC 0004: MCP Sidecar Process 架构（v2）

> **状态**: Draft (v2 重设计)
> **作者**: AuroraView Team
> **创建日期**: 2026-01-04
> **更新日期**: 2026-01-06
> **目标版本**: v0.5.0

## 摘要

本 RFC 提议将 MCP Server 重构为 **独立 Rust 进程（Sidecar Process）** 架构，解耦 MCP 服务与 WebView 生命周期。这种架构解决当前实现中的**时序耦合**、**事件循环依赖**和**跨线程调试困难**等问题。

**核心目标**：
1. **进程隔离**：MCP Server 作为独立进程运行，崩溃不影响主进程
2. **纯 Rust Sidecar**：Sidecar 本身不依赖 Python 解释器，跨 DCC 环境通用
3. **本地 IPC**：主进程与 Sidecar 通过 Named Pipe (Windows) / Unix Socket (macOS/Linux) 通信
4. **统一分发**：随 Python wheel 一起打包分发，支持 PyPI 发布
5. **跨平台**：Windows、macOS、Linux 全平台支持（Windows 优先）

> 重要约束：AuroraView Python 包最低兼容 **Python 3.7**。本 RFC 中所有 Python 示例需保持 3.7 可用。

## 动机

### 当前架构问题（v1 问题总结）

通过实际开发和调试，我们发现 Sidecar Thread 架构（v1）存在以下严重问题：

#### 问题 1：初始化时序耦合

```
时序问题：
1. Python: _start_mcp_server()     → 创建 MCP + Dispatcher
2. Python: show_blocking()         → 进入 Rust
3. Rust:   run_blocking()          → 设置 event_loop_proxy  ← 太晚了！
4. MCP 请求到来 → wake_event_loop() 发现 proxy 未设置 → 消息排队但无法唤醒
5. 只有用户点击 UI 才会触发事件循环处理消息
```

#### 问题 2：Embedded vs Standalone 双模式复杂度
- Embedded 模式需要 Timer drain 队列
- Standalone 模式依赖 Rust 事件循环
- 两种模式的 MCP 启动时机不一致，导致难以维护

#### 问题 3：MessageQueue 过度设计
- 7 步调用链：MCP Request → Dispatcher → MessageQueue → wake → EventLoop → process → Python
- 任何一步出错都会导致 MCP 调用卡住
- 跨 4+ 个文件调试，定位问题困难

#### 问题 4：跨语言调试困难
- Python ↔ Rust ↔ Tokio 三层交织
- GIL 获取时机不可预测
- 错误传播路径过长

### 设计目标

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Host Process (DCC / Desktop App)                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Python (DCC 解释器 / 系统 Python)                              │   │
│  │  ┌─────────────────────────────────────────────────────────────┐│   │
│  │  │  auroraview (Python API)                                    ││   │
│  │  │  ├─ WebView (Rust via PyO3)                                 ││   │
│  │  │  └─ Tool Handlers (Python functions)                        ││   │
│  │  └─────────────────────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ▲                                          │
│                              │ IPC (Named Pipe / Unix Socket)           │
│                              ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  auroraview-mcp-server (独立 Rust 二进制)                       │   │
│  │  ├─ MCP Server (Tokio + axum + rmcp)                            │   │
│  │  ├─ Tool Dispatcher                                              │   │
│  │  └─ IPC Client (连接到主进程)                                   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## 设计方案

### 1. 进程架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           System                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────┐     ┌─────────────────────────────┐  │
│  │   Main Process               │     │   MCP Sidecar Process       │  │
│  │   (auroraview + WebView)     │     │   (auroraview-mcp-server)   │  │
│  │                              │     │                              │  │
│  │  ┌────────────────────────┐  │     │  ┌────────────────────────┐ │  │
│  │  │  Python Runtime        │  │     │  │  Tokio Runtime         │ │  │
│  │  │  (DCC / System)        │  │     │  │  (独立)                │ │  │
│  │  └────────────────────────┘  │     │  └────────────────────────┘ │  │
│  │             │                │     │             │               │  │
│  │  ┌──────────▼─────────────┐  │     │  ┌──────────▼────────────┐  │  │
│  │  │  WebView               │  │     │  │  MCP Server           │  │  │
│  │  │  (wry/WebView2)        │  │     │  │  (axum + rmcp)        │  │  │
│  │  └────────────────────────┘  │     │  └──────────┬────────────┘  │  │
│  │             │                │     │             │               │  │
│  │  ┌──────────▼─────────────┐  │     │  ┌──────────▼────────────┐  │  │
│  │  │  IPC Server            │  │     │  │  IPC Client           │  │  │
│  │  │  (执行 tool handlers)  │◄─┼─────┼──│  (转发工具调用)       │  │  │
│  │  └────────────────────────┘  │     │  └───────────────────────┘  │  │
│  │                              │     │                              │  │
│  └──────────────────────────────┘     └─────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2. IPC 协议设计

#### 2.1 传输层 - 复用 ipckit

**复用现有的 `ipckit` crate**，不引入新的 IPC 实现：

```rust
// 使用 ipckit 的 LocalSocket
use ipckit::local_socket::{LocalSocketListener, LocalSocketStream};

// 平台自动适配
// - Windows: Named Pipe (\\.\pipe\ipckit_{name})
// - Unix: Unix Domain Socket (/tmp/ipckit_{name})
```

| 平台 | 传输方式 | 名称/路径（示例） |
|------|----------|--------------------|
| Windows | Named Pipe | `\\.\pipe\ipckit_auroraview_mcp_{pid}_{nonce}` |
| macOS | Unix Socket | `/tmp/ipckit_auroraview_mcp_{pid}_{nonce}` |
| Linux | Unix Socket | `/tmp/ipckit_auroraview_mcp_{pid}_{nonce}` |

**ipckit 复用优势**：
- ✅ 已有成熟实现，在 `ProcessPlugin` 中验证过
- ✅ 支持 `ShutdownState` 优雅关闭
- ✅ Python 客户端已实现（`auroraview.core.ipc_channel`）
- ✅ 跨平台测试基础较完善

**ipckit 增强需求（按需）**：
- [ ] 连接超时配置（connect timeout）
- [ ] keepalive / heartbeat（用于健康检查与检测半开连接）
- [ ] 连接复用或并发请求支持（避免单请求阻塞）

#### 2.2 协议选择：JSON-RPC 2.0 vs gRPC

| 特性 | JSON-RPC 2.0 | gRPC |
|------|--------------|------|
| **序列化效率** | 中等（JSON 文本） | 高（Protobuf 二进制） |
| **类型安全** | ❌ 无 schema | ✅ 强类型 .proto |
| **代码生成** | ❌ 不需要 | ✅ 需要 protoc |
| **调试友好** | ✅ 可读文本 | ❌ 二进制不可读 |
| **Python 依赖** | ✅ 无（内置 json） | ❌ grpcio (~10MB) |
| **Rust 依赖** | ✅ serde_json | ❌ tonic + prost |
| **流式支持** | ❌ 无 | ✅ 双向流 |

**决定：使用 JSON-RPC 2.0（与现有 IpcChannel 模式一致）**

理由：
1. **MCP 工具调用是低频操作**：每秒通常不超过 10 次，JSON 开销可忽略
2. **调试友好**：本地 IPC 调试可直接打印可读消息
3. **零额外依赖**：不需要 protobuf 编译步骤
4. **实现一致性**：与现有 `ipckit` / `IpcChannel` 的消息风格一致

> 注：若未来出现高频数据流需求（如实时日志 > 1000 msg/s），可考虑新增 MessagePack 编码作为可选项（不改变 JSON-RPC 语义，仅替换传输编码）。

#### 2.3 消息格式（JSON-RPC 2.0 规范化）

> 说明：成功响应使用 `result`，失败响应使用 `error`（JSON-RPC 标准），避免额外包一层 `ok`。

```json
// 请求：Sidecar → Main Process
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tool.call",
  "params": {
    "name": "api.get_samples",
    "arguments": {},
    "timeout_ms": 5000,
    "trace_id": "optional"
  }
}

// 成功响应：Main Process → Sidecar
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    "..."
  ]
}

// 失败响应：Main Process → Sidecar
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": 1001,
    "message": "Tool not found",
    "data": {
      "name": "api.get_samples"
    }
  }
}
```

建议错误码（初版）：
- `1001`: tool_not_found
- `1002`: invalid_arguments
- `1003`: execution_error（Python handler 抛异常）
- `1004`: timeout
- `1005`: cancelled（预留）
- `1006`: shutting_down

#### 2.4 生命周期消息

```json
// Sidecar 启动完成（并告知 MCP 监听端口）
{"jsonrpc": "2.0", "method": "lifecycle.ready", "params": {"port": 8080}}

// 主进程请求关闭（优雅关闭优先）
{"jsonrpc": "2.0", "method": "lifecycle.shutdown"}

// Sidecar 确认关闭
{"jsonrpc": "2.0", "method": "lifecycle.bye"}
```

> 约束：若未来支持 MCP over stdio，协议输出必须只写 stdout，日志必须只写 stderr，否则会污染 MCP 客户端解析。

#### 2.5 工具注册表同步（必须补齐的协议）

Sidecar 需要知道“有哪些工具可以对外暴露”，因此需要一个最小可用的同步机制：

- `tool.list`：Sidecar 启动后向主进程拉取一次全量工具清单
- `tool.changed`：主进程工具增删变更时（可选）向 Sidecar 推送增量

工具描述建议包含：
- `name`: 工具名（例如 `api.create_sphere`）
- `description`: 简短说明
- `inputSchema`: JSON Schema（与 MCP tools schema 对齐）
- `outputSchema`: JSON Schema（可选）
- `visibility`: `public|internal`（预留）
- `scopes`: 权限域（预留）

> 初版实现可以只做 `tool.list` + `tool.call`，`tool.changed` 作为增强项。

### 3. 安全与网络问题

#### 3.1 防火墙与端口

**IPC 通道（不受防火墙影响）**：
- ✅ Named Pipe / Unix Socket 是本地 IPC，不走网络栈
- ✅ 不需要任何网络权限

**MCP HTTP Server（仅本机回环）**：
- ⚠️ 端口监听可能触发企业安全软件提示
- ⚠️ 端口冲突风险

**缓解措施**：
- 仅绑定 `127.0.0.1`，不暴露到局域网
- 支持 `port=0` 自动端口分配
- （未来可选）Unix Socket/Named Pipe 直连模式，完全避免 TCP

#### 3.2 权限问题

| 场景 | 风险 | 缓解 |
|------|------|------|
| Named Pipe 创建 | 中（同用户其它进程可连接） | 使用 nonce + token 握手 +（Windows）ACL 仅限当前用户 |
| 端口 1024 以下 | 高（需要 admin） | 默认使用 > 1024 或 `0` 自动分配 |
| Unix Socket /tmp | 低 | 使用用户可写路径，并带随机 nonce |
| DCC 沙盒环境 | 中 | 文档说明配置与降级策略 |

#### 3.3 进程隔离安全（PID 不够，需 nonce/token）

仅靠 `parent_pid` 拼接 channel name 并不能完全隔离同机同用户的其它进程连接。

建议：
1. 主进程生成随机 `nonce`，使 IPC 名称不可猜测：`auroraview_mcp_{pid}_{nonce}`
2. 主进程生成一次性 `token`，在 IPC 连接建立后做 `auth.hello` 握手校验
3. Windows 平台补充 Pipe ACL（仅允许当前用户）作为增强项

### 4. 二进制分发

#### 4.1 Wheel 结构

```
auroraview-0.5.0-cp311-cp311-win_amd64.whl
├── auroraview/
│   ├── __init__.py
│   ├── core/
│   │   ├── __init__.py
│   │   └── webview.py
│   ├── mcp/
│   │   ├── __init__.py
│   │   └── sidecar.py          # Sidecar 管理器
│   └── _native/
│       ├── __init__.py
│       ├── _core.pyd           # Rust WebView 绑定 (Windows)
│       └── auroraview-mcp-server.exe  # MCP Sidecar 二进制
├── auroraview-0.5.0.dist-info/
└── ...

# macOS
auroraview/_native/auroraview-mcp-server

# Linux
auroraview/_native/auroraview-mcp-server
```

> 注：上例仅展示文件布局；实际 wheel 会按 AuroraView 的 Python 版本矩阵产出（最低支持 3.7）。

#### 4.2 Sidecar 管理器 (Python, 兼容 3.7)

```python
# auroraview/mcp/sidecar.py

import os
import subprocess
import sys
from pathlib import Path
from typing import Optional, Sequence


class McpSidecar(object):
    """MCP Sidecar Process Manager."""

    def __init__(self, mcp_port, ipc_name, token, parent_pid=None, extra_args=None):
        self.mcp_port = int(mcp_port)
        self.ipc_name = str(ipc_name)
        self.token = str(token)
        self.parent_pid = int(parent_pid) if parent_pid is not None else None
        self.extra_args = list(extra_args) if extra_args else []
        self.process = None  # type: Optional[subprocess.Popen]

    @staticmethod
    def get_binary_path():
        """Get platform-specific sidecar binary path."""
        native_dir = Path(__file__).resolve().parent.parent / "_native"
        if sys.platform == "win32":
            return native_dir / "auroraview-mcp-server.exe"
        return native_dir / "auroraview-mcp-server"

    def start(self):
        """Start the MCP Sidecar process."""
        binary = self.get_binary_path()
        if not binary.exists():
            raise FileNotFoundError("MCP Sidecar binary not found: %s" % (binary,))

        args = [
            str(binary),
            "--port",
            str(self.mcp_port),
            "--ipc",
            self.ipc_name,
            "--token",
            self.token,
        ]
        if self.parent_pid is not None:
            args += ["--parent-pid", str(self.parent_pid)]
        args += self.extra_args

        # 注意：不要默认使用 stdout=PIPE/stderr=PIPE，避免 buffer 填满导致子进程阻塞。
        # 建议通过环境变量控制侧向日志文件。
        env = os.environ.copy()
        self.process = subprocess.Popen(args, env=env)

    def stop(self, timeout=5):
        """Stop the MCP Sidecar process."""
        if not self.process:
            return

        # 优雅关闭优先（通过 IPC 发 lifecycle.shutdown），超时再 terminate/kill。
        # 这里仅示意，真实实现应调用 ipc_client.send_shutdown()。
        try:
            self.process.terminate()
            self.process.wait(timeout=timeout)
        except Exception:
            try:
                self.process.kill()
            except Exception:
                pass

    def is_alive(self):
        return bool(self.process and self.process.poll() is None)
```

### 5. 使用场景

#### 5.1 DCC 环境 (Maya/Houdini/Nuke)

```python
# Maya 插件中使用
import maya.cmds as cmds
from auroraview import WebView


def create_tool_panel():
    view = WebView(
        title="My Tool",
        url="http://localhost:5173",
        mcp=True,           # 自动启动 Sidecar
        mcp_port=0,         # 自动分配端口
    )

    @view.bind_call("api.create_sphere")
    def create_sphere(radius=1.0):
        """MCP 工具：创建球体（在 Maya 主线程执行）"""
        return cmds.polySphere(r=float(radius))[0]

    view.show()

# AI Agent 可以通过 MCP 调用（示意）：
# POST http://127.0.0.1:{port}/mcp
# {"method": "tools/call", "params": {"name": "api.create_sphere", "arguments": {"radius": 2.0}}}
```

#### 5.2 桌面应用

```python
from pathlib import Path
from auroraview import WebView

view = WebView(
    title="Desktop App",
    url="dist/index.html",
    mcp=True,
    mcp_port=0,
)

@view.bind_call("api.read_file")
def read_file(path):
    return Path(path).read_text()

view.show()
```

#### 5.3 便携式打包 (auroraview pack)

使用 AuroraView 自带的 `pack` 命令打包，Sidecar 二进制会自动嵌入：

```toml
# auroraview.pack.toml
[mcp]
enabled = true
port = 0

# Sidecar 二进制自动包含在 overlay / bin 中
```

**Pack 增强需求（Sidecar 支持）**：
- [ ] `auroraview-pack`: 添加 Sidecar 二进制嵌入逻辑
- [ ] Overlay 格式支持额外二进制
- [ ] 运行时提取 Sidecar 到临时目录

### 6. CI/CD 构建流程

#### 6.1 Cargo Workspace 结构

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
  "crates/auroraview-core",
  "crates/auroraview-mcp",
  "crates/auroraview-mcp-server",
]
```

#### 6.2 Sidecar Crate

```toml
# crates/auroraview-mcp-server/Cargo.toml
[package]
name = "auroraview-mcp-server"
version = "0.5.0"

[[bin]]
name = "auroraview-mcp-server"
path = "src/main.rs"

[dependencies]
auroraview-mcp = { path = "../auroraview-mcp" }
ipckit = { path = "../ipckit" }
tokio = { version = "1", features = ["full"] }
axum = "0.8"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
```

#### 6.3 GitHub Actions 构建矩阵（示意）

> 注意：示例使用 `actions/*@v6`。

```yaml
# .github/workflows/build-sidecar.yml
name: Build MCP Sidecar

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: auroraview-mcp-server.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: auroraview-mcp-server
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: auroraview-mcp-server
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: auroraview-mcp-server

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v6

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build Sidecar
        run: cargo build --release -p auroraview-mcp-server --target ${{ matrix.target }}

      - name: Upload Artifact
        uses: actions/upload-artifact@v6
        with:
          name: sidecar-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact }}

  package-wheel:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Download all artifacts
        uses: actions/download-artifact@v6
        with:
          path: binaries/

      - name: Build platform wheels
        run: python scripts/build_wheels.py

      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        with:
          password: ${{ secrets.PYPI_API_TOKEN }}
```

### 7. Sidecar 二进制实现（示意）

```rust
// crates/auroraview-mcp-server/src/main.rs
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "auroraview-mcp-server")]
#[command(about = "AuroraView MCP Sidecar Server")]
struct Args {
    /// MCP server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// IPC socket/pipe name
    #[arg(long)]
    ipc: String,

    /// Auth token for IPC handshake
    #[arg(long)]
    token: String,

    /// Parent process PID (auto-exit when parent dies)
    #[arg(long)]
    parent_pid: Option<u32>,
}

fn main() {
    // 仅示意：真实实现应
    // 1) 连接 IPC
    // 2) 完成 auth.hello(token) 握手
    // 3) 启动 MCP server
    // 4) 通知 lifecycle.ready
    // 5) 监听 lifecycle.shutdown 并优雅退出
}
```

## 实现计划

详见实现跟踪：`0004-implementation-tracker.md`。

## 兼容性

### 向后兼容

- **API 保持不变**：`WebView(mcp=True)` 继续工作
- **工具注册方式不变**：`@view.bind_call()` 继续工作
- **Python 回调仍在主线程执行**

### 迁移说明

用户无需修改代码。升级到 v0.5.0 后，MCP 会自动使用 Sidecar 进程架构。

### Breaking Changes

无（内部实现变更）。

## 替代方案对比

| 方案 | 优点 | 缺点 | 决定 |
|------|------|------|------|
| Sidecar Thread (v1) | 无额外进程 | 时序耦合、调试困难 | ❌ 放弃 |
| Sidecar Process (v2) | 完全隔离、易调试 | 进程管理开销 | ✅ 采用 |
| 嵌入 Python 解释器 | 不依赖系统 Python | 体积大、启动慢 | ❌ 不采用 |
| 使用 mayapy 启动 | 无需额外二进制 | 启动慢、依赖 DCC | ❌ 不采用 |

## 参考资料

- [RFC 0001: AuroraView MCP Server](./0001-auroraview-mcp-server.md)
- [RFC 0002: 嵌入式 MCP Server](./0002-embedded-mcp-server.md)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
