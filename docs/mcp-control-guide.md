# MCP Control Guide

This guide explains how to control which methods are exposed to MCP (Model Context Protocol) and how to customize MCP tool names.

## Overview

When using AuroraView's embedded MCP server, you can control which `bind_call` methods are exposed as MCP tools using the `mcp` and `mcp_name` parameters.

## Parameters

### `mcp` Parameter

- **Type**: `bool`
- **Default**: `True`
- **Purpose**: Control whether a method is exposed to MCP clients

**Usage**:

```python
# Exposed to MCP (default)
@view.bind_call("api.get_user")
def get_user(user_id: str) -> dict:
    return {"name": "Alice"}

# Hidden from MCP - only available to JavaScript
@view.bind_call("api._internal_debug", mcp=False)
def internal_debug() -> dict:
    return {"debug": "info"}
```

### `mcp_name` Parameter

- **Type**: `Optional[str]`
- **Default**: `None` (use original method name)
- **Purpose**: Provide a custom, user-friendly name for MCP tools

**Usage**:

```python
# Registered as "create_user" instead of "api.create_user_record"
@view.bind_call("api.create_user_record", mcp_name="create_user")
def create_user_record(name: str) -> dict:
    return {"name": name}
```

## Use Cases

### 1. Hide Internal Methods

Keep internal utility methods hidden from AI assistants while still using them in your application:

```python
@view.bind_call("api._internal_validate", mcp=False)
def _internal_validate(data: dict) -> bool:
    # Internal validation logic
    return True

# This is only available via JavaScript, not MCP
# await auroraview.call("api._internal_validate", {...})
```

### 2. User-Friendly Tool Names

Provide shorter, clearer names for AI assistants while keeping your code organized:

```python
@view.bind_call("api.create_user_record_in_database", mcp_name="create_user")
def create_user_record_in_database(name: str, email: str) -> dict:
    # Implementation
    return {"id": "123"}

# MCP clients see: create_user(name, email)
# JavaScript sees: api.create_user_record_in_database(...)
```

### 3. Explicit MCP Control

Make MCP exposure clear in your code:

```python
# Explicitly expose (redundant but clear)
@view.bind_call("api.delete_user", mcp=True)
def delete_user(user_id: str) -> dict:
    return {"deleted": True}

# Explicitly hide
@view.bind_call("api._admin_only", mcp=False)
def admin_only() -> dict:
    return {"admin": "data"}
```

### 4. Multiple Methods, Different Behavior

Control each method independently:

```python
# Public API - exposed to MCP
@view.bind_call("api.get_data")
def get_data() -> dict:
    return {"data": "public"}

# Internal API - hidden from MCP
@view.bind_call("api._get_cache", mcp=False)
def _get_cache() -> dict:
    return {"cache": "internal"}

# Admin API - custom name for clarity
@view.bind_call("api._admin_reset_cache", mcp_name="reset_cache")
def _admin_reset_cache() -> dict:
    return {"reset": True}
```

## Default Behavior

When `auto_expose_api=True` in `McpConfig`:

1. All `bind_call` methods are exposed to MCP by default (`mcp=True`)
2. Method names are used directly as MCP tool names
3. Only methods starting with `_` are filtered out

## Backward Compatibility

The new parameters are fully backward compatible:

```python
# Old code - still works exactly as before
@view.bind_call("api.echo")
def echo(message: str) -> str:
    return message
# Equivalent to: mcp=True, mcp_name=None
```

## Best Practices

### 1. Use Descriptive MCP Names

AI assistants work best with clear, descriptive names:

```python
# Good: Clear and concise
@view.bind_call("api.create_user_in_system", mcp_name="create_user")
def create_user_in_system(name: str) -> dict:
    ...

# Good: Action-oriented
@view.bind_call("api._delete_record", mcp_name="delete_record")
def _delete_record(id: str) -> dict:
    ...

# Avoid: Too technical
@view.bind_call("api.crud_create_entity", mcp_name="create_entity")
def crud_create_entity(entity: dict) -> dict:
    ...
```

### 2. Hide Internal Operations

Keep internal utility methods hidden:

```python
# Public API
@view.bind_call("api.get_user")
def get_user(id: str) -> dict:
    ...

# Internal helpers - hidden from MCP
@view.bind_call("api._validate_user", mcp=False)
def _validate_user(user: dict) -> bool:
    ...

@view.bind_call("api._transform_user", mcp=False)
def _transform_user(raw: dict) -> dict:
    ...
```

### 3. Consistent Naming Patterns

Use consistent patterns for related operations:

```python
# CRUD operations with consistent naming
@view.bind_call("api.crud_create_user", mcp_name="create_user")
def crud_create_user(data: dict) -> dict:
    ...

@view.bind_call("api.crud_read_user", mcp_name="read_user")
def crud_read_user(id: str) -> dict:
    ...

@view.bind_call("api.crud_update_user", mcp_name="update_user")
def crud_update_user(id: str, data: dict) -> dict:
    ...

@view.bind_call("api.crud_delete_user", mcp_name="delete_user")
def crud_delete_user(id: str) -> dict:
    ...
```

### 4. Document MCP Exposure

Document which methods are exposed to MCP in docstrings:

```python
@view.bind_call("api.get_user", mcp=True)
def get_user(user_id: str) -> dict:
    """Get user information by ID.

    Available to: JavaScript (auroraview.call) and MCP (AI assistants)
    """
    return {"name": "Alice"}

@view.bind_call("api._internal_log", mcp=False)
def _internal_log(message: str) -> dict:
    """Internal logging function.

    Available to: JavaScript (auroraview.call) only
    Hidden from: MCP (AI assistants)
    """
    return {"logged": True}
```

## Examples

See `examples/mcp_control_demo.py` for a complete working example demonstrating:
- Default behavior
- Hiding methods from MCP
- Custom MCP names
- Multiple methods with different settings
- Decorator usage

## Debugging

Enable debug logging to see MCP registration:

```python
import logging
logging.basicConfig(level=logging.DEBUG)

# You'll see output like:
# [MCP DEBUG] auto_expose_api=True, has_bound_functions=True
# [MCP DEBUG] Found 5 bound functions: ['api.get_user', 'api._internal_debug', ...]
# [MCP DEBUG] Registered tool: api.get_user
# [MCP DEBUG] Skipping api._internal_debug: mcp=False (hidden from MCP)
# [MCP DEBUG] Registered tool: api.create_user_record -> create_user (custom MCP name)
```

## API Reference

### bind_call Method Signature

```python
def bind_call(
    self,
    method: str,
    func: Optional[Callable[..., Any]] = None,
    *,
    allow_rebind: bool = True,
    mcp: bool = True,           # NEW: MCP exposure control
    mcp_name: Optional[str] = None,  # NEW: Custom MCP tool name
):
    ...
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|-------|----------|-------------|
| `method` | `str` | Required | Method name (e.g., "api.echo") |
| `func` | `Callable` | `None` | Python callable to bind |
| `allow_rebind` | `bool` | `True` | Allow rebinding existing methods |
| `mcp` | `bool` | `True` | Expose this method to MCP clients |
| `mcp_name` | `str` | `None` | Custom name for MCP tool |

## Migration Guide

If you have existing code and want to adopt these new parameters:

### Step 1: Review Existing Bindings

```python
# List all current bindings
print(view.get_bound_methods())
```

### Step 2: Identify Internal Methods

Mark internal methods with `mcp=False`:

```python
# Before
@view.bind_call("api._internal_validate")
def _internal_validate(data):
    ...

# After
@view.bind_call("api._internal_validate", mcp=False)
def _internal_validate(data):
    ...
```

### Step 3: Improve MCP Names

Add user-friendly names for complex method names:

```python
# Before
@view.bind_call("api.crud_operations_create_user_entity")
def create_user_entity(data):
    ...

# After
@view.bind_call("api.crud_operations_create_user_entity", mcp_name="create_user")
def create_user_entity(data):
    ...
```

### Step 4: Test MCP Exposure

```python
# Start the server
view.show()

# Check which tools are exposed
# Connect an MCP client to see the tool list
# Internal methods should not appear
```

## Troubleshooting

### Method Not Appearing in MCP

**Problem**: Your `bind_call` method doesn't appear in the MCP tool list.

**Solution**:
1. Check if `mcp=False` is set
2. Verify `auto_expose_api=True` in `McpConfig`
3. Check debug logs for registration messages
4. Ensure the method doesn't start with `_`

### Wrong MCP Name Appearing

**Problem**: The MCP tool has a different name than expected.

**Solution**:
1. Check if `mcp_name` is set to a different value
2. Verify the original method name in the debug logs
3. Look for mapping messages like: `Registered tool: api.x -> y (custom MCP name)`

### JavaScript Still Sees Hidden Methods

**Problem**: A method with `mcp=False` is still callable from JavaScript.

**Expected Behavior**: This is correct! `mcp=False` only hides the method from MCP, not from JavaScript.

## Future Enhancements

Planned features for MCP control:

1. **MCP Groups**: Group related tools and expose/hide entire groups
2. **MCP Annotations**: Add read-only, destructive, idempotent hints
3. **MCP Filters**: Advanced filtering by pattern or function type
4. **MCP Namespaces**: Separate MCP tools into logical namespaces

## See Also

- [MCP Usage Guide](mcp-usage-guide.md)
- [MCP Optimization Summary](mcp-optimization-summary.md)
- [RFC 0002: Embedded MCP Server](rfcs/0002-embedded-mcp-server.md)
- [RFC 0003: API Design Guidelines](rfcs/0003-api-design-guidelines.md)
