# AuroraView — AI Agent 导航图

> 渐进式披露入口。先读此图，再按任务类型跳转到对应深度文档。

## 项目一句话

AuroraView 是一个面向 DCC（Maya/Houdini/Blender 等）的轻量 WebView 框架，Rust 核心 + PyO3 Python 绑定，Windows 优先使用 WebView2 嵌入 Qt 宿主。

---

## 按任务快速导航

| 你的任务 | 先去这里 | 说明 |
|---|---|---|
| **了解整体架构与约定** | `llms.txt` | AI 友好的核心用法索引（5 分钟速读） |
| **写代码 / 改逻辑 / Review** | `.codebuddy/rules/` | 8 个按主题划分的执行约定，是 CI 与本地开发的真实约束 |
| **查完整 API 与架构细节** | `llms-full.txt` | 完整用法索引，包含所有 API 签名与模块说明 |
| **给人看的详细文档** | `docs/` | VitePress 站点，含 DCC 集成指南、API 文档、RFC |
| **了解打包/发布/CI** | `.codebuddy/rules/08-architecture.mdc` | 项目结构、auroraview-pack 打包系统、CI 流程 |
| **前端 JS ↔ Python 通信** | `.codebuddy/rules/05-frontend-api.mdc` + `.codebuddy/rules/07-event-system.mdc` | `window.auroraview` 协议与事件分发 |
| **Python 层接口** | `.codebuddy/rules/06-python-api.mdc` | `AuroraView` 基类、`bind_call`、`emit` |
| **测试策略** | `.codebuddy/rules/03-testing.mdc` | rstest / pytest、CI 矩阵、性能基线 |

---

## 30 秒项目速览

- **命令入口**：所有工具命令通过 `vx` 执行，任务编排通过 `vx just <task>`。
- **兼容底线**：Python 3.7+，不引入第三方 Python 依赖（仅一个 `.pyd`）。
- **测试入口**：`vx just test`（统一本地与 CI）。
- **构建入口**：`vx just build`。
- **技术栈**：Rust（windows-rs / webview2-com / PyO3）+ Python ABI3 wheel + TypeScript SDK。
- **事件循环**：Qt 宿主负责事件循环，Rust 不接管消息泵。

---

## 目录结构地图

```
├── crates/                 Rust crates
│   ├── auroraview-core/    核心协议与 WebView 后端抽象
│   ├── auroraview-cli/     CLI 工具与 Skills 分发
│   └── ...
├── python/auroraview/      Python 包（AuroraView 基类、DCC 宿主层）
├── packages/               TS/JS 包（前端 SDK）
│   └── auroraview-sdk/
├── gallery/                Gallery 演示应用（E2E 验证基准）
├── examples/               示例代码
├── docs/                   面向开发者的详细文档（VitePress）
├── submodules/             Git submodules（pack / protect / signals / extensions）
└── .codebuddy/rules/       AI 代理执行约定（真相之源）
```

---

## 关键约定速查（不可违背）

1. **禁止裸命令**：永远 `vx just build`，不要直接 `cargo build` 或 `pytest`。
2. **禁止代码内 emoji**：保持专业风格。
3. **函数命名**：简短精炼，使用行业标准术语，避免 `optimized`、`fixed` 等词。
4. **测试位置**：Rust 集成测试放在各 crate 的 `tests/` 目录，使用 `rstest`；不要内联单元测试。
5. **Skills 真相源**：官方 Skills 在 `crates/auroraview-cli/skills/<name>/SKILL.md`；`.cursor/skills/`、`.claude/skills/` 只是本地镜像，禁止复制。
6. **JS 事件统一**：Rust 层事件分发统一使用 `window.auroraview.trigger()`，不混用原生 `CustomEvent`。

---

## 外部参考

- **仓库**: https://github.com/loonghao/auroraview
- **PyPI**: https://pypi.org/project/auroraview
- **CHANGELOG**: `./CHANGELOG.md`
- **CONTRIBUTING**: `./CONTRIBUTING.md`
