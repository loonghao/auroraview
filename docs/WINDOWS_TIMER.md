# Windows 定时器后端

## 概述

EventTimer 现在支持三种定时器后端，按优先级排序：

1. **Qt QTimer**（最佳）- 精确间隔控制，跨平台
2. **Windows SetTimer**（通用）- 原生 Windows API，无依赖
3. **Maya scriptJob**（DCC 特定）- Maya 专用，频率不可控

这意味着 EventTimer 可以在**任何 Windows 环境**中使用，即使没有 Qt 或 Maya！

## Windows SetTimer 后端

### 特点

✅ **无依赖**：只需要 Windows API（ctypes）  
✅ **原生实现**：使用 Windows `SetTimer` API  
✅ **精确间隔**：支持毫秒级间隔控制  
✅ **主线程安全**：所有回调在主线程中执行  
✅ **通用性强**：适用于任何 Windows DCC 应用

### 工作原理

Windows SetTimer 后端使用 Windows 消息机制：

```
1. SetTimer(hwnd, timer_id, interval_ms, NULL)
   └─> 创建定时器，每隔 interval_ms 发送 WM_TIMER 消息

2. PeekMessageW(..., WM_TIMER, WM_TIMER, PM_REMOVE)
   └─> 从消息队列中取出 WM_TIMER 消息

3. 检查 msg.wParam == timer_id
   └─> 确认是我们的定时器消息

4. 调用 self._tick()
   └─> 执行定时器回调
```

### 使用方法

#### 基本用法

```python
from auroraview import WebView
from auroraview.event_timer import EventTimer

# 创建 WebView
webview = WebView(parent_hwnd=dcc_hwnd, embedded=True)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)  # 60 FPS

@timer.on_close
def handle_close():
    print("Window closed")
    timer.stop()

# 启动定时器
timer.start()

# 在主循环中处理定时器消息
while running:
    timer.process_timer_messages()  # 处理 WM_TIMER 消息
    # ... 其他处理 ...
```

#### 自动选择后端

EventTimer 会自动选择最佳后端：

```python
timer = EventTimer(webview)
timer.start()

# 检查使用的后端
if timer._timer_type == "qt":
    print("Using Qt QTimer")
elif timer._timer_type == "windows":
    print("Using Windows SetTimer")
elif timer._timer_type == "maya":
    print("Using Maya scriptJob")
```

### 在不同 DCC 中使用

#### Houdini

```python
import hou
from auroraview import WebView
from auroraview.event_timer import EventTimer

# 获取 Houdini 主窗口 HWND
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# 创建嵌入式 WebView
webview = WebView(parent_hwnd=hwnd, embedded=True)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)
timer.start()

# 在 Houdini 的事件循环中处理定时器消息
# 方法 1: 使用 Houdini 的 timer 节点
# 方法 2: 使用 Qt QTimer（如果 Houdini 有 Qt）
# 方法 3: 手动在脚本中调用
def update():
    timer.process_timer_messages()
    # 使用 hou.ui.addEventLoopCallback 注册
```

#### 3ds Max

```python
from pymxs import runtime as rt
from auroraview import WebView
from auroraview.event_timer import EventTimer

# 获取 3ds Max 主窗口 HWND
hwnd = rt.windows.getMAXHWND()

# 创建嵌入式 WebView
webview = WebView(parent_hwnd=hwnd, embedded=True)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)
timer.start()

# 在 3ds Max 的回调中处理定时器消息
def timer_callback():
    timer.process_timer_messages()

# 注册回调
rt.callbacks.addScript(rt.Name("systemPostReset"), timer_callback)
```

#### Blender

```python
import bpy
from auroraview import WebView
from auroraview.event_timer import EventTimer

# 获取 Blender 主窗口 HWND
import ctypes
hwnd = ctypes.windll.user32.GetActiveWindow()

# 创建嵌入式 WebView
webview = WebView(parent_hwnd=hwnd, embedded=True)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)
timer.start()

# 使用 Blender 的 timer
def timer_callback():
    timer.process_timer_messages()
    return 0.016  # 16ms

bpy.app.timers.register(timer_callback)
```

#### 通用 Windows 应用

```python
from auroraview import WebView
from auroraview.event_timer import EventTimer
import time

# 创建独立 WebView
webview = WebView(title="My App", width=800, height=600)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)
timer.start()

# 简单的主循环
while timer._running:
    timer.process_timer_messages()
    time.sleep(0.001)  # 避免忙等待
```

## 性能对比

### Qt QTimer vs Windows SetTimer

| 特性 | Qt QTimer | Windows SetTimer |
|------|-----------|------------------|
| 依赖 | 需要 Qt | 仅需 Windows |
| 精度 | 非常高 | 高 |
| CPU 使用 | 低 | 低 |
| 跨平台 | 是 | 否（仅 Windows） |
| 集成难度 | 简单（Qt 自动处理） | 中等（需要手动调用） |

### 推荐使用场景

- **有 Qt 环境**：优先使用 Qt QTimer（自动选择）
- **无 Qt 环境**：使用 Windows SetTimer
- **Maya 环境**：Qt QTimer > Windows SetTimer > Maya scriptJob
- **其他 DCC**：Qt QTimer（如果有）> Windows SetTimer

## 技术细节

### SetTimer API

```python
# Windows API 签名
SetTimer(
    hwnd: HWND,        # 窗口句柄
    timer_id: UINT,    # 定时器 ID
    interval: UINT,    # 间隔（毫秒）
    callback: LPVOID   # 回调函数（我们使用 NULL，通过消息处理）
) -> UINT              # 返回定时器 ID，失败返回 0
```

### 消息处理

```python
# 从消息队列中取出 WM_TIMER 消息
PeekMessageW(
    &msg,              # 消息结构体
    hwnd,              # 窗口句柄
    WM_TIMER,          # 最小消息类型
    WM_TIMER,          # 最大消息类型
    PM_REMOVE          # 从队列中移除消息
) -> BOOL              # 有消息返回 True
```

### 定时器 ID

我们使用 Python 对象的 ID 作为定时器 ID：

```python
timer_id = id(self) & 0xFFFFFFFF  # 确保在 32 位范围内
```

这确保了每个 EventTimer 实例都有唯一的定时器 ID。

## 常见问题

### Q: 为什么需要手动调用 `process_timer_messages()`？

A: Windows SetTimer 通过消息队列发送 WM_TIMER 消息。我们需要从消息队列中取出这些消息并处理。Qt QTimer 和 Maya scriptJob 会自动处理，但 Windows SetTimer 需要手动调用。

### Q: 可以在后台线程中使用吗？

A: 不可以。所有定时器后端都在主线程中运行，以避免 Rust PyO3 的线程安全问题。

### Q: 如果忘记调用 `process_timer_messages()` 会怎样？

A: 定时器消息会堆积在消息队列中，但不会被处理。WebView 的事件循环不会运行，窗口关闭检测也不会工作。

### Q: 可以同时使用多个 EventTimer 吗？

A: 可以。每个 EventTimer 都有唯一的定时器 ID，不会冲突。

### Q: 间隔精度如何？

A: Windows SetTimer 的精度通常在 10-15ms 左右，取决于系统定时器分辨率。对于 60 FPS（16ms）的需求来说足够了。

## 示例代码

完整示例请参考：
- `examples/windows_timer_example.py` - 基本用法
- `examples/maya-outliner/` - Maya 集成示例

## 总结

Windows SetTimer 后端为 EventTimer 提供了一个**通用的、无依赖的**定时器实现，使得 AuroraView 可以在任何 Windows DCC 应用中使用，而不仅限于 Maya 或 Qt 环境。

关键优势：
- ✅ 无需额外依赖
- ✅ 原生 Windows API
- ✅ 精确的间隔控制
- ✅ 主线程安全
- ✅ 适用于所有 Windows DCC

使用建议：
- 优先让 EventTimer 自动选择后端
- 如果使用 Windows SetTimer，记得在主循环中调用 `process_timer_messages()`
- 对于 DCC 集成，尽量使用 DCC 自带的定时器机制来调用 `process_timer_messages()`

