# Qt Integration Proposal

## 概述

将 AuroraView 与 Qt 集成可以避免许多 Windows HWND 相关的问题,并提供更好的跨平台支持。

## 当前问题

### HWND 方式的问题
1. **消息处理复杂** - 需要手动处理 Windows 消息循环
2. **线程安全问题** - Child 模式需要同线程,Owner 模式有跨线程问题
3. **窗口关闭问题** - 需要特殊处理 WM_DESTROY 和 WM_NCDESTROY 消息
4. **平台限制** - 只支持 Windows

### Qt 集成的优势
1. **自动消息处理** - Qt 的事件循环自动处理所有窗口消息
2. **线程安全** - Qt 的信号/槽机制天然支持跨线程通信
3. **跨平台** - 支持 Windows, macOS, Linux
4. **更好的集成** - 与 DCC 应用(Maya, Houdini 等)的 Qt 界面无缝集成

## 实现方案

### 方案 1: QWidget 包装器

创建一个 Qt Widget 来包装 WebView:

```python
from PySide2.QtWidgets import QWidget, QVBoxLayout
from PySide2.QtCore import Qt
from auroraview import WebView

class AuroraViewWidget(QWidget):
    """Qt Widget wrapper for AuroraView"""
    
    def __init__(self, parent=None, **webview_kwargs):
        super().__init__(parent)
        
        # Get this widget's HWND
        hwnd = self.winId()
        
        # Create WebView with this widget as parent
        self.webview = WebView(
            parent_hwnd=hwnd,
            parent_mode="child",  # Use child mode for Qt widget
            **webview_kwargs
        )
        
        # Set up layout
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.setLayout(layout)
        
        # Show webview
        self.webview.show()
    
    def closeEvent(self, event):
        """Handle Qt close event"""
        self.webview.close()
        super().closeEvent(event)

# Usage in Maya
from PySide2.QtWidgets import QDialog

class MyToolDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("My Tool")
        self.resize(800, 600)
        
        # Create layout
        layout = QVBoxLayout(self)
        
        # Add AuroraView widget
        self.webview_widget = AuroraViewWidget(
            self,
            title="My Tool",
            width=800,
            height=600
        )
        layout.addWidget(self.webview_widget)
        
        # Load content
        self.webview_widget.webview.load_url("http://localhost:3000")

# Show dialog
dialog = MyToolDialog(maya_main_window())
dialog.show()
```

### 方案 2: QWindow 集成

使用 Qt 的 QWindow 来管理窗口:

```rust
// In Rust
use windows::Win32::Foundation::HWND;
use raw_window_handle::{RawWindowHandle, Win32WindowHandle};

pub fn create_qt_integrated_webview(
    qt_widget_hwnd: u64,
    config: WebViewConfig,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    // Create window as child of Qt widget
    let window_builder = WindowBuilder::new()
        .with_parent_window(qt_widget_hwnd as isize)
        .with_decorations(false)  // No decorations for embedded widget
        .with_visible(true);
    
    // ... rest of the implementation
}
```

```python
# In Python
class QtIntegratedWebView(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        
        # Create native WebView as child of this widget
        self._webview = _CoreWebView.create_qt_integrated(
            self.winId(),
            title="",
            width=self.width(),
            height=self.height()
        )
    
    def resizeEvent(self, event):
        """Handle resize"""
        if hasattr(self, '_webview'):
            # Resize WebView window to match Qt widget
            self._webview.resize(self.width(), self.height())
        super().resizeEvent(event)
```

### 方案 3: QWebEngineView 替代

完全使用 Qt 的 WebEngine:

```python
from PySide2.QtWebEngineWidgets import QWebEngineView
from PySide2.QtWebChannel import QWebChannel
from PySide2.QtCore import QObject, Slot, Signal

class WebViewBridge(QObject):
    """Bridge between JavaScript and Python"""
    
    # Signals for sending events to JavaScript
    event_received = Signal(str, dict)
    
    def __init__(self):
        super().__init__()
    
    @Slot(str, dict)
    def send_event(self, event_name, data):
        """Receive events from JavaScript"""
        print(f"Event received: {event_name}, data: {data}")
        # Handle event in Python
        self.handle_event(event_name, data)
    
    def handle_event(self, event_name, data):
        """Override this to handle events"""
        pass

class QtWebView(QWebEngineView):
    """Qt-based WebView with event bridge"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        # Create web channel for JavaScript ↔ Python communication
        self.channel = QWebChannel()
        self.bridge = WebViewBridge()
        self.channel.registerObject('bridge', self.bridge)
        self.page().setWebChannel(self.channel)
        
        # Inject bridge script
        self.inject_bridge_script()
    
    def inject_bridge_script(self):
        """Inject JavaScript bridge"""
        script = """
        new QWebChannel(qt.webChannelTransport, function(channel) {
            window.bridge = channel.objects.bridge;
            
            // Send event to Python
            window.sendEvent = function(eventName, data) {
                window.bridge.send_event(eventName, data);
            };
        });
        """
        self.page().runJavaScript(script)
    
    def on(self, event_name, callback):
        """Register event handler"""
        def handler(name, data):
            if name == event_name:
                callback(data)
        
        self.bridge.handle_event = handler

# Usage
webview = QtWebView()
webview.on('my_event', lambda data: print(f"Event: {data}"))
webview.setHtml("<html><body><button onclick=\"sendEvent('my_event', {test: 'data'})\">Click</button></body></html>")
webview.show()
```

## 对比分析

| 方案 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| QWidget 包装器 | - 保留现有 Rust 代码<br>- 最小改动 | - 仍然依赖 HWND<br>- 仍有消息处理问题 | [STAR][STAR][STAR] |
| QWindow 集成 | - 更好的 Qt 集成<br>- 自动处理 resize | - 需要修改 Rust 代码<br>- 复杂度中等 | [STAR][STAR][STAR][STAR] |
| QWebEngineView | - 完全 Qt 原生<br>- 无 HWND 问题<br>- 跨平台 | - 需要重写所有代码<br>- 放弃 wry/tao | [STAR][STAR][STAR][STAR][STAR] |

## 推荐方案: QWebEngineView

### 理由
1. **完全避免 HWND 问题** - 不再需要处理 Windows 消息
2. **Qt 原生** - 与 Maya/Houdini 等 DCC 应用完美集成
3. **跨平台** - 支持 Windows, macOS, Linux
4. **成熟稳定** - Qt WebEngine 是成熟的解决方案
5. **简化代码** - 不需要 Rust,纯 Python 实现

### 实现步骤

#### 1. 创建 Qt WebView 模块

```python
# python/auroraview/qt_webview.py

from PySide2.QtWebEngineWidgets import QWebEngineView, QWebEnginePage
from PySide2.QtWebChannel import QWebChannel
from PySide2.QtCore import QObject, Slot, Signal, QUrl
from typing import Callable, Dict, Any
import json

class EventBridge(QObject):
    """JavaScript ↔ Python event bridge"""
    
    # Signal to send events to JavaScript
    python_to_js = Signal(str, str)  # (event_name, json_data)
    
    def __init__(self):
        super().__init__()
        self._handlers: Dict[str, list[Callable]] = {}
    
    @Slot(str, str)
    def js_to_python(self, event_name: str, json_data: str):
        """Receive events from JavaScript"""
        try:
            data = json.loads(json_data) if json_data else {}
            
            # Call registered handlers
            if event_name in self._handlers:
                for handler in self._handlers[event_name]:
                    handler(data)
        except Exception as e:
            print(f"Error handling event {event_name}: {e}")
    
    def register_handler(self, event_name: str, handler: Callable):
        """Register Python event handler"""
        if event_name not in self._handlers:
            self._handlers[event_name] = []
        self._handlers[event_name].append(handler)
    
    def emit_to_js(self, event_name: str, data: Any):
        """Send event to JavaScript"""
        json_data = json.dumps(data) if data else "{}"
        self.python_to_js.emit(event_name, json_data)


class AuroraViewQt(QWebEngineView):
    """Qt-based WebView with AuroraView API compatibility"""
    
    def __init__(self, parent=None, title="AuroraView", width=800, height=600):
        super().__init__(parent)
        
        self.setWindowTitle(title)
        self.resize(width, height)
        
        # Create event bridge
        self._bridge = EventBridge()
        self._channel = QWebChannel()
        self._channel.registerObject('auroraview_bridge', self._bridge)
        self.page().setWebChannel(self._channel)
        
        # Inject bridge script after page load
        self.loadFinished.connect(self._inject_bridge)
    
    def _inject_bridge(self):
        """Inject JavaScript bridge"""
        script = """
        new QWebChannel(qt.webChannelTransport, function(channel) {
            window.auroraview = {
                // Send event to Python
                send_event: function(eventName, data) {
                    var jsonData = JSON.stringify(data || {});
                    channel.objects.auroraview_bridge.js_to_python(eventName, jsonData);
                },
                
                // Receive events from Python
                on: function(eventName, callback) {
                    channel.objects.auroraview_bridge.python_to_js.connect(function(name, jsonData) {
                        if (name === eventName) {
                            var data = JSON.parse(jsonData);
                            callback(data);
                        }
                    });
                }
            };
            
            console.log('AuroraView bridge initialized');
        });
        """
        self.page().runJavaScript(script)
    
    def on(self, event_name: str) -> Callable:
        """Decorator to register event handler (AuroraView API compatibility)"""
        def decorator(func: Callable) -> Callable:
            self._bridge.register_handler(event_name, func)
            return func
        return decorator
    
    def register_callback(self, event_name: str, callback: Callable):
        """Register event handler (AuroraView API compatibility)"""
        self._bridge.register_handler(event_name, callback)
    
    def emit(self, event_name: str, data: Any = None):
        """Send event to JavaScript (AuroraView API compatibility)"""
        self._bridge.emit_to_js(event_name, data)
    
    def load_url(self, url: str):
        """Load URL"""
        self.setUrl(QUrl(url))
    
    def load_html(self, html: str):
        """Load HTML content"""
        self.setHtml(html)
```

#### 2. 使用示例

```python
# In Maya
from auroraview.qt_webview import AuroraViewQt

# Create WebView
webview = AuroraViewQt(
    parent=maya_main_window(),
    title="My Tool",
    width=800,
    height=600
)

# Register event handler
@webview.on('export_scene')
def handle_export(data):
    print(f"Exporting to: {data['path']}")
    # Do export...

# Load content
webview.load_html("""
<html>
<body>
    <button onclick="window.auroraview.send_event('export_scene', {path: '/tmp/scene.ma'})">
        Export Scene
    </button>
</body>
</html>
""")

# Show window
webview.show()
```

## 迁移路径

### 短期(保持兼容)
1. 保留现有的 Rust/wry 实现
2. 添加 Qt WebView 作为可选方案
3. 用户可以选择使用哪种实现

### 中期(推荐 Qt)
1. 文档中推荐使用 Qt 版本
2. 修复 Rust 版本的已知问题
3. 两个版本并行维护

### 长期(完全迁移)
1. 废弃 Rust/wry 实现
2. 只维护 Qt 版本
3. 简化代码库

## 总结

Qt 集成方案可以完全避免当前的 HWND 相关问题,并提供更好的跨平台支持和 DCC 集成。推荐采用 QWebEngineView 方案,作为长期解决方案。

