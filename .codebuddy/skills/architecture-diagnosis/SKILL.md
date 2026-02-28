---
name: architecture-diagnosis
description: |
  This skill produces an actionable architecture diagnosis report for a codebase.
  It guides the AI to scan the repository, map subsystem boundaries, identify design flaws
  with concrete evidence (files/functions), assess risks, and propose a phased refactor roadmap
  plus future extension scenarios.
  Use this skill when the team needs a "diagnosis report" to decide whether/how to refactor.
---

# Architecture Diagnosis

生成一份可用于评审与重构决策的“架构诊断书”，强调 **证据链、风险分级、渐进式改造路线**，避免泛泛而谈。

## 适用场景（When to Use）

- 对某个分支/版本进行 **设计缺陷盘点**，判断是否需要重构。
- 出现线程/事件循环/生命周期相关的疑难问题，需要从架构层面解释根因与系统性风险。
- 项目进入“功能膨胀期”，需要固化模块边界、配置规范、可观测性与测试策略。
- 新成员 onboarding，需要一份 **系统边界图 + 风险清单 + 可扩展方向**。

## 输入与约束（Inputs & Constraints）

在开始前，优先收集：
- 当前分支/目标版本（例如 PR/commit/branch）。
- 目标平台与宿主（Windows / macOS / Linux；DCC/Qt/Standalone）。
- 关键约束（例如：必须兼容 Python 3.7、必须零 Python 依赖、UI 必须主线程等）。

如果缺少关键约束，最多询问 1–2 个问题；其它信息尽量通过仓库搜索获得。

## 产出（Deliverables）

默认产出一个 Markdown 文档（推荐命名：`architecture-diagnosis.md`），包含以下结构：

1. **系统边界图（Subsystems & Boundaries）**：列出主要子系统、职责、依赖方向。
2. **关键约束与运行模式（Constraints & Run Modes）**：明确线程模型、消息泵归属、打包/部署模型。
3. **缺陷清单（Findings）**：按严重度分级（P0/P1/P2），每条必须包含：
   - 证据（文件/模块/函数/配置项）
   - 风险后果（会导致什么、影响谁、发生概率）
   - 建议（短期补丁 vs 中期重构）
4. **方案候选与取舍（Options & Trade-offs）**：给出 2–4 个可选方案，输出 trade-off 表，并给出推荐路径与验收标准。
5. **重构路线（Roadmap）**：按阶段列出可渐进落地的里程碑与验收标准。
6. **扩展点与应用场景（Extensions & Scenarios）**：未来可增长的方向与可复用能力。
7. **关键入口索引（Key Entry Points）**：后续排查与重构的主要文件/模块索引。


## 工作流程（Workflow）

### Step 1：做仓库“快速体检”与技术栈识别

执行以下动作：
- 列出根目录关键文件（例如：`Cargo.toml`、`pyproject.toml`、`package.json`、`justfile`、CI 配置）。
- 搜索并确定：
  - Rust workspace/主要 crates
  - Python 包入口与高阶 API（例如 `WebView` / `AuroraView`）
  - JS SDK 与注入脚本（例如 `window.*` 命名空间）
  - 示例与端到端应用（如 Gallery）
  - 测试组织策略（`tests/`、fixture、integration tests）

输出一个“组件地图（组件→职责→依赖）”草稿。

### Step 2：建立“运行模式与线程模型”真相表

必须回答清楚：
- 事件循环由谁拥有（宿主 Qt / 自己的 event loop / Win32 message pump）
- UI 操作必须在哪个线程（STA/main thread）
- IPC/消息队列如何被 drain（主动唤醒 vs timer/host pump）
- 关闭流程与资源释放由谁控制（Drop/GC/显式 close/state machine）
- 启动时序与就绪信号（谁先启动：事件循环/消息队列 proxy/timer/服务端；有没有明确的 ready 事件/屏障）

将这些写成一个 **Run Mode 矩阵表**（例如：StandaloneBlocking / StandaloneThreaded / EmbeddedHostPump / IPCOnly / PackedHeadless）。


### Step 3：扫描“架构异味”并固化证据

按下面的“高命中”模式搜索与归档证据：
- 重复实现/重复分支：同类消息/事件在多个文件重复处理。
- 跨层耦合：JS→Rust→Python→Rust 往返或层间依赖反向。
- 默认值/语义漂移：Rust/Python/JS 对同一参数不同默认或不同含义。
- 生命周期补丁：先构造再回填（`new_without_*` / `set_*`），或 Drop 中做破坏性动作。
- 超时/取消语义缺失：前端 Promise/回调 manager/服务端 timeout 不一致。
- 可观测性混杂：`tracing`/`logging`/`print` 混用；协议通道 stdout 被污染。
- 跨平台声明与现实不符：非 Windows/非 Qt 场景只“能编译但不可用”。

要求：每条发现都在文档中提供“证据定位”（文件/函数/配置项），不要只写结论。

### Step 4：给出“短期补丁 vs 中期重构”方案

对每个 P0/P1 问题：
- 给出 **短期修复**：最小改动、可快速降低风险。
- 给出 **中期重构**：如何收敛结构（去重复、状态机化、模式显式化、配置 schema 统一等）。

短期修复里，优先考虑这类“保守且可验证”的措施：
- 为关键子系统增加 **ready 信号/屏障**（例如 event loop ready 后再启动服务）。
- 确保队列 drain 是 **自驱动的**（不依赖“用户点击才迭代”）：
  - 主循环主动轮询 drain
  - 或有明确 tick（Timer/UserEvent::Tick）保障迭代

然后，输出一个独立章节 **“方案候选与取舍（Options & Trade-offs）”**：
- 给出 2–4 个可选方案（建议按 A/B/C/D 命名）。
- 用 trade-off 表对比：适用前提/边界、收益、代价/风险、复杂度、周期、可验证的验收标准。
- 给出推荐路径：短期止血 vs 中期收敛 vs 长期强解耦（可组合）。

重点：避免“大爆炸重写”，优先建议 **渐进式收敛**（先统一入口、再抽象策略、最后拆层）。



### Step 5：补充未来扩展点与应用场景

从仓库已有支点出发，列出可扩展方向，例如：
- DCC 工具面板/多窗口/多实例发现
- 插件体系与权限模型（capabilities/scopes）
- 自动化测试（CDP/Playwright/Headless backend）
- AI/MCP 集成（embedded vs external 的职责分工与融合路线）
- 打包分发（pack、资源管理、版本策略）

每个扩展点至少回答：现有支点是什么、缺口是什么、落地需要的关键改造。

## 证据规范（Evidence Rules）

- 给出具体定位：至少到文件名 + 模块/函数名；能给到配置 key 更好。
- 把“影响路径”写清：问题从哪里触发、经过哪些层、最终表现为何。
- 避免“凭感觉”的描述；如果没有证据，标注为假设并说明验证方法。

## 推荐搜索清单（Grep Patterns）

参考 `references/checklist.md` 中的模式与关键词，优先搜索：
- `TODO|TEMPORARY FIX|HACK|FIXME`
- `Drop|DestroyWindow|PostMessage|PeekMessage`
- `event_loop|message_pump|process_events|process_ipc_only`
- `timeout|DEFAULT_.*TIMEOUT|oneshot|await`
- `direct_execution|MessageQueue|dispatcher|proxy`

## 输出模板

使用 `references/templates.md` 的模板结构生成最终文档，并根据项目实际删减/补充。
