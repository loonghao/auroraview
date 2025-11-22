# Standalone Mode Fix: 统一使用 event_loop.run() + 性能优化

## 已知问题

### Chrome_WidgetWin_0 错误（退出时）

**错误信息：**
```
[ERROR:window_impl.cc(122)] Failed to unregister class Chrome_WidgetWin_0. Error = 1412
```

**状态：** 无害警告，不影响功能

**说明：**
- 错误 1412 = Windows 的 `ERROR_CLASS_ALREADY_EXISTS`
- 这是 WebView2/Chromium 在 Windows 上的已知清理问题
- 错误发生在 Windows 尝试注销窗口类时
- **这不会导致程序崩溃或失败**
- 这只是一个日志消息，可以安全忽略

**为什么会发生：**
- WebView2 运行时在初始化时创建窗口类
- 退出时，清理顺序存在竞争条件
- Windows 可能尝试注销已经被注销的类

**尝试过的修复：**
- 退出前显式关闭窗口：无效
- 使用 `ControlFlow::Exit`：无效
- 这似乎是 WebView2/Chromium 内部问题

**参考资料：**
- Tauri issue: https://github.com/tauri-apps/tauri/issues/7606
- 多个项目报告相同的无害警告
- 没有已知的完全消除此消息的方法

**建议：**
- 忽略此错误消息
- 它不表示程序有问题
- 尽管有警告，程序仍然正常退出

---

## 问题背景

### 问题 1: 卡死问题

之前的实现中存在两套不同的 CLI 实现：

1. **Rust CLI** (`src/bin/cli.rs`): 直接使用 `event_loop.run()` - **工作正常**
2. **Python CLI** (`python/auroraview/__main__.py`): 使用 `show_blocking()` → `run_return()` - **卡死**

**根本原因：**
- `event_loop.run()`: 会调用 `std::process::exit()`,适合独立程序
- `event_loop.run_return()`: 正常返回,适合嵌入式场景(DCC)
- 在 Windows 上,`run_return()` 需要特殊处理才能正常启动事件循环

### 问题 2: 启动性能差

即使修复了卡死问题，启动时间仍然很慢（~9.5秒），远慢于传统浏览器。

**根本原因：**
- Standalone 模式先创建 WebView，然后通过 JavaScript 的 `window.location.href` 加载 URL
- Rust CLI 使用 `.with_url()` 在创建时直接加载 URL
- JavaScript 方式需要等待 WebView 完全初始化 → 执行脚本 → 再加载页面（多了 2 个步骤）

## 解决方案

### 1. 架构重新设计（修复卡死问题）

**正确的设计应该是：**

- **Standalone 模式**: 独立窗口,使用 `event_loop.run()` - 进程退出时窗口关闭
- **Embedded 模式**: 嵌入到 DCC,使用 `run_return()` 或不运行事件循环 - DCC 控制生命周期

### 2. 性能优化（修复启动慢问题）

**关键改进：在创建时直接加载 URL，而不是创建后通过 JavaScript 加载**

#### 修复前（慢）
```rust
// 1. 创建空白 WebView
let webview = webview_builder.build(&window)?;

// 2. 通过 JavaScript 加载 URL（需要等待 WebView 初始化）
if let Some(ref url) = config.url {
    let script = js_assets::build_load_url_script(url);
    webview.evaluate_script(&script)?;  // ← 慢！需要等待初始化
}
```

#### 修复后（快）
```rust
// 在创建时直接加载 URL
if let Some(ref url) = config.url {
    webview_builder = webview_builder.with_url(url);  // ← 快！
}
let webview = webview_builder.build(&window)?;
```

### 实现细节

#### 1. 新增 `run_standalone()` 函数

**文件**: `src/webview/standalone.rs`

```rust
/// Run standalone WebView with event_loop.run() (blocking until window closes)
///
/// This function is designed for standalone applications where the WebView owns
/// the event loop and the process should exit when the window closes.
/// It uses event_loop.run() which calls std::process::exit() on completion.
///
/// IMPORTANT: This will terminate the entire process when the window closes!
/// Only use this for standalone mode, NOT for DCC integration (embedded mode).
pub fn run_standalone(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the WebView
    let mut webview_inner = create_standalone(config, ipc_handler, message_queue)?;
    
    // Take ownership of event loop and window
    let event_loop = webview_inner.event_loop.take().ok_or("Event loop is None")?;
    let window = webview_inner.window.take().ok_or("Window is None")?;
    let webview = webview_inner.webview.clone();
    
    // Show window
    window.set_visible(true);
    window.request_redraw();
    
    // Run the event loop - this will block until window closes and then exit the process
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
```

#### 2. Python 绑定

**文件**: `src/bindings/standalone_runner.rs`

```python
from auroraview._core import run_standalone

# Run standalone WebView (blocking until window closes, then exits process)
run_standalone(
    title="My App",
    width=800,
    height=600,
    url="https://example.com"
)
```

#### 3. 更新 Python CLI

**文件**: `python/auroraview/__main__.py`

```python
from auroraview._core import run_standalone

# Run standalone WebView (blocking until window closes, then exits process)
# This uses the same event_loop.run() approach as the Rust CLI
run_standalone(
    title=args.title,
    width=args.width,
    height=args.height,
    url=url,
    html=html_content,
    dev_tools=args.debug,
)
```

## 性能对比

### 修复前（使用 JavaScript 加载 URL）
```
04:23:33.713 - 开始创建
04:23:35.017 - 构建初始化脚本 (1.3秒)
04:23:43.194 - Active (8.2秒！)
总计：~9.5秒
```

### 修复后（使用 with_url 直接加载）
```
04:43:30.858 - 开始创建
04:43:31.642 - 构建初始化脚本 (0.8秒)
04:43:35.769 - Active (4.1秒)
总计：~4.9秒
```

**性能提升：约 50%！从 9.5 秒降到 4.9 秒！** 🚀

### 3. 用户体验优化（添加加载动画）

**问题：** 即使优化后，窗口显示到页面加载完成仍有 4-5 秒白屏

**解决方案：** 先显示加载动画，后台加载真实内容

#### 实现方式

1. **创建加载动画页面** (`src/assets/html/loading.html`)
   - 紫色渐变背景
   - 旋转的加载圆环
   - 脉动的文字提示
   - 进度条动画

2. **两阶段加载策略**
   ```rust
   // 第一阶段：立即显示加载动画
   let loading_html = include_str!("../assets/html/loading.html");
   webview_builder = webview_builder.with_html(loading_html);
   let webview = webview_builder.build(&window)?;

   // 第二阶段：后台加载真实内容
   if let Some(ref url) = target_url {
       let script = js_assets::build_load_url_script(url);
       webview.evaluate_script(&script)?;
   }
   ```

3. **消除白屏闪烁**
   ```rust
   // 窗口创建时设置为隐藏
   let mut window_builder = WindowBuilder::new()
       .with_visible(false);  // 避免白屏闪烁

   // 延迟 100ms 后显示窗口（确保加载动画已渲染）
   let show_time = std::time::Instant::now() + std::time::Duration::from_millis(100);

   event_loop.run(move |event, _, control_flow| {
       if !window_shown && std::time::Instant::now() >= show_time {
           window.set_visible(true);  // 此时加载动画已渲染完成
           window_shown = true;
       }
   });
   ```

4. **用户体验流程**
   - 0-0.6秒：创建 WebView（窗口隐藏）
   - 0.6-0.7秒：加载动画渲染（窗口仍隐藏）
   - 0.7秒：窗口显示，用户看到加载动画 ✨（无白屏！）
   - 0.7-5秒：后台加载真实页面（用户看到动画）
   - 5秒：页面加载完成，自动切换

## 最终效果对比

### 优化前
```
用户体验：白屏 → 白屏等待 8 秒 → 页面显示
启动时间：~9.5 秒
问题：卡死、慢、白屏体验差
```

### 优化后
```
用户体验：（窗口隐藏）→ 加载动画 ✨ → 页面显示
启动时间：~0.7 秒显示动画，~5 秒完全加载
优势：无卡死、快速、无白屏、优雅动画
```

### 时间线对比

**优化前：**
```
0s ────────────────────────────────────> 9.5s
   [创建] [白屏等待 8 秒 😞] [页面显示]
```

**优化后：**
```
0s ──────────────────────> 5s
   [创建] [加载动画 ✨] [页面显示]
   0.7s 显示动画（无白屏！）
```

## 优势

1. **统一实现**: Python CLI 和 Rust CLI 使用相同的底层实现
2. **更可靠**: 使用 `event_loop.run()` 避免了 `run_return()` 在 Windows 上的问题
3. **更快速**: 启动时间减少 50%，接近传统浏览器的体验
4. **无白屏**: 窗口延迟显示，确保加载动画已渲染
5. **更友好**: 优雅的加载动画替代白屏，用户体验大幅提升
6. **更清晰**: Standalone 和 Embedded 模式的区别更明确
7. **更易维护**: 只需维护一套实现

## 技术要点

1. **延迟显示窗口**: 使用 `.with_visible(false)` 创建隐藏窗口
2. **100ms 延迟**: 给 WebView 足够时间渲染加载动画
3. **Poll → Wait 切换**: 显示窗口后切换到 Wait 模式降低 CPU 使用
4. **两阶段加载**: 先加载动画，后台加载真实内容

## 使用场景

### Standalone 模式 (使用 `run_standalone`)

- ✅ 独立 Python 脚本
- ✅ CLI 应用
- ✅ 桌面应用
- ❌ DCC 插件 (会导致 DCC 退出)

### Embedded 模式 (使用 `WebView.show()`)

- ✅ Maya 插件
- ✅ Houdini 插件
- ✅ Blender 插件
- ✅ 任何 DCC 集成
- ❌ 独立脚本 (需要手动管理事件循环)

## 测试

```bash
# Python CLI (使用 run_standalone)
python -m auroraview --url https://example.com

# Rust CLI (使用 event_loop.run())
cargo run --bin auroraview -- --url https://example.com

# 两者现在使用相同的底层实现！
```

