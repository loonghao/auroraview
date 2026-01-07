# RFC 0004 实现跟踪 (v2)

> **RFC**: [0004-mcp-sidecar-architecture.md](./0004-mcp-sidecar-architecture.md)
> **开始日期**: 2026-01-04
> **更新日期**: 2026-01-06
> **目标版本**: v0.5.0

## 架构变更说明

**v1 → v2 重大变更**：从 Sidecar Thread 架构改为 Sidecar Process 架构。

### 变更原因

在 v1 实现过程中发现以下问题无法有效解决：

1. **时序耦合**：MCP Server 创建时 event_loop_proxy 尚未设置
2. **事件循环依赖**：MCP 工具调用依赖用户交互触发事件循环
3. **调试困难**：跨 Python/Rust/Tokio 多层调试路径过长

v2 采用独立进程架构，解耦 MCP 与 WebView 生命周期。

> 重要约束：AuroraView Python 包最低兼容 **Python 3.7**。

---

## 实现状态

| Phase | 状态 | 描述 |
|-------|------|------|
| Phase 1 | ✅ 完成 | IPC 基础设施 |
| Phase 2 | 🟡 进行中 | Sidecar 集成 (HTTP 服务器已完成) |
| Phase 3 | ⚪ 未开始 | 构建与分发 |
| Phase 4 | ⚪ 未开始 | 兼容性与测试 |

---

## Phase 1: IPC 基础设施

**预计时间**: 1 week

### 任务清单

- [x] 创建 `crates/auroraview-mcp-server` crate
  - [x] Cargo.toml 配置
  - [x] 基本 CLI 框架 (clap)
- [x] 实现跨平台 IPC（复用 `ipckit`）
  - [x] Windows Named Pipe（由 `ipckit::LocalSocket` 适配）
  - [x] Unix Domain Socket（由 `ipckit::LocalSocket` 适配）
  - [x] IPC 名称加入随机 nonce：`auroraview_mcp_{pid}_{nonce}`
  - [x] IPC 连接握手：`auth.hello(token)`
- [x] 实现 JSON-RPC 2.0（规范化）
  - [x] 请求/响应序列化（成功 `result` / 失败 `error`）
  - [x] 错误码集合（tool_not_found/invalid_arguments/timeout/...）
  - [x] 超时机制（Sidecar 侧 enforced）
- [x] 实现父进程监控
  - [x] 接收 `--parent-pid` 参数
  - [x] 定期检查父进程存活
  - [x] 父进程退出时自动退出

### 验收标准（DoD）

- [x] Sidecar 能启动并与主进程建立 IPC 连接（含 token 握手）
- [x] 端到端完成一次 `tool.call` 往返（成功 + 错误 + 超时三类）
- [x] 父进程退出后 Sidecar 自动退出（无僵尸进程）

### 测试覆盖

- 16 个单元测试和集成测试全部通过
- IPC 通信、JSON-RPC 协议、父进程监控、HTTP 服务器均已验证

### 相关文件（目标）

```
crates/auroraview-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI 入口
│   ├── ipc/
│   │   ├── mod.rs
│   │   ├── server.rs    # IPC Server (主进程侧)
│   │   └── client.rs    # IPC Client (Sidecar 侧)
│   └── protocol/
│       ├── mod.rs
│       └── jsonrpc.rs   # JSON-RPC 2.0 实现
```

---

## Phase 2: Sidecar 集成

**预计时间**: 1 week

### 任务清单

- [ ] 实现 Python `McpSidecar` 管理器（兼容 Python 3.7）
  - [ ] `get_binary_path()` - 定位 sidecar 二进制
  - [ ] `start()` - 启动 sidecar 进程（避免默认 PIPE 导致阻塞）
  - [ ] `stop()` - 优雅关闭优先（IPC `lifecycle.shutdown`），超时再 kill
  - [ ] `is_alive()` - 健康检查
- [ ] WebView MCP 集成
  - [ ] `mcp=True` 时自动启动 Sidecar
  - [ ] IPC Server 在主进程启动
  - [ ] 工具注册表同步（至少实现 `tool.list`）
- [ ] 工具调用路由
  - [ ] MCP 请求 → Sidecar → IPC → 主进程
  - [ ] Python handler 在主线程执行
  - [ ] 结果通过 IPC 返回

### 验收标准（DoD）

- [ ] `WebView(mcp=True)` 在 Standalone 与 Embedded 都能稳定启动 Sidecar
- [ ] `@view.bind_call()` 注册的工具可被 Sidecar `tool.list` 看到
- [ ] 外部 MCP 调用能触发主线程 handler 并正确返回结果

---

## Phase 3: 构建与分发

**预计时间**: 1 week

### 任务清单

- [ ] Cargo workspace 配置
  - [ ] 添加 `auroraview-mcp-server` 到 workspace
  - [ ] 配置跨编译目标
- [ ] GitHub Actions 构建矩阵
  - [ ] Windows x86_64
  - [ ] macOS x86_64
  - [ ] macOS aarch64 (Apple Silicon)
  - [ ] Linux x86_64
  - [ ] actions 版本统一更新为 `actions/*@v6`
- [ ] 平台特定 wheel 打包
  - [ ] `scripts/build_wheels.py` 脚本
  - [ ] 二进制嵌入到 wheel
  - [ ] 平台 tag 正确设置
- [ ] PyPI 发布
  - [ ] `twine` 上传配置
  - [ ] GitHub Release 集成

### 验收标准（DoD）

- [ ] wheel 内包含对应平台的 `auroraview-mcp-server` 二进制
- [ ] 在纯 Python 环境（无额外依赖）可定位并启动 sidecar

### Wheel 结构（示意）

```
auroraview-0.5.0-cp311-cp311-win_amd64.whl
├── auroraview/
│   ├── _native/
│   │   ├── _core.pyd
│   │   └── auroraview-mcp-server.exe  ← Sidecar 二进制
│   └── mcp/
│       └── sidecar.py
└── ...
```

---

## Phase 4: 兼容性与测试

**预计时间**: 1 week

### 任务清单

- [ ] DCC 集成测试
  - [ ] Maya 2024/2025 插件测试
  - [ ] Blender 4.x 插件测试
  - [ ] Houdini 20 插件测试
- [ ] 打包工具测试
  - [ ] PyInstaller 打包测试
  - [ ] Nuitka 打包测试
  - [ ] 便携式 EXE 测试
- [ ] 跨平台 CI
  - [ ] Windows CI
  - [ ] macOS CI (Intel + Apple Silicon)
  - [ ] Linux CI (Ubuntu)
- [ ] 性能测试
  - [ ] IPC 延迟测试
  - [ ] 并发 MCP 请求测试
  - [ ] 内存使用测试

### 验收标准（DoD）

- [ ] 至少一条自动化集成测试覆盖：启动 → tool.list → tools/call → 关闭
- [ ] 连续多次启动/关闭无资源泄漏与僵尸进程

### 测试矩阵（建议）

| 平台 | Python | DCC | 打包工具 |
|------|--------|-----|----------|
| Windows | 3.7–3.12 | Maya 2024/2025 | PyInstaller |
| macOS (Intel) | 3.7–3.12 | - | Nuitka |
| macOS (ARM) | 3.7–3.12 | - | - |
| Linux | 3.7–3.12 | Blender 4.x | - |

---

## 进展记录

### 2026-01-06 (下午)

**Phase 1 完成 + Phase 2 HTTP 服务器部分完成**

- 实现 MCP Streamable HTTP 服务器 (`http/` 模块)
  - `McpSidecarService` 实现 `rmcp::ServerHandler` trait
  - `HttpServer` 使用 rmcp 的 Streamable HTTP transport
  - 工具调用通过 IPC 转发到主进程
- 添加 4 个 HTTP 集成测试
- 总计 16 个测试全部通过

### 2026-01-06

- 完成 RFC v2 重设计
- 架构从 Sidecar Thread 改为 Sidecar Process
- 更新实现计划
- 完成 Phase 1 IPC 基础设施

### 2026-01-04 (v1 - 已废弃)

- ~~添加 `PythonCallbackDeferred` 消息类型~~
- ~~实现延迟回调机制~~
- 发现时序耦合问题，决定重新设计

---

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| IPC 性能开销 | 延迟增加 | 先用 JSON-RPC；必要时可选 MessagePack 编码 |
| 进程管理复杂度 | 可靠性 | 父进程监控 + 优雅关闭 + 超时 kill |
| 二进制分发体积 | 下载大小 | 使用 strip；UPX 压缩为可选项（需验证签名/误报风险） |
| DCC Python 版本 | 兼容性 | 最低支持 Python 3.7；示例与类型注解保持 3.7 语法 |

---

## 备注

- v2 架构是 breaking internal change，但 **API 保持向后兼容**
- 优先完成 Windows 平台，其他平台可并行开发
- 未来可选支持 MCP over stdio（不依赖 HTTP），但不作为 v0.5.0 必选目标
