# RFC 0015: `attach_drag_drop_handler` 共享 helper + 7 处 builder 统一

- 编号: 0015
- 标题: 文件拖放代理统一封装 `attach_drag_drop_handler` + `WebViewConfig.capture_file_drop` 开关
- 状态: Draft
- 创建日期: 2026-05-21
- 作者: AuroraView Core Team
- 拆分自: RFC 0013 §4.1 / §4.2.1 / §4.3.1 / §4.4
- 前置依赖: **RFC 0014**（`wry` / `tao` 通过 workspace dep 集中）
- 关联文件:
  - `crates/auroraview-core/src/builder/helpers.rs`（新增 `attach_drag_drop_handler` + `DragDropIpcSink` trait + `DispatchError`）
  - `crates/auroraview-core/src/builder/mod.rs`（`pub use`）
  - `crates/auroraview-core/Cargo.toml`（新增 `thiserror` 依赖，如尚未引入）
  - `src/webview/config.rs`（`WebViewConfig.capture_file_drop` + `WebViewBuilder::capture_file_drop`）
  - `src/ipc/handler.rs`（`impl DragDropIpcSink for IpcHandler`）
  - `src/webview/backend/native.rs`（路径 1）
  - `src/webview/desktop/webview_builder.rs`（路径 2）
  - `crates/auroraview-cli/src/cli/run.rs`（路径 6 + `RunArgs`）
  - `crates/auroraview-cli/src/cli/pack.rs`（`PackArgs` + `resolve_capture_file_drop`）
  - `crates/auroraview-cli/src/packed/webview/mod.rs`（路径 7 × 2 + `resolve_packed_capture_file_drop`）
  - `crates/auroraview-cli/src/packed/ipc.rs`（`impl DragDropIpcSink`）
  - `crates/auroraview-pack/src/manifest.rs`（`SecurityManifestConfig.capture_file_drop`）
  - `crates/auroraview-pack/src/config.rs`（`PackConfig.capture_file_drop` + `from_manifest` 映射）
  - `crates/auroraview-desktop/src/config.rs`（`DesktopConfig.capture_file_drop`）
  - `crates/auroraview-desktop/src/window/builder.rs`（路径 8）
  - `crates/auroraview-desktop/src/ipc/router.rs`（`impl DragDropIpcSink for IpcRouter`）
  - `src/webview/child_window.rs`（模块 docstring）
  - `python/auroraview/core/config.py::NewWindowConfig`（docstring 警示）

> Browser 模式相关的"永不挂"分支由 RFC 0016 单独处理；Python 三态契约由 RFC 0017 单独处理。本 RFC 仅完成 Rust 侧 helper 与 7 处 builder 中的 5 处直接接入（路径 1/2/6/7/8），路径 3/4/5/9/10 不在本 RFC 范围。

---

## 1. 摘要

新增 `capture_file_drop: bool` 开关（默认 `false`），统一控制 `wry::WebViewBuilder::with_drag_drop_handler` 是否被注册：

- `false` → 不调用 `with_drag_drop_handler`，WebView 走浏览器原生 HTML5 拖放语义；
- `true` → 调用 `with_drag_drop_handler`，事件代理为 IPC `file_drop_hover` / `file_drop` / `file_drop_cancelled`。

所有运行模式（standalone / DCC / CLI / packed / desktop crate）默认值统一为 `false`，**零特例**——这是相对当前实现的 breaking change（DCC 默认从 `true` 变 `false`）。

---

## 2. wry 行为说明与上游 Bug（关键）

**官方约定**（[wry docs](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_drag_drop_handler)）：

> Return `true` in the callback to block the OS' default behavior.
> Note, that if you do block this behavior, it won't be possible to drop files on `<input type="file">` forms.

**实际情况（已验证的上游 Bug）**：在 Windows（WebView2）上，**只要调用 `with_drag_drop_handler`，无论闭包返回 `true` 还是 `false`，WebView 内的 HTML5 `dragenter` / `dragover` / `drop` 事件都会被屏蔽**。社区上游均已确认：

- [tauri-apps/tauri#15138](https://github.com/tauri-apps/tauri/issues/15138)
- [tauri-apps/wry#157](https://github.com/tauri-apps/wry/issues/157)

**结论**：`capture_file_drop` 必须通过"**是否调用 `with_drag_drop_handler` 本身**"来切换，而非 handler 内返回 `false`。

**两种模式互斥**（开发者必须二选一）：

- `capture_file_drop = false`（默认）→ 完全不调用，HTML5 拖放可工作（适合 Monaco / CodeMirror / 富文本上传等组件）；
- `capture_file_drop = true` → 调用，前端 HTML5 拖放完全失效，但 IPC 拿得到完整本地路径。

未来若上游 wry/WebView2 修复 hybrid/passthrough 行为，本 RFC 的 helper 接口（§3.3）签名无需变更。

---

## 3. Helper 设计

### 3.1 `DragDropIpcSink` trait + `DispatchError`

在 `crates/auroraview-core/src/builder/helpers.rs` 中：

```rust
use std::sync::Arc;

/// Errors that may occur while dispatching a drag-drop event into the IPC pipeline.
///
/// **当前为单变体形态**：`IpcHandler::handle_message` 错误类型现为 `String`，
/// 无法精确归类到 `Disconnected` / `Serialization` 等语义化变体。
/// 等到 IpcHandler 错误类型 enum 化（独立 refactor，跨整个 IPC 子系统，与本
/// RFC 主题解耦）后，再追加 `Disconnected` / `Serialization` 变体。
///
/// `#[non_exhaustive]` 保证未来追加变体不算 breaking change。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DispatchError {
    /// 任意 sink 实现侧的底层错误。
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl DispatchError {
    /// 便捷构造：把任意 `Send + Sync + 'static` 错误包装为 `Backend` 变体。
    pub fn backend<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Backend(Box::new(err))
    }
}

/// Trait 抽象 helper 期望的 IPC 入口。
///
/// 实现侧只负责"传递错误"，不负责"打日志"——日志由 helper 内部统一打。
/// 这避免三个 impl 各打各的日志格式不一致问题。
pub trait DragDropIpcSink: Send + Sync + 'static {
    /// Forward a single drag-drop event into the IPC pipeline.
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), DispatchError>;
}
```

### 3.2 三处 IPC 入口的 `impl DragDropIpcSink`

每处 ≤ 5 行。`IpcHandler::handle_message` 当前返回 `Result<_, String>`，包装时使用 `DispatchError::backend`：

```rust
// src/ipc/handler.rs 末尾追加：

// String 不实现 std::error::Error，需要包一层。这里直接用 String 做 Display
// + 把字符串里的内容透传给 DispatchError::Backend 即可。
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct IpcStringError(String);

impl auroraview_core::builder::DragDropIpcSink for IpcHandler {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), auroraview_core::builder::DispatchError> {
        self.handle_message(IpcMessage {
            event: event_name.to_string(),
            data,
            id: None,
        })
        .map(|_| ())
        .map_err(|s| auroraview_core::builder::DispatchError::backend(IpcStringError(s)))
    }
}
```

`auroraview-desktop::IpcRouter` 与 `auroraview-cli::packed::ipc` 的 IPC 入口类型同样补上一条 `impl DragDropIpcSink`，结构镜像。

> **未来 IpcHandler enum 化后**：把 `String` 改成 `IpcError` enum，`map_err` 中按变体精确归类到 `DispatchError::Disconnected` / `DispatchError::Serialization` 等新变体。届时无需改 trait 签名（`DispatchError` 是 `#[non_exhaustive]`）。

### 3.3 `attach_drag_drop_handler` 函数签名

```rust
/// Conditionally attach the drag-drop proxy handler.
///
/// - `capture == false` —— builder 原样返回，**不**调用 `with_drag_drop_handler`，
///   wry 走浏览器原生 HTML5 拖放语义。`ipc_sink` 仅以**借用**形式传入，
///   helper 内部**不**做 `Arc::clone`，调用栈上 `Arc::strong_count` 真实保持不变
///   （契约可独立测试，详见 §6.1）。
/// - `capture == true` —— helper **内部**对 `ipc_sink` 做唯一一次 `Arc::clone`、
///   构造一个 `Send + Sync + 'static` 的 wry 回调挂载到 builder。事件经
///   `DragDropHandler::into_handler` 过滤后（`Over` 被丢弃）以
///   `file_drop_hover` / `file_drop` / `file_drop_cancelled` 三个事件名
///   转发给 `sink.dispatch(...)`；若返回 `Err(DispatchError)`，
///   helper 在闭包内打一条 `tracing::error!` 后丢弃事件（拖放路径不应阻塞）。
///
/// # 签名借用形态
///
/// `ipc_sink: &Arc<S>` 而非 `Arc<S>`：
/// - `capture=false` 时调用方栈上 `Arc::strong_count` 真实保持原值；
/// - `capture=true` 时 helper 内部 `Arc::clone(ipc_sink)` 一次（一次原子加，<5ns）；
/// - 调用方写 `&ipc_handler` 而非 `ipc_handler.clone()`，更短、更明确
///   "我把这个 Arc 借给 helper 看一眼，要不要 clone 你自己决定"。
///
/// # 生命周期参数 `'a`
///
/// `wry::WebViewBuilder` 的实际生命周期取决于构造方式：
/// - `WebViewBuilder::new()` —— `'static`；
/// - `WebViewBuilder::new_with_web_context(&mut web_context)` —— 借用
///   `web_context` 的生命周期，**非** `'static`。
///
/// 仓库内主要业务路径使用 `new_with_web_context`，因此 helper 必须用泛型
/// 生命周期 `'a` 同时兼容两类 builder。
///
/// # 静态分发
///
/// `where S: DragDropIpcSink`（**无** `?Sized`）。所有调用点统一传
/// `&Arc<具体类型>`，编译器为每个具体类型单态化一份实例。
///
/// **Note on upstream behavior**: due to a wry/WebView2 limitation,
/// registering `with_drag_drop_handler` (regardless of its return value)
/// suppresses HTML5 `dragover`/`drop` events inside the WebView. See §2.
pub fn attach_drag_drop_handler<'a, S>(
    builder: wry::WebViewBuilder<'a>,
    capture: bool,
    ipc_sink: &Arc<S>,
) -> wry::WebViewBuilder<'a>
where
    S: DragDropIpcSink,
{
    if !capture {
        // ipc_sink 仅借用，helper 不持有；strong_count 完全不变。
        return builder;
    }

    let sink = Arc::clone(ipc_sink); // helper 内部唯一 clone 点

    builder.with_drag_drop_handler(create_drag_drop_handler(
        move |event_name, data| {
            if let Err(err) = sink.dispatch(event_name, data) {
                tracing::error!(
                    target: "auroraview::drag_drop",
                    "Failed to dispatch {} via DragDropIpcSink: {}",
                    event_name,
                    err
                );
            }
        },
    ))
}
```

### 3.4 模块导出

在 `crates/auroraview-core/src/builder/mod.rs`：

```rust
pub use helpers::{attach_drag_drop_handler, DispatchError, DragDropIpcSink};
```

`DragDropHandler` / `DragDropEventData` / `as_event_name` / `create_drag_drop_handler` 全部保留不变。

### 3.5 调用方使用模式

```rust
use auroraview_core::builder::attach_drag_drop_handler;

// 5 处 builder 调用点（路径 1/2/6/7×2/8）全部统一为：
builder = attach_drag_drop_handler(
    builder,
    config.capture_file_drop,   // bool
    &ipc_handler,                // &Arc<IpcHandler>，helper 内部决定是否 clone
);
```

`!capture` 时调用方栈上 `Arc::strong_count` 真实保持原值，契约可独立测试。

### 3.6 不接入路径的处理

| # | 路径 | 文件 | 处理方式 |
|---|---|---|---|
| 3 | tab webview（旧） | `src/webview/tab_manager.rs:469` | 由 RFC 0016 处理（永不挂） |
| 4 | tab controller（旧） | `src/webview/tab_manager.rs:984` | 由 RFC 0016 处理（永不挂） |
| 5 | child window | `src/webview/child_window.rs` | 永不挂；构造接口不接受 `capture_file_drop` 参数；模块顶部 docstring 标注作用域限制；`python/auroraview/core/config.py::NewWindowConfig.new_window_mode` docstring 追加 `Note (RFC 0015)` 段说明事件不跨 webview 传播 |
| 9 | tab webview（新） | `crates/auroraview-browser/src/tab/manager.rs:122` | 由 RFC 0016 处理 |
| 10 | tab controller（新） | `crates/auroraview-browser/src/browser.rs:545` | 由 RFC 0016 处理 |

#### 3.6.1 child window 的设计要点

- `create_child_webview_window` 实际签名是 `(url: &str, width: u32, height: u32)`，**不接受 config**，无需新增参数。
- `child_window.rs` 顶部加模块 docstring：

  ```rust
  //! # Drag-drop behavior
  //!
  //! Child windows do not currently support the `capture_file_drop` IPC
  //! proxy. Pages loaded in a child window can use the browser-native
  //! HTML5 drag-drop API (`dragenter` / `dragover` / `drop`) directly.
  //! If your tool needs absolute file paths via IPC, open a top-level
  //! `AuroraView` instead, where `capture_file_drop=True` is supported.
  ```

- `NewWindowConfig.new_window_mode` 的 docstring 追加：

  ```python
  Note (RFC 0015):
      When ``new_window_mode="child_webview"`` is combined with
      ``capture_file_drop=True`` on the parent ``AuroraView``,
      the parent webview will receive ``file_drop*`` IPC events
      normally. The child windows opened via ``window.open``,
      however, run on independent event loops without an IPC
      channel back to the parent and **never** register
      ``with_drag_drop_handler`` regardless of any setting on
      the parent.
  ```

---

## 4. 配置层与 CLI

### 4.1 配置层字段

#### `WebViewConfig`（`src/webview/config.rs`）

```rust
pub struct WebViewConfig {
    // ...
    pub capture_file_drop: bool,
}
```

`Default::default()` 中 `capture_file_drop: false`。`WebViewBuilder` 链式方法（**无 `with_` 前缀**，与既有 `title()` / `url()` / `allow_file_protocol()` 等保持风格一致）：

```rust
impl WebViewBuilder {
    pub fn capture_file_drop(mut self, capture: bool) -> Self {
        self.config.capture_file_drop = capture;
        self
    }
}
```

#### `DesktopConfig`（`crates/auroraview-desktop/src/config.rs`）

镜像 `WebViewConfig`：顶层加 `capture_file_drop: bool`，`impl DesktopConfig` 加同名链式方法。两者无自动同步，分别由各自 builder 直接读取。

#### `auroraview-pack`（manifest + `PackConfig` 两层）

**manifest 层**（`crates/auroraview-pack/src/manifest.rs::SecurityManifestConfig`）：

```rust
pub struct SecurityManifestConfig {
    #[serde(default)]
    pub content_security_policy: Option<String>,

    /// Whether the packed app should capture file drop events as IPC events.
    /// `None` (omitted) → use code default (`false`).
    #[serde(default)]
    pub capture_file_drop: Option<bool>,
}
```

**`PackConfig` 层**（`crates/auroraview-pack/src/config.rs`）：

```rust
pub struct PackConfig {
    // ...
    #[serde(default)]
    pub capture_file_drop: bool,
}
```

`PackConfig::from_manifest`：

```rust
pack_config.capture_file_drop = manifest
    .security
    .as_ref()
    .and_then(|s| s.capture_file_drop)
    .unwrap_or(false);
```

> **现状核对**：`PackConfig` 当前代码中 `content_security_policy` 字段实际位置（顶层 vs 子结构）需要 Step 1 实施时二次确认；本 RFC 不假设具体层级，只要求 `capture_file_drop` 与 `content_security_policy` 在同一层、同样使用 `#[serde(default)]` 即可。

**Overlay 二进制兼容性**：`#[serde(default)]` 保证 4 种新旧交叉组合都安全兜底为 `false`，**无需 bump overlay version**。

### 4.2 CLI flag

#### `RunArgs`（`crates/auroraview-cli/src/cli/run.rs`）

> **修订（post-implementation）**：原方案为单向 `--capture-file-drop`
> bool flag，与 `PackArgs` 形态不一致。后续 review 指出未来给
> `auroraview run` 接 manifest / env var 时单向 flag 会强制把"用户没传"
> 解读成"显式 false"覆盖下层值。已对齐 `PackArgs`：双向 flag +
> `resolve_capture_file_drop` 还原 `Option<bool>`，`run` 入口在没有下层
> 来源时调用 `unwrap_or(false)` 落地，与原代码默认等价。

```rust
#[arg(
    long = "capture-file-drop",
    action = clap::ArgAction::SetTrue,
    overrides_with = "no_capture_file_drop"
)]
pub capture_file_drop: bool,

#[arg(
    long = "no-capture-file-drop",
    action = clap::ArgAction::SetTrue,
    overrides_with = "capture_file_drop"
)]
pub no_capture_file_drop: bool,
```

`pub fn resolve_capture_file_drop(args: &RunArgs) -> Option<bool>` 与
`pack.rs` 的同名函数语义完全一致；当前 `run` 入口直接调用
`.unwrap_or(false)` 落地，未来若接入 manifest / env var 即可平滑切到
`.or(manifest...).or(env...).unwrap_or(false)` 的合并链而不破坏
现有 CLI 表面。

#### `PackArgs`（`crates/auroraview-cli/src/cli/pack.rs`）

一对显式 flag + `resolve_capture_file_drop` 还原 `Option<bool>`：

```rust
#[arg(
    long = "capture-file-drop",
    action = clap::ArgAction::SetTrue,
    overrides_with = "no_capture_file_drop",
    help = "Force-enable [security].capture_file_drop in the packed overlay."
)]
pub capture_file_drop: bool,

#[arg(
    long = "no-capture-file-drop",
    action = clap::ArgAction::SetFalse,
    overrides_with = "capture_file_drop",
    help = "Force-disable [security].capture_file_drop in the packed overlay, \
            even if the manifest has it set to true."
)]
pub no_capture_file_drop: bool,
```

> **勘误（2026-05）**：上文 `no-capture-file-drop` 代码块中的 `action = clap::ArgAction::SetFalse` **是错误的**，请实现时改为 `clap::ArgAction::SetTrue`。原因：clap 4 的 `ArgAction::SetFalse` 在 flag **缺省时默认值为 `true`**（与 `SetTrue` 对称），这会让本节展示的 `resolve_capture_file_drop` 匹配臂全部错位——默认调用返回 `Some(false)` 而非 `None`，`--capture-file-drop` 单独传入则命中 `unreachable!()` 触发进程 panic。正确做法是两个 flag 都用 `SetTrue`，由 `overrides_with` 保证 `(true, true)` 不可达。已通过 clap 4.6 的 `try_parse_from` 实测覆盖全部 5 种输入组合验证。参考：<https://docs.rs/clap/latest/clap/builder/enum.ArgAction.html>。

辅助函数（建议放在 `pack.rs` 顶层、未来 RunArgs 升级时可复用）：

```rust
pub fn resolve_capture_file_drop(args: &PackArgs) -> Option<bool> {
    match (args.capture_file_drop, args.no_capture_file_drop) {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (true, true) => unreachable!("clap overrides_with should make this impossible"),
    }
}
```

**Pack 阶段合并规则**：

```
overlay 写入值 = resolve_capture_file_drop(&pack_args)
    .or(manifest.security.and_then(|s| s.capture_file_drop))
    .unwrap_or(false);
```

> **不用 clap `Option<bool>` + `num_args = 0..=1` 形态**：会有位置参数歧义（`auroraview pack --capture-file-drop my-app.toml` 会把 `my-app.toml` 当成 flag 值），且与 RunArgs 形态不一致。`cargo` / `rustup` / `wrangler` 等成熟 CLI 都用显式 flag 对。

### 4.2.1 Erratum (post-implementation)

§4.2 above shows `--no-capture-file-drop` declared with `clap::ArgAction::SetFalse`. **This is incorrect**: `SetFalse` semantically means "set to false **when present**" but defaults to `true` when absent (the inverse of `SetTrue`). Under that contract `resolve_capture_file_drop` would observe `(false, true)` for a default invocation and force-disable manifest values, plus `auroraview pack --capture-file-drop` would hit the `unreachable!()` branch and panic.

The implemented form uses `clap::ArgAction::SetTrue` for both flags:

```rust
#[arg(
    long = "no-capture-file-drop",
    action = clap::ArgAction::SetTrue,
    overrides_with = "capture_file_drop",
    help = "Force-disable [security].capture_file_drop in the packed overlay, \
            even if the manifest has it set to true."
)]
pub no_capture_file_drop: bool,
```

Both flags default to `false` when absent; `overrides_with` ensures only one wins when both are passed. The `resolve_capture_file_drop` truth table in §4.2 is correct as written.

### 4.3 Packed env var 逃生口

终端用户拿到 packed exe 后无法重新打包，需要运行期开关：

| 环境变量取值（大小写不敏感、`trim`）| 含义 |
|---|---|
| `1` / `true` / `on` / `yes` / `enabled` | 强制开启 |
| `0` / `false` / `off` / `no` / `disabled` | 强制关闭 |
| 未设置 | 沿用 overlay |
| 设置但取值无效（如 `=hello`）| 沿用 overlay + 一条 `tracing::warn!` |

实现位置：`crates/auroraview-cli/src/packed/webview/mod.rs`：

```rust
fn parse_truthy(s: &str) -> Option<bool> {
    let s = s.trim();
    if ["1", "true", "on", "yes", "enabled"]
        .iter()
        .any(|v| s.eq_ignore_ascii_case(v))
    {
        Some(true)
    } else if ["0", "false", "off", "no", "disabled"]
        .iter()
        .any(|v| s.eq_ignore_ascii_case(v))
    {
        Some(false)
    } else {
        None
    }
}

pub fn resolve_packed_capture_file_drop(overlay_value: bool) -> bool {
    let raw = match std::env::var("AURORAVIEW_CAPTURE_FILE_DROP") {
        Ok(v) => v,
        Err(_) => return overlay_value,
    };

    match parse_truthy(&raw) {
        Some(value) => {
            tracing::info!(
                target: "auroraview::capture_file_drop",
                "capture_file_drop overridden by AURORAVIEW_CAPTURE_FILE_DROP={raw:?} → {value}"
            );
            value
        }
        None => {
            tracing::warn!(
                target: "auroraview::capture_file_drop",
                "AURORAVIEW_CAPTURE_FILE_DROP={raw:?} is not a recognized boolean \
                 literal (expected one of: 1/true/on/yes/enabled / 0/false/off/no/disabled, \
                 case-insensitive). Falling back to overlay value: {overlay_value}"
            );
            overlay_value
        }
    }
}
```

> **取值字面量集合的来源说明**：识别 `1/0`（Windows 注册表）、`true/false`（Rust / 通用）、`on/off`（systemd）、`yes/no`（Docker / cron）、`enabled/disabled`（部分 DCC 配置惯例）。这是 AuroraView 自有的并集，便于在不同来源的运维场景下都能直觉地输入；未刻意对齐任何单一规范。

### 4.4 配置优先级（拆分后无变更）

入口互斥前提：5 个来源不会在同一进程中全部出现，每个具体入口下最多 2~3 个同时可见。

| 入口 | Python kwarg | Run flag | Pack flag | manifest | env var | code default |
|---|---|---|---|---|---|---|
| PyO3 嵌入 | ✅（RFC 0017）| ❌ | ❌ | ❌ | ❌ | ✅ |
| `auroraview run` | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| `auroraview pack`（写 overlay）| ❌ | ❌ | ✅ | ✅ | ❌ | ✅ |
| Packed app 运行时 | ❌ | ❌ | ❌ | ✅ (固化) | ✅ | ✅ |

详细取值表见原 RFC 0013 §4.5.1（拆分后保留为权威表）。

---

## 5. IPC 事件协议（保持不变）

复用 `DragDropEventData::to_json` 输出的现有 schema：

| Event Name | Payload 字段 |
|---|---|
| `file_drop_hover` | `{ hovering: true, paths: string[], position: {x,y} }` |
| `file_drop` | `{ paths: string[], position: {x,y}, timestamp: u64 }` |
| `file_drop_cancelled` | `{ hovering: false, reason: "left_window" }` |

> `Over` 事件被 helper 显式过滤掉（频次过高），与现状一致。
>
> **关于 `hovering` 字段**：`file_drop` payload **不包含** `hovering` 字段（只在 `file_drop_hover` / `file_drop_cancelled` 中出现）。前端**应通过事件名判断状态**，不要依赖统一的 `hovering` 字段。
>
> **作用域限定**：`file_drop*` 事件**只来自挂载了 `with_drag_drop_handler` 的那个 webview 自身**，不会跨 webview 传播：
>
> - 主窗 `capture_file_drop=True` **不会**传染给 `window.open` 触发的 child window（child window 永不挂 handler）；
> - Browser 内部业务 tab + controller 都不挂 handler（详见 RFC 0016）；
> - 同一进程内多个独立 `AuroraView` 实例之间，`file_drop*` 按各自 IPC 通路独立分发。

### 5.1 前端文档（替代原 D7/D16 诊断机制）

`docs/zh/guide/file-drop.md` 新增"故障排查"段：

> 如果你设置了 `capture_file_drop=True` 但 `auroraview.on('file_drop', ...)` 没收到事件，请检查：
>
> 1. 你的前端代码是否仍在使用 `window.addEventListener('drop', ...)`——开启 IPC 模式后浏览器原生 HTML5 拖放会被 wry/WebView2 上游 Bug 完全屏蔽，两者互斥；
> 2. IPC 桥是否已就绪（在 `window.addEventListener('auroraviewready', ...)` 内订阅，避免太早注册）；
> 3. 该 webview 是否真的挂了 handler——child window / Browser 模式下的 webview 永远不挂（详见 RFC 0015 §3.6 / RFC 0016）。

---

## 6. 测试方案

### 6.1 Rust 单元 / 集成测试

`crates/auroraview-core/tests/builder_tests.rs` 新增：

- **`attach_drag_drop_handler_smoke_capture_false`** — 调用 `attach_drag_drop_handler(builder, false, &Arc::new(NoopSink))`，断言编译通过、无 panic。
- **`attach_drag_drop_handler_does_not_clone_sink_when_capture_false`**：

  ```rust
  let sink = Arc::new(CountingSink::default());
  let before = Arc::strong_count(&sink);
  let _builder = attach_drag_drop_handler(builder, false, &sink);
  // helper 接受 &Arc<S>，capture=false 时连 Arc::clone 都不会发生；
  // strong_count 必须严格保持 before 值。
  assert_eq!(Arc::strong_count(&sink), before);
  assert_eq!(sink.dispatch_count(), 0);
  ```

- **`attach_drag_drop_handler_clones_sink_exactly_once_when_capture_true`** — `capture=true` 路径下 `Arc::strong_count` 严格 +1。
- **`attach_drag_drop_handler_dispatches_to_sink_when_capture_true`** — 单独直接 unit-test `DragDropHandler` 把事件喂给 sink 的路径。断言事件名映射、JSON payload 形状、`Over` 被过滤。
- **`dragdrop_dispatch_error_logged`** — sink dispatch 返回 `Err(DispatchError::Backend(...))` 时事件被丢弃 + 至少一条 `tracing::error!`（用 `tracing-test`）；输出文本包含 `"Failed to dispatch"`（**不**按变体精确匹配，因为当前只有单变体）。
- **`dragdropipcsink_blanket_send_sync`** — `assert_send_sync::<dyn DragDropIpcSink>()`，仅编译通过即可。
- **`test_child_window_does_not_register_drag_drop_handler`** — 由 §7.5 CI grep 守住"`child_window.rs` 中不存在 `with_drag_drop_handler` 字面量"。

`crates/auroraview-pack/tests/config_tests.rs` 新增 manifest 解析用例（`[security].capture_file_drop = true / false / 缺省`）。

`crates/auroraview-cli/tests/run_args_tests.rs` + `crates/auroraview-cli/tests/pack_args_tests.rs` 新增（`RunArgs` 与 `PackArgs` 现在共享 `Optional[bool]` 形态，测试矩阵也镜像）：

- 都不传 → `resolve_capture_file_drop(&args) == None`
- `--capture-file-drop` 单独传 → `Some(true)`
- `--no-capture-file-drop` 单独传 → `Some(false)`
- `--capture-file-drop --no-capture-file-drop` → `Some(false)`（`overrides_with` 后传覆盖前传）
- `--no-capture-file-drop --capture-file-drop` → `Some(true)`（同上、相反顺序，钉死 clap `overrides_with` 语义）
- `pack_merge_rule` 单元测试（仅 `pack_merge_rule_tests.rs`）：覆盖 §4.4 取值表中 pack 入口的所有有效组合。
- `packed_env_var_override`：mock `AURORAVIEW_CAPTURE_FILE_DROP=1/0/未设置/无效值` 4 种情况，断言行为符合 §4.3，识别命中时有 `tracing::info!`、无效时有 `tracing::warn!`。

### 6.2 测试 helper

放在 `crates/auroraview-core/tests/common/sinks.rs`：

- **`NoopSink`** — `dispatch` 返回 `Ok(())`，无副作用。
- **`CountingSink`** — 统计 `dispatch` 调用次数 + 可选构造选项控制返回 `Err(DispatchError)`。

### 6.3 手工冒烟矩阵

| 模式 | `capture=false`（默认）| `capture=true`（显式）|
|---|---|---|
| Standalone | HTML5 `drop` 可用 / `file_drop` IPC 不触发 | HTML5 `drop` 失效 / `file_drop` IPC 触发 |
| DCC（Maya 2025）| HTML5 `drop` 可用 / IPC 不触发 | HTML5 `drop` 失效 / IPC 触发 |
| Packed App（无 env var） | 同 Standalone | 同 Standalone（`auroraview pack --capture-file-drop`）|
| Packed + `AURORAVIEW_CAPTURE_FILE_DROP=1` | IPC 触发（无视 overlay 中 `false`）| IPC 触发 |
| Packed + `AURORAVIEW_CAPTURE_FILE_DROP=0` | 不触发 | 不触发（无视 overlay 中 `true`，**关键回归点**）|
| Packed + `AURORAVIEW_CAPTURE_FILE_DROP=hello`（无效）| 沿用 overlay + 一条 `tracing::warn!` | 同左（**关键回归点**）|
| Child window（主窗任意设置）| HTML5 `drop` 可用（永远，永不挂）| 仍永不挂（详见 §3.6）|

每条手工冒烟用例都需在 PR 描述或 release notes 中附截屏/录屏证据。

---

## 7. 实施步骤

1. **Step 1 — Core helper**：`attach_drag_drop_handler` + `DragDropIpcSink` trait + `DispatchError`（单变体）+ `crates/auroraview-core/Cargo.toml` 新增 `thiserror` 依赖（如不存在）+ `IpcHandler` / `IpcRouter` / packed `IpcSink` 三处 `impl DragDropIpcSink` + `crates/auroraview-core/tests/builder_tests.rs` 单元测试。前置依赖 RFC 0014 已合并。
2. **Step 2 — `WebViewConfig` 字段**：`src/webview/config.rs` 新增字段 + `WebViewBuilder::capture_file_drop` 链式方法。
3. **Step 3 — Builder 改造（5 处）**：路径 1（`webview/backend/native.rs`）/ 路径 2（`webview/desktop/webview_builder.rs`）/ 路径 6（`auroraview-cli/cli/run.rs`）/ 路径 7（`auroraview-cli/packed/webview/mod.rs` × 2）/ 路径 8（`auroraview-desktop/window/builder.rs`）。每处插入位置：在所有其它 `.with_xxx()` 调用**之后**、`build()` **之前**调用 `builder = attach_drag_drop_handler(builder, config.capture_file_drop, &ipc_handler);`。

   > **路径 1（DCC `NativeBackend`）的 PR 描述必须分开标注两件事**：(a) 代码等价封装（保留现有 IPC 转发逻辑、事件 schema、`DragDropHandler` 行为）；(b) 运行时行为 Breaking（DCC 默认从 `true` 变 `false`）。

4. **Step 4 — Pack 链路**：`SecurityManifestConfig.capture_file_drop` + `PackConfig.capture_file_drop` + `from_manifest` 映射 + `PackArgs` flag 对 + `resolve_capture_file_drop` 辅助函数 + `resolve_packed_capture_file_drop` + `parse_truthy`。
5. **Step 5 — Child window 边界**：`child_window.rs` 模块 docstring + `NewWindowConfig.new_window_mode` docstring 警示。
6. **Step 6 — 文档与示例**：`docs/zh/guide/file-drop.md`（含 §5.1 故障排查段，替代原 D7/D16 诊断机制）+ CHANGELOG `### Breaking Changes` + DCC 迁移指南（"显式传 `capture_file_drop=True` 即可恢复旧行为"）+ gallery / examples 检查。

每步通过 `vx just test` 验证。Browser 模式相关改动（路径 3/4/9/10）由 RFC 0016 在本 RFC 之后单独提交。

---

## 8. 兼容性

### 8.1 Breaking

⚠️ **DCC（Qt/Maya/Houdini/Nuke）嵌入场景默认行为变化**：

- 之前默认拦截浏览器拖放、自动发出 `file_drop` IPC；新版本默认不再拦截。
- 影响范围：依赖 `auroraview.on('file_drop', ...)` 的 DCC 工具。
- **迁移**：DCC 用户在构造 `AuroraView` 时显式传 `capture_file_drop=True`（依赖 RFC 0017 完成 Python 透传链）。
- 告知渠道：CHANGELOG、docs/zh/guide、release notes 显著标注。

零特例理由：
1. 心智模型一致；
2. 与上游 Bug 解耦（DCC 之前默认 `true` 实际是建立在 wry/WebView2 上游 Bug 副作用上的"伪稳态"）；
3. 迁移成本仅一行代码。

### 8.2 非破坏区域

- IPC schema、事件名、`DragDropHandler` 内部行为均不变。
- CLI 既有 flag 全部保留；新增 flag 默认关闭。
- `auroraview.pack.toml` 旧字段全部保留；`#[serde(default)]` 保证 4 种新旧交叉组合都安全兜底为 `false`，**无需 bump overlay version**。

### 8.3 版本与发布节奏

随下一个 0.x.0 minor 版本发布（不预留过渡 deprecation warning shim）；DCC 用户迁移成本极低（一行代码），引入 warning shim 反而会让"已正确传 `True`"的用户困惑。

---

## 9. 风险

| 风险 | 评估 | 对策 |
|---|---|---|
| DCC 用户因默认值变化丢失拖放 | 中 | 详见 §8.3；迁移仅需一行代码 |
| Helper 签名 `&Arc<S>` 调用方误删 `&` | 低 | 编译期类型不匹配会立即报错；§6.1 测试断言 strong_count 不变作为兜底 |
| `wry::WebViewBuilder` 生命周期 `'a` 与未来 wry API 演进耦合 | 低 | RFC 0014 已通过 workspace dep 集中管理；wry breaking 升级时一处升级全 workspace 同步 |
| `DispatchError` 单变体可观测性弱 | 低 | 当前 IpcHandler 错误就是 `String`，本身就无法精确归类；未来 enum 化后追加变体不算 breaking |
| Child window 不支持 IPC 代理 | 低 | Rust API 不暴露（接口纯净，无 silent failure）+ Python `NewWindowConfig` docstring 警示 + §5 IPC 作用域限定段 |

---

## 10. 后续依赖与 RFC

- **RFC 0016**（Browser 模式禁用）依赖本 RFC 的 helper：`Browser::new` / `TabManager::new` 在调用 `attach_drag_drop_handler` 时永远传 `capture=false`。
- **RFC 0017**（Python 三态契约）依赖本 RFC 的 `WebViewConfig.capture_file_drop` 字段：Python 透传链一路到达该字段。
- 未来 IpcHandler 错误类型 enum 化后，可独立 RFC 给 `DispatchError` 追加 `Disconnected` / `Serialization` 等语义化变体（`#[non_exhaustive]` 保证向后兼容）。
- 上游 wry 修复 hybrid 模式后，给 `DragDropHandler` 加事件穿透开关（helper 签名无需变更）。
