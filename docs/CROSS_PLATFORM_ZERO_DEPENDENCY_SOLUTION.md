# 跨平台零 Python 依赖 WebView 解决方案调研

## 调研目标

寻找一个**跨平台** (Windows, macOS, Linux) 的 Rust WebView 解决方案,满足以下严格要求:

1. ✅ **Rust 实现** - 整个解决方案用 Rust 编写
2. ✅ **零 Python 依赖** - 编译后的 .pyd/.so 不依赖任何第三方 Python 包
3. ✅ **独立分发** - 用户只需安装我们的 Python 包
4. ✅ **嵌入模式支持** - 支持嵌入到现有窗口 (DCC 应用如 Maya)

---

## 1. 核心发现 ⚠️

### 1.1 关键结论

**❌ 不存在完美的跨平台嵌入式窗口解决方案**

经过全面调研,**没有任何 Rust WebView 库提供跨平台的嵌入模式支持**:

- ❌ **wry** - 仅在 Windows/macOS/Linux(X11) 支持子窗口,但**不支持嵌入到外部窗口**
- ❌ **webview/webview** (C/C++) - 不支持嵌入模式
- ❌ **Tauri** - 基于 wry,相同限制
- ❌ **Dioxus** - 基于 wry,相同限制

### 1.2 wry 的 "Child WebView" 功能

**重要澄清**: wry 的 `build_as_child()` **不是**我们需要的嵌入模式!

```rust
// wry 的 "child webview" - 在自己的窗口内创建子 WebView
let webview = WebViewBuilder::new()
    .with_bounds(Rect { ... })
    .build_as_child(&window)  // ❌ 这是在 wry 自己的窗口内
    .unwrap();
```

**这不是嵌入到外部窗口 (如 Maya 窗口)**,而是:
- 在 wry 创建的窗口内部创建一个子 WebView
- 仍然需要 wry 拥有父窗口
- **无法嵌入到 Maya/Houdini 等 DCC 应用的窗口**

---

## 2. 现有 Rust WebView 库详细分析

### 2.1 wry (tauri-apps/wry)

**项目**: https://github.com/tauri-apps/wry

**跨平台支持**: ✅ Windows, macOS, Linux, iOS, Android

**底层技术**:
- Windows: WebView2 (Edge Chromium)
- macOS: WebKit
- Linux: WebKitGTK
- iOS: WebKit
- Android: Android WebView

**嵌入模式支持**: ❌ **不支持**

**关键 Issues**:
- [#650 - Construct WebView from raw window handle](https://github.com/tauri-apps/wry/issues/650)
  - **状态**: Closed as "not planned"
  - **原因**: 需要大规模重构
  
- [#677 - Integrate WebView into raw window](https://github.com/tauri-apps/wry/issues/677)
  - **状态**: Open,但无进展
  - **结论**: wry 不支持从 raw window handle 创建 WebView

**Child WebView 功能**:
```rust
// ⚠️ 这不是嵌入到外部窗口!
WebViewBuilder::new()
    .with_bounds(Rect { ... })
    .build_as_child(&window)  // window 必须是 wry/tao 创建的
```

**限制**:
- ❌ 无法从外部 HWND/NSView/GtkWidget 创建 WebView
- ❌ 必须使用 tao/winit 创建的窗口
- ❌ 不支持嵌入到 DCC 应用

**结论**: ❌ **不适用于我们的场景**

---

### 2.2 webview/webview (C/C++)

**项目**: https://github.com/webview/webview

**语言**: C/C++ (有 Rust 绑定)

**跨平台支持**: ✅ Windows, macOS, Linux

**底层技术**:
- Windows: WebView2
- macOS: WebKit (Cocoa)
- Linux: WebKitGTK

**嵌入模式支持**: ❌ **不支持**

**API 设计**:
```c
// 只能创建独立窗口
webview_t w = webview_create(0, NULL);
webview_set_title(w, "Example");
webview_run(w);
```

**限制**:
- ❌ 没有 API 接受外部窗口句柄
- ❌ 必须创建独立窗口
- ❌ 不支持嵌入模式

**Rust 绑定**:
- [Boscop/web-view](https://github.com/Boscop/web-view) - 已过时
- 没有活跃维护的 Rust 绑定

**结论**: ❌ **不适用**

---

### 2.3 其他 Rust WebView 项目

#### Tauri
- **基于**: wry + tao
- **限制**: 与 wry 相同,不支持嵌入模式
- **结论**: ❌ 不适用

#### Dioxus
- **基于**: wry (desktop 模式)
- **限制**: 与 wry 相同
- **结论**: ❌ 不适用

---

## 3. 跨平台嵌入模式的技术挑战

### 3.1 为什么没有跨平台嵌入模式?

**根本原因**: 不同平台的窗口系统差异巨大

#### Windows
```rust
// 需要 HWND
SetParent(child_hwnd, parent_hwnd);
```

#### macOS
```objc
// 需要 NSView
[parentView addSubview:childView];
```

#### Linux (X11)
```c
// 需要 Window (X11 ID)
XReparentWindow(display, child_window, parent_window, x, y);
```

#### Linux (Wayland)
```c
// Wayland 不支持窗口重新父化!
// 必须使用 GTK 容器
gtk_container_add(GTK_CONTAINER(parent), child);
```

### 3.2 跨平台抽象的困难

**问题**:
1. **窗口句柄类型不同** - HWND vs NSView vs Window vs GtkWidget
2. **生命周期管理不同** - 谁拥有窗口?谁负责销毁?
3. **消息循环不同** - Windows 消息泵 vs Cocoa run loop vs GTK main loop
4. **Wayland 的限制** - 不支持传统的窗口重新父化

**为什么 wry 不支持**:
- wry 依赖 `tao::Window` 对象
- `tao::Window` 无法从外部窗口句柄创建
- 添加此功能需要大规模重构

---

## 4. 可行的替代方案

### 方案 A: 平台特定实现 (推荐) ⭐⭐⭐⭐⭐

**策略**: 为每个平台编写特定的嵌入代码

#### Windows
```rust
// 使用 windows-rs 直接操作 WebView2
use windows::Win32::UI::WindowsAndMessaging::*;
use webview2_com::*;

// 创建 WebView2 控制器
let controller = create_webview2_controller(parent_hwnd).await?;
```

#### macOS
```rust
// 使用 objc 直接操作 WKWebView
use objc::*;

// 创建 WKWebView 并添加到父视图
let webview = WKWebView::new(frame);
parent_view.addSubview(webview);
```

#### Linux
```rust
// 使用 gtk-rs 直接操作 WebKitGTK
use gtk::prelude::*;
use webkit2gtk::*;

// 创建 WebView 并添加到 GTK 容器
let webview = WebView::new();
parent_container.add(&webview);
```

**优势**:
- ✅ **完全控制** - 精确控制每个平台的行为
- ✅ **零依赖** - 只依赖系统原生 WebView
- ✅ **最佳性能** - 直接调用平台 API

**劣势**:
- ❌ **维护成本高** - 需要维护三套代码
- ❌ **平台专家知识** - 需要深入了解每个平台

**实现示例**:
```rust
// python/auroraview/src/lib.rs
#[cfg(target_os = "windows")]
mod windows_webview;

#[cfg(target_os = "macos")]
mod macos_webview;

#[cfg(target_os = "linux")]
mod linux_webview;

#[pyclass]
pub struct WebView {
    #[cfg(target_os = "windows")]
    inner: windows_webview::WindowsWebView,
    
    #[cfg(target_os = "macos")]
    inner: macos_webview::MacOSWebView,
    
    #[cfg(target_os = "linux")]
    inner: linux_webview::LinuxWebView,
}
```

**结论**: ✅ **这是唯一可行的跨平台零依赖方案**

---

### 方案 B: 使用 Qt WebEngine (备选) ⭐⭐⭐⭐

**问题**: ❌ **违反"零 Python 依赖"要求**

即使用 Rust 实现,Qt WebEngine 仍然需要:
- Qt 运行时库 (QtCore, QtGui, QtWebEngine)
- 用户必须安装 Qt

**结论**: ❌ 不符合要求

---

### 方案 C: 创建独立窗口 (妥协方案) ⭐⭐

**策略**: 不嵌入,而是创建独立的浮动窗口

```rust
// 使用 wry 创建独立窗口
let webview = WebViewBuilder::new()
    .with_url("https://example.com")
    .build(&window)?;
```

**优势**:
- ✅ 跨平台支持
- ✅ 使用现有的 wry

**劣势**:
- ❌ 不是嵌入式窗口
- ❌ 用户体验差 (独立窗口)
- ❌ 不符合 DCC 应用的集成需求

**结论**: ⚠️ 仅作为最后的备选方案

---

## 5. 推荐方案: 平台特定实现

### 5.1 架构设计

```
auroraview (Python 包)
  ↓
auroraview_core.pyd/.so (Rust + PyO3)
  ↓
┌─────────────┬──────────────┬─────────────┐
│   Windows   │    macOS     │    Linux    │
│  WebView2   │   WebKit     │ WebKitGTK   │
│ (windows-rs)│   (objc)     │  (gtk-rs)   │
└─────────────┴──────────────┴─────────────┘
```

### 5.2 依赖清单

#### Windows
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
] }
webview2-com = "0.33"
```

#### macOS
```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2"
cocoa = "0.25"
webkit2 = "0.1"  # 或直接使用 objc 调用
```

#### Linux
```toml
[target.'cfg(target_os = "linux")'.dependencies]
gtk = "0.18"
webkit2gtk = "2.0"
```

### 5.3 零 Python 依赖保证

**关键**: 所有依赖都是 Rust crate,编译成二进制

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]  # 编译成 .pyd/.so

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module", "abi3-py37"] }
# 平台特定依赖 (见上)
```

**编译后**:
```
auroraview/
  __init__.py
  _core.pyd  # Windows (包含所有依赖)
  _core.so   # Linux (包含所有依赖)
  _core.dylib  # macOS (包含所有依赖)
```

**用户安装**:
```bash
pip install auroraview  # 无需其他依赖!
```

---

## 6. 实现路线图

### 阶段 1: Windows 实现 (优先)

1. ✅ 使用 `windows-rs` + `webview2-com`
2. ✅ 实现嵌入模式 (SetParent)
3. ✅ 解决窗口关闭问题 (已完成)
4. ✅ 通过 PyO3 暴露 API

### 阶段 2: macOS 实现

1. 使用 `objc` crate
2. 创建 WKWebView
3. 嵌入到 NSView
4. 处理 Cocoa 事件循环

### 阶段 3: Linux 实现

1. 使用 `gtk-rs` + `webkit2gtk`
2. 创建 WebKitWebView
3. 嵌入到 GTK 容器
4. 处理 GTK 主循环

### 阶段 4: 统一 API

```rust
#[pyclass]
pub struct WebView {
    #[cfg(target_os = "windows")]
    inner: WindowsWebView,
    
    #[cfg(target_os = "macos")]
    inner: MacOSWebView,
    
    #[cfg(target_os = "linux")]
    inner: LinuxWebView,
}

#[pymethods]
impl WebView {
    #[new]
    pub fn new(parent_handle: usize) -> PyResult<Self> {
        #[cfg(target_os = "windows")]
        return Ok(Self {
            inner: WindowsWebView::new(parent_handle as HWND)?,
        });
        
        #[cfg(target_os = "macos")]
        return Ok(Self {
            inner: MacOSWebView::new(parent_handle as *mut NSView)?,
        });
        
        #[cfg(target_os = "linux")]
        return Ok(Self {
            inner: LinuxWebView::new(parent_handle as *mut GtkWidget)?,
        });
    }
}
```

---

## 7. 成功案例参考

### 7.1 VST 插件开发

**背景**: VST 音频插件也需要嵌入 UI 到宿主窗口

**解决方案**: 平台特定实现
- Windows: 使用 HWND
- macOS: 使用 NSView
- Linux: 使用 X11 Window

**参考项目**:
- [vst-rs](https://github.com/RustAudio/vst-rs)
- 每个平台都有独立的窗口处理代码

### 7.2 浏览器嵌入

**Chromium Embedded Framework (CEF)**:
- 也是平台特定实现
- Windows: HWND
- macOS: NSView
- Linux: X11/GTK

---

## 8. 最终建议

### 8.1 推荐方案

**✅ 采用平台特定实现**

**理由**:
1. ✅ **唯一可行的跨平台零依赖方案**
2. ✅ **完全控制** - 可以精确解决每个平台的问题
3. ✅ **最佳性能** - 直接使用系统原生 WebView
4. ✅ **零 Python 依赖** - 所有代码编译成二进制

### 8.2 实施建议

**短期 (1-2 个月)**:
1. ✅ 完善 Windows 实现 (已基本完成)
2. 🔄 开始 macOS 实现
3. 📋 规划 Linux 实现

**中期 (3-6 个月)**:
1. 完成所有平台实现
2. 统一 Python API
3. 编写跨平台测试

**长期 (6+ 个月)**:
1. 优化性能
2. 添加高级功能
3. 完善文档

### 8.3 不推荐的方案

❌ **不要尝试**:
1. 等待 wry 添加嵌入模式支持 (不会发生)
2. 使用 Qt WebEngine (违反零依赖要求)
3. 创建独立窗口 (不符合需求)

---

## 9. 参考资料

### 9.1 Rust 项目
- [wry](https://github.com/tauri-apps/wry) - 跨平台 WebView (不支持嵌入)
- [webview/webview](https://github.com/webview/webview) - C/C++ WebView
- [windows-rs](https://github.com/microsoft/windows-rs) - Windows API 绑定
- [objc](https://github.com/SSheldon/rust-objc) - Objective-C 运行时
- [gtk-rs](https://gtk-rs.org/) - GTK 绑定

### 9.2 平台文档
- [WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/)
- [WKWebView](https://developer.apple.com/documentation/webkit/wkwebview)
- [WebKitGTK](https://webkitgtk.org/)

### 9.3 相关 Issues
- [wry#650](https://github.com/tauri-apps/wry/issues/650) - Raw window handle
- [wry#677](https://github.com/tauri-apps/wry/issues/677) - Integrate into raw window

---

## 10. 总结

### 核心结论

1. ❌ **不存在现成的跨平台嵌入式 WebView Rust 库**
2. ✅ **平台特定实现是唯一可行方案**
3. ✅ **可以实现零 Python 依赖**
4. ✅ **我们当前的 Windows 实现方向正确**

### 行动建议

**立即行动**:
- ✅ 继续完善 Windows 实现
- 🔄 开始 macOS 原型开发
- 📋 研究 Linux/GTK 集成

**未来规划**:
- 📋 创建统一的跨平台 API
- 📋 编写平台特定的测试
- 📋 优化构建和分发流程

**成功指标**:
- ✅ 用户只需 `pip install auroraview`
- ✅ 无需安装 Qt 或其他依赖
- ✅ 支持 Windows, macOS, Linux
- ✅ 可以嵌入到 DCC 应用 (Maya, Houdini 等)

