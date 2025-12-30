# Blender 集成

AuroraView 通过浮动窗口模式与 Blender 集成（Blender 不使用 Qt）。

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Blender | 3.0 | 4.0+ |
| Python | 3.10 | 3.11+ |

## 集成模式

Blender 使用**原生模式 (HWND)** 或**桌面模式**：

- 浮动工具窗口
- 无 Qt 依赖
- 使用 `bpy.app.timers` 进行主线程调度

## 快速开始

### 浮动窗口

```python
from auroraview import WebView
import bpy

# 获取 Blender 窗口 HWND
def get_blender_hwnd():
    import ctypes
    return ctypes.windll.user32.GetForegroundWindow()

webview = WebView.create(
    title="Blender 工具",
    parent=get_blender_hwnd(),
    mode="owner",
    width=400,
    height=600
)
webview.load_url("http://localhost:3000")
webview.show()
```

### 独立窗口

```python
from auroraview import run_desktop

run_desktop(
    title="Blender 工具",
    url="http://localhost:3000"
)
```

## API 通信

```python
from auroraview import WebView
import bpy

class BlenderAPI:
    def get_selected_objects(self):
        """获取选中的对象"""
        return [obj.name for obj in bpy.context.selected_objects]
    
    def create_cube(self, name="Cube", size=2.0):
        """创建立方体"""
        bpy.ops.mesh.primitive_cube_add(size=size)
        obj = bpy.context.active_object
        obj.name = name
        return obj.name

webview = WebView.create(api=BlenderAPI())
```

## 线程调度器

Blender 使用 `bpy.app.timers` 进行主线程调度：

```python
from auroraview.utils import ensure_main_thread

@ensure_main_thread
def safe_blender_operation():
    """在主线程执行 Blender 操作"""
    import bpy
    bpy.ops.object.select_all(action='DESELECT')
```

### 后端实现

```python
import bpy
import threading
from queue import Queue

class BlenderDispatcherBackend:
    def __init__(self):
        self._queue = Queue()
    
    def run_deferred(self, func, *args, **kwargs):
        def timer_callback():
            func(*args, **kwargs)
            return None  # 不重复
        bpy.app.timers.register(timer_callback)
    
    def run_sync(self, func, *args, **kwargs):
        if self.is_main_thread():
            return func(*args, **kwargs)
        
        result = [None]
        error = [None]
        event = threading.Event()
        
        def timer_callback():
            try:
                result[0] = func(*args, **kwargs)
            except Exception as e:
                error[0] = e
            finally:
                event.set()
            return None
        
        bpy.app.timers.register(timer_callback)
        event.wait()
        
        if error[0]:
            raise error[0]
        return result[0]
    
    def is_main_thread(self):
        return threading.current_thread() is threading.main_thread()
```

## 另请参阅

- [线程调度器](../guide/thread-dispatcher.md)
- [浮动面板](../guide/floating-panel.md)
