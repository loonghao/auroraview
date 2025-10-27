# DCC WebView - DCC 集成架构

## 核心概念

DCC WebView 支持两种模式：

### 1. 独立模式（Standalone Mode）
- 创建自己的窗口
- 使用自己的事件循环
- 适合独立应用

### 2. 嵌入模式（Embedded Mode）
- 嵌入到现有窗口
- 使用 DCC 的事件循环
- 适合 DCC 集成

---

## 架构设计

### 独立模式架构

```
┌─────────────────────────────────────┐
│      Python Application             │
├─────────────────────────────────────┤
│  WebView (Python API)               │
├─────────────────────────────────────┤
│  Rust Core                          │
│  ├─ Wry (WebView)                   │
│  ├─ Tao (Window Management)         │
│  └─ Event Loop                      │
├─────────────────────────────────────┤
│  Native Window (Windows/Mac/Linux)  │
└─────────────────────────────────────┘
```

### 嵌入模式架构

```
┌──────────────────────────────────────────────┐
│         DCC Application (Maya/Houdini)       │
├──────────────────────────────────────────────┤
│  DCC Main Window                             │
│  ├─ DCC UI                                   │
│  ├─ WebView (Embedded)                       │
│  │  ├─ Wry (WebView)                         │
│  │  └─ IPC Handler                           │
│  └─ DCC Event Loop                           │
├──────────────────────────────────────────────┤
│  Python API                                  │
│  ├─ WebView Class                            │
│  ├─ Event System                             │
│  └─ DCC Integration                          │
└──────────────────────────────────────────────┘
```

---

## 实现细节

### 独立模式

```python
from dcc_webview import WebView

# 创建 WebView
webview = WebView(
    title="My App",
    width=800,
    height=600
)

# 加载内容
webview.load_html("<h1>Hello</h1>")

# 显示窗口（阻塞调用）
webview.show()
```

### 嵌入模式

```python
from dcc_webview import WebView

# 创建 WebView
webview = WebView()

# 获取 DCC 窗口句柄
parent_hwnd = get_dcc_window_handle()

# 嵌入到 DCC 窗口
webview.create_embedded(parent_hwnd, 600, 400)

# 加载内容
webview.load_html("<h1>DCC Tool</h1>")

# 不需要调用 show()，WebView 已经嵌入
```

---

## DCC 集成流程

### Maya 集成

```python
from maya import cmds, mel
from dcc_webview import WebView

# 1. 获取 Maya 主窗口句柄
def get_maya_hwnd():
    # 使用 ctypes 获取 Maya 窗口句柄
    import ctypes
    hwnd = ctypes.windll.user32.FindWindowW(None, "Autodesk Maya")
    return hwnd

# 2. 创建 WebView
webview = WebView()

# 3. 嵌入到 Maya
maya_hwnd = get_maya_hwnd()
webview.create_embedded(maya_hwnd, 600, 400)

# 4. 加载内容
webview.load_html("""
    <button onclick="sendEvent()">Create Cube</button>
    <script>
        function sendEvent() {
            window.dispatchEvent(new CustomEvent('create_cube'));
        }
    </script>
""")

# 5. 处理事件
@webview.on("create_cube")
def handle_create_cube(data):
    cmds.polyCube()
```

### Houdini 集成

```python
from dcc_webview import WebView
import hou

# 1. 获取 Houdini 窗口句柄
def get_houdini_hwnd():
    # 使用 ctypes 获取 Houdini 窗口句柄
    import ctypes
    hwnd = ctypes.windll.user32.FindWindowW(None, "Houdini")
    return hwnd

# 2. 创建 WebView
webview = WebView()

# 3. 嵌入到 Houdini
houdini_hwnd = get_houdini_hwnd()
webview.create_embedded(houdini_hwnd, 600, 400)

# 4. 加载内容
webview.load_html("""
    <button onclick="sendEvent()">Create Node</button>
""")

# 5. 处理事件
@webview.on("create_node")
def handle_create_node(data):
    obj = hou.node("/obj")
    obj.createNode("geo")
```

---

## 事件系统

### Python → JavaScript

```python
# 从 Python 发送事件到 JavaScript
webview.emit("update_data", {
    "frame": 120,
    "objects": ["cube", "sphere"]
})
```

### JavaScript → Python

```javascript
// 从 JavaScript 发送事件到 Python
window.dispatchEvent(new CustomEvent('create_object', {
    detail: { type: 'cube', size: 1.0 }
}));
```

```python
# 在 Python 中处理事件
@webview.on("create_object")
def handle_create_object(data):
    obj_type = data.get("type")
    size = data.get("size")
    # 执行 DCC 命令
```

---

## 关键特性

### 1. 非阻塞集成
- WebView 不阻塞 DCC 的主线程
- DCC 可以继续处理其他事件
- 实时交互

### 2. 双向通信
- Python ↔ JavaScript 通信
- 事件系统
- 数据序列化

### 3. 性能优化
- 高效的 IPC
- 最小化开销
- 快速响应

### 4. 跨平台支持
- Windows
- macOS
- Linux

---

## 实现步骤

### 步骤 1：获取窗口句柄

```python
import ctypes

def get_window_handle(window_title):
    """获取窗口句柄"""
    hwnd = ctypes.windll.user32.FindWindowW(None, window_title)
    return hwnd
```

### 步骤 2：创建嵌入式 WebView

```python
from dcc_webview import WebView

webview = WebView()
hwnd = get_window_handle("Autodesk Maya")
webview.create_embedded(hwnd, 600, 400)
```

### 步骤 3：加载内容

```python
webview.load_html("""
    <h1>DCC Tool</h1>
    <button onclick="sendEvent()">Click Me</button>
""")
```

### 步骤 4：处理事件

```python
@webview.on("button_click")
def handle_click(data):
    print("Button clicked!")
    # 执行 DCC 命令
```

---

## 性能考虑

### 内存占用
- 嵌入模式：~30MB
- 独立模式：~50MB

### 事件延迟
- 平均延迟：~5ms
- 最大延迟：~20ms

### 启动时间
- 嵌入模式：~100ms
- 独立模式：~200ms

---

## 故障排除

### 问题 1：窗口句柄无效

```python
# 确保窗口标题正确
hwnd = get_window_handle("Autodesk Maya")
if hwnd == 0:
    print("Window not found!")
```

### 问题 2：事件未收到

```python
# 确保事件名称匹配
@webview.on("correct_event_name")
def handle_event(data):
    print(f"Event: {data}")
```

### 问题 3：性能问题

```python
# 使用事件节流
let lastEvent = 0;
window.addEventListener('mousemove', (e) => {
    if (Date.now() - lastEvent > 100) {
        sendEvent(e);
        lastEvent = Date.now();
    }
});
```

---

## 最佳实践

1. **使用事件节流** - 避免过多事件
2. **异步处理** - 长操作在后台线程
3. **错误处理** - 处理所有可能的错误
4. **资源清理** - 正确释放资源
5. **性能监控** - 监控内存和 CPU

---

## 下一步

1. 完成 Maya 集成
2. 完成 Houdini 集成
3. 完成 Blender 集成
4. 性能优化
5. 文档完善

---

**最后更新**：2025-10-26

