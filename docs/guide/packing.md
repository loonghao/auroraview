# Application Packing

This guide describes the AuroraView packing system, which bundles frontend assets, Python code, and dependencies into a single executable.

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

## Configuration

### Pack Configuration File (auroraview.pack.toml)

```toml
# ============================================================================
# Package Information
# ============================================================================
[package]
name = "my-app"
version = "1.0.0"
description = "My application description"
authors = ["Your Name <your@email.com>"]
license = "MIT"

# ============================================================================
# Application Configuration
# ============================================================================
[app]
title = "My Application"
frontend_path = "./dist"         # Built frontend directory
# url = "https://example.com"    # Alternative: load from URL
allow_new_window = false          # Deprecated: use new_window_mode="child_webview" instead

# ============================================================================
# Window Configuration
# ============================================================================
[window]
width = 1280
height = 720
min_width = 800
min_height = 600
resizable = true
frameless = false
transparent = false
always_on_top = false
start_position = "center"

# ============================================================================
# Bundle Configuration
# ============================================================================
[bundle]
icon = "./assets/my-app-icon.png"
identifier = "com.mycompany.myapp"
copyright = "Copyright 2025 My Company"
category = "Productivity"
short_description = "A great application"

# ============================================================================
# Platform-Specific Bundle Configuration
# ============================================================================
[bundle.platform.windows]
console = false                  # Hide console window for GUI apps
file_version = "1.0.0.0"
product_version = "1.0.0"
file_description = "My Application"
product_name = "My Application"
company_name = "My Company"

[bundle.platform.macos]
# bundle_identifier = "com.mycompany.myapp"
# minimum_system_version = "10.15"

[bundle.platform.linux]
# categories = ["Utility", "Development"]
# appimage = true

# ============================================================================
# Python Backend Configuration (FullStack mode)
# ============================================================================
[python]
enabled = true
version = "3.11"
entry_point = "main:run"         # module:function format
packages = ["pyyaml", "requests"]
include_paths = [".", "src"]
exclude = ["__pycache__", "*.pyc", "tests"]
optimize = 1                     # Bytecode optimization level (0-2)
strategy = "standalone"          # standalone, pyoxidizer, embedded, portable, system

# Python process configuration
[python.process]
console = false
module_search_paths = ["$EXTRACT_DIR", "$SITE_PACKAGES"]
filesystem_importer = true

# Environment isolation configuration
[python.isolation]
pythonpath = true                # Isolate PYTHONPATH
path = true                      # Isolate PATH

# Python code protection (optional)
[python.protection]
enabled = true
method = "bytecode"              # "bytecode" (fast) or "py2pyd" (slow)
optimization = 2

# Encryption settings (for bytecode method)
[python.protection.encryption]
enabled = true
algorithm = "x25519"             # "x25519" (fast) or "p256" (FIPS compliant)

# ============================================================================
# Build Configuration
# ============================================================================
[build]
before = ["npm run build"]       # Commands before build
after = []                       # Commands after build
resources = ["./public"]
exclude = ["*.map", "*.ts", "node_modules"]
out_dir = "./pack-output"
release = true

# ============================================================================
# Runtime Environment Configuration
# ============================================================================
[runtime]
[runtime.env]
APP_ENV = "production"
LOG_LEVEL = "info"

# ============================================================================
# Hooks Configuration
# ============================================================================
[hooks]
before_collect = []
after_pack = []

[[hooks.collect]]
source = "./examples/*.py"
dest = "resources/examples"
preserve_structure = false
description = "Example files"

# ============================================================================
# Debug Configuration
# ============================================================================
[debug]
enabled = false
devtools = false
verbose = false
```

### Module Search Paths

The `module_search_paths` configuration controls Python's import path at runtime:

| Variable | Description |
|----------|-------------|
| `$EXTRACT_DIR` | Root extraction directory |
| `$SITE_PACKAGES` | The `site-packages` subdirectory |
| `$RESOURCES_DIR` | The `resources` subdirectory |
| `$PYTHON_HOME` | Python runtime directory |

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

## Vx dependency bootstrap

AuroraView Pack can download/embed vx runtime and other assets during packing to provide a unified toolchain for offline installs.

```toml
[vx]
enabled = true
runtime_url = "https://github.com/loonghao/vx/releases/download/vx-v0.6.10/vx-0.6.10-x86_64-pc-windows-msvc.zip"
runtime_checksum = "<sha256>"
cache_dir = "./.pack-cache/vx"
ensure = ["uv", "node@20", "go@1.22", "rust@stable"]
allow_insecure = false
allowed_domains = ["github.com", "objects.githubusercontent.com"]
block_unknown_domains = false
require_checksum = false

[[downloads]]
name = "vx-runtime"
url = "https://github.com/loonghao/vx/releases/download/vx-v0.6.10/vx-0.6.10-x86_64-pc-windows-msvc.zip"
checksum = "<sha256>"
extract = true
strip_components = 1
stage = "before_collect"
dest = "python/bin/vx"
executable = ["vx.exe"]

[hooks]
use_vx = true

[hooks.vx]
before_collect = ["vx --version"]
after_pack = ["vx uv pip list"]
```

- `downloads.stage` supports `before_collect`, `before_pack`, `after_pack`.
- `hooks.use_vx` wraps legacy hooks with `vx`; `hooks.vx.*` always run via vx.
- `AURORAVIEW_OFFLINE=1` uses cached artifacts only.
- Runtime installer prefers `vx uv pip` when `AURORAVIEW_VX_PATH` or PATH provides `vx`.

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
[debug]
enabled = true
verbose = true
```

## Code Protection

AuroraView provides two methods to protect your Python source code from reverse engineering:

### Protection Methods

| Method | Speed | Requirements | Protection Level | Description |
|--------|-------|--------------|------------------|-------------|
| `bytecode` | **Fast** | Python only | High | Encrypts Python bytecode with ECC + AES-256-GCM |
| `py2pyd` | Slow | C/C++ compiler | Maximum | Compiles to native `.pyd`/`.so` via Cython |

### Bytecode Encryption (Recommended)

The `bytecode` method is the default and recommended approach:

1. **Compiles** `.py` files to `.pyc` bytecode
2. **Encrypts** bytecode with AES-256-GCM (symmetric encryption)
3. **Protects** the AES key with ECC (X25519 or P-256)
4. **Decrypts** at runtime via a bootstrap loader

```
┌─────────────────────────────────────────────────────────────────┐
│                        Pack Time (Build)                        │
├─────────────────────────────────────────────────────────────────┤
│  .py ──► py_compile ──► .pyc bytecode                          │
│                              │                                  │
│                              ▼                                  │
│                    AES-256-GCM encrypt                          │
│                              │                                  │
│                              ▼                                  │
│                      encrypted .pyc.enc                         │
│                                                                 │
│  Also: AES key ──► ECC public key encrypt ──► encrypted key    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                       Runtime (User Machine)                    │
├─────────────────────────────────────────────────────────────────┤
│  1. encrypted key ──► embedded private key decrypt ──► AES key │
│                                                                 │
│  2. .pyc.enc ──► AES decrypt ──► .pyc bytecode (~GB/s)         │
│                                                                 │
│  3. marshal.loads() + exec() to execute                        │
└─────────────────────────────────────────────────────────────────┘
```

**Configuration:**

```toml
[python.protection]
enabled = true
method = "bytecode"              # Fast, no C compiler needed
optimization = 2                 # Python bytecode optimization (0-2)

[python.protection.encryption]
enabled = true
algorithm = "x25519"             # "x25519" (fast) or "p256" (FIPS compliant)
```

**Encryption Algorithms:**

| Algorithm | Speed | Security | Use Case |
|-----------|-------|----------|----------|
| `x25519` | **Fast** | Modern, 128-bit | Default, recommended |
| `p256` | Moderate | NIST/FIPS compliant | Government/enterprise |

### py2pyd Compilation (Maximum Protection)

The `py2pyd` method compiles Python to native machine code:

1. **Converts** `.py` to C code via Cython
2. **Compiles** C code to native `.pyd` (Windows) or `.so` (Linux/macOS)
3. **Replaces** original `.py` files with compiled extensions

**Configuration:**

```toml
[python.protection]
enabled = true
method = "py2pyd"                # Slow, requires C compiler
optimization = 3                 # C compiler optimization (0-3)
keep_temp = false                # Keep temp files for debugging
```

**Requirements:**
- C/C++ compiler (MSVC on Windows, GCC/Clang on Linux/macOS)
- Cython (installed automatically via uv)

**Note:** This method is significantly slower because it creates a new virtual environment for each file being compiled.

### Excluding Files from Protection

You can exclude specific files or patterns:

```toml
[python.protection]
enabled = true
method = "bytecode"
exclude = [
    "config.py",           # Keep config readable
    "**/tests/**",         # Skip test files
    "setup.py",            # Skip setup files
]
```

## Headless CLI Mode

A packed executable is a GUI app by default — double-clicking it (or launching
with no arguments) opens the window exactly as before. It can *also* run your
registered Python commands from a terminal, without opening a window.

```bash
my-app.exe                          # GUI window (unchanged)
my-app.exe -h                       # list CLI-enabled commands
my-app.exe list                     # list commands (add --json for machine output)
my-app.exe run export --path ./out  # run one command, print result, exit
my-app.exe -V                       # print version
```

Only an explicit reserved verb (`run`, `list`) or flag (`-h`/`--help`,
`-V`/`--version`) as the **first** argument triggers the CLI path. A bare file
path (file association, drag-and-drop) always opens the GUI, so nothing existing
breaks.

### Exposing commands to the CLI

CLI exposure is **opt-in** — commands are GUI-only unless you say otherwise. Set
`cli` on `@webview.command`:

```python
@webview.command(name="export", help="Export the project", cli=True)
def export(path: str, dpi: int = 300) -> dict:
    return {"written": path, "dpi": dpi}

# Add a short alias:
@webview.command(name="export-document-image", cli="exi")
def export_document_image(path: str) -> dict: ...

# Multiple aliases:
@webview.command(name="validate", cli=["val", "v"])
def validate() -> dict: ...
```

The `cli` value doubles as the on/off switch and the alias list:

| `cli` value          | Meaning                          |
|----------------------|----------------------------------|
| `False` (default)    | GUI only, not exposed to the CLI |
| `True`               | exposed, no alias                |
| `"exi"`              | exposed, alias `exi`             |
| `["exi", "edi"]`     | exposed, multiple aliases        |

Use `help=` and `args_help={...}` to enrich the `-h` output; both fall back to
the function docstring / signature when omitted. To bulk-enable existing
commands, call `webview.commands.enable_cli("export", "validate")` or pass a
`{name: alias}` mapping.

### Passing arguments

Both keyword and positional forms work, and they can be mixed (positional
first, keywords after):

```bash
my-app.exe run export --path ./out --dpi 600   # keyword
my-app.exe run export ./out 600                 # positional (signature order)
my-app.exe run export ./out --dpi 600           # mixed
```

Values are coerced from the parameter's type annotation (`int`, `float`, `bool`,
`str`); anything else is parsed as JSON (`--config '{"a":1}'`). Booleans accept
`--flag` / `--no-flag`. Exit codes follow: `0` success, `1` the command raised,
`2` command not found or bad arguments.

### How it works

The command list shown by `-h`/`list` is collected **at pack time** and embedded
in the overlay, so those commands return instantly without starting Python. The
collection runs the bundled entry point once with `AURORAVIEW_CLI_DUMP=1`. This
requires running the bundled interpreter on the build host, so it is skipped
(with a warning) when cross-platform packing or when not using the `standalone`
Python strategy — `-h`/`list` then show no commands until repacked on a matching
host. `run` extracts the Python runtime on first use (cached afterwards) and
invokes the command in a one-shot process — it does not reuse the GUI's
persistent backend.

On Windows the packed exe attaches to the parent console at startup so CLI
output reaches the terminal (a double-click still shows no console). On
macOS/Linux there is no GUI-subsystem stdio isolation, so output works directly.

### The Windows `.cmd` shim

The packed exe is GUI-subsystem so a double-click never flashes a console. The
trade-off is that `cmd.exe` and PowerShell do **not** wait for a GUI-subsystem
process — `app.exe run export ...` would return to the prompt before the
command's output lands, and the exit code would be lost.

To fix this, when the pack exposes CLI commands and is **not** built
console-subsystem (the default), the builder drops a `<name>.cmd` shim beside
the exe:

```bat
@echo off
start "" /wait /b "%~dp0app.exe" %*
exit /b %ERRORLEVEL%
```

`start /wait` runs the exe synchronously so the terminal blocks until it
finishes, `%*` forwards every argument, and `exit /b %ERRORLEVEL%` propagates
the exit code. `%~dp0` resolves the exe relative to the shim's own directory, so
it works regardless of the caller's current directory.

In a terminal, invoke the shim so output and exit codes behave correctly:

```bash
app.cmd -h                       # blocks, prints help, then returns
app.cmd run export --path ./out  # blocks until the command finishes
app.exe                          # double-click / GUI launch still uses the exe
```

The shim is best-effort: if it can't be written (e.g. a read-only output
directory) the build still succeeds and only logs a warning. It is not produced
for console-subsystem builds (`[bundle.platform.windows] console = true`), where
the exe already blocks the terminal on its own.

## Best Practices

1. **Use `site-packages` for dependencies**: All third-party packages go to `python/site-packages/`

2. **Use `bin/` for executables**: External binaries go to `python/bin/`

3. **Separate API from UI logic**: In packed mode, Python only provides API, Rust handles UI

4. **Handle both modes**: Design code to work in both development and packed modes

5. **Use environment variables**: Check `AURORAVIEW_PACKED` to adapt behavior

6. **Log to stderr**: In packed mode, stdout is for JSON-RPC, use stderr for logging

7. **Use bytecode protection for production**: Enable `[python.protection]` with `method = "bytecode"` for fast, secure code protection
