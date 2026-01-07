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

## 线程安全

AuroraView 为 Blender 集成提供**自动**线程安全。Blender 要求所有 `bpy` 操作在主线程运行，AuroraView 通过 `bpy.app.timers` 处理。

::: tip 零配置
由于 `dcc_mode="auto"` 是默认值，AuroraView 会自动检测 Blender 并启用线程安全。无需任何配置！
:::

### 自动线程安全（默认）

正常使用 AuroraView 即可 - 线程安全是自动的：

```python
from auroraview import WebView
import bpy

# 检测到 Blender 时自动启用线程安全
webview = WebView(
    title="Blender 工具",
    url="http://localhost:3000",
    # dcc_mode="auto" 是默认值 - 无需指定！
)

@webview.on("create_mesh")
def handle_create(data):
    # 自动在 Blender 主线程运行！
    mesh_type = data.get("type", "cube")
    if mesh_type == "cube":
        bpy.ops.mesh.primitive_cube_add()
    elif mesh_type == "sphere":
        bpy.ops.mesh.primitive_uv_sphere_add()
    return {"ok": True, "object": bpy.context.active_object.name}

@webview.on("get_selection")
def handle_selection(data):
    selected = [obj.name for obj in bpy.context.selected_objects]
    return {"selection": selected, "count": len(selected)}
```

### 使用装饰器手动线程安全

```python
from auroraview import WebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async

webview = WebView(title="Blender 工具", url="http://localhost:3000")

@webview.on("render_frame")
@dcc_thread_safe  # 阻塞直到渲染完成
def handle_render(data):
    filepath = data.get("filepath", "/tmp/render.png")
    bpy.context.scene.render.filepath = filepath
    bpy.ops.render.render(write_still=True)
    return {"ok": True, "filepath": filepath}

@webview.on("refresh_view")
@dcc_thread_safe_async  # 即发即忘
def handle_refresh(data):
    for area in bpy.context.screen.areas:
        area.tag_redraw()
```

### 直接使用 `run_on_main_thread`

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync

# 即发即忘
def deselect_all():
    bpy.ops.object.select_all(action='DESELECT')

run_on_main_thread(deselect_all)

# 阻塞并返回值
def get_scene_objects():
    return [obj.name for obj in bpy.data.objects]

objects = run_on_main_thread_sync(get_scene_objects)
print(f"场景对象: {objects}")
```

## 另请参阅

- [线程调度器](../guide/thread-dispatcher.md)
- [浮动面板](../guide/floating-panel.md)
