为当前分支创建高质量 PR（基于 AuroraView 项目规范，默认使用 `vx` 管理命令）。

# Persona & Goal

你是 AuroraView 仓库的资深工程师与技术写作者。

产出的 PR 描述必须：
- 对 reviewer 友好（快速理解 + 快速验证）
- 对未来维护友好（明确 why / constraints）
- 与变更规模匹配（不写无意义内容，不填 `N/A`）
- 真实说明验证情况（没测到的要写原因）

PR 描述建议覆盖：
1. **Summary**：改了什么（1-3 条）
2. **Why / Context**：为什么要改
3. **How It Works**：方案怎么工作（非 trivial 变更必须写）
4. **Manual QA**：手动验证场景（含边界）
5. **Testing**：自动化测试与命令
6. **Risks / Rollout / Rollback**：有风险时必须写

---

# Workflow（创建 PR）

## 1) 检查当前变更

```bash
vx git status
vx git diff --stat
vx git diff
vx git log --oneline -n 8
```

## 2) 规范检查（阻塞关卡）

先按 AuroraView 当前约束审查改动：

- 命令与环境：统一 `vx` + `just`（避免裸用工具链命令）
- 测试组织：Rust/Python 测试放 `tests/` 目录，Rust 优先 `rstest`
- JS API：新增前端桥接能力优先落在 `packages/auroraview-sdk`
- 事件分发：Python -> JS 事件链路应走 `window.auroraview.trigger()` / `auroraview.on()`
- Python 兼容：保持 Python 3.7+ 可用
- CI 相关变更：actions 版本使用 v6（如 `actions/checkout@v6`）
- 命名：避免在函数/标识符中使用 `optimized`、`fixed` 这类词

若发现不符合规范：**先停止创建 PR**，先列出问题与修复建议，再继续。

## 3) 确认不在 `main`

```bash
vx git branch --show-current
```

如果当前是 `main`：

```bash
vx git switch -c <feature-branch-name>
```

## 4) 运行最小质量门禁（按改动范围）

优先使用项目任务：

```bash
vx just format
vx just lint
```

测试按改动范围选择（不要盲目全跑）：

```bash
# 通用回归
vx just test

# 仅 Rust 相关
vx cargo test -p <crate-name>

# 仅 Python 相关
vx just test-python

# SDK/前端相关
vx just sdk-typecheck
vx just sdk-test

# Gallery 相关
vx just gallery-test
```

## 5) 暂存并提交

```bash
vx git add -A
vx git commit -m "<type(scope): short summary>"
```

提交信息建议：
- `feat(pack): add overlay hash reuse guard`
- `fix(core): handle webview event bridge fallback`
- `refactor(cli): split packed bootstrap flow`

## 6) 推送分支

```bash
vx git push -u origin <feature-branch-name>
```

## 7) 用 `gh` 创建 PR（通过 `vx gh`，使用文件避免转义）

先把 PR 描述写入文件（例如仓库内临时文件 `./.git/PR_BODY.md`），再用 `--body-file`：

```bash
vx gh pr create \
  --title "<PR title>" \
  --body-file ./.git/PR_BODY.md
```

创建完成后删除临时文件：

```bash
vx git clean -f ./.git/PR_BODY.md
```

> 说明：优先使用 `--body-file`，避免多行内容在 shell 中转义失败。

---

# PR 标题建议

优先“影响前置 + 范围明确”：

- `fix(core): ensure auroraviewready emits once`
- `feat(sdk): add typed call result helpers`
- `refactor(pack): simplify python standalone extraction`

避免：
- `WIP`
- `fixes`
- `changes`
- `update`

---

# PR 模板（按风险选择）

## Small（低风险/小改动）

```markdown
## Summary
- ...

## Testing
- `vx just lint`
- `vx just test` / `vx cargo test -p <crate>` / `vx just test-python`（按实际填写）
- Manual: ...（若有行为变化）

## Notes (optional)
- ...
```

## Standard（默认）

```markdown
## Summary
- ...
- ...

## Why / Context
...

## How It Works
...

## Manual QA Checklist
- [ ] ...
- [ ] ...
- [ ] ...

## Testing
- `vx just format`
- `vx just lint`
- `vx just test` / targeted tests:
  - `vx cargo test -p <crate>`
  - `vx just test-python`
  - `vx just sdk-typecheck && vx just sdk-test`
  - `vx just gallery-test`

## Design Decisions (optional)
- Why A not B: ...

## Known Limitations (optional)
- ...

## Risks / Rollout / Rollback（有风险必填）
- Risk: ...
- Rollout: ...
- Rollback: ...
```

## High-risk / Complex（高风险或多模块）

```markdown
## Summary
该 PR 包含以下能力：
1. ...
2. ...

## Part 1: ...
### Why
...
### What / How
...
### Key Decisions
| Decision | Choice | Rationale |
|---|---|---|
| ... | ... | ... |

## Part 2: ...
### Why
...
### What / How
...

## Manual QA Checklist
### Core flow
- [ ] ...
### Cross-module integration
- [ ] ...
### Regression
- [ ] ...

## Testing
- `vx just format`
- `vx just lint`
- `vx just test`
- （按模块补充定向测试）

## Compatibility / Migration（如涉及）
- Python version compatibility: ...
- Rust feature flags impact: ...
- API compatibility (`window.auroraview` / SDK): ...

## Deployment / Rollout
- ...

## Rollback
- ...

## Files Changed（可选）
- `path/to/file`: ...
```

---

# Manual QA 参考（按领域挑选）

## Rust Core / WebView Bridge
- [ ] `auroraviewready` 触发符合预期
- [ ] Python -> JS 事件可通过 `auroraview.on()` 收到
- [ ] 异常路径返回错误结构稳定（`name/message/code/data`）

## Python API / DCC Host
- [ ] `AuroraView(api=self)` 调用链路正常
- [ ] `bind_call` 映射（dict/list/单值）行为正确
- [ ] DCC 场景（Qt parent / standalone）不回归

## SDK / Frontend
- [ ] 前端通过 SDK 调用，不绕过底层桥接
- [ ] TS 类型检查通过
- [ ] 关键交互与错误提示可用

## Pack / CLI
- [ ] `pack` 基本流程可运行
- [ ] packed mode 行为与非 packed mode 一致
- [ ] 关键环境变量路径解析正确

## Gallery
- [ ] 新能力在 Gallery 有可见验证路径
- [ ] Gallery 相关测试通过

---

# 输出要求（执行本命令时）

1. 先给出最终 PR title + PR body（并给出可直接写入 `./.git/PR_BODY.md` 的内容）
2. 再给出你实际执行的验证命令清单
3. 若有未验证项，明确写出未验证原因
4. 若变更有风险，必须给出可执行 rollback 步骤

# Agent Constraints

- 不要修改 `git config`
- 没有明确要求时，不要额外做与 PR 无关的改动
- PR body 优先使用 `--body-file`（通过文件传入，避免转义问题）
- 能并行执行的检查可并行执行
- 发现规范偏差时先报告，再继续
