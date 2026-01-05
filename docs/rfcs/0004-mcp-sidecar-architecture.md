# RFC 0004: MCP Sidecar Thread 架构

> **状态**: Draft
> **作者**: AuroraView Team
> **创建日期**: 2026-01-04
> **目标版本**: v0.4.0

## 摘要

本 RFC 提议将 MCP Server 重构为 "Sidecar Thread" 架构，使其在独立线程中运行 Tokio Runtime，通过 MessageQueue 与主线程 Event Loop 通信。这种架构解决了当前 MCP 请求可能被 UI 阻塞的问题，同时支持 CI 环境下的 headless 测试。

**核心目标**：
1. **非阻塞 MCP**：MCP 服务永远不会被 UI 阻塞
2. **单进程架构**：无需额外进程，作为 sidecar 线程运行
3. **IPC 通信**：复用现有 MessageQueue 机制
4. **CI 支持**：支持 headless 模式进行 UI 自动化测试
5. **线程安全**：Python 回调在主线程执行，满足 DCC 要求

## 动机

### 当前问题

当前 MCP 实现存在以下问题：

1. **GIL 阻塞**：Python 回调需要在主线程执行，可能阻塞 MCP 请求处理
2. **线程安全**：DCC 应用（Maya/Blender 等）要求 UI 操作在主线程执行
3. **响应延迟**：MCP 工具执行等待 Python 回调完成时，整个请求被阻塞

### 目标

```
AI Agent ──MCP Protocol──> Sidecar Thread ──MessageQueue──> Main Thread
                                                                │
                                                          Event Loop
                                                                │
                                                           WebView/UI
```

## 设计方案

### 架构概览

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Single Process                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────────────┐    ┌─────────────────────────────┐   │
│  │   Sidecar Thread         │    │      Main Thread             │   │
│  │   (Tokio Runtime)        │    │      (Event Loop)            │   │
│  │                          │    │                              │   │
│  │  ┌────────────────────┐  │    │  ┌────────────────────────┐ │   │
│  │  │   MCP Server       │  │    │  │    Event Loop          │ │   │
│  │  │   (axum + rmcp)    │  │    │  │    (tao)               │ │   │
│  │  └────────┬───────────┘  │    │  └───────────┬────────────┘ │   │
│  │           │              │    │              │              │   │
│  │  ┌────────▼───────────┐  │    │  ┌───────────▼────────────┐ │   │
│  │  │   Tool Registry    │  │    │  │     WebView            │ │   │
│  │  │   - eval_js        │  │    │  │     (wry)              │ │   │
│  │  │   - emit_event     │  │    │  └───────────┬────────────┘ │   │
│  │  │   - load_url       │  │    │              │              │   │
│  │  └────────┬───────────┘  │    │  ┌───────────▼────────────┐ │   │
│  │           │              │    │  │    IPC Handler         │ │   │
│  │           │              │    │  │    (Python Callbacks)  │ │   │
│  └───────────┼──────────────┘    │  └────────────────────────┘ │   │
│              │                    │              ▲              │   │
│              │  ┌─────────────────┴──────────────┤              │   │
│              │  │                                │              │   │
│              ▼  ▼                                │              │   │
│  ┌────────────────────────────────────┐         │              │   │
│  │        Message Queue               │─────────┘              │   │
│  │   (Arc<Mutex<VecDeque<Message>>>)  │                        │   │
│  └────────────────────────────────────┘                        │   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 消息类型

```rust
pub enum WebViewMessage {
    // 现有消息类型
    EvalJs(String),
    EvalJsAsync { script: String, callback_id: u64 },
    EmitEvent { event_name: String, data: Value },
    LoadUrl(String),
    LoadHtml(String),
    SetVisible(bool),
    Reload,
    StopLoading,
    Close,
    WindowEvent { event_type: WindowEventType, data: Value },
    
    // 新增：带响应通道的消息（用于需要返回值的 MCP 工具）
    EvalJsWithResponse {
        script: String,
        response_tx: oneshot::Sender<Result<Value, String>>,
    },
}
```

### MCP Tool 执行流程

#### Fire-and-Forget 模式（推荐）

```
AI ──call_tool("emit_event")──> MCP Server
                                    │
                                    ▼
                            push EmitEvent to MessageQueue
                                    │
                                    ▼
                            return Ok({"status": "queued"})
                                    │
        <───────────────────────────┘
        
                            (异步处理)
                                    │
                            Main Thread polls MessageQueue
                                    │
                                    ▼
                            WebView.emit(event, data)
```

#### Request-Response 模式（需要返回值时）

```
AI ──call_tool("eval_js")──> MCP Server
                                    │
                                    ▼
                            create oneshot channel
                            push EvalJsWithResponse to MessageQueue
                                    │
                                    ▼
                            await response_rx (with timeout)
                                    │
        <───────────────────────────┘
        
                            Main Thread polls MessageQueue
                                    │
                                    ▼
                            WebView.eval_script(js)
                                    │
                                    ▼
                            send result via response_tx
```

### CI/Headless 支持

```rust
pub struct HeadlessWebView {
    message_queue: Arc<MessageQueue>,
    // 模拟 WebView 行为，用于 CI 测试
}

impl HeadlessWebView {
    pub fn process_messages(&self) {
        // 处理消息但不实际渲染
        // 可以记录操作日志用于断言
    }
}
```

## 实现计划

### Phase 1: MessageQueue 增强

- [ ] 添加 `EvalJsWithResponse` 消息类型
- [ ] 实现 oneshot channel 支持
- [ ] 添加超时机制

### Phase 2: MCP Tools 重构

- [ ] 重构 `eval_js` 工具使用消息队列
- [ ] 重构 `emit_event` 工具使用消息队列
- [ ] 移除 Python 回调的直接调用

### Phase 3: Sidecar Thread 启动

- [ ] 在独立线程启动 Tokio Runtime
- [ ] MCP Server 在 sidecar 线程运行
- [ ] 通过 MessageQueue 与主线程通信

### Phase 4: CI/Headless 模式

- [ ] 实现 HeadlessWebView
- [ ] CI 环境自动检测
- [ ] 测试框架集成

## 兼容性

### 向后兼容

- 现有 API 保持不变
- `WebView(mcp=True)` 继续工作
- Python 回调仍在主线程执行

### Breaking Changes

无

## 替代方案

### 方案 A: 独立进程

使用独立进程运行 MCP Server，通过 Unix Socket/Named Pipe 通信。

**优点**：完全隔离
**缺点**：进程管理复杂，部署困难

**决定**：不采用，用户明确要求不增加额外进程。

### 方案 B: 异步回调

使用 Python asyncio 处理回调。

**优点**：不阻塞主线程
**缺点**：DCC 应用不支持 asyncio，需要在主线程执行

**决定**：不采用，不满足 DCC 集成需求。

## 参考资料

- [RFC 0001: AuroraView MCP Server](./0001-auroraview-mcp-server.md)
- [RFC 0002: 嵌入式 MCP Server](./0002-embedded-mcp-server.md)
- [tao Event Loop](https://github.com/nicegui-org/nicegui)
- [wry WebView](https://github.com/nicegui-org/nicegui)

