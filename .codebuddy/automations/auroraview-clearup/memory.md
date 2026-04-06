# AuroraView Cleanup Agent Memory

## 2026-04-07 04:05 — Round ~37

### Branch: `auto-improve` (HEAD: `7420ad4`)

### Baseline
- **Cargo check**: PASS
- **Cargo clippy**: PASS (0 warnings)
- **Ruff**: PASS (0 warnings, import sorting clean)
- **Tests**: All passing (from prior round state)

### Actions Taken

**Commit 1: `58c0178` - [cleanup] remove unused deps and temp artifacts**
1. Removed unused dependencies from root `Cargo.toml`:
   - `hyper = "1.8"` — no direct `use hyper` in src/ or crates/; transitively pulled by warp
   - `hyper-util = "0.1"` — same as above; no direct usage anywhere
2. Deleted temporary AI-generated artifacts from root:
   - `llms-full.txt` (24.13 KB) — LLM model list artifact
   - `llms.txt` (5.01 KB) — LLM model list artifact
   - `test_output.txt` (576.27 KB) — already gitignored; deleted from disk only
3. Net change: -2 dependencies, -796 lines (29KB transient files removed)
4. Verified: cargo check PASS after removal (hyper still available via warp transitive dep)

**Commit 2: `7420ad4` - [cleanup] docs: update dead_code count**
5. Updated CLEANUP_TODO.md:
   - Corrected `#[allow(dead_code)]` count from "~5" to accurate "~57" across workspace
   - Added categorization: BOM API预留 (~14 in webview_inner.rs alone), standalone mode fields,
     platform-conditional code, IPC internal structs, legacy wry-backend methods
   - Recorded Round 37 achievements in resolved items list

### Full Scan Results (Round ~37 Discovery Phase)

#### Confirmed Clean Areas
- **Python bare `except:` + `pass`**: 0 instances in project Python code
  - Only 3 found in `crates/auroraview-protect/src/runtime_gen.rs` (Rust-generated obfuscation code, intentional)
- **Import ordering (I001)**: All clean across python/, examples/, scripts/, gallery/
- **Empty/near-empty Python files**:
  - `tests/python/__init__.py`: 2 lines (comment only) — standard test package marker, keep
- **TODO/FIXME/HACK/XXX in source code**:
  - 8 TODOs (mostly in docs/CLEANUP_TODO + 2 in source)
  - 13 FIXMEs (mostly in docs + protocol_handlers.rs URL examples)
  - 28 HACKs (all Cargo.toml workspace version alignment comments or justfile hakari references)
  - Source code is clean of actionable TODO/HACK markers
- **pyproject.toml**: Minimal and clean — only `typing_extensions` runtime dep for py3.7
- **ctrlc, muda**: Both confirmed used (ctrlc::try_set_handler in desktop.rs, muda re-exported via tray-icon)
- **simd-json**: Used in auroraview-core/src/json.rs, properly transitively depended

### Metrics
- Files changed: 5 (Cargo.toml, Cargo.lock, llms-full.txt, llms.txt, CLEANUP_TODO.md)
- Net lines: -782 deletions (796 - 14 doc update)
- Dependencies removed: 2 (hyper, hyper-util)
- Transient files deleted: ~605KB total
- Clippy warnings: 0 / Ruff warnings: 0
- Total `#[allow(dead_code)]`: ~57 (structural/BOM API/platform-gated — low priority)

### Quality Gate
- Workspace `cargo check`: PASS
- Workspace `cargo clippy --all-targets`: PASS (0 warnings)
- `uv run ruff check python/ examples/ scripts/ gallery/ --select I001`: PASS
- `git push origin auto-improve`: Success [cleanup-done]

### Security Notes
- GitHub Dependabot reports 48 vulnerabilities (1 critical, 25 high) — still pending dedicated deps round

### Findings for Future Rounds
- **GitHub dep vulnerabilities**: 48 (1 critical, 25 high) — pending dedicated deps round
- **`#[allow(dead_code)]`**: ~57 remaining (BOM API 预留占多数, standalone mode fields, feature-gated code)
- **Legacy aliases still exported**: `run_standalone`, `run_tab_browser`, `create_for_dcc()` — backward compat
- **Rust events.rs deprecated callbacks**: 4 DEPRECATED navigation callbacks — candidates for v0.6+
- **CLEANUP_TODO high-priority item**: packed/webview.rs parking_lot migration (~16 RwLock sites, 1700+ line file)
- **DCC + Desktop IpcRouter duplication**: ~90% code overlap — extract ipc-common crate candidate

---

## Previous Rounds Summary (Rounds 1-36)

### Cumulative Achievements:
- **Stale files removed**: build_cli.py, $null, pr_body.md, .gitcommitmsg, llms*.txt
- **Dependencies cleaned**: active-win-pos-rs, hyper, hyper-util
- **Unsafe impls removed**: 4 unnecessary (TabManager x2, SignalRegistry, EventBus)
- **Import ordering fixes**: ~30+ files across crates/
- **`test_` prefix removals**: ~900+ functions
- **Clippy fixes**: ~20+ errors/warnings across multiple rounds
- **Dead code removal**: extract_resources, print_targets, ScopedTimer, legacy_embedded,
  find_free_port_with_timeout, emit_event, MOBILE_BOOKMARKS, and more
- **Deprecated API migration**: run_standalone→run_desktop in examples, allow_new_window removal
- **ServiceInfo placeholder fixed**: now raises ImportError instead of silent pass
- **Duplicate imports removed**: __init__.py redundant submodule import
- **parking_lot migrations**: core production code + plugins.rs completed
- **All rounds passed quality gates**: cargo check ✅, clippy ✅, ruff ✅, tests ✅
