# RFC 0013: 默认文件拖放处理器开关与重构

- **状态**：Draft v9
- **作者**：grimeszhang
- **创建日期**：2026-05-18
- **最近修订**：2026-05-18（v9：**⚠️ 语义反转 BREAKING-of-BREAKING**——把字段从"opt-in"改为"opt-out"。`use_default_file_drop = false`（默认）现在表示*安装* wry 文件拖放处理器并 emit `file_drop_*` IPC 事件；`= true` 表示*跳过* wry handler、保留浏览器原生 DnD。详见 §0。v8：①§4.5.3 明确"原 `:162` 的 `let event_loop_proxy = ...` 整行删除"；②**新增 §4.5.0 crate 依赖前置**：`auroraview-desktop` 须给 `auroraview-core` 开启 `wry-builder` feature，否则路径不存在；③§4.6 修正"漂移源头"措辞（`..Default::default()` 那处才是漂移点）；④**§4.5.1 + §4.5.3 拆出独立 RFC 0014**：`PluginEvent` 协议升级与 XSS 修复作为 0013 的前置 PR 单独 land；本 RFC 以"假设 RFC 0014 已合入"为前提描述 desktop runtime 接入。v7：①helper 约束放宽去掉 `Sync`；②§4.5 desktop runtime / tab_manager 补完整代码片段；③§4.5.1 顺手修事件名拼接 XSS 隐患；④CLI 入口本期不暴露开关。字段名沿用 `use_default_file_drop`）
- **⚠️ BREAKING (v9)**：v8 之前的语义"`use_default_file_drop=true` 才注册 wry handler"在 v9 已**反转**为"`=true` 才*跳过*"。所有 v8 已落地的代码、测试、Python kwarg 含义都需要按 §0 同步翻转。

---

## 0. v9 语义反转专栏（READ FIRST）

| 维度 | v8（旧） | v9（新） |
|---|---|---|
| `use_default_file_drop = false`（默认） | 不注册 wry handler，前端拿不到 `file_drop_*` IPC | **注册** wry handler，前端默认能收到 `file_drop_hover` / `file_drop` / `file_drop_cancelled` |
| `use_default_file_drop = true` | 注册 wry handler | **跳过** wry handler，保留浏览器原生 DnD（`<input type="file">` 等） |
| `Default::default()` | `false` = 不装 | `false` = **装**（数值未变，含义反了） |
| Python kwarg 不传 / `None` | 回退 Rust default（=不装） | 回退 Rust default（=**装**） |
| Helper 形参名 | `enabled: bool` | `use_default_file_drop: bool`（含义即"是否使用浏览器*自带*的 default handler 而**不**走 wry"） |

**为什么反转**：v8 的 opt-in 模型把"前端拿到完整文件路径"这个最常见的需求藏在了 kwarg 后面，违反最小惊讶。v9 改为 opt-out：默认能用，特殊场景（前端只想要 `<input type="file">`）才显式 `=true` 关闭。

**v9 落地清单（已合入此 PR 的改动）**：
1. `crates/auroraview-core/src/builder/drag_drop.rs::install_default_file_drop_with`：形参 `enabled` → `use_default_file_drop`，判断从 `if !enabled { return builder; }` 改为 `if use_default_file_drop { return builder; }`，doc 同步反转。
2. `src/webview/drag_drop_bridge.rs::install_default_file_drop_to_ipc`：形参同步重命名，语义透传。
3. `crates/auroraview-browser/src/tab/manager.rs`：外层 `if self.config.use_default_file_drop` 反转为 `if !self.config.use_default_file_drop`；内层硬编码 `true` 改为 `false`（因为外层已决定要装，新语义下"装"=`false`）。
4. `src/webview/tab_manager.rs` / `src/webview/desktop.rs` / `src/webview/backend/native.rs`：实参传递不变（仍是 `config.use_default_file_drop`），仅注释反转。
5. `crates/auroraview-core/tests/builder_tests.rs`：测试名 `file_drop_helper_disabled_skips_callback` → `file_drop_helper_skips_callback_when_use_default_file_drop_true`；辅助函数形参重命名；其他测试断言不变（因为它们走 `create_drag_drop_handler` 而非 `install_default_file_drop_with`，不受参数语义影响）。
6. Python 高层封装 `python/auroraview/core/webview.py` / `python/auroraview/integration/qt/_core.py` 的 docstring 反转；Python 测试 `tests/python/unit/test_file_drop_events.py` 等仅 docstring 调整（这些测试都是 signature/forwarding 测，断言行为本身就与语义反转正交）。

**v9 未变的部分**：字段名 `use_default_file_drop` / 函数名 `install_default_file_drop_with` / Python kwarg 名 / 事件名（`file_drop_hover` / `file_drop` / `file_drop_cancelled`）/ 三处 Config 结构 / 整个 §4 接入拓扑——以下章节如果与上表不一致，以上表为准。

---

## 1. 概述

当前仅 `desktop.rs` 与 `backend/native.rs` 两处直接调用 `wry::WebViewBuilder::with_drag_drop_handler`，把 OS 拖放事件桥接为 IPC 事件——实际推给前端的有 3 个：`file_drop_hover`（Enter）、`file_drop`（Drop）、`file_drop_cancelled`（Leave）；`file_drop_over` 因频次过高被 `helpers::create_drag_drop_handler` 静默丢弃。其余 builder 入口都没有桥接（历史盲区）。

本 RFC：

1. 在 `auroraview-core::builder::drag_drop` 暴露 **low-level helper** `install_default_file_drop_with`（不耦合 `IpcHandler`）。
2. 在 `auroraview::webview::drag_drop_bridge` 暴露 **high-level helper** `install_default_file_drop_to_ipc`（仅 IpcHandler 入口复用）。
3. 在 3 个 Config 上新增 `use_default_file_drop: bool`，**默认 `false`**（CLI 入口本期不暴露，详见 §4.10）。
4. 3 个 PyO3 入口同步暴露 `use_default_file_drop: Option<bool>` kwarg，未传时回退到对应 `Default::default()`。

> 不把 OS 拖放事件源迁入 Plugin 体系——`with_drag_drop_handler` 是 builder-time only API，迁移并不彻底。

---

## 2. 入口盘点

> 数据来源：`rg "with_drag_drop_handler|WebViewBuilder::new|new_with_web_context"` 实测结果。

| # | 入口 | Python API | builder 调用点 | 现状 | v4 后 |
|---|---|---|---|---|---|
| 1 | Standalone | `run_desktop(...)` | `src/webview/desktop.rs:330` | ✅ 有桥接（`:719`） | ✅ |
| 2 | Qt 嵌入 / Packed AuroraView | `AuroraView(...)` | `src/webview/backend/native.rs:1139` | ✅ 有桥接（`:1460`） | ✅ |
| 3 | Multi-tab Browser | `run_browser(...)` | `src/webview/tab_manager.rs:469`（content）/`:984`（UI） | ❌ 无 | ✅（仅 content tab） |
| 4 | Desktop Runtime | `run_desktop_app(DesktopConfig(...))` | `crates/auroraview-desktop/src/window/builder.rs:88` | ❌ 无 | ✅ |
| 5 | CLI Run | `auroraview-cli run` | `crates/auroraview-cli/src/cli/run.rs:363` | ❌ 无 | ⚠️ **本期不接**（见 §4.10） |
| 6 | CLI Packed | `auroraview-cli` 打包出的 exe | `crates/auroraview-cli/src/packed/webview.rs:841,939` | ❌ 无 | ⚠️ **本期不接**（见 §4.10） |
| 7 | Multi-tab 内层 | （同 #3 下层，未直接暴露 Python） | `crates/auroraview-browser/src/{tab/manager.rs:122, browser.rs:545}` | ❌ 无 | ✅（Rust API） |
| 8 | Child Window（Win-only） | `AuroraView` 内部弹窗 | `src/webview/child_window.rs:72` | ❌ 无 | ⚠️ **不**新增 kwarg（见 §4.6） |

#1/#2 共享 `WebViewConfig`；#3 用 `TabManagerConfig`；#4 用 `auroraview-desktop::DesktopConfig`；#5/#6 用 `auroraview-cli::RunArgs`（本期不动）；#7 是 #3 内部组件；#8 是 fire-and-forget 函数。

→ 字段加到 **3 个 Config**（`WebViewConfig` / `TabManagerConfig` / `DesktopConfig`）；PyO3 暴露 **3 个 kwarg**（`run_desktop` / `AuroraView::new` 共享 `WebViewConfig`，`run_browser`，`PyDesktopConfig`）。

---

## 3. 设计

### 3.1 两层 helper

```
auroraview-core::builder::install_default_file_drop_with        (low-level)
                ▲                                  ▲
                │                                  │
auroraview::webview::drag_drop_bridge      其它无 IpcHandler 的入口
::install_default_file_drop_to_ipc         直接调 low-level，自定义 on_event：
（包装 Arc<IpcHandler>，给 #1/#2 用）       - tab_manager → EventLoopProxy
                                            - desktop runtime → IpcRouter
                                            - cli/packed → 自有路由
```

**为什么拆两层**：`tab_manager` / `auroraview-desktop` / `auroraview-cli` 不持有 `IpcHandler`，无法复用 high-level helper，但都能直接调 low-level。

### 3.2 关键约束

- `with_drag_drop_handler` 是 **builder-time only**：必须在 webview 构造前确定。本 RFC 不提供运行时 setter。
- 与 `ScopeConfig::fs` 无关：`fs` scope 控制插件命令的磁盘访问；本开关只控制 wry handler 是否注册。

---

## 4. 详细变更

### 4.1 Low-level helper（`auroraview-core`，**新增函数**）

`crates/auroraview-core/src/builder/drag_drop.rs`（该模块已在 `wry-builder` cfg gate 下，新函数无需重复 `#[cfg]`）：

```rust
/// 根据 `enabled` 决定是否注册默认文件拖放处理器（low-level）。
///
/// - `false` → 直接返回原 builder。
/// - `true`  → 注册 wry handler，把 OS 拖放事件转成 `(event_name, json)` 交给 `on_event`。
///
/// 仅推送 3 类事件（`Over` 因频次过高在 `helpers::create_drag_drop_handler` 内静默丢弃）：
/// `file_drop_hover` / `file_drop` / `file_drop_cancelled`。
///
/// 不依赖任何 IPC / EventLoop 抽象。
///
/// # Bound 设计说明
///
/// 这里 callback 只要 `Send + 'static`，**不要求 `Sync`**。原因：
/// - 底层 `helpers::create_drag_drop_handler` 内部用 `Arc::new(callback)` 包一层，
///   wry 的 handler trait 也只要求 `'static`；`Sync` 是过度约束。
/// - desktop runtime 接入端会把 `tao::EventLoopProxy<UserEvent>` 捕获进 closure，
///   而 `EventLoopProxy<T>` 在某些平台**不是** `Sync`（仅 `Send`）；若此处要求 `Sync`
///   会直接卡死该接入。
pub fn install_default_file_drop_with<'a, F>(
    builder: wry::WebViewBuilder<'a>,
    enabled: bool,
    log_tag: &'static str,
    on_event: F,
) -> wry::WebViewBuilder<'a>
where
    F: Fn(&str, serde_json::Value) + Send + 'static,
{
    if !enabled {
        tracing::debug!("[{}] Default file-drop handler disabled", log_tag);
        return builder;
    }
    tracing::debug!("[{}] Installing default file-drop handler", log_tag);
    builder.with_drag_drop_handler(crate::builder::create_drag_drop_handler(on_event))
}
```

> ⚠️ `wry::WebViewBuilder<'a>` 必须带生命周期（`'a` 绑定到 `WebContext` 借用），不能写成 `wry::WebViewBuilder`。

> ⚠️ **连带改动**：底层 `crates/auroraview-core/src/builder/helpers.rs::create_drag_drop_handler` 当前签名是 `F: Fn(&str, Value) + Send + Sync + 'static`，需要**同步放宽为 `Send + 'static`**——否则上层 helper 即便去掉 `Sync` 约束，调用 `create_drag_drop_handler(on_event)` 依然会被卡。`DragDropHandler::new` 内部仅 `Arc::new(callback)`，不要求 `Sync`，本次放宽是合法且最小化的改动。已使用方（`tests/builder_tests.rs` 的三处 `move |...| atomic.fetch_add(...)` 与 `Mutex::lock()` 等）都已经满足 `Send`，无回归。

`builder/mod.rs` 在 `wry-builder` feature 段补一行 `pub use drag_drop::install_default_file_drop_with;`。

### 4.2 High-level helper（`auroraview`，**新文件**）

`src/webview/drag_drop_bridge.rs`：

```rust
//! wry drag-drop → IpcHandler 桥接。仅供有 IpcHandler 的入口（desktop / native）复用。
use std::sync::Arc;
use crate::ipc::{IpcHandler, IpcMessage};

pub fn install_default_file_drop_to_ipc<'a>(
    builder: wry::WebViewBuilder<'a>,
    ipc_handler: Arc<IpcHandler>,
    enabled: bool,
    log_tag: &'static str,
) -> wry::WebViewBuilder<'a> {
    auroraview_core::builder::install_default_file_drop_with(
        builder, enabled, log_tag,
        move |event_name, data| {
            let msg = IpcMessage { event: event_name.to_string(), data, id: None };
            if let Err(e) = ipc_handler.handle_message(msg) {
                tracing::error!("[{}] Error handling {}: {}", log_tag, event_name, e);
            }
        },
    )
}
```

`src/webview/mod.rs` 加 `pub mod drag_drop_bridge;`。

### 4.3 配置字段（3 处同形）

三个 Config 各加一个字段，`Default = false`，并提供 fluent setter `with_default_file_drop(self, bool)`：

| Config | 文件 | 备注 |
|---|---|---|
| `WebViewConfig` | `src/webview/config.rs` | 字段紧贴 `allow_file_protocol`；无 serde 影响 |
| `TabManagerConfig` | `src/webview/tab_manager.rs:248` | 无 serde 影响 |
| `auroraview-desktop::DesktopConfig` | `crates/auroraview-desktop/src/config.rs` | 派生了 Serde，必须加 `#[serde(default)]` |

> CLI（`auroraview-cli::RunArgs`）本期**不**新增 flag，理由见 §4.10。

字段文档统一措辞（核心信息，不啰嗦）：

```rust
/// 是否注册内置 wry 文件拖放处理器。开启时，OS 拖放事件被桥接为 3 个 IPC 事件：
/// `file_drop_hover` / `file_drop` / `file_drop_cancelled`
/// （`Over` 类事件因频次过高在 helper 内静默丢弃，不向前端推送）。
///
/// 默认 `false`：浏览器原生 DnD（如 `<input type="file">`）保持工作。
/// 设为 `true`：可以拿到完整文件路径（浏览器出于安全限制不暴露的能力）。
///
/// **Builder-time only**：必须在 WebView 创建前设置。
pub use_default_file_drop: bool,
```

### 4.4 Backend 接入（IpcHandler 入口，high-level helper）

| 文件 | 替换位置 | log_tag |
|---|---|---|
| `src/webview/desktop.rs` | `:716-731` 整段 | `"standalone"` |
| `src/webview/backend/native.rs` | `:1457-1472` 整段 | `"NativeBackend"` |

替换形如：

```rust
webview_builder = crate::webview::drag_drop_bridge::install_default_file_drop_to_ipc(
    webview_builder, ipc_handler.clone(), config.use_default_file_drop, "standalone");
```

### 4.5 Backend 接入（其它入口，low-level helper）

#### 4.5.0 前置：crate 依赖检查（必做，否则下面所有 low-level 接入都编译失败）

`install_default_file_drop_with` 与 `create_drag_drop_handler` 都在 `auroraview-core` 的 **`wry-builder` feature gate 下**（参 `crates/auroraview-core/Cargo.toml:80-81` `wry-builder = ["wry"]`）。当前各 crate 的依赖配置：

| crate | 当前 | 需要 | 说明 |
|---|---|---|---|
| `auroraview`（主 crate） | ✅ `features = ["wry-builder"]` | — | 现状即可 |
| `auroraview-browser` | ✅ `features = ["wry-builder"]` | — | 现状即可 |
| `auroraview-desktop` | ❌ 无 feature | ✅ **必须加 `features = ["wry-builder"]`** | 不加则 `auroraview_core::builder::install_default_file_drop_with` 路径不存在，§4.5.3 的接入代码直接编译失败 |
| `auroraview-cli` / `auroraview-dcc` | ❌ 无 feature | — | 本期不接（§4.10 / §4.7） |

**改动**：`crates/auroraview-desktop/Cargo.toml:12`：

```toml
auroraview-core = { path = "../auroraview-core", features = ["wry-builder"] }
```

> 不引入循环依赖：`auroraview-core` 仅向 `auroraview-plugins` / `auroraview-signals` / `auroraview-assets` 反向依赖，与 `auroraview-desktop` 之间是单向依赖关系（已实测）。
>
> 该改动属于 §7 步骤 8 的**前置子步**（编号为 8a-pre），独立可编译，不引入任何运行时行为变化（仅打开未使用的可选编译单元）。

| 文件 | 接入位置 | 事件路由 | log_tag |
|---|---|---|---|
| `src/webview/tab_manager.rs` | `:469`（content tab）；`:984`（UI controller）**不接**（仅渲染 tab bar/toolbar，不展示用户内容） | `proxy.send_event(TabManagerEvent::FileDrop { tab_id, event_name, data })`，**新增**该事件变体（详见 §4.5.2） | `"TabManager"` |
| `crates/auroraview-desktop/src/window/builder.rs` | `:88` 之前注入；**前置改造**：把 `:162` 的 `event_loop.create_proxy()` 提到 `:84` 之后（即 `web_context` 创建前），让 closure 能捕获 proxy | `proxy.send_event(UserEvent::PluginEvent { event: name.into(), data })`（直接透传 `serde_json::Value`，依赖 **RFC 0014** 协议升级；详见 §4.5.1 / §4.5.3） | `"DesktopRuntime"` |
| `crates/auroraview-browser/src/tab/manager.rs` | `:122` | 沿用该 crate 自有路由（与 `tab_manager.rs` 同形：`EventLoopProxy<BrowserEvent>` + 新增 `FileDrop` 变体） | `"BrowserTab"` |
| `crates/auroraview-browser/src/browser.rs` | `:545` UI controller **不接**（同上理由） | — | — |

> CLI 入口（#5 / #6）本期**不接**，详见 §4.10。

> 落地前先用 `rg` 复核精确行号。`auroraview-desktop` 的 `IpcRouter` 当前**没有** `emit_event` 公开 API（其设计是 JS→Rust 单向），因此走 `UserEvent::PluginEvent` 通道是最小侵入方案；如果希望规整，可单独立项给 `IpcRouter` 增加 `emit_event(event, data)`。

#### 4.5.1 依赖：RFC 0014 `auroraview-desktop::PluginEvent` 协议升级 + XSS 修复

> ⚠️ **本节内容已拆出为独立 RFC 0014**（`docs/rfcs/0014-desktop-plugin-event-payload_zh.md`）。
>
> 拆分原因：
> - `UserEvent::PluginEvent` 协议升级（`data: String → serde_json::Value`）与 §4.5.3 的 XSS 顺手修，**与"默认文件拖放开关"主题正交**。
> - 二者都是 `auroraview-desktop` 内部协议变更，需独立 review、独立回滚、独立 release notes 条目。
> - 让 RFC 0013 的 BREAKING 公告范围保持单一（仅 file-drop 默认值变更），避免读者把两件事混读。
>
> **依赖关系**：RFC 0014 是本 RFC §4.5.3（desktop runtime 接入 low-level helper）的**前置 PR**——本 RFC §4.5.3 的接入代码假设 `UserEvent::PluginEvent.data` 已经是 `serde_json::Value`。落地顺序：**RFC 0014 先 land → 再 land 本 RFC 步骤 8c**（详见 §7）。
>
> RFC 0014 的范围（仅供本 RFC 读者快速理解依赖面，详细方案见 0014）：
> - `auroraview-desktop::UserEvent::PluginEvent { event, data: String }` → `{ event, data: serde_json::Value }`（`crates/auroraview-desktop/src/event_loop/user_event.rs:19`）
> - `crates/auroraview-desktop/src/event_loop/handler.rs:126-140` 顺手修事件名 + data 的字符串拼接 XSS 隐患（改用 `serde_json::to_string` 转义后嵌入 JS）
> - `tests/event_loop_tests.rs` 三处构造改为 `json!({...})`
>
> 不影响（与 RFC 0014 一致）：
> - `auroraview/src/webview/event_loop.rs::UserEvent::PluginEvent`（主 crate 自己的同名变体，独立类型）
> - `auroraview-cli/src/packed/events.rs::UserEvent::PluginEvent`（packed 自己的，独立类型）
> - `auroraview-core::events::CoreUserEvent::PluginEvent`（无生产消费者，本期不动）
> - 任何 PyO3 / Python 公开 API（`PluginEvent` 是 crate 内部 enum）

#### 4.5.2 `tab_manager` 接入完整代码

**新增事件变体**（在 `src/webview/tab_manager.rs` 现有 `TabManagerEvent` enum 中追加）：

```rust
/// File-drop event from a content tab's wry handler.
/// Routed back to the IpcHandler associated with that tab.
FileDrop {
    tab_id: TabId,
    event_name: &'static str,         // "file_drop_hover" / "file_drop" / "file_drop_cancelled"
    data: serde_json::Value,
},
```

**接入点**（`tab_manager.rs:469` 附近，content tab 的 `WebViewBuilder` 链上）：

```rust
// 前置：proxy 与 tab_id 在该 closure 作用域可见
let proxy_for_drop = self.proxy.clone();          // EventLoopProxy<TabManagerEvent>，已 Send
let tab_id_for_drop = tab_id;                     // Copy

webview_builder = auroraview_core::builder::install_default_file_drop_with(
    webview_builder,
    config.use_default_file_drop,
    "TabManager",
    move |event_name, data| {
        // event_name 是 &'static str（来自 DragDropEventType::as_event_name），可直接 copy
        if let Err(e) = proxy_for_drop.send_event(TabManagerEvent::FileDrop {
            tab_id: tab_id_for_drop,
            event_name,
            data,
        }) {
            tracing::error!("[TabManager] Failed to dispatch FileDrop for tab {:?}: {}", tab_id_for_drop, e);
        }
    },
);
```

**消费端**（`TabManagerEvent` 处理分支，按现有 IpcHandler 持有方式路由）：

```rust
TabManagerEvent::FileDrop { tab_id, event_name, data } => {
    if let Some(ipc) = self.ipc_handlers.get(&tab_id) {
        let msg = IpcMessage { event: event_name.to_string(), data, id: None };
        if let Err(e) = ipc.handle_message(msg) {
            tracing::error!("[TabManager] tab {:?} {} dispatch err: {}", tab_id, event_name, e);
        }
    } else {
        tracing::warn!("[TabManager] FileDrop for unknown tab {:?}", tab_id);
    }
}
```

> `ipc_handlers` 的实际字段名以代码现状为准；落地时按现有 tab→IpcHandler 索引方式套用。

#### 4.5.3 `auroraview-desktop` 接入完整代码

> **前提**：本节代码假设 RFC 0014 已合入——即 `UserEvent::PluginEvent.data` 已经是 `serde_json::Value`。落地顺序见 §7 步骤 8。

**前置 proxy**：在 `crates/auroraview-desktop/src/window/builder.rs` 中，把 `let event_loop_proxy = event_loop.create_proxy();`（当前在 `:162`）**前移到 `:84` 之后、`:88` 之前**，即 `let mut web_context = create_web_context(&config);` 之后、`let mut webview_builder = WebViewBuilder::new_with_web_context(&mut web_context);` 之前：

```rust
// :84 之后插入：
let event_loop_proxy = event_loop.create_proxy();
let proxy_for_drop = event_loop_proxy.clone();   // 给 drag-drop closure 使用

// :88 不变：
let mut web_context = create_web_context(&config);
let mut webview_builder = WebViewBuilder::new_with_web_context(&mut web_context);
```

> ⚠️ 前移后，**原 `:162` 的 `let event_loop_proxy = event_loop.create_proxy();` 整行删除**——该 binding 此时已经在前面创建，重复 `let` 会 shadow，且把 proxy 提前是本次改动的核心目的。下面 `DesktopWindow { event_loop_proxy, .. }` 的构造表达式继续复用前移后的同名 binding（已在作用域内），无需修改。

**接入 helper**（紧跟在 `with_ipc_handler` 注册之前或之后均可，无依赖）：

```rust
webview_builder = auroraview_core::builder::install_default_file_drop_with(
    webview_builder,
    config.use_default_file_drop,
    "DesktopRuntime",
    move |event_name, data| {
        // event_name: &'static str；UserEvent::PluginEvent.event: String
        if let Err(e) = proxy_for_drop.send_event(UserEvent::PluginEvent {
            event: event_name.to_string(),
            data,                                 // 直接透传 serde_json::Value（依赖 RFC 0014）
        }) {
            tracing::error!("[DesktopRuntime] Failed to dispatch PluginEvent {}: {}", event_name, e);
        }
    },
);
```

> 此处 `UserEvent::PluginEvent.data: serde_json::Value` 是 RFC 0014 落地后的协议形态。若 RFC 0014 因故未先 land，本步骤 8c 必须降级为 `data: serde_json::to_string(&data).unwrap_or_else(|_| "null".into())`，但这违反两份 RFC 的合并意图，**强烈不建议**。

**风险记录**：
- 若 RFC 0014 滞后，本节接入代码无法编译——这是有意的依赖约束，确保协议升级不被绕过。
- 消费端的 XSS 修复属于 RFC 0014 范围，本 RFC 不重复列出。

### 4.6 PyO3 入口（3 处统一签名）

三个 PyO3 入口都新增 `use_default_file_drop: Option<bool> = None` kwarg；`Some(b)` 时写入对应 Config 字段，`None` 时不动（回退 `Default::default()` = `false`）：

| 函数 / 类 | 文件 | 写入目标 |
|---|---|---|
| `run_desktop` / `AuroraView::new`（共享 `WebViewConfig`） | `src/bindings/desktop_runner.rs` / `src/webview/core/main.rs` | `WebViewConfig` 构造块（`run_desktop` 处显式列字段、无 `..Default::default()`，编译期天然防漂移；`AuroraView::new` 处末尾有 `..Default::default()`，**漂移源头在此处**——必须显式列出 `use_default_file_drop`，否则即便 PyO3 kwarg 拿到了 `Some(true)` 也会被 `Default::default() = false` 静默覆盖。建议先构造 config（含正确字段值）再 move，不要先 `Default::default()` 后部分赋值） |
| `run_browser` | `src/bindings/tab_browser.rs:78` | `TabManagerConfig` |
| `PyDesktopConfig::new` | `src/bindings/runtime_desktop.rs` | `auroraview_desktop::DesktopConfig`；同时给 `PyDesktopConfig` 加 fluent 方法 |

> CLI（#5/#6）本期不暴露此开关（详见 §4.10）。

### 4.7 Child Window（不改）

`create_child_webview_window(url, w, h)` 是 fire-and-forget API，**本期不**新增入参。理由：
- 调用面很窄（`AuroraView` 内部弹窗），外部 Python 用户不直接用；
- 独立 event loop 与 `IpcHandler` 不共享，**当前根本没有 IPC 通道**——加 file-drop handler 也没有地方路由。

后续如有需求，可调 low-level helper 接 channel，单独立项。

### 4.8 不改 `auroraview-core::CoreConfig`

跨 crate 序列化用的精简版，目前 `auroraview-core` 自身没有任何代码消费 file-drop 开关，因此不在 `CoreConfig` 上加字段。未来 `auroraview-core` 若需要承载该开关，再单独立项。

### 4.9 Python 高层封装

`python/auroraview/**` 中显式 kwargs 透传的位置同步追加 `use_default_file_drop: Optional[bool] = None`；`**kwargs` 直通的位置零改动。落地命令：

```
rg "run_desktop\(|AuroraView\(|run_browser\(|run_desktop_app\(" python/
```

### 4.10 CLI 入口本期不暴露开关

`auroraview-cli` 的两个入口（#5 `cli/run.rs:363`、#6 `packed/webview.rs:841,939`）本期**不新增 `--use-default-file-drop` flag、也不调用 helper**。理由：

1. **CLI 入口当前没有前端可消费的 IPC 通道**：file-drop 事件即便注册了 wry handler，也只能落到 `tracing` 日志，前端拿不到——这违反最小惊讶原则（用户开 flag 期待"前端能收到事件"）。
2. **保留半残 flag 是 footgun**：之前 v6 设计的"flag + 仅 tracing 日志"方案让用户难以分辨 flag 是真生效还是空挡，反而比"明确不支持"体验更差。
3. **关闭默认行为不是 BREAKING**：CLI 入口现状本就**没有** file-drop 桥接（盘点 §2 标 ❌），保持现状即可。

**对外文档措辞**：CLI 模式不支持 `file_drop_*` IPC 事件；如需该能力，请使用 SDK 入口（`run_desktop` / `AuroraView` / `run_browser` / `run_desktop_app`）。

> 后续若 CLI 端引入完整的前端 IPC 通道（如 packed 模式接入 `IpcHandler`），可再立项追加该开关。届时只需在 `RunArgs` 加一行 `#[arg(long)] use_default_file_drop: bool` 并把 helper 接到对应 closure。

---

## 5. 默认值与兼容性

### 5.1 默认值矩阵

| 来源 | 默认 | 与现网差异 |
|---|---|---|
| `WebViewConfig::default()` | `false` | ⚠️ **行为变更**（之前无条件注册） |
| `TabManagerConfig::default()` | `false` | 无（之前就没有） |
| `auroraview-desktop::DesktopConfig::default()` | `false` | 无 |
| `auroraview-cli`（CLI 入口） | — | 本期不暴露开关，沿用"无 file-drop 桥接"现状（详见 §4.10） |
| Python `run_desktop()` / `AuroraView()` 不传 kwarg | 回退 `false` | ⚠️ **行为变更** |

### 5.2 BREAKING 公告

**仅对 #1/#2 是 BREAKING**。Release notes 建议措辞：

> ⚠️ Breaking: `run_desktop` / `AuroraView` 默认不再注册 wry 文件拖放处理器。如果前端依赖 `file_drop_hover` / `file_drop` / `file_drop_cancelled` IPC 事件来获取完整文件路径，请显式传入 `use_default_file_drop=True`。（注：`file_drop_over` 历史上已被 helper 静默丢弃，文档与代码本就不一致，本次顺带在文档上对齐为 3 类事件。）

| 调用方场景 | 变更前 | 变更后 |
|---|---|---|
| 前端用 `file_drop` IPC 拿完整路径 | 自动可用 | 必须显式 `=True` |
| 前端只用浏览器 DnD（`<input type="file">` 等） | 受 wry handler 抑制（参 `auroraview-core/src/builder/drag_drop.rs:156`） | 浏览器默认行为完全恢复 |

### 5.3 Serde 兼容性

- `WebViewConfig` / `TabManagerConfig`：当前未派生 Serde（含 `ProtocolCallback` 等不可序列化字段）→ 加字段无影响。
- `auroraview-desktop::DesktopConfig`：派生 Serde，新字段必须 `#[serde(default)]` 保证旧 JSON/TOML 反序列化得 `false`。

---

## 6. 测试计划

### 6.1 单元测试

| 位置 | 用例 |
|---|---|
| `auroraview-core/tests/builder_tests.rs` | `install_default_file_drop_with(_, false, _, on_event)`：on_event 不被触发；`= true` 时 **3 类**事件（Enter/Drop/Leave）各触发一次，Over 事件不触发 |
| `auroraview-core/tests/builder_tests.rs`（同上） | 默认值一致性：三个 Config 的 `Default::default().use_default_file_drop` 全部为 `false`（防漂移） |
| `auroraview-core/tests/builder_tests.rs`（同上） | helper 仅要求 `Send + 'static`：用一个 `!Sync` 的 callback（如捕获 `Cell<u32>` 的 closure，或 mock 一个 `Send + !Sync` 类型）确保编译通过，防止未来谁不小心把 `Sync` 加回来 |
| `src/webview/config.rs`（已有 test 模块） | `WebViewConfig::default().use_default_file_drop == false` + fluent setter |
| `src/webview/drag_drop_bridge.rs`（新增 `#[cfg(test)]`） | mock `IpcHandler` 接收 3 类事件，断言 `IpcMessage.event` 名称正确 |
| `src/webview/tab_manager.rs` | `TabManagerConfig::default()` 默认 + setter |
| `auroraview-desktop/tests/...` | `DesktopConfig::default()` 默认 + serde round-trip：旧 TOML（无该字段）反序列化得 `false` |
| ~~`auroraview-desktop/tests/event_loop_tests.rs`（XSS 修复回归）~~ | ~~已移交 RFC 0014~~ |

### 6.2 集成测试

| 入口 | 测试 |
|---|---|
| Standalone | `tests/python/unit/test_file_drop_events.py` 改造为显式 `=True` 用例；新增 `=False` 用例验证不发事件 |
| Qt 嵌入 | `tests/python/integration/test_qt_lifecycle.py` 加 `=True/False` 两条 |
| Multi-tab | 加 `run_browser(use_default_file_drop=True)` 用例，断言 `TabManagerEvent::FileDrop` 能消费 |
| Desktop Runtime | 加 `DesktopConfig(use_default_file_drop=True)` 用例 |

### 6.3 跨平台手动 Checklist

| 平台 | 场景 | `=true` 期望 | `=false` 期望 |
|---|---|---|---|
| Windows / macOS / Linux | 拖文件到窗口空白处 | 收到 `file_drop` IPC | 浏览器默认行为 |
| Windows / macOS / Linux | 拖到 `<input type="file">` | 收到 `file_drop` IPC | input 接受文件 |

> `=false` 行为依赖 wry 内部实现，落地前至少在 Windows 跑一次 Checklist 并写进 release notes。

### 6.4 编译矩阵（每步都需通过）

```
cargo check -p auroraview-core --features wry-builder
cargo check -p auroraview --features wry-builder
cargo check -p auroraview-desktop
cargo check -p auroraview-cli
cargo check -p auroraview-browser
cargo check --all-targets
```

---

## 7. 落地步骤（每步独立可编译）

> **跨 RFC 依赖**：步骤 8c 依赖 RFC 0014 已 land。**RFC 0014 必须先于本 RFC 步骤 8c 合入**——若 0014 进度受阻，可先合入步骤 1-7（与 0014 完全无关），步骤 8 起暂停，等 0014 合入后再继续。

1. Low-level helper + 单测 → `auroraview-core`（**含底层 `helpers::create_drag_drop_handler` 同步去掉 `Sync` 约束**，参 §4.1）
2. High-level helper → `src/webview/drag_drop_bridge.rs` + `mod.rs`
3. `WebViewConfig` 加字段 + fluent + 单测
4. `desktop.rs` / `native.rs` 切到 high-level helper
5. PyO3 `run_desktop` / `AuroraView::new` 暴露 kwarg
6. `TabManagerConfig` + `TabManagerEvent::FileDrop`（含消费端分支，参 §4.5.2） + `tab_manager.rs:469` 接 low-level
7. PyO3 `run_browser` 透传
8. `auroraview-desktop`（拆 3 个 commit，皆独立可编译）：
   - **8a-pre**：`crates/auroraview-desktop/Cargo.toml` 给 `auroraview-core` 开启 `wry-builder` feature（参 §4.5.0）。**该子步必须独立成 commit/PR**，纯依赖配置，零运行时影响，便于 bisect。
   - **8a**：~~（原 PluginEvent 协议升级 + XSS 修复，已拆出至 RFC 0014）~~
   - **8b**：`DesktopConfig` 加 `use_default_file_drop` 字段（`#[serde(default)]`）+ fluent
   - **8c**：`window/builder.rs` 把 `event_loop.create_proxy()` 前移（**原 `:162` 那行整行删除**）；接入 low-level helper（参 §4.5.3）。**前提：RFC 0014 已 land**。
9. PyO3 `PyDesktopConfig` 透传 + fluent
10. `auroraview-browser` 接 low-level（仅 Rust API）
11. Python 高层封装显式透传（按需）
12. 集成测试（`tests/python/**`）
13. 跨平台手动 Checklist（§6.3）
14. 文档：`docs/zh/` 加迁移指引；`CHANGELOG.md` 标 BREAKING；说明 `file_drop_over` 文档对齐；说明 CLI 入口本期不暴露开关（参 §4.10）

> 推荐拆 PR：
> - **(0014)** RFC 0014：`auroraview-desktop::PluginEvent` 协议升级 + XSS 修复（独立 RFC，独立 PR）
> - **(a)** helper（含底层约束放宽）+ `WebViewConfig` + #1/#2 切换 + PyO3 透传 + 单测（步骤 1-5）。**与 (0014) 并行无依赖**。
> - **(b1)** tab_manager（`TabManagerEvent::FileDrop` + `:469` 接入）+ PyO3 `run_browser`（步骤 6-7）。**与 (0014) 并行无依赖**。
> - **(b2)** auroraview-desktop：**前置 8a-pre（Cargo feature）** + 8b（Config）+ 8c（接入，依赖 0014）+ PyO3 `PyDesktopConfig`（步骤 8-9）
> - **(c)** auroraview-browser + Python 封装 + 集成测试 + 文档（步骤 10-14）
>
> (a) / (b1) 与 (0014) 完全独立，可三路并行；(b2) 必须等 (0014) 合入。

---

## 8. 风险与权衡

| 风险 | 缓解 |
|---|---|
| BREAKING 让现有依赖 `file_drop_*` 的应用静默失效 | §5.2 release notes；docs 迁移指引；按 SemVer 提 minor++（0.x）或 major++（1.x） |
| 多 Config 同名字段未来语义漂移 | 单测断言三处默认值一致 |
| 关闭后浏览器 DnD 行为依赖 wry 实现 | §6.3 跨平台 Checklist；release notes 注明已验证矩阵 |
| 多入口同时改动，单 PR 巨大 | §7 拆 PR：(0014) / (a) / (b1) / (b2) / (c) 五路 |
| `enabled=true` 时事件分发失败导致路径丢失 | 与现状一致（`tracing::error!` + 丢弃）；如需 fallback 单独立项 |
| 多 PyO3 入口默认值漂移（某入口改成 `Some(true)` 默认而其它没跟上） | §6.1 单测断言"三处 Config 默认值全为 `false`" |
| `WebViewConfig` 的 `..Default::default()` 构造路径漏写新字段，导致 PyO3 kwarg 静默失效 | §4.6 措辞警告；Code review checklist；建议使用 fluent setter 链而非部分赋值 |
| RFC 0014 进度受阻 → 本 RFC 步骤 8c 阻塞 | 步骤 1-7 与 0014 完全无关，可先 land；0014 合入前 desktop runtime 维持现状（无 file-drop 桥接，与盘点 §2 当前列一致） |
| `auroraview-desktop` 开启 `wry-builder` feature 后引入 wry 编译依赖 | 仅放大 desktop crate 自身的 wry 依赖范围（已 transitively 通过 `wry = "0.54.4"` 直接依赖在 `Cargo.toml:16`），实测无新增 crate 引入 |
| helper 约束放宽到 `Send + 'static` 后未来误回退到 `Sync` | §6.1 增加 `!Sync` callback 编译用例 |
| `auroraview-core::CoreUserEvent::PluginEvent` 与 `auroraview-desktop` 的协议漂移 | 由 RFC 0014 处理；本 RFC 不引入新漂移 |
| CLI 入口本期不接，用户期望可能错配 | §4.10 文档明确说明；`run_desktop`/`AuroraView`/`run_browser`/`run_desktop_app` 仍可覆盖绝大多数场景 |

---

## 9. 非目标

- 不把 OS 拖放事件源迁入 Plugin 体系（builder-time API 限制下迁移不彻底）。
- 不改 `auroraview-core::builder::drag_drop` 的事件抽象。
- 不新增 JS 端可 invoke 的 drag-drop 命令。
- 不在 `auroraview-core::CoreConfig` 加字段。
- 不给 `child_window.rs` 加入参。
- 不实现"运行时 setter"——wry API 限制下只对**之后创建**的 webview 生效，徒增预期错配。

