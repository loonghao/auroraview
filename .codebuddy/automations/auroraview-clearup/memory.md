# AuroraView Cleanup Agent Memory

## 2026-04-05 05:14 — Round ~26

### Branch: `auto-improve` (HEAD: `ace6278`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings)
- **Tests**: All passing
- Iterate Agent committed ~5 test/expansion commits since Round 25

### Actions Taken

**Commit: `ace6278` - [cleanup] deprecated-api**
1. Updated `examples/local_assets_example.py`:
   - `run_standalone` → `run_desktop` (import and call sites, including HTML docstring examples)
   - This was the only example file still using the legacy alias
2. Removed deprecated `allow_new_window=True` from `gallery/main.py`:
   - Already had `new_window_mode="child_webview"` set, so purely redundant deprecated param
   - Updated log message to remove reference to removed parameter
3. Fixed `ServiceInfo` placeholder class in `__init__.py`:
   - Changed from empty `pass` body to raising `ImportError`
   - Now consistent with `ServiceDiscovery` placeholder behavior (prevents silent no-op instances)
4. Removed duplicate submodule import in `__init__.py` line 420:
   - `from . import core, integration, ui, utils` was duplicated with lines 453-456
   - The first import (line 420) only served for creating backward-compat aliases at lines 425-432
   - Kept the second import (line 453) which is the organized access version with comments

### Code Review Findings (Iterate Agent's new commits)
- No new structural issues found this round
- Deprecated API usage is now fully cleaned from examples/ and gallery/

### Metrics
- Files changed: 3 (examples/local_assets_example.py, gallery/main.py, python/__init__.py)
- Net lines: +3 insertions, -6 deletions (-3 net)
- Deprecated API references removed: 4 (run_standalone x3, allow_new_window x1)
- Clippy warnings: 0 / Ruff warnings: 0
- Total `#[allow(dead_code)]`: ~26 remaining (structural/feature-gated — low priority)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy`: PASS (0 warnings)
- `uv run ruff check python/ examples/ scripts/ gallery/`: PASS (0 warnings)
- `git push origin auto-improve`: Success [cleanup-done]

### Security Notes
- GitHub Dependabot reports 48 vulnerabilities (1 critical, 25 high) — still pending dedicated deps round

### Findings for Future Rounds
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — pending dedicated deps round
- **`#[allow(dead_code)]`**: ~26 remaining (mostly BOM API预留, standalone mode fields, feature-gated code)
- **Legacy aliases still exported**: `run_standalone`, `run_tab_browser`, `create_for_dcc()` — kept for backward compat but could add stronger deprecation warnings
- **Rust events.rs deprecated callbacks**: 4 DEPRECATED navigation callbacks (`on_navigation_started/completed/failed/load_progress`) — candidates for removal in v0.6+
- **`ThemeColors::PartialEq`**: Only compares 3 of 14 fields — potential logic issue worth documenting or fixing

---

## Previous Rounds Summary (Rounds 1-25)

### Cumulative Achievements:
- **Stale files removed**: build_cli.py, $null, pr_body.md, .gitcommitmsg, etc.
- **Dependencies cleaned**: active-win-pos-rs
- **Unsafe impls removed**: 4 unnecessary (TabManager ×2, SignalRegistry, EventBus)
- **Import ordering fixes**: ~30+ files across crates/
- **`test_` prefix removals**: ~900+ functions
- **Clippy fixes**: ~20+ errors/warnings across multiple rounds
- **Dead code removal**: extract_resources, print_targets, ScopedTimer, legacy_embedded, find_free_port_with_timeout, emit_event, MOBILE_BOOKMARKS, and more
- **Deprecated API migration**: run_standalone→run_desktop in examples, allow_new_window removal in gallery
- **ServiceInfo placeholder fixed**: now raises ImportError instead of silent pass
- **Duplicate imports removed**: __init__.py redundant submodule import
- **All rounds passed quality gates**: cargo check ✅, clippy ✅, ruff ✅, tests ✅
