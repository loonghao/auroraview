# AuroraView Auto-Improve Memory

## Last Execution: 2026-04-09 20:28 (UTC+8)

### Branch Status
- Branch: `auto-improve`
- Remote sync: already based on origin/main (v0.4.19), pushed 4 new commits + 1 iteration marker
- Workspace clean: Yes (memory.md modified only)

### Completed in This Iteration (Round 7)

**Round 7a: browser/config + dcc/config + telemetry/sentry/guard + core/templates** (commit `a2aaa50`)
- browser/config_tests: 19 → 68 tests (+49): BrowserFeatures serde, Theme variants, Send+Sync, DockSide serde roundtrip, CDP port matrix, builder overwrite, clone independence
- dcc/config_tests: 26 → 77 tests (+51): DccType detect env, Send+Sync, data_dir PathBuf, unicode, url variants, partial eq matrix
- telemetry/sentry_tests: 20 → 28 tests (+8): Send+Sync, default sample rates, multiple configs independence, clone independence
- telemetry/guard_tests: 22 → 33 tests (+11): determinism, idempotency, enable-only/disable-only loops
- core/templates_tests: 25 → 36 tests (+11): emit nonempty output, multiple renders consistent, LoadUrl edge cases, ApiReg max methods

**Round 7b: pack/overlay + cli/lib_tests** (commit `4edb08a`)
- pack/overlay_tests: 18 → 28 tests (+10): data_new empty assets, add_and_count, unicode asset name, large asset, write_level_1, config size roundtrip, same content same hash
- cli/lib_tests: 18 → 38 tests (+20): ICON_PNG_BYTES static/len, load_window_icon idempotent, normalize_url localhost variants, no_double_slash, absolute_url unchanged scheme

**Round 7c: telemetry_init + id_generator** (commit `331923f`)
- telemetry_init_tests: 25 → 37 tests (+12): custom log_level, service_name, metrics/traces flags, serde for_testing, Send+Sync, clone independence
- id_generator_tests: 26 → 35 tests (+9): Send+Sync, next_string format, next_with_prefix format, mixed calls, current before next, unicode prefix, count matches current

### Key Learnings
- `DccType::detect()` uses env vars in priority order: MAYA → HFS → NUKE → BLENDER → ADSK_3DSMAX → UE_ROOT
- `BrowserFeatures` directly exported from `auroraview_browser` crate (no sub-module path needed)
- `telemetry_init_tests` uses process-global state; `enabled=false` config is safe for repeated init/drop cycles
- `IdGenerator` shares counter across `next()`, `next_string()`, `next_with_prefix()` calls

### Next Iteration Targets (Priority Order)

1. **core/window_tests.rs** — 26 markers, check actual count
2. **plugins/fs_plugin_tests.rs** — 25 markers, likely ~25 actual
3. **plugins/core/router_tests.rs** — 27 markers
4. **plugins/clipboard_tests.rs** — 27 markers  
5. **pack/python_standalone_tests.rs** — 27 markers
6. **telemetry/span_ext_tests.rs** — 26 markers
7. **Any test file under 30 actual tests** across all crates

### Known Pre-existing Issues (from prior iterations, NOT blocking)
- `auroraview-core` assets_tests fail (need `vx just assets-build`)
- `auroraview` 2 test_desktop_module/test_webview_submodules fail (assets issue)
- GitHub: 39 Dependabot vulnerabilities (transitive deps)
- `cargo audit`: 22 allowed warnings (gtk3 bindings from wry)

## Cumulative Progress (across iterations)

**CSP Security (COMPLETE)**
**Inject JS/CSS (COMPLETE)**
**Hot Reload (COMPLETE)**
**Signal/Clone Optimization (COMPLETE)**
**Doctest Fixes (COMPLETE)**
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
**Signal connect_ref (COMPLETE)**
**DCC IpcRouter off() (COMPLETE)**
**Browser TabListenerMap (COMPLETE)**
**Extensions Runtime coverage (COMPLETE)**
**ExtensionHost coverage (COMPLETE)**
**Browser NavigationManager coverage (COMPLETE)**
**AI Agent session/message coverage (COMPLETE)**
**Protect crypto coverage (COMPLETE)**
**Protect config coverage (COMPLETE)**
**AI Agent actions/providers coverage (COMPLETE)**
**Protect RuntimeGenerator coverage (COMPLETE)**
**Telemetry concurrent metrics coverage (COMPLETE)**
**Protect obfuscator integration (COMPLETE)**
**AI Agent protocol deep coverage (COMPLETE)**
**DCC compile error fix (COMPLETE)**
**Protect Protector integration (COMPLETE)**
**Pack Builder system coverage (COMPLETE)**
**Pack packer/progress coverage (COMPLETE)**
**Telemetry is_initialized coverage (COMPLETE)**
**Core utils comprehensive coverage (COMPLETE)**
**Core json/port/id_generator comprehensive coverage (COMPLETE)**
**Pack HooksConfig coverage (COMPLETE)**
**Core bom_tests comprehensive (COMPLETE)**
**Core config_tests comprehensive (COMPLETE)**
**Core metrics_tests comprehensive (COMPLETE)**
**Core templates_tests comprehensive (COMPLETE)**
**Core signals_tests comprehensive (COMPLETE)**
**Core protocol_tests comprehensive (COMPLETE)**
**Desktop config_tests comprehensive (COMPLETE)**
**Desktop ipc_tests comprehensive (COMPLETE)**
**Pack metrics_tests comprehensive (COMPLETE)**
**Pack overlay_tests comprehensive (COMPLETE)**
**Pack lib_tests (COMPLETE)**
**Pack bundle_tests comprehensive (COMPLETE)**
**Pack license_tests comprehensive (COMPLETE)**
**Pack deps_collector/FileHashCache (COMPLETE)**
**Pack pyoxidizer_tests comprehensive (COMPLETE)**
**Signals signal_tests comprehensive (COMPLETE):** 61 tests
**Pack manifest_tests comprehensive (COMPLETE):** 45 tests
**Core error_tests (COMPLETE):** 52 tests
**Desktop error_tests + window_manager_tests (COMPLETE):** 13 + 30 = 43 tests
**Pack python_standalone_tests expansion (COMPLETE):** 13 → 39 tests
**Desktop tray_tests + event_loop_tests (COMPLETE):** 23 + 27 = 50 tests
**Pack error_tests (COMPLETE):** 50 tests
**DCC error_tests (COMPLETE):** 22 tests
**Testing unit_tests (COMPLETE):** 78 tests
**Browser error_tests (COMPLETE):** 29 tests
**CLI args_tests (COMPLETE):** 45 tests
**Assets assets_tests (COMPLETE):** 28 tests
**PluginCore error_tests + scope_tests (COMPLETE):** 41 + 32 = 73 tests
**PluginCore request_tests + router_tests (COMPLETE):** 28 + 18 = 46 tests
**PluginCore types_tests (COMPLETE):** 27 tests
**PluginFs operations_tests (COMPLETE):** 51 tests
**Browser bookmarks_tests expansion (COMPLETE):** 7 → 36 tests
**Browser history_tests expansion (COMPLETE):** 12 → 40 tests
**DCC webview_thread_safety_tests expansion (COMPLETE):** 9 → 45 tests
**Browser config_tests expansion (COMPLETE):** 5 → 39 tests
**Browser theme_tests expansion (COMPLETE):** 6 → 33 tests
**Core cli_tests expansion (COMPLETE):** 9 → 41 tests
**Plugins router_tests expansion (COMPLETE):** 18 → 39 tests
**Browser devtools_tests expansion (COMPLETE):** 18 → 51 tests
**DCC window_manager_tests expansion (COMPLETE):** 8 → 43 tests
**Core ipc_tests expansion (COMPLETE):** 8 → 68 tests
**Core protocol_tests expansion (COMPLETE):** ~37 → 59 tests
**Core thread_safety_tests expansion (COMPLETE):** 19 → 39 tests
**DCC ipc_tests expansion (COMPLETE):** 15 → 31 tests
**Plugins fs_plugin_tests expansion (COMPLETE):** 15 → 25 tests
**Browser tab_tests expansion (COMPLETE):** 17 → 34 tests
**Browser navigation_tests expansion (COMPLETE):** 36 → 61 tests
**Plugins scope_tests expansion (COMPLETE):** 15 → 47 tests
**DCC config_tests expansion (COMPLETE):** 11 → 50 tests
**Notifications notification_tests expansion (COMPLETE):** 35 → 70 tests
**Bookmarks bookmark_tests expansion (COMPLETE):** 40 → 69 tests
**History history_tests expansion (COMPLETE):** 45 → 81 tests
**Downloads download_tests expansion (COMPLETE):** 49 → 88 tests
**Settings settings_tests expansion (COMPLETE):** 45 → 81 tests
**Plugins shell_tests expansion (COMPLETE):** 38 → 68 tests
**Core service_discovery_tests expansion (COMPLETE):** 55 → 79 tests
**DCC error_tests expansion (COMPLETE):** 22 → 50 tests
**AI Agent session_tests expansion (COMPLETE):** ~57 → 85 tests
**Telemetry span_ext_tests expansion (COMPLETE):** ~12 → 39 tests
**Protect crypto_tests expansion (COMPLETE):** 30 → 70 tests
**Plugins process_tests expansion (COMPLETE):** 13 → 44 tests
**Plugins clipboard_tests expansion (COMPLETE):** 4 → 27 tests
**Plugins dialog_tests expansion (COMPLETE):** 11 → 52 tests
**Plugins fs_types_tests expansion (COMPLETE):** 20 → 55 tests
**Plugins extensions_tests expansion (COMPLETE):** 31 → 56 tests
**Plugins types_tests expansion (COMPLETE):** 21 → 59 tests
**Core dom_tests expansion (COMPLETE):** 3 → 86 tests
**Core icon_tests expansion (COMPLETE):** 2 → 24 tests
**Core menu_tests expansion (COMPLETE):** 7 → 37 tests
**Core window_style_tests expansion (COMPLETE):** 2 → 20 tests
**AI Agent agent_tests expansion (COMPLETE):** 5 → 35 tests
**Core window_tests expansion (COMPLETE):** 5 → 19 tests
**Core vibrancy_tests expansion (COMPLETE):** 22 → 43 tests
**Core icon_converter_tests expansion (COMPLETE):** 7 → 42 tests
**Telemetry sentry_tests expansion (COMPLETE):** 10 → 20 tests
**Telemetry guard_tests expansion (COMPLETE):** 12 → 22 tests
**Protect bytecode_integration_test expansion (COMPLETE):** 3 → 20 tests
**Telemetry telemetry_init_tests expansion (COMPLETE):** 9 → 25 tests
**Core click_through_tests expansion (COMPLETE):** 13 → 38 tests
**Desktop error_tests expansion (COMPLETE):** 13 → 29 tests
**Pack lib_tests expansion (COMPLETE):** 5 → 13 tests
**Pack overlay_tests expansion (COMPLETE):** 8 → 18 tests
**Pack pyoxidizer_tests expansion (COMPLETE):** 13 → 30 tests
**Pack metrics_tests expansion (COMPLETE):** 13 → 28 tests
**Pack hooks_tests expansion (COMPLETE):** 19 → 33 tests
**Pack license_tests expansion (COMPLETE):** 15 → 30 tests
**Pack deps_collector_tests expansion (COMPLETE):** 16 → 28 tests
**Pack builder_tests expansion (COMPLETE)**: 59 tests
**Pack config/packer/progress expansion (COMPLETE)**: ~80 + ~55 + ~42 = ~177 new tests
**Core events_tests expansion (COMPLETE)**: 0 → 39 tests
**Core cleanup_tests expansion (COMPLETE)**: 0 → 12 tests
**Core lifecycle_tests (NEW, COMPLETE)**: 0 → 52 tests
**Core message_processor_tests (NEW, COMPLETE)**: 0 → 47 tests
**Core error_pages_tests (NEW, COMPLETE)**: 0 → 65 tests
**MCP ui/api/page/debug/telemetry tool tests (NEW, COMPLETE)**: 108 → 222 tests (+114)
**MCP resources/providers + discovery + connection extended (NEW, COMPLETE)**: 222 → 346 tests (+124)
**MCP total with new test files**: 346 → 756 tests (all passing)
**Tabs tab_tests expansion (COMPLETE)**: 51 → 123 tests (+72)
**Devtools devtools_tests expansion (COMPLETE)**: 65 → 106 tests (+41)
**Notifications notification_tests expansion (COMPLETE)**: 62 → 123 tests (+61)
**Bookmarks bookmark_tests expansion (COMPLETE)**: 60 → 102 tests (+42)
**Downloads download_tests expansion (COMPLETE)**: 73 → 117 tests (+44)
**History history_tests expansion (COMPLETE)**: 70 → 111 tests (+41)
**Desktop ipc_tests expansion (COMPLETE)**: 27 → 46 tests (+19)
**Desktop event_loop_tests expansion (COMPLETE)**: 27 → 42+ tests (+15)
**Core port_tests expansion (COMPLETE)**: 12 → 28 tests (+16)
**Core cleanup_tests expansion (COMPLETE)**: 12 → 28 tests (+16)
**Core templates_tests expansion (COMPLETE)**: 14 → 26 tests (+12)
**Core metrics_tests expansion (COMPLETE)**: 17 → 30 tests (+13)
**Browser error_tests expansion R4 (COMPLETE)**: 22 → 48 tests (+26)
**Core icon_tests expansion R4 (COMPLETE)**: 16 → 46 tests (+30)
**Core window_tests expansion R4 (COMPLETE)**: 17 → 31 tests (+14)
**DCC error_tests expansion R4 (COMPLETE)**: 13 → 29 tests (+16)
**AI Agent agent_tests expansion R4 (COMPLETE)**: 24 → 51 tests (+27)
**Pack bundle_tests expansion R4 (COMPLETE)**: 12 → 24 tests (+12)
**Desktop tray_tests expansion R4 (COMPLETE)**: 21 → 31 tests (+10)
**Pack lib_tests expansion R5 (COMPLETE)**: 3 → 18 tests (+15)
**CLI lib_tests expansion R5 (COMPLETE)**: 6 → 22 tests (+16)
**Core id_generator_tests expansion R5 (COMPLETE)**: 17 → 35 tests (+18)
**Telemetry error_tests expansion R5 (COMPLETE)**: 16 → 30 tests (+14)
**Core window_style_tests expansion R5 (COMPLETE)**: 17 → 32 tests (+15)
**CLI cli_tests expansion R5 (COMPLETE)**: 8 → 17 tests (+9)
**Plugins router_tests expansion R5 (COMPLETE)**: 18 → 30 tests (+12)
**Pack bundle_tests expansion R5 (COMPLETE)**: 24 → 36 tests (+12)
**Plugins request_tests expansion R6 (COMPLETE)**: 20 → 42 tests (+22)
**Plugins types_tests (core) expansion R6 (COMPLETE)**: 21 → 39 tests (+18)
**Plugins error_tests (core) expansion R6 (COMPLETE)**: 24 → 62 tests (+38)
**Telemetry config_tests expansion R6 (COMPLETE)**: 21 → 33 tests (+12)
**Protect bytecode_integration_test expansion R6 (COMPLETE)**: 20 → 32 tests (+12)
**Core port_tests expansion R6 (COMPLETE)**: 23 → 32 tests (+9)
**Core cleanup_tests expansion R6 (COMPLETE)**: 28 → 30 tests (+2)
**Protect config_tests expansion R6 (COMPLETE)**: 25 → 37 tests (+12)
**Desktop config_tests expansion R6 (COMPLETE)**: 22 → 35 tests (+13)
**Browser config_tests expansion R7 (COMPLETE)**: 19 → 68 tests (+49)
**DCC config_tests expansion R7 (COMPLETE)**: 26 → 77 tests (+51)
**Telemetry sentry_tests expansion R7 (COMPLETE)**: 20 → 28 tests (+8)
**Telemetry guard_tests expansion R7 (COMPLETE)**: 22 → 33 tests (+11)
**Core templates_tests expansion R7 (COMPLETE)**: 25 → 36 tests (+11)
**Pack overlay_tests expansion R7 (COMPLETE)**: 18 → 28 tests (+10)
**CLI lib_tests expansion R7 (COMPLETE)**: 18 → 38 tests (+20)
**Telemetry init_tests expansion R7 (COMPLETE)**: 25 → 37 tests (+12)
**Core id_generator_tests expansion R7 (COMPLETE)**: 26 → 35 tests (+9)
