## Architecture Diagnosis Report Template

> 将本模板复制到目标输出文档（默认：`architecture-diagnosis.md`），再按项目实际增删。

### 1) 系统边界与模块划分（Subsystems & Boundaries）
- **Rust**
  - 运行时（runtime/webview/ipc）
  - 可复用底座（core/assets/backend abstraction）
  - 插件系统（plugins/scope）
  - MCP（embedded server）
  - CLI/Pack
- **Python**
  - 高阶 API（WebView/AuroraView）
  - mixins（api/js/dom/events）
  - 集成层（Qt/DCC）
- **JS**
  - SDK（window namespace / call/on/trigger）
  - 注入脚本与版本同步策略
- **Apps**
  - Gallery / examples
- **Testing**
  - Rust tests / Python tests / E2E

### 2) 关键约束与运行模式（Constraints & Run Modes）
- **约束**：平台、宿主、线程模型、依赖限制（如 Python 3.7、零依赖）。
- **Run Mode 矩阵**（表格）：
  - mode 名称
  - event loop 归属
  - UI 线程
  - IPC drain 机制
  - 支持能力与限制
- **启动时序与就绪信号（Boot Order & Readiness）**：
  - 哪些组件需要先后顺序（event loop / proxy / timer / server / bridge）
  - 是否有明确 ready 事件/屏障（例如 event loop ready 后再启动 server）
  - 如果没有 ready 信号，列出竞态窗口与短期止血方案


### 3) 发现清单（Findings）

#### P0（必须优先处理）
- **问题标题**
  - **证据**：文件/模块/函数/配置 key
  - **风险**：表现、影响面、概率
  - **短期建议**：最小补丁
  - **中期建议**：结构收敛/重构方向

#### P1（高维护成本/易回归）
（同上结构）

#### P2（质量/体验/可观测性）
（同上结构）

### 4) 方案候选与取舍（Options & Trade-offs）

> 目的：为团队提供 2–4 个可选改造方案，并明确 trade-off，避免“只有一个方向”。

- **候选方案列表**（建议按 A/B/C/D 形式编号）：
  - **方案 A（最小改动/止血）**：一句话描述。
  - **方案 B（统一机制/中等改动）**：一句话描述。
  - **方案 C（补齐时序/就绪屏障）**：一句话描述。
  - **方案 D（强解耦/Sidecar/长期）**：一句话描述。

- **Trade-off 表**（示例字段，可按项目裁剪）：

| 方案 | 核心思路 | 适用前提/边界 | 优点 | 代价/风险 | 复杂度 | 实施周期 | 验收标准（必须可验证） |
|------|----------|---------------|------|-----------|--------|----------|------------------------|
| A | | | | | 低/中/高 | 天/周/月 | |
| B | | | | | 低/中/高 | 天/周/月 | |
| C | | | | | 低/中/高 | 天/周/月 | |
| D | | | | | 低/中/高 | 天/周/月 | |

- **推荐路径（Recommendation）**：
  - **推荐采用**：方案 X（以及原因：风险/收益/团队成本/兼容性）。
  - **组合策略（可选）**：短期先 A/C 止血 + 中期落地 B + 长期准备 D。
  - **退出标准（Exit Criteria）**：何时算“方案成功/可以停止继续重构”。

### 5) 渐进式重构路线（Roadmap）
- **Phase 0（止血）**：收敛入口/统一超时/禁止协议污染
- **Phase 1（结构收敛）**：统一消息处理、模式显式化、状态机化
- **Phase 2（平台化）**：schema 生成、插件与 MCP 统一能力模型

每个 phase 写清：
- 目标
- 变更范围
- 验收标准（可测试/可观测的指标）

### 6) 扩展点与应用场景（Extensions & Scenarios）
- DCC 工具面板/多窗口
- 权限与审计
- 自动化测试（CDP/Playwright/Headless）
- AI/MCP 融合
- 打包分发与版本管理

每条至少写：现有支点、缺口、落地关键改造。

### 7) 关键入口索引（Key Entry Points）
- runtime / ipc / bridge / plugin / mcp / sdk / gallery / tests 的关键文件索引
