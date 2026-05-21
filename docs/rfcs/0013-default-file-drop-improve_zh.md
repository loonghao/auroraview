# RFC 0013: 文件拖放代理统一与 `capture_file_drop` 开关（拆分总览）

- 编号: 0013
- 标题: 文件拖放代理统一与 `capture_file_drop` 开关 — 总览 / 拆分索引
- 状态: Superseded by RFCs 0014–0017
- 创建日期: 2026-05-20
- 最近修订: 2026-05-21
- 作者: AuroraView Core Team

---

## 0. 文档定位

本文件原为单一大 RFC（v1–v14，5 附录、26 处修订标号），随评审深度增加暴露出三类结构性问题：

1. **核心抽象与代码现状的隐含前提不对齐**（如 `IpcHandler::handle_message` 实为 `Result<_, String>`、`IpcHandler` 不持有 webview eval 通路），导致 D14 / D16 在主路径上无法落地；
2. **修订之间互相否定**（D5 不加 `BrowserConfig.capture_file_drop` 字段 vs D17 又要在 `Browser::new` 入口读该字段并清零）；
3. **诊断机制 ROI 严重失衡**（D7→D16 整套 sink trait 扩展只为打一行 `console.info`）。

按"每件事独立可回滚、reviewer 心智负担降一个量级"原则，将原方案拆为 **4 个独立 RFC**，每个 RFC 自带最小落地范围、独立可回滚 PR、独立测试边界。本文件不再承载设计内容，仅作索引与决策日志。

---

## 1. 拆分后的 RFC 索引

| RFC | 标题 | 范围 | 与本 RFC 的关系 |
|---|---|---|---|
| **0014** | `wry` / `tao` 集中到 `[workspace.dependencies]` | 仅 `Cargo.toml`，5 个 crate 改为 `{ workspace = true }` | 提取自原 §4.1.5 D9 修订段 |
| **0015** | `attach_drag_drop_handler` 共享 helper + 统一 7 处 builder | `auroraview-core::builder` + 7 处调用点 + `WebViewConfig.capture_file_drop` 字段 | 提取自原 §4.1 / §4.2.1 / §4.3.1 |
| **0016** | Browser 模式禁用 `capture_file_drop` | `auroraview-browser` + `tab_manager` 入口的运行期 warn 路径 | 提取自原 §4.3.4（D5/D17） |
| **0017** | Python `capture_file_drop` 三态契约（`Optional[bool]`） | `python/auroraview/core/` + PyO3 binding + CI grep 防回归 | 提取自原 §4.2.5（D3） |

**实施顺序建议**：0014 → 0015 → 0016 → 0017。0014 是其它 3 个的前置（消除 wry 版本飘移）；0015 提供 helper；0016 在 0015 helper 基础上加 Browser 模式的"不挂"分支；0017 在 0015 字段基础上做 Python 透传。每两个之间均可独立合并、独立回滚。

---

## 2. 从原方案中显式删除的内容

下列内容在 v14 仍存在，但拆分后**不再保留**，理由附后。如果未来出现真实需求，应作为独立 RFC 重新提出。

### 2.1 D7 / D16 诊断机制（整体删除）

**原设计**：在 `capture_file_drop=true` 路径下、首次 `Enter` / `Drop` 事件命中时，通过 `DragDropHandler::diagnostic_once: Arc<Once>` + `DragDropIpcSink::notify_diagnostic_once(&self, _script: &'static str)` 协议，由 sink 把诊断 JS 送到 webview 端 `evaluate_script`。

**删除理由**：

1. **主路径不可实现**：`src/ipc/handler.rs::IpcHandler` 只持有 `message_queue: Option<Arc<MessageQueue>>`，**不持有** `WebViewHandle`，**没有** `eval(script)` 接口。`crates/auroraview-desktop/src/ipc/handler.rs::IpcRouter` 同样只有 `handlers: DashMap`，无任何 webview 反向通路。要让 `notify_diagnostic_once` 真的能在 DevTools 看到，需要新增 `WebViewMessage::EvaluateScript` 变体、改造 `IpcHandler` 接受 eval channel、串通 `event_loop` —— 工程量与"统一文件拖放"主题完全无关。
2. **测试通过 ≠ 功能正确**：v14 §7.1 的 `dragdrop_diagnostic_once_fires_only_on_first_event` 用 `DiagnosticCountingSink` 计数即可通过，但生产代码上诊断永远到不了 webview。这是典型的 mock-driven false confidence。
3. **ROI 失衡**：整套机制（Once 守卫 + trait 扩展 + 三个 IPC 入口都改 + 测试 helper）的工程量是 RFC 主线的若干倍，目标只是在 DevTools 控制台多打一行 `console.info`。

**替代方案**：在前端用户文档 `docs/zh/guide/file-drop.md`（与 RFC 0015 同 PR 提交）中加一段说明：

> 如果你设置了 `capture_file_drop=True` 但 `auroraview.on('file_drop', ...)` 没收到事件，请检查：
>
> 1. 浏览器 HTML5 `dragover` / `drop` 是否仍在监听（受 wry/WebView2 上游 Bug 影响，开启 `capture_file_drop` 后两者互斥，详见 RFC 0015 §2）；
> 2. IPC 通道是否已建立（参考 `auroraview.on('auroraviewready', ...)` 检查桥就绪）。

文档 + 用户主动 grep 的成本 ≈ 0；与 RFC 主线代码完全解耦。

### 2.2 `DragDropIpcSink::notify_diagnostic_once` 方法（整体删除）

随 §2.1 一并移除。`DragDropIpcSink` trait 表面回归单方法：

```rust
pub trait DragDropIpcSink: Send + Sync + 'static {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), DispatchError>;
}
```

三个 impl（`IpcHandler` / `IpcRouter` / packed `IpcSink`）每处 ≤ 5 行，详见 RFC 0015 §3.1。

### 2.3 D14 `DispatchError` 三变体语义化（暂时退到单变体）

**原设计**：`DispatchError` 含 `Disconnected` / `Serialization(serde_json::Error)` / `Backend(Box<dyn Error>)` 三变体，按底层错误类型精确归类，让契约测试可对变体做模式匹配。

**退化决策**：

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DispatchError {
    /// 任何 sink 实现侧的底层错误。当前 `IpcHandler::handle_message` 错误类型
    /// 是 `String`，无法精确归类到 `Disconnected` / `Serialization`，统一走此变体。
    /// 等到 IpcHandler 的错误类型 enum 化（独立 refactor，不在本 RFC 内）后，
    /// 再补 `Disconnected` / `Serialization` 等语义化变体。`#[non_exhaustive]`
    /// 保证未来加变体不算 breaking。
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync + 'static>),
}
```

**理由**：v14 §4.1.3 的归类示例引用了 `IpcError::ChannelClosed | SendFailed | Serialization` 等不存在的 enum 变体。代码现状是 `pub fn handle_message(&self, message: IpcMessage) -> Result<serde_json::Value, String>`，错误类型是 `String`，无任何结构。强行落地三变体 enum 必须先把 `IpcHandler` 的错误类型 enum 化，影响面跨整个 IPC 子系统、且与本 RFC 主题无关。

**`#[non_exhaustive]` 保留**：未来 IpcError enum 化后追加变体不算 breaking。详见 RFC 0015 §3.2。

### 2.4 D5 / D17 协调（二选一，本 RFC 选 D5 路线 + 0016 内置事实化）

原 v14 同时声称：

- **D5**（§4.3.4.3 表格）：`BrowserConfig.capture_file_drop` ❌ **不新增**
- **D17**（§4.3.4.4）：`Browser::new(cfg)` 入口检查 `cfg.tab_webview_config.capture_file_drop == true` 并清零

后者要读字段、前者不让加字段，互相否定。RFC 0016 取 **D5 路线**：`BrowserConfig` / `TabManagerConfig` 不加 `capture_file_drop` 字段，`Browser::new` / `TabManager::new` 也**不读 cfg**——两个入口直接把 `attach_drag_drop_handler` 调用替换为不挂载（`capture=false` 常量），没有"运行期检查 + 清零"逻辑。

如果用户真的需要"在 Browser 模式下也用 IPC 拖放"，必须把对应页面提升到顶层 `AuroraView` 实例（与 v14 的"推荐路径"一致，只是删去了 cfg → warn → 清零 这条多余链路）。

详见 RFC 0016 §3。

### 2.5 D18 packed runtime mode 分流（与 §2.4 一并简化）

由于 §2.4 已把"运行期 warn"删除（Browser 模式根本不读 cfg、没有需要 warn 的字段），原 v14 §4.2.4.3 D18 修订引入的 `PackedRuntimeMode::TopLevelAuroraView` / `Browser` 枚举与 `resolve_packed_capture_file_drop_with_mode` 函数也**一并删除**。

`AURORAVIEW_CAPTURE_FILE_DROP` env var 在 packed runtime 解析层不再 mode 分流；packed runtime 在 Browser 入口本就不读 `capture_file_drop`，env var 设了等于"对一个不被读取的字段做了运行时覆盖"，不会产生矛盾日志。

详见 RFC 0016 §4。

---

## 3. 从原方案中保留的核心决策（无变更）

下列设计在拆分后各 RFC 中**完整保留**，本节列出供 reviewer 一次性核对：

1. **默认值 `false` / 零特例**：所有运行模式（standalone / DCC / CLI / packed）默认 `capture_file_drop = false`，DCC 不再有"默认 `true`"特例。详见 RFC 0015 §1。
2. **`capture` 通过"是否调用 `with_drag_drop_handler`"来切换**，而非"挂上后返回 `false`"——后者受 wry/WebView2 上游 Bug 影响在 Windows 不可行。详见 RFC 0015 §2。
3. **helper 接受 `&Arc<S>` 借用形态**（D15 修订），`!capture` 时 `Arc::strong_count` 真实保持原值、契约可独立测试。详见 RFC 0015 §3.3。
4. **helper 签名泛型生命周期 `'a`**（D1 修订），同时兼容 `WebViewBuilder::new()` 与 `WebViewBuilder::new_with_web_context(&mut web_context)`。详见 RFC 0015 §3.3。
5. **CLI flag**：`auroraview run` 单向 flag、`auroraview pack` 一对显式 flag (`--capture-file-drop` / `--no-capture-file-drop`) + `resolve_capture_file_drop` 还原 `Option<bool>`（D2 修订）。详见 RFC 0015 §4。
6. **Packed app env var 逃生口** `AURORAVIEW_CAPTURE_FILE_DROP`，`parse_truthy` 大小写不敏感识别 `1/true/on/yes/enabled` × `0/false/off/no/disabled`，无效值打 warn 不静默回落（D4 修订）。详见 RFC 0015 §4.3。
7. **Python 三态契约**（`Optional[bool]`）+ §7.5 CI grep 防回归。详见 RFC 0017。
8. **`#[serde(default)]` 已保证 manifest / overlay 二进制兼容**，无需 bump overlay version。详见 RFC 0015 §4.4。
9. **Controller webview 永不挂载**（§4.3.2）—— 在 0016 中作为"全 Browser 永不挂"的特例自然涵盖，不再单列代码注释段。
10. **Child window 永不挂载**（§4.3.3）+ `NewWindowConfig.new_window_mode` docstring 警示（D11 修订）。详见 RFC 0015 §3.6。
11. **IPC 事件 schema 不变**：`file_drop_hover` / `file_drop` / `file_drop_cancelled` 现有 payload 字段保持。详见 RFC 0015 §5。
12. **作用域限定**（D13 修订）：`file_drop*` 事件只来自挂载 handler 的那个 webview 自身，不跨 webview 传播。详见 RFC 0015 §5 末尾段。

---

## 4. 兼容性总结

- **DCC 默认行为变化**（v14 §6.1）保留：DCC 用户从 `file_drop` 自动触发 → 需要显式 `capture_file_drop=True`。仍按 0.x minor breaking 走，CHANGELOG 显著标注。
- **manifest / overlay 二进制兼容性**保留：`#[serde(default)]` 让新旧 runtime / 新旧 overlay 4 种交叉组合都安全兜底为 `false`，无需 bump overlay version。
- IPC schema、事件名、`DragDropHandler` 内部行为均不变，已订阅 `file_drop` 的前端代码无需修改。
- CLI 既有 flag 全部保留；新增 flag 默认关闭。

---

## 5. 历史修订日志（保留为索引）

v14 版本的全部修订（D1–D18）的来源问题、落点章节、状态见下表。拆分后各 RFC 在自己的"修订关联"段引用这张表的对应行，本文件作为 single source of truth 不再二次维护。

| 编号 | 来源问题 | v14 落点 | 拆分后归属 |
|---|---|---|---|
| D1 | helper 生命周期参数 | §4.1.2 | RFC 0015 §3.3 |
| D2 | Pack CLI flag 形态 | §4.2.4.2 | RFC 0015 §4.2 |
| D3 | `to_kwargs` 注释锚点 | §4.2.5.3 | RFC 0017 §3.3 |
| D4 | env var 取值识别 | §4.2.4.3 | RFC 0015 §4.3 |
| D5 | Browser 模式禁用 | §4.3.4 | RFC 0016 §3 |
| D6 | trait `Result` 返回 | §4.1.2 / §4.1.3 | RFC 0015 §3.1（`DispatchError` 退化为单变体，见本文 §2.3）|
| D7 | 诊断 JS 注入 | §4.1.6 | **删除**（本文 §2.1）|
| D8 | 取消 `?Sized` | §4.1.2 | RFC 0015 §3.3 |
| D9 | workspace dep 集中 | §4.1.5 | RFC 0014 |
| D11 | child window 死代码 | §4.3.3 | RFC 0015 §3.6 |
| D12 | PyO3 + packed 同进程假想 | §11 / §4.5.1 | RFC 0017 §4 |
| D13 | IPC 作用域限定 | §4.4 | RFC 0015 §5 |
| D14 | `DispatchError` 语义化 | §4.1.2 | **退化为单变体**（本文 §2.3，等 IpcHandler enum 化后再补） |
| D15 | helper `&Arc<S>` 借用 | §4.1.2 | RFC 0015 §3.3 |
| D16 | Once 守卫诊断 | §4.1.6 | **删除**（本文 §2.1）|
| D17 | Browser mutate 入参 | §4.3.4.4 | **删除**（与 D5 二选一，本文 §2.4）|
| D18 | packed env var mode 分流 | §4.2.4.3 | **删除**（与 §2.4 一并简化，本文 §2.5）|

---

## 6. 未决问题与后续 RFC

本拆分版本范围内**无未决问题**。后续可独立 RFC 跟进：

- **`IpcHandler` 错误类型 enum 化**（独立 refactor，跨整个 IPC 子系统，与本主题解耦）。完成后再回头补 `DispatchError::{Disconnected, Serialization, ...}` 语义化变体（`#[non_exhaustive]` 保证向后兼容）。
- **Child window IPC 通路打通**（届时 RFC 0015 §3.6 的"始终不挂"约束可松绑）。
- **上游 wry 修复 hybrid 模式**后，给 `DragDropHandler` 加事件穿透开关。
- **Per-tab `capture_file_drop` 覆盖**（如出现真实需求；目前 RFC 0016 已显式禁用）。
- **`auroraview run` 升级为双向覆盖 flag**（如出现真实需求）。

---

## 7. 参考

- 拆分后 RFC: 0014 / 0015 / 0016 / 0017
- `crates/auroraview-core/src/builder/drag_drop.rs` — `DragDropHandler::into_handler` 实现
- `src/ipc/handler.rs::IpcHandler` — 错误类型为 `String`（影响 §2.3 决策）
- wry `WebViewBuilder::with_drag_drop_handler` 文档：https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_drag_drop_handler
- 上游 Bug 跟踪：[tauri#15138](https://github.com/tauri-apps/tauri/issues/15138)、[wry#157](https://github.com/tauri-apps/wry/issues/157)
