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

### `#[allow(dead_code)]` annotations (~95 total)
- **Status**: Structural / Feature-gated
- **Reason**: Most are BOM API stubs awaiting implementation or feature-gated code.
  Not actionable until features are implemented.

### `crates/auroraview-pack/tests/metrics_tests.rs` — sleep-based timing assertions
- **Status**: TODO (logged Round 21)
- **Reason**: The suite relies on `thread::sleep(Duration::from_millis(...))`, which makes it slower
  and more timing-sensitive than necessary.
- **Action**: Introduce a controllable timing helper or loosen the test strategy to avoid wall-clock sleeps.

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
