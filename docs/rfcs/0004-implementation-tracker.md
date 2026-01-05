# RFC 0004 实现跟踪

> **RFC**: [0004-mcp-sidecar-architecture.md](./0004-mcp-sidecar-architecture.md)
> **开始日期**: 2026-01-04
> **目标版本**: v0.4.0

## 实现状态

| Phase | 状态 | 描述 |
|-------|------|------|
| Phase 1 | 🟡 进行中 | MessageQueue 增强 |
| Phase 2 | ⚪ 未开始 | MCP Tools 重构 |
| Phase 3 | ⚪ 未开始 | Sidecar Thread 启动 |
| Phase 4 | ⚪ 未开始 | CI/Headless 模式 |

## Phase 1: MessageQueue 增强

### 任务清单

- [x] 添加 `PythonCallbackDeferred` 消息类型
- [ ] 添加 `EvalJsWithResponse` 消息类型（带 oneshot channel）
- [ ] 实现响应超时机制
- [ ] 添加 `EmitEventWithResponse` 消息类型

### 相关文件

- `src/ipc/message_queue.rs` - 消息队列实现
- `src/webview/event_loop.rs` - 事件循环消息处理
- `src/ipc/handler.rs` - IPC 处理器

### 进展记录

**2026-01-04**
- 已添加 `PythonCallbackDeferred` 消息类型
- 已实现延迟回调机制，Python 回调在主线程执行
- 构建通过

## Phase 2: MCP Tools 重构

### 任务清单

- [ ] 重构 `eval_js` 工具
  - [ ] 使用 MessageQueue 发送消息
  - [ ] 支持同步等待响应（带超时）
- [ ] 重构 `emit_event` 工具
  - [ ] Fire-and-forget 模式
- [ ] 重构 `load_url` / `load_html` 工具
- [ ] 移除 Python 回调直接调用

### 相关文件

- `crates/auroraview-mcp/src/server.rs` - MCP 服务实现
- `crates/auroraview-mcp/src/tool.rs` - 工具注册

## Phase 3: Sidecar Thread 启动

### 任务清单

- [ ] 创建独立 Tokio Runtime
- [ ] 在 sidecar 线程启动 MCP Server
- [ ] 共享 MessageQueue 引用
- [ ] 优雅关闭机制

### 相关文件

- `src/webview/core/mod.rs` - WebView 核心
- `crates/auroraview-mcp/src/lib.rs` - MCP 入口

## Phase 4: CI/Headless 模式

### 任务清单

- [ ] 实现 `HeadlessWebView` 结构
- [ ] CI 环境自动检测
- [ ] 消息处理模拟
- [ ] 测试框架集成

### 相关文件

- `src/webview/headless.rs` (新增)
- `tests/` - 测试目录

## 测试计划

### 单元测试

- [ ] MessageQueue 消息推送/处理测试
- [ ] oneshot channel 超时测试
- [ ] 延迟回调执行测试

### 集成测试

- [ ] MCP 工具调用不阻塞测试
- [ ] 并发 MCP 请求测试
- [ ] CI headless 模式测试

### E2E 测试

- [ ] Gallery 应用 MCP 功能测试
- [ ] DCC 集成测试（Maya/Blender）

## 备注

- 保持向后兼容，不引入 breaking changes
- 优先实现 fire-and-forget 模式
- CI 模式可后续迭代

