# AuroraView Auto-Improve Memory

## Last Execution: 2026-04-10 09:48 (UTC+8)

### Branch Status
- Branch: `auto-improve`
- Remote sync: pushed 2 commits in this session (R17 complete)
- Workspace clean: Yes

### Completed in This Iteration (Round 17)

**Round 17: MCP HTTP Transport Activation + AG-UI SSE Endpoint**

| Item | Status |
|------|--------|
| Replace warp with axum (aligns with rmcp tower API) | DONE |
| McpRunner::start() binds real TCP port via axum | DONE |
| MCP Streamable HTTP transport at /mcp (StreamableHttpService) | DONE |
| AG-UI SSE endpoint at /agui/events with run_id filter | DONE |
| AguiEvent enum (14 event types, full AG-UI protocol) | DONE |
| AguiBus broadcast channel (clone-cheap, 256-cap) | DONE |
| agui_tests: 24 tests | DONE |
| http_transport_tests: 9 integration tests (real TCP) | DONE |
| server_tests: 18 tests (unchanged) | DONE |
| Total: 51 tests (was 18 in R16) | DONE |
| Clippy clean, zero warnings | DONE |

**Key implementation facts R17:**
- `StreamableHttpService` is a Tower service — use `axum::Router::nest_service("/mcp", svc)`
- axum 0.8 requires explicit features: `http1`, `tokio`, `query` for `axum::extract::Query`
- `tokio-stream` with `sync` feature provides `BroadcastStream` for broadcast::Receiver → futures::Stream
- AG-UI SSE endpoint uses `axum::response::Sse` + `KeepAlive` (15s interval)
- `CancellationToken` (from tokio-util) wires graceful shutdown from McpRunner::stop()
- McpError::Io wraps std::io::Error for TCP bind failures

### Next Iteration Targets (Priority Order)

1. **Python bindings for McpServer**: Expose `McpServer(port).start()` / `.stop()` via PyO3
   - Create `auroraview-mcp-py` feature or extend `python/` bindings
   - Target: `from auroraview import McpServer; server = McpServer(7890); server.start()`
2. **McpRunner::emit_agui integration with MCP tools**: Wire tool execution to AG-UI events
   - When `screenshot` tool is called → emit `ToolCallStart/End` via AguiBus
   - Let dcc-mcp-core see real-time progress in Web UI
3. **AG-UI streaming test with actual events**: Verify SSE stream delivers events to reqwest reader
4. **auroraview-mcp-py integration test**: Verify Python API works from a real Python process

### Known Pre-existing Issues (from prior iterations, NOT blocking)
- `auroraview-core` assets_tests fail in CI (need `vx just assets-build`)
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
**Core config_tests comprehensive (COMPLETE):** 45 tests R9
**Core metrics_tests comprehensive (COMPLETE)**
**Core templates_tests comprehensive (COMPLETE)**
**Core signals_tests comprehensive (COMPLETE)**
**Core protocol_tests comprehensive (COMPLETE)**
**Desktop config_tests comprehensive (COMPLETE):** 47 tests R9
**Desktop ipc_tests comprehensive (COMPLETE)**
**Pack metrics_tests comprehensive (COMPLETE):** 47 tests R9
**Pack overlay_tests comprehensive (COMPLETE)**
**Pack lib_tests (COMPLETE):** 42 tests R10
**Pack bundle_tests comprehensive (COMPLETE)**
**Pack license_tests comprehensive (COMPLETE):** 45 tests R9
**Pack deps_collector/FileHashCache (COMPLETE):** 42 tests R9
**Pack pyoxidizer_tests comprehensive (COMPLETE)**
**Signals signal_tests comprehensive (COMPLETE):** 61 tests
**Pack manifest_tests comprehensive (COMPLETE):** 45 tests
**Core error_tests (COMPLETE):** 52 tests
**Desktop error_tests + window_manager_tests (COMPLETE):** 42 + 42 = 84 tests R10
**Pack python_standalone_tests expansion (COMPLETE):** 39 → 69 tests R12
**Desktop tray_tests + event_loop_tests (COMPLETE):** 47 + 55 = 102 tests R9
**Pack error_tests (COMPLETE):** 50 tests
**DCC error_tests (COMPLETE):** 48 tests R9
**Testing unit_tests (COMPLETE):** 78 tests
**Browser error_tests (COMPLETE):** 62 tests R9
**CLI args_tests (COMPLETE):** 45 tests
**CLI lib_tests (COMPLETE):** 50 tests R10
**Assets assets_tests (COMPLETE):** 28 tests
**PluginCore error_tests + scope_tests (COMPLETE):** 41 + 32 = 73 tests
**PluginCore request_tests + router_tests (COMPLETE):** 28 + 18 = 46 tests
**PluginCore types_tests (COMPLETE):** 27 tests
**PluginFs operations_tests (COMPLETE):** 51 tests
**Browser bookmarks_tests expansion (COMPLETE):** 7 → 52 tests R9 → 51 R15
**Browser history_tests expansion (COMPLETE):** 12 → 53 tests R9 → 50 R15
**DCC webview_thread_safety_tests expansion (COMPLETE):** 9 → 45 tests
**Browser config_tests expansion (COMPLETE):** 5 → 44 tests → 50 R15
**Browser theme_tests expansion (COMPLETE):** 6 → 48 tests R9 → 50 R15
**Core cli_tests expansion (COMPLETE):** 9 → 44 tests R9 → 50 R15
**Plugins router_tests expansion (COMPLETE):** 18 → 53 tests R12
**Browser devtools_tests expansion (COMPLETE):** 18 → 41 tests → 50 R15
**DCC window_manager_tests expansion (COMPLETE):** 8 → 55 tests R9
**Core ipc_tests expansion (COMPLETE):** 8 → 68 tests
**Core protocol_tests expansion (COMPLETE):** ~37 → 59 tests
**Core thread_safety_tests expansion (COMPLETE):** 19 → 39 tests
**DCC ipc_tests expansion (COMPLETE):** 15 → 45 tests R9
**Pack hooks_tests expansion (COMPLETE):** 33 → 47 tests R9
**Telemetry guard_tests expansion (COMPLETE):** 33 → 44 tests R9
**Telemetry config_tests expansion (COMPLETE):** 33 → 44 tests R9
**Browser tab_tests expansion (COMPLETE):** 17 → 50 tests R9 → 55 R15
**DCC error_tests expansion R9 (COMPLETE):** 22 → 48 tests
**Telemetry sentry_tests expansion (COMPLETE):** 41 → 51 tests R10
**Telemetry telemetry_init_tests expansion (COMPLETE):** 37 → 53 tests R10
**Telemetry span_ext_tests expansion (COMPLETE):** 26 → 57 tests R10
**Telemetry error_tests expansion (COMPLETE):** 30 → 44 tests R10
**Core backend_tests expansion (COMPLETE):** 37 → 58 tests R10
**Core events_tests expansion (COMPLETE):** 37 → 52 tests R10
**Core lifecycle_tests expansion (COMPLETE):** 41 → 67 tests R10
**Core window_tests expansion (COMPLETE):** 26 → 46 tests R10
**Core window_style_tests expansion (COMPLETE):** 25 → 47 tests R10 → 51 R15
**Core menu_tests expansion (COMPLETE):** 33 → 53 tests R10
**Desktop window_manager_tests expansion (COMPLETE):** 30 → 42 tests R10
**Desktop error_tests expansion (COMPLETE):** 29 → 42 tests R10
**Core icon_tests expansion (COMPLETE):** 32 → 66 tests R11
**Core icon_converter_tests expansion (COMPLETE):** 37 → 74 tests R11
**CLI ipc_integration_tests expansion (COMPLETE):** 5 → 25 → 37 active tests R11/R13
**CLI packed_tests expansion (COMPLETE):** 31 → 58 tests R11
**Core port_tests flaky fix (COMPLETE):** R11
**Plugins clipboard_tests expansion (COMPLETE):** 27 → 55 tests R11
**Plugins fs_plugin_tests expansion (COMPLETE):** 25 → 41 tests R12
**Plugins router_tests expansion R12 (COMPLETE):** 39 → 53 tests
**Pack python_standalone_tests expansion R12 (COMPLETE):** 39 → 69 tests
**Plugins scope_tests expansion (COMPLETE):** 30 → 66 tests R12
**Plugins types_tests expansion (COMPLETE):** 31 → 84 tests R12
**AI Agent agent_tests expansion (COMPLETE):** 31 → 72 tests R12
**Pack pyoxidizer_tests expansion (COMPLETE):** 30 → 52 tests R13
**Core cleanup_tests expansion (COMPLETE):** 30 → 47 tests R13
**Plugins dialog_tests expansion (COMPLETE):** 35 → 70 tests R13
**Plugins process_tests expansion (COMPLETE):** 36 → 64 tests R13
**Core json_tests expansion (COMPLETE):** 35 → 56 tests R13
**Core id_generator_tests expansion (COMPLETE):** 37 → 46 tests R13
**Core port_tests expansion (COMPLETE):** 30 → 47 tests R14
**Plugins request_tests expansion (COMPLETE):** 32 → 62 tests R14
**Protect obfuscator_tests expansion (COMPLETE):** 33 → 50 tests R14
**Pack bundle_tests expansion (COMPLETE):** 36 → 46 tests R14
**Protect protector_tests expansion (COMPLETE):** 33 → 52 tests R14
**Core templates_tests expansion (COMPLETE):** 36 → 44 tests R14 → 50 R15
**Pack overlay_tests expansion (COMPLETE):** 37 → 49 tests R14
**Core metrics_tests R15 (COMPLETE):** 39 → 51 tests
**Pack metrics_tests R15 (COMPLETE):** 41 → 54 tests
**Telemetry metrics_tests R15 (COMPLETE):** 37 → 50 tests
**CLI args_tests R15 (COMPLETE):** 40 → 50 tests
**CLI cli_tests R15 (COMPLETE):** 32 → 50 tests
**CLI lib_tests R15 (COMPLETE):** 38 → 50 tests
**Core utils_tests R15 (COMPLETE):** 43 → 50 tests
**Core message_processor_tests R15 (COMPLETE):** 43 → 50 tests
**Core signals_tests R15 (COMPLETE):** 43 → 50 tests
**MCP Server auroraview-mcp crate R16 (COMPLETE):** 18 tests, 9 tools, rmcp 1.3
**MCP HTTP Transport + AG-UI SSE R17 (COMPLETE):** 51 tests, real TCP binding, axum+rmcp
