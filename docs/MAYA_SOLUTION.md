# Maya Integration - 完整解决方案

## 🎯 问题总结

你遇到的两个问题：

### 问题 1: WebView 阻塞 Maya 主线程
**现象**: 即使使用后台线程，WebView 仍然会卡住 Maya

**根本原因**: 
- `EventLoop` 和 `WebView` 都不是 `Send` trait
- 无法在线程间传递这些对象
- 即使在后台线程运行，`run_return()` 仍然是阻塞调用

### 问题 2: 关闭 WebView 后 Maya 无法退出
**现象**: 关闭 WebView 窗口后，Maya 界面闪烁，无法正常退出

**根本原因**:
- 事件循环退出后，窗口资源没有完全清理
- Maya 的 Qt 事件循环与 Tao 事件循环冲突

---

## ✅ 推荐解决方案：使用 Embedded 模式

### 方案说明

**核心思路**: 使用 Maya 的主窗口作为父窗口，创建 embedded 模式的 WebView。

**为什么这样可以解决问题**:
1. Embedded 模式不运行自己的事件循环
2. 依赖 Maya 的 Qt 事件循环处理消息
3. 完全非阻塞
4. 生命周期与 Maya 绑定

### 完整代码示例

```python
#!/usr/bin/env python
"""
Maya WebView Integration - Correct Solution

This example shows the CORRECT way to integrate WebView with Maya.
"""

import logging
from auroraview import WebView

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def get_maya_main_window_hwnd():
    """Get Maya main window HWND."""
    try:
        import maya.OpenMayaUI as omui
        import shiboken2
        from PySide2 import QtWidgets
        
        # Get Maya main window
        maya_main_window_ptr = omui.MQtUtil.mainWindow()
        maya_main_window = shiboken2.wrapInstance(
            int(maya_main_window_ptr), 
            QtWidgets.QWidget
        )
        
        # Get HWND
        hwnd = maya_main_window.winId()
        logger.info(f"Maya main window HWND: 0x{hwnd:x}")
        return hwnd
    except Exception as e:
        logger.error(f"Failed to get Maya HWND: {e}")
        return None


def create_ai_chat_webview():
    """Create AI chat WebView for Maya."""
    
    # Step 1: Get Maya's main window HWND
    parent_hwnd = get_maya_main_window_hwnd()
    if parent_hwnd is None:
        logger.error("Cannot get Maya window handle")
        return None
    
    # Step 2: Create WebView in embedded mode
    webview = WebView(
        title="AI Chat - Maya Integration",
        width=1200,
        height=800,
        dev_tools=True,
        parent_hwnd=parent_hwnd,  # KEY: This enables embedded mode
        parent_mode="owner"  # Use "owner" for cross-thread safety
    )
    
    # Step 3: Register event handlers
    @webview.on("get_scene_info")
    def handle_get_scene_info(data):
        """Handle request for Maya scene information."""
        logger.info("Website requested scene info")
        
        try:
            import maya.cmds as cmds
            selection = cmds.ls(selection=True)
        except:
            selection = []
        
        # Send response back to website
        webview.emit("scene_info_response", {
            "selection": selection,
            "selection_count": len(selection)
        })
    
    @webview.on("execute_code")
    def handle_execute_code(data):
        """Handle code execution request from website."""
        code = data.get("code", "")
        logger.info(f"Executing code: {code}")
        
        try:
            import maya.cmds as cmds
            # Execute code in Maya
            exec(code, {"cmds": cmds})
            webview.emit("code_executed", {"success": True})
        except Exception as e:
            logger.error(f"Code execution failed: {e}")
            webview.emit("code_executed", {
                "success": False,
                "error": str(e)
            })
    
    # Step 4: Load content
    webview.load_url("https://knot.woa.com/chat?web_key=1c2a6b4568f24e00a58999c1b7cb0f6e")
    
    # Step 5: Show WebView (NON-BLOCKING!)
    webview.show()
    
    logger.info("✅ WebView created successfully")
    logger.info("✅ Maya remains fully responsive")
    
    # Step 6: IMPORTANT - Store reference to prevent garbage collection
    import __main__
    __main__.ai_chat_webview = webview
    
    return webview


# Usage in Maya Script Editor:
# exec(open('path/to/this/script.py').read())
# webview = create_ai_chat_webview()
```

---

## ⚠️ 当前限制

### JavaScript 注入不工作

**问题**: 在 embedded 模式下，`webview.eval_js()` 和 `webview.emit()` 不工作

**原因**: Embedded 模式没有运行事件循环，所以消息队列中的消息不会被处理

**临时解决方案**: 
1. 使用网站自己的功能（如果可能）
2. 等待下一个版本的修复

**计划修复**: 
- 在 embedded 模式下启动一个轻量级的消息处理线程
- 该线程定期检查消息队列并处理消息
- 不运行完整的事件循环，只处理 WebView 消息

---

## 🔧 替代方案（如果必须使用 JavaScript 注入）

如果你现在就需要 JavaScript 注入功能，可以使用以下临时方案：

### 方案 A: 使用 Standalone 模式 + 手动生命周期管理

```python
import threading
import time
from auroraview import WebView

# Create WebView without parent (standalone mode)
webview = WebView(
    title="AI Chat",
    width=1200,
    height=800,
    dev_tools=True
)

# Register handlers
@webview.on("get_scene_info")
def handle_get_scene_info(data):
    # ... handle event

# Load URL
webview.load_url("https://knot.woa.com/chat?web_key=...")

# Show in background thread
def show_webview():
    webview.show()  # This will block this thread

thread = threading.Thread(target=show_webview, daemon=True)
thread.start()

# Wait for WebView to be ready
time.sleep(2)

# Now you can inject JavaScript
injection_script = """
(function() {
    console.log('Injected!');
    // Your injection code here
})();
"""

webview.eval_js(injection_script)

# Store reference
import __main__
__main__.ai_chat_webview = webview
```

**优点**:
- JavaScript 注入可以工作
- `eval_js()` 和 `emit()` 都可以使用

**缺点**:
- 可能仍然有轻微的阻塞
- 需要手动管理生命周期
- 关闭 WebView 后可能需要手动清理

### 方案 B: 等待修复后的版本

我正在实现一个修复，将在下一个版本中提供：
- Embedded 模式将启动消息处理线程
- 完全非阻塞
- JavaScript 注入正常工作

---

## 📊 方案对比

| 特性 | Embedded 模式（推荐） | Standalone 模式 |
|------|---------------------|----------------|
| **阻塞 Maya** | ❌ 不阻塞 | ⚠️ 可能轻微阻塞 |
| **生命周期** | ✅ 自动管理 | ⚠️ 需要手动管理 |
| **JavaScript 注入** | ❌ 当前不工作 | ✅ 工作正常 |
| **事件通信** | ❌ 当前不工作 | ✅ 工作正常 |
| **Maya 退出** | ✅ 正常退出 | ⚠️ 可能需要手动清理 |
| **推荐度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |

---

## 🎯 最终建议

### 如果你不需要 JavaScript 注入
**使用 Embedded 模式** (`examples/08_maya_integration_fixed.py`)
- 完全非阻塞
- 自动生命周期管理
- Maya 可以正常退出

### 如果你必须使用 JavaScript 注入
**使用 Standalone 模式** (`examples/07_ai_chat_non_blocking.py`)
- JavaScript 注入可以工作
- 需要接受可能的轻微阻塞
- 需要手动管理生命周期

### 最佳方案（等待下一版本）
下一个版本将修复 embedded 模式的消息处理问题，届时你可以：
- 使用 embedded 模式（非阻塞）
- JavaScript 注入正常工作
- 完美的 Maya 集成

---

## 📝 下一步

1. **立即可用**: 使用 `examples/08_maya_integration_fixed.py`（embedded 模式）
2. **如需 JS 注入**: 使用 `examples/07_ai_chat_non_blocking.py`（standalone 模式）
3. **等待更新**: 关注下一个版本的发布

---

## 🔗 相关文档

- [MAYA_INTEGRATION_ISSUES.md](./MAYA_INTEGRATION_ISSUES.md) - 详细技术分析
- [THIRD_PARTY_INTEGRATION.md](./THIRD_PARTY_INTEGRATION.md) - JavaScript 注入指南
- [examples/08_maya_integration_fixed.py](../examples/08_maya_integration_fixed.py) - Embedded 模式示例
- [examples/07_ai_chat_non_blocking.py](../examples/07_ai_chat_non_blocking.py) - Standalone 模式示例

