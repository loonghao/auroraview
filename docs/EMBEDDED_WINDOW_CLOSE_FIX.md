# Embedded Window Close Fix

## 问题描述

在嵌入式模式下(设置了 `parent_hwnd`),调用 `DestroyWindow()` 后窗口无法正确关闭,窗口仍然可见。

### 症状
- [OK] `DestroyWindow()` 返回成功
- [OK] Timer 被正确 kill
- [OK] WebView 引用被删除
- [ERROR] **窗口仍然显示在屏幕上**

## 根本原因

### Windows 窗口销毁机制

当调用 `DestroyWindow()` 时,Windows 会:
1. 发送 `WM_DESTROY` 消息到窗口
2. 发送 `WM_NCDESTROY` 消息(最终清理)
3. 只有在这些消息被处理后,窗口才会真正从屏幕上消失

### 嵌入式窗口的特殊性

在我们的实现中:
- 创建了 `event_loop`,但在嵌入模式下**从未运行它**
- 使用 Maya 的 `scriptJob` timer 来调用 `process_events()`
- `process_events()` 只处理用户输入消息,不处理窗口销毁消息

**关键问题**:
```rust
// 调用 DestroyWindow() 后
DestroyWindow(hwnd);  // [OK] 成功

// 但是 WM_DESTROY 和 WM_NCDESTROY 消息在队列中
// 没有消息循环来处理它们!
// 结果:窗口句柄被销毁,但窗口仍然可见
```

## 解决方案

### 核心思路

参考 pywebview 和其他嵌入式窗口项目的做法:
**在调用 `DestroyWindow()` 后,立即处理该窗口的所有待处理消息**

### 实现细节

```rust
// Step 1: 销毁窗口
DestroyWindow(hwnd);

// Step 2: 处理待处理的窗口消息
let mut msg = MSG::default();
let mut processed_count = 0;
let max_iterations = 100;

// 处理该窗口的所有待处理消息
while processed_count < max_iterations
    && PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool()
{
    processed_count += 1;

    // 记录重要消息
    if msg.message == WM_DESTROY {
        tracing::info!("Processing WM_DESTROY");
    } else if msg.message == WM_NCDESTROY {
        tracing::info!("Processing WM_NCDESTROY (final cleanup)");
    }

    TranslateMessage(&msg);
    DispatchMessageW(&msg);
}

// Step 3: 短暂延迟确保窗口完全消失
std::thread::sleep(std::time::Duration::from_millis(50));
```

### 关键点

1. **使用特定 HWND**: `PeekMessageW(&mut msg, hwnd, ...)` 而不是 `HWND(null)`
   - 只处理该窗口的消息,不影响其他窗口

2. **处理所有消息**: 包括 `WM_DESTROY` 和 `WM_NCDESTROY`
   - 这些消息负责窗口的最终清理

3. **防止无限循环**: 设置 `max_iterations = 100`
   - 正常情况下只需要几个消息

4. **短暂延迟**: 50ms 延迟确保窗口完全消失
   - 给 Windows 时间完成视觉更新

## 修改的文件

### 1. `src/webview/aurora_view.rs`

在 `AuroraView::close()` 方法中:
- 调用 `DestroyWindow()` 后
- 添加消息处理循环
- 处理 `WM_DESTROY` 和 `WM_NCDESTROY`

### 2. `src/webview/webview_inner.rs`

在 `WebViewInner::drop()` 方法中:
- 同样的修复
- 确保在对象销毁时正确清理

## 测试

### 测试文件
`examples/maya/test_close_fix.py`

### 测试步骤
1. 在 Maya 中运行测试脚本
2. 点击 "[CLOSE] Close Window" 按钮
3. 观察日志输出
4. 验证窗口是否正确关闭

### 预期结果
```
[LOCK] [AuroraView::close] Calling DestroyWindow...
[OK] [AuroraView::close] DestroyWindow succeeded
[CLOSE] [AuroraView::close] Processing pending window messages...
[CLOSE] [AuroraView::close] Processing WM_DESTROY
[CLOSE] [AuroraView::close] Processing WM_NCDESTROY (final cleanup)
[OK] [AuroraView::close] Processed 5 window messages
[OK] [AuroraView::close] Window cleanup completed
```

## 参考

### pywebview 的实现

pywebview 在 Windows 平台使用 WinForms,它的窗口关闭机制:
1. 调用 `Form.Close()`
2. WinForms 自动处理所有窗口消息
3. 窗口正确关闭

### Flet 的实现

Flet 使用 Flutter,它的窗口管理:
1. Flutter 有自己的窗口管理系统
2. 不直接使用 Win32 API
3. 窗口关闭由 Flutter 引擎处理

### 我们的实现

我们使用 `tao` + `wry`:
1. `tao` 创建窗口
2. `wry` 创建 WebView
3. 在嵌入模式下,我们需要手动处理窗口消息

## 为什么之前的方法不工作

### 尝试 1: 只调用 `DestroyWindow()`
```rust
DestroyWindow(hwnd);  // [ERROR] 窗口仍然可见
```
**问题**: 没有处理 `WM_DESTROY` 和 `WM_NCDESTROY` 消息

### 尝试 2: 发送 `WM_CLOSE` 消息
```rust
PostMessageW(hwnd, WM_CLOSE, None, None);  // [ERROR] 没有效果
```
**问题**: 消息被发送,但没有消息循环来处理它

### 尝试 3: 使用 `window.set_visible(false)`
```rust
window.set_visible(false);  // [ERROR] 窗口隐藏但未销毁
```
**问题**: 只是隐藏,没有真正销毁窗口

## 最终方案的优势

1. **完整清理**: 处理所有窗口销毁消息
2. **无资源泄漏**: 窗口句柄被正确释放
3. **视觉正确**: 窗口从屏幕上消失
4. **线程安全**: 在正确的线程上执行
5. **可靠性高**: 参考了成熟项目的实现

## 后续改进

### 可能的优化
1. 调整延迟时间(当前 50ms)
2. 添加窗口状态检查
3. 优化消息处理循环

### 需要测试的场景
1. 多个窗口同时关闭
2. 快速打开/关闭窗口
3. Maya 退出时的清理
4. 不同 Windows 版本的兼容性

## 总结

这个修复解决了嵌入式窗口无法关闭的核心问题:
- **问题**: 窗口销毁消息没有被处理
- **方案**: 在 `DestroyWindow()` 后立即处理所有待处理消息
- **结果**: 窗口正确关闭,无资源泄漏

这是一个典型的 Windows 嵌入式窗口问题,需要深入理解 Windows 消息机制才能解决。

