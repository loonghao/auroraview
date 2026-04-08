# AuroraView Cleanup TODO

Tracked by the cleanup agent. Items here are confirmed improvement opportunities
that require larger effort or coordination before implementation.

---

## High Priority

### `crates/auroraview-pack/src/packed/webview.rs` — parking_lot migration
- **Status**: **RESOLVED (Round 41)**
- **Reason**: File no longer exists — likely refactored into `packer/desktop.rs` or other modules.
  The `auroraview-pack` crate no longer uses `std::sync::Mutex` or `std::sync::RwLock`.
- **Action**: No action needed. Removed from TODO list.

---

## Medium Priority

### DCC + Desktop `IpcRouter` code duplication (~90%)
- **Status**: TODO (logged Round 2)
- **Reason**: `dcc` and `desktop` crates share nearly identical `IpcRouter` implementations.
  Recommend extracting shared logic into a new `auroraview-ipc-common` crate.
- **Risk**: Medium — involves cross-crate refactoring.

### API alias redundancy in WindowManagers
- **Status**: TODO (logged Round 2)
- **Reason**: `get()`/`get_info()` and `list()`/`window_ids()` are duplicated in both
  DCC and Desktop WindowManagers.
- **Action**: Consolidate to one canonical name per operation and deprecate aliases.

### `crates/auroraview-core/src/assets.rs` — browser controller fallback for source checkouts
- **Status**: TODO (logged Round 22)
- **Reason**: `get_browser_controller_html()` still hard-depends on built `frontend/dist/browser-controller/index.html`.
  In source checkouts without built frontend assets, Python browser integration tests still hit a panic.
- **Action**: Add a fallback HTML shell for the browser controller, similar to the new loading/error fallbacks,
  or relax obsolete tests that assume built frontend assets are always present.

### `pyproject.toml` / mypy toolchain — restore Python 3.7-compatible type-check gate
- **Status**: TODO (logged Round 22)
- **Reason**: The installed mypy no longer accepts `python_version = 3.7`, so `uv run mypy python tests`
  fails before type analysis even starts.
- **Action**: Pin a mypy version that still supports Python 3.7 targets or split legacy-target checks from modern dev-tool execution.

### `create_for_dcc` deprecated methods (webview.py + factory.py)
- **Status**: **RESOLVED (Round 45)**
- **Reason**: Deprecated since 0.4.0, current version 0.4.18 (18 minor versions later).
  Zero callers found in codebase (no tests, examples, docs, or external usage).
- **Action**: Removed `create_for_dcc()` from both `webview.py` and `factory.py`.
  Net change: -60 lines.

### Unused `deprecated` export from `__init__.py`
- **Status**: **RESOLVED (Round 45)**
- **Reason**: `deprecated` was imported and exported in `__init__.py` but never called
  anywhere in the codebase. No definition found — likely a leftover from early development.
- **Action**: Removed `deprecated` from both import and `__all__` export lists.
  Net change: -2 lines.

### Clippy `nonminimal_bool` warnings in test code
- **Status**: **RESOLVED (Round 45)**
- **Reason**: `message_processor_tests.rs:184-185` used `!(a >= b)` instead of `<`.
- **Action**: Simplified to direct comparison operators.

---

## Low Priority

### `issues.md` at repo root
- **Status**: RESOLVED (Round 33)
- **Reason**: File no longer exists — likely deleted in a prior round or never created.

### `#[allow(dead_code)]` annotations (~36 total in src/ + 21 in crates/)
- **Status**: Structural / Feature-gated / BOM API预留
- **Reason**: Majority are justified:
  - BOM API 预留方法 (webview_inner.rs: is_window_valid, toggle_fullscreen, hide, focus)
  - Standalone mode fields (window, event_loop, auto_show, backend)
  - Platform-conditional code (Windows-only NativeBackend, warmup stages)
  - IPC internal structs (ThreadedConfig, MessageQueueConfig)
  - Legacy wry-backend methods (create_embedded, create_standalone, create_desktop)
- Not actionable until features are implemented or backend migration completes.
- **Update (Round ~37)**: Full scan shows ~57 annotations across src/ and crates/.
  Previous count of 5 was only tracking a specific subset. The majority are structural.
- **Update (Round 41)**: Reduced to 3 annotations in production code:
  - `json_tests.rs`: test-local struct (normal pattern)
  - `vibrancy.rs::DWMSBT_TRANSIENTWINDOW`: future Win11 acrylic constant
  - `overlay.rs::HEADER_SIZE_UNUSED`: TODO(cleanup) reserved constant
  LockOrderGuard.name field now uses Debug impl (dead_code removed).
- **Update (Round 42)**: Reduced to 2 annotations in production code:
  - `json_tests.rs`: test-local struct (normal pattern)
  - `vibrancy.rs::DWMSBT_TRANSIENTWINDOW`: future Win11 acrylic constant
  - `overlay.rs::HEADER_SIZE_UNUSED` removed — zero references, was reserved for future validation that never materialized.
- **Update (Round 43)**: Improved DWMSBT_TRANSIENTWINDOW documentation with MSFT reference URL.
  Count remains at 2. Both annotations are justified:
  - `json_tests.rs`: test-local struct (normal pattern, safe to keep)
  - `vibrancy.rs::DWMSBT_TRANSIENTWINDOW`: Win11 DWM API reserved constant with doc reference

### Deprecated run_standalone in documentation
- **Status**: **RESOLVED (Round 43)**
- **Reason**: Updated both EN and ZH API index docs to clearly mark `run_standalone` as removed.
- **Action**: Changed from "Deprecated" to "Removed" with migration note pointing to `run_desktop`.

### README stale tech stack versions (Rust/PyO3/Wry/Tao)
- **Status**: **RESOLVED (Round 44)**
- **Reason**: README.md and README_zh.md showed Rust 1.75+, PyO3 0.22, Wry 0.47, Tao 0.30;
  actual values are Rust 1.90+, PyO3 0.27, Wry 0.54, Tao 0.34.
- **Action**: Updated badges, tech stack section in both EN and ZH READMEs.

### README broken link DCC_INTEGRATION.md
- **Status**: **RESOLVED (Round 44)**
- **Reason**: Both READMEs referenced `./docs/DCC_INTEGRATION.md` which does not exist.
- **Action**: Changed link to point to `./docs/dcc/index.md`.

### Missing parking_lot dependency in auroraview-telemetry
- **Status**: **RESOLVED (Round 44)**
- **Reason**: `telemetry/python.rs` was migrated to use `parking_lot::Mutex` in Round 43,
  but `parking_lot` was never added to `auroraview-telemetry/Cargo.toml`.
- **Action**: Added `parking_lot = "0.12"` to telemetry crate dependencies.

### pytest-qt in wrong dependency group
- **Status**: **RESOLVED (Round 44)**
- **Reason**: `pytest-qt` was in `[project.optional-dependencies].qt` (runtime optional)
  instead of `[dependency-groups].test` (test-only).
- **Action**: Moved `pytest-qt` to test dependency group; removed from qt optional deps.

### Stale coverage omit paths (pyproject.toml)
- **Status**: **RESOLVED (Round 44)**
- **Reason**: coverage omit referenced `*/auroraview/qt_integration.py` and
  `*/auroraview/webview.py` which do not exist in the Python package.
- **Action**: Removed nonexistent paths from coverage omit configuration.

### `crates/auroraview-pack/tests/metrics_tests.rs` — sleep-based timing assertions
- **Status**: TODO (logged Round 21)
- **Reason**: The suite relies on `thread::sleep(Duration::from_millis(...))`, which makes it slower
  and more timing-sensitive than necessary.
- **Action**: Introduce a controllable timing helper or loosen the test strategy to avoid wall-clock sleeps.

### Rust events.rs deprecated navigation callbacks (4 methods)
- **Status**: **RESOLVED (Round 42)**
- **Reason**: These methods no longer exist in the codebase — they were removed in a prior refactoring round.
  No action needed.

### Unused Python `deprecated()` decorator (CLEANED Round ~31)
- **Status**: RESOLVED
- **Reason**: `event_emitter.deprecated()` had 0 callers. Removed with unused `warnings` import.
- **Net change**: -17 lines

---

## Structural Assessment (Round 33)

### Large module files (>500 lines) in `auroraview-core`
| File | Lines | Recommendation |
|------|-------|----------------|
| `builder/window_style.rs` | **1056** | Win32 窗口样式操作 — 可拆分为 child/owner/frameless/shadow 子模块 |
| `assets.rs` | **699** | 静态资源加载器（大量 getter 函数）— 考虑用 macro 生成重复的 JS getter |
| `assets/html/error_pages.rs` | **602** | HTML 错误页面模板 — CSS 常量可提取到独立文件，模板函数可用 Askama 替代 |
| `dom/ops.rs` | **597** | DOM 操作枚举 + JS 代码生成 — op_to_js match 可按类别拆分到子模块 |
| `protocol.rs` | **513** | 协议工具 + MemoryAssets — 两个职责可分离为 `protocol_utils.rs` + `memory_assets.rs` |

> **Note**: 以上均为观察性发现，当前不构成直接重构理由。各模块功能内聚性良好，拆分应在功能扩展时自然进行。

### 循环依赖
- **结论**: 未发现循环依赖。依赖方向为健康的有向无环图 (DAG)。

### 极简模块 (<100 行有效代码)
- `cli/mod.rs` (11行), `dom/mod.rs` (20行), `assets/html/mod.rs` (11行): 正常桶模块模式
- `builder/common_config.rs` (30行), `builder/protocol.rs` (48行), `cli/url_utils.rs` (42行): 功能单一但独立性强

---

## Resolved

- [x] `build_cli.py` deleted (Round 1)

- [x] `active-win-pos-rs` unused dep removed (Round 1)
- [x] Empty `if TYPE_CHECKING: pass` blocks removed (Round 2)
- [x] Redundant `unsafe impl Send+Sync` for TabManager, SignalRegistry, EventBus (Rounds 2-3)
- [x] `pr_body.md`, `.gitcommitmsg` stale files deleted (Round 3)
- [x] parking_lot migration complete in core production code (Rounds 3-4, f16e7df)
- [x] Import ordering violations (Rounds 4-5, 6 batches)
- [x] 5 `approx_constant` clippy errors in settings_tests.rs (Round 5)
- [x] Duplicate `escape_js_string` in assets.rs removed (Round 6)
- [x] Inline JS escaping in webview.rs replaced with `escape_js_string` (Round 6)
- [x] `plugins.rs` parking_lot migration — 13 poison error handlers removed, -25 net lines (Round ~35)
- [x] Unused `hyper` + `hyper-util` deps removed from root Cargo.toml (Round ~37, 58c0178)
- [x] Temporary artifacts deleted: llms-full.txt, llms.txt (Round ~37, -29KB)
- [x] `#[allow(dead_code)]` count updated to accurate ~57 across workspace (Round ~37)
- [x] `LockOrderGuard.name` field now uses Debug impl — dead_code removed (Round 41)
- [x] `window_style.rs` Mutex migrated to parking_lot (Round 41)
- [x] `telemetry/python.rs` Mutex migrated to parking_lot (Round 41)
- [x] `window_style.rs` mismatched brace fix from Round 41 parking_lot migration (Round 42)
- [x] `HEADER_SIZE_UNUSED` constant removed from overlay.rs — zero refs (Round 42)
- [x] Deprecated navigation callbacks confirmed removed from events.rs (Round 42)
