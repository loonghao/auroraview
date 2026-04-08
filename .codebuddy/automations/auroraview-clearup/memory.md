# AuroraView Cleanup Agent Memory

## 2026-04-08 Round 43

### Branch: `auto-improve` (HEAD: `8c8d383`)

### Baseline
- **Cargo check**: PASS ✅
- **Cargo clippy**: PASS (0 warnings) ✅
- **Ruff**: PASS (0 warnings) ✅
- **Tests**: All passing

### Actions Taken

**Commit 1: `e86fc01` - [cleanup] docs: update deprecated run_standalone references in API documentation [cleanup-done]**
1. `docs/api/index.md`: Updated `run_standalone` section from "Deprecated" to clearly mark as "Removed"
2. `docs/zh/api/index.md`: Same update for Chinese documentation
3. Changed wording to explicitly state the alias is no longer available and points to `run_desktop`
4. Net change: +6 lines, -4 lines (more informative documentation)

**Commit 2: `376b982` - [cleanup] rust-code: improve DWMSBT_TRANSIENTWINDOW documentation with MSFT reference [cleanup-done]**
1. `vibrancy.rs`: Added Microsoft Docs URL to DWM_SYSTEMBACKDROP_TYPE constants section
2. Improved DWMSBT_TRANSIENTWINDOW doc comment with clearer description of its reserved purpose
3. Net change: +3 lines, -1 line

**Commit 3: `8c8d383` - [cleanup] docs: update CLEANUP_TODO.md with Round 43 achievements [cleanup-done]**
1. Updated dead_code count trend with Round 43 data
2. Added new resolved item for run_standalone documentation cleanup
3. Documented DWMSBT_TRANSIENTWINDOW documentation improvement

### Additional Discovery

#### Confirmed Clean Areas (unchanged from Round 42)
- **Clippy**: 0 warnings after cleanup
- **Ruff**: 0 warnings across all Python code; import sorting clean; formatting clean
- **Docs**: No stale API references (run_standalone now correctly marked as removed)
- **Tests**: No `#[test]\n#[ignore]` instances; all skip reasons are valid
- **Dependencies**: No duplicates in cargo tree; workspace deps well-organized
- **Build system**: Only build.rs files are legitimate (CLI resource embedding + workspace hack)
- **Production std::sync usage**: Zero Mutex/RwLock — fully migrated to parking_lot
- **TODO(cleanup) markers in source code**: 0
- **unused_imports/unused_variables/unused_mut allow annotations**: 0

#### Metrics Summary
| Metric | Count | Status |
|--------|-------|--------|
| `#[allow(dead_code)]` | 2 | Stable (both justified) |
| `#[allow(clippy::*)` | ~11 | All type_complexity or platform-specific, justified |
| `TODO/FIXME/WARN` markers | 123+ | Active development indicators, not cleanup targets |

#### Remaining 2 `#[allow(dead_code)]` Annotations:
1. `json_tests.rs`: test-local struct (normal pattern, safe to keep)
2. `vibrancy.rs::DWMSBT_TRANSIENTWINDOW`: Win11 DWM API constant with MSFT doc reference (justified reserve)

### Quality Gate
- Workspace `cargo check`: PASS ✅
- Workspace `cargo clippy --all-targets`: PASS (0 warnings) ✅
- `uv run ruff check python/ examples/ scripts/ gallery/`: PASS ✅
- Pushed to remote: YES ✅

### Focus for Next Rounds
1. **Dependency security audit** — 38 vulnerabilities (from Dependabot push output) — HIGH PRIORITY but requires careful version pinning
2. **Large module assessment** — `window_style.rs` (1056 lines), `assets.rs` (699 lines) candidates for splitting
3. **IpcRouter deduplication** — DCC + Desktop share ~90% identical code (Medium priority, needs coordination)
4. **Consider evaluating test sleep-based assertions** — `metrics_tests.rs` uses thread::sleep for timing
5. **Python mypy compatibility** — Pin a mypy version supporting Python 3.7 targets

---

## Previous Rounds Summary (Rounds 1-42)

### Cumulative Achievements:
- **Stale files removed**: build_cli.py, $null, pr_body.md, .gitcommitmsg, llms*.txt
- **Dependencies cleaned**: active-win-pos-rs (re-added), hyper, hyper-util
- **Unsafe impls removed**: 4 unnecessary (TabManager x2, SignalRegistry, EventBus)
- **Import ordering fixes**: ~30+ files across crates/
- **`test_` prefix removals**: ~900+ functions
- **Clippy fixes**: ~20+ errors/warnings across multiple rounds
- **Dead code removal**: extract_resources, print_targets, ScopedTimer, legacy_embedded,
  find_free_port_with_timeout, emit_event, MOBILE_BOOKMARKS, dynamic_response,
  ExtensionViewHandle.hwnd, HEADER_SIZE, HEADER_SIZE_UNUSED, and more
- **Deprecated API migration**: run_standalone->run_desktop in examples, allow_new_window removal
- **ServiceInfo placeholder fixed**: now raises ImportError instead of silent pass
- **Duplicate imports removed**: __init__.py redundant submodule import
- **parking_lot migrations**: core production code + plugins.rs + window_style.rs + telemetry/python.rs completed
- **`#[allow]` attribute precision improvements** (Round 40): cfg-gated unused_variables/unused_mut
- **LockOrderGuard Debug impl** (Round 41): name field now used, dead_code removed
- **All rounds passed quality gates**: cargo check ✅, clippy ✅, ruff ✅, tests ✅
- **Merge conflict resolution**: Round 38 resolved 6 test file conflicts from main branch integration
- **Compilation fix** (Round 42): window_style.rs mismatched brace from prior round parking_lot migration
- **Documentation updates** (Round 43): run_standalone marked as removed in EN/ZH API docs, DWMSBT_TRANSIENTWINDOW improved with MSFT reference
