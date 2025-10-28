# 🔍 线程问题诊断和修复指南

## 问题症状

### 症状 1：点击按钮时 Maya 卡住

**表现：**
- WebView 正常显示
- 点击按钮时，Maya 完全无响应
- 无法操作 Maya，必须强制关闭

**原因：** JavaScript 回调在 WebView 事件循环中执行，而事件循环运行在后台线程中。

### 症状 2：关闭 WebView 时 Maya 退出

**表现：**
- 关闭 WebView 窗口
- Maya 直接退出，没有任何警告

**原因：** 后台线程是 daemon 线程，当它异常退出时，可能导致整个进程崩溃。

## 🔧 修复步骤

### 步骤 1：编译最新代码

```bash
# 确保使用最新的代码（已修改 daemon=False）
maturin develop --release
```

### 步骤 2：在 Maya 中测试

```python
# 在 Maya 脚本编辑器中运行

import sys
import os

# 添加项目路径
project_root = r"C:\Users\hallo\Documents\augment-projects\dcc_webview"
python_path = os.path.join(project_root, "python")
if python_path not in sys.path:
    sys.path.insert(0, python_path)

from auroraview import WebView

# 创建 WebView
webview = WebView(title="Test", width=400, height=300)

# 创建简单的 HTML
html = """
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: Arial; padding: 20px; }
        button { padding: 10px 20px; font-size: 16px; }
    </style>
</head>
<body>
    <h1>Test WebView</h1>
    <button onclick="alert('Button clicked!')">Click Me</button>
    <p>If Maya freezes when you click the button, there's still a threading issue.</p>
</body>
</html>
"""

webview.load_html(html)
webview.show_async()

print("WebView started. Try clicking the button.")
print("If Maya freezes, the threading issue is not fixed yet.")
```

### 步骤 3：观察结果

| 结果 | 含义 | 下一步 |
|------|------|------|
| ✓ 点击按钮，Maya 不卡 | 修复成功 | 完成 |
| ✗ 点击按钮，Maya 卡住 | 需要实现非阻塞事件处理 | 见下文 |
| ✗ 关闭 WebView，Maya 退出 | daemon=False 修复未生效 | 重新编译 |

## 🚀 如果仍然卡住：完整修复方案

### 问题根源

当前的 `core.show()` 是**阻塞的**：

```rust
// src/webview/mod.rs
pub fn show_window(&self) -> PyResult<()> {
    let mut inner = self.inner.borrow_mut();
    if inner.is_none() {
        let webview = Self::create_standalone(...)?;
        inner = Some(webview);
    }
    
    // 这是阻塞的 - 一直运行事件循环
    inner.as_mut().unwrap().run_event_loop_blocking();
    Ok(())
}
```

### 解决方案：实现非阻塞事件处理

需要在 Rust 中添加一个新方法：

```rust
// src/webview/mod.rs

#[pymethods]
impl AuroraView {
    /// Show WebView and return immediately (non-blocking)
    fn show_non_blocking(&self) -> PyResult<()> {
        let mut inner = self.inner.borrow_mut();
        if inner.is_none() {
            let webview = Self::create_standalone(...)?;
            inner = Some(webview);
        }
        
        // 只创建 WebView，不运行事件循环
        // 事件循环应该在 Python 层定期调用
        Ok(())
    }
    
    /// Process one event from the event loop (non-blocking)
    fn process_event(&self) -> PyResult<bool> {
        // 处理一个事件并返回
        // 返回 true 如果还有事件，false 如果窗口关闭
        Ok(true)
    }
}
```

### Python 层的修改

```python
# python/auroraview/webview.py

def show_async(self) -> None:
    """Show WebView in background thread (non-blocking)."""
    
    def _run_webview():
        try:
            from ._core import WebView as _CoreWebView
            core = _CoreWebView(...)
            core.load_html(self._stored_html)
            
            # 使用非阻塞方式
            core.show_non_blocking()
            
            # 定期处理事件
            import time
            while True:
                has_events = core.process_event()
                if not has_events:
                    break  # 窗口关闭
                time.sleep(0.01)  # 让出 CPU
        finally:
            self._is_running = False
    
    self._show_thread = threading.Thread(target=_run_webview, daemon=False)
    self._show_thread.start()
```

## 📊 修复前后对比

### 修复前

```
WebView 事件循环（后台线程）
    ↓
JavaScript 事件
    ↓
Python 回调（同步，在事件循环中）
    ↓
尝试调用 Maya API
    ↓
❌ 死锁 - Maya 卡住
```

### 修复后

```
WebView 事件循环（后台线程）
    ↓
JavaScript 事件
    ↓
事件队列
    ↓
后台线程定期处理
    ↓
Python 回调（异步）
    ↓
消息队列
    ↓
Maya 主线程处理
    ↓
✓ 调用 Maya API - 无死锁
```

## 🧪 验证修复

### 测试 1：基本响应性

```python
# 在 Maya 中运行
webview = WebView(title="Test", width=400, height=300)
webview.load_html("<button>Click Me</button>")
webview.show_async()

# 在 Maya 中执行其他操作
import maya.cmds as cmds
cmds.polyCube()  # 应该能正常执行，不会卡住
```

### 测试 2：事件处理

```python
# 在 Maya 中运行
webview = WebView(title="Test", width=400, height=300)

html = """
<button onclick="window.pywebview.api.test_callback()">Test</button>
<script>
    window.pywebview = window.pywebview || {};
    window.pywebview.api = {
        test_callback: function() {
            console.log("Callback executed");
        }
    };
</script>
"""

webview.load_html(html)
webview.show_async()

# 点击按钮 - Maya 应该保持响应
```

## 📝 检查清单

- [ ] 编译最新代码：`maturin develop --release`
- [ ] 在 Maya 中创建 WebView
- [ ] 点击按钮，观察 Maya 是否卡住
- [ ] 关闭 WebView，观察 Maya 是否退出
- [ ] 如果仍然卡住，实现非阻塞事件处理
- [ ] 重新编译并测试
- [ ] 验证所有功能正常

## 🆘 故障排除

### 问题：仍然卡住

**解决方案：**
1. 检查是否重新编译：`maturin develop --release`
2. 检查 Python 是否加载了新的 .pyd 文件
3. 实现非阻塞事件处理（见上文）

### 问题：WebView 不显示

**解决方案：**
1. 检查 HTML 是否正确加载
2. 检查 WebView 是否在后台线程中创建
3. 查看日志输出

### 问题：关闭 WebView 时 Maya 仍然退出

**解决方案：**
1. 检查 daemon=False 是否生效
2. 检查是否有异常导致线程崩溃
3. 添加更多日志记录

## 📚 相关文件

- `REAL_THREADING_ISSUES.md` - 详细的问题分析
- `IMMEDIATE_FIX_PLAN.md` - 修复计划
- `DCC_THREADING_SOLUTION.md` - 消息队列解决方案
- `examples/maya_event_queue_integration.py` - 集成示例

---

**现在就开始测试吧！** 按照上面的步骤进行诊断和修复。

