# RFC 0005: MCP 架构重心调整 - 优先 Embedded 模式

> **状态**: Draft  
> **开始日期**: 2026-01-07  
> **作者**: AuroraView Team  
> **相关 RFC**: [RFC 0004](./0004-mcp-sidecar-architecture.md)

## 摘要

基于 RFC 0004 实现过程中发现的稳定性问题，本 RFC 提出将 MCP 架构重心从 Sidecar 模式调整为 **Embedded 优先**，同时保留 Sidecar 作为特殊场景的可选方案。

## 背景与问题

### RFC 0004 实现中发现的关键问题

1. **Sidecar 启动协议不可靠**：Python 读 stdout 判定 ready，但 sidecar 日志写 stderr，导致启动卡死
2. **IPC 协议实现缺陷**：BufReader 重复创建导致消息丢失风险
3. **线程安全问题未解决**：Sidecar 工具调用在 IPC 后台线程执行，DCC/Unreal 环境高风险
4. **复杂度与收益不匹配**：进程管理、IPC 协议、错误处理复杂度高，但核心价值（解耦）在大多数场景下不必要

### 当前实现状态对比

| 特性 | Embedded 模式 | Sidecar 模式 |
|------|---------------|---------------|
| 稳定性 | ✅ 高（主线程执行） | ❌ 低（多个协议缺陷） |
| 性能 | ✅ 快（无 IPC 开销） | ❌ 慢（序列化开销） |
| 复杂度 | ✅ 简单 | ❌ 复杂（进程+IPC） |
| DCC 兼容性 | ✅ 安全（事件循环调度） | ❌ 危险（后台线程） |
| 生命周期 | ❌ 与 WebView 耦合 | ✅ 独立进程 |

## 提案

### 1. 架构重心调整

**新的优先级顺序**：
1. **Embedded 模式**（默认）：同进程 MCP Server，通过 WebView 事件循环确保主线程执行
2. **Sidecar 模式**（可选）：仅在特殊场景（如无 WebView 的纯 API 服务）使用

### 2. 实现策略

#### 2.1 Embedded 模式增强（Phase 1 - 立即执行）

**目标**：将 Embedded 模式打造为生产就绪的主要方案

**任务清单**：
- [x] 确保 `WebView._core.create_mcp_server()` 路径稳定可用
- [ ] 增强错误处理和日志记录
- [ ] 添加 MCP 工具执行超时机制（避免主线程卡死）
- [ ] 完善 DCC 线程安全测试（Maya/Houdini/Unreal）
- [ ] 优化工具注册和发现机制

**关键设计原则**：
```python
# 正确的使用方式（通过 WebView 创建）
webview = WebView(mcp=True)  # 自动使用 embedded 模式
webview.bind_call("my_tool", my_handler)

# 错误的使用方式（直接创建，会有线程安全问题）
server = McpServer()  # ❌ 不推荐，可能在 Tokio 线程执行
```

#### 2.2 Sidecar 模式修复（Phase 2 - 选择性执行）

**目标**：修复已知缺陷，但不作为主要推广方向

**必修问题**：
- [ ] 修复 ready 协议：sidecar 向 stdout 输出 `READY <port>`
- [ ] 修复 IPC BufReader：连接级别创建，避免 buffer 丢失
- [ ] 强制主线程执行：集成 `thread_dispatcher.run_on_main_thread_sync`

**可选增强**：
- [ ] 实现 `allowed_origins`/`require_auth` 安全机制
- [ ] 添加 MCP over stdio 支持（无 HTTP 依赖）

#### 2.3 开发者指导（Phase 3 - 文档和示例）

**目标**：提供清晰的使用指导，避免误用

**文档结构**：
```
docs/mcp/
├── README.md                 # MCP 概览和快速开始
├── embedded-mode.md          # Embedded 模式详细指南（主要）
├── sidecar-mode.md          # Sidecar 模式使用场景和注意事项
├── dcc-integration.md       # DCC 环境集成最佳实践
└── troubleshooting.md       # 常见问题和调试指南
```

### 3. 迁移路径

#### 3.1 现有用户

**当前行为保持不变**：
- `WebView(mcp=True)` 继续使用 `auto` 模式（sidecar → embedded fallback）
- 现有 `bind_call` 注册的工具继续工作

**推荐迁移**：
```python
# 旧方式（仍然支持）
webview = WebView(mcp=True)  # auto 模式

# 新方式（明确指定，推荐）
webview = WebView(mcp={"mode": "embedded"})  # 明确使用 embedded
```

#### 3.2 新用户

**默认推荐 Embedded 模式**：
```python
from auroraview import WebView

# 简单用法
webview = WebView(mcp=True)  # 自动选择最佳模式（embedded 优先）

@webview.bind_call
def my_tool(param: str) -> str:
    """My custom tool."""
    return f"Result: {param}"

webview.show()
```

## 实施计划

### Phase 1: Embedded 模式稳定化（1 周）

**目标**：确保 Embedded 模式在所有支持的 DCC 环境中稳定运行

**任务**：
- [ ] 完善 `create_mcp_server` Rust 绑定
- [ ] 添加工具执行超时机制（默认 30s）
- [ ] 增强错误处理和用户友好的错误消息
- [ ] 编写 DCC 集成测试（Maya/Houdini/Unreal）

**验收标准**：
- [ ] 在 Maya 2024/2025 中连续启动/关闭 100 次无 crash
- [ ] 在 Unreal 中 MCP 工具调用确保在 Game Thread 执行
- [ ] 工具执行超时后不影响 WebView 响应性

### Phase 2: Sidecar 模式修复（2 周，可选）

**目标**：修复已知缺陷，提供可选的解耦方案

**任务**：
- [ ] 实现 `READY <port>` stdout 协议
- [ ] 修复 IPC BufReader 生命周期
- [ ] 集成主线程调度器
- [ ] 添加安全机制（auth/origins）

**验收标准**：
- [ ] Sidecar 启动成功率 > 99%（100 次测试）
- [ ] IPC 并发测试无消息丢失
- [ ] DCC 环境中工具调用在主线程执行

### Phase 3: 文档和示例（1 周）

**目标**：提供完整的开发者指导

**任务**：
- [ ] 编写完整的 MCP 使用指南
- [ ] 创建 DCC 集成示例
- [ ] 更新 Gallery 展示 MCP 最佳实践
- [ ] 录制视频教程

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| Embedded 模式性能问题 | 低 | 中 | 添加异步工具执行支持 |
| DCC 兼容性问题 | 中 | 高 | 全面的集成测试矩阵 |
| 用户迁移阻力 | 低 | 低 | 保持向后兼容，渐进式迁移 |
| Sidecar 修复成本过高 | 中 | 低 | Phase 2 可选执行 |

## 成功指标

### 技术指标
- [ ] Embedded 模式 DCC 兼容性 > 95%
- [ ] MCP 工具调用延迟 < 100ms（P95）
- [ ] 内存使用增量 < 50MB
- [ ] 启动时间增量 < 500ms

### 用户体验指标
- [ ] MCP 相关 issue 数量减少 > 50%
- [ ] 文档满意度 > 4.5/5
- [ ] 新用户上手时间 < 10 分钟

## 决策记录

### 2026-01-07: 架构重心调整决定

**决策**：优先 Embedded 模式，Sidecar 作为可选方案

**理由**：
1. Embedded 模式稳定性和性能明显优于 Sidecar
2. 大多数用户场景不需要进程解耦
3. DCC 环境的线程安全要求使 Embedded 更适合
4. 开发和维护成本更低

**影响**：
- 重新分配开发资源优先级
- 更新文档和示例重点
- 调整测试策略

## 附录

### A. 技术架构对比

#### Embedded 模式架构
```
Frontend (JS) → WebView Bridge → Event Loop → Python Handler (Main Thread)
```

#### Sidecar 模式架构
```
Frontend (JS) → HTTP → Sidecar Process → IPC → Main Process → Python Handler (?)
```

### B. 使用场景决策树

```
需要 MCP 支持？
├─ 是 → 有 WebView 实例？
│   ├─ 是 → 使用 Embedded 模式 ✅
│   └─ 否 → 需要进程隔离？
│       ├─ 是 → 使用 Sidecar 模式（谨慎）
│       └─ 否 → 考虑其他方案
└─ 否 → 无需 MCP
```

### C. 相关资源

- [MCP 协议规范](https://modelcontextprotocol.io/)
- [AuroraView WebView API 文档](../api/webview.md)
- [DCC 集成指南](../integration/dcc.md)
- [线程安全最佳实践](../best-practices/threading.md)