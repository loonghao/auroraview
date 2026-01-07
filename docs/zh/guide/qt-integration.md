# Qt 集成最佳实践

本指南介绍将 AuroraView 与基于 Qt 的 DCC 应用程序（Maya、Houdini、Nuke、3ds Max 等）集成的最佳实践。

## 快速开始

### 推荐：使用 `QtWebView`

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=maya_main_window(),  # 可选：任何 QWidget
    title="My Tool",
    width=800,
    height=600,
)

webview.load_url("http://localhost:3000")
webview.show()
```

## 理解事件处理（为什么要用 `QtWebView`）

AuroraView 会通过消息队列把工作安全地派发到正确的 UI 线程，其中包括：

- `webview.eval_js(...)`
- `webview.emit(...)`
- 为 `auroraview.call(...)` 返回结果

如果队列没有被处理，JS 执行与 RPC 回包可能会被延迟。

### 解决方案

`QtWebView` 会安装 Qt 版本的事件处理器（`QtEventProcessor`），从而：

- 先处理 Qt 事件（`QCoreApplication.processEvents()`）
- 再处理 AuroraView 消息队列（`WebView.process_events()`）

默认情况下，`emit()` / `eval_js()` 之后会自动触发上述处理（除非你显式关闭 `auto_process`）。

### 避免：自建 ScriptJob / Idle Hook

一般不建议在 Maya/Houdini 里为了 `process_events()` 自己搭一套 idle 循环。
优先使用 `QtWebView`，它会为你接好正确的处理策略。

## 常见模式

### 模式 1：Python → JavaScript（推送事件）

```python
from auroraview import QtWebView

webview = QtWebView(title="My Tool")
webview.emit("update_scene", {"objects": ["cube", "sphere"]})
```

### 模式 2：JavaScript → Python（即发即忘）

```python
@webview.on("get_scene_data")
def handle_get_scene_data(data):
    selection = cmds.ls(selection=True)
    webview.emit("scene_data_response", {"selection": selection})
```

```javascript
window.auroraview.on("scene_data_response", (data) => {
  console.log("Selection:", data.selection);
});

window.auroraview.send_event("get_scene_data", {});
```

### 模式 3：JavaScript → Python（带返回值 RPC）

```python
@webview.bind_call("api.get_scene_hierarchy")
def get_scene_hierarchy(root: str = "scene"):
    return {"root": root, "nodes": []}
```

```javascript
const result = await window.auroraview.call("api.get_scene_hierarchy", { root: "scene" });
console.log("Hierarchy:", result);
```

## 诊断

### 查看事件处理器状态

```python
diag = webview.get_diagnostics()
print(f"Processor: {diag['event_processor_type']}")
print(f"Processed: {diag['event_process_count']}")
print(f"Has processor: {diag['has_event_processor']}")
print(f"Processor OK: {diag['processor_is_correct']}")
```

### 故障排除：`auroraview.call()` 超时

如果 `auroraview.call()` 超时：

- 确认 Python 端用的是 `@webview.bind_call(...)` / `bind_api(...)`（而不是 `@webview.on(...)`）。
- 在 Qt DCC 环境里，确认使用 `QtWebView`（或确保安装了 Qt 事件处理器）。

## 将 WebView 嵌入现有 Qt 界面

一个常见的使用场景是将 AuroraView 嵌入到现有的 Qt 应用程序中——例如，为传统工具添加 AI 助手面板。

::: tip 自动初始化
`QtWebView` 遵循标准 Qt widget 语义。当嵌入到父 widget（如 `QDockWidget`）中时，它会在父 widget 变为可见时自动初始化。你不需要直接在 `QtWebView` 上调用 `show()`——只需将其添加到布局中并显示父 widget 即可。
:::

### 基本嵌入

`QtWebView` 继承自 `QWidget`，因此可以添加到任何 Qt 布局中：

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QMainWindow, QDockWidget, QVBoxLayout, QWidget
from qtpy.QtCore import Qt

class MyExistingTool(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("我的传统工具")

        # 你现有的 UI 设置...
        self.setup_main_ui()

        # 添加 AuroraView 作为停靠窗口（侧边栏）
        self.add_ai_panel()

    def setup_main_ui(self):
        # 你现有工具的主要内容
        central = QWidget()
        self.setCentralWidget(central)

    def add_ai_panel(self):
        """添加 AI 助手面板到侧边栏"""
        dock = QDockWidget("AI 助手", self)
        dock.setAllowedAreas(Qt.LeftDockWidgetArea | Qt.RightDockWidgetArea)

        # 创建 WebView - 线程安全是自动的
        self.ai_webview = QtWebView(
            parent=dock,
            url="http://localhost:3000/ai-agent",
            width=400,
            height=600,
        )

        dock.setWidget(self.ai_webview)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)
        # WebView 会在父 widget 显示时自动初始化！
```

### 双向通信：宿主工具 ↔ WebView

集成的关键是在现有工具和嵌入的 WebView 之间建立双向通信。

#### 1. 将宿主工具的 API 暴露给 WebView

```python
class MyExistingTool(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setup_main_ui()
        self.add_ai_panel()
        self.setup_webview_bridge()

    def setup_webview_bridge(self):
        """设置宿主工具与 WebView 之间的通信"""

        # 方法 1：绑定整个 API 对象
        self.ai_webview.bind_api(HostToolAPI(self))

        # 方法 2：绑定单个函数
        @self.ai_webview.bind_call("host.get_current_state")
        def get_state():
            return {
                "active_tab": self.get_active_tab(),
                "selected_items": self.get_selected_items(),
                "project_path": self.project_path,
            }

        @self.ai_webview.bind_call("host.execute_action")
        def execute_action(action: str, params: dict = None):
            return self.execute_tool_action(action, params or {})

class HostToolAPI:
    """暴露给 AI Agent WebView 的 API"""

    def __init__(self, host: MyExistingTool):
        self.host = host

    def get_selected_items(self) -> list:
        """获取宿主工具中当前选中的项目"""
        return self.host.get_selected_items()

    def open_file(self, path: str) -> dict:
        """在宿主工具中打开文件"""
        try:
            self.host.open_file(path)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def run_command(self, cmd: str) -> dict:
        """在宿主工具中执行命令"""
        result = self.host.execute_command(cmd)
        return {"ok": True, "result": result}
```

```javascript
// 在 AI Agent 中（WebView 端）
// 调用宿主工具的 API
const state = await auroraview.call("host.get_current_state");
console.log("当前项目:", state.project_path);

// 在宿主工具中执行操作
await auroraview.call("host.execute_action", {
  action: "create_node",
  params: { type: "sphere", name: "MySphere" }
});

// 或使用绑定的 API 对象
const items = await auroraview.api.get_selected_items();
await auroraview.api.open_file("/path/to/file.txt");
```

#### 2. 从宿主工具推送更新到 WebView

```python
class MyExistingTool(QMainWindow):
    def on_selection_changed(self):
        """当用户在宿主工具中选择项目时调用"""
        # 推送选择更新到 AI Agent
        self.ai_webview.emit("host:selection_changed", {
            "items": self.get_selected_items(),
            "count": len(self.get_selected_items()),
        })

    def on_file_opened(self, path: str):
        """当文件被打开时调用"""
        self.ai_webview.emit("host:file_opened", {
            "path": path,
            "content_type": self.detect_content_type(path),
        })

    def on_error(self, error: Exception):
        """将错误转发给 AI Agent 以提供上下文"""
        self.ai_webview.emit("host:error", {
            "message": str(error),
            "type": type(error).__name__,
        })
```

```javascript
// 在 AI Agent 中（WebView 端）
// 监听宿主事件
auroraview.on("host:selection_changed", (data) => {
  console.log(`用户选择了 ${data.count} 个项目:`, data.items);
  // AI 现在可以提供上下文感知的建议
});

auroraview.on("host:file_opened", (data) => {
  console.log("文件已打开:", data.path);
  // AI 可以分析该文件
});

auroraview.on("host:error", (data) => {
  console.log("宿主工具出错:", data.message);
  // AI 可以建议修复方案
});
```

#### 3. 处理 AI Agent 请求

```python
class MyExistingTool(QMainWindow):
    def setup_webview_bridge(self):
        # ... 之前的绑定 ...

        # 监听 AI agent 请求
        @self.ai_webview.on("ai:request_context")
        def handle_context_request(data):
            """AI agent 请求更多上下文"""
            context_type = data.get("type")

            if context_type == "full_state":
                self.ai_webview.emit("ai:context_response", {
                    "state": self.get_full_state(),
                    "history": self.get_action_history(),
                })
            elif context_type == "selected_content":
                self.ai_webview.emit("ai:context_response", {
                    "content": self.get_selected_content(),
                })

        @self.ai_webview.on("ai:execute_suggestion")
        def handle_suggestion(data):
            """执行 AI 建议的操作"""
            action = data.get("action")
            params = data.get("params", {})

            try:
                result = self.execute_tool_action(action, params)
                self.ai_webview.emit("ai:execution_result", {
                    "ok": True,
                    "result": result,
                })
            except Exception as e:
                self.ai_webview.emit("ai:execution_result", {
                    "ok": False,
                    "error": str(e),
                })
```

### 完整示例：带 AI 助手的资产浏览器

```python
from auroraview import QtWebView
from qtpy.QtWidgets import (
    QMainWindow, QDockWidget, QTreeWidget, QTreeWidgetItem,
    QVBoxLayout, QWidget, QSplitter
)
from qtpy.QtCore import Qt

class AssetBrowser(QMainWindow):
    """带 AI 助手侧边栏的传统资产浏览器工具"""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("资产浏览器")
        self.resize(1200, 800)

        self._setup_ui()
        self._setup_ai_panel()
        self._connect_signals()

    def _setup_ui(self):
        """设置主资产浏览器 UI"""
        splitter = QSplitter(Qt.Horizontal)

        # 资产树（现有功能）
        self.asset_tree = QTreeWidget()
        self.asset_tree.setHeaderLabels(["名称", "类型", "大小"])
        self._populate_assets()
        splitter.addWidget(self.asset_tree)

        # 预览面板（现有功能）
        self.preview = QWidget()
        splitter.addWidget(self.preview)

        self.setCentralWidget(splitter)

    def _setup_ai_panel(self):
        """将 AI 助手添加为停靠窗口"""
        dock = QDockWidget("AI 助手", self)

        self.ai_view = QtWebView(
            parent=dock,
            url="http://localhost:3000/asset-ai",
        )

        # 暴露资产浏览器 API
        self.ai_view.bind_api(AssetBrowserAPI(self))

        dock.setWidget(self.ai_view)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)
        # WebView 会在父 widget 显示时自动初始化！

    def _connect_signals(self):
        """将 Qt 信号连接到 WebView 事件"""
        self.asset_tree.itemSelectionChanged.connect(self._on_selection_changed)
        self.asset_tree.itemDoubleClicked.connect(self._on_item_activated)

    def _on_selection_changed(self):
        """将选择转发给 AI 助手"""
        items = self.asset_tree.selectedItems()
        self.ai_view.emit("asset:selection", {
            "assets": [self._item_to_dict(item) for item in items],
        })

    def _on_item_activated(self, item):
        """将激活事件转发给 AI 助手"""
        self.ai_view.emit("asset:activated", self._item_to_dict(item))

    def _item_to_dict(self, item: QTreeWidgetItem) -> dict:
        return {
            "name": item.text(0),
            "type": item.text(1),
            "size": item.text(2),
        }

    def _populate_assets(self):
        # ... 填充树 ...
        pass

class AssetBrowserAPI:
    """AI 助手与资产浏览器交互的 API"""

    def __init__(self, browser: AssetBrowser):
        self.browser = browser

    def get_selected_assets(self) -> list:
        items = self.browser.asset_tree.selectedItems()
        return [self.browser._item_to_dict(item) for item in items]

    def search_assets(self, query: str, asset_type: str = None) -> list:
        # 实现搜索逻辑
        return []

    def open_asset(self, name: str) -> dict:
        # 实现打开逻辑
        return {"ok": True}

    def get_asset_metadata(self, name: str) -> dict:
        # 返回资产元数据
        return {}
```

## 性能建议

### 高频 JS 更新建议批量

```python
# 低效：多次 flush
for i in range(100):
    webview.eval_js(f"updateNode({i})")

# 高效：一次 flush
script = "\n".join(f"updateNode({i})" for i in range(100))
webview.eval_js(script)
```

## 最佳实践总结

| 场景 | 建议 |
|------|------|
| 嵌入 Qt 布局 | 将 `QtWebView` 作为普通 `QWidget` 使用 |
| 暴露宿主 API | 使用 `bind_api()` 或 `@bind_call()` |
| 推送更新到 WebView | 使用 `emit()` 配合命名空间事件 |
| 处理 WebView 请求 | 使用 `@on()` 装饰器 |
| 线程安全 | 使用 `dcc_mode="auto"` 自动处理 |
