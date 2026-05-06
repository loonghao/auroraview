# AuroraView Auto-Improve Memory #

## Session Summary - 2026-05-06 (Iteration #145):

### ✅ Completed (Iteration #145):
1. **Fixed compilation errors in `auroraview-mcp` crate**:
   - Added missing imports to `cdp/connect.rs`: `use std::sync::Arc;`, `use tokio::sync::Mutex;`, `use tracing::error;`, `use super::{CdpClient, CdpClientInner, CdpError};`
   - Added missing `serde_json::Value` import to `cdp/page.rs`, `cdp/dom.rs`, `cdp/runtime.rs`, `cdp/network.rs`
   - Fixed method call ambiguity in `cdp/runtime.rs`: changed `.cloned()` to `.map(|v| v.clone())` (twice)
   - Removed unused import in `cdp/page.rs`: `warn`

2. **Verified `auroraview-mcp` crate compilation and tests**:
   - `cargo check -p auroraview-mcp` passed ✅
   - 80 unit tests passed ✅

3. **Committed and pushed changes**:
   - Commit: `dd03986` - `fix(mcp): resolve compilation errors in cdp module`
   - Pushed to `auto-improve` branch ✅

4. **MCP Server implementation status**:
   - ✅ Implemented tools: `screenshot`, `eval_js`, `load_url`, `send_event`
   - ✅ mDNS broadcast: implemented (`mdns.rs`)
   - ✅ AG-UI SSE endpoint: implemented (`runner.rs` - `GET /agui/events`)
   - ❌ Placeholder tools (not yet implemented): `get_hwnd`, `list_webviews`, `create_webview`, `close_webview`
   - ❌ Missing tool: `load_html`

---

## Next Iteration Plan (Iteration #146):

### Priority 1: Implement missing MCP tools
1. **Implement `load_html` tool**:
   - Add `load_html` method to `cdp/page.rs` (use CDP `Page.setDocumentContent` or data URL)
   - Add `LoadHtmlParams` to `mcp_server/params.rs`
   - Add `load_html` tool handler to `mcp_server/mod.rs`

2. **Implement or update placeholder tools**:
   - `get_hwnd` - requires AuroraView core CDP extension API (Q3 2026 target)
   - `list_webviews` - requires AuroraView core CDP extension API
   - `create_webview` - requires AuroraView core CDP extension API
   - `close_webview` - requires AuroraView core CDP extension API
   - Note: These are blocked by AuroraView core changes, keep as placeholders

### Priority 2: Continue with large file refactoring
Files still needing refactoring (from memory.md #144):
1. `crates/auroraview-pack/src/manifest.rs` - 1690 lines ❌
   - **Note**: `crates/auroraview-pack/src/manifest/` directory exists (untracked), might be already refactored
2. `src/webview/backend/native.rs` - 1623 lines
3. `src/webview/webview_inner.rs` - 1589 lines
4. `crates/auroraview-plugins/src/browser_bridge.rs` - 1502 lines
5. `src/webview/tab_manager.rs` - 1341 lines
6. `crates/auroraview-plugins/src/process.rs` - 1190 lines
7. `crates/auroraview-cli/src/packed/backend.rs` - 1141 lines
8. `src/webview/config.rs` - 1113 lines

### Priority 3: Code Quality
- [ ] Run `cargo clippy --workspace` and fix any new warnings
- [ ] Run full test suite (`cargo test --workspace`)
- [ ] Check for any remaining large files

---

## Checklist for Next Iteration:
### Mandatory Requirement (MUST DO):
- [ ] **Implement `load_html` tool** (if not blocked by core changes)
- [ ] **Check if `crates/auroraview-pack/src/manifest.rs` has been refactored** (check `manifest/` directory)
- [ ] **Continue with next large file refactoring**

### Security:
- [ ] Investigate Dependabot alerts (43 vulnerabilities reported by GitHub)
- [ ] Identify vulnerability sources (Rust/JavaScript/GitHub Actions)
- [ ] Update vulnerable dependencies or document why not fixing

### Code quality:
- [ ] Fix any clippy warnings
- [ ] Fix any fmt issues
- [ ] Run full test suite

### Push:
- [ ] Commit changes with descriptive message
- [ ] Push to `auto-improve`

---

## Important Reminders:
- **MANDATORY REQUIREMENT**: Single file must not exceed 1000 lines
- **Current status**: `auroraview-mcp` crate ✅ (compiles and tests pass)
- **NEXT**: Implement `load_html` tool OR continue large file refactoring
- **This task does NOT stop until ALL large files are refactored AND all MCP tools are implemented**

---

## GitHub Security Alerts (2026-05-06):
- 43 vulnerabilities found on `loonghao/auroraview` default branch
- 18 high, 24 moderate, 1 low
- URL: https://github.com/loonghao/auroraview/security/dependabot
- **TODO**: Investigate and fix in future iterations
