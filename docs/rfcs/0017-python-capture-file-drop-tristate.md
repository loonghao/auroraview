# RFC 0017: Python `capture_file_drop` Tri-State Contract (`Optional[bool]`)

- Number: 0017
- Title: Preserve `Optional[bool]` tri-state for `capture_file_drop` through the Python passthrough chain + CI grep regression guard
- Status: Draft
- Created: 2026-05-21
- Authors: AuroraView Core Team
- Split from: RFC 0013 §4.2.5 / §4.5.1 key point 1 (D3 / D12 revisions)
- Prerequisites: **RFC 0015** (provides Rust `WebViewConfig.capture_file_drop: bool` field)
- Affected files:
  - `python/auroraview/core/config.py::ContentConfig`
  - `python/auroraview/core/config.py::WebViewConfig.from_kwargs`
  - `python/auroraview/core/config.py::WebViewConfig.to_kwargs`
  - `python/auroraview/core/factory.py`
  - `python/auroraview/core/mixins/factory.py`
  - `python/auroraview/core/mixins/content.py`
  - `python/auroraview/integration/qt/_core.py`
  - `python/auroraview/__main__.py` (CLI entry passthrough)
  - `src/bindings/desktop_runner.rs` (PyO3 binding-layer `unwrap_or(false)` fallback)
  - `scripts/ci/check_capture_file_drop_defaults.py` (new CI grep)
  - `tests/python/unit/test_file_drop_events.py`
  - `tests/python/integration/test_capture_file_drop_passthrough.py` (new)
  - `tests/python/unit/test_child_window_isolation.py` (new)

---

## 1. Summary

The Python side of `capture_file_drop` uses `Optional[bool]` tri-state:

| Value | Meaning |
|---|---|
| `None` (default) | Not specified; let the lower layer (Rust `WebViewConfig`) default kick in |
| `True` | Explicitly enable IPC proxy |
| `False` | Explicitly disable IPC proxy |

**No** intermediate layer in the passthrough chain may flatten `None` into `False` via `setdefault("capture_file_drop", False)` / `kwargs.get("capture_file_drop", False)` / `value or False` / etc. The tri-state must **pass through unchanged** to the Rust PyO3 binding layer, where `unwrap_or(false)` provides the final fallback.

---

## 2. Motivation

### 2.1 The Real Engineering Value of the Tri-State Contract

1. **Prevent silent override at intermediate layers**: any layer that flattens `None` loses the real intent. If the PyO3 entry point gains a new lower-layer source in the future (e.g. `[user_settings]` toml, remote config), `None` can naturally fall back to that lower layer; if intermediate layers had already flattened, the new source could never take effect.
2. **Cross-entry semantic alignment**: `Option<bool>` is already the unified language in manifest (RFC 0015 §4.1) and Pack flag (RFC 0015 §4.2); the Python kwarg uses the same tri-state.
3. **Test observability**: tri-state lets "user did not pass" and "user passed False" be independently asserted in integration tests.

### 2.2 Rust `unwrap_or(false)` Does Not Mask Lower Layers Under Current Architecture

The PyO3 entry point and the packed-app runtime **do not run in the same process**:

- The standalone exe produced by `auroraview pack` runs the Rust entry directly, **without** going through PyO3 binding;
- Under `PackMode::FullStack`, Python is a child process communicating with the packed runtime via IPC.

So the PyO3 binding's `unwrap_or(false)` fallback **does not** mask the packed overlay's value. The tri-state's value lies in preventing silent override and leaving room for future extensions, **not** in "overriding manifest", which is not a current scenario anyway.

---

## 3. Design

### 3.1 Field Home: `ContentConfig`

`capture_file_drop` lives in `ContentConfig`, alongside the existing `allow_file_protocol`.

**Reasons**:

1. **Same semantic family**: `allow_file_protocol` and `capture_file_drop` are both "the host actively granting web content the ability to access local file paths" capability gates.
2. **`WindowConfig` boundary is clear**: existing fields are visual / geometric / presentation properties of the window — no "web-content capability" or "event-routing" fields.
3. **Top level is reserved for cross-group globals**: top-level fields are global properties shared across multiple sub-groups; `capture_file_drop` has a clear sub-group home.

### 3.2 `ContentConfig` Field Definition

```python
@dataclass
class ContentConfig:
    """Initial content configuration and content-side capability gates.

    Attributes:
        url: URL to load (optional)
        html: HTML content to load (optional)
        asset_root: Root directory for auroraview:// protocol (optional)
        allow_file_protocol: Enable file:// protocol (default: False, security risk)
        capture_file_drop: Forward OS file drops as IPC ``file_drop`` events.
            Tri-state with ``Optional[bool]``:

            - ``None`` (default) — inherit lower-layer default (currently ``False``).
            - ``True`` — force enable; HTML5 ``dragover``/``drop`` inside the
              WebView become inert (upstream wry/WebView2 limitation, see RFC 0015 §2).
            - ``False`` — force disable.

            Note: ``capture_file_drop`` is ignored in multi-tab Browser mode
            (RFC 0016). For absolute file paths via IPC, use a top-level
            ``AuroraView`` instance.
    """

    url: Optional[str] = None
    html: Optional[str] = None
    asset_root: Optional[str] = None
    allow_file_protocol: bool = False
    capture_file_drop: Optional[bool] = None
```

### 3.3 `from_kwargs` / `to_kwargs` Passthrough Rules

In `from_kwargs`, the `ContentConfig` constructor uses `kwargs.get(key)` **without a default**:

```python
content = ContentConfig(
    url=kwargs.get("url"),
    html=kwargs.get("html"),
    asset_root=kwargs.get("asset_root"),
    allow_file_protocol=kwargs.get("allow_file_protocol", False),
    # KEY: no default. The `None` / `True` / `False` tri-state must be
    # preserved all the way to the Rust side.
    capture_file_drop=kwargs.get("capture_file_drop"),
)
```

`to_kwargs` adds the corresponding passthrough; **never** write `or False` / `bool(...)` to flatten `None`. **The comment must explicitly anchor the semantic**:

```python
# RFC 0017 tri-state contract: capture_file_drop must remain Optional[bool]
# all the way to the Rust PyO3 binding (src/bindings/desktop_runner.rs),
# where Rust's unwrap_or(false) provides the final fallback. Adding a default
# here or in any downstream glue (or False / setdefault / bool(...)) would
# collapse "not passed" and "explicit False" into one state, breaking the
# tri-state semantics.
"capture_file_drop": self.content.capture_file_drop,   # Optional[bool], passed to Rust verbatim
```

### 3.4 Per-File Change List

| File | Change |
|---|---|
| `python/auroraview/core/config.py` | `ContentConfig` adds the field + `from_kwargs` switches to `kwargs.get("capture_file_drop")` (no default) + `to_kwargs` adds passthrough in the `# Content` block + comment anchor |
| `python/auroraview/core/factory.py` | When forwarding to the Rust binding layer, do **not** flatten `None` |
| `python/auroraview/core/mixins/factory.py` | Do not introduce `setdefault`; `**kwargs` passes `capture_file_drop` through verbatim |
| `python/auroraview/core/mixins/content.py` | If it constructs `ContentConfig`, sync passthrough (consistent with `allow_file_protocol`) |
| `python/auroraview/__main__.py` | CLI entry forwards `--capture-file-drop` to the lower layer (only `True` / not-passed) |
| `python/auroraview/integration/qt/_core.py` | The DCC (Qt) path applies **no** special handling; uses `ContentConfig.capture_file_drop = None` default |
| `src/bindings/desktop_runner.rs` | PyO3 binding field mapping: receives `Option<bool>` from Python; on landing into Rust `WebViewConfig.capture_file_drop`, `unwrap_or(false)`; Rust's `WebViewConfig` itself stays `bool` (the bottom-most source of truth) |

> **Bindings outside the passthrough chain**: `src/bindings/webview2.rs` exposes a minimal HWND-based handle API and does **not** accept `WebViewConfig`, so it requires no passthrough.

### 3.5 DCC Call-Chain Field Flow

| # | Layer | Type / function | Field | Type |
|---|---|---|---|---|
| 1 | Python user code | `AuroraView(parent=..., capture_file_drop=True)` | kwarg | `Optional[bool]` |
| 2 | Python base class | `core/config.py::ContentConfig` | `capture_file_drop: Optional[bool]` | tri-state |
| 3 | Python factory | `core/factory.py::create_webview` | passthrough dict | tri-state |
| 4 | Python mixin | `core/mixins/factory.py` / `mixins/content.py` | passthrough, no `setdefault` | tri-state |
| 5 | DCC Qt layer | `integration/qt/_core.py` | **direct passthrough, no special handling** | tri-state |
| 6 | PyO3 binding | `src/bindings/desktop_runner.rs` | receives `Option<bool>`, lands with `unwrap_or(false)` | tri-state → `bool` |
| 7 | Rust core | `src/webview/config.rs::WebViewConfig` | `capture_file_drop: bool` | `bool` (bottom-most) |
| 8 | Rust backend | `src/webview/backend/native.rs::NativeBackend::create_webview` | reads and passes to `attach_drag_drop_handler` | `bool` |

---

## 4. Test Plan

### 4.1 Python Unit Tests

`tests/python/unit/test_file_drop_events.py` adds:

- `test_capture_file_drop_default_none` — construct `AuroraView()` without the kwarg, assert `cfg.content.capture_file_drop is None` (verify the tri-state: not passed = `None`, **not** `False`).
- `test_capture_file_drop_explicit_true` — explicit `True` → `cfg.content.capture_file_drop is True`.
- `test_capture_file_drop_explicit_false` — explicit `False` → `cfg.content.capture_file_drop is False` (guard against "explicit False being flattened to None").

### 4.2 Python Integration Tests

`tests/python/integration/test_capture_file_drop_passthrough.py` (new) covers tri-state passthrough across call-chain layers 1→8:

- `test_passthrough_explicit_true`: construct `AuroraView(capture_file_drop=True)`, use the PyO3 test hook `_dump_config()` to assert Rust `WebViewConfig.capture_file_drop` receives `true`.
- `test_passthrough_explicit_false`: construct `AuroraView(capture_file_drop=False)`, assert Rust receives `false` (verifies that explicit `False` is not flattened by an intermediate `setdefault`).
- `test_passthrough_omitted_falls_to_default_false` (**core defense**): construct `AuroraView()` (without the kwarg), assert:
  - At intermediate dataclasses: `cfg.content.capture_file_drop is None` (stays `None` until reaching Rust);
  - At the final Rust side: `_dump_config().capture_file_drop is False` (from Rust's `unwrap_or(false)` fallback).

### 4.3 Child Window Isolation Tests

`tests/python/unit/test_child_window_isolation.py` (new) protects the D11 outcome of RFC 0015 §3.6 — "users constructing `AuroraView` should **not** be incorrectly rejected for combining `capture_file_drop=True` with `new_window_mode='child_webview'`":

- `test_main_window_capture_file_drop_with_child_webview_mode_is_legal`: construct `AuroraView(capture_file_drop=True, new_window_mode="child_webview")`, assert **no errors are raised during construction** (this is a legal config: the main window's IPC works normally, and the child window not attaching the handler is the expected behavior).
- `test_main_window_capture_file_drop_false_with_child_webview_mode_is_legal`: same but `capture_file_drop=False`.
- `test_main_window_capture_file_drop_omitted_with_child_webview_mode_is_legal`: same but kwarg not passed (covers the `None` branch).

> **Purpose**: prevent anyone from later adding "if `child_webview`, reject `capture_file_drop`" over-blocking logic at the Python layer. This file does **not** verify child-window internal code (the Rust side guards that via RFC 0015 §6.1 + §5 CI grep).

---

## 5. CI Regression Grep

`scripts/ci/check_capture_file_drop_defaults.py` (integrated into `vx just test`):

```bash
#!/usr/bin/env bash
set -euo pipefail

# Forbidden pattern 1: setdefault for capture_file_drop in the Python passthrough chain
if rg --type py "setdefault\(.{0,20}['\"]capture_file_drop['\"]" python/auroraview/ ; then
    echo "ERROR: capture_file_drop must be passed through as Optional[bool] (RFC 0017 §3.3)"
    exit 1
fi

# Forbidden pattern 2: flattening None into False in the Python passthrough chain
if rg --type py "(get|pop)\(['\"]capture_file_drop['\"],\s*(True|False)" python/auroraview/ ; then
    echo "ERROR: do not provide a default for capture_file_drop in Python passthrough; \
           the field must remain Optional[bool] until it reaches Rust unwrap_or (RFC 0017 §3.3)"
    exit 1
fi

# Forbidden pattern 3: or False / or True flattening
if rg --type py "capture_file_drop\s+or\s+(True|False)" python/auroraview/ ; then
    echo "ERROR: do not flatten Optional[bool] capture_file_drop with 'or' in Python (RFC 0017 §3.3)"
    exit 1
fi

echo "OK: capture_file_drop passthrough rules satisfied."
```

The only allowed exception is `unwrap_or(false)` at the PyO3 binding layer in `src/bindings/desktop_runner.rs` (located in Rust, where the grep does not match).

> **No lint plugin**: in-repo grep is a sufficient "tripwire"; cross-language lint configuration would introduce new dependencies. This script and §4.2's `test_passthrough_omitted_falls_to_default_false` form a "static + runtime" double defense.

---

## 6. Implementation Steps

1. **Step 1 — `ContentConfig` field**: `python/auroraview/core/config.py` adds `capture_file_drop: Optional[bool] = None` + docstring.
2. **Step 2 — `from_kwargs` / `to_kwargs`**: switch to `kwargs.get("capture_file_drop")` without default + `to_kwargs` adds passthrough + comment anchor.
3. **Step 3 — Passthrough layers**: `factory.py` / `mixins/factory.py` / `mixins/content.py` / `integration/qt/_core.py` / `__main__.py` all sync passthrough; forbid `setdefault` / `or False`.
4. **Step 4 — PyO3 binding**: `src/bindings/desktop_runner.rs` receives `Option<bool>` and lands with `unwrap_or(false)` into Rust `WebViewConfig.capture_file_drop`.
5. **Step 5 — CI grep**: add `scripts/ci/check_capture_file_drop_defaults.py`, integrated into `vx just test`.
6. **Step 6 — Tests**: add all three test classes from §4; expose `_dump_config` PyO3 test hook (only enabled under `cfg(test)`) so integration tests can observe the Rust-side value.
7. **Step 7 — DCC migration guide**: prominently announce in CHANGELOG / docs/zh/guide:

   > Previously, when using AuroraView in DCC hosts like Maya / Houdini / Nuke, file drops were automatically forwarded as `file_drop` IPC events. Starting from this version, the DCC path defaults to "do not intercept", consistent with standalone and other modes.
   >
   > If your DCC tool relies on `file_drop` events, explicitly pass `capture_file_drop=True` when constructing `AuroraView`:
   >
   > ```python
   > from auroraview import AuroraView
   >
   > class MyDccTool(AuroraView):
   >     def __init__(self, parent=None):
   >         super().__init__(parent=parent, capture_file_drop=True)
   > ```

Each step is verified with `vx just test`.

---

## 7. Compatibility

- **Python API**: `AuroraView(...)` / `ContentConfig(...)` add a kwarg with default `None` → existing callers that did not pass the kwarg behave consistently with the new Rust default `false` — **no breakage for standalone / CLI / packed users**.
- **DCC user behavior change**: previously the DCC path defaulted to `True` (auto-intercept drops); the new version defaults to `None → false`. **This is the direct manifestation of the breaking change called out in RFC 0015 §8.1** — DCC users must explicitly pass `capture_file_drop=True` at construction time (one line of code).
- **Python kwarg `None` vs `False`**: under the current architecture, both end up as `false` on the Rust side, but the intermediate data structures differ (used for future extensions); guarded by the tri-state contract + CI grep double defense.

---

## 8. Risks

| Risk | Assessment | Mitigation |
|---|---|---|
| Intermediate layer overrides user input (silent override) | Medium | Triple defense: §3.3 passthrough rule forbids `setdefault` / `or False`; §4.2 integration test `test_passthrough_omitted_falls_to_default_false`; §5 CI grep |
| User bypasses `from_kwargs` and constructs `WebViewConfig` directly, then `to_kwargs` flattens it | Medium | §3.3's inline comment anchor near `to_kwargs` provides nearby context for users debugging glue code |
| User confuses `None` vs `False` | Low | docstring + DCC migration guide clarify "not passed = use code default", "explicit `False` = force off"; under the current architecture, both end up the same, with the difference only surfacing in future extensions |

---

## 9. Follow-up RFCs

- If the PyO3 entry point later wires up a new lower-layer source (`[user_settings]` toml / remote config / hot reload), the `None`-falls-to-lower-layer semantic kicks in naturally; the tri-state contract's preserved value comes into play **without** RFC revision.
- If a "single-process Python interpreter + packed runtime merger" architecture is introduced in the future (no current plan), an independent RFC will then handle priority merging between `unwrap_or(false)` and packed overlay.
