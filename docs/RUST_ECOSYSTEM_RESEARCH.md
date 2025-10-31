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

- [OK] 创建嵌入式窗口成功(设置 parent HWND)
- [OK] 窗口显示正常
- [ERROR] 调用 `DestroyWindow()` 后窗口仍然可见
- [ERROR] WM_DESTROY 和 WM_NCDESTROY 消息未被处理

### 1.3 根本原因

**在嵌入模式下,`tao` 不运行事件循环**:

```rust
// 正常模式 (独立窗口)
event_loop.run(|event, _, control_flow| {
    // 自动处理所有 Windows 消息
});

// 嵌入模式 (parent HWND)
// [ERROR] 不运行事件循环
// [ERROR] 消息队列中的 WM_DESTROY 无人处理
```

---

## 2. Rust GUI 框架调研

### 2.1 winit (tao 的上游)

**项目**: https://github.com/rust-windowing/winit

**特点**:
- [OK] 跨平台窗口创建库
- [OK] 提供事件循环抽象
- [OK] 支持子窗口 (child window)

**嵌入模式支持**:
```rust
// winit 支持设置父窗口
use winit::platform::windows::WindowBuilderExtWindows;

let window = WindowBuilder::new()
    .with_parent_window(parent_hwnd)
    .build(&event_loop)?;
```

**问题**:
- [ERROR] **与我们相同的问题** - 需要运行事件循环
- [ERROR] 嵌入模式下不能运行 `event_loop.run()`
- [ERROR] 没有提供独立的消息泵 API

**结论**: [ERROR] **不适用** - `tao` 就是 `winit` 的 fork,问题相同

---

### 2.2 native-windows-gui (NWG)

**项目**: https://github.com/gabdube/native-windows-gui

**特点**:
- [OK] 纯 Windows GUI 库
- [OK] 轻量级,直接封装 Windows API
- [OK] 提供消息循环管理

**消息循环实现**:
```rust
use native_windows_gui as nwg;

// NWG 提供消息循环
nwg::dispatch_thread_events();
```

**嵌入模式支持**:
- [WARNING] **主要用于创建独立窗口**
- [WARNING] 不是为嵌入场景设计的
- [WARNING] 没有找到嵌入模式的文档或示例

**结论**: [WARNING] **部分适用** - 可以参考其消息循环实现,但不直接支持嵌入模式

---

### 2.3 druid

**项目**: https://github.com/linebender/druid

**特点**:
- [OK] 数据驱动的 GUI 框架
- [OK] 使用 `druid-shell` 处理窗口
- [OK] 跨平台支持

**问题**:
- [ERROR] **不支持嵌入模式**
- [ERROR] 必须创建独立窗口
- [ERROR] 框架较重,不适合我们的轻量级需求

**结论**: [ERROR] **不适用**

---

### 2.4 iced

**项目**: https://github.com/iced-rs/iced

**特点**:
- [OK] 现代化 GUI 框架
- [OK] 基于 Elm 架构
- [OK] 使用 `winit` 作为窗口后端

**问题**:
- [ERROR] **与 winit 相同的限制**
- [ERROR] 不支持嵌入模式
- [ERROR] 框架较重

**结论**: [ERROR] **不适用**

---

### 2.5 egui

**项目**: https://github.com/emilk/egui

**特点**:
- [OK] 即时模式 GUI (Immediate Mode)
- [OK] 轻量级
- [OK] 可以嵌入到任何渲染循环

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
- [WARNING] **egui 是 UI 框架,不是窗口管理器**
- [WARNING] 仍然需要底层窗口(如 winit)
- [WARNING] 不解决我们的窗口关闭问题

**结论**: [ERROR] **不适用** - 解决的是不同层面的问题

---

## 3. Windows 消息循环 Crate 调研

### 3.1 windows-rs (官方)

**项目**: https://github.com/microsoft/windows-rs

**特点**:
- [OK] Microsoft 官方 Rust Windows API 绑定
- [OK] 完整的 Windows API 覆盖
- [OK] 类型安全

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
- [OK] **这正是我们当前使用的**
- [OK] 轻量级,无额外依赖
- [OK] 直接控制消息处理

**结论**: [OK] **已在使用** - 这是最轻量级的方案

---

### 3.2 winapi (旧版)

**项目**: https://github.com/retep998/winapi-rs

**状态**: [WARNING] **已被 windows-rs 取代**

**结论**: [ERROR] **不推荐** - 使用 `windows-rs` 代替

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

**结论**: [WARNING] **wry/tao 本身不提供解决方案**

---

### 4.2 fltk-webview

**项目**: https://github.com/MoAlyousef/fltk-webview

**特点**:
- [OK] 将 WebView 嵌入到 FLTK 窗口
- [OK] 支持从 raw window handle 创建

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
- [ERROR] **需要引入 FLTK 框架**
- [ERROR] 不是轻量级解决方案
- [ERROR] 与我们的 `wry` 架构不兼容

**结论**: [ERROR] **不适用** - 需要切换整个 GUI 框架

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
- [WARNING] **需要外部定期调用**
- [WARNING] 在 Maya 中需要使用 `cmds.scriptJob`
- [WARNING] 我们已经在这样做了

**结论**: [OK] **已采用** - 这是我们当前的方案

---

## 6. 最终结论

### 6.1 Rust 生态系统现状

**关键发现**:
1. [ERROR] **没有轻量级的 Rust crate 提供完整的嵌入式窗口消息循环管理**
2. [ERROR] **所有 GUI 框架都假设控制事件循环**
3. [OK] **最轻量级的方案就是直接使用 `windows-rs`**

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
- [OK] **最轻量级** - 只依赖 `windows-rs`
- [OK] **完全控制** - 精确控制消息处理
- [OK] **无额外依赖** - 不引入重度框架
- [OK] **性能最优** - 直接调用 Windows API

---

## 7. 替代方案对比

| 方案 | 轻量级 | 嵌入支持 | 跨平台 | 维护成本 | 推荐度 |
|------|--------|----------|--------|----------|--------|
| **当前方案 (windows-rs)** | [OK] 最轻 | [OK] 完全 | [ERROR] 仅 Windows | [WARNING] 中等 | [STAR][STAR][STAR][STAR][STAR] |
| Qt WebEngine | [ERROR] 重度 | [OK] 完全 | [OK] 全平台 | [OK] 低 | [STAR][STAR][STAR][STAR] |
| native-windows-gui | [OK] 轻量 | [WARNING] 部分 | [ERROR] 仅 Windows | [WARNING] 中等 | [STAR][STAR] |
| fltk-webview | [WARNING] 中等 | [OK] 完全 | [OK] 全平台 | [WARNING] 中等 | [STAR][STAR] |
| winit/tao | [OK] 轻量 | [ERROR] 不支持 | [OK] 全平台 | - | [ERROR] |

---

## 8. 最终建议

### 8.1 短期方案 (推荐) [OK]

**继续使用当前的 `windows-rs` 方案**

**理由**:
1. [OK] **最轻量级** - 无额外依赖
2. [OK] **已经实现** - 代码已经工作
3. [OK] **性能最优** - 直接 Windows API
4. [OK] **完全控制** - 可以精确调试

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

1. [ERROR] **Rust 生态系统中没有轻量级的嵌入式窗口消息循环管理库**
2. [OK] **我们当前的 `windows-rs` 方案是最轻量级的**
3. [OK] **不需要引入额外的 crate 或框架**
4. [WARNING] **如果需要跨平台,Qt WebEngine 是最佳选择**

### 行动建议

**立即行动**:
- [OK] 继续使用当前方案
- [OK] 优化消息处理逻辑
- [OK] 添加更详细的日志

**未来考虑**:
-  如果需要跨平台,评估 Qt WebEngine
-  关注 wry/tao 的更新,看是否添加嵌入模式支持
-  考虑将消息泵逻辑封装成独立模块

**不推荐**:
- [ERROR] 不要引入 FLTK 或其他 GUI 框架
- [ERROR] 不要切换到 native-windows-gui
- [ERROR] 不要尝试使用 winit/tao 的嵌入模式(不存在)

