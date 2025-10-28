# Maya Integration Summary - AuroraView WebView

## 问题分析

### 原始问题
在 Maya 中执行 `webview.show()` 时，整个 Maya 主线程会被冻结，无法退出 Maya。

### 根本原因
Rust 核心的 `show()` 方法运行一个阻塞事件循环，直到窗口关闭。这会阻塞 Maya 的主线程。

## 解决方案演进

### 第一阶段：非阻塞模式（show_async）
**问题：** 尝试在后台线程中发送 Rust 对象
```
PanicException: AuroraView is unsendable, but sent to another thread
```

**解决方案：** 在后台线程中创建新的 WebView 实例
- ✓ 主线程创建 WebView，加载内容
- ✓ 后台线程创建新实例，运行事件循环
- ✓ Maya 主线程保持响应

**文件：**
- `python/auroraview/webview.py` - 实现 `show_async()`
- `examples/maya_quick_test.py` - 快速测试示例

### 第二阶段：嵌入式模式（create_embedded）
**问题：** 独立窗口不是 Maya UI 的一部分

**解决方案：** 使用 `create_embedded()` 将 WebView 嵌入到 Maya
- ✓ 获取 Maya 主窗口的 HWND
- ✓ 使用 `create_embedded(hwnd, width, height)`
- ✓ WebView 作为可停靠面板出现
- ✓ 完全集成到 Maya UI

**文件：**
- `examples/maya_embedded_integration.py` - 基础嵌入式示例
- `examples/maya_workspace_control.py` - 完整生产示例
- `docs/MAYA_EMBEDDED_INTEGRATION.md` - 详细集成指南

## 三种集成方式对比

| 特性 | 独立窗口 | 嵌入式基础 | 嵌入式完整 |
|------|---------|----------|----------|
| 文件 | `maya_quick_test.py` | `maya_embedded_integration.py` | `maya_workspace_control.py` |
| 复杂度 | ⭐ 简单 | ⭐⭐ 中等 | ⭐⭐⭐ 复杂 |
| 集成度 | ✗ 无 | ✓ 部分 | ✓ 完全 |
| 可停靠 | ✗ 否 | ✓ 是 | ✓ 是 |
| 工作区保存 | ✗ 否 | ⚠️ 部分 | ✓ 是 |
| 非阻塞 | ✓ 是 | ✓ 是 | ✓ 是 |
| 事件通信 | ✓ 是 | ✓ 是 | ✓ 是 |

## 关键代码片段

### 获取 Maya 窗口句柄

```python
import maya.OpenMayaUI as omui

def get_maya_main_window_hwnd():
    main_window_ptr = omui.MQtUtil.mainWindow()
    if main_window_ptr is None:
        raise RuntimeError("Could not get Maya main window pointer")
    return int(main_window_ptr)
```

### 创建嵌入式 WebView

```python
from auroraview import WebView

hwnd = get_maya_main_window_hwnd()
webview = WebView(title="My Tool", width=600, height=500)
webview.load_html(html_content)
webview._core.create_embedded(hwnd, 600, 500)
```

### 事件通信

**Python → JavaScript：**
```python
webview.emit("response", {"status": "ok"})
```

**JavaScript → Python：**
```javascript
window.dispatchEvent(new CustomEvent('my_event', {
    detail: { data: 'value' }
}));
```

**Python 事件处理：**
```python
@webview.on("my_event")
def handle_my_event(data):
    print(f"Received: {data}")
```

## 文件结构

```
examples/
├── maya_quick_test.py              # 独立窗口 - 快速测试
├── maya_embedded_integration.py     # 嵌入式 - 基础示例
├── maya_workspace_control.py        # 嵌入式 - 完整示例
├── maya_test_tool.py                # 完整功能测试工具
└── test_async_fix.py                # 线程安全测试

docs/
├── MAYA_TESTING_GUIDE.md            # 测试指南
├── MAYA_EMBEDDED_INTEGRATION.md     # 嵌入式集成指南
├── ASYNC_DCC_INTEGRATION.md         # 异步集成指南
└── DCC_INTEGRATION_GUIDE.md         # DCC 集成指南

python/auroraview/
└── webview.py                       # 核心 Python API
    ├── show()                       # 阻塞模式
    ├── show_async()                 # 非阻塞模式
    ├── load_html()                  # 加载 HTML
    ├── load_url()                   # 加载 URL
    ├── on()                         # 事件处理
    └── emit()                       # 事件发送

src/webview/mod.rs                   # Rust 核心
├── show()                           # 阻塞事件循环
└── create_embedded()                # 嵌入式模式
```

## 测试结果

✓ 所有 45 个单元测试通过
✓ 所有 ruff 检查通过
✓ 代码覆盖率：63%
✓ 线程安全：已验证
✓ 事件通信：已验证

## 推荐使用流程

### 快速原型（5 分钟）
1. 使用 `examples/maya_quick_test.py`
2. 复制到 Maya 脚本编辑器
3. 执行并测试

### 生产工具（15 分钟）
1. 使用 `examples/maya_workspace_control.py` 作为模板
2. 自定义 HTML UI
3. 添加事件处理
4. 测试和部署

## 常见问题

### Q: 为什么 WebView 不显示？
A: 检查是否使用了 `create_embedded()` 而不是 `show()`。确保 HWND 正确。

### Q: 如何在 WebView 中创建 Maya 对象？
A: 使用事件通信。在 JavaScript 中发送事件，在 Python 中处理并调用 `cmds`。

### Q: 工作区会保存 WebView 吗？
A: 嵌入式模式会保存面板位置，但需要额外配置才能保存内容状态。

### Q: 支持哪些平台？
A: 目前支持 Windows（HWND）。macOS 和 Linux 需要相应的窗口句柄实现。

## 下一步

1. ✓ 修复线程安全问题
2. ✓ 实现嵌入式模式
3. ✓ 创建完整示例
4. ⏳ 支持 macOS/Linux 窗口句柄
5. ⏳ 创建 Maya 插件包装器
6. ⏳ 添加工作区持久化

## 相关文档

- `MAYA_QUICK_START.md` - 快速开始指南
- `docs/MAYA_EMBEDDED_INTEGRATION.md` - 嵌入式集成详细指南
- `docs/ASYNC_DCC_INTEGRATION.md` - 异步集成指南
- `docs/MAYA_TESTING_GUIDE.md` - 测试指南

## 提交历史

```
7cadbde - docs: update quick start with embedded mode recommendations
934b014 - feat: add embedded WebView integration for Maya
b6bc91d - test: add comprehensive async thread safety test suite
b3914ef - fix: resolve thread safety issue in show_async()
4e77c41 - docs: update quick start guide with thread safety fix
3e16b59 - docs: add Maya quick start guide
2b38bc6 - docs: add comprehensive Maya testing examples and guides
4a14a6c - fix: organize imports and remove unused imports in test_webview.py
13804c0 - feat: add non-blocking show_async() method for DCC integration
```

## 总结

AuroraView 现在提供了三种方式在 Maya 中集成 WebView：

1. **独立窗口** - 最简单，适合快速原型
2. **嵌入式基础** - 中等复杂度，集成到 Maya
3. **嵌入式完整** - 最专业，完全集成和工作区支持

所有方式都是非阻塞的，不会冻结 Maya 主线程。选择最适合你的需求的方式！

---

**推荐开始：** 复制 `examples/maya_workspace_control.py` 到 Maya 脚本编辑器并执行！

