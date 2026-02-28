---
name: mcp-tester
description: |
  This skill helps test and debug the local development version of AuroraView MCP Server.
  It guides AI to configure mcp.json, start the local MCP server, run tests, and troubleshoot issues.
  Use this skill when developing or debugging the auroraview-mcp package.
---

# AuroraView MCP Tester

This skill guides testing and debugging of the local AuroraView MCP Server development version.

## When to Use

- Testing local MCP server changes before committing
- Debugging MCP tool implementations
- Verifying MCP server functionality
- Troubleshooting connection issues
- Running integration tests with AI assistants

## Prerequisites

Before testing, ensure:

1. **Local environment is set up**:
   ```bash
   cd packages/auroraview-mcp
   vx uv sync
   ```

2. **AuroraView instance is running** (for connection tests):
   ```bash
   # Start Gallery for testing
   just gallery
   # Or run a specific example
   just example hello_world
   ```

## Step 1: Configure mcp.json for Local Development

Update `~/.codebuddy/mcp.json` to use the local development version:

```json
{
  "mcpServers": {
    "auroraview-dev": {
      "command": "vx",
      "args": [
        "uv",
        "--directory",
        "C:/Users/hallo/Documents/augment-projects/dcc_webview/packages/auroraview-mcp",
        "run",
        "auroraview-mcp"
      ],
      "env": {
        "AURORAVIEW_DEFAULT_PORT": "9222",
        "AURORAVIEW_PROJECT_ROOT": "C:/Users/hallo/Documents/augment-projects/dcc_webview",
        "AURORAVIEW_EXAMPLES_DIR": "C:/Users/hallo/Documents/augment-projects/dcc_webview/examples",
        "AURORAVIEW_GALLERY_DIR": "C:/Users/hallo/Documents/augment-projects/dcc_webview/gallery"
      }
    }
  }
}
```

### Configuration Options

| Variable | Description | Default |
|----------|-------------|---------|
| `AURORAVIEW_DEFAULT_PORT` | Default CDP port for WebView | `9222` |
| `AURORAVIEW_SCAN_PORTS` | Ports to scan (comma-separated) | `9222,9223,9224,9225` |
| `AURORAVIEW_PROJECT_ROOT` | Path to project root | Auto-detected |
| `AURORAVIEW_EXAMPLES_DIR` | Path to examples directory | Auto-detected |
| `AURORAVIEW_GALLERY_DIR` | Path to Gallery directory | Auto-detected |
| `AURORAVIEW_DCC_MODE` | DCC mode (maya, blender, etc.) | None |

## Step 2: Test MCP Tools

### Discovery Tools

Test instance discovery:

```
# Using AI assistant with auroraview-dev MCP
User: Discover all running AuroraView instances

AI: [Call discover_instances]
→ Expected: List of instances with port, pid, title info
```

### Connection Tools

Test connection to instances:

```
User: Connect to AuroraView on port 9222

AI: [Call connect(port=9222)]
→ Expected: Connection success message with instance details
```

### Gallery Tools

Test Gallery management:

```
User: Start the Gallery application

AI: [Call run_gallery(port=9222)]
→ Expected: Gallery started with PID and port info

User: Get Gallery status

AI: [Call get_gallery_status]
→ Expected: Running status, port, PID

User: Stop Gallery

AI: [Call stop_gallery]
→ Expected: Gallery stopped confirmation
```

### Sample Tools

Test sample management:

```
User: List available samples

AI: [Call get_samples]
→ Expected: List of samples with names and descriptions

User: Run the hello_world sample

AI: [Call run_sample(name="hello_world")]
→ Expected: Sample started with PID

User: Get sample source code

AI: [Call get_sample_source(name="hello_world")]
→ Expected: Python source code of the sample
```

### UI Tools

Test UI automation:

```
User: Take a screenshot

AI: [Call take_screenshot]
→ Expected: Base64 encoded screenshot

User: Get page snapshot

AI: [Call get_snapshot]
→ Expected: Accessibility tree structure

User: Click on element with selector "#button"

AI: [Call click(selector="#button")]
→ Expected: Click success confirmation
```

### API Tools

Test Python API calls:

```
User: List available API methods

AI: [Call list_api_methods]
→ Expected: List of methods exposed via auroraview.api

User: Call API method echo with message "test"

AI: [Call call_api(method="echo", kwargs={"message": "test"})]
→ Expected: API response
```

### Debug Tools

Test debugging capabilities:

```
User: Get console logs

AI: [Call get_console_logs]
→ Expected: List of console messages

User: Get backend status

AI: [Call get_backend_status]
→ Expected: Backend status info
```

## Step 3: Debug Common Issues

### Issue: MCP Server Not Starting

**Symptoms**: AI cannot connect to auroraview-dev MCP

**Debug Steps**:
1. Check if uv is installed: `vx uv --version`
2. Check if dependencies are synced: `cd packages/auroraview-mcp && vx uv sync`
3. Test manual start: `cd packages/auroraview-mcp && vx uv run auroraview-mcp`
4. Check for Python errors in output

### Issue: Cannot Discover Instances

**Symptoms**: `discover_instances` returns empty list

**Debug Steps**:
1. Ensure AuroraView is running with CDP enabled
2. Check port configuration: `AURORAVIEW_SCAN_PORTS`
3. Test CDP endpoint manually: `curl http://localhost:9222/json`
4. Check firewall settings

### Issue: Connection Fails

**Symptoms**: `connect` tool returns error

**Debug Steps**:
1. Verify instance is running: `discover_instances` first
2. Check WebSocket connection: `wscat -c ws://localhost:9222/devtools/page/xxx`
3. Check for multiple instances on same port
4. Verify CDP version compatibility

### Issue: API Calls Fail

**Symptoms**: `call_api` returns error

**Debug Steps**:
1. Verify connection is established
2. Check if API method exists: `list_api_methods`
3. Verify method signature and parameters
4. Check console logs for Python errors

## Step 4: Run Automated Tests

### Unit Tests

```bash
# Run all MCP tests
just mcp-test

# Run specific test file
cd packages/auroraview-mcp
vx uv run pytest tests/test_tools.py -v

# Run with coverage
just mcp-test-cov
```

### Debug Client

```bash
# Run built-in debug client
just mcp-debug

# Interactive mode
just mcp-debug-interactive

# Test specific functionality
cd packages/auroraview-mcp
vx uv run python scripts/debug_client.py --test discover
vx uv run python scripts/debug_client.py --test connect --port 9222
```

### MCP Inspector

```bash
# Launch visual debugger
just mcp-inspector

# Opens http://localhost:5173 for interactive testing
```

## Step 5: Verify Changes

After making changes to MCP server code:

1. **Lint code**:
   ```bash
   just mcp-lint
   ```

2. **Format code**:
   ```bash
   just mcp-format
   ```

3. **Run tests**:
   ```bash
   just mcp-test
   ```

4. **Full CI check**:
   ```bash
   just mcp-ci
   ```

## Available justfile Commands

| Command | Description |
|---------|-------------|
| `just mcp-dev` | Start MCP server in development mode |
| `just mcp-debug` | Run built-in debug client tests |
| `just mcp-debug-interactive` | Start interactive debug mode |
| `just mcp-inspector` | Launch MCP Inspector web UI |
| `just mcp-test` | Run unit tests |
| `just mcp-test-cov` | Run tests with coverage |
| `just mcp-lint` | Lint code with ruff |
| `just mcp-format` | Format code with ruff |
| `just mcp-build` | Build package |
| `just mcp-ci` | Run full CI check |

## MCP Tool Reference

### Discovery Tools
- `discover_instances` - Discover all running AuroraView instances
- `connect` - Connect to an AuroraView instance
- `disconnect` - Disconnect from current instance
- `list_dcc_instances` - Discover instances in DCC environments

### Page Tools
- `list_pages` - List all pages in connected instance
- `select_page` - Select a page by ID or URL pattern
- `get_page_info` - Get detailed page information
- `reload_page` - Reload the current page

### API Tools
- `call_api` - Call AuroraView Python API method
- `list_api_methods` - List available API methods
- `emit_event` - Emit event to frontend

### UI Tools
- `take_screenshot` - Capture page or element screenshot
- `get_snapshot` - Get accessibility tree snapshot
- `click` - Click on an element
- `fill` - Fill input with text
- `evaluate` - Execute JavaScript code
- `hover` - Hover over an element

### Gallery Tools
- `run_gallery` - Start the Gallery application
- `stop_gallery` - Stop the running Gallery
- `get_gallery_status` - Get Gallery running status
- `get_samples` - List available samples
- `run_sample` - Run a sample application
- `stop_sample` - Stop a running sample
- `get_sample_source` - Get sample source code
- `list_processes` - List running sample processes
- `stop_all_samples` - Stop all running samples
- `get_project_info` - Get AuroraView project info

### Debug Tools
- `get_console_logs` - Get console log messages
- `get_network_requests` - Get network request history
- `get_backend_status` - Get Python backend status
- `get_memory_info` - Get memory usage info
- `clear_console` - Clear console logs

### DCC Tools
- `get_dcc_context` - Get current DCC environment context
- `execute_dcc_command` - Execute DCC native commands
- `sync_selection` - Synchronize selection between DCC and WebView
- `set_dcc_selection` - Set selection in DCC application
- `get_dcc_scene_info` - Get detailed scene information
- `get_dcc_timeline` - Get timeline/animation information
- `set_dcc_frame` - Set current frame in DCC application

## Best Practices

1. **Always start with discovery**: Run `discover_instances` before attempting to connect
2. **Check connection status**: Verify connection before running UI or API tools
3. **Use appropriate ports**: Gallery uses 9222 by default, DCC instances may use 9223+
4. **Monitor console logs**: Use `get_console_logs` to debug issues
5. **Clean up resources**: Stop samples and disconnect when done testing

## Issue Handling Workflow

When encountering issues during MCP testing, follow this structured workflow:

### Step 1: Summarize the Issue

Provide a clear summary including:

- **Issue Type**: Bug / Missing Feature / Design Flaw / Performance / Documentation
- **Severity**: Critical / High / Medium / Low
- **Affected Components**: List affected files/modules
- **Root Cause Analysis**: Brief explanation of why the issue occurs
- **Impact**: What functionality is affected

Example summary:
```
## Issue Summary

**Type**: Missing Feature
**Severity**: High
**Component**: `python/auroraview/core/webview.py`

**Problem**: Python WebView class does not expose `remote_debugging_port` parameter,
preventing CDP connection for MCP testing.

**Root Cause**: Rust layer supports the parameter but Python binding does not pass it through.

**Impact**: MCP cannot connect to Gallery/samples for testing.
```

### Step 2: Wait for Developer Decision

After summarizing, **STOP and ask the developer** which action to take:

```
## Recommended Actions

Please choose how to proceed:

1. **Fix in current branch** - Quick fix, low risk, directly related to current work
2. **Create new branch** - Larger change, needs isolation, may affect other features
3. **Create GitHub Issue** - Track for later, not blocking current work
4. **Create RFC proposal** - Significant design change, needs discussion

Your choice: [1/2/3/4]
```

### Step 3: Execute Based on Decision

#### Option 1: Fix in Current Branch

```bash
# Verify current branch
git branch --show-current

# Make the fix
# ... code changes ...

# Commit with descriptive message
git add -A
git commit -m "fix: add remote_debugging_port to Python WebView"
```

#### Option 2: Create New Branch (Based on Remote Main)

**IMPORTANT**: Always create new branches based on `origin/main` to ensure a clean base.

```bash
# 1. Fetch latest remote main
git fetch origin main

# 2. Create and switch to new branch based on origin/main
git checkout -b fix/webview-cdp-port origin/main

# 3. Make the fix
# ... code changes ...

# 4. Commit and push
git add -A
git commit -m "fix: add remote_debugging_port to Python WebView"
git push -u origin fix/webview-cdp-port

# 5. Create PR (optional)
gh pr create --title "fix: add remote_debugging_port to Python WebView" --body "..."
```

**Why base on `origin/main`?**
- Ensures clean history without unrelated changes from other branches
- Makes PR review easier (only shows your changes)
- Avoids merge conflicts from stale local branches

#### Option 3: Create GitHub Issue

Use the GitHub MCP tool to create an issue:

```
# Call github MCP tool: issue_write
{
  "owner": "user",
  "repo": "dcc_webview",
  "title": "[Bug] Python WebView missing remote_debugging_port parameter",
  "body": "## Problem\n\n...\n\n## Expected Behavior\n\n...\n\n## Proposed Solution\n\n...",
  "labels": ["bug", "python", "mcp"]
}
```

#### Option 4: Create RFC Proposal

Use the `rfc-creator` skill to create a formal proposal:

```
# Invoke rfc-creator skill
# RFC should include:
# - Problem statement
# - Proposed solution
# - Alternatives considered
# - Implementation plan
# - Breaking changes (if any)
```

### Decision Guidelines

| Situation | Recommended Action |
|-----------|-------------------|
| Simple bug, < 50 lines change | Fix in current branch |
| Bug affecting multiple files | Create new branch |
| Feature request, not urgent | Create GitHub Issue |
| API design change | Create RFC |
| Breaking change | Create RFC + Issue |
| Performance issue | Create Issue first, then branch |
| Documentation gap | Fix in current branch |

### Issue Templates

#### Bug Report Template
```markdown
## Bug Report

**Title**: [Component] Brief description

**Environment**:
- OS: Windows 11
- Python: 3.11
- Branch: feat/mcp-server-implementation

**Steps to Reproduce**:
1. ...
2. ...

**Expected Behavior**: ...

**Actual Behavior**: ...

**Root Cause**: ...

**Proposed Fix**: ...
```

#### Feature Request Template
```markdown
## Feature Request

**Title**: [Component] Brief description

**Use Case**: Why is this needed?

**Proposed Solution**: How should it work?

**Alternatives Considered**: Other approaches?

**Implementation Notes**: Technical details
```

### Tracking Multiple Issues

When multiple issues are discovered during testing:

1. Create a summary table:

| # | Issue | Severity | Action | Status |
|---|-------|----------|--------|--------|
| 1 | CDP port not exposed | High | Fix in branch | Pending |
| 2 | Missing error handling | Medium | GitHub Issue | Created |
| 3 | API design inconsistency | Low | RFC | Pending |

2. Address issues in priority order (Critical > High > Medium > Low)

3. Update status after each action
