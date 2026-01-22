# Logging and Error Handling Guide

This guide explains how to properly use logging and error handling in AuroraView applications across all three layers: Python, Rust, and JavaScript.

## Overview

AuroraView uses a unified logging and error handling architecture that works consistently across:
- **Python** (backend API handlers)
- **Rust** (WebView host and plugin system)
- **JavaScript** (frontend SDK)

### Key Principles

1. **Use structured logging** - Log messages with level prefixes that can be parsed
2. **Distinguish logs from errors** - Not all stderr output is an error
3. **Packed mode awareness** - In packed mode, stdout is reserved for JSON-RPC

## Python Logging

### Using the Logger

Always use `get_logger()` from `auroraview.utils.logging` instead of `print()`:

```python
from auroraview.utils.logging import get_logger

logger = get_logger(__name__)

# Different log levels
logger.debug("Detailed debug information")
logger.info("General information")
logger.warning("Something might be wrong")
logger.error("Something went wrong")
logger.critical("Fatal error")
```

### Why Not `print()`?

In **packed mode** (when your app is bundled as an executable):
- `stdout` is used for JSON-RPC communication with the Rust host
- `stderr` is monitored for errors
- Using `print(..., file=sys.stderr)` for debug logs will trigger `backend_error` events

The `get_logger()` function automatically:
- Detects packed mode via `AURORAVIEW_PACKED` environment variable
- Uses structured log format: `[LEVEL:module] message`
- Only ERROR and CRITICAL levels trigger backend errors in the frontend

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AURORAVIEW_LOG_LEVEL` | Log level (DEBUG, INFO, WARNING, ERROR, CRITICAL) | WARNING |
| `AURORAVIEW_LOG_ENABLED` | Enable/disable logging (1/0) | 1 |
| `AURORAVIEW_LOG_VERBOSE` | Enable verbose debug output | 0 |
| `AURORAVIEW_PACKED` | Set automatically in packed mode | - |

### Configuration

```python
from auroraview.utils.logging import configure_logging

# Enable debug logging for development
configure_logging(level="DEBUG", verbose=True)

# Use JSON format for structured logging
configure_logging(level="INFO", use_json=True)
```

## Rust Layer Behavior

The Rust layer reads Python's stderr and applies intelligent filtering:

```
[DEBUG:module] message  → Logged locally, NOT sent to frontend
[INFO:module] message   → Logged locally, NOT sent to frontend
[WARNING:module] message → Logged locally, NOT sent to frontend
[ERROR:module] message  → Logged + sent as backend_error
[CRITICAL:module] message → Logged + sent as backend_error
```

Messages without a recognized level prefix are scanned for error keywords:
- `error`, `exception`, `traceback`, `fatal` → Treated as errors
- Other messages → Logged locally only

## JavaScript Frontend

### Handling Backend Errors

The frontend SDK distinguishes between fatal and non-fatal errors:

```typescript
import { createAuroraView } from '@auroraview/sdk';

const av = createAuroraView();

// Configure error handling behavior
av.setConfig({
  callTimeoutMs: 30000,           // RPC timeout
  backendFailFast: true,          // Fail pending calls on fatal errors
  failFastSeverity: 'fatal',      // Only fail on fatal errors
});

// Listen for backend errors
av.on('backend_error', (detail) => {
  console.warn('Backend error:', detail.message);
  // Non-fatal errors don't clear pending calls
});
```

### Error Codes

The SDK provides standard error codes matching Rust's `PluginErrorCode`:

```typescript
import type { PluginErrorCode } from '@auroraview/sdk';

try {
  await av.call('api.some_method');
} catch (error) {
  if (error.code === 'TIMEOUT') {
    // Handle timeout
  } else if (error.code === 'FILE_NOT_FOUND') {
    // Handle file not found
  }
}
```

### Fatal vs Non-Fatal Errors

Fatal errors that trigger fail-fast:
- `process has exited`
- `backend ready timeout`
- `stdout closed`
- `connection lost`
- `fatal error`
- `crash`

Non-fatal errors (logged but don't clear pending calls):
- Debug/info log output
- Warnings
- Transient errors

## Best Practices

### 1. Always Use Structured Logging

```python
# ✅ Good
logger = get_logger(__name__)
logger.info(f"Processing file: {filename}")

# ❌ Bad
print(f"Processing file: {filename}", file=sys.stderr)
```

### 2. Choose Appropriate Log Levels

```python
# DEBUG: Detailed information for debugging
logger.debug(f"Variable state: {vars}")

# INFO: General operational information
logger.info(f"Started processing {count} items")

# WARNING: Something unexpected but not critical
logger.warning(f"Config not found, using defaults")

# ERROR: Something failed but app can continue
logger.error(f"Failed to save file: {e}")

# CRITICAL: App cannot continue
logger.critical(f"Database connection failed: {e}")
```

### 3. Include Context in Error Messages

```python
try:
    result = process_data(data)
except Exception as e:
    logger.error(f"Failed to process data (id={data.id}): {e}")
    raise
```

### 4. Use Try/Except for API Handlers

```python
@webview.bind_call("api.process_file")
def process_file(path: str) -> dict:
    logger = get_logger(__name__)
    try:
        logger.info(f"Processing: {path}")
        result = do_processing(path)
        return {"ok": True, "result": result}
    except FileNotFoundError:
        logger.warning(f"File not found: {path}")
        return {"ok": False, "error": "File not found"}
    except Exception as e:
        logger.error(f"Processing failed: {e}")
        return {"ok": False, "error": str(e)}
```

## Debugging Tips

### Enable Verbose Logging

```bash
# Set environment variable before running
export AURORAVIEW_LOG_LEVEL=DEBUG
export AURORAVIEW_LOG_VERBOSE=1
```

### Check Log Output

In packed mode, logs go to stderr with structured prefixes:
```
[DEBUG:dependency_api] Starting installation
[INFO:dependency_api] Found 3 packages to install
[WARNING:dependency_api] Package xyz is deprecated
[ERROR:dependency_api] Installation failed: network error
```

### Frontend Console

Open DevTools (F12) to see:
- `[AuroraView] Backend error received: ...` for non-fatal errors
- `[AuroraView] Fatal backend error: ...` for fatal errors
- `[AuroraView] No pending call for id: ...` if there's an IPC sync issue

## Migration Guide

If you're migrating from `print()` to `logging`:

```python
# Before
import sys
print(f"[MyModule] Processing {item}", file=sys.stderr)

# After
from auroraview.utils.logging import get_logger
logger = get_logger(__name__)
logger.info(f"Processing {item}")
```

The new approach:
- Works correctly in both normal and packed modes
- Allows log level filtering
- Doesn't trigger false backend_error events
- Supports JSON structured logging
