# 统一 WebView API

AuroraView 提供了一个统一的 API，可以根据您的使用场景自动选择合适的 WebView 实现。这简化了开发流程，无需在 `WebView`、`QtWebView` 和 `AuroraView` 类之间做选择。

## 快速开始

```python
from auroraview import create_webview

# 1. 独立窗口（无父窗口）
webview = create_webview(url="http://localhost:3000")
webview.show()

# 2. Qt 集成（传入 QWidget 父窗口）
webview = create_webview(parent=maya_main_window(), url="http://localhost:3000")
webview.show()

# 3. HWND 集成（传入整数 HWND）
webview = create_webview(parent=unreal_hwnd, url="http://localhost:3000")
webview.show()
```

## 工作原理

`create_webview()` 函数会自动检测父窗口类型并选择合适的实现：

| 父窗口类型 | 实现 | 使用场景 |
|------------|------|----------|
| `None` | `WebView` | 独立桌面应用 |
| `QWidget` | `QtWebView` | Qt 系 DCC（Maya、Houdini、Nuke）|
| `int` (HWND) | `WebView` 嵌入模式 | Unreal Engine、自定义应用 |

## 统一参数

所有参数在不同实现间保持一致：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `parent` | `QWidget/int/None` | `None` | 父窗口或 HWND |
| `title` | `str` | `"AuroraView"` | 窗口标题 |
| `width` | `int` | `800` | 窗口宽度（像素）|
| `height` | `int` | `600` | 窗口高度（像素）|
| `url` | `str` | `None` | 要加载的 URL |
| `html` | `str` | `None` | 要加载的 HTML 内容 |
| `debug` | `bool` | `True` | 启用开发者工具（F12）|
| `context_menu` | `bool` | `True` | 启用右键菜单 |
| `frame` | `bool` | `True` | 显示窗口边框 |
| `transparent` | `bool` | `False` | 透明背景 |
| `background_color` | `str` | `None` | 背景颜色（CSS 格式）|
| `asset_root` | `str` | `None` | 自定义协议根目录 |
| `allow_file_protocol` | `bool` | `False` | 启用 file:// 协议 |
| `mode` | `str` | `"auto"` | 嵌入模式 |
| `api` | `Any` | `None` | 暴露给 JS 的 API 对象 |

## 嵌入模式

`mode` 参数控制 WebView 与父窗口的关系：

| 模式 | 说明 | 适用场景 |
|------|------|----------|
| `"none"` | 独立窗口 | 桌面应用 |
| `"child"` | WS_CHILD 嵌入 | 紧密 Qt 集成 |
| `"owner"` | GWLP_HWNDPARENT | 跨线程安全，DCC 工具 |
| `"auto"` | 自动检测 | 默认（推荐）|

当 `mode="auto"`（默认）时：
- 无父窗口 → `"none"`
- QWidget 父窗口 → `"child"`
- HWND 父窗口 → `"owner"`

---

## 窗口显示 (show)

AuroraView 提供了一个**统一的 `show()` 方法**，适用于所有场景。您无需记住不同用例的不同方法。

### 基本用法

```python
from auroraview import create_webview

# 所有场景都使用相同的 show() 方法
webview = create_webview(url="http://localhost:3000")
webview.show()  # 就这么简单！
```

### show() 的工作原理

`show()` 方法会自动检测您的环境并采取适当的行为：

| 场景 | 行为 | 是否阻塞？ |
|------|------|-----------|
| 独立模式（无父窗口）| 打开窗口，运行事件循环 | 是（阻塞直到关闭）|
| Qt Widget 父窗口 | 显示控件，启动事件计时器 | 否（立即返回）|
| HWND 父窗口 | 打开嵌入窗口 | 否（立即返回）|
| 打包模式（.exe）| 作为 API 服务器运行 | 是（阻塞等待请求）|

### 独立模式

对于独立桌面应用，`show()` 会阻塞直到窗口关闭：

```python
from auroraview import create_webview

webview = create_webview(
    url="http://localhost:3000",
    title="我的桌面应用",
    width=1024,
    height=768
)

# 在此阻塞，直到用户关闭窗口
webview.show()

print("窗口已关闭！")  # 窗口关闭后执行
```

**强制非阻塞**（高级用法）：

```python
webview.show(wait=False)  # 立即返回
# 警告：脚本退出时窗口会关闭！
# 保持脚本运行：
input("按回车键退出...")
```

### Qt 集成模式

使用 Qt 父窗口时，`show()` 立即返回并与 Qt 事件循环集成：

```python
from auroraview import create_webview
from PySide2.QtWidgets import QMainWindow, QDockWidget

class MyDCCTool(QMainWindow):
    def __init__(self):
        super().__init__()

        # 使用 Qt 父窗口创建 WebView
        self.webview = create_webview(
            parent=self,  # 传入 Qt 控件作为父窗口
            url="http://localhost:3000",
            title="我的工具"
        )

        # 添加到停靠面板
        dock = QDockWidget("Web 面板", self)
        dock.setWidget(self.webview)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)

        # 无需调用 show() - WebView 通过 showEvent 自动初始化
        # 当父窗口显示时自动触发
```

**Qt 集成要点：**

1. **通过 `showEvent` 自动初始化** - WebView 在 Qt 控件变为可见时自动初始化（标准 Qt 语义）
2. **无需显式调用 `show()`** - 嵌入布局/停靠面板时，显示父窗口会触发 `showEvent`
3. **Qt 管理生命周期** - WebView 遵循 Qt 控件生命周期
4. **事件计时器** - AuroraView 启动计时器处理 WebView 事件

### 嵌入 Qt 布局

将 WebView 嵌入 Qt 布局时，有两种方式：

**方式 1：直接嵌入（推荐）**

```python
from auroraview import create_webview
from PySide2.QtWidgets import QWidget, QVBoxLayout

class MyPanel(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

        layout = QVBoxLayout(self)

        # 创建 WebView 作为子控件
        self.webview = create_webview(
            parent=self,
            url="http://localhost:3000"
        )
        layout.addWidget(self.webview)

        # 无需调用 show() - 父窗口显示时自动触发
```

**方式 2：显式调用 show（很少需要）**

```python
# 仅当需要在父窗口显示前强制初始化时：
self.webview = create_webview(parent=self, url="http://localhost:3000")
layout.addWidget(self.webview)
self.webview.show()  # 强制立即初始化（通常不需要）
```

### DCC 特定集成

#### Maya

```python
from auroraview import create_webview
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

def get_maya_window():
    ptr = omui.MQtUtil.mainWindow()
    return wrapInstance(int(ptr), QWidget)

# 创建为 Maya 主窗口的子窗口
webview = create_webview(
    parent=get_maya_window(),
    url="http://localhost:3000",
    title="Maya 工具"
)
# 无需显式调用 show() - 当 Maya 窗口布局更新时自动初始化
```

#### Houdini

```python
from auroraview import create_webview
import hou

webview = create_webview(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    title="Houdini 工具"
)
webview.show()  # 非阻塞
```

#### Blender（浮动窗口）

```python
from auroraview import create_webview

# Blender 不使用 Qt，所以我们创建浮动窗口
webview = create_webview(
    url="http://localhost:3000",
    title="Blender 工具",
    always_on_top=True  # 保持在 Blender 之上
)
webview.show(wait=False)  # 非阻塞以便与 Blender 集成
```

#### Unreal Engine

```python
from auroraview import create_webview
import unreal

hwnd = unreal.get_editor_window_hwnd()

webview = create_webview(
    parent=hwnd,  # 直接传入 HWND
    url="http://localhost:3000",
    mode="owner"  # 使用 owner 模式确保跨线程安全
)
webview.show()  # 非阻塞
```

### 总结：一个方法，所有场景

| 用例 | 代码 | 说明 |
|------|------|------|
| 桌面应用 | `webview.show()` | 阻塞直到关闭 |
| Qt 停靠面板 | `webview.show()` | 非阻塞，Qt 生命周期 |
| Qt 布局子控件 | 添加到布局，父窗口显示 | 自动初始化 |
| Maya/Houdini | `webview.show()` | 非阻塞 |
| Blender | `webview.show(wait=False)` | 浮动窗口 |
| Unreal | `webview.show()` | 非阻塞，HWND 嵌入 |

**记住：** 只需使用 `show()` - AuroraView 会处理其余的一切！

---

## API 绑定

AuroraView 提供两种方式将 Python 函数暴露给 JavaScript：

### bind_call - 绑定单个函数

使用 `bind_call()` 绑定单个 Python 函数：

```python
from auroraview import create_webview

webview = create_webview(url="http://localhost:3000")

# 方式 1：直接绑定
def echo(message: str) -> str:
    return f"Echo: {message}"

webview.bind_call("api.echo", echo)

# 方式 2：装饰器风格
@webview.bind_call("api.greet")
def greet(name: str) -> str:
    return f"Hello, {name}!"

webview.show()
```

**JavaScript 端：**

```javascript
// 调用绑定的函数
const result = await auroraview.api.echo({ message: "Hello" });
console.log(result);  // "Echo: Hello"

const greeting = await auroraview.api.greet({ name: "World" });
console.log(greeting);  // "Hello, World!"
```

### bind_api - 绑定对象方法

使用 `bind_api()` 暴露对象的所有公共方法：

```python
from auroraview import create_webview

class MyAPI:
    def echo(self, message: str) -> str:
        return f"Echo: {message}"

    def add(self, a: int, b: int) -> int:
        return a + b

    def get_data(self) -> dict:
        return {"status": "ok", "count": 42}

    def _private_method(self):
        """以 _ 开头的方法不会被暴露"""
        pass

api = MyAPI()
webview = create_webview(url="http://localhost:3000", api=api)
# 或者：webview.bind_api(api)
webview.show()
```

**JavaScript 端：**

```javascript
const echo = await auroraview.api.echo({ message: "test" });
const sum = await auroraview.api.add({ a: 1, b: 2 });
const data = await auroraview.api.get_data();
```

### 参数传递约定

| Python 调用 | params 类型 | JavaScript 调用 |
|-------------|-------------|-----------------|
| `func()` | None | `auroraview.api.func()` |
| `func(**params)` | `dict` | `auroraview.api.func({key: value})` |
| `func(*params)` | `list` | `auroraview.api.func([arg1, arg2])` |
| `func(params)` | 其他 | `auroraview.api.func(value)` |

### 幂等性和重新绑定

```python
# 安全：bind_api 在命名空间级别是幂等的
webview.bind_api(api)  # 首次绑定
webview.bind_api(api)  # 静默跳过（已绑定）

# 如需强制重新绑定
webview.bind_api(api, allow_rebind=True)

# 检查绑定状态
if webview.is_namespace_bound("api"):
    print("API 命名空间已绑定")

if webview.is_method_bound("api.echo"):
    print("echo 方法已绑定")

# 列出所有已绑定的方法
methods = webview.get_bound_methods()
print(methods)  # ["api.echo", "api.add", "api.get_data"]
```

---

## 线程安全

### DCC 线程安全模式

DCC 应用（Maya、Houdini、Blender 等）要求 UI 操作在主线程运行。AuroraView 提供自动线程安全：

```python
from auroraview import create_webview

# 自动检测 DCC 环境（默认）
webview = create_webview(parent=maya_hwnd, dcc_mode="auto")

# 显式模式
webview = create_webview(parent=maya_hwnd, dcc_mode=True)   # 始终启用
webview = create_webview(parent=maya_hwnd, dcc_mode=False)  # 禁用（独立应用）
```

### 线程安全的事件处理器

当启用 `dcc_mode` 时，事件处理器自动在 DCC 主线程运行：

```python
@webview.on("create_object")
def handle_create(data):
    # 自动在 Maya 主线程运行！
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### 手动线程调度

对于精细控制，使用线程调度工具：

```python
from auroraview.utils.thread_dispatcher import (
    run_on_main_thread,
    run_on_main_thread_sync,
    dcc_thread_safe,
    is_main_thread,
)

# 即发即忘执行
def update_viewport():
    import maya.cmds as cmds
    cmds.refresh()

run_on_main_thread(update_viewport)

# 阻塞执行并获取返回值
def get_selection():
    import maya.cmds as cmds
    return cmds.ls(selection=True)

selected = run_on_main_thread_sync(get_selection)

# 装饰器风格
@dcc_thread_safe
def safe_operation():
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### 线程安全的 WebView 包装器

用于跨线程 WebView 操作：

```python
webview = create_webview(parent=dcc_hwnd)
webview.show()

# 获取线程安全包装器
safe = webview.thread_safe()

# 以下可从任意线程调用：
safe.eval_js("updateUI()")
safe.emit("status", {"ready": True})
safe.load_url("https://example.com")

# 阻塞式 JavaScript 执行
title = safe.eval_js_sync("document.title", timeout_ms=5000)
```

### API 绑定线程安全

所有绑定操作都受锁保护：

```python
# 线程安全：多线程可以调用 bind_call/bind_api
import threading

def bind_in_thread():
    webview.bind_call("api.method", some_function)

threads = [threading.Thread(target=bind_in_thread) for _ in range(10)]
for t in threads:
    t.start()
for t in threads:
    t.join()
```

---

## 高级场景

### 自定义命名空间

```python
class SceneAPI:
    def export(self, path: str) -> bool:
        # 导出逻辑
        return True

class ToolAPI:
    def apply(self, settings: dict) -> dict:
        # 应用工具设置
        return {"status": "applied"}

webview.bind_api(SceneAPI(), namespace="scene")
webview.bind_api(ToolAPI(), namespace="tool")
```

**JavaScript：**

```javascript
await auroraview.scene.export({ path: "/tmp/scene.fbx" });
await auroraview.tool.apply({ settings: { strength: 0.8 } });
```

### 错误处理

Python 异常自动传播到 JavaScript：

```python
@webview.bind_call("api.risky_operation")
def risky_operation():
    raise ValueError("出错了")
```

**JavaScript：**

```javascript
try {
    await auroraview.api.risky_operation();
} catch (error) {
    console.error(error.name);     // "ValueError"
    console.error(error.message);  // "出错了"
}
```

### 使用 Channel 进行异步操作

用于流式数据或长时间运行的操作：

```python
@webview.bind_call("api.process_large_file")
def process_large_file(path: str):
    channel = webview.create_channel()

    def process():
        with open(path, 'rb') as f:
            total = os.path.getsize(path)
            processed = 0
            while chunk := f.read(8192):
                processed += len(chunk)
                channel.send({"progress": processed / total * 100})
            channel.close()

    threading.Thread(target=process).start()
    return {"channel_id": channel.id}
```

**JavaScript：**

```javascript
const { channel_id } = await auroraview.api.process_large_file({ path: "/large/file" });
auroraview.on(`channel:${channel_id}`, (data) => {
    console.log(`进度: ${data.progress}%`);
});
```

### 热重载支持

绑定的函数支持热重载场景：

```python
def echo_v1(message: str) -> str:
    return f"v1: {message}"

webview.bind_call("api.echo", echo_v1)

# 稍后更新函数
def echo_v2(message: str) -> str:
    return f"v2: {message}"

webview.bind_call("api.echo", echo_v2, allow_rebind=True)
# 现在 JavaScript 调用将使用 echo_v2
```

---

## DCC 集成示例

### Maya 集成

```python
from auroraview import create_webview
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

def get_maya_window():
    ptr = omui.MQtUtil.mainWindow()
    return wrapInstance(int(ptr), QWidget)

class MayaAPI:
    def create_cube(self) -> str:
        import maya.cmds as cmds
        return cmds.polyCube()[0]

    def get_selection(self) -> list:
        import maya.cmds as cmds
        return cmds.ls(selection=True)

    def export_fbx(self, path: str) -> bool:
        import maya.cmds as cmds
        cmds.file(path, exportSelected=True, type="FBX export")
        return True

webview = create_webview(
    parent=get_maya_window(),
    url="http://localhost:3000",
    title="我的 Maya 工具",
    api=MayaAPI()
)
webview.show()
```

### Houdini 集成

```python
from auroraview import create_webview
import hou

class HoudiniAPI:
    def create_node(self, node_type: str, name: str) -> str:
        obj = hou.node("/obj")
        node = obj.createNode(node_type, name)
        return node.path()

    def get_selected_nodes(self) -> list:
        return [n.path() for n in hou.selectedNodes()]

webview = create_webview(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    title="我的 Houdini 工具",
    api=HoudiniAPI()
)
webview.show()
```

### Unreal Engine 集成

```python
from auroraview import create_webview
import unreal

hwnd = unreal.get_editor_window_hwnd()

class UnrealAPI:
    def spawn_actor(self, class_name: str, location: list) -> str:
        # 生成 Actor 逻辑
        return "actor_id"

    def get_selected_actors(self) -> list:
        return [str(a) for a in unreal.EditorLevelLibrary.get_selected_level_actors()]

webview = create_webview(
    parent=hwnd,
    url="http://localhost:3000",
    title="我的 Unreal 工具",
    mode="owner",
    api=UnrealAPI()
)
webview.show()
```

### 独立桌面应用

```python
from auroraview import create_webview, run_app

webview = create_webview(
    url="http://localhost:3000",
    title="我的应用",
    width=1024,
    height=768
)
webview.show()

# 或使用便捷函数
run_app(url="http://localhost:3000", title="我的应用")
```

---

## 迁移指南

### 从 WebView 迁移

```python
# 之前
from auroraview.core import WebView
webview = WebView(
    title="工具",
    parent_hwnd=hwnd,
    embed_mode="owner",
    dev_tools=True,
    decorations=False
)

# 之后
from auroraview import create_webview
webview = create_webview(
    title="工具",
    parent=hwnd,
    mode="owner",
    debug=True,
    frame=False
)
```

### 从 QtWebView 迁移

```python
# 之前
from auroraview import QtWebView
webview = QtWebView(
    parent=widget,
    dev_tools=True,
    frameless=True
)

# 之后
from auroraview import create_webview
webview = create_webview(
    parent=widget,
    debug=True,
    frame=False
)
```

### 从 AuroraView 迁移

```python
# 之前
from auroraview import AuroraView
webview = AuroraView(
    url="http://localhost:3000",
    debug=True,
    api=my_api
)

# 之后
from auroraview import create_webview
webview = create_webview(
    url="http://localhost:3000",
    debug=True,
    api=my_api
)
```

## 参数映射

统一 API 规范化了参数名称：

| 统一参数 | WebView | QtWebView | AuroraView |
|----------|---------|-----------|------------|
| `debug` | `debug` / `dev_tools` | `dev_tools` | `debug` |
| `frame` | `frame` / `decorations` | `frameless`（取反）| - |
| `parent` | `parent` / `parent_hwnd` | `parent` | `parent` / `parent_hwnd` |
| `mode` | `mode` / `embed_mode` | `embed_mode` | `embed_mode` |

## 向后兼容

旧版 API（`WebView`、`QtWebView`、`AuroraView`）仍然完全支持：

```python
# 以下方式仍然有效
from auroraview import WebView, QtWebView, AuroraView
from auroraview.core import WebView
from auroraview.integration import QtWebView, AuroraView
```

---

## 最佳实践

### API 设计

1. **使用描述性方法名** - `export_scene()` 而非 `exp()`
2. **返回结构化数据** - 返回字典而非多个值
3. **优雅处理错误** - 抛出有意义的异常
4. **文档化参数** - 使用类型提示和文档字符串

```python
class WellDesignedAPI:
    def export_scene(self, path: str, format: str = "fbx") -> dict:
        """导出当前场景。

        Args:
            path: 导出文件路径
            format: 导出格式（fbx, obj, gltf）

        Returns:
            包含 success, path, size 键的字典
        """
        # 实现
        return {"success": True, "path": path, "size": 1024}
```

### 线程安全

1. **使用 `dcc_mode="auto"`** - 让 AuroraView 检测环境
2. **避免阻塞主线程** - 对长时间操作使用 channel
3. **使用 `thread_safe()` 包装器** - 用于跨线程 WebView 访问

### 性能

1. **批量 API 调用** - 减少 JS 和 Python 之间的往返
2. **使用 channel 进行流式传输** - 不要轮询进度
3. **延迟加载重型模块** - 在函数内部导入 DCC 模块

```python
class PerformantAPI:
    def batch_operation(self, items: list) -> list:
        """在一次调用中处理多个项目"""
        return [self._process_item(item) for item in items]

    def _process_item(self, item):
        # 延迟导入
        import maya.cmds as cmds
        return cmds.polyCube()[0]
```

### 可维护性

1. **按领域分离 API 类** - `SceneAPI`、`ToolAPI`、`RenderAPI`
2. **使用命名空间** - `bind_api(api, namespace="scene")`
3. **版本化 API** - 考虑向后兼容性

```python
# 组织良好的 API 结构
class SceneAPI:
    """场景管理操作"""
    pass

class ToolAPI:
    """工具操作"""
    pass

class RenderAPI:
    """渲染操作"""
    pass

webview.bind_api(SceneAPI(), namespace="scene")
webview.bind_api(ToolAPI(), namespace="tool")
webview.bind_api(RenderAPI(), namespace="render")
```

---

## 故障排除

### 常见问题

**问：JavaScript 调用返回 undefined**

检查：
1. 方法是否正确绑定：`webview.is_method_bound("api.method")`
2. 页面是否已加载：`webview.is_loaded()`
3. 方法名是否完全匹配（区分大小写）

**问：DCC 中的线程安全错误**

确保：
1. 设置了 `dcc_mode="auto"` 或 `dcc_mode=True`
2. 对手动处理器使用 `@dcc_thread_safe` 装饰器
3. 对跨线程访问使用 `thread_safe()` 包装器

**问：JavaScript 中 API 不可用**

验证：
1. 在 `show()` 之前调用了 `bind_api()` 或 `bind_call()`
2. 检查浏览器控制台是否有错误
3. 确保 `window.auroraview` 已就绪（使用 `auroraviewready` 事件）

```javascript
window.addEventListener('auroraviewready', () => {
    // 可以安全使用 auroraview.api.*
    console.log('AuroraView 已就绪');
});
```

**问：重复绑定警告**

使用幂等绑定：
```python
webview.bind_api(api)  # 可以安全地多次调用
# 或先检查：
if not webview.is_namespace_bound("api"):
    webview.bind_api(api)
```
