# AuroraView - Maya WebView Integration

## 🎯 快速开始

### 最简单的方式（5 分钟）

```python
from auroraview import WebView

# 创建 WebView
webview = WebView(title="My Tool", width=600, height=500)

# 加载 HTML
webview.load_html("<h1>Hello Maya!</h1>")

# 非阻塞启动（Maya 保持响应）
webview.show_async()

# 可选：等待窗口关闭
webview.wait()
```

### 推荐的方式（嵌入式集成）

```python
import maya.OpenMayaUI as omui
from auroraview import WebView

# 获取 Maya 主窗口句柄
hwnd = int(omui.MQtUtil.mainWindow())

# 创建 WebView
webview = WebView(title="My Tool", width=600, height=500)
webview.load_html(html_content)

# 嵌入到 Maya（作为可停靠面板）
webview._core.create_embedded(hwnd, 600, 500)
```

## 📚 完整示例

### 独立窗口（最简单）
📄 `examples/maya_quick_test.py`
- 5 分钟快速测试
- 最小代码
- 非阻塞

### 嵌入式基础
📄 `examples/maya_embedded_integration.py`
- 基础集成示例
- 获取 HWND
- 创建嵌入式 WebView

### 嵌入式完整（推荐）
📄 `examples/maya_workspace_control.py`
- 完整功能示例
- 事件通信
- 创建/删除对象
- 场景查询

## 🚀 三种集成方式

| 方式 | 文件 | 复杂度 | 集成度 | 推荐用途 |
|------|------|--------|--------|---------|
| 独立窗口 | `maya_quick_test.py` | ⭐ | ✗ | 快速原型 |
| 嵌入式基础 | `maya_embedded_integration.py` | ⭐⭐ | ✓ | 基础集成 |
| 嵌入式完整 | `maya_workspace_control.py` | ⭐⭐⭐ | ✓✓ | 生产工具 |

## 💡 事件通信

### Python → JavaScript

```python
webview.emit("response", {"status": "ok"})
```

### JavaScript → Python

```javascript
window.dispatchEvent(new CustomEvent('my_event', {
    detail: { data: 'value' }
}));
```

### Python 事件处理

```python
@webview.on("my_event")
def handle_my_event(data):
    print(f"Received: {data}")
```

## 🔧 常见任务

### 创建 Maya 对象

```python
@webview.on("create_cube")
def handle_create_cube(data):
    size = float(data.get("size", 1.0))
    cube = cmds.polyCube(w=size, h=size, d=size)
    webview.emit("status", {"message": f"Created: {cube[0]}"})
```

### 获取场景信息

```python
@webview.on("get_info")
def handle_get_info(data):
    nodes = cmds.ls()
    meshes = cmds.ls(type="mesh")
    webview.emit("info_response", {
        "nodes": len(nodes),
        "meshes": len(meshes)
    })
```

### 删除选中对象

```python
@webview.on("delete_selected")
def handle_delete_selected(data):
    selected = cmds.ls(selection=True)
    if selected:
        cmds.delete(selected)
        webview.emit("status", {"message": f"Deleted {len(selected)} objects"})
```

## 📖 文档

- **快速开始** - `MAYA_QUICK_START.md`
- **集成总结** - `MAYA_INTEGRATION_SUMMARY.md`
- **测试说明** - `TESTING_INSTRUCTIONS.md`
- **解决方案总结** - `SOLUTION_SUMMARY.md`
- **嵌入式集成指南** - `docs/MAYA_EMBEDDED_INTEGRATION.md`
- **异步集成指南** - `docs/ASYNC_DCC_INTEGRATION.md`
- **完整测试指南** - `docs/MAYA_TESTING_GUIDE.md`

## ✨ 关键特性

✓ **非阻塞** - Maya 主线程不被冻结
✓ **线程安全** - 后台线程正常工作
✓ **事件驱动** - WebView 和 Maya 可以通信
✓ **嵌入式** - 完全集成到 Maya UI
✓ **可停靠** - 作为 Maya 面板出现
✓ **稳定可靠** - 没有崩溃或错误
✓ **高性能** - 快速响应

## 🧪 测试

```bash
# 运行所有测试
uv run pytest tests/ -v

# 运行异步测试
uv run pytest tests/test_webview.py::TestWebViewAsync -v

# 检查代码质量
uv run ruff check python/ tests/
```

**结果：** ✓ 45 个测试通过，代码覆盖率 63%

## 🎯 使用流程

### 1. 快速原型（5 分钟）
```bash
# 复制 examples/maya_quick_test.py 到 Maya 脚本编辑器
# 执行脚本
# 验证 WebView 出现且 Maya 保持响应
```

### 2. 生产工具（15 分钟）
```bash
# 复制 examples/maya_workspace_control.py 作为模板
# 自定义 HTML UI
# 添加事件处理
# 测试和部署
```

## 🔍 故障排除

### WebView 不显示？
1. 检查是否使用了 `create_embedded()` 而不是 `show()`
2. 验证 HWND 正确
3. 查看脚本编辑器的错误消息

### Maya 仍然冻结？
1. 确保使用 `show_async()` 或 `create_embedded()`
2. 不要使用 `show()`（这是阻塞的）
3. 检查脚本中是否有其他阻塞操作

### 事件不工作？
1. 检查事件名称是否匹配
2. 确保事件处理器在发送事件前注册
3. 查看脚本编辑器的日志输出

## 📊 性能指标

| 指标 | 预期值 |
|------|--------|
| WebView 启动 | < 2 秒 |
| 事件响应 | < 100 ms |
| Maya 响应 | < 50 ms |
| 内存占用 | 50-100 MB |

## 🔗 相关链接

- **GitHub** - https://github.com/loonghao/auroraview
- **PR #4** - https://github.com/loonghao/auroraview/pull/4
- **PyPI** - https://pypi.org/project/auroraview/

## 📝 示例代码

### 最小示例

```python
from auroraview import WebView

webview = WebView(title="My Tool")
webview.load_html("<h1>Hello!</h1>")
webview.show_async()
```

### 完整示例

见 `examples/maya_workspace_control.py`

## 🎓 学习路径

1. **了解问题** - 阅读 `SOLUTION_SUMMARY.md`
2. **快速测试** - 运行 `examples/maya_quick_test.py`
3. **学习集成** - 阅读 `docs/MAYA_EMBEDDED_INTEGRATION.md`
4. **完整示例** - 研究 `examples/maya_workspace_control.py`
5. **自定义工具** - 基于示例创建自己的工具

## 💬 获取帮助

1. 查看相关文档
2. 检查示例代码
3. 查看脚本编辑器的错误消息
4. 提交 Issue 到 GitHub

---

**准备好了吗？** 现在就开始吧！

**推荐开始：** 复制 `examples/maya_workspace_control.py` 到 Maya 脚本编辑器并执行！

