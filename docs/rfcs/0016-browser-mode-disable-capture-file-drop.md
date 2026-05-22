# RFC 0016: Disable `capture_file_drop` in Browser Mode

- Number: 0016
- Title: Multi-tab `Browser` mode hard-disables `capture_file_drop` (never registers `with_drag_drop_handler`)
- Status: Draft
- Created: 2026-05-21
- Authors: AuroraView Core Team
- Split from: RFC 0013 §4.3.4 (D5 / D17 / D18 revisions)
- Prerequisites: **RFC 0015** (provides `attach_drag_drop_handler` helper)
- Affected files:
  - `src/webview/tab_manager.rs::TabManager` (paths 3 / 4 — business tab + controller)
  - `crates/auroraview-browser/src/tab/manager.rs` (path 9 — business tab)
  - `crates/auroraview-browser/src/browser.rs` (path 10 — controller)
  - `crates/auroraview-browser/src/config.rs::BrowserConfig` (**not** modified)
  - `crates/auroraview-cli/src/packed/webview/mod.rs` (Browser-mode entry in packed)

---

## 1. Summary

In multi-tab `Browser` mode (the new path `auroraview-browser::Browser` and the legacy path `webview/tab_manager::TabManager`), **no webview ever calls `with_drag_drop_handler`** — business tabs and the controller are treated identically.

Implementation: `BrowserConfig` / `TabManagerConfig` do **not** add a `capture_file_drop` field; `attach_drag_drop_handler` is invoked with **a literal `capture=false` constant**. There is no "runtime warn + zero out" logic, avoiding the design contradiction in RFC 0013 v14 between D5 (do not add the field) and D17 (read the field and zero it out).

---

## 2. Motivation

### 2.1 Why "business tabs attached, controller not attached" is unworkable

An early proposal envisioned "controller never attached, business tabs share `BrowserConfig.capture_file_drop`", which seems self-consistent — but **directly conflicts with the wry/WebView2 `IDropTarget` behavior model**:

1. **Business tabs and the controller are two independent WebView2 instances**, layered at the window level (the controller renders the tab bar / address bar / menu strip — the UI shell — while business tabs occupy the "content area" inside the shell).
2. WebView2's `IDropTarget` takeover is **decided per WebView**: controller not attached → controller area falls back to browser-native HTML5 OS drop; business tab attached → tab area swallows the OS drop and converts it to IPC. **Two semantics coexist at the pixel level.**
3. When a user drags a file across the controller / tab boundary (the most common scenario: dragging a file from the desktop, mouse first crossing the tab bar, then landing on the tab content area), which WebView the mouse hit-tests onto depends entirely on pixel-level layout:
   - When the mouse is over the tab bar: triggers the controller's HTML5 `dragover` (used for tab-rearrange gesture detection);
   - When the mouse moves into the tab content area: **HTML5 events suddenly stop firing and the business tab's IPC `file_drop_hover` fires instead**;
   - The user releases at an edge: possibly neither side fires, or the controller HTML5 `drop` fires first followed by the business tab IPC `file_drop` — **the state machine cannot converge**.
4. The front-end **cannot** write coherent hover feedback, because the event stream is two independent webviews bubbling two incompatible protocols.
5. Even if the business page only cares about "get the path on drop", the OS layer has already shown the drag image inside the controller's HTML5 `dragover` feedback during hover (mouse still on the controller), which is a starkly different UX from the single-webview case.

> **This is not a boundary bug solvable at the RFC implementation layer**: it is a fundamental constraint of wry/WebView2's "stacked multi-webview" model. To make "business tab gets IPC + controller keeps HTML5" actually work, wry would need to expose a "parent-child webview drag-drop protocol merge" API, which does not exist.

### 2.2 What Users Should Use Instead

Pages that need "drop a file, get the absolute path" have two routes:

- **Option A (HTML5)**: use browser-native HTML5 drag-drop on the web side (`dragover` + `drop` + `DataTransfer`). Note `DataTransfer.files` does not expose absolute paths on the web platform — only filenames + byte streams. This is a web-security constraint.
- **Option B (recommended)**: promote the page that needs "absolute-path IPC" to a top-level `AuroraView` instance (independent window, single webview), and set `capture_file_drop=True` on that instance.

---

## 3. Design

### 3.1 `BrowserConfig` / `TabManagerConfig` Add No Field

**`crates/auroraview-browser/src/config.rs::BrowserConfig`**: do not add `capture_file_drop`.

**`crates/auroraview-browser/src/config.rs::BrowserConfigBuilder`**: do not add a `capture_file_drop` chain method.

**`src/webview/tab_manager.rs::TabManagerConfig`**: do not add `capture_file_drop`.

**Reason**: an empty field would only make users believe "the config took effect", which is worse than "field does not exist → compile-time rejection". The current 0.x stage relies on Rust's type system to reject this misuse; once upstream wry fixes the issue, an independent RFC can introduce the field.

### 3.2 4 Builder Call Sites Pass `capture=false` Literal

| # | File | Change |
|---|---|---|
| 3 | `src/webview/tab_manager.rs::create_tab_webview` (business tab) | `builder = attach_drag_drop_handler(builder, false, &ipc_handler);` literal false |
| 4 | `src/webview/tab_manager.rs:984` (controller) | Same, literal false |
| 9 | `crates/auroraview-browser/src/tab/manager.rs:122` (business tab) | Same |
| 10 | `crates/auroraview-browser/src/browser.rs:545` (controller) | Same |

Each builder site gets a comment:

```rust
// Browser mode (controller and all business tabs) never attaches
// with_drag_drop_handler. In stacked multi-webview scenarios, attaching
// causes the OS drag-drop state machine to fail to converge across
// boundaries (see RFC 0016 §2.1). Pages that need "drop a file, get the
// absolute path" should be promoted to a top-level AuroraView instance
// with capture_file_drop=True.
builder = attach_drag_drop_handler(builder, false, &ipc_handler);
```

### 3.3 `Browser::new` / `TabManager::new` Do Not Read cfg, No Warn Logic

Since `BrowserConfig` does not contain `capture_file_drop`, `Browser::new` / `TabManager::new` need **no** check, zeroing, or warn at the entry.

> **Difference from RFC 0013 v14 D17**: v14 D17 envisioned `Browser::new(cfg) → mut effective_cfg = cfg; if effective_cfg.tab_webview_config.capture_file_drop { warn!; effective_cfg.tab_webview_config.capture_file_drop = false; }`, but v14 D5 simultaneously claimed `BrowserConfig` does not add that field — D17's code referenced a field D5 forbade. This RFC drops the runtime-check path of D17 and returns to "D5 as the single source of truth".

### 3.4 Packed Runtime Needs No Mode Branching

Since §3.3 has removed the "runtime warn", the `PackedRuntimeMode::TopLevelAuroraView` / `Browser` enum and `resolve_packed_capture_file_drop_with_mode` function introduced by v14 §4.2.4.3 D18 are **also removed**.

`AURORAVIEW_CAPTURE_FILE_DROP` env var is **no longer mode-branched** at the packed runtime parsing layer. In Browser mode, packed runtime never reads `capture_file_drop` (all 4 builder call sites pass a literal `false`); setting the env var equals "runtime override on a never-read field" — no contradictory log can be produced because there simply is no second warn.

> If an end user sets `AURORAVIEW_CAPTURE_FILE_DROP=1` in Browser mode hoping to enable the IPC proxy, **it indeed does not take effect** and **there is indeed no notice**. This is a trade-off: providing an env-var notice in Browser mode would require returning to v14 D18's mode-enum path, comparable in engineering cost to RFC 0015 §3.6's child-window doc caveat. This RFC chooses "doc + active user grep", consistent with the RFC 0015 §5.1 troubleshooting section.

### 3.5 Relationship With RFC 0015's IPC Scope-Limitation Paragraph

RFC 0015 §5's final paragraph already states:

> Browser-internal business tabs and the controller never attach the handler (see RFC 0016);
> Multiple independent `AuroraView` instances within the same process dispatch `file_drop*` independently along each instance's IPC path.

This RFC and RFC 0015 §5 echo each other; front-end subscribers should assume "the event comes from the current webview itself" rather than expect cross-webview bubbling.

---

## 4. Test Plan

### 4.1 Rust Tests

`crates/auroraview-browser/tests/browser_drag_drop_isolation_tests.rs` adds:

- **`browser_never_attaches_drag_drop_handler`**: enforced by §5 CI grep ("every `attach_drag_drop_handler` call in `browser.rs` / `tab/manager.rs` has a literal `false` as its second argument").
- **`browser_config_does_not_expose_capture_file_drop`**: compile-time assertion that `BrowserConfig` does not contain `capture_file_drop`:

  ```rust
  // Reverse-prove the field's absence by attempting to build a struct
  // literal with capture_file_drop (compile failure proves absence,
  // matching RFC 0016 §3.1).
  // Guard with #[cfg(test_compile_fail)] or the trybuild crate.
  ```

  > Simplified version: enforce by §5 CI grep ("`config.rs`'s `BrowserConfig` definition does not contain `capture_file_drop`").

`src/webview/tab_manager.rs` gets the mirrored tests.

### 4.2 Manual Smoke Matrix

| Mode | `capture=false` (default) | `capture=true` (any attempted setting) |
|---|---|---|
| Multi-tab business tab | HTML5 `drop` works / IPC does not fire | **HTML5 `drop` still works / IPC still does not fire** (`BrowserConfig` rejects the field at compile time) |
| Multi-tab controller | HTML5 `drop` works (always) | Same |
| Packed Browser mode + `AURORAVIEW_CAPTURE_FILE_DROP=1` | IPC does not fire (env var no-op in Browser mode) | Same |

### 4.3 Documentation Verification

`docs/zh/guide/file-drop.md` and `docs/zh/guide/multi-tab.md` (if present) must clarify in the multi-tab section:

> `capture_file_drop` is unavailable in Browser mode. To get an absolute path of a dropped file via IPC, use a top-level `AuroraView` instance (independent window, single webview).

---

## 5. CI Regression Grep

`scripts/ci/check_browser_no_drag_drop_capture.py` (or `just check-browser-no-capture`):

```bash
# Forbid capture_file_drop fields in BrowserConfig / TabManagerConfig
if rg "capture_file_drop" crates/auroraview-browser/src/config.rs ; then
    echo "ERROR: BrowserConfig must not expose capture_file_drop (RFC 0016 §3.1)"
    exit 1
fi
if rg "capture_file_drop" src/webview/tab_manager.rs ; then
    echo "ERROR: TabManagerConfig must not expose capture_file_drop (RFC 0016 §3.1)"
    exit 1
fi

# Forbid attach_drag_drop_handler's 2nd arg in Browser/TabManager paths
# from being any literal other than `false` (a dynamic value would imply a
# config entry forbidden by RFC 0016).
if rg "attach_drag_drop_handler\([^,]+,\s*[^f]" crates/auroraview-browser/src/ src/webview/tab_manager.rs ; then
    echo "ERROR: Browser/TabManager paths must always pass capture=false to attach_drag_drop_handler (RFC 0016 §3.2)"
    exit 1
fi

echo "OK: Browser mode never attaches drag-drop handler."
```

---

## 6. Implementation Steps

1. **Step 1 — Browser controller**: at `crates/auroraview-browser/src/browser.rs:545`, add `attach_drag_drop_handler(builder, false, &ipc_handler)` + comment.
2. **Step 2 — Browser business tab**: same at `crates/auroraview-browser/src/tab/manager.rs:122`.
3. **Step 3 — Legacy tab_manager path**: mirror at the two sites in `src/webview/tab_manager.rs` (business tab :469 + controller :984).
4. **Step 4 — CI grep**: add `scripts/ci/check_browser_no_drag_drop_capture.py`, integrated into the `vx just test` flow.
5. **Step 5 — Docs**: clarify in `docs/zh/guide/file-drop.md` / multi-tab section the migration path "Browser mode does not support `capture_file_drop`; please use a top-level AuroraView instance"; CHANGELOG calls this out under the multi-tab section.

Each step is verified with `vx just test`.

---

## 7. Compatibility

- **Fully API non-breaking**: `BrowserConfig` / `TabManagerConfig` / `BrowserConfigBuilder` public surfaces are unchanged.
- **Runtime behavior**:
  - Previously, no webview in Browser mode actually attached `with_drag_drop_handler` (RFC 0013 v14 §2.1 path table confirmed); behavior remains unchanged after this RFC (still not attached).
  - The only change is that the 4 builder sites now uniformly call `attach_drag_drop_handler(builder, false, ...)`, aligning code style with the other 5 builder sites.
- **Zero migration cost.**

---

## 8. Risks

| Risk | Assessment | Mitigation |
|---|---|---|
| Users want IPC drag-drop in Browser mode | Low | Docs clearly point to "top-level `AuroraView` instance"; CI grep + compile-time absence are a double defense |
| Future wry fix for multi-webview drag-drop protocol | Low | An independent RFC can re-introduce `BrowserConfig.capture_file_drop` at that point; this RFC's "4 literal-false sites" reverses to "read and apply config" |
| No notice for env var in packed Browser mode | Low | Centrally covered by RFC 0015 §5.1 troubleshooting docs |

---

## 9. Follow-up RFCs

- After upstream wry exposes a "parent-child webview drag-drop protocol merge" or "window-wide unified `IDropTarget`" API, an independent RFC can re-introduce `BrowserConfig.capture_file_drop` + per-tab override.
