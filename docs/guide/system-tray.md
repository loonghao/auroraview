# System Tray

AuroraView supports system tray integration for background applications.

## Basic Usage

```python
from auroraview import run_desktop

run_desktop(
    title="My Background App",
    html=my_html,
    width=400,
    height=300,
    system_tray=True,      # Enable system tray
    hide_on_close=True,    # Minimize to tray instead of closing
)
```

## Features

- **System tray icon** with context menu
- **Hide to tray** on window close
- **Click to show** - restore window from tray
- **Custom tray icon** support

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `system_tray` | Enable system tray | `False` |
| `hide_on_close` | Minimize to tray on close | `False` |
| `tray_icon` | Custom tray icon path | App icon |

## Example: Background Service

```python
from auroraview import WebView

html = """
<!DOCTYPE html>
<html>
<body>
    <h1>Background Service</h1>
    <p>This app runs in the system tray.</p>
    <button onclick="auroraview.send_event('minimize_to_tray')">
        Minimize to Tray
    </button>
</body>
</html>
"""

webview = WebView.create(
    title="Background Service",
    html=html,
    system_tray=True,
    hide_on_close=True,
)

@webview.on("minimize_to_tray")
def handle_minimize(data):
    webview.hide()

webview.show()
```
