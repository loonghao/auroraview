# AuroraView - 异步 DCC 集成指南

## 概述

本指南介绍如何使用 AuroraView 的异步 API (`show_async()`) 在 DCC 应用中集成 WebView，而不阻塞主线程。

## 问题背景

在 DCC 应用（如 Maya、Houdini、Blender）中集成 WebView 时，传统的 `show()` 方法会阻塞应用的主线程：

```python
# ❌ 这会阻塞 Maya 主线程
webview = WebView(title="My Tool")
webview.load_html(html_content)
webview.show()  # 阻塞！Maya 无法响应用户输入
```

## 解决方案：使用 `show_async()`

`show_async()` 方法在后台线程中启动 WebView，允许 DCC 应用的主线程继续运行：

```python
# ✓ 这不会阻塞 Maya 主线程
webview = WebView(title="My Tool")
webview.load_html(html_content)
webview.show_async()  # 立即返回，WebView 在后台运行
print("Maya is still responsive!")
```

## 基本用法

### 简单示例

```python
from auroraview import WebView

# 创建 WebView
webview = WebView(
    title="My DCC Tool",
    width=600,
    height=500
)

# 加载 HTML
webview.load_html("""
    <html>
    <body>
        <h1>Hello from DCC!</h1>
        <button onclick="alert('Button clicked!')">Click Me</button>
    </body>
    </html>
""")

# 在后台启动（非阻塞）
webview.show_async()

# 主线程继续执行
print("WebView is running in background")

# 可选：等待 WebView 关闭
webview.wait()
print("WebView closed")
```

### Maya 集成示例

```python
from auroraview import WebView
import maya.cmds as cmds

def create_maya_tool():
    """创建 Maya 工具"""
    webview = WebView(
        title="AuroraView - Maya Tool",
        width=600,
        height=500
    )
    
    # 注册事件处理
    @webview.on("create_cube")
    def handle_create_cube(data):
        size = data.get("size", 1.0)
        cmds.polyCube(w=size, h=size, d=size)
        print(f"Created cube with size: {size}")
    
    # 加载 UI
    webview.load_html("""
        <html>
        <body>
            <h1>Maya Tool</h1>
            <input type="number" id="size" value="1.0">
            <button onclick="createCube()">Create Cube</button>
            
            <script>
                function createCube() {
                    const size = document.getElementById('size').value;
                    window.dispatchEvent(new CustomEvent('create_cube', {
                        detail: { size: parseFloat(size) }
                    }));
                }
            </script>
        </body>
        </html>
    """)
    
    # 在后台启动（Maya 保持响应）
    webview.show_async()
    
    # 可选：等待用户关闭窗口
    webview.wait()
    print("Tool closed")

# 在 Maya 中调用
create_maya_tool()
```

## API 参考

### `show_async()`

在后台线程中启动 WebView。

**签名：**
```python
def show_async(self) -> None
```

**特点：**
- 非阻塞：立即返回
- 线程安全：在独立线程中运行
- 防止重复：多次调用会被忽略

**示例：**
```python
webview.show_async()
print("Returned immediately")
```

### `wait(timeout=None)`

等待 WebView 关闭。

**签名：**
```python
def wait(self, timeout: Optional[float] = None) -> bool
```

**参数：**
- `timeout`: 最大等待时间（秒），`None` 表示无限等待

**返回值：**
- `True`: WebView 已关闭
- `False`: 超时

**示例：**
```python
webview.show_async()

# 等待最多 60 秒
if webview.wait(timeout=60):
    print("WebView closed by user")
else:
    print("Timeout waiting for WebView")
```

### `close()`

关闭 WebView 窗口。

**签名：**
```python
def close(self) -> None
```

**特点：**
- 自动等待后台线程完成
- 超时 5 秒

**示例：**
```python
webview.show_async()
# ... 做一些事情 ...
webview.close()
```

## 最佳实践

### 1. 错误处理

```python
try:
    webview.show_async()
except Exception as e:
    print(f"Error: {e}")
```

### 2. 资源清理

```python
try:
    webview.show_async()
    webview.wait()
finally:
    webview.close()
```

### 3. 使用上下文管理器

```python
with WebView(title="My Tool") as webview:
    webview.load_html(html)
    webview.show_async()
    webview.wait()
# 自动清理
```

### 4. 事件处理

```python
webview = WebView(title="Tool")

@webview.on("user_action")
def handle_action(data):
    print(f"User action: {data}")

webview.load_html(html)
webview.show_async()
```

## 与 `show()` 的对比

| 特性 | `show()` | `show_async()` |
|------|---------|----------------|
| 阻塞主线程 | ✓ | ✗ |
| 立即返回 | ✗ | ✓ |
| 适合 DCC 集成 | ✗ | ✓ |
| 适合独立应用 | ✓ | ✗ |
| 线程安全 | N/A | ✓ |

## 常见问题

### Q: 如何在 Maya 中使用？

A: 在 Maya Python 控制台中运行：
```python
from examples import maya_integration_async
maya_integration_async.create_maya_tool()
```

### Q: WebView 在后台运行时能否与 Maya 通信？

A: 是的，事件处理完全支持。使用 `@webview.on()` 装饰器注册事件处理器。

### Q: 如何确保线程安全？

A: AuroraView 内部处理所有线程同步。只需使用 `show_async()` 即可。

### Q: 能否同时运行多个 WebView？

A: 可以，但建议一次只运行一个，以避免资源竞争。

## 性能考虑

- 后台线程使用 daemon 模式，应用退出时自动清理
- 事件处理在后台线程中执行
- 与 Maya 的通信通过 IPC 进行，线程安全

## 故障排除

### WebView 不显示

- 检查 HTML 内容是否有效
- 确保调用了 `load_html()` 或 `load_url()`
- 查看日志输出

### 应用崩溃

- 确保在 WebView 关闭前不要销毁对象
- 使用 `wait()` 或 `close()` 进行适当清理

### 事件未触发

- 确保事件处理器在 `show_async()` 之前注册
- 检查 JavaScript 中的事件名称是否匹配

