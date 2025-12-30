# Unreal Engine 集成

AuroraView 通过 Python 脚本和原生 HWND 嵌入与 Unreal Engine 集成。

## 架构

```
┌─────────────────────────────────────────────┐
│            Unreal Engine 编辑器             │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────┐ │
│  │  Slate UI   │ ◄──► │  AuroraView      │ │
│  │  容器       │      │  (WebView2)      │ │
│  └─────────────┘      └──────────────────┘ │
│         │                      │            │
│         │ HWND                 │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │      Python / 蓝图 API              │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Unreal Engine | 5.0 | 5.3+ |
| Python | 3.9 | 3.11+ |
| 操作系统 | Windows 10 | Windows 11 |

## 集成模式

Unreal Engine 使用**原生模式 (HWND)** 进行 WebView 嵌入：

- 无需 Qt 依赖
- 直接 HWND 嵌入到 Slate 容器
- 使用 `register_slate_post_tick_callback()` 进行主线程执行

## 设置指南

### 步骤 1：启用 Python 插件

1. 打开 **编辑 → 插件**
2. 搜索 "Python Editor Script Plugin"
3. 启用插件
4. 重启 Unreal 编辑器

### 步骤 2：安装 AuroraView

```python
# 在 Unreal Python 控制台或启动脚本中
import subprocess
import sys

# 安装到 Unreal 的 Python 环境
subprocess.check_call([sys.executable, "-m", "pip", "install", "auroraview"])
```

### 步骤 3：基础用法

```python
import unreal
from auroraview import WebView

# 获取编辑器窗口 HWND
def get_editor_hwnd():
    # 平台特定的 HWND 获取
    import ctypes
    return ctypes.windll.user32.GetForegroundWindow()

# 创建以 Unreal 为父窗口的 WebView
webview = WebView.create(
    title="我的 Unreal 工具",
    parent=get_editor_hwnd(),
    mode="owner",
    width=800,
    height=600,
)

webview.load_url("http://localhost:3000")
webview.show()
```

## 线程模型

### 理解 Unreal 的线程机制

Unreal Engine 有严格的线程要求：

| 线程类型 | 描述 | 安全操作 |
|----------|------|----------|
| **游戏线程** | 游戏逻辑的主线程 | 所有 Unreal API 调用 |
| **渲染线程** | GPU 操作 | 仅渲染相关 |
| **后台线程** | 异步任务 | 非 Unreal 操作 |

**关键**：大多数 Unreal Python API 调用必须在**游戏线程**中进行。

### ❌ 错误：从后台线程调用 Unreal API

```python
import threading
import unreal

def background_task():
    # 不要这样做 - 会崩溃或导致未定义行为！
    actors = unreal.EditorLevelLibrary.get_selected_level_actors()
    
thread = threading.Thread(target=background_task)
thread.start()
```

### ✅ 正确：使用线程调度器

AuroraView 为 Unreal Engine 提供了线程调度器后端：

```python
from auroraview.utils import run_on_main_thread, ensure_main_thread

@ensure_main_thread
def update_actor_transform(actor_name, location):
    """此函数始终在游戏线程运行。"""
    import unreal
    actor = unreal.EditorLevelLibrary.get_actor_reference(actor_name)
    if actor:
        actor.set_actor_location(location, False, False)

# 可以从任何线程安全调用
update_actor_transform("MyActor", unreal.Vector(100, 200, 300))
```

### 即发即忘 vs 阻塞调用

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync

# 即发即忘（非阻塞）
def create_actor():
    import unreal
    unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.StaticMeshActor,
        unreal.Vector(0, 0, 0)
    )

run_on_main_thread(create_actor)  # 立即返回

# 阻塞并获取返回值
def get_selection():
    import unreal
    return unreal.EditorLevelLibrary.get_selected_level_actors()

actors = run_on_main_thread_sync(get_selection)  # 等待结果
print(f"选中了 {len(actors)} 个 Actor")
```

### Unreal 后端实现

Unreal 调度器后端使用 Slate tick 回调：

```python
from auroraview.utils.thread_dispatcher import (
    ThreadDispatcherBackend,
    register_dispatcher_backend
)

class UnrealDispatcherBackend(ThreadDispatcherBackend):
    """Unreal Engine 的线程调度器。"""
    
    def is_available(self) -> bool:
        try:
            import unreal
            return True
        except ImportError:
            return False
    
    def run_deferred(self, func, *args, **kwargs):
        import unreal
        unreal.register_slate_post_tick_callback(
            lambda _: func(*args, **kwargs)
        )
    
    def run_sync(self, func, *args, **kwargs):
        import unreal
        import threading
        
        if self.is_main_thread():
            return func(*args, **kwargs)
        
        result = [None]
        error = [None]
        event = threading.Event()
        
        def wrapper(_):
            try:
                result[0] = func(*args, **kwargs)
            except Exception as e:
                error[0] = e
            finally:
                event.set()
        
        unreal.register_slate_post_tick_callback(wrapper)
        event.wait()
        
        if error[0]:
            raise error[0]
        return result[0]
    
    def is_main_thread(self) -> bool:
        import unreal
        return unreal.is_in_game_thread()

# 以高优先级注册
register_dispatcher_backend(UnrealDispatcherBackend, priority=150)
```

## API 通信

### Python 到 JavaScript

```python
from auroraview import WebView

class UnrealAPI:
    def get_selected_actors(self):
        """获取编辑器中当前选中的 Actor。"""
        import unreal
        actors = unreal.EditorLevelLibrary.get_selected_level_actors()
        return [{"name": a.get_name(), "class": a.get_class().get_name()} 
                for a in actors]
    
    def spawn_actor(self, class_name, location):
        """在指定位置生成 Actor。"""
        import unreal
        actor_class = unreal.load_class(None, class_name)
        loc = unreal.Vector(location['x'], location['y'], location['z'])
        return unreal.EditorLevelLibrary.spawn_actor_from_class(
            actor_class, loc
        ).get_name()

webview = WebView.create(api=UnrealAPI())
```

### JavaScript 到 Python

```javascript
// 获取选中的 Actor
const actors = await auroraview.api.get_selected_actors();
console.log('选中:', actors);

// 生成新 Actor
const name = await auroraview.api.spawn_actor(
    '/Game/MyBlueprint.MyBlueprint_C',
    { x: 0, y: 0, z: 100 }
);
```

### 线程安全的事件处理器

处理来自 JavaScript 的事件时，确保线程安全：

```python
from auroraview import WebView
from auroraview.utils import ensure_main_thread

webview = WebView.create(api=UnrealAPI())

@webview.on("transform_actor")
@ensure_main_thread
def handle_transform(data):
    """处理变换事件 - 始终在游戏线程运行。"""
    import unreal
    actor_name = data['actor']
    location = data['location']
    
    actor = unreal.EditorLevelLibrary.get_actor_reference(actor_name)
    if actor:
        actor.set_actor_location(
            unreal.Vector(location['x'], location['y'], location['z']),
            False, False
        )
```

## 故障排除

### WebView 不显示

**原因**：HWND 未正确获取或 Slate 容器未就绪。

**解决方案**：确保在创建 WebView 之前 widget 已完全构建。

### 找不到 Python 模块

**原因**：AuroraView 未安装在 Unreal 的 Python 环境中。

**解决方案**：
```python
import sys
print(sys.executable)  # 检查 Unreal 使用的 Python
# 安装到该特定 Python
```

### 主线程错误 / 崩溃

**原因**：从后台线程调用 Unreal API。

**解决方案**：使用 `@ensure_main_thread` 装饰器或 `run_on_main_thread()`。

```python
# 检查是否在游戏线程
import unreal
if unreal.is_in_game_thread():
    # 可以安全调用 Unreal API
    do_unreal_operation()
else:
    # 必须调度到游戏线程
    run_on_main_thread(do_unreal_operation)
```

### 使用 `run_on_main_thread_sync` 时死锁

**原因**：在游戏线程被阻塞时从游戏线程调用 `run_on_main_thread_sync`。

**解决方案**：调用前检查线程：

```python
from auroraview.utils import is_main_thread, run_on_main_thread_sync

def safe_get_selection():
    def _get():
        import unreal
        return unreal.EditorLevelLibrary.get_selected_level_actors()
    
    if is_main_thread():
        return _get()
    else:
        return run_on_main_thread_sync(_get)
```

## 开发状态

| 功能 | 状态 |
|------|------|
| 基础集成 | 🚧 开发中 |
| HWND 嵌入 | 🚧 开发中 |
| 线程调度器 | ✅ 已支持 |
| 编辑器工具 Widget | 📋 计划中 |
| 蓝图集成 | 📋 计划中 |

## 资源

- [Unreal Python API](https://docs.unrealengine.com/5.0/en-US/PythonAPI/)
- [Slate UI 框架](https://docs.unrealengine.com/5.0/en-US/slate-ui-framework-in-unreal-engine/)
- [编辑器脚本](https://docs.unrealengine.com/5.0/en-US/scripting-the-unreal-editor-using-python/)

## 另请参阅

- [线程调度器](../guide/thread-dispatcher) - 统一的线程调度 API
- [DCC 概览](./index) - 所有 DCC 集成概述
