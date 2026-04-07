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

### `crates/auroraview-pack/tests/metrics_tests.rs` — sleep-based timing assertions
- **Status**: TODO (logged Round 21)
- **Reason**: The suite relies on `thread::sleep(Duration::from_millis(...))`, which makes it slower
  and more timing-sensitive than necessary.
- **Action**: Introduce a controllable timing helper or loosen the test strategy to avoid wall-clock sleeps.

### Rust events.rs deprecated navigation callbacks (4 methods)
- **Status**: TODO (logged Round ~31)
- **Reason**: 4 DEPRECATED callbacks (`on_navigation_started/completed/failed`, `on_load_progress`)
  have replacement APIs (`on_navigation`, `on_progress`). No external callers found in Python code.
- **Risk**: Low — but requires confirming no external users depend on these before removal.
- **Action**: Add `#[deprecated]` attribute or plan for removal in v0.6+.

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
