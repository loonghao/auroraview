# AuroraView MCP Server Design Guidelines

## 1. API Response Format

All MCP-exposed APIs MUST return a standardized JSON envelope to ensure consistent error handling and client-side processing.

### Standard Envelope
```json
{
  "ok": boolean,
  "data": any,       // Present if ok is true
  "error": string    // Present if ok is false
}
```

### Variants (deprecated)
Some legacy APIs return additional fields at the top level. New APIs MUST use the `data` field.

- **Recommended**: `{"ok": true, "data": { "pid": 123, "mode": "pipe" }}`
- **Deprecated**: `{"ok": true, "pid": 123, "mode": "pipe"}` (Flat return)

## 2. Auto-Normalization in `bind_call`

**Key Feature**: `bind_call` automatically normalizes all return values to the standard response format. Developers can simply return data directly without manual wrapping.

### Supported Return Types

The `normalize()` function supports **any JSON-serializable value**, not just dictionaries:

```python
# Dict (most common)
return {"name": "Alice"}
# JS: {ok: true, data: {name: "Alice"}}

# List
return [1, 2, 3]
# JS: {ok: true, data: [1, 2, 3]}

# String
return "hello"
# JS: {ok: true, data: "hello"}

# Number
return 42
# JS: {ok: true, data: 42}

# Boolean
return True
# JS: {ok: true, data: true}

# None
return None
# JS: {ok: true, data: null}
```

### How It Works

1. **Plain return values** are wrapped as `{ok: true, data: <value>}`
2. **Already-standard responses** (with `ok` field) pass through unchanged
3. **Exceptions** are caught and converted to `{ok: false, error: <message>}`

### Examples

```python
# Method 1: Just return data (recommended for simple cases)
@view.bind_call("api.get_user")
def get_user(id: int):
    return {"name": "Alice", "id": id}
# JS receives: {ok: true, data: {name: "Alice", id: 123}}

# Method 2: Explicit response for error handling
@view.bind_call("api.delete_user")
def delete_user(id: int):
    if not user_exists(id):
        return {"ok": False, "error": "User not found"}
    delete(id)
    return {"ok": True, "data": {"deleted": id}}

# Method 3: Using ok()/err() helpers (cleanest)
from auroraview import ok, err

@view.bind_call("api.create_user")
def create_user(name: str):
    if not name:
        return err("Name is required")
    return ok({"id": 123, "name": name})
```

## 3. High-Performance JSON Serialization

AuroraView uses **Rust-powered JSON serialization** (via `simd-json`) for all IPC communication, providing 2-3x faster performance than Python's `json` module.

### Automatic Usage

All `bind_call` responses are automatically serialized using the Rust JSON encoder. No action needed from developers.

### Manual Usage

For custom serialization needs, use the exported functions:

```python
from auroraview import json_loads, json_dumps, json_dumps_bytes

# Parse JSON (2-3x faster than json.loads)
data = json_loads('{"name": "test", "value": 123}')

# Serialize to string (2-3x faster than json.dumps)
json_str = json_dumps({"name": "test", "value": 123})

# Serialize to bytes (zero-copy, most efficient for network)
json_bytes = json_dumps_bytes({"name": "test"})
```

### Benefits

- **No Python dependencies**: No need to install `orjson` or other third-party JSON libraries
- **SIMD acceleration**: Uses CPU vector instructions for maximum throughput
- **Zero-copy parsing**: Minimizes memory allocations
- **Consistent behavior**: Same serialization across Python and Rust layers

## 4. Response Utilities

AuroraView provides helper functions to create standardized responses. Import from `auroraview`:

```python
from auroraview import ok, err, wrap_response
```

### `ok(data)` - Success Response

```python
# Simple success
return ok({"name": "test", "version": "1.0"})
# Output: {"ok": true, "data": {"name": "test", "version": "1.0"}}

# List data
return ok([1, 2, 3])
# Output: {"ok": true, "data": [1, 2, 3]}

# Single value
return ok("hello")
# Output: {"ok": true, "data": "hello"}

# No data
return ok()
# Output: {"ok": true, "data": null}
```

### `err(message)` - Error Response

```python
return err("File not found")
# Output: {"ok": false, "error": "File not found"}

# With error code
return err("Invalid parameter", code="INVALID_PARAM")
# Output: {"ok": false, "error": "Invalid parameter", "code": "INVALID_PARAM"}
```

### `@wrap_response` - Auto-wrap Decorator

Automatically wraps function returns in standard format and catches exceptions:

```python
@wrap_response
def get_info():
    return {"name": "test"}  # Automatically wrapped
# Output: {"ok": true, "data": {"name": "test"}}

@wrap_response
def failing_func():
    raise ValueError("Something went wrong")
# Output: {"ok": false, "error": "Something went wrong"}
```

### Full Example

```python
from auroraview import ok, err

@view.bind_call("api.get_user")
def get_user(user_id: str = "") -> dict:
    if not user_id:
        return err("User ID is required")
    
    user = database.find_user(user_id)
    if not user:
        return err(f"User '{user_id}' not found")
    
    return ok({
        "id": user.id,
        "name": user.name,
        "email": user.email,
    })
```

## 5. Naming Conventions

- **Status Field**: Always use `ok` (boolean). Avoid `success`, `status`, or `is_valid`.
- **Data Field**: Use `data` for the main payload.
  - If returning a list: `{"ok": true, "data": [...]}`
  - If returning an object: `{"ok": true, "data": {...}}`
  - If returning a simple value: `{"ok": true, "data": "some value"}`
- **Error Field**: Use `error` (string) for human-readable error messages.

## 6. Duplicate Definitions

- Ensure each API method name (e.g., `api.get_mcp_info`) is defined exactly once.
- Prefer defining APIs in specialized backend modules (`backend/process_api.py`, `backend/child_api.py`) rather than `main.py`.

## 7. Error Handling

- **Never** return error messages as valid data types (e.g., returning a string starting with `# Error` when the client expects source code).
- Always set `ok: false` and provide an `error` message.

## 8. MCP Tool Registration

- When registering tools in the MCP server, ensure tool names are namespaced (e.g., `api.run_sample`) if they map directly to AuroraView API calls.
- Tool descriptions should be extracted from the Python docstrings.
