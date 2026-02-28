## Architecture Mitigation/Refactor Patterns (Common)

> 这些是诊断时经常会提出的“解耦/止血”模式，供写短期修复与中期重构建议时复用。

### 1) Ready Signal / Barrier（就绪信号/屏障）
- **动机**：两个子系统独立启动但存在隐式依赖（典型：server 需要 event_loop_proxy 或需要队列 drain）。
- **做法**：
  - 在 event loop 启动后发出 `ready` 事件（或设置一次性 latch），再启动依赖方。
  - 诊断文档里要明确：没有 ready 时会发生什么竞态窗口。

### 2) Self-Driven Drain（队列 drain 自驱动）
- **动机**：症状是“只有点击 UI 才响应”，通常说明 drain 依赖外部输入。
- **做法**：
  - 主循环里主动 drain（例如每次 `MainEventsCleared` 都尝试处理队列）。
  - 或增加显式 tick（Timer / `UserEvent::Tick`）确保循环迭代。

### 3) Split Execution Paths（按工具性质拆执行路径）
- **动机**：某些工具不触 UI，可直接在后台执行；触 UI 的必须回主线程。
- **做法**：
  - 引入标记（capability/attribute）决定 tool 执行线程：
    - `direct_execution=true`（非 UI）
    - `dispatch_to_main=true`（UI/宿主约束）
  - 文档要强调：默认值选择应以 DCC 安全为准。

### 4) Sidecar Process（推荐的强解耦模式）
- **动机**：跨语言/线程边界太多，且 server 生命周期与 UI 强耦合。
- **做法**：
  - 将 server 独立为子进程（或外部进程），主进程仅提供受控 IPC（JSON-RPC/HTTP）。
  - 优势：隔离崩溃、隔离 GIL、易调试、可重启。
  - 代价：部署复杂度上升、需要 capability/授权模型。

### 5) Single Source of Truth（配置/超时单一来源）
- **动机**：Rust/Python/JS 默认值漂移，导致回调/超时撕裂。
- **做法**：
  - 从 Rust config/schema 生成 Python 与 TS 定义。
  - timeout/capabilities 在注入脚本与后端共享。
