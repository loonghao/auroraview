# RFC 0015: `attach_drag_drop_handler` Shared Helper + 7-Builder Unification

- Number: 0015
- Title: File-drop proxy unification via `attach_drag_drop_handler` + `WebViewConfig.capture_file_drop` switch
- Status: Draft
- Created: 2026-05-21
- Authors: AuroraView Core Team
- Split from: RFC 0013 §4.1 / §4.2.1 / §4.3.1 / §4.4
- Prerequisites: **RFC 0014** (centralize `wry` / `tao` via workspace dep)
- Affected files:
  - `crates/auroraview-core/src/builder/helpers.rs` (adds `attach_drag_drop_handler` + `DragDropIpcSink` trait + `DispatchError`)
  - `crates/auroraview-core/src/builder/mod.rs` (`pub use`)
  - `crates/auroraview-core/Cargo.toml` (adds `thiserror` dep if not present)
  - `src/webview/config.rs` (`WebViewConfig.capture_file_drop` + `WebViewBuilder::capture_file_drop`)
  - `src/ipc/handler.rs` (`impl DragDropIpcSink for IpcHandler`)
  - `src/webview/backend/native.rs` (path 1)
  - `src/webview/desktop/webview_builder.rs` (path 2)
  - `crates/auroraview-cli/src/cli/run.rs` (path 6 + `RunArgs`)
  - `crates/auroraview-cli/src/cli/pack.rs` (`PackArgs` + `resolve_capture_file_drop`)
  - `crates/auroraview-cli/src/packed/webview/mod.rs` (path 7 ×2 + `resolve_packed_capture_file_drop`)
  - `crates/auroraview-cli/src/packed/ipc.rs` (`impl DragDropIpcSink`)
  - `crates/auroraview-pack/src/manifest.rs` (`SecurityManifestConfig.capture_file_drop`)
  - `crates/auroraview-pack/src/config.rs` (`PackConfig.capture_file_drop` + `from_manifest` mapping)
  - `crates/auroraview-desktop/src/config.rs` (`DesktopConfig.capture_file_drop`)
  - `crates/auroraview-desktop/src/window/builder.rs` (path 8)
  - `crates/auroraview-desktop/src/ipc/router.rs` (`impl DragDropIpcSink for IpcRouter`)
  - `src/webview/child_window.rs` (module docstring)
  - `python/auroraview/core/config.py::NewWindowConfig` (docstring caveat)

> The "never attach" branch for Browser mode is handled separately by RFC 0016; the Python tri-state contract is handled separately by RFC 0017. This RFC covers only the Rust-side helper and 5 of the 7 builder call sites (paths 1/2/6/7/8); paths 3/4/5/9/10 are out of scope.

---

## 1. Summary

Introduce a `capture_file_drop: bool` switch (default `false`) that uniformly controls whether `wry::WebViewBuilder::with_drag_drop_handler` is registered:

- `false` → no `with_drag_drop_handler` call; the WebView uses browser-native HTML5 drag-drop semantics.
- `true` → `with_drag_drop_handler` is registered; events are proxied as IPC `file_drop_hover` / `file_drop` / `file_drop_cancelled`.

All run modes (standalone / DCC / CLI / packed / desktop crate) share the same default of `false`, **with zero exceptions** — this is a breaking change relative to the current implementation (DCC default flips from `true` to `false`).

---

## 2. wry Behavior and the Upstream Bug (Critical)

**Official contract** ([wry docs](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_drag_drop_handler)):

> Return `true` in the callback to block the OS' default behavior.
> Note, that if you do block this behavior, it won't be possible to drop files on `<input type="file">` forms.

**Reality (verified upstream bug)**: on Windows (WebView2), **calling `with_drag_drop_handler` at all — regardless of whether the closure returns `true` or `false` — suppresses HTML5 `dragenter` / `dragover` / `drop` inside the WebView**. Confirmed upstream:

- [tauri-apps/tauri#15138](https://github.com/tauri-apps/tauri/issues/15138)
- [tauri-apps/wry#157](https://github.com/tauri-apps/wry/issues/157)

**Conclusion**: `capture_file_drop` must be toggled by **whether `with_drag_drop_handler` is called at all**, not by the handler returning `false`.

**Two mutually-exclusive modes** (developers must pick one):

- `capture_file_drop = false` (default) → handler is never registered; HTML5 drag-drop works (suitable for Monaco / CodeMirror / rich-text uploads, etc.).
- `capture_file_drop = true` → handler is registered; HTML5 drag-drop becomes inert inside the WebView, but IPC delivers full local paths.

If upstream wry/WebView2 fixes the hybrid/passthrough behavior in the future, the helper interface (§3.3) signature requires no change.

---

## 3. Helper Design

### 3.1 `DragDropIpcSink` trait + `DispatchError`

In `crates/auroraview-core/src/builder/helpers.rs`:

```rust
use std::sync::Arc;

/// Errors that may occur while dispatching a drag-drop event into the IPC pipeline.
///
/// **Currently a single-variant form**: `IpcHandler::handle_message`'s error
/// type is `String`, which cannot be precisely classified into semantic
/// variants like `Disconnected` / `Serialization`. Once IpcHandler's error
/// type is enum-ified (a separate refactor across the entire IPC subsystem,
/// decoupled from this RFC), `Disconnected` / `Serialization` variants will
/// be added.
///
/// `#[non_exhaustive]` ensures appending variants in the future is not a
/// breaking change.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DispatchError {
    /// Any sink-side underlying error.
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl DispatchError {
    /// Convenience constructor: wraps any `Send + Sync + 'static` error as a
    /// `Backend` variant.
    pub fn backend<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Backend(Box::new(err))
    }
}

/// Trait abstraction over the IPC entry point expected by the helper.
///
/// Implementations are responsible only for forwarding errors, not for
/// logging — logging is centralized in the helper. This avoids the three
/// impls each emitting inconsistent log formats.
pub trait DragDropIpcSink: Send + Sync + 'static {
    /// Forward a single drag-drop event into the IPC pipeline.
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), DispatchError>;
}
```

### 3.2 Three IPC Entry-Point `impl DragDropIpcSink`

Each impl is ≤ 5 lines. `IpcHandler::handle_message` currently returns `Result<_, String>`; wrap with `DispatchError::backend`:

```rust
// Appended at the bottom of src/ipc/handler.rs:

// String does not implement std::error::Error, so wrap it. Display passes
// the string through; DispatchError::Backend carries the wrapper.
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

`auroraview-desktop::IpcRouter` and `auroraview-cli::packed::ipc` get a mirrored `impl DragDropIpcSink`.

> **Once IpcHandler is enum-ified later**: change `String` to `IpcError`, and in `map_err` classify variants precisely into `DispatchError::Disconnected` / `DispatchError::Serialization` / etc. The trait signature does not need to change (`DispatchError` is `#[non_exhaustive]`).

### 3.3 `attach_drag_drop_handler` Function Signature

```rust
/// Conditionally attach the drag-drop proxy handler.
///
/// - `capture == false` — the builder is returned unchanged; `with_drag_drop_handler`
///   is **not** called; wry uses browser-native HTML5 drag-drop semantics.
///   `ipc_sink` is passed only as a **borrow**; the helper does **not**
///   `Arc::clone`, so the caller's `Arc::strong_count` truly stays unchanged
///   (independently testable, see §6.1).
/// - `capture == true` — the helper performs **exactly one** `Arc::clone`
///   internally, builds a `Send + Sync + 'static` wry callback, and attaches
///   it. Events are filtered by `DragDropHandler::into_handler` (`Over` is
///   dropped), then forwarded as `file_drop_hover` / `file_drop` /
///   `file_drop_cancelled` to `sink.dispatch(...)`. If dispatch returns
///   `Err(DispatchError)`, the helper logs a single `tracing::error!` inside
///   the closure and discards the event (the drag-drop path must never block).
///
/// # Borrow form
///
/// `ipc_sink: &Arc<S>` rather than `Arc<S>`:
/// - when `capture=false`, the caller's stack-side `Arc::strong_count` truly
///   stays at the original value;
/// - when `capture=true`, the helper internally calls `Arc::clone(ipc_sink)`
///   once (one atomic increment, < 5 ns);
/// - the caller writes `&ipc_handler` rather than `ipc_handler.clone()`,
///   shorter and clearer: "I'm letting the helper peek; whether to clone
///   is its decision."
///
/// # Lifetime parameter `'a`
///
/// The actual lifetime of `wry::WebViewBuilder` depends on its constructor:
/// - `WebViewBuilder::new()` — `'static`;
/// - `WebViewBuilder::new_with_web_context(&mut web_context)` — borrows the
///   `web_context` lifetime, **not** `'static`.
///
/// The repo's main business paths use `new_with_web_context`, so the helper
/// must use a generic lifetime `'a` to support both forms.
///
/// # Static dispatch
///
/// `where S: DragDropIpcSink` (no `?Sized`). All call sites pass
/// `&Arc<concrete-type>`; the compiler monomorphizes one copy per concrete
/// type.
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
        // ipc_sink is borrowed; the helper does not retain it; strong_count
        // is unchanged.
        return builder;
    }

    let sink = Arc::clone(ipc_sink); // the helper's only clone point

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

### 3.4 Module Exports

In `crates/auroraview-core/src/builder/mod.rs`:

```rust
pub use helpers::{attach_drag_drop_handler, DispatchError, DragDropIpcSink};
```

`DragDropHandler` / `DragDropEventData` / `as_event_name` / `create_drag_drop_handler` are all preserved unchanged.

### 3.5 Caller Pattern

```rust
use auroraview_core::builder::attach_drag_drop_handler;

// All 5 builder call sites (paths 1/2/6/7×2/8) unify to:
builder = attach_drag_drop_handler(
    builder,
    config.capture_file_drop,   // bool
    &ipc_handler,                // &Arc<IpcHandler>; helper decides whether to clone
);
```

When `!capture`, the caller's stack-side `Arc::strong_count` truly stays unchanged, and the contract is independently testable.

### 3.6 Handling for Non-attached Paths

| # | Path | File | Treatment |
|---|---|---|---|
| 3 | tab webview (legacy) | `src/webview/tab_manager.rs:469` | Handled by RFC 0016 (never attached) |
| 4 | tab controller (legacy) | `src/webview/tab_manager.rs:984` | Handled by RFC 0016 (never attached) |
| 5 | child window | `src/webview/child_window.rs` | Never attached; the constructor does not accept `capture_file_drop`; module-top docstring marks the scope limitation; `python/auroraview/core/config.py::NewWindowConfig.new_window_mode` docstring appends a `Note (RFC 0015)` block clarifying that events do not propagate across webviews |
| 9 | tab webview (new) | `crates/auroraview-browser/src/tab/manager.rs:122` | Handled by RFC 0016 |
| 10 | tab controller (new) | `crates/auroraview-browser/src/browser.rs:545` | Handled by RFC 0016 |

#### 3.6.1 Child Window Design Notes

- `create_child_webview_window`'s actual signature is `(url: &str, width: u32, height: u32)` — it does **not** accept config, so no new parameter is needed.
- `child_window.rs` gets a module-top docstring:

  ```rust
  //! # Drag-drop behavior
  //!
  //! Child windows do not currently support the `capture_file_drop` IPC
  //! proxy. Pages loaded in a child window can use the browser-native
  //! HTML5 drag-drop API (`dragenter` / `dragover` / `drop`) directly.
  //! If your tool needs absolute file paths via IPC, open a top-level
  //! `AuroraView` instead, where `capture_file_drop=True` is supported.
  ```

- `NewWindowConfig.new_window_mode`'s docstring appends:

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

## 4. Configuration Layer & CLI

### 4.1 Configuration Fields

#### `WebViewConfig` (`src/webview/config.rs`)

```rust
pub struct WebViewConfig {
    // ...
    pub capture_file_drop: bool,
}
```

`Default::default()` sets `capture_file_drop: false`. The `WebViewBuilder` chain method (**no `with_` prefix**, consistent with existing `title()` / `url()` / `allow_file_protocol()` style):

```rust
impl WebViewBuilder {
    pub fn capture_file_drop(mut self, capture: bool) -> Self {
        self.config.capture_file_drop = capture;
        self
    }
}
```

#### `DesktopConfig` (`crates/auroraview-desktop/src/config.rs`)

Mirror `WebViewConfig`: add `capture_file_drop: bool` at the top level and a same-named chain method on `impl DesktopConfig`. The two are not auto-synchronized; each builder reads from its own.

#### `auroraview-pack` (manifest + `PackConfig` two layers)

**Manifest layer** (`crates/auroraview-pack/src/manifest.rs::SecurityManifestConfig`):

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

**`PackConfig` layer** (`crates/auroraview-pack/src/config.rs`):

```rust
pub struct PackConfig {
    // ...
    #[serde(default)]
    pub capture_file_drop: bool,
}
```

`PackConfig::from_manifest`:

```rust
pack_config.capture_file_drop = manifest
    .security
    .as_ref()
    .and_then(|s| s.capture_file_drop)
    .unwrap_or(false);
```

> **Sanity check**: `PackConfig`'s current `content_security_policy` field placement (top level vs nested) needs a second pass at Step 1 implementation time; this RFC does not assume a specific layer, only that `capture_file_drop` and `content_security_policy` sit at the same layer with the same `#[serde(default)]`.

**Overlay binary compatibility**: `#[serde(default)]` makes all 4 cross-combinations of new/old runtime × new/old overlay safely fall back to `false`; **no overlay version bump is needed**.

### 4.2 CLI Flags

#### `RunArgs` (`crates/auroraview-cli/src/cli/run.rs`)

> **Revision (post-implementation)**: the original proposal was a single-direction `--capture-file-drop` bool flag, inconsistent with `PackArgs`'s shape. Subsequent review pointed out that, when `auroraview run` later wires up manifest / env var, a single-direction flag would force "user did not pass" to be interpreted as "explicit false" overriding the lower layer. Now aligned with `PackArgs`: bidirectional flags + `resolve_capture_file_drop` recovering `Option<bool>`. The `run` entry point currently calls `unwrap_or(false)` to land, equivalent to the original code default.

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

`pub fn resolve_capture_file_drop(args: &RunArgs) -> Option<bool>` has identical semantics to `pack.rs`'s same-named function. The current `run` entry point directly calls `.unwrap_or(false)` to land; once manifest / env var are wired up, switching to `.or(manifest...).or(env...).unwrap_or(false)` is a smooth chain change without breaking the existing CLI surface.

#### `PackArgs` (`crates/auroraview-cli/src/cli/pack.rs`)

A pair of explicit flags + `resolve_capture_file_drop` recovering `Option<bool>`:

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

> **Erratum (2026-05)**: `action = clap::ArgAction::SetFalse` in the `no-capture-file-drop` block above **is incorrect**; implementations must use `clap::ArgAction::SetTrue`. Reason: clap 4's `ArgAction::SetFalse` defaults the flag's value to `true` **when absent** (symmetric with `SetTrue`), which would shift every match arm in `resolve_capture_file_drop` below — a default invocation would return `Some(false)` instead of `None`, and passing only `--capture-file-drop` would hit the `unreachable!()` and panic the process. The correct approach is `SetTrue` for both flags, with `overrides_with` ensuring `(true, true)` is unreachable. Verified against clap 4.6 with `try_parse_from` covering all 5 input combinations. Reference: <https://docs.rs/clap/latest/clap/builder/enum.ArgAction.html>.

Helper function (recommended at the top of `pack.rs`, reusable when `RunArgs` is upgraded later):

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

**Pack-stage merge rule**:

```
overlay value = resolve_capture_file_drop(&pack_args)
    .or(manifest.security.and_then(|s| s.capture_file_drop))
    .unwrap_or(false);
```

> **Why not clap `Option<bool>` + `num_args = 0..=1`**: positional-arg ambiguity (`auroraview pack --capture-file-drop my-app.toml` would treat `my-app.toml` as the flag's value), and the form is inconsistent with RunArgs. Mature CLIs like `cargo` / `rustup` / `wrangler` all use explicit flag pairs.

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

### 4.3 Packed Env-Var Escape Hatch

End users with a packed exe cannot re-pack, so a runtime switch is needed:

| Env-var value (case-insensitive, trimmed) | Meaning |
|---|---|
| `1` / `true` / `on` / `yes` / `enabled` | Force enable |
| `0` / `false` / `off` / `no` / `disabled` | Force disable |
| Unset | Use the overlay value |
| Set but invalid (e.g. `=hello`) | Use the overlay value + emit one `tracing::warn!` |

Implementation in `crates/auroraview-cli/src/packed/webview/mod.rs`:

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

> **Source notes for the recognized literal set**: `1/0` (Windows registry), `true/false` (Rust / generic), `on/off` (systemd), `yes/no` (Docker / cron), `enabled/disabled` (some DCC config conventions). This is AuroraView's own union, intuitive for ops users coming from different ecosystems; not deliberately aligned with any single standard.

### 4.4 Configuration Precedence (Unchanged Post-Split)

Entry-point exclusion premise: the 5 sources cannot all appear in the same process; each entry sees at most 2–3 of them.

| Entry | Python kwarg | Run flag | Pack flag | manifest | env var | code default |
|---|---|---|---|---|---|---|
| PyO3 embed | ✅ (RFC 0017) | ❌ | ❌ | ❌ | ❌ | ✅ |
| `auroraview run` | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| `auroraview pack` (writes overlay) | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ |
| Packed app runtime | ❌ | ❌ | ❌ | ✅ (frozen) | ✅ | ✅ |

Detailed value table: see original RFC 0013 §4.5.1 (preserved as the authoritative table post-split).

---

## 5. IPC Event Protocol (Unchanged)

Reuses the existing schema emitted by `DragDropEventData::to_json`:

| Event Name | Payload Fields |
|---|---|
| `file_drop_hover` | `{ hovering: true, paths: string[], position: {x,y} }` |
| `file_drop` | `{ paths: string[], position: {x,y}, timestamp: u64 }` |
| `file_drop_cancelled` | `{ hovering: false, reason: "left_window" }` |

> `Over` events are explicitly filtered by the helper (too high a frequency), unchanged from current behavior.
>
> **About the `hovering` field**: the `file_drop` payload does **not** include a `hovering` field (it appears only in `file_drop_hover` / `file_drop_cancelled`). Front-end code **should branch on the event name**, not rely on a unified `hovering` field.
>
> **Scope limitation**: `file_drop*` events come **only from the webview that actually attached `with_drag_drop_handler`**, with no cross-webview propagation:
>
> - A main window with `capture_file_drop=True` does **not** propagate to child windows opened via `window.open` (child windows never attach the handler);
> - Browser-internal business tabs and the controller never attach (see RFC 0016);
> - Multiple independent `AuroraView` instances within the same process dispatch `file_drop*` independently along each instance's IPC path.

### 5.1 Front-end Documentation (Replaces D7/D16 Diagnostic Mechanism)

`docs/zh/guide/file-drop.md` adds a "Troubleshooting" section:

> If you set `capture_file_drop=True` but `auroraview.on('file_drop', ...)` does not receive events, check:
>
> 1. Whether your front-end code still uses `window.addEventListener('drop', ...)` — once IPC mode is enabled, browser-native HTML5 drag-drop is fully suppressed by an upstream wry/WebView2 bug; the two are mutually exclusive.
> 2. Whether the IPC bridge is ready (subscribe inside `window.addEventListener('auroraviewready', ...)` to avoid registering too early).
> 3. Whether the relevant webview actually attached the handler — child windows / Browser-mode webviews never attach (see RFC 0015 §3.6 / RFC 0016).

---

## 6. Test Plan

### 6.1 Rust Unit / Integration Tests

`crates/auroraview-core/tests/builder_tests.rs` adds:

- **`attach_drag_drop_handler_smoke_capture_false`** — calls `attach_drag_drop_handler(builder, false, &Arc::new(NoopSink))`, asserts compilation and no panic.
- **`attach_drag_drop_handler_does_not_clone_sink_when_capture_false`**:

  ```rust
  let sink = Arc::new(CountingSink::default());
  let before = Arc::strong_count(&sink);
  let _builder = attach_drag_drop_handler(builder, false, &sink);
  // The helper takes &Arc<S>; with capture=false, no Arc::clone happens at all,
  // so strong_count must strictly stay at `before`.
  assert_eq!(Arc::strong_count(&sink), before);
  assert_eq!(sink.dispatch_count(), 0);
  ```

- **`attach_drag_drop_handler_clones_sink_exactly_once_when_capture_true`** — under `capture=true`, `Arc::strong_count` strictly increments by 1.
- **`attach_drag_drop_handler_dispatches_to_sink_when_capture_true`** — directly unit-tests the path of `DragDropHandler` feeding events into the sink. Asserts event-name mapping, JSON payload shape, and that `Over` is filtered.
- **`dragdrop_dispatch_error_logged`** — when sink dispatch returns `Err(DispatchError::Backend(...))`, the event is dropped and at least one `tracing::error!` fires (use `tracing-test`); the output text contains `"Failed to dispatch"` (we do **not** match precisely on the variant, since there is currently only a single variant).
- **`dragdropipcsink_blanket_send_sync`** — `assert_send_sync::<dyn DragDropIpcSink>()`, compile-only.
- **`test_child_window_does_not_register_drag_drop_handler`** — guarded by §7.5 CI grep ("`child_window.rs` must not contain the `with_drag_drop_handler` literal").

`crates/auroraview-pack/tests/config_tests.rs` adds manifest parsing cases (`[security].capture_file_drop = true / false / omitted`).

`crates/auroraview-cli/tests/run_args_tests.rs` + `crates/auroraview-cli/tests/pack_args_tests.rs` are added (`RunArgs` and `PackArgs` now share the `Optional[bool]` shape; their test matrices mirror each other):

- Neither flag passed → `resolve_capture_file_drop(&args) == None`
- `--capture-file-drop` alone → `Some(true)`
- `--no-capture-file-drop` alone → `Some(false)`
- `--capture-file-drop --no-capture-file-drop` → `Some(false)` (`overrides_with` lets the later flag win)
- `--no-capture-file-drop --capture-file-drop` → `Some(true)` (same, opposite order — pins clap's `overrides_with` semantics)
- `pack_merge_rule` unit test (only in `pack_merge_rule_tests.rs`): covers all valid combinations of the pack entry in §4.4's value table.
- `packed_env_var_override`: mocks `AURORAVIEW_CAPTURE_FILE_DROP=1/0/unset/invalid` (4 cases), asserts behavior matches §4.3, with `tracing::info!` on recognized hits and `tracing::warn!` on invalid values.

### 6.2 Test Helpers

In `crates/auroraview-core/tests/common/sinks.rs`:

- **`NoopSink`** — `dispatch` returns `Ok(())`, no side effects.
- **`CountingSink`** — counts `dispatch` calls + an optional construction option to return `Err(DispatchError)`.

### 6.3 Manual Smoke Matrix

| Mode | `capture=false` (default) | `capture=true` (explicit) |
|---|---|---|
| Standalone | HTML5 `drop` works / `file_drop` IPC does not fire | HTML5 `drop` inert / `file_drop` IPC fires |
| DCC (Maya 2025) | HTML5 `drop` works / IPC does not fire | HTML5 `drop` inert / IPC fires |
| Packed app (no env var) | Same as Standalone | Same as Standalone (`auroraview pack --capture-file-drop`) |
| Packed + `AURORAVIEW_CAPTURE_FILE_DROP=1` | IPC fires (overrides overlay `false`) | IPC fires |
| Packed + `AURORAVIEW_CAPTURE_FILE_DROP=0` | Does not fire | Does not fire (overrides overlay `true`; **key regression point**) |
| Packed + `AURORAVIEW_CAPTURE_FILE_DROP=hello` (invalid) | Uses overlay value + one `tracing::warn!` | Same (**key regression point**) |
| Child window (any main-window setting) | HTML5 `drop` works (always; never attached) | Still never attached (see §3.6) |

Each manual smoke case requires screenshot/screencast evidence in the PR description or release notes.

---

## 7. Implementation Steps

1. **Step 1 — Core helper**: `attach_drag_drop_handler` + `DragDropIpcSink` trait + `DispatchError` (single variant) + add `thiserror` dep to `crates/auroraview-core/Cargo.toml` if absent + `IpcHandler` / `IpcRouter` / packed `IpcSink` three `impl DragDropIpcSink` + `crates/auroraview-core/tests/builder_tests.rs` unit tests. Prerequisite: RFC 0014 has landed.
2. **Step 2 — `WebViewConfig` field**: `src/webview/config.rs` adds the field + `WebViewBuilder::capture_file_drop` chain method.
3. **Step 3 — Builder rewrite (5 sites)**: path 1 (`webview/backend/native.rs`) / path 2 (`webview/desktop/webview_builder.rs`) / path 6 (`auroraview-cli/cli/run.rs`) / path 7 (`auroraview-cli/packed/webview/mod.rs` × 2) / path 8 (`auroraview-desktop/window/builder.rs`). Insertion point at each: **after** all other `.with_xxx()` calls and **before** `build()`, call `builder = attach_drag_drop_handler(builder, config.capture_file_drop, &ipc_handler);`.

   > **Path 1 (DCC `NativeBackend`) PR description must call out two things separately**: (a) code-equivalent encapsulation (existing IPC-forwarding logic, event schema, and `DragDropHandler` behavior preserved); (b) runtime-behavior breaking change (DCC default flips from `true` to `false`).

4. **Step 4 — Pack chain**: `SecurityManifestConfig.capture_file_drop` + `PackConfig.capture_file_drop` + `from_manifest` mapping + `PackArgs` flag pair + `resolve_capture_file_drop` helper + `resolve_packed_capture_file_drop` + `parse_truthy`.
5. **Step 5 — Child window boundary**: `child_window.rs` module docstring + `NewWindowConfig.new_window_mode` docstring caveat.
6. **Step 6 — Docs and examples**: `docs/zh/guide/file-drop.md` (with §5.1 troubleshooting section, replacing D7/D16) + CHANGELOG `### Breaking Changes` + DCC migration guide ("explicitly pass `capture_file_drop=True` to restore the old behavior") + gallery / examples sweep.

Each step is verified with `vx just test`. Browser-mode-related changes (paths 3/4/9/10) ship after this RFC, in RFC 0016.

---

## 8. Compatibility

### 8.1 Breaking

⚠️ **DCC (Qt/Maya/Houdini/Nuke) embedded scenarios undergo a default-behavior change**:

- Previously the browser drop was intercepted by default and `file_drop` IPC fired automatically; the new version no longer intercepts by default.
- Affected: DCC tools that rely on `auroraview.on('file_drop', ...)`.
- **Migration**: DCC users explicitly pass `capture_file_drop=True` when constructing `AuroraView` (depends on the Python passthrough chain in RFC 0017).
- Communication channels: CHANGELOG, docs/zh/guide, release notes prominently called out.

Reasons for zero exceptions:
1. Mental model consistency.
2. Decouples from the upstream bug (DCC's previous `true` default was a "pseudo-stable" state built on top of the wry/WebView2 upstream-bug side-effect).
3. Migration cost is one line of code.

### 8.2 Non-breaking Areas

- IPC schema, event names, and `DragDropHandler` internals are unchanged.
- All existing CLI flags are preserved; new flags default to off.
- All existing `auroraview.pack.toml` fields are preserved; `#[serde(default)]` makes all 4 cross-combinations safely fall back to `false`. **No overlay version bump required.**

### 8.3 Versioning and Release Cadence

Ships in the next 0.x.0 minor release (no transitional deprecation-warning shim); DCC migration cost is extremely low (one line of code), and a warning shim would only confuse users who have already correctly passed `True`.

---

## 9. Risks

| Risk | Assessment | Mitigation |
|---|---|---|
| DCC users lose drag-drop due to the default change | Medium | See §8.3; migration is one line |
| Helper signature `&Arc<S>`; callers may forget the `&` | Low | Type mismatch fails immediately at compile time; the §6.1 strong_count assertion is a backstop |
| `wry::WebViewBuilder` lifetime `'a` couples to future wry API evolution | Low | RFC 0014 already centralizes via workspace dep; a wry breaking bump propagates everywhere from one place |
| Single-variant `DispatchError` weakens observability | Low | Current IpcHandler error is `String` and inherently unclassifiable; once enum-ified, appending variants is non-breaking |
| Child window does not support IPC proxy | Low | Rust API does not expose it (clean interface, no silent failure) + Python `NewWindowConfig` docstring caveat + §5 IPC scope-limitation paragraph |

---

## 10. Follow-up Dependencies and RFCs

- **RFC 0016** (Browser mode disabled) depends on this RFC's helper: `Browser::new` / `TabManager::new` always pass `capture=false` when calling `attach_drag_drop_handler`.
- **RFC 0017** (Python tri-state contract) depends on this RFC's `WebViewConfig.capture_file_drop` field: the Python passthrough chain reaches that field end to end.
- Once IpcHandler error type is enum-ified, an independent RFC can add `Disconnected` / `Serialization` / etc. semantic variants to `DispatchError` (`#[non_exhaustive]` guarantees backward compatibility).
- After upstream wry fixes hybrid mode, an event-passthrough switch can be added to `DragDropHandler` (helper signature requires no change).
