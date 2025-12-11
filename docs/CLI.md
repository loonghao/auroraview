# AuroraView CLI

AuroraView provides a command-line interface for quickly launching WebView windows with URLs or local HTML files.

## Installation

The CLI is automatically installed when you install the auroraview package:

```bash
pip install auroraview
# or
uv pip install auroraview
```

## Usage

### Load a URL

```bash
auroraview --url https://example.com
```

### Load a Local HTML File

```bash
auroraview --html /path/to/file.html
```

### Custom Window Configuration

```bash
auroraview --url https://example.com \
  --title "My App" \
  --width 1024 \
  --height 768
```

## Options

- `-u, --url <URL>` - URL to load in the WebView
- `-f, --html <HTML>` - Local HTML file to load in the WebView
- `-t, --title <TITLE>` - Window title (default: "AuroraView")
- `--width <WIDTH>` - Window width in pixels (default: 800)
- `--height <HEIGHT>` - Window height in pixels (default: 600)
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Examples

### Quick Web Preview

```bash
# Preview a website
auroraview --url https://github.com

# Preview with custom size
auroraview --url https://github.com --width 1920 --height 1080
```

### Local Development

```bash
# Preview local HTML file
auroraview --html index.html

# Preview with custom title
auroraview --html dist/index.html --title "My App Preview"
```

### Using with uvx (No Installation Required)

```bash
# Run directly without installing
uvx auroraview --url https://example.com

# Load local file
uvx auroraview --html test.html
```

## Development

### Building the CLI

```bash
# Build the CLI binary
cargo build --release --features cli --bin auroraview

# The binary will be at: target/release/auroraview (or auroraview.exe on Windows)
```

### Testing

```bash
# Test with URL
./target/release/auroraview --url https://example.com

# Test with local file
./target/release/auroraview --html test.html
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

Due to a [known limitation](https://github.com/astral-sh/uv/issues/10165) in uv/uvx, the `auroraview` command does not work with Python 3.7 on Windows. You will see an error like:

```
SyntaxError: Non-UTF-8 code starting with '\xe8' in file ...\auroraview.exe
```

**Workaround**: Use `python -m auroraview` instead:

```bash
# Instead of: uvx --python 3.7 auroraview --url https://example.com
# Use:
uvx --python 3.7 --from auroraview python -m auroraview --url https://example.com

# Or with pip-installed package:
python -m auroraview --url https://example.com
```

This issue only affects Python 3.7 on Windows. Python 3.8+ works correctly with the `auroraview` command.

### Binary Not Found

If you get "auroraview: command not found", ensure the package is properly installed:

```bash
pip install --force-reinstall auroraview
```

### WebView2 Not Found (Windows)

On Windows, WebView2 is required. It's pre-installed on Windows 10/11, but if you encounter issues:

1. Download and install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Permission Denied (Linux/macOS)

If you get permission errors, ensure the binary is executable:

```bash
chmod +x $(which auroraview)
```

## See Also

- [Python API Documentation](../README.md)
- [DCC Integration Guide](DCC_INTEGRATION.md)
- [Examples](../examples/)

