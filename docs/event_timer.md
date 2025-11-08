# EventTimer - 自动事件处理

## 概述

`EventTimer` 是一个基于定时器的事件处理器，专为 DCC 应用中的 WebView 嵌入模式设计。它提供了一个自动化的事件循环，无需手动调用 `process_events()`。

**重要说明**：EventTimer 不使用后台线程，而是利用宿主应用的事件循环来避免 Rust PyO3 绑定的线程安全问题。所有事件处理都在主线程中进行，确保线程安全。

## 支持的定时器后端

EventTimer 支持三种定时器后端，按优先级自动选择：

1. **Qt QTimer**（最佳）- 精确间隔控制，跨平台，自动处理
2. **Windows SetTimer**（通用）- 原生 Windows API，无依赖，需要手动调用 `process_timer_messages()`
3. **Maya scriptJob**（DCC 特定）- Maya 专用，自动处理，但频率不可控

这意味着 EventTimer 可以在**任何环境**中使用：
- ✅ Maya（Qt QTimer 或 Maya scriptJob）
- ✅ Houdini（Qt QTimer 或 Windows SetTimer）
- ✅ 3ds Max（Windows SetTimer）
- ✅ Blender（Windows SetTimer）
- ✅ 任何 Windows 应用（Windows SetTimer）

## 为什么需要 EventTimer？

### 传统方法的问题

在嵌入模式下，传统的事件处理需要手动调用 `process_events()`：

```python
# Maya 示例 - 传统方法
import maya.cmds as cmds

webview = WebView(parent_hwnd=maya_hwnd, embedded=True)
webview.show()

# 需要创建 scriptJob 来定期处理事件
def process_events():
    if webview.process_events():
        # 窗口关闭
        cmds.scriptJob(kill=job_id)

job_id = cmds.scriptJob(event=["idle", process_events])
```

**问题：**
1. 需要手动管理 scriptJob
2. 需要手动检测窗口关闭
3. 代码冗长且容易出错
4. 不同 DCC 应用需要不同的实现

### EventTimer 的优势

```python
# Maya 示例 - 使用 EventTimer
from auroraview import WebView, EventTimer

webview = WebView(parent_hwnd=maya_hwnd, embedded=True)
webview.show()

# 创建定时器 - 自动处理所有事件
timer = EventTimer(webview, interval_ms=16)

@timer.on_close
def handle_close():
    print("窗口已关闭")
    timer.stop()

timer.start()
```

**优势：**
1. ✅ 自动处理窗口消息
2. ✅ 自动检测窗口关闭
3. ✅ 统一的 API，适用于所有 DCC 应用
4. ✅ 支持上下文管理器
5. ✅ 可配置的刷新率（默认 60 FPS）

## 工作原理

### 多策略消息检测

EventTimer 使用三种策略来检测窗口关闭：

```rust
// Rust 端实现
pub fn process_messages_enhanced(hwnd_value: u64) -> bool {
    // 策略 1: 检查窗口有效性
    if !IsWindow(hwnd).as_bool() {
        return true;  // 窗口已失效
    }

    // 策略 2: 处理窗口特定消息
    while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
        if msg.message == WM_CLOSE || msg.message == WM_DESTROY {
            return true;
        }
        // 分发消息
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    // 策略 3: 处理线程级消息
    while PeekMessageW(&mut msg, NULL, 0, 0, PM_REMOVE).as_bool() {
        if msg.message == WM_QUIT {
            return true;
        }
        // 分发消息
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    return false;
}
```

### 为什么需要多策略？

1. **窗口特定消息** (`PeekMessageW(hwnd, ...)`):
   - 捕获发送到特定窗口的消息（WM_CLOSE, WM_DESTROY）
   - 适用于用户点击关闭按钮的情况

2. **窗口有效性检查** (`IsWindow(hwnd)`):
   - 检测窗口是否仍然存在
   - 适用于窗口被外部销毁的情况

3. **线程消息** (`PeekMessageW(NULL, ...)`):
   - 捕获线程级别的消息（WM_QUIT）
   - 适用于应用程序退出的情况

## API 参考

### EventTimer

```python
class EventTimer:
    """定时器事件处理器"""

    def __init__(
        self,
        webview: WebView,
        interval_ms: int = 16,
        check_window_validity: bool = True
    ):
        """初始化事件定时器

        Args:
            webview: WebView 实例
            interval_ms: 定时器间隔（毫秒），默认 16ms (~60 FPS)
            check_window_validity: 是否检查窗口有效性
        """

    def start(self) -> None:
        """启动定时器"""

    def stop(self) -> None:
        """停止定时器"""

    def on_close(self, callback: Callable[[], None]) -> Callable:
        """注册窗口关闭回调

        Example:
            @timer.on_close
            def handle_close():
                print("窗口已关闭")
        """

    def on_tick(self, callback: Callable[[], None]) -> Callable:
        """注册定时器 tick 回调

        Example:
            @timer.on_tick
            def handle_tick():
                print("Tick")
        """

    @property
    def is_running(self) -> bool:
        """检查定时器是否运行中"""

    @property
    def interval_ms(self) -> int:
        """获取定时器间隔"""

    @interval_ms.setter
    def interval_ms(self, value: int) -> None:
        """设置定时器间隔（需要重启定时器才生效）"""
```

## 使用示例

### 基础用法

```python
from auroraview import WebView, EventTimer

# 创建 WebView
webview = WebView(title="My App", width=800, height=600)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)

# 注册关闭回调
@timer.on_close
def handle_close():
    print("窗口已关闭")
    timer.stop()

# 启动定时器
timer.start()

# 保持主线程运行
while timer.is_running:
    time.sleep(0.1)
```

### Maya 集成

```python
import maya.cmds as cmds
from maya import OpenMayaUI as omui
import shiboken2
from auroraview import WebView, EventTimer

# 获取 Maya 主窗口 HWND
maya_window = omui.MQtUtil.mainWindow()
maya_hwnd = int(shiboken2.getCppPointer(maya_window)[0])

# 创建嵌入式 WebView
webview = WebView(
    parent_hwnd=maya_hwnd,
    embedded=True,
    width=800,
    height=600,
    url="http://localhost:3000"
)
webview.show()

# 创建定时器 - 无需 scriptJob！
timer = EventTimer(webview, interval_ms=16)

@timer.on_close
def handle_close():
    print("WebView 已关闭")
    timer.stop()

timer.start()
```

### Houdini 集成

```python
import hou
import shiboken2
from auroraview import WebView, EventTimer

# 获取 Houdini 主窗口 HWND
main_window = hou.qt.mainWindow()
houdini_hwnd = int(shiboken2.getCppPointer(main_window)[0])

# 创建嵌入式 WebView
webview = WebView(
    parent_hwnd=houdini_hwnd,
    embedded=True,
    width=800,
    height=600
)
webview.show()

# 创建定时器
timer = EventTimer(webview, interval_ms=16)

@timer.on_close
def handle_close():
    print("WebView 已关闭")
    timer.stop()

timer.start()
```

### 上下文管理器

```python
from auroraview import WebView, EventTimer

webview = WebView(title="My App", width=800, height=600)
webview.show()

# 使用上下文管理器自动管理定时器生命周期
with EventTimer(webview, interval_ms=16) as timer:
    @timer.on_close
    def handle_close():
        print("窗口已关闭")

    # 定时器会在退出 with 块时自动停止
    while timer.is_running:
        time.sleep(0.1)
```

### 自定义 Tick 回调

```python
from auroraview import WebView, EventTimer

webview = WebView(title="My App", width=800, height=600)
webview.show()

timer = EventTimer(webview, interval_ms=16)

# 每个 tick 都会调用
@timer.on_tick
def handle_tick():
    # 可以在这里执行自定义逻辑
    # 例如：更新 UI、检查状态等
    pass

@timer.on_close
def handle_close():
    timer.stop()

timer.start()
```

## 性能考虑

### 刷新率选择

- **16ms (60 FPS)**: 默认值，适合大多数场景
- **33ms (30 FPS)**: 降低 CPU 使用，适合后台运行
- **8ms (120 FPS)**: 更高响应性，适合交互密集的应用

```python
# 60 FPS - 默认
timer = EventTimer(webview, interval_ms=16)

# 30 FPS - 节能模式
timer = EventTimer(webview, interval_ms=33)

# 120 FPS - 高性能模式
timer = EventTimer(webview, interval_ms=8)
```

### CPU 使用

EventTimer 在后台线程中运行，不会阻塞主线程。每个 tick 的 CPU 使用取决于：

1. 消息队列中的消息数量
2. 窗口有效性检查（Windows API 调用）
3. 用户定义的 tick 回调

## 最佳实践

1. **总是注册 on_close 回调**：确保在窗口关闭时停止定时器
2. **使用上下文管理器**：自动管理定时器生命周期
3. **选择合适的刷新率**：平衡响应性和 CPU 使用
4. **避免在 tick 回调中执行耗时操作**：会影响定时器精度

## 故障排除

### 窗口关闭未被检测到

如果窗口关闭未被检测到，检查：

1. 是否启用了窗口有效性检查：
   ```python
   timer = EventTimer(webview, check_window_validity=True)
   ```

2. 是否注册了 on_close 回调：
   ```python
   @timer.on_close
   def handle_close():
       timer.stop()
   ```

3. 检查日志输出，查看是否有错误信息

### 定时器未运行

确保调用了 `start()` 方法：

```python
timer = EventTimer(webview)
timer.start()  # 必须调用！
```

### CPU 使用过高

降低刷新率：

```python
# 从 60 FPS 降低到 30 FPS
timer = EventTimer(webview, interval_ms=33)
```

## 与传统方法对比

| 特性 | 传统方法 (scriptJob) | EventTimer |
|------|---------------------|------------|
| 代码量 | 多 | 少 |
| 易用性 | 复杂 | 简单 |
| 跨 DCC 兼容 | 需要适配 | 统一 API |
| 自动清理 | 手动 | 自动 |
| 上下文管理器 | ❌ | ✅ |
| 窗口有效性检查 | 手动 | 自动 |
| 多策略检测 | ❌ | ✅ |

## 总结

EventTimer 提供了一个简单、可靠、高性能的事件处理解决方案，特别适合 DCC 应用中的 WebView 嵌入场景。通过多策略消息检测和自动化的生命周期管理，它大大简化了事件处理的复杂性。

