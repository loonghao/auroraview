# AuroraView adapter contract and pluggable backend abstraction

**Status**: prototype, in-branch. S1 of the per-host package split (PIP-3195).
**Scope**: contracts + a runnable prototype. No repository is created, no code is
migrated (that is S2 and needs owner authorisation).

## 1. Why this exists

The per-host split was approved on the grounds of **organisational and evolution
boundaries**, not dependency weight. That makes the binding constraint the
*dependency direction*:

```text
auroraview (core)  <-- imported by -->  auroraview_maya / _unreal / _unity / ...
```

Core must never import a host module. Today that invariant is violated in one
place: `DccType` (`crates/auroraview-dcc/src/config.rs`) is a closed enum that
names Maya, Houdini, Nuke, Blender, Max and Unreal, and `DccType::detect()`
sniffs their environment variables. Adding a host means editing core.

This document defines the two contracts that remove that need:

| Contract | Question it answers |
|---|---|
| **Host adapter** | how a host registers itself, is discovered, and embeds a surface |
| **Render backend** | how the web engine is selected, replaced, and probed |

## 2. What already exists (measured, not assumed)

| Thing | Where | State |
|---|---|---|
| Thread dispatch plugin registry | `python/auroraview/utils/thread_dispatcher/registry.py` | **real and working**: priority, lazy `"module:ClassName"` import, `AURORAVIEW_DISPATCHER` override, `register_dispatcher_backend()` |
| `WebViewBackend` trait | `crates/auroraview-core/src/backend/traits.rs` | exists, 289 lines, well designed |
| `BackendFactory` | `crates/auroraview-core/src/backend/factory.rs` | **closed `match`** over `BackendType`; no registration point |
| Host enumeration | `crates/auroraview-dcc/src/config.rs` (`DccType`) | closed enum; env-var sniffing |

Two measured facts drove the design:

1. **`BackendType::WebView2` on Windows returns the same `WryBackend` as
   `BackendType::Wry`.** The two families are not actually distinguishable
   today, so "system native vs Chromium" is currently cosmetic.
2. **`WKWebView` / `WebKitGTK` return `Err(WebViewError::Internal("not yet
   implemented"))`** — an opaque internal error, not a structured "unsupported,
   here is how to enable it".

The trait layer was already right. What was missing was the registration point
and the capability-probing convention. That is what S1 adds.

## 3. Dependency direction

```text
   auroraview-contract          (traits only, zero dependencies)
        ^                ^
        |                |
   auroraview-core     auroraview-maya / -unreal / -unity / ...
        ^                ^
        |                |
        +----------------+
     adapters depend on core and on the contract;
     core depends on the contract and on NO adapter
```

`crates/auroraview-contract` has **an empty `[dependencies]` section on
purpose**. It is the firewall: because it is a leaf with no dependencies, an
adapter crate can depend on it without pulling in a WebView engine, and core can
accept adapters without naming them. A new host becomes a new crate that
registers itself.

## 4. Host adapter contract

Rust (`crates/auroraview-contract/src/host.rs`), Python
(`python/auroraview/adapter/base.py`).

```rust
pub trait HostAdapter: Send + Sync {
    // identity
    fn id(&self) -> &'static str;              // "maya", "unreal"
    fn display_name(&self) -> &'static str;
    fn ui_framework(&self) -> UiFramework;      // Qt | Slate | Win32 | Cocoa | Gtk | Wpf | Html | Unknown
    fn thread_model(&self) -> ThreadModel;      // HostUiThread | GameThread | StaApartment | Any
    fn embed_mode(&self) -> EmbedMode;          // NativeChild | OutOfProcess | Floating

    // discovery -- cheap, must not panic/raise, no module-scope host imports
    fn detect(&self) -> bool;
    fn version(&self) -> Option<String>;
    fn parent_handle(&self) -> Option<u64>;

    // the only required dispatch primitive
    fn run_deferred(&self, job: HostJob) -> HostResult<()>;

    // deliberately NOT required -- see 4.1
    fn try_run_sync(&self, job: HostJob) -> HostResult<()> { /* refuses */ }

    fn probe(&self, feature: Features) -> CapabilitySupport;
    fn info(&self) -> HostInfo;
}

pub struct HostRegistry { /* priority-ordered, lazy */ }
impl HostRegistry {
    pub fn register<A: HostAdapter + Default + 'static>(&mut self, priority: i32);
    pub fn detect_with(&self, env_override: Option<&str>) -> Option<Selection<Arc<dyn HostAdapter>>>;
    pub fn detect(&self) -> Option<Selection<Arc<dyn HostAdapter>>>;  // honours AURORAVIEW_HOST
    pub fn list(&self) -> Vec<(i32, String, bool)>;
}
```

Python:

```python
register_host_adapter(GodotHostAdapter, priority=140)                      # in-tree
register_host_adapter("auroraview_godot.adapter:GodotHostAdapter", 140)    # separate dist
```

The string form is resolved lazily, so `auroraview_godot` is imported only when
discovery actually probes it — the same mechanism the dispatcher already uses.

### 4.1 Key decision: blocking sync dispatch is NOT part of the contract

`try_run_sync` has a default implementation that **refuses**. Blocking the host
thread is the concrete failure mode behind two real constraints:

- **PowerPoint / Office (COM STA)**: blocking the apartment's message pump makes
  the host report "not responding".
- **Unreal (GameThread)**: blocking stalls the editor.

`ThreadModel::blocking_dispatch_is_safe()` encodes this, and the refusal message
differs accordingly: an adapter that simply has not implemented sync gets
"implement it or use `run_deferred`"; an STA/GameThread host gets "your thread
model forbids it". Only `run_deferred` is required, which is safe everywhere.

### 4.2 Key decision: reuse the dispatcher, do not fork it

The six built-in adapters are ~15 lines of metadata each and delegate
`run_deferred` to the matching dispatcher backend
(`python/auroraview/adapter/hosts.py`). The dispatcher keeps owning threading;
the adapter adds identity, discovery and embedding. **Zero dispatch logic is
duplicated**, and `DispatcherPriority` (MAYA 200 → FALLBACK 0) is mirrored as
`AdapterPriority` so both registries read identically.

## 5. Render backend contract

Rust (`crates/auroraview-contract/src/backend.rs`), Python
(`python/auroraview/adapter/backends.py`).

```rust
pub enum BackendFamily { Native, Chromium, Other }

pub trait RenderBackend: Send + Sync {
    fn id(&self) -> &'static str;              // "native", "chromium"
    fn family(&self) -> BackendFamily;
    fn available(&self) -> bool;               // cheap, non-panicking
    fn missing_requirement(&self) -> Option<String>;
    fn capabilities(&self) -> Features;
    fn probe(&self, feature: Features) -> CapabilitySupport;
    fn create_surface(&self, spec: &SurfaceSpec) -> BackendResult<Box<dyn RenderSurface>>;
}

pub fn default_backend_registry() -> BackendRegistry;  // native 100, chromium 50
```

Both required families are represented:

| Backend | Family | `available()` | Notes |
|---|---|---|---|
| `NativeWebviewBackend` | Native | true (links wry) | WebView2 / WKWebView / WebKitGTK; the path in use today |
| `ChromiumBackend` | Chromium | **false** | declared, not linked — the whole point of the capability contract |

`ChromiumBackend` being present-but-unavailable is intentional. It makes the
second path a first-class citizen of the contract instead of a TODO, and it is
exactly the situation the probing convention exists to describe.

### 5.1 Capability probing never fails

```rust
pub enum CapabilitySupport {
    Supported,
    Unsupported { reason: String, how_to_enable: String },
    Unknown { reason: String },
}

impl CapabilitySupport {
    pub fn require(self, subject: &str, feature: &str) -> Result<(), CapabilityError>;
}
```

Rules:

- **`probe()` returns, it does not throw.** A missing feature is data.
- **`Unsupported` always carries `how_to_enable`** — a build flag, an extra, an
  env var. Asserted by tests on both sides.
- **`Unknown` is first-class.** It is how an implementation says "runtime
  dependent, verify at runtime" instead of guessing. `TRANSPARENCY` on the
  native backend uses it.
- **`require()` is the single fallible conversion**, used only by callers that
  cannot degrade gracefully.
- **An unavailable backend answers everything as `Unsupported`** — not even
  `Unknown`. If it is not in the build it cannot answer at all.

## 6. Shared registry mechanics

Both registries use one idiom, in both languages:

- priority ordering, highest first
- **lazy construction** — a candidate is only built when probed or selected
- environment override (`AURORAVIEW_HOST` / `AURORAVIEW_BACKEND`)
- **a failed override degrades to priority order with a warning**, never an error
- **a candidate that raises while probed is skipped with a warning** — discovery
  is shared infrastructure, so one broken third-party adapter must not take down
  host detection for everyone

This is the same design as `thread_dispatcher.registry`, so there is one
extension idiom in AuroraView, not two.

## 7. Migration impact (what moves in S2)

| Crate / module | Change | Breaking? |
|---|---|---|
| `crates/auroraview-contract` | **new**, zero deps | no |
| `crates/auroraview-core` | depend on contract; `backend/factory.rs` delegates to `BackendRegistry` instead of its closed `match` | no (additive) |
| `crates/auroraview-core/src/backend/traits.rs` | `WebViewBackend` stays; becomes one impl of `RenderBackend` | no |
| `crates/auroraview-core/src/backend/factory.rs` | `BackendType` enum becomes a compatibility alias over registry lookups | **deprecate, keep** |
| `crates/auroraview-dcc/src/config.rs` | `DccType` becomes a deprecated shim; `detect()` delegates to `HostRegistry` | no (shim kept) |
| `crates/auroraview-dcc/src/webview.rs:95` | reads `config.dcc_type.name()`; switch to the detected `HostInfo` | internal |
| `python/auroraview/utils/thread_dispatcher/` | **unchanged** — adapters delegate to it | no |
| `python/auroraview/adapter/` | **new** | no |
| new: `auroraview-maya`, `-unreal`, `-unity`, … | one crate + one Python dist each; ~15 lines of metadata + a dispatcher backend | n/a |

`DccType` and `BackendType` are both kept as deprecated shims, so nothing that
compiles today breaks.

## 8. Deliberately out of scope

- **No repository creation, no org changes, no code migration** (S2; needs owner
  authorisation).
- **PIP-3214 (path A groundwork)** is untouched: `--parent-hwnd` answers "how
  does a child process embed into a host window", this answers "how is the
  backend chosen and how does an adapter depend on core". Complementary.
- **Surface creation is not rewired.** `BackendRegistry` selects; construction
  still goes through `auroraview.core`. Rewiring it is the next step once the
  contract is agreed.
- **No CI matrix changes yet** (S3).

## 9. Verification

```bash
cargo test  -p auroraview-contract                     # 46 passed
cargo clippy -p auroraview-contract -- -D warnings     # clean
cargo fmt -p auroraview-contract -- --check            # clean

pytest tests/python/unit/test_adapter_contract.py      # 51 passed
```

The native backend reports `available() == False` in a source checkout without a
built `auroraview._core` extension. That is the contract working, not a bug: an
unavailable backend reports a structured "unsupported, install a released
wheel" instead of failing at import.
