# AuroraView Packing Guide

This document describes the AuroraView packing system, which bundles frontend assets, Python code, and dependencies into a single executable.

## Overview

AuroraView supports multiple packing strategies for different deployment scenarios:

| Strategy | Description | Output Size | Use Case |
|----------|-------------|-------------|----------|
| `standalone` | Embedded Python runtime (python-build-standalone) | ~50-100MB | Fully offline distribution |
| `pyoxidizer` | PyOxidizer-based embedding | ~30-50MB | Single-file with optimizations |
| `embedded` | Overlay mode, requires system Python | ~15MB | When Python is available |
| `portable` | Directory with bundled runtime | Varies | Development/testing |
| `system` | Uses system Python | ~5MB | Minimal distribution |

## Architecture

### Development Mode vs Packed Mode

**Development Mode** (non-packed):
```
Python main.py
    ├── Creates WebView (via Rust binding)
    ├── Loads frontend (dist/index.html)
    ├── Registers API callbacks (@view.bind_call)
    ├── Creates PluginManager
    └── view.show() - starts event loop

[Python is the main process, directly controls WebView]
```

**Packed Mode**:
```
app.exe (Rust)
    ├── Extracts resources and Python runtime
    ├── Creates WebView
    ├── Loads frontend (from overlay)
    ├── Starts Python backend process (main.py)
    │       └── Runs as API server (JSON-RPC over stdin/stdout)
    └── Event loop (Rust main thread)

[Rust is the main process, Python is a subprocess providing API]
```

This is a **frontend-backend separation architecture**:
- Rust controls WebView lifecycle (more stable)
- Python crash doesn't affect UI
- Can restart Python backend without restarting UI
- Better process isolation and error handling

### Communication Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Packed Executable                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    IPC (stdin/stdout)    ┌──────────────┐ │
│  │   WebView   │ ◄──────────────────────► │   Python     │ │
│  │  (Frontend) │                          │  (Backend)   │ │
│  │             │    JSON-RPC Protocol     │              │ │
│  │  React/Vue  │ ◄──────────────────────► │  API Server  │ │
│  └─────────────┘                          └──────────────┘ │
│        │                                         │         │
│        │ Plugin System                           │         │
│        ▼                                         │         │
│  ┌─────────────┐                                 │         │
│  │   Rust      │                                 │         │
│  │  Plugins    │ ◄───────────────────────────────┘         │
│  │ (process,   │   Forward plugin commands                 │
│  │  shell...)  │                                           │
│  └─────────────┘                                           │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

### Packed Overlay Structure

```
overlay/
├── frontend/                  # Web assets
│   ├── index.html
│   ├── assets/
│   │   ├── index-xxx.js
│   │   └── index-xxx.css
│   └── ...
├── python/                    # Python code and dependencies
│   ├── main.py               # Entry point
│   ├── site-packages/        # Third-party dependencies
│   │   ├── auroraview/       # AuroraView Python package
│   │   │   ├── __init__.py
│   │   │   ├── _core.pyd     # Rust binding
│   │   │   └── ...
│   │   └── pyyaml/           # Other dependencies
│   └── bin/                  # External binaries
│       └── ffmpeg.exe        # (if configured)
├── resources/                 # Additional resources
│   └── examples/             # Example files
├── python_runtime.json        # Python runtime metadata
└── python_runtime.tar.gz      # Embedded Python distribution
```

### Runtime Extraction Directory

When the packed executable runs, files are extracted to:

```
Windows: %LOCALAPPDATA%\AuroraView\python\{app-name}\
Linux:   ~/.cache/AuroraView/python/{app-name}/
macOS:   ~/Library/Caches/AuroraView/python/{app-name}/
```

Extracted structure:
```
{app-name}/
├── main.py
├── site-packages/
│   └── auroraview/
├── bin/
├── resources/
│   └── examples/
└── __pycache__/
```

## Configuration

### Pack Configuration File (auroraview.pack.toml)

```toml
[app]
name = "my-app"
version = "1.0.0"
title = "My Application"

[window]
width = 1200
height = 800
resizable = true
debug = false

[frontend]
path = "dist"                    # Built frontend directory

[python]
entry_point = "main:run_gallery" # module:function or file.py
include_paths = ["src"]          # Python source directories
packages = ["pyyaml"]            # Pip packages to include
version = "3.10"                 # Python version
strategy = "standalone"          # Packing strategy

# Module search paths (runtime PYTHONPATH)
# Special variables: $EXTRACT_DIR, $RESOURCES_DIR, $SITE_PACKAGES
module_search_paths = ["$EXTRACT_DIR", "$SITE_PACKAGES"]

# Enable filesystem importer for dynamic imports
filesystem_importer = true

# External binaries to bundle
external_bin = ["path/to/ffmpeg.exe"]

# Additional resources
resources = ["assets", "config"]

# Exclude patterns
exclude = ["__pycache__", "*.pyc", "tests"]

[hooks]
# Collect additional files at pack time
collect = [
    { from = "examples/*.py", to = "resources/examples" }
]

[env]
# Environment variables to inject at runtime
MY_VAR = "value"

[license]
# License validation (optional)
type = "token"
public_key = "..."
```

### Module Search Paths

The `module_search_paths` configuration controls Python's import path at runtime:

| Variable | Description |
|----------|-------------|
| `$EXTRACT_DIR` | Root extraction directory |
| `$SITE_PACKAGES` | The `site-packages` subdirectory |
| `$RESOURCES_DIR` | The `resources` subdirectory |

Default: `["$EXTRACT_DIR", "$SITE_PACKAGES"]`

This allows:
- `$EXTRACT_DIR` - Import from root (e.g., `import main`)
- `$SITE_PACKAGES` - Import bundled packages (e.g., `import auroraview`)

## Building

### Using Just Commands

```bash
# Build and pack Gallery
just gallery-pack

# Pack with custom config
just pack --config path/to/config.toml

# Build frontend first, then pack
just gallery-build
just gallery-pack
```

### Using CLI Directly

```bash
# Pack application
cargo run -p auroraview-cli --release -- pack --config auroraview.pack.toml

# Pack with build step
cargo run -p auroraview-cli --release -- pack --config auroraview.pack.toml --build
```

## Runtime Behavior

### Environment Variables

The packed executable sets these environment variables for the Python backend:

| Variable | Description |
|----------|-------------|
| `AURORAVIEW_PACKED` | Set to `"1"` in packed mode |
| `AURORAVIEW_RESOURCES_DIR` | Path to extracted resources |
| `AURORAVIEW_EXAMPLES_DIR` | Path to examples (if present) |
| `AURORAVIEW_PYTHON_PATH` | Module search paths |
| `PYTHONPATH` | Same as above (for subprocess) |

### Detecting Packed Mode in Python

```python
import os

PACKED_MODE = os.environ.get("AURORAVIEW_PACKED", "0") == "1"

if PACKED_MODE:
    # Running in packed mode
    resources_dir = os.environ.get("AURORAVIEW_RESOURCES_DIR")
    # Run as API server
    run_api_server()
else:
    # Running in development mode
    # Create WebView directly
    view = WebView(...)
    view.show()
```

### JSON-RPC Protocol

In packed mode, Python communicates with Rust via JSON-RPC over stdin/stdout:

**Request** (Rust → Python):
```json
{
    "id": "unique-id",
    "method": "api.get_samples",
    "params": {}
}
```

**Response** (Python → Rust):
```json
{
    "id": "unique-id",
    "ok": true,
    "result": [...]
}
```

**Error Response**:
```json
{
    "id": "unique-id",
    "ok": false,
    "error": "Error message"
}
```

## Caching

### Python Runtime Cache

The Python runtime is cached to avoid re-extraction on every run:

```
Windows: %LOCALAPPDATA%\AuroraView\python-standalone\
```

Cache key: `cpython-{version}-{target}.tar.gz`

A version marker file is used to detect when re-extraction is needed.

### File Hash Cache

The `FileHashCache` system tracks file content changes:

```rust
use auroraview_pack::FileHashCache;

let mut cache = FileHashCache::load(&cache_path)?;

// Check if file changed
if cache.has_changed(&file_path, "key")? {
    // Process file
    cache.update("key", &file_path)?;
}

cache.save(&cache_path)?;
```

This enables incremental packing in future versions.

## Troubleshooting

### Common Issues

**Module not found error**:
- Check `module_search_paths` configuration
- Verify package is in `packages` list
- Check if package was successfully collected during packing

**Python backend not starting**:
- Check `entry_point` format (`module:function` or `file.py`)
- Verify Python version compatibility
- Check stderr output for errors

**Resources not found**:
- Verify `hooks.collect` patterns
- Check `AURORAVIEW_RESOURCES_DIR` environment variable
- Ensure resources are in overlay

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug ./my-app.exe
```

Or in config:
```toml
[window]
debug = true
```

## Best Practices

1. **Use `site-packages` for dependencies**: All third-party packages go to `python/site-packages/`

2. **Use `bin/` for executables**: External binaries go to `python/bin/`

3. **Separate API from UI logic**: In packed mode, Python only provides API, Rust handles UI

4. **Handle both modes**: Design code to work in both development and packed modes

5. **Use environment variables**: Check `AURORAVIEW_PACKED` to adapt behavior

6. **Log to stderr**: In packed mode, stdout is for JSON-RPC, use stderr for logging

## See Also

- [CLI Documentation](CLI.md)
- [Architecture Overview](ARCHITECTURE.md)
- [IPC Architecture](IPC_ARCHITECTURE.md)
