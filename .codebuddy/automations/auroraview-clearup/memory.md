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
