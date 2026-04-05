# AuroraView Cleanup TODO

Tracked by the cleanup agent. Items here are confirmed improvement opportunities
that require larger effort or coordination before implementation.

---

## High Priority

### `crates/auroraview-cli/src/packed/webview.rs` — parking_lot migration
- **Status**: TODO
- **Reason**: File still uses `std::sync::RwLock` (poison-based API).
  Migrating requires replacing every `match x.read() { Ok(g) => ... }` pattern (~16 call sites)
  with direct `x.read()` guard access (parking_lot is non-poisoning).
- **Risk**: Medium — all lock usage must be verified; the file is 1700+ lines.
- **Action**: Migrate in a dedicated commit; ensure all `if let Ok(g) = x.read()` /
  `match x.read() { Ok(g) => ..., Err(e) => ... }` patterns are updated.

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
- **Status**: TODO (logged Round 1)
- **Reason**: Contains a code review report; should be moved to `docs/` or converted
  to GitHub Issues for proper tracking.

### `#[allow(dead_code)]` annotations (~58 total)
- **Status**: Structural / Feature-gated
- **Reason**: Most are BOM API stubs awaiting implementation or feature-gated code.
  Not actionable until features are implemented.
- **Update (Round ~27)**: Count updated from 95 to 58. Major reductions from prior rounds:
  - webview_inner.rs: still 12 (BOM API)
  - backend/native.rs: 6
  - ipc/*: 16 across handler, message_queue, threaded, backend
  - event_loop.rs: 4
  - message_pump.rs: 3
  - Others: vibrancy.rs (1), overlay.rs (1), view_manager.rs (1)

### `crates/auroraview-pack/tests/metrics_tests.rs` — sleep-based timing assertions
- **Status**: TODO (logged Round 21)
- **Reason**: The suite relies on `thread::sleep(Duration::from_millis(...))`, which makes it slower
  and more timing-sensitive than necessary.
- **Action**: Introduce a controllable timing helper or loosen the test strategy to avoid wall-clock sleeps.

---

## Structural Assessment (Round 24)

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
