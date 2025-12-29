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
