# Rust 生态系统嵌入式窗口解决方案调研

## 调研目的

在 Rust 生态系统中寻找**轻量级**的解决方案,用于解决 `wry`/`tao` 嵌入式窗口(设置 parent HWND)的关闭问题,**避免引入 Qt 这样的重度依赖**。

---

## 1. 核心问题回顾

### 1.1 当前架构

```
AuroraView (Rust)
  ↓
wry (WebView wrapper)
  ↓
tao (Window creation, fork of winit)
  ↓
Windows API (HWND, DestroyWindow, etc.)
```

### 1.2 问题症状

- ✅ 创建嵌入式窗口成功(设置 parent HWND)
- ✅ 窗口显示正常
- ❌ 调用 `DestroyWindow()` 后窗口仍然可见
- ❌ WM_DESTROY 和 WM_NCDESTROY 消息未被处理

### 1.3 根本原因

**在嵌入模式下,`tao` 不运行事件循环**:

```rust
// 正常模式 (独立窗口)
event_loop.run(|event, _, control_flow| {
    // 自动处理所有 Windows 消息
});

// 嵌入模式 (parent HWND)
// ❌ 不运行事件循环
// ❌ 消息队列中的 WM_DESTROY 无人处理
```

---

## 2. Rust GUI 框架调研

### 2.1 winit (tao 的上游)

**项目**: https://github.com/rust-windowing/winit

**特点**:
- ✅ 跨平台窗口创建库
- ✅ 提供事件循环抽象
- ✅ 支持子窗口 (child window)

**嵌入模式支持**:
```rust
// winit 支持设置父窗口
use winit::platform::windows::WindowBuilderExtWindows;

let window = WindowBuilder::new()
    .with_parent_window(parent_hwnd)
    .build(&event_loop)?;
```

**问题**:
- ❌ **与我们相同的问题** - 需要运行事件循环
- ❌ 嵌入模式下不能运行 `event_loop.run()`
- ❌ 没有提供独立的消息泵 API

**结论**: ❌ **不适用** - `tao` 就是 `winit` 的 fork,问题相同

---

### 2.2 native-windows-gui (NWG)

**项目**: https://github.com/gabdube/native-windows-gui

**特点**:
- ✅ 纯 Windows GUI 库
- ✅ 轻量级,直接封装 Windows API
- ✅ 提供消息循环管理

**消息循环实现**:
```rust
use native_windows_gui as nwg;

// NWG 提供消息循环
nwg::dispatch_thread_events();
```

**嵌入模式支持**:
- ⚠️ **主要用于创建独立窗口**
- ⚠️ 不是为嵌入场景设计的
- ⚠️ 没有找到嵌入模式的文档或示例

**结论**: ⚠️ **部分适用** - 可以参考其消息循环实现,但不直接支持嵌入模式

---

### 2.3 druid

**项目**: https://github.com/linebender/druid

**特点**:
- ✅ 数据驱动的 GUI 框架
- ✅ 使用 `druid-shell` 处理窗口
- ✅ 跨平台支持

**问题**:
- ❌ **不支持嵌入模式**
- ❌ 必须创建独立窗口
- ❌ 框架较重,不适合我们的轻量级需求

**结论**: ❌ **不适用**

---

### 2.4 iced

**项目**: https://github.com/iced-rs/iced

**特点**:
- ✅ 现代化 GUI 框架
- ✅ 基于 Elm 架构
- ✅ 使用 `winit` 作为窗口后端

**问题**:
- ❌ **与 winit 相同的限制**
- ❌ 不支持嵌入模式
- ❌ 框架较重

**结论**: ❌ **不适用**

---

### 2.5 egui

**项目**: https://github.com/emilk/egui

**特点**:
- ✅ 即时模式 GUI (Immediate Mode)
- ✅ 轻量级
- ✅ 可以嵌入到任何渲染循环

**嵌入模式**:
```rust
// egui 可以嵌入到现有窗口
let egui_ctx = egui::Context::default();

// 在渲染循环中
egui_ctx.run(input, |ctx| {
    egui::Window::new("My Window").show(ctx, |ui| {
        ui.label("Hello");
    });
});
```

**问题**:
- ⚠️ **egui 是 UI 框架,不是窗口管理器**
- ⚠️ 仍然需要底层窗口(如 winit)
- ⚠️ 不解决我们的窗口关闭问题

**结论**: ❌ **不适用** - 解决的是不同层面的问题

---

## 3. Windows 消息循环 Crate 调研

### 3.1 windows-rs (官方)

**项目**: https://github.com/microsoft/windows-rs

**特点**:
- ✅ Microsoft 官方 Rust Windows API 绑定
- ✅ 完整的 Windows API 覆盖
- ✅ 类型安全

**消息循环示例**:
```rust
use windows::Win32::UI::WindowsAndMessaging::*;

unsafe {
    let mut msg = MSG::default();
    
    // 标准消息循环
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    
    // 或者使用 PeekMessage (非阻塞)
    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
```

**优势**:
- ✅ **这正是我们当前使用的**
- ✅ 轻量级,无额外依赖
- ✅ 直接控制消息处理

**结论**: ✅ **已在使用** - 这是最轻量级的方案

---

### 3.2 winapi (旧版)

**项目**: https://github.com/retep998/winapi-rs

**状态**: ⚠️ **已被 windows-rs 取代**

**结论**: ❌ **不推荐** - 使用 `windows-rs` 代替

---

## 4. wry/tao 生态系统调研

### 4.1 Tauri 项目

**项目**: https://github.com/tauri-apps/tauri

**相关 Issues**:
- [#650 - Construct WebView from raw window handle](https://github.com/tauri-apps/wry/issues/650)
- [#677 - Integrate WebView into raw window](https://github.com/tauri-apps/wry/issues/677)

**关键发现**:

1. **wry 不支持从 raw window handle 创建 WebView**
   - Issue #650 请求此功能,但被标记为 "not planned"
   - 原因: 需要大规模重构

2. **嵌入模式的限制**
   - wry 依赖 `tao::Window` 对象
   - 无法从现有 HWND 创建 Window
   - Qt 支持 `QWindow::fromWinId()`,但 tao 不支持

3. **社区解决方案**
   - 有人提出使用 `fltk-webview` crate
   - 但这需要切换到 FLTK GUI 框架

**结论**: ⚠️ **wry/tao 本身不提供解决方案**

---

### 4.2 fltk-webview

**项目**: https://github.com/MoAlyousef/fltk-webview

**特点**:
- ✅ 将 WebView 嵌入到 FLTK 窗口
- ✅ 支持从 raw window handle 创建

**示例**:
```rust
use fltk::*;
use fltk_webview::*;

let app = app::App::default();
let mut win = window::Window::default();

// 从 FLTK 窗口获取 raw handle
let handle = win.raw_handle();

// 创建 WebView
let webview = Webview::create(false, Some(handle));
```

**问题**:
- ❌ **需要引入 FLTK 框架**
- ❌ 不是轻量级解决方案
- ❌ 与我们的 `wry` 架构不兼容

**结论**: ❌ **不适用** - 需要切换整个 GUI 框架

---

## 5. 其他项目的解决方案

### 5.1 VST 插件开发

**背景**: VST 插件也需要嵌入到宿主窗口

**解决方案**:
1. **使用 `run_return`** (tao 支持)
   ```rust
   // 不阻塞的事件循环
   event_loop.run_return(|event, _, control_flow| {
       // 处理事件
   });
   ```

2. **宿主定期调用 `idle()`**
   - 宿主每帧调用插件的 `idle()` 方法
   - 插件在 `idle()` 中处理消息

**问题**:
- ⚠️ **需要外部定期调用**
- ⚠️ 在 Maya 中需要使用 `cmds.scriptJob`
- ⚠️ 我们已经在这样做了

**结论**: ✅ **已采用** - 这是我们当前的方案

---

## 6. 最终结论

### 6.1 Rust 生态系统现状

**关键发现**:
1. ❌ **没有轻量级的 Rust crate 提供完整的嵌入式窗口消息循环管理**
2. ❌ **所有 GUI 框架都假设控制事件循环**
3. ✅ **最轻量级的方案就是直接使用 `windows-rs`**

### 6.2 为什么没有轻量级解决方案?

**原因分析**:

1. **Rust GUI 生态系统的设计哲学**
   - 大多数框架假设**拥有**事件循环
   - 嵌入模式是边缘用例

2. **Windows API 的复杂性**
   - 消息循环看似简单,实则复杂
   - 需要处理各种边缘情况
   - 没有"银弹"解决方案

3. **跨平台的挑战**
   - 大多数 Rust GUI 库追求跨平台
   - 嵌入模式在不同平台差异巨大
   - 很难提供统一抽象

### 6.3 我们的当前方案是最优的

**已实现的方案**:
```rust
// src/webview/aurora_view.rs
unsafe {
    DestroyWindow(hwnd);
    
    // 手动处理待处理的消息
    let mut msg = MSG::default();
    while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
```

**优势**:
- ✅ **最轻量级** - 只依赖 `windows-rs`
- ✅ **完全控制** - 精确控制消息处理
- ✅ **无额外依赖** - 不引入重度框架
- ✅ **性能最优** - 直接调用 Windows API

---

## 7. 替代方案对比

| 方案 | 轻量级 | 嵌入支持 | 跨平台 | 维护成本 | 推荐度 |
|------|--------|----------|--------|----------|--------|
| **当前方案 (windows-rs)** | ✅ 最轻 | ✅ 完全 | ❌ 仅 Windows | ⚠️ 中等 | ⭐⭐⭐⭐⭐ |
| Qt WebEngine | ❌ 重度 | ✅ 完全 | ✅ 全平台 | ✅ 低 | ⭐⭐⭐⭐ |
| native-windows-gui | ✅ 轻量 | ⚠️ 部分 | ❌ 仅 Windows | ⚠️ 中等 | ⭐⭐ |
| fltk-webview | ⚠️ 中等 | ✅ 完全 | ✅ 全平台 | ⚠️ 中等 | ⭐⭐ |
| winit/tao | ✅ 轻量 | ❌ 不支持 | ✅ 全平台 | - | ❌ |

---

## 8. 最终建议

### 8.1 短期方案 (推荐) ✅

**继续使用当前的 `windows-rs` 方案**

**理由**:
1. ✅ **最轻量级** - 无额外依赖
2. ✅ **已经实现** - 代码已经工作
3. ✅ **性能最优** - 直接 Windows API
4. ✅ **完全控制** - 可以精确调试

**改进建议**:
```rust
// 可以封装成独立的消息泵模块
pub struct MessagePump {
    hwnd: HWND,
}

impl MessagePump {
    pub fn process_pending_messages(&self) -> bool {
        unsafe {
            let mut msg = MSG::default();
            let mut processed = false;
            
            while PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                processed = true;
            }
            
            processed
        }
    }
}
```

### 8.2 长期方案 (如果需要跨平台)

**迁移到 Qt WebEngine**

**时机**:
- 当需要支持 macOS/Linux 时
- 当维护成本成为问题时
- 当需要更多 GUI 功能时

---

## 9. 参考资料

### 9.1 Rust 项目
- [winit](https://github.com/rust-windowing/winit) - 跨平台窗口库
- [tao](https://github.com/tauri-apps/tao) - winit fork,用于 Tauri
- [wry](https://github.com/tauri-apps/wry) - 跨平台 WebView 库
- [native-windows-gui](https://github.com/gabdube/native-windows-gui) - Windows GUI 库
- [windows-rs](https://github.com/microsoft/windows-rs) - 官方 Windows API 绑定

### 9.2 相关 Issues
- [wry#650 - Construct WebView from raw window handle](https://github.com/tauri-apps/wry/issues/650)
- [wry#677 - Integrate WebView into raw window](https://github.com/tauri-apps/wry/issues/677)
- [winit#159 - Support for creating child windows](https://github.com/rust-windowing/winit/issues/159)

### 9.3 Windows API 文档
- [Message Loop](https://docs.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues)
- [PeekMessage](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew)
- [DestroyWindow](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)

---

## 10. 总结

### 核心结论

1. ❌ **Rust 生态系统中没有轻量级的嵌入式窗口消息循环管理库**
2. ✅ **我们当前的 `windows-rs` 方案是最轻量级的**
3. ✅ **不需要引入额外的 crate 或框架**
4. ⚠️ **如果需要跨平台,Qt WebEngine 是最佳选择**

### 行动建议

**立即行动**:
- ✅ 继续使用当前方案
- ✅ 优化消息处理逻辑
- ✅ 添加更详细的日志

**未来考虑**:
- 📋 如果需要跨平台,评估 Qt WebEngine
- 📋 关注 wry/tao 的更新,看是否添加嵌入模式支持
- 📋 考虑将消息泵逻辑封装成独立模块

**不推荐**:
- ❌ 不要引入 FLTK 或其他 GUI 框架
- ❌ 不要切换到 native-windows-gui
- ❌ 不要尝试使用 winit/tao 的嵌入模式(不存在)

