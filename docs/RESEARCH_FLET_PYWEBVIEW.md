# Flet 和 pywebview 嵌入式窗口解决方案调研

## 调研目的

调研 Flet 和 pywebview 项目如何解决嵌入式窗口(设置 parent HWND)的关闭问题,为 AuroraView 提供参考。

## 1. Flet 项目分析

### 1.1 技术架构

**核心技术栈:**
- **前端**: Flutter (Dart)
- **后端**: Python
- **通信**: WebSocket/HTTP
- **平台**: Windows, macOS, Linux, iOS, Android, Web

**关键特点:**
1. **不使用 HWND 嵌入** - Flet 使用 Flutter 的原生窗口系统
2. **完全独立的窗口** - 每个 Flet 应用都是独立的 Flutter 应用
3. **不依赖浏览器引擎** - 使用 Flutter 的 Skia 渲染引擎

### 1.2 窗口管理方式

```python
# Flet 的窗口创建方式
import flet as ft

def main(page: ft.Page):
    page.title = "My App"
    page.add(ft.Text("Hello"))

ft.run(main)  # 创建独立的 Flutter 窗口
```

**关键发现:**
- ✅ Flet **不支持嵌入到其他应用** (如 Maya)
- ✅ 所有窗口都是**独立的顶层窗口**
- ✅ 使用 Flutter 的窗口生命周期管理
- ✅ 不涉及 Windows HWND 父子关系

### 1.3 对 AuroraView 的启示

❌ **Flet 不适用于我们的场景**,因为:
1. 不支持嵌入到 DCC 应用(Maya, Houdini 等)
2. 必须是独立应用
3. 无法解决我们的 HWND 嵌入问题

---

## 2. pywebview 项目分析

### 2.1 技术架构

**核心技术栈:**
- **Windows**: WinForms + Edge WebView2 / MSHTML
- **macOS**: Cocoa + WebKit
- **Linux**: GTK + WebKit 或 QT + QtWebEngine
- **语言**: Python (使用 pythonnet 调用 .NET)

**关键特点:**
1. ✅ **支持嵌入模式** - 可以设置 parent window
2. ✅ **轻量级** - 使用系统原生 WebView
3. ✅ **跨平台** - 支持主流桌面平台

### 2.2 Windows 实现 (WinForms)

根据 GitHub 源码分析,pywebview 在 Windows 上的实现:

**文件**: `webview/platforms/winforms.py`

**关键代码片段** (基于源码分析):

```python
# pywebview 使用 WinForms 创建窗口
from System.Windows.Forms import Form, Application

class BrowserView(Form):
    def __init__(self, window):
        self.window = window
        # 创建 WinForms 窗口
        
    def show(self):
        # 显示窗口
        if self.window.parent:
            # 如果有父窗口,设置为子窗口
            self.set_parent(self.window.parent)
        
        # 运行消息循环
        Application.Run(self)
```

### 2.3 窗口关闭处理

**pywebview 的关键策略:**

1. **使用 WinForms 的事件系统**
   ```python
   # WinForms 自动处理窗口消息
   self.FormClosing += self.on_closing
   self.FormClosed += self.on_closed
   ```

2. **不需要手动处理 WM_DESTROY**
   - WinForms 框架自动处理所有 Windows 消息
   - 包括 WM_CLOSE, WM_DESTROY, WM_NCDESTROY

3. **消息循环由 WinForms 管理**
   ```python
   # WinForms 提供完整的消息循环
   Application.Run(form)  # 自动处理所有消息
   ```

### 2.4 嵌入模式实现

**pywebview 如何处理父窗口:**

```python
# 设置父窗口 (伪代码,基于分析)
from System import IntPtr
from System.Windows.Forms import NativeWindow

def set_parent(self, parent_handle):
    # 将 WinForms 窗口设置为子窗口
    parent_ptr = IntPtr(parent_handle)
    # 使用 Win32 API 设置父窗口
    SetParent(self.Handle, parent_ptr)
```

**关键点:**
- ✅ 使用 `SetParent` API 设置父子关系
- ✅ WinForms 自动处理所有窗口消息
- ✅ 不需要手动 pump 消息

### 2.5 为什么 pywebview 没有我们的问题?

**核心原因: WinForms 框架的完整性**

1. **完整的消息循环**
   - WinForms 提供完整的 `Application.Run()` 消息循环
   - 自动处理所有 Windows 消息,包括 WM_DESTROY

2. **事件驱动模型**
   - 使用 C# 事件系统,不需要手动处理消息
   - `FormClosing`, `FormClosed` 等事件自动触发

3. **框架级别的资源管理**
   - WinForms 自动管理窗口生命周期
   - Dispose 模式确保资源正确释放

---

## 3. AuroraView 当前问题分析

### 3.1 我们的架构

```
Rust (wry/tao) → Windows API → HWND
```

**问题:**
- ❌ 使用 `tao` 创建窗口,但在嵌入模式下**不运行事件循环**
- ❌ 调用 `DestroyWindow()` 后,消息队列中的 WM_DESTROY 无人处理
- ❌ 窗口句柄被销毁,但窗口仍然可见

### 3.2 pywebview 的架构

```
Python → pythonnet → C# WinForms → Windows API
```

**优势:**
- ✅ WinForms 提供完整的消息循环
- ✅ 事件系统自动处理窗口关闭
- ✅ 框架级别的资源管理

---

## 4. 解决方案对比

### 方案 A: 继续使用 Rust/wry (当前方案)

**优点:**
- 轻量级,无需额外依赖
- 跨平台支持好
- 性能优秀

**缺点:**
- ❌ 需要手动处理 Windows 消息
- ❌ 嵌入模式下的窗口管理复杂
- ❌ 需要自己实现消息泵

**当前修复:**
```rust
// 在 DestroyWindow 后处理消息
DestroyWindow(hwnd);

let mut msg = MSG::default();
while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
    TranslateMessage(&msg);
    DispatchMessageW(&msg);
}
```

### 方案 B: 使用 Qt WebEngine (推荐)

**优点:**
- ✅ 完整的窗口管理框架
- ✅ 自动处理所有消息
- ✅ 与 DCC 应用(Maya/Houdini)完美集成
- ✅ 跨平台支持
- ✅ 成熟稳定

**缺点:**
- 需要 Qt 依赖
- Python 实现,性能略低于 Rust

**实现:**
```python
from PySide2.QtWebEngineWidgets import QWebEngineView

class AuroraViewQt(QWebEngineView):
    def __init__(self, parent=None):
        super().__init__(parent)
        # Qt 自动处理所有窗口消息
```

### 方案 C: 使用 WinForms (类似 pywebview)

**优点:**
- ✅ 完整的消息循环
- ✅ 事件驱动模型
- ✅ 自动资源管理

**缺点:**
- ❌ 仅支持 Windows
- ❌ 需要 pythonnet 依赖
- ❌ 跨平台支持差

---

## 5. 最终建议

### 5.1 短期方案 (已实现)

继续使用 Rust/wry,但添加消息处理:

```rust
// src/webview/aurora_view.rs
unsafe {
    DestroyWindow(hwnd);
    
    // 处理待处理的消息
    let mut msg = MSG::default();
    while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    
    // 短暂延迟确保窗口消失
    std::thread::sleep(std::time::Duration::from_millis(50));
}
```

**状态**: ✅ 已实现并测试

### 5.2 长期方案 (推荐)

**迁移到 Qt WebEngine**

**理由:**
1. ✅ **完全避免 HWND 问题** - Qt 框架自动处理
2. ✅ **与 DCC 应用完美集成** - Maya/Houdini 都使用 Qt
3. ✅ **跨平台支持** - Windows, macOS, Linux
4. ✅ **成熟稳定** - Qt WebEngine 是成熟的解决方案
5. ✅ **简化代码** - 不需要处理底层 Windows API

**实现路径:**
1. 创建 `python/auroraview/qt_webview.py` (✅ 已完成)
2. 提供与现有 API 兼容的接口
3. 逐步迁移用户到 Qt 版本
4. 长期废弃 Rust 版本

---

## 6. 关键发现总结

### 6.1 Flet

- ❌ **不适用** - 不支持嵌入模式
- 使用 Flutter 独立窗口
- 无法解决我们的问题

### 6.2 pywebview

- ✅ **部分适用** - 支持嵌入模式
- 使用 WinForms 框架避免手动消息处理
- 关键: **框架级别的消息循环**

### 6.3 核心教训

**问题根源:**
- 在嵌入模式下不运行事件循环
- 手动调用 `DestroyWindow()` 后,消息无人处理

**解决思路:**
1. **短期**: 手动处理 WM_DESTROY 消息 (已实现)
2. **长期**: 使用提供完整消息循环的框架 (Qt)

---

## 7. 参考资料

- [Flet GitHub](https://github.com/flet-dev/flet)
- [pywebview GitHub](https://github.com/r0x0r/pywebview)
- [pywebview WinForms 实现](https://github.com/r0x0r/pywebview/blob/master/webview/platforms/winforms.py)
- [Qt WebEngine 文档](https://doc.qt.io/qt-5/qtwebengine-index.html)
- [Windows 消息处理文档](https://docs.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues)

---

## 8. 结论

**Flet**: 不适用,不支持嵌入模式

**pywebview**: 使用 WinForms 框架避免手动消息处理,这是关键

**AuroraView 最佳方案**: 
- **短期**: 继续使用 Rust + 手动消息处理 ✅
- **长期**: 迁移到 Qt WebEngine 🎯

Qt WebEngine 方案可以完全避免当前的所有 HWND 相关问题,并提供更好的 DCC 集成。

