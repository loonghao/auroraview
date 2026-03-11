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

## 5. 提交与 PR 约定

- 提交前确保关键检查通过（lint/test/build）。
- PR 描述需包含：改动目标、影响范围、验证方式、风险点。
- 若改动涉及文档或流程，请同步更新相关说明文件。
