### AuroraView 全项目架构诊断书（供重构决策/路线规划）

> 目标：帮助后续开发者判断“哪些问题必须修、哪些值得重构、怎么渐进落地”，并列出未来可扩展方向与应用场景。
> 
> 背景约束（重要）：Windows 优先、DCC 宿主（Qt/PySide）拥有消息泵、WebView2/STA 线程要求、Python 3.7 兼容、仅单一 `.pyd` 交付。

---

## 1. 系统边界与当前模块划分（你现在拥有什么）

### 1.1 Rust（Workspace）
- **运行时 + PyO3 bindings（根 crate）**：`src/webview/*`、`src/ipc/*`、`src/webview/core/main.rs`
  - 负责 WebView 的生命周期、消息队列、事件循环/消息泵、Python 暴露接口。
- **可复用底座（auroraview-core）**：`crates/auroraview-core/*`
  - assets（JS/HTML）、backend 抽象、ipc 抽象、service discovery 等。
- **插件系统（auroraview-plugins）**：`crates/auroraview-plugins/*`
  - `plugin:<plugin>|<cmd>` 命令体系、scope 权限模型。
- **embedded MCP（auroraview-mcp）**：`crates/auroraview-mcp/*`
  - 内置 HTTP MCP server，服务于 AI/工具调用。
- **CLI/Pack**：`crates/auroraview-cli/*`、`crates/auroraview-pack/*`
  - 分发/打包/工具链。

### 1.2 Python（对用户最直接的 API 层）
- 高阶入口：`python/auroraview/core/webview.py`（`WebView`）
  - 负责模式识别（standalone/embedded/packed）、show 路径、事件 timer、MCP 启停、API 混入等。
- mixins（API/JS/DOM/事件等）：`python/auroraview/core/mixins/*`
- MCP 默认工具集：`python/auroraview/mcp/default_tools.py`

### 1.3 JS SDK（NPM 包 + 注入脚本上游）
- `packages/auroraview-sdk/*`
  - `inject/event_bridge.ts` 提供 `window.auroraview` 的桥接与事件/调用模型。
  - 该 SDK 同时也是 Rust 内置注入 assets 的来源之一（需要保持一致）。

### 1.4 Gallery（能力展示/集成验证/打包模板）
- `gallery/*`：Vite/React 前端 + Python 后端示例，覆盖插件、多窗口、MCP/CDP 等能力。

### 1.5 外置 MCP（与 embedded MCP 并存的另一条路线）
- `packages/auroraview-mcp/*`：外置 MCP（通常结合 CDP/自动化/发现）

---

## 2. 核心问题画像（为什么开发者会觉得“线程安全与设计问题很严重”）

AuroraView 当前处于“能力快速扩张阶段”，表现为：
- **运行模式多**（Standalone blocking / Standalone threaded / Embedded host-pump / IPC-only / Packed headless），但“模式边界”主要靠注释、默认值与隐式约定维持。
- **跨层耦合重**（JS → Rust → Python → Rust 往返、Python 里承担太多 runtime 决策）。
- **消息处理逻辑未收敛**（同类消息在不同文件/分支重复实现）。
- **配置/默认值不一致**（Rust/Python/JS 对同一概念使用不同默认值或不同语义）。

这些会导致典型症状：
- “某功能只在点击 UI 时才响应”（队列无人 drain / pump 依赖被隐藏）；
- “偶现死锁/超时/回调丢失”（跨线程 + 超时策略不一致）；
- “修一个 bug 需要改 3 个文件、且容易回归”（重复逻辑 + 模式分裂）。

---

## 3. 设计缺陷 / 架构问题清单（带证据、影响、建议）

> 严重度：P0（高概率造成卡死/崩溃/数据错乱）→ P1（高维护成本/易回归）→ P2（质量与体验问题）。

### 3.1 P0：消息处理逻辑重复，统一层“写了但没接上”
- **证据**：`src/webview/message_processor.rs` 明确写当前逻辑仍在 `event_loop.rs` 与 `backend/native.rs` 重复；同时 `webview_inner.rs` 自己也在处理队列。
- **影响**：同一 `WebViewMessage` 在不同模式/分支行为不同；修复 MCP/事件/关闭流程时极易回归。
- **建议**：
  - **短期**：将所有路径统一调用 `message_processor::process_message_queue`（至少收敛 80%）。
  - **中期**：抽象“Standalone vs Embedded vs IPC-only”策略，减少 if/else 分叉。

### 3.2 P0：线程模型与运行模式边界不清（隐式约定过多）
- **证据**：Python `WebView.show()` 根据 `parent`、`wait`、packed mode 自动切换，并存在 `_show_non_blocking()` 在后台线程重新创建 core 的路径（`python/auroraview/core/webview.py`）。
- **影响**：
  - 在 DCC 宿主中，UI 必须在主线程；如果错误走后台线程或错误依赖 `_async_core`，会导致死锁/无响应。
  - API 调用者不清楚哪些方法必须 pump/tick 才生效。
- **建议**：
  - **短期**：引入不可变“运行模式枚举”（例如 `RunMode`），在构造期确定并记录，关键 API 对不合法调用给出 fail-fast 错误提示。
  - **中期**：把 runtime 状态机收敛到 Rust（Python 只做驱动/适配），减少 Python 承担底层生命周期决策。

### 3.3 P0：生命周期临时补丁（先 new_without_webview 再回填）说明对象所有权不稳
- **证据**：`src/webview/webview_inner.rs`、`src/webview/backend/native.rs` 的 `run_event_loop_blocking()` 存在 `EventLoopState::new_without_webview(...)` 再 `set_webview(...)` 的“临时修复”。
- **影响**：生命周期竞态窗口变多，未来多窗口/托盘/后台任务更难维护。
- **建议**：
  - **短期**：让 `EventLoopState` 直接持有必要资源（例如 `Arc<Mutex<...>>`），删除“先构造后注入”的初始化路径。
  - **中期**：拆分不可变配置与可变运行态，减少 Option 字段。

### 3.4 P0：Windows 消息泵与 DestroyWindow 的干预风险（尤其 embedded/Qt）
- **证据**：`src/webview/message_pump.rs`（PeekMessage/DestroyWindow/PostMessage）、`src/webview/event_loop.rs` 主动 pump、`webview_inner.rs` Drop 中 DestroyWindow。
- **影响**：宿主（Qt/DCC）拥有消息泵时，重复 pump/销毁可能干扰宿主或触发偶现崩溃；Drop 中做破坏性操作会导致释放时机不可控。
- **建议**：
  - **短期**：Embedded/IPC-only 模式严格禁止全局 pump，只允许目标 hwnd；Drop 不做 Destroy，改为仅发 Close 请求。
  - **中期**：明确“HostOwnedPump vs SelfPump”策略并固化关闭状态机。

### 3.5 P1：超时策略跨层不一致（JS 30s vs Rust 5s）导致回调撕裂
- **证据**：`packages/auroraview-sdk/src/inject/event_bridge.ts` 默认 30s；`src/ipc/js_callback.rs` 默认 5s。
- **影响**：前端已超时/后端仍在执行；或后端先清理 pending callback，导致结果回传丢失。
- **建议**：
  - **短期**：将 timeout 设为单一来源（Rust config 注入到 JS）；并允许按 call/tool 覆盖。
  - **中期**：引入取消语义与请求生命周期协议（request id、timeout、cancel）。

### 3.6 P1：IPC 节流/批处理参数语义不一致（同名多义、默认互相打架）
- **证据**：Rust `WebViewConfig` 默认 `ipc_batch_size=10`，Python 默认 `ipc_batch_size=0`（0=unlimited）；`MessageQueueConfig.batch_interval_ms=16` 与 backend tick 上限也在不同层定义。
- **影响**：性能问题难定位，调参效果不可预测。
- **建议**：
  - **短期**：拆分并重命名（例如 `max_messages_per_tick`/`queue_flush_interval_ms`/`wake_batch_ms`），统一 0/None 语义。
  - **中期**：配置 schema 自动生成（Rust→Python→TS）减少漂移。

### 3.7 P1：事件分发机制不统一（`auroraview.trigger` vs `CustomEvent`）
- **证据**：`src/webview/message_processor.rs` 仍使用 `CustomEvent`；而 `src/webview/js_assets.rs`/内置模板使用 `window.auroraview.trigger`。
- **影响**：两套订阅体系并存，某些事件“有人收不到”，在注入时序变化下表现为随机 bug。
- **建议**：
  - **短期**：统一跨语言事件只用 `window.auroraview.trigger`。
  - **中期**：固化桥协议（schema+版本），CI 校验注入脚本与 SDK 同步。

### 3.8 P1：JS plugin invoke 路径绕行 Python（JS→Rust→Python→Rust），耦合重且性能差
- **证据**：Rust 将 invoke 事件转成 Python `__plugin_invoke__`；Python 再调用 Rust `PluginManager.handle_command()`；结果通过 `__invoke_result__` 回 JS。
- **影响**：多次序列化、GIL、异常路径难保证 promise 可靠；插件系统边界不清。
- **建议**：
  - **短期**：统一 `PluginResponse` 格式与错误模型，减少 Python 自行拼 payload。
  - **中期**：Rust runtime 直接完成 invoke→response，Python 仅做授权/事件回调。

### 3.9 P1：MCP 双轨并存但边界不清（embedded MCP vs 外置 MCP）
- **证据**：`crates/auroraview-mcp` 与 `packages/auroraview-mcp` 同时存在，能力部分重叠。
- **影响**：开发者不清楚该用哪条路线，工具命名/协议/错误模型可能漂移。
- **建议**：
  - **短期**：明确职责边界（embedded：本地 API 与轻量工具；external：CDP/UI 自动化与跨进程发现）。
  - **中期**：共享工具 schema + capability negotiation（同一 tool 定义可由不同执行端实现）。

### 3.10 P2：可观测性风格混杂（tracing/logging/print），且 packed mode 存在 stdout 污染风险
- **证据**：Rust 有大量“带文本前缀”的日志；Python mixins 里存在 `print("[AuroraView DEBUG] ...")`；packed mode 使用 stdin/stdout 协议时易被污染。
- **影响**：CI/线上难解析；协议输出被污染会导致“看似随机的通信失败”。
- **建议**：
  - **短期**：库代码禁用 print；packed 模式 stdout 仅输出协议，日志一律 stderr。
  - **中期**：统一结构化日志字段与 trace/span。

---

## 4. 重构建议（如何“渐进式”降低风险，而不是大爆炸重写）

### 4.1 近期（1–2 周）“止血型”改造
- **收敛消息处理**：把 `WebViewMessage` 的处理尽可能集中到 `src/webview/message_processor.rs`，减少多处重复。
- **模式显式化**：在 Rust/Python 暴露 `RunMode`，在创建时锁定；核心 API 根据模式做 fail-fast。
- **跨层超时统一**：将 call/callback timeout 做成单一来源并注入 JS。
- **禁用 print + 统一日志出口**：packed 模式严禁 stdout 被日志污染。

### 4.2 中期（1–2 月）“结构型”改造
- **Rust runtime 状态机**：把“pump/queue/close/lifecycle”收敛到 Rust 状态机，Python 层只驱动/适配。
- **HostOwnedPump/IPC-only 固化**：把 Qt/DCC 作为一等场景，形成标准适配模板（定时器/idle 回调/窗口生命周期）。
- **插件 invoke 去 Python 中转**：Rust 直接响应 invoke，Python 只提供可选授权与事件订阅。

### 4.3 长期（3–6 月）“平台化”方向
- **统一 schema 生成**：配置/协议/tool 定义从 Rust schema 生成 Python 与 TS，减少默认漂移。
- **MCP 双路线融合**：capability negotiation + shared tool schema，支持“同一 tool 定义、多执行端实现”。

---

## 5. 未来可扩展点与应用场景（基于现有代码的“自然生长方向”）

### 5.1 DCC 工具面板与工作流
- Maya/Houdini/Nuke/3dsMax 的标准化面板（Dock/Panel），统一嵌入与生命周期。
- Pipeline 内工具（资产浏览器、发布器、检查器、渲染提交、LookDev）可直接用 React/Vite 构建 UI。

### 5.2 权限/安全（企业落地关键）
- 以 `scope` 为核心的 capability 系统（文件/进程/网络/剪贴板等）+ 审计日志。
- 工具签名与脚本保护（配合 `aurora-protect`/pack）。

### 5.3 自动化与测试
- 基于 `remote_debugging_port` + CDP 的 UI 自动化：回归测试、录制回放、性能采样。
- Headless backend 扩展为“正式 automation backend”，让 CI 不依赖真实 WebView2/窗口环境。

### 5.4 AI/MCP 集成
- embedded MCP：低延迟、本地 API 与状态查询。
- external MCP：跨进程发现 + UI 自动化（CDP）+ 多实例调度。
- 未来：统一 tool schema，Agent 可选择对“当前窗口 / 指定 DCC 实例 / 远端机器”执行。

### 5.5 多窗口/多文档与协作
- child_webview 多窗口模型（已存在 `NewWindowMode`），可扩展到多面板、多视口。
- service discovery + session 管理可扩展到多实例协作（例如渲染农场/远程 DCC）。

---

## 6. 给开发者的“判断是否要重构”的快速准则

- 如果你频繁遇到：
  - “必须点一下 UI 才响应”、
  - “偶发卡死/超时/回调丢失”、
  - “修一个 bug 牵扯多个文件与模式分支”，

  那么建议优先推进：
  - **消息处理收敛（3.1）** + **模式显式化（3.2）** + **超时统一（3.5）**。

- 如果你计划扩展：
  - 更复杂的插件体系、更强的 MCP/自动化、更多 DCC 集成，

  那么建议尽早推进：
  - **Rust runtime 状态机收敛** + **插件 invoke 去 Python 中转** + **schema 生成**。

---

### 附：关键文件索引（排查/重构入口）
- **Rust runtime**：`src/webview/event_loop.rs`、`src/webview/webview_inner.rs`、`src/webview/backend/native.rs`、`src/webview/message_processor.rs`
- **IPC/MQ**：`src/ipc/message_queue.rs`、`src/ipc/js_callback.rs`、`src/ipc/mcp_dispatcher.rs`
- **Python 高阶入口**：`python/auroraview/core/webview.py`
- **JS bridge**：`packages/auroraview-sdk/src/inject/event_bridge.ts`
- **embedded MCP**：`crates/auroraview-mcp/src/server.rs`
- **external MCP**：`packages/auroraview-mcp/README.md`
