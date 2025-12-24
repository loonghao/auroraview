# CLI Reference

AuroraView provides two command-line interfaces:

1. **`auroraview`** (Python) - Installed via pip, for quick WebView previews
2. **`auroraview-cli`** (Rust) - Standalone binary for packaging and advanced features

## Quick Start

### Python CLI (via pip)

```bash
pip install auroraview
# or
uv pip install auroraview
```

```bash
# Load a URL
auroraview --url https://example.com

# Load a local HTML file
auroraview --html /path/to/file.html

# Custom window configuration
auroraview --url https://example.com --title "My App" --width 1024 --height 768
```

### Rust CLI (for packaging)

Download from [GitHub Releases](https://github.com/loonghao/auroraview/releases) or build from source:

```bash
# Build from source
cargo build -p auroraview-cli --release
# Binary: target/release/auroraview-cli (or auroraview-cli.exe on Windows)
```

```bash
# Pack a URL-based application
auroraview-cli pack --url https://example.com --output myapp

# Pack a frontend project (React, Vue, etc.)
auroraview-cli pack --frontend ./dist --output myapp

# Pack a fullstack application (frontend + Python backend)
auroraview-cli pack --config auroraview.pack.toml
```

## CLI Comparison

| Feature | `auroraview` (Python) | `auroraview-cli` (Rust) |
|---------|----------------------|-------------------------|
| Installation | `pip install auroraview` | GitHub Releases or `cargo build` |
| URL/HTML preview | ✅ | ✅ |
| Application packaging | ❌ | ✅ |
| FullStack mode (Python backend) | ❌ | ✅ |
| Embedded Python runtime | ❌ | ✅ |
| No Python required | ❌ | ✅ |

## Python CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `-u, --url <URL>` | URL to load in the WebView | - |
| `-f, --html <FILE>` | Local HTML file to load | - |
| `--assets-root <DIR>` | Assets root directory for local HTML files | HTML file's directory |
| `-t, --title <TITLE>` | Window title | "AuroraView" |
| `-w, --width <WIDTH>` | Window width in pixels (set to 0 to maximize) | 1024 |
| `-H, --height <HEIGHT>` | Window height in pixels (set to 0 to maximize) | 768 |
| `-d, --debug` | Enable debug logging (DevTools) | false |
| `--allow-new-window` | Allow opening new windows (e.g., via window.open) | false |
| `--allow-file-protocol` | Enable file:// protocol support | false |
| `--always-on-top` | Keep window always on top | false |
| `-h, --help` | Print help information | - |

## Rust CLI Commands

### Preview

```bash
auroraview-cli --url https://example.com
auroraview-cli --html index.html
```

### Pack

```bash
# URL mode - creates a standalone browser for a specific URL
auroraview-cli pack --url https://example.com --output myapp

# Frontend mode - bundles static assets
auroraview-cli pack --frontend ./dist --output myapp

# FullStack mode - bundles frontend + Python backend
auroraview-cli pack --config auroraview.pack.toml
```

## Examples

### Quick Web Preview (Python)

```bash
# Preview a website
auroraview --url https://github.com

# Preview with custom size
auroraview --url https://github.com --width 1920 --height 1080

# Preview with always-on-top window
auroraview --url https://github.com --always-on-top

# Enable DevTools for debugging
auroraview --url https://github.com --debug
```

### Local Development (Python)

```bash
# Preview local HTML file
auroraview --html index.html

# Preview with custom title
auroraview --html dist/index.html --title "My App Preview"

# Specify assets root directory
auroraview --html dist/index.html --assets-root ./assets

# Enable file:// protocol for local resources
auroraview --html index.html --allow-file-protocol
```

### Using with uvx (No Installation Required)

```bash
# Run directly without installing
uvx auroraview --url https://example.com

# Load local file
uvx auroraview --html test.html
```

### Application Packaging (Rust CLI)

```bash
# Pack a simple URL app
auroraview-cli pack --url https://myapp.com --output myapp

# Pack a React/Vue frontend
auroraview-cli pack --frontend ./build --output myapp --title "My App"

# Pack a fullstack app with Python backend
auroraview-cli pack --config auroraview.pack.toml
```

## Platform Support

The CLI is supported on:

- **Windows**: Uses WebView2 (built into Windows 10/11)
- **macOS**: Uses WKWebView (built into macOS)
- **Linux**: Uses WebKitGTK (requires installation)

### Linux Dependencies

On Linux, you need to install WebKitGTK:

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev

# Fedora/CentOS
sudo dnf install webkit2gtk3-devel

# Arch Linux
sudo pacman -S webkit2gtk
```

## Troubleshooting

### Python 3.7 on Windows with uvx

Due to a [known limitation](https://github.com/astral-sh/uv/issues/10165) in uv/uvx, the `auroraview` command does not work with Python 3.7 on Windows.

**Workaround**: Use `python -m auroraview` instead:

```bash
# Instead of: uvx --python 3.7 auroraview --url https://example.com
# Use:
uvx --python 3.7 --from auroraview python -m auroraview --url https://example.com

# Or with pip-installed package:
python -m auroraview --url https://example.com
```

### Binary Not Found

If you get "auroraview: command not found":

```bash
pip install --force-reinstall auroraview
```

### WebView2 Not Found (Windows)

On Windows, WebView2 is required. It's pre-installed on Windows 10/11, but if you encounter issues:

1. Download and install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Permission Denied (Linux/macOS)

If you get permission errors:

```bash
chmod +x $(which auroraview)
```
