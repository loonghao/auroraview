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
import subprocess
import sys
subprocess.check_call([sys.executable, "-m", "pip", "install", "auroraview"])
```

### 步骤 3：基础用法

```python
import unreal
from auroraview import WebView

def get_editor_hwnd():
    import ctypes
    return ctypes.windll.user32.GetForegroundWindow()

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

## 线程调度器

```python
from auroraview.utils import ensure_main_thread

@ensure_main_thread
def update_actor_transform(actor_name, location):
    """此函数始终在游戏线程运行"""
    import unreal
    actor = unreal.EditorLevelLibrary.get_actor_reference(actor_name)
    if actor:
        actor.set_actor_location(location, False, False)

# 可以从任何线程安全调用
update_actor_transform("MyActor", unreal.Vector(100, 200, 300))
```

## API 通信

```python
from auroraview import WebView
import unreal

class UnrealAPI:
    def get_selected_actors(self):
        """获取编辑器中选中的 Actor"""
        actors = unreal.EditorLevelLibrary.get_selected_level_actors()
        return [{"name": a.get_name(), "class": a.get_class().get_name()} 
                for a in actors]
    
    def spawn_actor(self, class_name, location):
        """在指定位置生成 Actor"""
        actor_class = unreal.load_class(None, class_name)
        loc = unreal.Vector(location['x'], location['y'], location['z'])
        return unreal.EditorLevelLibrary.spawn_actor_from_class(
            actor_class, loc
        ).get_name()

webview = WebView.create(api=UnrealAPI())
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

- [线程调度器](../guide/thread-dispatcher.md)
- [DCC 概览](./index.md)
