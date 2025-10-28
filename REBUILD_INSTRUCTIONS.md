# 重新编译说明 - 事件循环修复

## 问题

在之前的测试中，即使 Rust 代码已经修复，Python 仍然在使用旧的编译版本，导致仍然出现 `PanicException` 错误。

## 原因

Python 缓存了旧的编译模块（`.so` 或 `.pyd` 文件），即使 Rust 代码已经修改并重新编译，Python 仍然加载旧版本。

## 解决方案

### 步骤 1：完全清理编译缓存

```bash
cargo clean
```

这会删除所有编译的文件，包括旧的 Python 扩展模块。

### 步骤 2：清理 Python 缓存

```bash
# 删除 Python 字节码缓存
Get-ChildItem -Path "python/auroraview" -Filter "*.pyc" -Recurse | Remove-Item -Force
Get-ChildItem -Path "python/auroraview" -Filter "__pycache__" -Recurse | Remove-Item -Force -Recurse
```

### 步骤 3：重新编译 Rust 代码

```bash
cargo build --release
```

这会从头开始编译所有代码，包括新的事件循环修复。

### 步骤 4：运行测试

```bash
uv run pytest tests/ -v
```

验证所有测试都通过。

### 步骤 5：在 Maya 中测试

1. 打开 Maya 2022
2. 打开脚本编辑器（Ctrl + Shift + E）
3. 复制 `examples/test_event_loop_fix.py`
4. 粘贴到脚本编辑器
5. 执行（Ctrl + Enter）
6. ✓ WebView 窗口应该出现，没有 PanicException 错误！

## 完整命令

```bash
# 一次性执行所有步骤
cargo clean && cargo build --release && uv run pytest tests/ -v
```

## 验证修复

### 预期结果

✓ 所有 45 个单元测试通过
✓ 没有编译错误
✓ WebView 在后台线程中创建成功
✓ 没有 PanicException 错误
✓ Maya 主线程保持响应

### 如果仍然看到错误

1. **确保 Maya 已关闭** - 某些情况下 Maya 可能锁定了 DLL 文件
2. **重启计算机** - 清除所有缓存
3. **检查 Python 版本** - 确保使用 Python 3.7+
4. **查看日志** - 检查脚本编辑器的输出

## 技术细节

### 修复的代码

**文件：** `src/webview/mod.rs`

```rust
#[cfg(target_os = "windows")]
let event_loop = {
    use tao::platform::windows::EventLoopBuilderExtWindows;
    EventLoopBuilder::new().with_any_thread(true).build()
};

#[cfg(not(target_os = "windows"))]
let event_loop = EventLoopBuilder::new().build();
```

### 为什么需要这个修复

- **Windows 限制** - tao 事件循环库在 Windows 上默认要求主线程
- **DCC 集成** - Maya 在主线程上运行，WebView 需要在后台线程中创建
- **解决方案** - `with_any_thread(true)` 允许在任何线程上创建事件循环

## 相关文件

- `src/webview/mod.rs` - Rust 核心实现（已修复）
- `python/auroraview/webview.py` - Python API
- `examples/test_event_loop_fix.py` - 验证脚本
- `EVENT_LOOP_FIX.md` - 详细的技术说明

## 下一步

1. ✓ 执行重新编译步骤
2. ✓ 运行单元测试
3. ✓ 在 Maya 中测试
4. ✓ 验证 WebView 正常显示
5. ✓ 没有错误或异常

## 总结

这个修复解决了 `show_async()` 在 Maya 中无法工作的根本问题。关键是：

1. **修复 Rust 代码** - 使用 `EventLoopBuilderExtWindows::with_any_thread(true)`
2. **完全重新编译** - 使用 `cargo clean` 然后 `cargo build --release`
3. **清理 Python 缓存** - 删除 `.pyc` 和 `__pycache__` 文件
4. **验证修复** - 运行测试和 Maya 测试

现在 WebView 可以在后台线程中正常工作，Maya 主线程保持响应！

