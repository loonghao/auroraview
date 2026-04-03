# AuroraView Cleanup Agent Memory

## 2026-03-30 23:10 — Round 1 (Initial)

### Branch: `auto-improve` (created from `main` at `2cfa745`)

### Baseline
- **Cargo check**: PASS (all workspace crates compile)
- **Clippy**: PASS (0 warnings)
- **auroraview-assets**: Had compile error due to missing `frontend/dist/` directory; fixed with `.gitkeep`

### Actions Taken (Commit: `db9273e`)
1. **Deleted `build_cli.py`** — Stale build script, replaced by justfile `build-cli` recipe. Not referenced in CI or justfile.
2. **Deleted `$null`** — Junk file from PowerShell redirection error. Added `$null` to `.gitignore`.
3. **Removed `active-win-pos-rs` dependency** — Declared in root Cargo.toml but never `use`d in any .rs file.
4. **Added `crates/auroraview-assets/frontend/dist/.gitkeep`** — rust-embed `#[derive(RustEmbed)]` requires the target folder to exist. Added `.gitkeep` with gitignore exclusion rule.
5. **Kept `ipckit`** — Initially removed but found it IS used in `src/ipc/async_handler.rs` and `src/ipc/message_queue.rs`.

### Findings Logged for Future Rounds
- **56 TODO comments** across Rust codebase; `crates/auroraview-plugins/src/extensions.rs` alone has 22 (all placeholder stubs).
- **30 `#[allow(dead_code)]`** annotations; `crates/auroraview-extensions/` has the most dead code fields.
- **24 `#[ignore]` tests** — all have valid reasons (timing-sensitive, requires CDP/display/Python runtime).
- **5 empty `if TYPE_CHECKING: pass` blocks** in Python code — can be safely removed.
- **1 suspicious `pass`** in `python/auroraview/features/persistence.py:141` — `to_dict()` method appears unimplemented.
- **`issues.md`** at root is a code review report, has value but should be moved to docs/ or converted to GitHub Issues.
- **webview2-com 0.38.2 vs 0.39.1** — Normal: wry 0.54.4 pulls 0.38.2 as transitive dep; root crate uses 0.39.1 directly.

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)

## 2026-03-31 02:10 — Round 2

### Branch: `auto-improve` (HEAD: `68e5e91`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- Iterate Agent committed 4 perf changes since Round 1 (DashMap migration in DCC/Desktop/Plugins/Tabs)

### Actions Taken (Commit: `68e5e91`)
1. **Removed 5 empty `if TYPE_CHECKING: pass` blocks** — `telemetry.py`, `javascript.py`, `events.py`, `api.py`, `backend.py`; also removed unused `TYPE_CHECKING` imports
2. **Removed `unsafe impl Send + Sync` for `TabManager`** — All fields (DashMap, parking_lot::RwLock, AtomicU32, Vec<Box<dyn Fn + Send + Sync>>) auto-derive Send+Sync; unsafe impls were redundant and a latent safety risk

### Code Review Findings (Iterate Agent's perf commits)
- **DashMap migration**: Fully complete in DCC, Desktop, Tabs; `process.rs` correctly keeps `RwLock` for non-HashMap `event_callback`
- **HIGH: 13 `.unwrap()` calls** in `process.rs` on `RwLock/Mutex` — risk of panic on poison. Recommend switching to `parking_lot::RwLock` (no poison). Logged for CLEANUP_TODO.
- **MED: DCC+Desktop IpcRouter ~90% code duplication** — Recommend extracting shared crate. Logged for CLEANUP_TODO.
- **MED: API alias redundancy** — `get()`/`get_info()` and `list()`/`window_ids()` in both WindowManagers. Logged for CLEANUP_TODO.
- **LOW: `persistence.py:141`** — `to_dict()` method body is empty `pass`, possibly unimplemented (carried from Round 1)

### Metrics
- `#[allow(dead_code)]`: 87 (unchanged from Round 1 — structural, mostly BOM API / feature-gated)
- Empty `if TYPE_CHECKING:` blocks: 5 → 0
- `unsafe impl`: Removed 2 unnecessary from TabManager
- `todo!()` / `unimplemented!()`: 0

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)

## 2026-03-31 05:18 — Round 3

### Branch: `auto-improve` (HEAD: `ba5a1d9`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- Iterate Agent committed 3 DashMap migration batches since Round 2

### Actions Taken (Commit: `ba5a1d9`)
1. **Removed `unsafe impl Send+Sync` for `SignalRegistry`** — fields auto-derive Send+Sync via parking_lot::RwLock
2. **Removed `unsafe impl Send+Sync` for `EventBus`** — same reason
3. **Deleted `pr_body.md`** — stale PR description file
4. **Deleted `.gitcommitmsg`** — stale commit message draft

### Findings for Future Rounds
- **WebViewProxy unsafe impl** (proxy.rs:56-57): KEPT — contains non-Send types via MessageQueue/JsCallbackManager
- **~166 `.unwrap()` on std::sync locks** in process.rs/browser_bridge.rs — recommend parking_lot migration
- **7 extension APIs still on `RwLock<HashMap>`** — tab_groups, omnibox, management, history, downloads, cookies, bookmarks
- **issues.md** — still at root, recommend converting to GitHub Issues
- **87 `#[allow(dead_code)]`** — structural, mostly BOM API/feature-gated
- **46 GitHub dependency vulnerabilities** (24 high) — needs deps-focused round

### Metrics
- `unsafe impl Send/Sync`: 6 → 2
- Stale files removed: 2
- `#[allow(dead_code)]`: 87 (unchanged)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)

## 2026-03-31 08:25 — Round 4

### Branch: `auto-improve` (HEAD: `70aefdf`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- Iterate Agent committed 4 perf changes since Round 3 (parking_lot migration in bookmarks, history, plugins/core, remaining extension APIs, DevToolsManager)

### Actions Taken (Commit: `70aefdf`)
1. **Normalized import ordering in 5 files** — After parking_lot migration, `use parking_lot::*` was placed between `std::*` imports. Fixed to standard grouping: `std` → external crates → internal crates with blank line separators.
   - `crates/auroraview-plugins/src/process.rs`
   - `crates/auroraview-plugins/src/browser_bridge.rs`
   - `crates/auroraview-history/src/manager.rs`
   - `crates/auroraview-plugins/core/src/router.rs`
   - `crates/auroraview-bookmarks/src/manager.rs`

### Code Review Findings (Iterate Agent's perf commits since Round 3)
- **parking_lot migration COMPLETE** in process.rs, browser_bridge.rs, bookmarks, history, plugins/core — 0 `.unwrap()` on lock operations (resolved Round 2/3 HIGH finding)
- **DevToolsManager migrated to DashMap** — devtools.rs uses DashMap + parking_lot::RwLock
- **Extension APIs fully migrated** — all 7 APIs now on DashMap (resolved Round 3 finding)
- **`persistence.py:141`** — Confirmed: `_item_to_dict()` is `@abstractmethod`, NOT a bug. Closing.
- **WebViewProxy unsafe impl** (proxy.rs:56-57): KEPT — only remaining unsafe Send+Sync
- **`#[allow(unused_imports)]`**: 8 in 4 files — structural/feature-gated
- **Main crate (`src/`) still uses `std::sync::Mutex`** in ~15 files — may be intentional (PyO3 constraints)

### Metrics
- `unsafe impl Send/Sync`: 2 (unchanged)
- `#[allow(dead_code)]`: ~95
- Import ordering violations: 5 → 0
- **46 GitHub dep vulnerabilities** (24 high) — pending

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)

## 2026-03-31 11:32 — Round 5

### Branch: `auto-improve` (HEAD: `4a11896`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: Had 5 `approx_constant` errors in settings_tests.rs (from iterate Agent's test commit)

### Iterate Agent Changes Reviewed (3 commits since Round 4)
1. `33149f5` — thiserror migration for WebViewError, BomError, PortError + doc comments on `pub mod` — **GOOD**
2. `bc50fe9` — Comprehensive tests for notifications and settings crates — **GOOD** but contained clippy violation
3. `321363d` — Replace `.unwrap()` with proper error propagation, clipboard parking_lot — **GOOD**

### Actions Taken (Commit: `4a11896`)
1. **Fixed 5 clippy `approx_constant` errors** in `settings_tests.rs` — replaced `3.14` with `2.78`
2. **Deleted junk files** `test_other.txt`, `test_out.txt` — cargo test output remnants
3. **Added `test_*.txt` to `.gitignore`**
4. **Normalized import grouping** in ai-agent `session.rs` and `wrapper.rs`

### Findings for Future Rounds
- **46 GitHub dependency vulnerabilities** (24 high) — needs dedicated deps round
- **`#[allow(dead_code)]`**: ~95 (structural)
- **`unsafe impl Send/Sync`**: 2 (WebViewProxy only)
- **`issues.md`** at root — should be moved to docs/ or GitHub Issues
- **11 files in `src/` using `std::sync::Mutex`** — likely PyO3 constraints
- **`serde_json::to_value().unwrap()`**: 0 in production, 2 in tests (acceptable)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 warnings)

## 2026-04-01 06:31 — Round 6

### Branch: `auto-improve` (HEAD: `e18cd63`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 3 features since Round 5: hot-reload (`--watch` flag), inject_js/inject_css from pack manifest, CSS injection via `<style>` element

### Actions Taken (Commits: `8c23090`, `ee9e9ef`, `e18cd63`)
1. **Fixed import ordering in `run.rs`** — `use std::path` was placed after `anyhow`/`clap`; moved to top
2. **Fixed import ordering in `packed_tests.rs`** — `use std::*` appeared after `auroraview_cli`/`auroraview_core`; moved to top
3. **Fixed import ordering in `config_tests.rs`** — `use std::path::PathBuf` appeared after `auroraview_pack`; moved to top
4. **Fixed import ordering in `packed/mod.rs`** — `use std::time::Instant` appeared after two external crates; moved to top

### Code Review Findings (Iterate Agent's 3 commits)
- **hot-reload design**: `RunEvent` enum with `Reload` variant is correct future-proof pattern; RAII watcher handle correct
- **`canonicalize().unwrap_or_else(|_| html_path.clone())`** in `run.rs:307` — safe fallback (file existence pre-validated); acceptable
- **`build_css_injection_script`**: correct JS template literal escaping (backtick + backslash); consistent with existing escape utilities
- **`notify = "8.0"`**: used only in `auroraview-cli`; no duplication in dep tree; version constraint appropriate
- **GitHub dep vulnerabilities**: now 47 (25 high) — pending dedicated deps round

### Metrics
- Import ordering violations fixed: 4
- Clippy warnings: 0 (unchanged)
- Ruff warnings: 0 (unchanged)
- `unsafe impl Send/Sync`: 2 (unchanged)
- `#[allow(dead_code)]`: ~95 (structural)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check`: PASS (0 warnings)

## 2026-04-01 12:51 — Round 8

### Branch: `auto-improve` (HEAD: `66471b0`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 2 test batch commits since Round 7:
  - `b5fbd06` — browser crate tests (tabs/bookmarks/downloads/history, 214 tests)
  - `e3eb62f` — devtools crate tests (84 tests)

### Actions Taken (Commits: `0c9e8d7`, `66471b0`)
1. **Fixed import ordering in `tab_tests.rs`** — `std::sync::*` was after `rstest`/external crates; moved to top
2. **Fixed import ordering in `download_tests.rs`** — `std::path::Path` was after `rstest`/external crates; moved to top
3. **Deleted `check_err.txt` and `clippy_out.txt`** — stale cargo/clippy debug output files left by Iterate Agent
4. **Added `check_err.txt` and `clippy_out.txt` to `.gitignore`** — prevent recurrence

### Code Review Findings (Iterate Agent's 2 test commits)
- **214 browser tests** (tabs/bookmarks/downloads/history): all pass, clippy clean, rstest parametric pattern well-applied
- **84 devtools tests**: all pass, fixture pattern `#[fixture] fn default_manager()` is good practice
- **`history_tests.rs` and `bookmark_tests.rs`**: import ordering is clean (no std, all external)
- **`devtools_tests.rs`**: two separate `auroraview_devtools` use groups acceptable (different sub-paths); no `std` imports needed

### Metrics
- Import ordering violations fixed: 2
- Stale debug files removed: 2
- Clippy warnings: 0 (unchanged)
- Ruff warnings: 0 (unchanged)
- `unsafe impl Send/Sync`: 2 (WebViewProxy only, unchanged)
- `#[allow(dead_code)]`: ~95 (structural, unchanged)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check`: PASS (0 warnings)

## 2026-04-01 22:07 — Round 11

### Branch: `auto-improve` (HEAD: `0ad3f22`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 9 test batch commits since Round 10 (browser/protect/ai-agent/telemetry tests)

### Actions Taken (Commits: `8787f85`, `7ce0f10`, `b8e740a`, `0ad3f22`)
1. **Import ordering in `metrics_tests.rs`** — `std` imports after external crates; moved to top
2. **Import consolidation in `runtime_gen_tests.rs`** — merged split `auroraview_protect::` imports
3. **Removed `test_` prefix from `session_tests.rs`** (48 fns) — project convention
4. **Removed `test_` prefix from `metrics_tests.rs`** (37 fns) — project convention
5. **Import ordering in `dcc/ipc/handler.rs`** — `std::sync::Arc` after external crates; moved to top

### Findings for Future Rounds
- **`fd96da1` regression**: `TabListenerMap`/`on_event`/`off_event` removed from `TabManager` and `ListenerId`/`off()`/`listener_count()` removed from `IpcRouter` (functional revert, not cleanup scope)
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — pending dedicated deps round
- **`#[allow(dead_code)]`**: ~95 (structural, unchanged)

### Metrics
- Import ordering violations fixed: 3
- `test_` prefix violations removed: 85 fns (2 files)
- Clippy warnings: 0 / Ruff warnings: 0

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check`: PASS (0 warnings)

## 2026-04-02 04:19 — Round 13

### Branch: `auto-improve` (HEAD: `353807e`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 12 commits since Round 12: core/desktop/pack/telemetry test expansions (bom_tests ×59, config_tests ×35, metrics/signals/protocol/id_generator/port/templates/utils tests, desktop config_tests+ipc_tests, pack overlay/packer/progress/lib/metrics/hooks tests, telemetry guard tests)

### Actions Taken (Commits: `7b3be84`, `353807e`)
1. **Fixed import ordering in 9 new test files** — all had `use std::*` after external/internal crates:
   - `crates/auroraview-core/tests/metrics_tests.rs`
   - `crates/auroraview-core/tests/protocol_tests.rs`
   - `crates/auroraview-core/tests/signals_tests.rs`
   - `crates/auroraview-core/tests/id_generator_tests.rs`
   - `crates/auroraview-core/tests/port_tests.rs`
   - `crates/auroraview-desktop/tests/config_tests.rs`
   - `crates/auroraview-desktop/tests/ipc_tests.rs`
   - `crates/auroraview-pack/tests/metrics_tests.rs`
   - `crates/auroraview-pack/tests/packer_tests.rs`

### Code Review Findings (Iterate Agent's commits)
- **Pack/telemetry new public API** (`is_initialized`, `Packer`, `TargetPacker`, `PluginRegistry` re-exports): clean, no issues
- **Test naming**: no `test_` prefix violations in new files (existing `assets_tests.rs` with `test_` prefixes is pre-existing, not new)
- **`overlay_tests.rs`, `progress_tests.rs`, `hooks_tests.rs`**: import ordering clean, no std imports mixed
- **GitHub dep vulnerabilities**: ~48 (1 critical, 25 high) — still pending dedicated deps round

### Metrics
- Import ordering violations fixed: 9
- Clippy warnings: 0 / Ruff warnings: 0
- `unsafe impl Send/Sync`: 2 (WebViewProxy only, unchanged)
- `#[allow(dead_code)]`: ~95 (structural, unchanged)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check`: PASS (0 warnings)

## 2026-04-02 16:52 — Round 16

### Branch: `auto-improve` (HEAD: `1644e31`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff (python/examples/scripts/gallery)**: PASS (0 warnings) — first time full Python scope is clean
- Iterate Agent committed 1 test batch since Round 15:
  - `6da160c` — desktop tray_tests (23) + event_loop_tests (27) for TrayConfig/TrayMenuItem/UserEvent

### Actions Taken (Commits: `8ebc908`, `1644e31`)
1. **Removed `test_` prefix from 50 test functions** in 2 new desktop test files:
   - `tray_tests.rs` — 23 fns
   - `event_loop_tests.rs` — 27 fns
2. **Extracted inline `use std::path::PathBuf`** from `tray_config_with_icon_path` to file top
3. **Fixed 220+ ruff violations** in examples/scripts/gallery scope (previously unscoped):
   - W293/W291 × 118: trailing/blank-line whitespace (in HTML/CSS string literals)
   - F821 × 8: undefined names in `ai_chat_demo.py` (missing `Optional`, `QMetaObject`, `Q_ARG` imports — real bugs)
   - F401 × 6: unused imports in `qt_browser.py`, `ai_chat_demo.py`, `dependency_installer.py`
   - I001 × 5: import ordering violations
   - E402 × 4: module-level import not at top (added `# noqa: E402` for sys.path pattern)

### Code Review Findings (Iterate Agent's commit)
- **tray_tests.rs**: clean structure, serde round-trip tests, parametric `#[case]` well-applied
- **event_loop_tests.rs**: clean structure, Clone/Debug/parametric coverage
- **`ai_chat_demo.py` F821**: `QMetaObject` / `Q_ARG` / `Optional` were genuinely missing imports (not style — actual runtime bugs)

### Metrics
- `test_` prefix violations removed: 50 fns (2 files)
- Ruff violations fixed: 220+ (across 17 files)
- Real Python bugs fixed: 8 F821 undefined names
- Clippy warnings: 0 / Ruff warnings: 0 (project code scope)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 warnings)
- `uv run ruff check examples/ scripts/ gallery/ python/`: PASS (0 warnings)
- `git push origin auto-improve`: PASS



### Branch: `auto-improve` (HEAD: `c55ef8d`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: Had 3 errors + 1 warning across 2 crates before fix
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 3 test batch commits since Round 14:
  - `1bc7ac7` — core error_tests (52 tests for WebViewError/BomError/PortError/ServiceDiscoveryError)
  - `ee49024` — desktop error_tests (13) + window_manager_tests (30)
  - `573d161` — pack python_standalone_tests expanded 13→39

### Actions Taken (Commit: `c55ef8d`)
1. **Fixed `settings_tests.rs:42-43`** — `3.14f64` re-introduced by iterate Agent (clippy `approx_constant` error); replaced with `1.5f64`
2. **Fixed `devtools_tests.rs:106`** — `assert_eq!(back.is_open, true)` → `assert!(back.is_open)` (clippy `bool_assert_comparison`)
3. **Removed `test_` prefix from 12 fns** in `python_standalone_tests.rs`; changed `#[test]` → `#[rstest]`
4. **Extracted inline `use` statements** from `concurrent_create_is_safe` in `window_manager_tests.rs` → file top

### Code Review Findings (new test files)
- **`core/error_tests.rs`**: clean — `std::io` before external crates, no `test_` prefix ✓
- **`desktop/error_tests.rs`**: clean — same ✓
- **`desktop/window_manager_tests.rs`**: inline `use` extracted (fixed this round) ✓
- **`pack/python_standalone_tests.rs`**: 12 `test_` prefix violations fixed this round ✓

### Findings for Future Rounds
- **`settings_tests.rs` approx_constant recurring**: iterate Agent repeatedly re-introduces `3.14`; this is the 2nd fix (Round 5 + Round 15). May need to add `#![deny(clippy::approx_constant)]` to the test file.
- **GitHub dep vulnerabilities**: ~48 (1 critical, 25 high) — still pending dedicated deps round
- **`#[allow(dead_code)]`**: ~95 (structural, unchanged)

### Metrics
- Clippy errors/warnings fixed: 3 errors (approx_constant ×2) + 1 warning (bool_assert_comparison)
- `test_` prefix violations removed: 12 fns (python_standalone_tests.rs)
- Inline use statements extracted: 2 (window_manager_tests.rs)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (exit 0, 0 warnings)
- `uv run ruff check`: PASS (0 warnings)
- `git push origin auto-improve`: PASS



### Branch: `auto-improve` (HEAD: `e54135b`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 3 test batch commits since Round 13:
  - `3188641` — bundle/license/deps_collector/pyoxidizer tests (4 files)
  - `b0120b2` — signal_tests expanded to 61 tests
  - `1196041` — manifest_tests expanded to 45 tests

### Actions Taken (Commits: `7a1a2b4`, `e54135b`)
1. **Removed `test_` prefix from 101 test functions** across 5 new test files:
   - `manifest_tests.rs` — 45 fns
   - `bundle_tests.rs` — 12 fns
   - `deps_collector_tests.rs` — 16 fns
   - `license_tests.rs` — 15 fns
   - `pyoxidizer_tests.rs` — 13 fns
2. **Fixed import ordering in 2 files**:
   - `bundle_tests.rs` — `use auroraview_pack` was before `use std::fs`
   - `deps_collector_tests.rs` — `use auroraview_pack` was before `use std::path::PathBuf`
3. **Removed stale inline `use auroraview_pack::BundleBuilder`** from `bundle_is_empty_check` (already imported at top)

### Code Review Findings
- **signal_tests.rs**: import ordering correct (std first); no `test_` prefix violations — CLEAN
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — still pending dedicated deps round

### Metrics
- `test_` prefix violations removed: 101 fns (5 files)
- Import ordering violations fixed: 2
- Clippy warnings: 0 / Ruff warnings: 0
- `unsafe impl Send/Sync`: 2 (WebViewProxy only, unchanged)
- `#[allow(dead_code)]`: ~95 (structural, unchanged)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check`: PASS (0 warnings)
- `git push origin auto-improve`: PASS

## 2026-04-02 20:10 — Round 17

### Branch: `auto-improve` (HEAD: `bcf04ba`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 4 test batch commits since Round 16:
  - `03f4f3a` — cli args_tests (45) + assets assets_tests (28)
  - `e0bee90` — browser error_tests (29)
  - `c4af0d3` — dcc error_tests (22)
  - `14821eb` — pack error_tests (50)
  - `cf9e351` — testing unit_tests (78)

### Actions Taken (Commits: `234dd4d`, `bcf04ba`)
1. **Removed `test_` prefix from 97 test functions** across 3 files:
   - `crates/auroraview-assets/tests/assets_tests.rs` — 19 fns; `#[test]` → `#[rstest]`
   - `crates/auroraview-cli/tests/args_tests.rs` — 56 fns; `#[test]` → `#[rstest]`
   - `crates/auroraview-browser/tests/error_tests.rs` — 22 fns; `#[test]` → `#[rstest]`
2. **Fixed import ordering** in `crates/auroraview-pack/tests/error_tests.rs` — `use std::path::PathBuf` was after `use auroraview_pack`; moved to top
3. **Clean files** (no violations): `dcc/error_tests.rs` (22 fns, no `test_` prefix), `testing/unit_tests.rs` (78 fns, already clean)

### Code Review Findings (Iterate Agent's 5 commits)
- **`testing/unit_tests.rs`**: Already follows project convention (no `test_` prefix, `#[rstest]` throughout) — CLEAN
- **`dcc/error_tests.rs`**: Clean — no `test_` prefix violations, no std imports needed
- **`pack/error_tests.rs`**: Import ordering fixed this round
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — still pending dedicated deps round

### Metrics
- `test_` prefix violations removed: 97 fns (3 files)
- Import ordering violations fixed: 1
- Clippy warnings: 0 / Ruff warnings: 0
- `unsafe impl Send/Sync`: 2 (WebViewProxy only, unchanged)
- `#[allow(dead_code)]`: ~95 (structural, unchanged)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check python/ examples/ scripts/ gallery/`: PASS (0 warnings)
- `git push origin auto-improve`: PASS

## 2026-04-03 23:16 — Round 18

### Branch: `auto-improve` (HEAD: `6a3258a`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 4 test batch commits since Round 17:
  - `5e48aab` — plugin-core request_tests (28) + router_tests (18)
  - `2a6823f` — plugin-core error_tests (41) + scope_tests (32)
  - `fdb5d2f` — plugins/browser types_tests (27), fs operations_tests (51), bookmarks_tests (36) + history_tests (40)

### Actions Taken (Commit: `6a3258a`)
1. **Removed `test_` prefix from 53 test functions** across 2 files:
   - `crates/auroraview-browser/tests/config_tests.rs` — 24 fns
   - `crates/auroraview-browser/tests/theme_tests.rs` — 29 fns
2. **Fixed import ordering in `config_tests.rs`** — `auroraview_browser::devtools::DockSide` and `auroraview_browser::{...}` reordered per Rust convention (alphabetical within external group)
3. **Extracted inline `use std::sync::atomic::*`** from 2 functions in `router_tests.rs` → file top; reordered: `std::*` → `auroraview_*` → `serde_json`
4. **Fixed import ordering in `webview_thread_safety_tests.rs`** — `std::sync::Arc` + `std::thread` moved before `auroraview_dcc` and `rstest`

### Code Review Findings (Iterate Agent's commits)
- **plugin-core tests**: `error_tests.rs`, `scope_tests.rs`, `types_tests.rs` — clean, no `test_` prefix, no std ordering issues
- **fs/operations_tests.rs**: clean — `std::fs as std_fs` aliased at top, no ordering violations
- **browser bookmarks_tests.rs + history_tests.rs**: clean, no violations

### Metrics
- `test_` prefix violations removed: 53 fns (2 files)
- Import ordering violations fixed: 2 files (router_tests, webview_thread_safety_tests)
- Clippy warnings: 0 / Ruff warnings: 0

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 warnings)
- `uv run ruff check python/ examples/ scripts/ gallery/`: PASS (0 warnings)
- `git push origin auto-improve`: PASS [cleanup-done]

## 2026-04-03 07:32 — Round 20

### Branch: `auto-improve` (HEAD: `2dd7f11`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 7 test expansion commits since Round 19:
  - `7859981` — notifications/bookmarks/history expanded
  - `ef7ee89` — download_tests 49→88
  - `65a2a92` — settings_tests 45→81
  - `dc9816b` — shell_tests 38→68
  - `5c46f3c` — service_discovery_tests 55→79
  - `250fd75` — dcc/error_tests 22→50
  - `a542b1c` — chore(iteration): done

### Actions Taken (Commit: `2dd7f11`)
1. **Removed `test_` prefix from 257 test functions** across 6 files + fixed `#[test]` → `#[rstest]`:
   - `bookmark_tests.rs` — 46 fns
   - `download_tests.rs` — 50 fns
   - `history_tests.rs` — 50+3 fns
   - `notification_tests.rs` — 17+2 fns
   - `shell_tests.rs` — 8 fn name renames (already `#[rstest]`)
   - `settings_tests.rs` — 68 `#[test]` → `#[rstest]`
2. **Fixed import ordering** in `service_discovery_tests.rs` — `use std::collections::HashMap` + `use std::net::TcpListener` moved before `auroraview_core::` and `rstest`
3. **Fixed import ordering** in `notification_tests.rs` — `use std::sync::*` moved before `auroraview_notifications::` and `rstest`
4. **Extracted inline `use std::sync::Arc` + `use std::thread`** to file top in `bookmark_tests.rs`, `download_tests.rs`, `history_tests.rs`, `notification_tests.rs`
5. **Added `std::thread` to top-level imports** in `settings_tests.rs`
6. **dcc/error_tests.rs**: Already clean (no `test_` prefix, no std imports) — no changes needed

### Code Review Findings (Iterate Agent's commits)
- **`service_discovery_tests.rs`**: std imports were after external crates — fixed this round
- **`settings_tests.rs`**: used `#[test]` throughout instead of `#[rstest]` — fixed this round
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — still pending dedicated deps round

### Metrics
- `test_` prefix violations removed: 257 fns (6 files)
- `#[test]` → `#[rstest]` replacements: 68 (settings_tests.rs)
- Import ordering violations fixed: 2 files
- Inline use statements extracted: 4 files
- Clippy warnings: 0 / Ruff warnings: 0

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 warnings)
- `uv run ruff check python/ examples/ scripts/ gallery/`: PASS (0 warnings)
- `git push origin auto-improve`: PASS [cleanup-done]

## 2026-04-02 01:15 — Round 12

### Branch: `auto-improve` (HEAD: `9057610`)

### Baseline
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- Iterate Agent committed 5 commits since Round 11: 50 pack builder tests, 33 protect protector tests, fix dcc ListenerId pub use

### Actions Taken (Commits: `154d239`, `9057610`)
1. **Fixed import ordering in `protector_tests.rs`** — `std::fs` / `std::path::Path` were after `auroraview_protect` and `tempfile`; moved to top per project convention

### Code Review Findings (Iterate Agent's 5 commits)
- **`a7636f6` test(pack)**: 50 builder tests — naming clean, no `test_` prefix, no std imports needed; split use paths acceptable
- **`985b1d4` test(protect)**: 33 protector tests — naming clean, import order fixed this round
- **`496c69d` fix(dcc)**: removed stale `ListenerId` pub re-export — correct minimal fix
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — pending dedicated deps round

### Metrics
- Import ordering violations fixed: 1
- Clippy warnings: 0 / Ruff warnings: 0
- `unsafe impl Send/Sync`: 2 (WebViewProxy only, unchanged)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 new warnings)
- `uv run ruff check`: PASS (0 warnings)

## 2026-04-03 21:09 — Round 22

### Branch: `auto-improve` (commits: `cda56b8`, `e0b5c74`)

### Baseline
- **Workspace cargo test**: initially blocked by missing built frontend pages (`loading/index.html`, `error/index.html`) in source checkout
- **Workspace cargo clippy**: PASS before new fixes
- **Python gates**: `mypy` blocked by toolchain/config mismatch (`python_version = 3.7` unsupported by installed mypy); `pytest` exposed a mix of stale assertions and optional-runtime assumptions

### Actions Taken
1. **Added safe source-checkout fallbacks in `crates/auroraview-core/src/assets.rs`**
   - `get_loading_html()` now falls back to embedded loading HTML with `root` marker
   - `get_error_html()` now falls back to an embedded error shell compatible with `build_error_page()` injection
2. **Hardened process exit behavior in `crates/auroraview-plugins/src/process.rs`**
   - Added a dedicated exit watcher so `process:exit` is not lost due to stdout timing races
   - Made `check_exit()` idempotent so concurrent paths do not double-emit
3. **Kept root Rust smoke tests compatible with source checkouts**
   - Included existing `src/lib.rs` smoke-test cleanup in the round commit so built frontend assets are no longer assumed during root library tests
4. **Cleaned Python integration tests that had drifted from current behavior**
   - Replaced brittle fixed sleeps with polling in `test_gallery_plugin_api.py`
   - Updated optional Playwright/Qt tests to skip cleanly when browser binaries or `qtpy` are absent
   - Updated stale integration assertions in `test_standalone_runner.py` and `test_integration.py` to match current fallback/lifecycle behavior
5. **Logged remaining follow-ups to `CLEANUP_TODO.md`**
   - browser-controller page still lacks a source-checkout fallback
   - mypy toolchain no longer supports the project’s Python 3.7 target setting

### Verification
- `cargo test -p auroraview-core --test assets_tests -- --nocapture`: PASS
- `cargo test --test protocol_handlers_integration -- --nocapture`: PASS
- `cargo test --test standalone_integration -- --nocapture`: PASS
- `cargo test --workspace -- --nocapture`: PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS
- `uv run maturin develop`: PASS (refreshed local `python/auroraview/_core.pyd`)
- `uv run pytest tests/python/integration/test_gallery_plugin_api.py -q`: PASS
- `uv run pytest tests/python/integration/test_standalone_runner.py -q`: PASS
- `uv run pytest tests/python/integration/test_integration.py -q`: PASS
- `uv run mypy python tests`: FAIL before analysis because installed mypy rejects `python_version = 3.7`
- `uv run pytest tests/python/unit tests/python/integration --maxfail=1 -q`: still not fully green; latest remaining blocker is `browser-controller/index.html` missing in source checkout, which causes `Browser._get_browser_html()` to panic

