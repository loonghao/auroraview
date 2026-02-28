## Architecture Diagnosis Checklist

> 用于“结构化扫仓库”的检查清单。执行时按优先级选取，不要求一次性全覆盖。

### A. 仓库/技术栈识别
- [ ] 识别入口文件：`Cargo.toml` / `pyproject.toml` / `package.json` / `justfile` / CI。
- [ ] 识别子系统：Rust runtime、Python API、JS SDK、示例应用（Gallery）、MCP、插件。
- [ ] 识别分发模型：wheel/.pyd、pack/overlay、自包含 runtime。

### B. 运行模式与线程模型（必做）
- [ ] 列出 run mode（Standalone/Embedded/IPC-only/Packed）。
- [ ] 标出事件循环归属（host-owned vs self-owned）。
- [ ] 标出 UI 线程要求（STA/main thread）。
- [ ] 标出 IPC drain 驱动方式（proxy wake vs timer/host pump）。
- [ ] 标出关闭与资源释放策略（显式 close / Drop / GC）。
- [ ] 标出 **启动时序**（event loop / timer / proxy / server 谁先谁后），以及是否存在明确的 **ready 信号/屏障**。
- [ ] 验证 drain 是否 **自驱动**：不会依赖“用户点击/窗口交互”才能前进。


### C. 典型架构异味（高命中）
- [ ] 消息处理重复：同一消息类型在多个文件/分支处理。
- [ ] 跨层耦合：JS→Rust→Python→Rust 或 Python 承担 runtime 决策。
- [ ] 默认值漂移：Rust/Python/JS 同一配置默认不一致。
- [ ] 生命周期补丁：`new_without_*` / `set_*` 之类后注入；Drop 中做 destroy。
- [ ] 超时语义不统一：JS promise timeout 与 Rust/Python callback timeout 不一致。
- [ ] 可观测性混乱：`tracing`/`logging`/`print` 混用；stdout 协议污染。
- [ ] 跨平台承诺不一致：非目标平台 fail-late（运行期才炸）。

### D. 搜索关键词（建议并行 grep）
- **Lifecycle**: `Drop`, `close(`, `destroy`, `DestroyWindow`, `WM_CLOSE`, `dispose`
- **Threading**: `Send`, `Sync`, `spawn`, `block_on`, `tokio`, `GIL`, `STA`, `main thread`
- **Event loop**: `event_loop`, `message_pump`, `process_events`, `process_ipc_only`, `ControlFlow`, `MainEventsCleared`, `NewEvents`, `StartCause`, `UserEvent`
- **IPC**: `MessageQueue`, `dispatcher`, `proxy`, `set_event_loop_proxy`, `send_event`, `oneshot`, `channel`, `post_message`, `ProcessMessages`, `Tick`

- **Timeout**: `timeout`, `DEFAULT_.*TIMEOUT`, `deadline`
- **Smells**: `TODO`, `FIXME`, `HACK`, `TEMPORARY`, `workaround`

### E. 风险分级建议
- **P0**：可导致卡死/崩溃/数据错乱；或高频且难定位的回归风险。
- **P1**：高维护成本、结构性重复、跨层漂移导致持续问题。
- **P2**：质量/体验问题（日志噪音、文档不一致、可测试性差）。

### F. 方案候选与取舍（必做）
- [ ] 给出 2–4 个候选方案（建议按 A/B/C/D），并说明“适用边界”。
- [ ] 输出 trade-off 表：收益、代价/风险、复杂度、周期、验收标准。
- [ ] 给出推荐路径：短期止血（如 ready/drain）、中期结构收敛、长期强解耦（如 sidecar）。
