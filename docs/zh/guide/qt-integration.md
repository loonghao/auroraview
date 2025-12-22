# Qt 集成最佳实践

本指南介绍将 AuroraView 与基于 Qt 的 DCC 应用程序（Maya、Houdini、Nuke 等）集成的最佳实践。

## 快速开始

### 推荐：使用 QtWebView

```python
from auroraview import QtWebView

# 创建 WebView 作为 Qt 组件
webview = QtWebView(
    parent=maya_main_window(),  # 可选：任何 QWidget
    title="My Tool",
    width=800,
    height=600
)

# 加载内容
webview.load_url("http://localhost:3000")

# 显示组件
webview.show()

# 就是这样！事件处理是自动的。
```

### 避免：手动事件处理

```python
# 不要这样做 - 使用 QtWebView 时不需要
from auroraview import WebView

webview = WebView.create(...)
webview.show()

# 手动事件处理（使用 QtWebView 时不需要）
def process_events():
    webview.process_events()  # 不必要！

cmds.scriptJob(event=["idle", process_events])
```

## 理解事件处理

### 问题

AuroraView 使用消息队列执行 JavaScript：

1. Python 调用 `eval_js(script)` → 脚本推入队列
2. 队列需要处理 → 脚本执行
3. 没有处理 → **脚本永远不执行，Promise 挂起**

### 解决方案

`QtWebView` 在每次 `eval_js()` 调用后自动处理事件：

```python
# 当你调用：
webview.eval_js("console.log('Hello')")

# QtWebView 自动：
# 1. 将脚本推入队列
# 2. 调用 process_events()  ← 自动！
# 3. 脚本立即执行
```

## 常见模式

### 模式 1：Python → JavaScript 通信

```python
from auroraview import QtWebView

webview = QtWebView(title="My Tool")

# 这会立即工作 - 不需要手动事件处理
webview.eval_js("console.log('Hello from Python')")
webview.emit("update_scene", {"objects": ["cube", "sphere"]})
```

### 模式 2：JavaScript → Python 通信

```python
# Python 端
@webview.on("get_scene_data")
def handle_get_scene_data(data):
    # 从 DCC 获取场景数据
    selection = cmds.ls(selection=True)
    # 发送回 JavaScript - 自动事件处理！
    webview.emit("scene_data_response", {"selection": selection})
```

```javascript
// JavaScript 端
window.auroraview.on("scene_data_response", (data) => {
    console.log("Selection:", data.selection);
});

window.auroraview.send_event("get_scene_data", {});
```

### 模式 3：使用 auroraview.call() API

```python
# Python 端
@webview.bind_call("get_scene_hierarchy")
def get_scene_hierarchy(params):
    # 返回场景层级
    return {"nodes": [...]}
```

```javascript
// JavaScript 端 - Promise 自动解析！
const result = await window.auroraview.call("get_scene_hierarchy");
console.log("Hierarchy:", result);  // 正常工作！
```

## 诊断

### 检查事件处理

```python
# 获取诊断信息
diag = webview.get_diagnostics()

print(f"处理的事件数: {diag['event_process_count']}")
print(f"上次处理时间: {diag['last_event_process_time']}")
print(f"Hook 已安装: {diag['has_post_eval_hook']}")
print(f"Hook 正确: {diag['hook_is_correct']}")
```

### 故障排除

如果 `auroraview.call()` Promise 挂起：

1. **检查 hook 安装**：
   ```python
   diag = webview.get_diagnostics()
   assert diag['has_post_eval_hook'], "Hook 未安装！"
   assert diag['hook_is_correct'], "Hook 错误！"
   ```

2. **检查事件处理**：
   ```python
   # 每次 eval_js/emit 调用后应该增加
   before = webview.get_diagnostics()['event_process_count']
   webview.eval_js("console.log('test')")
   after = webview.get_diagnostics()['event_process_count']
   assert after > before, "事件未被处理！"
   ```

3. **启用调试日志**：
   ```python
   import logging
   logging.basicConfig(level=logging.DEBUG)
   ```

## 性能考虑

### 事件处理开销

- 每次 `eval_js()` 调用触发事件处理
- 开销：每次调用约 1-2ms
- 对于高频更新，批量处理调用：

```python
# 低效：100 次事件处理循环
for i in range(100):
    webview.eval_js(f"updateNode({i})")

# 高效：1 次事件处理循环
script = "\n".join(f"updateNode({i})" for i in range(100))
webview.eval_js(script)
```

## Qt 组件集成

### 作为可停靠组件

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt

# 创建停靠组件
dock = QDockWidget("My Tool", main_window)

# 创建 WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# 设置为停靠组件内容
dock.setWidget(webview)
main_window.addDockWidget(Qt.RightDockWidgetArea, dock)
```

### 作为标签页组件

```python
from qtpy.QtWidgets import QTabWidget

tab_widget = QTabWidget()

webview1 = QtWebView(parent=tab_widget)
webview1.load_url("http://localhost:3000/tool1")

webview2 = QtWebView(parent=tab_widget)
webview2.load_url("http://localhost:3000/tool2")

tab_widget.addTab(webview1, "工具 1")
tab_widget.addTab(webview2, "工具 2")
```

### 自定义组件子类

```python
from auroraview import QtWebView

class MyToolWidget(QtWebView):
    def __init__(self, parent=None):
        super().__init__(parent=parent, width=400, height=300)
        self.load_url("http://localhost:3000")
        self._setup_handlers()
    
    def _setup_handlers(self):
        @self.on("tool_action")
        def handle_action(data):
            self._process_action(data)
    
    def _process_action(self, data):
        # 自定义逻辑
        pass
```

## 总结

**应该做**：
- 对基于 Qt 的 DCC 使用 `QtWebView`
- 信任自动事件处理
- 使用 `get_diagnostics()` 进行故障排除
- 批量处理高频 `eval_js()` 调用

**不应该做**：
- 使用 `QtWebView` 时手动调用 `process_events()`
- 在 Qt 环境中使用 `WebView.create()`
- 为事件处理创建 scriptJob
- 进行数百次单独的 `eval_js()` 调用
