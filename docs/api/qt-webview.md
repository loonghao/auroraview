# QtWebView API

`QtWebView` is a Qt widget wrapper for seamless integration with Qt-based DCC applications.

## Import

```python
from auroraview import QtWebView
```

## Constructor

```python
QtWebView(
    parent: QWidget = None,
    url: str = None,
    html: str = None,
    width: int = 800,
    height: int = 600,
    auto_prewarm: bool = True,
)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `parent` | `QWidget` | `None` | Parent Qt widget |
| `url` | `str` | `None` | URL to load |
| `html` | `str` | `None` | HTML content to load |
| `width` | `int` | `800` | Widget width |
| `height` | `int` | `600` | Widget height |
| `auto_prewarm` | `bool` | `True` | Auto pre-warm WebView2 |

## Basic Usage

```python
from auroraview import QtWebView

# Simple usage
webview = QtWebView(
    parent=parent_widget,
    url="http://localhost:3000"
)
webview.show()
```

## Methods

### load_url(url: str)

Load a URL.

```python
webview.load_url("https://example.com")
```

### load_html(html: str)

Load HTML content.

```python
webview.load_html("<h1>Hello</h1>")
```

### emit(event: str, data: Any)

Emit an event to JavaScript.

```python
webview.emit("update", {"value": 42})
```

### bind_api(obj: object)

Bind an API object.

```python
class MyAPI:
    def get_data(self):
        return {"value": 42}

webview.bind_api(MyAPI())
```

### on(event: str)

Register an event handler.

```python
@webview.on("button_clicked")
def handle_click(data):
    print(f"Clicked: {data}")
```

## Qt Widget Methods

`QtWebView` inherits from `QWidget`, so all standard Qt methods are available:

```python
# Qt widget methods
webview.setMinimumSize(400, 300)
webview.setMaximumSize(1920, 1080)
webview.resize(800, 600)
webview.move(100, 100)
webview.setVisible(True)
webview.setEnabled(True)
```

## Docking Support

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt

# Create dock widget
dock = QDockWidget("My Tool", main_window)

# Create WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# Set as dock content
dock.setWidget(webview)
main_window.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## Layout Integration

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QVBoxLayout, QDialog

dialog = QDialog(parent_widget)
layout = QVBoxLayout(dialog)

webview = QtWebView(parent=dialog)
webview.load_url("http://localhost:3000")

layout.addWidget(webview)
dialog.show()
webview.show()
```

## WebView2 Pre-warming

`QtWebView` automatically pre-warms WebView2 on first instantiation:

```python
from auroraview.integration.qt import WebViewPool, QtWebView

# Explicit pre-warm (optional)
WebViewPool.prewarm()

# Check status
if WebViewPool.has_prewarmed():
    print(f"Pre-warm took {WebViewPool.get_prewarm_time():.2f}s")

# Disable auto-prewarm
webview = QtWebView(parent=parent, auto_prewarm=False)

# Cleanup (automatic on exit)
WebViewPool.cleanup()
```

## Lifecycle Management

`QtWebView` automatically handles cleanup when the parent widget is destroyed:

```python
# WebView closes automatically when parent is destroyed
webview = QtWebView(parent=dcc_main_window())
webview.show()
```

## Example: DCC Tool

```python
from auroraview import QtWebView

class MyDCCTool(QtWebView):
    def __init__(self, parent=None):
        super().__init__(parent=parent, width=400, height=600)
        self.load_url("http://localhost:3000")
        self._setup_api()

    def _setup_api(self):
        self.bind_api(self)

    def get_scene_data(self) -> dict:
        """API method callable from JavaScript"""
        return {"objects": ["cube", "sphere"]}

    def select_object(self, name: str = "") -> dict:
        """API method callable from JavaScript"""
        # DCC-specific selection logic
        return {"ok": True, "selected": name}

# Usage
tool = MyDCCTool(parent=dcc_main_window())
tool.show()
```

## Signals

`QtWebView` emits Qt signals for various events:

```python
# Connect to Qt signals
webview.destroyed.connect(lambda: print("WebView destroyed"))
```

## Thread Safety

`QtWebView` is designed to be used from the main Qt thread. All WebView operations should be performed on the main thread.

For background operations, use Qt's signal/slot mechanism:

```python
from qtpy.QtCore import QThread, Signal

class Worker(QThread):
    result_ready = Signal(dict)

    def run(self):
        # Background work
        result = {"data": "processed"}
        self.result_ready.emit(result)

# Connect to WebView
worker = Worker()
worker.result_ready.connect(lambda data: webview.emit("data_ready", data))
worker.start()
```
