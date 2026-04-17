# AGENTES

面向本仓库 AI/自动化代理与贡献者的执行约定。

## 1. 命令与环境约束

- 所有工具命令统一通过 `vx` 执行，不直接调用裸命令。
- 任务编排统一使用 `justfile`，入口为 `vx just <task>`。
- 常用命令示例：
  - `vx just build`
  - `vx just test`
  - `vx just lint`
  - `vx python`
  - `vx uv`
  - `vx cargo`
  - `vx git`
  - `vx npm` / `vx npx`

## 2. 开发与校验流程（建议）

在提交前，优先按以下顺序执行：

1. `vx just format`
2. `vx just lint`
3. `vx just test`
4. `vx just build`

要求本地与 CI 使用同一套 `just` 入口，避免“本地通过、CI 失败”的流程分叉。

## 3. 兼容性与实现约束

- Python 代码需兼容 **Python 3.7+**（DCC 环境优先）。
- 优先使用成熟依赖与业内标准方案，避免重复造轮子。
- Rust 测试尽量放在各 crate 的 `tests/` 目录，优先采用 `rstest`。

## 4. WebView / DCC 项目约定（摘要）

- Windows 优先 WebView2 后端（Rust + `webview2-com`）。
- DCC 嵌入场景由 Qt 宿主负责事件循环，Rust 不接管 Qt 消息泵。
- 前端统一使用 `window.auroraview` 命名空间与桥接协议。

## 5. E2E 测试与可视化验证

本项目使用 **ProofShot** + **agent-browser** 进行 E2E 测试和自我迭代。

### 工具链

| 工具 | 职责 | 安装 |
|------|------|------|
| [ProofShot](https://github.com/AmElmo/proofshot) | 会话录制、截图、错误收集、PR 证据上传 | `npm install -g proofshot` |
| [agent-browser](https://github.com/vercel-labs/agent-browser) | 无头浏览器控制（CDP） | 随 ProofShot 一同安装 |

### 常用命令

```bash
# 安装工具链
vx just e2e-install

# 快速 E2E 验证（打包 Gallery + CDP 测试 + 截图证据）
vx just e2e-proofshot

# 自我迭代循环（检测 → 分析 → 修复 → 重新验证）
vx just e2e-iterate

# 上传验证证据到 PR
vx just e2e-pr

# 手动探索（交互式快照）
vx just e2e-snapshot
```

### 自我迭代流程

Agent 在开发过程中应遵循如下 E2E 自我迭代循环：

1. **构建** → `vx just gallery-pack-debug`
2. **启动** → `vx just e2e-start`（启动 Gallery + 等待 CDP）
3. **验证** → `vx just e2e-snapshot` / `vx just e2e-screenshot`
4. **分析** → 审查截图、控制台错误、SUMMARY.md
5. **修复** → 根据发现修改代码
6. **重复** → 回到步骤 1，直到所有检查通过
7. **记录** → 将非显而易见的发现记录到 `.learnings/`

### E2E 验证清单

- Gallery 无控制台错误启动
- 页面导航正常
- `auroraview.api.*` 调用无 rejection
- 事件系统 (`auroraview.on/emit`) 工作正常
- 视觉回归检查通过 (`proofshot diff`)

## 6. Skills 分发（AuroraView 自有技能）

AuroraView 的官方技能（如 `qt-to-auroraview-migration`）内嵌在 `auroraview-cli` 二进制里，单一源为
`crates/auroraview-cli/skills/<skill-name>/SKILL.md`。Agent 不要从仓库里直接复制 `.cursor/skills/`、
`.claude/skills/` 等位置的副本 —— 那些是各工具自举生成的本地镜像，不是真相之源。

```bash
# 列出当前二进制内置的技能
auroraview-cli skills list

# 安装到某个 agent 工具的约定目录（项目内）
auroraview-cli skills install --target claude
auroraview-cli skills install --target cursor
auroraview-cli skills install --target all          # 覆盖所有已知工具

# 安装到任意路径
auroraview-cli skills install --path ./some/dir

# 安装到用户全局（~/.claude/skills/ 等）
auroraview-cli skills install --target cursor --global

# 打印某工具的解析路径而不实际写入
auroraview-cli skills path --target cursor
```

新增技能只需往 `crates/auroraview-cli/skills/` 丢一个 `<name>/SKILL.md`，`include_dir!` 在构建时自动打包，
不需要改任何 Rust 代码。镜像目录（`.claude/skills/` 等）已加入 `.gitignore`，避免各工具的 bootstrap 重新污染仓库。

## 7. 提交与 PR 约定

- 提交前确保关键检查通过（lint/test/build）。
- PR 描述需包含：改动目标、影响范围、验证方式、风险点。
- 若改动涉及文档或流程，请同步更新相关说明文件。
- UI 变更的 PR 建议附带 ProofShot 证据（`vx just e2e-pr`）。
