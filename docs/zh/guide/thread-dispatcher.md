# 线程调度器

线程调度器模块提供了一个统一的 API，用于在不同的 DCC（数字内容创作）应用程序中在主线程/UI 线程上执行代码。许多 DCC 应用程序要求某些操作必须在主线程上执行以确保线程安全。

## 概述

不同的 DCC 应用程序有不同的主线程执行 API：

| DCC 应用程序 | 延迟执行 | 阻塞执行 |
|-------------|---------|---------|
| **Maya** | `maya.utils.executeDeferred()` | `maya.utils.executeInMainThreadWithResult()` |
| **Houdini** | `hdefereval.executeDeferred()` | `hdefereval.executeInMainThread()` |
| **Blender** | `bpy.app.timers.register()` | 基于队列的方式 |
| **Nuke** | `nuke.executeInMainThread()` | `nuke.executeInMainThreadWithResult()` |
| **3ds Max** | `MaxPlus.Core.EvalOnMainThread()` | 带返回值的包装器 |
| **Unreal Engine** | `register_slate_post_tick_callback()` | 基于事件的方式 |
| **Qt 应用程序** | `QTimer.singleShot()` | 基于事件的方式 |

线程调度器抽象了这些差异，提供了一个一致的 API，能够自动检测并使用适当的后端。

## 核心特性

- **懒加载**：DCC 后端仅在需要时才加载，避免导入错误
- **字符串注册**：通过模块路径注册后端以实现懒加载
- **环境变量覆盖**：通过 `AURORAVIEW_DISPATCHER` 强制指定后端
- **优先级选择**：优先级高的后端优先尝试
- **自动检测**：自动检测当前 DCC 环境

## 快速开始

### 基本用法

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync

# 即发即忘执行（非阻塞）
def create_cube():
    import maya.cmds as cmds
    cmds.polyCube()

run_on_main_thread(create_cube)

# 阻塞执行并获取返回值
def get_selection():
    import maya.cmds as cmds
    return cmds.ls(selection=True)

selected = run_on_main_thread_sync(get_selection)
print(f"已选择: {selected}")
```

### 使用装饰器

```python
from auroraview.utils import ensure_main_thread, defer_to_main_thread

# 确保函数始终在主线程上运行
@ensure_main_thread
def update_viewport():
    import maya.cmds as cmds
    cmds.refresh()

# 可以从任何线程安全调用
update_viewport()

# 即发即忘装饰器
@defer_to_main_thread
def log_message(msg):
    print(f"[主线程] {msg}")

# 立即返回，稍后在主线程上执行
log_message("来自后台线程的问候！")
```

### 检查当前线程

```python
from auroraview.utils import is_main_thread

if is_main_thread():
    # 可以直接安全调用 DCC API
    do_dcc_operation()
else:
    # 需要调度到主线程
    run_on_main_thread(do_dcc_operation)
```

## API 参考

### 函数

#### `run_on_main_thread(func, *args, **kwargs) -> None`

在主线程上执行函数，不等待结果（即发即忘）。

**参数：**
- `func`：要执行的函数
- `*args`：位置参数
- `**kwargs`：关键字参数

**示例：**
```python
def create_sphere(radius):
    import maya.cmds as cmds
    cmds.polySphere(radius=radius)

run_on_main_thread(create_sphere, 2.0)
```

#### `run_on_main_thread_sync(func, *args, **kwargs) -> T`

在主线程上执行函数并等待结果（阻塞）。

**参数：**
- `func`：要执行的函数
- `*args`：位置参数
- `**kwargs`：关键字参数

**返回：** 函数的返回值

**异常：** 重新抛出函数中的任何异常

**示例：**
```python
def get_scene_name():
    import maya.cmds as cmds
    return cmds.file(q=True, sceneName=True)

scene = run_on_main_thread_sync(get_scene_name)
```

#### `is_main_thread() -> bool`

检查当前线程是否是主线程/UI 线程。

**返回：** 如果在主线程上返回 `True`，否则返回 `False`

#### `ensure_main_thread(func) -> Callable`

确保函数在主线程上运行的装饰器。如果从后台线程调用，函数将被调度到主线程，调用会阻塞直到完成。

**示例：**
```python
@ensure_main_thread
def safe_ui_update():
    # 这始终在主线程上运行
    update_ui_elements()
```

#### `defer_to_main_thread(func) -> Callable`

将函数执行延迟到主线程的装饰器（即发即忘）。被装饰的函数立即返回 `None`。

**示例：**
```python
@defer_to_main_thread
def async_log(message):
    print(message)

async_log("这稍后打印")  # 立即返回
```

### 后端管理

#### `get_dispatcher_backend() -> ThreadDispatcherBackend`

获取当前活动的后端。

#### `list_dispatcher_backends() -> List[Tuple[int, str, bool]]`

列出所有已注册的后端及其优先级和可用性。

**返回：** `(priority, name, is_available)` 元组列表

**示例：**
```python
for priority, name, available in list_dispatcher_backends():
    status = "+" if available else "-"
    print(f"{status} {name} (优先级={priority})")
```

#### `register_dispatcher_backend(backend, priority=0, *, name="")`

注册自定义后端。支持类注册和字符串注册两种方式。

**参数：**
- `backend`：`ThreadDispatcherBackend` 的子类，或 `"module:ClassName"` 格式的字符串路径
- `priority`：值越高越优先尝试（默认：0）
- `name`：后端的可选显示名称

**示例 - 类注册：**
```python
register_dispatcher_backend(MyDCCBackend, priority=250)
```

**示例 - 字符串注册（懒加载）：**
```python
# 仅在调用 get_dispatcher_backend() 时才加载
register_dispatcher_backend(
    "my_package.dispatchers:MyDCCBackend",
    priority=250,
    name="MyDCC"
)
```

#### `unregister_dispatcher_backend(backend) -> bool`

注销之前注册的后端。

**参数：**
- `backend`：要注销的后端类或字符串路径

**返回：** 如果找到并移除返回 `True`，否则返回 `False`

#### `clear_dispatcher_backends() -> None`

清除所有已注册的后端并重置为初始状态。主要用于测试。

## 环境变量覆盖

您可以使用 `AURORAVIEW_DISPATCHER` 环境变量强制指定后端：

```bash
# 强制使用 Qt 后端
export AURORAVIEW_DISPATCHER=qt

# 强制使用 fallback 后端
export AURORAVIEW_DISPATCHER=fallback
```

有效值（不区分大小写）：`maya`、`houdini`、`nuke`、`blender`、`max`、`unreal`、`qt`、`fallback`

## 自定义后端

### 类注册

```python
from auroraview.utils.thread_dispatcher import (
    ThreadDispatcherBackend,
    register_dispatcher_backend
)

class MyDCCBackend(ThreadDispatcherBackend):
    """MyDCC 应用程序的自定义后端。"""
    
    def is_available(self) -> bool:
        try:
            import mydcc
            return True
        except ImportError:
            return False
    
    def run_deferred(self, func, *args, **kwargs):
        import mydcc
        mydcc.execute_deferred(lambda: func(*args, **kwargs))
    
    def run_sync(self, func, *args, **kwargs):
        import mydcc
        return mydcc.execute_in_main_thread(lambda: func(*args, **kwargs))
    
    def is_main_thread(self) -> bool:
        import mydcc
        return mydcc.is_main_thread()

# 以高优先级注册
register_dispatcher_backend(MyDCCBackend, priority=250, name="MyDCC")
```

### 字符串注册（懒加载）

对于外部包或避免 DCC 不可用时的导入错误：

```python
from auroraview.utils import register_dispatcher_backend

# 通过模块路径注册 - 仅在需要时才加载
register_dispatcher_backend(
    "my_package.dispatchers:MyDCCBackend",
    priority=250,
    name="MyDCC"
)
```

这在以下情况特别有用：
- 您的后端依赖于可能未安装的 DCC 特定模块
- 您想分发一个跨多个 DCC 工作的插件
- 您需要延迟导入直到 DCC 环境完全初始化

## 内置后端

以下后端默认已注册（按优先级排序）：

| 后端 | 优先级 | 描述 |
|-----|-------|------|
| Maya | 200 | 使用 `maya.utils` 模块 |
| Houdini | 190 | 使用 `hdefereval` 模块 |
| Nuke | 180 | 使用 `nuke` 模块 |
| Blender | 170 | 使用 `bpy.app.timers` |
| 3ds Max | 160 | 使用 `MaxPlus` 或 `pymxs` |
| Unreal | 150 | 使用 slate tick 回调 |
| Qt | 100 | 使用 `QTimer.singleShot()` |
| Fallback | 0 | 直接执行（会记录警告） |

## 使用场景

### 后台处理与 UI 更新

```python
import threading
from auroraview.utils import run_on_main_thread

def process_data(data):
    """在后台线程中处理数据。"""
    result = heavy_computation(data)
    
    # 在主线程上更新 UI
    run_on_main_thread(update_progress_bar, 100)
    run_on_main_thread(show_result, result)

# 启动后台线程
thread = threading.Thread(target=process_data, args=(my_data,))
thread.start()
```

### 线程安全的事件处理器

```python
from auroraview import WebView
from auroraview.utils import ensure_main_thread

webview = WebView()

@webview.on("export_scene")
@ensure_main_thread
def handle_export(data):
    """处理导出事件 - 始终在主线程上运行。"""
    import maya.cmds as cmds
    cmds.file(data['path'], exportSelected=True, type='mayaAscii')
```

### 带返回值的异步操作

```python
from concurrent.futures import ThreadPoolExecutor
from auroraview.utils import run_on_main_thread_sync

def get_scene_info():
    """从主线程获取场景信息。"""
    return run_on_main_thread_sync(lambda: {
        'name': cmds.file(q=True, sceneName=True),
        'objects': len(cmds.ls(dag=True)),
    })

with ThreadPoolExecutor() as executor:
    future = executor.submit(get_scene_info)
    info = future.result()
```

## 最佳实践

1. **最小化主线程工作**：保持主线程操作简短，避免 UI 冻结。

2. **批量操作**：将多个 DCC 操作组合到单个主线程调用中。

3. **对即发即忘使用延迟执行**：当不需要返回值时使用 `run_on_main_thread`。

4. **处理异常**：使用 `run_sync` 时用 try/except 包装主线程操作。

5. **调度前检查线程**：使用 `is_main_thread()` 避免不必要的调度。

```python
from auroraview.utils import is_main_thread, run_on_main_thread_sync

def safe_operation():
    if is_main_thread():
        return do_operation()
    else:
        return run_on_main_thread_sync(do_operation)
```

6. **对插件使用字符串注册**：分发插件时，使用字符串注册以避免导入错误。

## 故障排除

### 函数未执行

**原因**：DCC 应用程序的事件循环未运行。

**解决方案**：确保应用程序处于交互状态，而不是在启动或关闭期间。

### 使用 `run_sync` 时死锁

**原因**：在主线程被阻塞时从主线程调用 `run_on_main_thread_sync`。

**解决方案**：调用前检查 `is_main_thread()`，或改用 `run_on_main_thread`（非阻塞）。

### 选择了错误的后端

**原因**：检测到多个 DCC 环境。

**解决方案**：以更高优先级注册自定义后端，或设置 `AURORAVIEW_DISPATCHER` 环境变量。

### DCC 模块导入错误

**原因**：DCC 特定模块在当前环境中不可用。

**解决方案**：使用字符串注册进行懒加载，或在导入前检查可用性。

## 另请参阅

- [Qt 集成](./qt-integration) - Qt 特定集成指南
- [Maya 集成](../dcc/maya) - Maya 特定指南
- [DCC 概览](../dcc/) - 所有 DCC 集成概览

## WebView 的 DCC 线程安全

当将 AuroraView WebView 集成到 DCC 应用程序时，您需要处理 WebView 线程和 DCC 主线程之间的线程安全。AuroraView 为此提供了专门的工具。

### 挑战

- **WebView 线程**：WebView2 在其自己的 STA 线程上运行
- **DCC 主线程**：DCC API（Maya cmds、Blender bpy 等）必须从主线程调用
- **事件处理器**：`@webview.on()` 处理器可能从 WebView 线程调用

### 使用 `@dcc_thread_safe` 装饰器

`@dcc_thread_safe` 装饰器自动将函数执行调度到 DCC 主线程：

```python
from auroraview import WebView
from auroraview.utils import dcc_thread_safe

webview = WebView(parent=dcc_hwnd)

@webview.on("create_object")
@dcc_thread_safe  # 确保在 DCC 主线程上运行
def handle_create(data):
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### 使用 `dcc_mode`

在 WebView 上启用 `dcc_mode` 以自动包装所有回调：

```python
# 所有回调自动在 DCC 主线程上运行
webview = WebView(parent=dcc_hwnd, dcc_mode=True)

@webview.on("create_object")
def handle_create(data):  # 不需要装饰器！
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### 线程安全包装器

使用 `thread_safe()` 进行跨线程 WebView 操作：

```python
webview = WebView(parent=dcc_hwnd)

# 获取线程安全包装器
safe = webview.thread_safe()

# 可以从任何线程调用：
safe.eval_js("updateStatus('ready')")
safe.emit("data_loaded", {"count": 100})
safe.load_url("https://example.com")
```

### 使用 `@dcc_thread_safe_async` 进行即发即忘

对于不需要返回值的操作：

```python
from auroraview.utils import dcc_thread_safe_async

@dcc_thread_safe_async
def update_viewport():
    import maya.cmds as cmds
    cmds.refresh()

# 立即返回，稍后在主线程上执行
update_viewport()
```

### 另请参阅

- [RFC 0002: DCC 线程安全](/rfcs/0002-dcc-thread-safety) - 详细设计文档
