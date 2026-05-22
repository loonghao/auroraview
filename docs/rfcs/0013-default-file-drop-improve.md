# RFC 0013: File-drop Proxy Unification & `capture_file_drop` Switch (Split Index)

- Number: 0013
- Title: File-drop proxy unification & `capture_file_drop` switch — overview / split index
- Status: Superseded by RFCs 0014–0017
- Created: 2026-05-20
- Last revised: 2026-05-21
- Authors: AuroraView Core Team

---

## 0. Document Purpose

This document was originally a single large RFC (v1–v14, with 5 appendices and 26 numbered revisions). As review depth increased, three structural problems surfaced:

1. **Core abstractions misaligned with code reality** (e.g. `IpcHandler::handle_message` is actually `Result<_, String>`; `IpcHandler` does not own a webview eval channel), causing D14 / D16 to be unimplementable on the main path.
2. **Revisions contradicting each other** (D5 said "do not add `BrowserConfig.capture_file_drop`" while D17 wanted `Browser::new` to read that field at the entry point and zero it out).
3. **Severely imbalanced ROI for the diagnostic mechanism** (the entire D7→D16 sink trait extension existed solely to print one line of `console.info`).

Following the principle of "every change independently revertible, reviewer cognitive load reduced by an order of magnitude", the original proposal is split into **4 independent RFCs**, each with its own minimum landing scope, independently revertible PR, and independent test boundary. This document no longer carries design content; it serves only as an index and a decision log.

---

## 1. Post-Split RFC Index

| RFC | Title | Scope | Relationship to this RFC |
|---|---|---|---|
| **0014** | Centralize `wry` / `tao` into `[workspace.dependencies]` | `Cargo.toml` only; 5 crates switch to `{ workspace = true }` | Extracted from original §4.1.5 D9 revision |
| **0015** | `attach_drag_drop_handler` shared helper + 7 builder unification | `auroraview-core::builder` + 7 call sites + `WebViewConfig.capture_file_drop` field | Extracted from original §4.1 / §4.2.1 / §4.3.1 |
| **0016** | Disable `capture_file_drop` in Browser mode | `auroraview-browser` + `tab_manager` runtime warn path | Extracted from original §4.3.4 (D5/D17) |
| **0017** | Python `capture_file_drop` tri-state contract (`Optional[bool]`) | `python/auroraview/core/` + PyO3 binding + CI grep regression guard | Extracted from original §4.2.5 (D3) |

**Recommended landing order**: 0014 → 0015 → 0016 → 0017. 0014 is a hard prerequisite for the other three (eliminates wry version drift); 0015 provides the helper; 0016 builds on the 0015 helper to add the "never attach" branch for Browser mode; 0017 builds on the 0015 field to add Python passthrough. Any two of them can be merged or reverted independently.

---

## 2. Content Explicitly Removed From the Original Proposal

The items below still existed in v14 but are **not retained** after the split. Rationale follows. If a real need emerges in the future, each should be re-proposed as an independent RFC.

### 2.1 D7 / D16 Diagnostic Mechanism (Removed Entirely)

**Original design**: under the `capture_file_drop=true` path, on the first `Enter` / `Drop` hit, a `DragDropHandler::diagnostic_once: Arc<Once>` + `DragDropIpcSink::notify_diagnostic_once(&self, _script: &'static str)` protocol would have the sink push a diagnostic JS snippet into the webview via `evaluate_script`.

**Reasons for removal**:

1. **Unimplementable on the main path**: `src/ipc/handler.rs::IpcHandler` only holds `message_queue: Option<Arc<MessageQueue>>` — it does **not** hold a `WebViewHandle` and has **no** `eval(script)` API. `crates/auroraview-desktop/src/ipc/handler.rs::IpcRouter` only has `handlers: DashMap`, with no reverse channel into the webview. To make `notify_diagnostic_once` actually visible in DevTools, we would need a new `WebViewMessage::EvaluateScript` variant, refactor `IpcHandler` to accept an eval channel, and thread it through `event_loop` — engineering work entirely orthogonal to the "unify file-drop" theme.
2. **Tests passing ≠ feature working**: the v14 §7.1 `dragdrop_diagnostic_once_fires_only_on_first_event` test passes with a `DiagnosticCountingSink`, but in production the diagnostic JS never reaches the webview. This is textbook mock-driven false confidence.
3. **ROI mismatch**: the entire mechanism (Once guard + trait extension + changes across all three IPC entry points + test helpers) costs several times the engineering work of the RFC main line, only to print one extra line of `console.info` in DevTools.

**Replacement**: add a section in the user-facing front-end docs `docs/zh/guide/file-drop.md` (shipped with RFC 0015):

> If you set `capture_file_drop=True` but `auroraview.on('file_drop', ...)` does not receive events, check:
>
> 1. Whether the browser HTML5 `dragover` / `drop` listeners are still active (due to an upstream wry/WebView2 bug, the two are mutually exclusive once `capture_file_drop` is enabled — see RFC 0015 §2);
> 2. Whether the IPC channel is established (subscribe inside `auroraview.on('auroraviewready', ...)` to verify the bridge is ready).

Doc + active user grep cost ≈ 0; fully decoupled from the RFC main line.

### 2.2 `DragDropIpcSink::notify_diagnostic_once` Method (Removed Entirely)

Removed alongside §2.1. The `DragDropIpcSink` trait surface returns to a single method:

```rust
pub trait DragDropIpcSink: Send + Sync + 'static {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), DispatchError>;
}
```

Each of the three impls (`IpcHandler` / `IpcRouter` / packed `IpcSink`) is ≤ 5 lines — see RFC 0015 §3.1.

### 2.3 D14 `DispatchError` Three-Variant Semantics (Reverted to Single Variant)

**Original design**: `DispatchError` would have `Disconnected` / `Serialization(serde_json::Error)` / `Backend(Box<dyn Error>)` variants, classified precisely by underlying error type so contract tests could pattern-match on variants.

**Downgrade decision**:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DispatchError {
    /// Any sink-side underlying error. Currently `IpcHandler::handle_message`'s
    /// error type is `String`, which cannot be precisely classified as
    /// `Disconnected` / `Serialization`, so all errors funnel through this
    /// variant. Once IpcHandler's error type is enum-ified (a separate
    /// refactor across the entire IPC subsystem, decoupled from this RFC),
    /// new semantic variants like `Disconnected` / `Serialization` will be
    /// added here. `#[non_exhaustive]` ensures that adding variants is not
    /// a breaking change.
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync + 'static>),
}
```

**Reason**: the v14 §4.1.3 classification example referenced `IpcError::ChannelClosed | SendFailed | Serialization` — variants that do not exist. The current code is `pub fn handle_message(&self, message: IpcMessage) -> Result<serde_json::Value, String>`, where the error type is `String` with no structure. Forcing a three-variant enum to land would require enum-ifying `IpcHandler`'s error type first, which spans the entire IPC subsystem and is unrelated to this RFC's theme.

**Keep `#[non_exhaustive]`**: once the IpcError enum lands later, appending variants will not be breaking. See RFC 0015 §3.2.

### 2.4 D5 / D17 Reconciliation (Pick One; This RFC Picks D5 + 0016 In-Code Effective)

The original v14 simultaneously claimed:

- **D5** (§4.3.4.3 table): `BrowserConfig.capture_file_drop` is **not** added.
- **D17** (§4.3.4.4): `Browser::new(cfg)` checks `cfg.tab_webview_config.capture_file_drop == true` at the entry and zeroes it out.

The latter must read the field; the former forbids adding it. Self-contradiction. RFC 0016 takes the **D5 path**: `BrowserConfig` / `TabManagerConfig` do not add a `capture_file_drop` field; `Browser::new` / `TabManager::new` do **not** read cfg either — both entry points simply replace the `attach_drag_drop_handler` call with a non-attaching one (constant `capture=false`), with no "runtime check + zero out" logic.

If a user genuinely needs "use IPC drop in Browser mode", they must promote the relevant page to a top-level `AuroraView` instance (consistent with v14's "recommended path", just removing the redundant cfg → warn → zero chain).

See RFC 0016 §3.

### 2.5 D18 Packed Runtime Mode Branching (Simplified Together With §2.4)

Since §2.4 has removed the "runtime warn" (Browser mode does not read cfg at all, no field to warn about), the `PackedRuntimeMode::TopLevelAuroraView` / `Browser` enum and `resolve_packed_capture_file_drop_with_mode` function introduced by v14 §4.2.4.3 D18 are **also removed**.

`AURORAVIEW_CAPTURE_FILE_DROP` env var is no longer mode-branched at the packed runtime parsing layer; in Browser mode the packed runtime never reads `capture_file_drop` to begin with, so setting the env var equals "runtime override on a field that is never read" — no contradictory log can be produced.

See RFC 0016 §4.

---

## 3. Core Decisions Retained From the Original Proposal (Unchanged)

The designs below are **fully retained** in the post-split RFCs. Listed here for one-shot reviewer sign-off:

1. **Default `false` / zero exceptions**: all run modes (standalone / DCC / CLI / packed) default to `capture_file_drop = false`; no more "DCC defaults to `true`" exception. See RFC 0015 §1.
2. **`capture` is toggled by "whether to call `with_drag_drop_handler`"**, not by "attach but return `false`" — the latter is unworkable on Windows due to an upstream wry/WebView2 bug. See RFC 0015 §2.
3. **Helper takes `&Arc<S>` borrow form** (D15 revision); when `!capture`, `Arc::strong_count` truly stays unchanged and the contract is independently testable. See RFC 0015 §3.3.
4. **Helper signature carries a generic lifetime `'a`** (D1 revision), compatible with both `WebViewBuilder::new()` and `WebViewBuilder::new_with_web_context(&mut web_context)`. See RFC 0015 §3.3.
5. **CLI flags**: `auroraview run` and `auroraview pack` both use a paired explicit flag form (`--capture-file-drop` / `--no-capture-file-drop`) plus `resolve_capture_file_drop` to recover `Option<bool>` (D2 revision). See RFC 0015 §4.
6. **Packed app env var escape hatch** `AURORAVIEW_CAPTURE_FILE_DROP`; `parse_truthy` is case-insensitive and recognizes `1/true/on/yes/enabled` × `0/false/off/no/disabled`; invalid values warn instead of silently falling back (D4 revision). See RFC 0015 §4.3.
7. **Python tri-state contract** (`Optional[bool]`) + §7.5 CI grep regression guard. See RFC 0017.
8. **`#[serde(default)]` already guarantees manifest / overlay binary compatibility**; no overlay version bump needed. See RFC 0015 §4.4.
9. **Controller webview never attached** (§4.3.2) — naturally absorbed by 0016 as a special case of "the entire Browser path never attaches"; no separate code-comment block.
10. **Child window never attached** (§4.3.3) + `NewWindowConfig.new_window_mode` docstring caveat (D11 revision). See RFC 0015 §3.6.
11. **IPC event schema unchanged**: existing `file_drop_hover` / `file_drop` / `file_drop_cancelled` payload fields are preserved. See RFC 0015 §5.
12. **Scope limitation** (D13 revision): `file_drop*` events come only from the webview that actually attached the handler; they do not propagate across webviews. See RFC 0015 §5 final paragraph.

---

## 4. Compatibility Summary

- **DCC default behavior change** (v14 §6.1) preserved: DCC users go from "`file_drop` fires automatically" to "must explicitly pass `capture_file_drop=True`". Still treated as a 0.x minor breaking change, prominently called out in CHANGELOG.
- **manifest / overlay binary compatibility** preserved: `#[serde(default)]` ensures all 4 cross-combinations of new/old runtime × new/old overlay safely fall back to `false`; no overlay version bump needed.
- IPC schema, event names, and `DragDropHandler` internals are unchanged; existing front-end code subscribed to `file_drop` requires no modification.
- All existing CLI flags are retained; new flags default to off.

---

## 5. Historical Revision Log (Retained as Index)

The full set of v14 revisions (D1–D18) — source problem, v14 location, and post-split landing — is consolidated below. Each post-split RFC references rows of this table in its "revision linkage" section; this document is the single source of truth and is not maintained twice.

| ID | Source problem | v14 location | Post-split home |
|---|---|---|---|
| D1 | Helper lifetime parameter | §4.1.2 | RFC 0015 §3.3 |
| D2 | Pack CLI flag form | §4.2.4.2 | RFC 0015 §4.2 |
| D3 | `to_kwargs` comment anchor | §4.2.5.3 | RFC 0017 §3.3 |
| D4 | Env var value recognition | §4.2.4.3 | RFC 0015 §4.3 |
| D5 | Disable in Browser mode | §4.3.4 | RFC 0016 §3 |
| D6 | Trait `Result` return | §4.1.2 / §4.1.3 | RFC 0015 §3.1 (`DispatchError` reduced to single variant — see §2.3 above) |
| D7 | Diagnostic JS injection | §4.1.6 | **Removed** (§2.1 above) |
| D8 | Drop `?Sized` | §4.1.2 | RFC 0015 §3.3 |
| D9 | Workspace dep centralization | §4.1.5 | RFC 0014 |
| D11 | Child window dead code | §4.3.3 | RFC 0015 §3.6 |
| D12 | PyO3 + packed same-process assumption | §11 / §4.5.1 | RFC 0017 §4 |
| D13 | IPC scope limitation | §4.4 | RFC 0015 §5 |
| D14 | `DispatchError` semantic variants | §4.1.2 | **Reduced to single variant** (§2.3 above; revisit after IpcHandler enum-ification) |
| D15 | Helper `&Arc<S>` borrow | §4.1.2 | RFC 0015 §3.3 |
| D16 | Once-guarded diagnostic | §4.1.6 | **Removed** (§2.1 above) |
| D17 | Browser mutate input cfg | §4.3.4.4 | **Removed** (mutually exclusive with D5; §2.4 above) |
| D18 | Packed env var mode branching | §4.2.4.3 | **Removed** (simplified together with §2.4; §2.5 above) |

---

## 6. Open Questions and Follow-up RFCs

The split version has **no open questions** within its scope. Follow-ups that could land as independent RFCs:

- **`IpcHandler` error type enum-ification** (independent refactor across the entire IPC subsystem, decoupled from this theme). Once landed, `DispatchError::{Disconnected, Serialization, ...}` semantic variants can be added (`#[non_exhaustive]` guarantees backward compatibility).
- **Wire IPC channel into child windows** (after which the "never attach" constraint in RFC 0015 §3.6 can be relaxed).
- **Once upstream wry fixes hybrid mode**, add an event-passthrough switch on `DragDropHandler`.
- **Per-tab `capture_file_drop` override** (if a real need arises; currently explicitly disabled by RFC 0016).
- **`auroraview run` upgraded to a bidirectional override flag** (if a real need arises).

---

## 7. References

- Post-split RFCs: 0014 / 0015 / 0016 / 0017
- `crates/auroraview-core/src/builder/drag_drop.rs` — `DragDropHandler::into_handler` implementation
- `src/ipc/handler.rs::IpcHandler` — error type is `String` (drives the §2.3 decision)
- wry `WebViewBuilder::with_drag_drop_handler` docs: <https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_drag_drop_handler>
- Upstream bug trackers: [tauri#15138](https://github.com/tauri-apps/tauri/issues/15138), [wry#157](https://github.com/tauri-apps/wry/issues/157)
