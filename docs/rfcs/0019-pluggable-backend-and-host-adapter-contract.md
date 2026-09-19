# RFC 0019: Pluggable Rendering Backend and Host Adapter Contract

- Number: 0019
- Title: Make the WebView backend a real pluggable seam, and define the contract that `auroraview-unreal` / `auroraview-maya` / `auroraview-unity` implement
- Status: Draft
- Created: 2026-09-19
- Authors: AuroraView Core Team
- Affected code: `crates/auroraview-core/src/backend/`, `src/webview/backend/`, `crates/auroraview-ue`, `python/auroraview/utils/thread_dispatcher/`, `python/auroraview/core/backend.py`, `crates/auroraview-cli/skills/`
- Related: RFC 0007 (WebView/Browser unified architecture), RFC 0011 (unified IPC), PR #460 (DCC-MCP WebView adapter)

> **Note on numbering.** This document was referred to as "RFC 0008" during planning.
> `0008` is already taken twice (`0008-ai-agent-integration.md`, `0008-auroraview-testing-redesign.md`),
> so it is filed as `0019`, the next free number. Content is unchanged.

---

## 0. Scope and non-goals

**In scope:** design only. No implementation code ships with this RFC.

**Out of scope (deliberately deferred):**

- Creating GitHub repositories or moving code between repositories. The physical split
  into `try-auroraview/*` is blocked on an owner decision. Every contract below is written
  so that it works **identically whether the adapter lives in this monorepo or in its own repo**.
- CEF/Chromium implementation work. This RFC fixes the seam and the feature names; the
  second backend lands after the seam is proven.
- Changing the organization's CI topology.

---

## 1. Current state (verified against `main` @ v0.5.10)

The planning assumption "the backend is hardcoded to wry, there is no abstraction" is
**half wrong**. There are in fact *two* backend abstractions, and neither is the seam we need.

### 1.1 Layer 1 — `auroraview_core::backend` (exists, aspirational, off the hot path)

`crates/auroraview-core/src/backend/` contains a complete, well-documented abstraction:

| File | Contents |
| --- | --- |
| `traits.rs` (289 L) | `WebViewBackend`, `EmbeddableBackend`, `EventLoopBackend` |
| `factory.rs` (172 L) | `BackendType` enum, `BackendConfig`, `BackendFactory`, `AURORAVIEW_BACKEND` env var |
| `wry_impl.rs` (362 L) | `WryBackend` — the only implementation |
| `lifecycle.rs` (448 L) | `AtomicLifecycle` lock-free state machine |
| `message_processor.rs` (356 L) | `MessageProcessor`, `ProcessingMode` |
| `settings.rs` (161 L) | `WebViewSettings` / `WebViewSettingsImpl` |

Problems:

1. **Nothing on the hot path uses it.** The only consumers are
   `crates/auroraview-core/tests/backend_tests.rs` and a re-export block in
   `src/webview/backend/mod.rs`. It is a parallel abstraction.
2. **The enum lies.** `BackendType` declares four variants, but `BackendFactory::create`
   maps `WebView2` to `WryBackend` (an alias, not a backend), and returns
   `Err(WebViewError::Internal("... not yet implemented"))` for `WKWebView` and `WebKitGTK`.
3. **`mod wry_impl` is private.** `wry_impl` is not `pub`, so no external crate can
   implement against it or learn from it.

### 1.2 Layer 2 — `PyBindingsBackend` (the real hot path, leaks wry/tao)

The code that actually creates a WebView is `NativeBackend`
(`src/webview/backend/native.rs`, 1382 L), which implements the `PyBindingsBackend` trait
declared in `src/webview/backend/mod.rs` (242 L).

`PyBindingsBackend` is a good operational trait — `load_url`, `load_html`, `eval_js`,
`emit`, `process_events`, `process_ipc_only`, `run_blocking`, `lifecycle_state`,
`request_close` — but **three of its methods leak wry/tao types into the signature**:

```rust
fn webview(&self) -> Arc<Mutex<WryWebView>>;                              // wry::WebView
fn window(&self) -> Option<&tao::window::Window>;                         // tao
fn take_event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>>; // tao
```

These three signatures are the entire reason a second backend is impossible today.
Any new backend would have to produce a `wry::WebView`, which is a contradiction.

### 1.3 Dependency facts

- Root crate `Cargo.toml`: `wry = { workspace = true }` and `tao = { workspace = true }`,
  **unconditional**. `wry 0.54.4` / `tao 0.34.6` are pinned at `[workspace.dependencies]`.
- `crates/auroraview-core`: `wry` is **already optional**, gated behind the `wry-builder`
  feature. So core is closer to being backend-free than the root crate is.
- Other direct wry users that will need to route through the seam:
  `crates/auroraview-browser/src/{browser.rs,tab/manager.rs,tab/state.rs}`,
  `crates/auroraview-desktop/src/window/{builder.rs,mod.rs}`,
  `src/webview/desktop/webview_builder.rs`, `src/webview/{child_window,event_loop,message_processor,protocol_handlers,tab_manager,webview_inner}.rs`,
  `crates/auroraview-cli/src/{cli/run.rs,packed/webview/*,protocol_handlers.rs}`.

### 1.4 Python surface that must not break

`BackendType` is **public API**:

- `python/auroraview/core/backend.py` defines `BackendType` (`WRY`, `WEBVIEW2`, `WKWEBVIEW`,
  `WEBKITGTK`) and `get_backend_type()`.
- It is re-exported from `python/auroraview/core/__init__.py` and
  `python/auroraview/__init__.py`, and listed in `__all__`.

`AURORAVIEW_BACKEND` is also already a documented env var on both sides
(`BackendFactory::ENV_BACKEND` in Rust, docstring in `backend.py`), accepting
`wry` / `webview2`|`wv2`|`webview_2` / `wkwebview`|`wk`|`webkit` / `webkitgtk`|`gtk`.

**This is the good news: the user-facing selection mechanism already exists and already
has the name we want. It just does not reach the hot path.**

---

## 2. Design — Part 1: the pluggable backend

### 2.1 Principle: promote, do not reinvent

We do **not** introduce a third abstraction. We:

1. Promote `auroraview_core::backend::WebViewBackend` from Layer 1 to the single contract.
2. Repair Layer 2 so that `PyBindingsBackend` no longer names wry/tao types.
3. Make `BackendFactory` a registry instead of a hardcoded `match`.

### 2.2 The two traits

**`Backend`** — a *factory*, one value per compiled backend. Owns nothing, creates WebViews.

```rust
/// A compiled-in rendering backend. One instance per process.
pub trait Backend: Send + Sync {
    /// Stable name, must match the `AURORAVIEW_BACKEND` string and the cargo feature suffix.
    fn name(&self) -> &'static str;

    /// What this backend can do. Lets core degrade gracefully instead of failing at
    /// call time. Queried before construction, so it must be cheap and static.
    fn capabilities(&self) -> BackendCapabilities;

    /// Create a WebView. `parent` is how DCC embedding reaches the backend.
    fn create_webview(&self, init: BackendInit) -> WebViewResult<Box<dyn BackendWebView>>;
}

#[derive(Debug, Clone, Default)]
pub struct BackendCapabilities {
    /// Can attach to a foreign native window (HWND / NSView / X11 id).
    pub native_parenting: bool,
    /// Can run its own blocking event loop (standalone desktop mode).
    pub owned_event_loop: bool,
    /// Exposes a CDP endpoint, enabling `auroraview-testing` and agent automation.
    pub cdp: bool,
    /// Supports the offscreen capture path used by `window.auroraview.screenshot`.
    pub screenshot: bool,
}

pub struct BackendInit {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub url: Option<String>,
    pub html: Option<String>,
    /// Native parent window, when embedding into a DCC. `None` = own a top-level window.
    pub parent: Option<NativeHandle>,
    pub asset_root: Option<PathBuf>,
    pub settings: WebViewSettingsImpl,
    /// Host owns the message loop (Qt/DCC mode) => backend must not pump natively.
    pub skip_message_pump: bool,
    /// Ceiling on IPC messages per tick; 0 = unlimited. Needed by Houdini's busy main thread.
    pub max_messages_per_tick: usize,
}
```

**`BackendWebView`** — a *created instance*. This is `WebViewBackend` + `EmbeddableBackend`
+ `EventLoopBackend` from Layer 1, merged and trimmed to the minimum face the request asks for
(window / content / JS / events back), plus the event-loop cooperation methods that
`NativeBackend` already implements.

```rust
pub trait BackendWebView: Send + Sync {
    // ---- window ----
    fn native_handle(&self) -> Option<NativeHandle>;
    fn set_bounds(&self, x: i32, y: i32, w: u32, h: u32) -> WebViewResult<()>;
    fn set_visible(&self, visible: bool) -> WebViewResult<()>;
    fn focus(&self) -> WebViewResult<()>;

    // ---- content ----
    fn load_url(&self, url: &str) -> WebViewResult<()>;
    fn load_html(&self, html: &str) -> WebViewResult<()>;

    // ---- JS ----
    fn eval_js(&self, script: &str) -> WebViewResult<()>;
    fn eval_js_with_callback(
        &self,
        script: &str,
        cb: JavaScriptCallback,
    ) -> WebViewResult<()>;

    // ---- events back to the host (the missing piece) ----
    /// Installed exactly once, before the first navigation. Replaces the current
    /// ad-hoc channel wiring inside `NativeBackend`.
    fn set_event_sink(&self, sink: Box<dyn BackendEventSink>) -> WebViewResult<()>;

    // ---- event loop cooperation (DCC vs standalone) ----
    /// Full tick. Used in standalone mode, where we own the loop.
    fn process_events(&self) -> ProcessResult;
    /// IPC only, no native pump. Used in Qt/DCC mode, where the host owns the loop.
    fn process_ipc_only(&self) -> ProcessResult;
    /// Blocking run. Only valid when `capabilities().owned_event_loop`.
    fn run_blocking(&mut self) -> WebViewResult<()>;

    // ---- lifecycle ----
    fn lifecycle_state(&self) -> LifecycleState;
    fn request_close(&self) -> WebViewResult<()>;
}
```

**`BackendEventSink`** — how events come *back* out. Today they arrive through closures
captured at construction time inside `native.rs`, which is precisely why the constructor
cannot be shared across backends.

```rust
pub trait BackendEventSink: Send + Sync {
    /// A `window.auroraview.*` call arrived from JS.
    fn on_ipc(&self, msg: IpcMessage);
    fn on_navigation(&self, ev: NavigationEvent);
    fn on_page_title(&self, title: String);
    /// Return `false` to veto the close.
    fn on_close_requested(&self) -> bool;
}
```

### 2.3 `NativeHandle` — killing the `u64`

`BackendConfig::parent_handle` is currently `Option<u64>`, and `EmbeddableBackend::native_handle()`
returns `Option<u64>`. A bare `u64` cannot express a macOS `NSView*`, and it is unsigned on
Windows where `HWND` is signed. Introduce one neutral type and use it on both sides:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHandle {
    Win32(std::num::NonZeroIsize),
    AppKit(NonNull<c_void>),
    X11(u32),
    Wayland(NonNull<c_void>),
}
```

Migration: keep `parent_handle: Option<u64>` on `BackendConfig` as a deprecated field that
converts through `NativeHandle::try_from(u64)`, so no Python or C caller breaks.

### 2.4 Registry instead of `match`

`BackendFactory::create` becomes a registry lookup, and the `cfg`-gated enum is preserved
so the public Rust API does not churn:

```rust
pub struct BackendFactory;

impl BackendFactory {
    pub const ENV_BACKEND: &'static str = "AURORAVIEW_BACKEND"; // unchanged

    /// Backends compiled into this build, in preference order.
    pub fn registered() -> &'static [&'static dyn Backend] {
        &[
            #[cfg(feature = "backend-wry")]
            &WryBackendProvider,
            #[cfg(feature = "backend-cef")]
            &CefBackendProvider,
        ]
    }

    pub fn create(config: &BackendConfig) -> WebViewResult<Box<dyn WebViewBackend>> { ... }
}
```

Selection order, unchanged from today: `AURORAVIEW_BACKEND` env var, then
`config.backend_type`, then first registered backend. Because the env var already exists
and is already documented, **enabling a second backend requires no new user-facing knob**.

### 2.5 Cargo feature naming

Existing precedent in the root `Cargo.toml` is mixed (`feature-tabs`, `features-core`,
`win-webview2`, `runtime-dcc`, `wry-builder`). We add one consistent family and do not
rename anything existing:

| Feature | Effect | Default |
| --- | --- | --- |
| `backend-wry` | `dep:auroraview-wry` (moved out of `wry_impl.rs`); wry 0.54 / tao 0.34 | **yes** |
| `backend-cef` | `dep:auroraview-cef` | no |
| `backend-chromium` | reserved; `dep:auroraview-chromium` | no |

Rules:

1. `backend-wry` is in `default`, so `cargo build` and the published wheel are byte-for-byte
   equivalent to today.
2. Multiple backends may be compiled in; runtime selection is via `AURORAVIEW_BACKEND`.
   Building with **zero** backends is a compile error (add a `compile_error!` guard).
3. `win-webview2` stays as-is; it configures the WebView2 runtime inside the wry backend,
   it is not a backend of its own. This resolves the current `BackendType::WebView2`
   alias confusion: `webview2` remains a valid `AURORAVIEW_BACKEND` string that selects
   the wry backend on Windows, exactly as it does today.

New crate layout under `crates/`:

```
crates/auroraview-wry/        # today's backend/wry_impl.rs, promoted, pub
crates/auroraview-cef/        # placeholder: Backend impl only, no CEF code yet
```

### 2.6 The CEF/Chromium integration point, and why it is harder than wry

CEF is the right second backend to design against because it is the hardest, and it
exposes which parts of our seam are wry-shaped by accident. Three constraints must be
handled by the seam, not worked around per-backend:

1. **Multi-process.** CEF spawns a separate subprocess and needs a helper executable on
   disk. `BackendInit` needs an `app_support: Option<AppSupportPaths>` field (helper path,
   cache dir) that wry simply ignores. `BackendCapabilities` must therefore grow a
   `needs_subprocess_helper: bool`.
2. **Window creation is asynchronous.** With wry, `create_webview` returns a live view.
   With CEF the browser is created on the UI thread and reports readiness later.
   `BackendEventSink` therefore needs `on_created(Result<(), BackendError>)`, and
   `create_webview` must be allowed to return a view whose `load_url` is not yet valid.
   Core already queues operations through `MessageQueue`, so this is absorbable — but it
   must be designed in now, not retrofitted.
3. **Message pump.** CEF wants `CefDoMessageLoopWork` on a timer, which conflicts with
   Qt/DCC hosts owning the loop. This is the `skip_message_pump` /
   `process_ipc_only` split, which already exists for wry — confirming that split is the
   right abstraction. CEF additionally needs `on_schedule_message_pump_work(delay)` on
   the sink.

Net: `BackendEventSink` ships with four methods today (§2.2) and gains `on_created` and
`on_schedule_message_pump_work` when the CEF backend lands. Both have wry-compatible
default implementations, so the wry path is unaffected.

### 2.7 What is explicitly NOT breaking

| Surface | Guarantee |
| --- | --- |
| `auroraview.BackendType` | Same enum, same four members, still exported from `auroraview` and `auroraview.core`. New members (`CEF`, `CHROMIUM`) are **added**, never removed. |
| `get_backend_type()` | Unchanged default (`WRY`). |
| `AURORAVIEW_BACKEND` | Same name, same accepted strings. New strings `cef` / `chromium` accepted; unknown strings still raise `ValueError` as today. |
| `create_webview(...)` / `WebView(...)` kwargs | No new required arguments. `parent` accepts the same int handle as today. |
| `eval_js` / `load_url` / `load_html` / `emit` | Identical signatures and semantics. |
| `window.auroraview.*` JS protocol | Unchanged. Backend choice is invisible to page code. |
| Wheel contents | A default build ships exactly one backend (wry), so the wheel does not grow. |

The only behavioural change: requesting a backend that is not compiled in currently
returns `Err(Internal("not yet implemented"))` from `BackendFactory` but silently works
through `NativeBackend`. After the seam lands it fails consistently in both paths, with
an error listing `BackendFactory::registered()`. This is a bug fix, not a break.

### 2.8 Sequencing

| Phase | Deliverable | Exit test |
| --- | --- | --- |
| P0 | Extract `crates/auroraview-wry`; introduce `NativeHandle`, `BackendEventSink`, `BackendCapabilities`. `NativeBackend` keeps its behaviour but stops naming `WryWebView` / `tao::Window` / `tao::EventLoop` in the trait. | `cargo build` + full wheel build produce no diff in behaviour; `AURORAVIEW_BACKEND` still honoured. |
| P1 | `BackendFactory` becomes a registry; `backend-wry` feature lands in `default`; `wry`/`tao` become optional in the root crate. | `cargo build --no-default-features --features backend-wry` works; building with no backend fails with a clear error. |
| P2 | `crates/auroraview-cef` placeholder implementing `Backend` + `BackendWebView` with `unimplemented!()` bodies behind `backend-cef`. | CI compiles the crate and asserts `capabilities()` reports `needs_subprocess_helper: true`. |
| P3 | Real CEF implementation. | Gallery app runs on `AURORAVIEW_BACKEND=cef` on Windows. |

P0/P1 are pure refactors with no user-visible change and can land in 0.6.x.
P2/P3 are additive and can land in 0.7.

---

## 3. Design — Part 2: host adapter package contract

### 3.1 The split rule

One sentence, testable in review:

> **Anything that imports a host module (`maya`, `unreal`, `hou`, `nuke`, `bpy`, `pymxs`,
> `UnityEditor`) leaves core. Everything host-agnostic, and everything shared by two or
> more dispatchers, stays.**

This rule is not stylistic — it is what preserves the project's "no mandatory third-party
Python dependency, one `.pyd`" guarantee, which is the reason DCC hosts accept AuroraView
at all.

### 3.2 What moves, what stays

| Item | Today | Decision | Why |
| --- | --- | --- | --- |
| `python/auroraview/utils/thread_dispatcher/base.py` | core | **Stays** | `ThreadDispatcherBackend` ABC is the contract itself. |
| `.../backends/fallback.py`, `qt.py` | core | **Stays** | Host-agnostic. Qt is not one DCC. |
| `.../backends/{maya,blender,houdini,max,nuke,unreal}.py` | core | **Moves out** to the matching package | Each imports a host module. |
| `.../registry.py` | core | **Stays**, and becomes the discovery point | Reads the `auroraview.hosts` entry points. |
| `.../wrapper.py` | core | **Stays** | Host-agnostic. |
| `crates/auroraview-ue` (568 L: `GameThreadId`, `UeGameThreadExecutor`) | monorepo | **Moves out** to `auroraview-unreal` | Pure threading/GC helpers; **depends on no wry/tao**, so it is already portable. |
| `python/auroraview/integration/qt/` + `QtWebView` | core | **Stays** | Qt crosses Maya/Houdini/Nuke/3ds Max. |
| `AuroraView` (HWND path) | core | **Stays** | Generic, host-agnostic embedding path. |
| `detect_host_dcc()` (PR #460) | core, PR #460 | **Stays** in core, but delegates | Env-var probe (`MAYA_LOCATION`, `HFS`, `NUKE_PATH`, `BLENDER_SYSTEM_SCRIPTS`, `3DSMAX_LOCATION`, `UE_ROOT`) becomes the fallback when no entry point claims the process. |
| `docs/dcc/*.md` | core | **Move** the host-specific pages out with their packages; keep `docs/dcc/index.md` as the pointer | Docs follow code. |

Note the asymmetry this reveals: **unreal is the only host with crate-level code today**
(`crates/auroraview-ue`), yet it is also the host whose Rust code is *most* separable —
it has no wry dependency. Maya, by contrast, has only a 45-line Python dispatcher. So the
"migration" is uneven by nature: unreal moves a Rust crate plus a dispatcher, maya moves
one dispatcher and needs new parent-window/lifecycle code, unity moves nothing and needs
everything.

### 3.3 Package names and layout

Distribution name uses a hyphen, import name uses an underscore (PyPI normalises both):

| PyPI dist | Import | Repo (proposed) | Host module |
| --- | --- | --- | --- |
| `auroraview-unreal` | `auroraview_unreal` | `try-auroraview/auroraview-unreal` | `unreal` |
| `auroraview-maya` | `auroraview_maya` | `try-auroraview/auroraview-maya` | `maya`, `maya.utils` |
| `auroraview-unity` | `auroraview_unity` | `try-auroraview/auroraview-unity` | `UnityEditor` (C#) |

Canonical layout — the Rust `crates/` directory is present only where a host needs
native code, which is why it is optional in this template:

```
auroraview-<host>/
├── crates/auroraview-<host>/     # OPTIONAL. Native code (today: only unreal)
│   └── src/lib.rs
├── python/auroraview_<host>/
│   ├── __init__.py               # exports: adapter (see §3.4), and a host WebView subclass
│   ├── dispatcher.py             # ThreadDispatcherBackend impl
│   ├── host.py                   # parent window discovery
│   └── lifecycle.py              # host lifecycle hooks (PIE, scene open, shutdown)
├── skills/auroraview-<host>/     # SKILL.md + tools.yaml + scripts/
├── tests/
│   ├── unit/
│   └── integration/              # requires a live host; skipped in normal CI
└── pyproject.toml
```

### 3.4 The contract itself

A host package registers one object through the **`auroraview.hosts`** entry point group.
Entry points are the mechanism that keeps core host-free: core never imports a host, it
asks Python's metadata who is installed.

```toml
# auroraview-unreal/pyproject.toml
[project.entry-points."auroraview.hosts"]
unreal = "auroraview_unreal:adapter"
```

```python
class HostAdapter:
    """Contract every auroraview-<host> package implements."""

    name: str                       # "unreal" | "maya" | "unity" — matches BackendType-agnostic id

    def detect(self) -> bool:
        """Is this host actually running in this process? Cheap, no side effects."""

    def parent_handle(self) -> int | None:
        """Native parent window handle, or None to own a top-level window.

        unreal : Slate main frame / PIE window
        maya   : int(MQtUtil.mainWindow()) -> HWND
        unity  : EditorWindow native handle via the C# bridge
        """

    def dispatcher(self) -> ThreadDispatcherBackend:
        """Main/game-thread dispatch. Must satisfy
        auroraview.utils.thread_dispatcher.base.ThreadDispatcherBackend."""

    def lifecycle_hooks(self) -> dict[str, Callable[[], None]]:
        """OPTIONAL. Called by core at: 'startup', 'before_show', 'shutdown'.
        unreal: PIE start/stop, GC protection of the WebView.
        maya  : scene-open / new-scene teardown.
        """
```

Core consumes it in `thread_dispatcher/registry.py`:

```python
def resolve() -> ThreadDispatcherBackend:
    for ep in entry_points(group="auroraview.hosts"):
        adapter = ep.load()
        if adapter.detect():
            return adapter.dispatcher()
    return FallbackDispatcherBackend()   # today's behaviour when nothing matches
```

`detect_host_dcc()` from PR #460 is kept as the fallback for hosts with no installed
package, so existing behaviour survives the transition.

### 3.5 Per-host notes

**Unreal** — smallest delta, highest value. The dispatcher already exists
(`unreal.register_slate_post_tick_callback`, 30 s sync timeout, `is_game_thread()`
vs `is_in_game_thread()` compatibility shim). `crates/auroraview-ue` already provides
`GameThreadId` / `UeGameThreadExecutor` with no wry dependency. Remaining work is
`host.py` (Slate parent handle) and `lifecycle.py` (PIE begin/end, GC pinning).

**Maya** — Qt-first. Dispatcher exists (`maya.utils.executeDeferred` /
`executeInMainThreadWithResult`). Parent handle comes from
`maya.OpenMayaUI.MQtUtil.mainWindow()`. The real work is `lifecycle.py`: a Maya panel must
survive scene open/close and workspace switching, which is where most Maya UI bugs live.

**Unity — a correction to the plan.** Unity ships no Python interpreter, so
`auroraview-unity` **cannot be an ordinary pip package**. It has to be a UPM package
(`Packages/...` with `package.json`) exposing a C# API, with the Python side, if any,
arriving through the Unity Python integration rather than the reverse. Its dispatcher is
`EditorApplication.delayCall`, not a thread-dispatcher backend.

This is a different build, test, and release pipeline from the other two — which is an
independent argument for the proposed ordering: **ship the contract for all three, but
implement unreal and maya first, and treat unity as a separate pipeline decision.**

### 3.6 Version coupling

- **Python:** `auroraview-unreal` declares `auroraview>=0.6,<0.7`. Upper bound is
  mandatory — the contract is not yet stable. Publish a compatibility table in each
  adapter README mapping adapter version to core version range.
- **Rust:** in-monorepo, `crates/auroraview-ue` uses `version.workspace = true`. Once split,
  it pins a published `auroraview-core`. During the transition, a path dependency plus a
  CI job that builds each adapter repo against core `main` is the pragmatic choice;
  publishing `auroraview-core` to crates.io is the end state.
- **Contract surface is deliberately narrow.** Adapters may depend on exactly two things:
  (a) the Python `auroraview` public API, and (b) the `auroraview.hosts` entry point
  protocol. Everything else — including `crates/auroraview-core` internals — is private, so
  core can refactor freely between adapter releases.
- **Contract test:** ship `auroraview-host-contract`, a pytest plugin each adapter repo runs
  in CI. It asserts the four entry points exist, `dispatcher()` satisfies
  `ThreadDispatcherBackend`, `detect()` is side-effect free, and `parent_handle()` returns
  `None` or a positive int. This is what makes "implements the contract" a CI fact rather
  than a README claim.

---

## 4. Design — Part 3: agent-first gap list

### 4.1 What already exists

| Asset | Current capability |
| --- | --- |
| `crates/auroraview-ai-agent` | `ActionRegistry` with 6 built-in browser actions (Navigate, Search, Click, Type, Screenshot, Scroll); `protocol/{a2ui,agui}.rs`; providers; session; ui |
| `crates/auroraview-mcp` | MCP server, `cdp/`, `registry.rs`, and `CdpAuroraViewAdapter` implementing `dcc_mcp_protocols::adapters::DccAdapter` over CDP |
| `packages/auroraview-ai` | `DCCTool` / `DCCToolCategory` decorators, agent config, protocol, tools |
| `packages/auroraview-sdk` | TS bridge (`core/{bridge,events,types}.ts`), `inject/`, React/Vue adapters, `AgentSidebar.tsx`, extensions |
| `crates/auroraview-testing` | CDP-based testing, `a11y/`, `snapshot.rs`, `inspector.rs`, Python bindings |
| `crates/auroraview-cli skills` | `auroraview skills` subcommand; **one** skill shipped: `qt-to-auroraview-migration` |
| PR #460 | Python `auroraview.dcc_mcp` (`AuroraViewAdapter`, `detect_host_dcc`, `AuroraViewQtHost`, `start_server`) plus the `auroraview-webview` skill package exposing `eval_js` / `screenshot` / `load_url` / `load_html` |

The gap is not capability, it is **coherence and discoverability**. Ranked:

### 4.2 G1 — Two competing MCP adapter contracts (highest priority)

There are already two implementations of "AuroraView as a DCC-MCP adapter":

- Rust `CdpAuroraViewAdapter` in `crates/auroraview-mcp/src/adapter/mod.rs`, implementing
  `DccAdapter`.
- Python `AuroraViewAdapter` in PR #460, subclassing `dcc_mcp_core.WebViewAdapter`.

PR #460's own rationale states it chose `WebViewAdapter` "instead of the Rust `DccAdapter`
trait, which the CLI does not consume." Both are correct about their own layer, and an
agent sees two different tool sets for one product.

**Resolution:** one contract per consumer. `WebViewAdapter` (PR #460) is the
control-plane surface the CLI consumes and becomes the canonical agent-facing adapter.
`CdpAuroraViewAdapter` is repositioned as the in-process CDP transport that backs it, not
as a parallel adapter. Both must expose the **same tool names** for the same operations,
enforced by a shared fixture.

### 4.3 G2 — Agents write code where they should emit data

Today an agent building AuroraView UI writes HTML/CSS/JS by hand. That is slow, hard to
validate, and produces inconsistent UI. The highest-leverage agent-first change is a
**declarative UI schema** — a JSON component spec the agent emits, which a renderer turns
into DOM. Agents are reliable at structured output and unreliable at long code generation.

### 4.4 G3 — No typed tool discovery from Python

`@webview.command` / `bind_call` bindings have no introspectable schema. An agent must read
source to learn what a panel exposes. Add `webview.tool_schema() -> dict` returning JSON
Schema, derived from the existing decorator metadata — the same metadata RFC 0018 already
extends for `cli=True`.

### 4.5 G4 — No headless validation loop

An agent cannot check whether the UI it just produced is correct without opening a window.
`crates/auroraview-testing` (CDP + snapshot) and `python/auroraview/testing/inspector.py`
already provide the primitives. Close the loop with a supported
`auroraview.testing.render_offscreen()` + DOM assertion path, so an agent can iterate
without a display.

### 4.6 G5 — Screenshot round-trip is CSP-fragile

`window.auroraview.screenshot` depends on html2canvas and, as PR #460 notes, breaks under a
strict CSP. Vision feedback is the main way an agent verifies UI, so this needs a native
path — `Page.captureScreenshot` through the CDP client that `crates/auroraview-mcp/src/cdp`
already owns, exposed as a first-class tool.

### 4.7 G6 — One skill shipped, no authoring kit

`crates/auroraview-cli/skills/` contains exactly one skill. If "agent-first" means agents
build skills for AuroraView, they need a scaffold and a validator:
`auroraview skills new <name>` generating the `SKILL.md` + `tools.yaml` + `scripts/` shape
PR #460 established, plus `auroraview skills validate`. The `auroraview-webview` skill in
PR #460 becomes the worked example.

### 4.8 G7 — Agents poll where they should subscribe

Events reach agents only by polling. An event feed (SSE or WS) over the existing
`window.auroraview.trigger()` stream, plus a subscribe tool, would let an agent react to UI
state instead of sampling it.

---

## 5. Alignment with in-flight work

| Work | Relationship |
| --- | --- |
| PR #460 (`agent/agent/177b6909646e`) | Supplies the canonical Python adapter and the `auroraview-webview` skill. §3.4 keeps `detect_host_dcc()` as the fallback and reuses the skill layout as the template for G6. §4.2 resolves its contract conflict with the Rust `CdpAuroraViewAdapter`. **PR #460 should merge first**; this RFC references it rather than redefining it. |
| RFC 0007 | Splits WebView/Browser into feature crates. This RFC is orthogonal — 0007 composes *features above* the WebView, this one abstracts *the renderer below* it. |
| RFC 0011 (unified IPC) | `BackendEventSink` (§2.2) is the backend-facing edge of the same IPC contract; it must stay message-compatible. |
| RFC 0018 (packed CLI mode) | Its `@webview.command(..., cli=True)` metadata is the same metadata G3 wants as JSON Schema. One extension serves both. |
| Sibling: `--parent-hwnd` / `AURORAVIEW_PARENT` (PIP-3214) | Supplies the parent-handle plumbing that `BackendInit.parent` (§2.2) consumes. `NativeHandle` is the type both should agree on. |

---

## 6. Open questions

1. Does `BackendWebView` need to be object-safe *and* `Send`? `WryWebView` is `!Send` on
   Windows. Today `NativeBackend` works around this with `Arc<Mutex<WryWebView>>` plus
   UI-thread marshalling. The RFC proposes keeping that pattern inside each backend
   implementation, with `Send` required only on the trait — needs a spike to confirm for CEF.
2. Should `auroraview-core` be published to crates.io before or after the adapter split?
3. For Unity: is a UPM-only package acceptable, or is a pip-installable Python bridge
   required for parity?

---

## 7. Success criteria

- [ ] `cargo build --no-default-features --features backend-cef` compiles with no wry in the tree.
- [ ] `AURORAVIEW_BACKEND=<unknown>` fails with an error listing the compiled-in backends,
      on both the Rust and Python paths.
- [ ] No symbol from `wry` or `tao` appears in any `pub` signature in `src/webview/backend/`.
- [ ] A host package can add main-thread dispatch and parent-window discovery with zero
      changes to `auroraview` core, proven by a fixture adapter installed via
      `auroraview.hosts`.
- [ ] The `auroraview-host-contract` pytest plugin passes for unreal and maya.
- [ ] `auroraview-webview` (PR #460) and `CdpAuroraViewAdapter` expose identical tool names
      for identical operations.
