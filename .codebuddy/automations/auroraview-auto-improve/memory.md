# AuroraView Auto-Improve Memory

## Last Execution: 2026-04-01 09:06 (UTC+8)

### Branch Status
- Branch: `auto-improve` (rebased on `origin/main`, 3 commits ahead this round)
- Pushed: Yes (commits `eb4ac87`, `e600002`, `daee0bc`)

### Completed in This Iteration

1. **TabEvent builder API** (commit `eb4ac87`)
   - `events.rs`: added `pin_tab`, `mute_tab`, `reorder_tab`, `duplicate_tab`, `toggle_devtools`, `open_devtools`, `close_devtools` builder methods
   - `close_tab` and `activate_tab` updated to `impl Into<TabId>` — avoids mandatory `.to_string()` at call sites
   - `tab_tests.rs`: 4 original + 13 new builder tests = 17 total
   - `window_manager.rs` (DCC): `get_info(&str)` now directly queries DashMap (no intermediate `to_string()`)
   - `view_manager.rs` (extensions): minor comment clarification in `get_view_by_type`

2. **IpcRouter split_last optimization + test expansion** (commit `e600002`)
   - `ipc/handler.rs`: `handle_event` now uses `split_last()` to move value into final listener
   - `ipc_tests.rs`: 8 original + 7 new tests (on event, multiple listeners, invoke, unknown type, invalid JSON, methods listing) = 15 total

### Cumulative Progress (across iterations)

**CSP Security (COMPLETE)**
**Inject JS/CSS (COMPLETE)**
**Hot Reload (COMPLETE):** HTML mode + URL-mode polling
**Signal/Clone Optimization (COMPLETE):** Signal emit, EventBus, RuntimeManager, ScriptInjector, TabState, TabManager
**Doctest Fixes (COMPLETE):** 20/20
**CLI AtomicBool (COMPLETE)**
**SAFETY Audit (COMPLETE)**
**Lock Migration (COMPLETE)**
**Safety & Code Quality (COMPLETE)**
**Pack Crate (COMPLETE)**
**AI Agent (COMPLETE)**
**Plugins/Extensions API (COMPLETE)**
**Browser DevTools (COMPLETE)**
**DCC Integration (MAJOR)**
**Thread Safety (COMPLETE)**
**Error handling audit (COMPLETE)**
**Documentation (COMPLETE)**

**Test counts (updated):**
- auroraview-browser/tab_tests: 17 (was 4)
- auroraview-dcc/ipc_tests: 15 (was 8)
- All other crates unchanged from previous

**Clippy status:** Zero warnings across all modified crates

### Known Pre-existing Issues
- `auroraview-core` assets_tests fail (need `vx just assets-build`)
- `auroraview` 2 test_desktop_module/test_webview_submodules fail (assets issue)
- GitHub: 48 Dependabot vulnerabilities (transitive deps)
- `cargo audit`: 22 allowed warnings (gtk3 bindings from wry)

### Next Iteration Targets (Priority Order)
1. **Signal handler Fn(&T)**: Add `connect_ref(Fn(&T))` alongside existing `connect(Fn(T))` — zero-clone alternative without breaking API
2. **auroraview-settings tests**: Review and expand test coverage
3. **auroraview-cli packed_tests**: Look for any coverage gaps in the packed webview path
4. **DCC ipc on() deduplication**: Consider allowing unsubscribe from specific on() listeners (currently no way to remove)
5. **Performance profiling**: Profile WebView startup path for sub-150ms target
