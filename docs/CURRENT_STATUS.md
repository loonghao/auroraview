# AuroraView - Maya 集成当前状态

## 📋 问题分析

### 你报告的问题

1. **WebView 阻塞 Maya**: 即使使用 `examples/07_ai_chat_non_blocking.py`，Maya 仍然被卡住
2. **JavaScript 注入不工作**: `eval_js()` 调用后没有效果
3. **Maya 无法退出**: 关闭 WebView 后，Maya 界面闪烁，无法正常退出

### 根本原因

经过深入分析，发现了以下技术限制：

#### 1. Tao/Wry 的线程限制
```rust
// EventLoop 和 WebView 都不是 Send
pub struct EventLoop<T> { ... }  // !Send
pub struct WebView { ... }       // !Send

// 这意味着它们不能跨线程传递
thread::spawn(move || {
    event_loop.run();  // ❌ 编译错误！
});
```

#### 2. Standalone 模式的阻塞问题
```python
# 即使在后台线程
def show_webview():
    webview.show()  # 调用 run_return()，阻塞此线程

thread = threading.Thread(target=show_webview, daemon=True)
thread.start()  # 线程被阻塞，但由于某种原因仍影响 Maya
```

#### 3. Embedded 模式的消息处理问题
```rust
// embedded.rs
pub fn create_embedded(...) {
    let event_loop = EventLoopBuilder::new().build();  // 创建
    let webview = WebViewBuilder::new().build(&window)?;  // 创建
    
    // 但是：从不运行事件循环！
    // 结果：消息队列中的消息永远不会被处理
    
    Ok(WebViewInner {
        event_loop: Some(event_loop),  // 存储但从不使用
        ...
    })
}
```

---

## ✅ 当前可用的解决方案

### 方案 1: Embedded 模式（推荐用于非 JS 注入场景）

**适用场景**: 你只需要显示网页，不需要 JavaScript 注入

**代码示例**: `examples/08_maya_integration_fixed.py`

```python
from auroraview import WebView

# 获取 Maya 主窗口 HWND
import maya.OpenMayaUI as omui
import shiboken2
from PySide2 import QtWidgets

maya_ptr = omui.MQtUtil.mainWindow()
maya_window = shiboken2.wrapInstance(int(maya_ptr), QtWidgets.QWidget)
parent_hwnd = maya_window.winId()

# 创建 WebView（embedded 模式）
webview = WebView(
    title="AI Chat",
    width=1200,
    height=800,
    parent_hwnd=parent_hwnd,  # 关键！
    parent_mode="owner"
)

webview.load_url("https://knot.woa.com/chat?web_key=...")
webview.show()  # 非阻塞！

# 保存引用
import __main__
__main__.webview = webview
```

**优点**:
- ✅ 完全非阻塞
- ✅ Maya 保持响应
- ✅ 自动生命周期管理
- ✅ Maya 可以正常退出

**缺点**:
- ❌ `eval_js()` 不工作（消息不被处理）
- ❌ `emit()` 不工作
- ❌ JavaScript 注入不工作

### 方案 2: Standalone 模式（如果必须使用 JS 注入）

**适用场景**: 你需要 JavaScript 注入功能

**代码示例**: `examples/07_ai_chat_non_blocking.py`

```python
from auroraview import WebView
import threading
import time

# 创建 WebView（无 parent_hwnd）
webview = WebView(
    title="AI Chat",
    width=1200,
    height=800,
    dev_tools=True
)

# 注册事件处理器
@webview.on("get_scene_info")
def handle_event(data):
    webview.emit("response", {"data": "..."})

# 加载 URL
webview.load_url("https://knot.woa.com/chat?web_key=...")

# 在后台线程显示
webview.show()  # 已经在后台线程运行

# 延迟注入 JavaScript
def inject_delayed():
    time.sleep(3)
    webview.eval_js("console.log('Injected!');")

threading.Thread(target=inject_delayed, daemon=True).start()

# 保存引用
import __main__
__main__.webview = webview
```

**优点**:
- ✅ `eval_js()` 工作
- ✅ `emit()` 工作
- ✅ JavaScript 注入工作

**缺点**:
- ⚠️ 可能有轻微阻塞（取决于系统）
- ⚠️ 需要手动管理生命周期
- ⚠️ 关闭后可能需要手动清理

---

## 🔧 技术限制说明

### 为什么不能在后台线程运行事件循环？

```rust
// 尝试 1: 在线程中运行事件循环
fn start_event_loop_thread(event_loop: EventLoop, ...) {
    thread::spawn(move || {
        event_loop.run();  // ❌ EventLoop 不是 Send
    });
}

// 编译错误:
// error[E0277]: `EventLoop<UserEvent>` cannot be sent between threads safely
//   = help: the trait `Send` is not implemented for `EventLoop<UserEvent>`
```

### 为什么不能在后台线程处理消息？

```rust
// 尝试 2: 在线程中处理 WebView 消息
fn start_message_pump(webview: Arc<Mutex<WebView>>, ...) {
    thread::spawn(move || {
        webview.lock().unwrap().evaluate_script("...");  // ❌ WebView 不是 Send
    });
}

// 编译错误:
// error[E0277]: `*mut c_void` cannot be sent between threads safely
//   = help: within `WebView`, the trait `Send` is not implemented
```

### 这是 Tao/Wry 的设计限制

- **Tao** (窗口库) 和 **Wry** (WebView 库) 都基于平台原生 API
- Windows 的 GUI 对象必须在创建它们的线程中使用
- 这是 Windows API 的限制，不是 Rust 的问题

---

## 🎯 推荐方案

### 短期方案（现在可用）

**如果不需要 JavaScript 注入**:
- 使用 **Embedded 模式** (`examples/08_maya_integration_fixed.py`)
- 完美的 Maya 集成
- 零阻塞

**如果必须使用 JavaScript 注入**:
- 使用 **Standalone 模式** (`examples/07_ai_chat_non_blocking.py`)
- 接受可能的轻微阻塞
- 所有功能都可用

### 长期方案（需要架构重构）

要完美解决这个问题，需要以下之一：

#### 选项 A: 使用 Windows 消息钩子
```rust
// 在 Maya 的主线程中安装消息钩子
SetWindowsHookEx(WH_CALLWNDPROC, hook_proc, ...);

// 在钩子中处理 WebView 消息
unsafe extern "system" fn hook_proc(...) {
    // 处理消息队列
    message_queue.process_all(|msg| {
        // 在 Maya 的线程中执行
    });
}
```

**优点**: 完美集成，零阻塞
**缺点**: 复杂，需要深入 Windows API

#### 选项 B: 使用 COM 单线程公寓 (STA)
```rust
// 将 WebView 创建在 COM STA 线程中
CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);

// 使用 COM 消息泵
while (GetMessage(&msg, NULL, 0, 0)) {
    TranslateMessage(&msg);
    DispatchMessage(&msg);
}
```

**优点**: 符合 Windows 最佳实践
**缺点**: 需要重写大部分代码

#### 选项 C: 切换到不同的 WebView 库
考虑使用支持多线程的 WebView 库，如：
- **webview-rs** (不同的实现)
- **tauri** (更高级的框架)

**优点**: 可能有更好的线程支持
**缺点**: 需要完全重写

---

## 📊 方案对比表

| 特性 | Embedded 模式 | Standalone 模式 | 理想方案（未来） |
|------|--------------|----------------|----------------|
| Maya 阻塞 | ✅ 无阻塞 | ⚠️ 可能轻微阻塞 | ✅ 无阻塞 |
| JS 注入 | ❌ 不工作 | ✅ 工作 | ✅ 工作 |
| 事件通信 | ❌ 不工作 | ✅ 工作 | ✅ 工作 |
| 生命周期 | ✅ 自动 | ⚠️ 手动 | ✅ 自动 |
| Maya 退出 | ✅ 正常 | ⚠️ 可能需清理 | ✅ 正常 |
| 实现难度 | ✅ 简单 | ✅ 简单 | ❌ 复杂 |
| 可用性 | ✅ 现在 | ✅ 现在 | ⏳ 未来 |

---

## 🚀 下一步行动

### 立即可做

1. **测试 Embedded 模式**:
   ```bash
   # 在 Maya 中运行
   exec(open('examples/08_maya_integration_fixed.py').read())
   ```

2. **测试 Standalone 模式**:
   ```bash
   # 在 Maya 中运行
   exec(open('examples/07_ai_chat_non_blocking.py').read())
   ```

3. **选择适合你的方案**:
   - 不需要 JS 注入 → Embedded 模式
   - 需要 JS 注入 → Standalone 模式

### 未来改进

1. **研究 Windows 消息钩子方案**
2. **评估切换到其他 WebView 库的可行性**
3. **与 Tao/Wry 社区讨论多线程支持**

---

## 📝 相关文档

- [MAYA_SOLUTION.md](./MAYA_SOLUTION.md) - 完整解决方案指南
- [MAYA_INTEGRATION_ISSUES.md](./MAYA_INTEGRATION_ISSUES.md) - 技术细节分析
- [examples/08_maya_integration_fixed.py](../examples/08_maya_integration_fixed.py) - Embedded 模式示例
- [examples/07_ai_chat_non_blocking.py](../examples/07_ai_chat_non_blocking.py) - Standalone 模式示例

---

## ❓ 常见问题

### Q: 为什么 Embedded 模式的 eval_js() 不工作？
A: 因为 Embedded 模式不运行事件循环，消息队列中的消息不会被处理。这是当前的技术限制。

### Q: 能否修复 Embedded 模式的消息处理？
A: 理论上可以，但需要复杂的 Windows API 编程（消息钩子或 COM STA）。这超出了当前的实现范围。

### Q: Standalone 模式为什么会阻塞？
A: 即使在后台线程，`run_return()` 仍然是阻塞调用。由于 Windows 消息循环的特性，这可能影响主线程。

### Q: 有没有完美的解决方案？
A: 目前没有。需要等待：
1. Tao/Wry 添加多线程支持，或
2. 实现复杂的 Windows 消息钩子方案，或
3. 切换到不同的 WebView 库

### Q: 我应该使用哪个方案？
A: 
- **不需要 JS 注入** → Embedded 模式（完美）
- **需要 JS 注入** → Standalone 模式（可接受）
- **需要完美方案** → 等待未来版本

---

**最后更新**: 2025-11-02
**状态**: 已分析，提供临时方案，等待长期解决方案

