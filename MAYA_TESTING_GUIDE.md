# Maya 测试指南 - 事件循环修复

## 🎯 关键问题

Maya 的 Python 环境可能缓存了旧的编译模块，导致即使 Rust 代码已经修复，Python 仍然使用旧版本。

## ✅ 解决方案

### 步骤 1：诊断模块加载

首先，运行诊断脚本来检查哪个版本的模块被加载：

1. 打开 Maya 2022
2. 打开脚本编辑器（Ctrl + Shift + E）
3. 切换到 Python 标签
4. 复制 `examples/diagnose_module_loading.py` 的全部内容
5. 粘贴到脚本编辑器
6. 执行（Ctrl + Enter）

**预期输出：**
```
# __main__ : ====================================================================== #
# __main__ : Diagnosing Module Loading #
# __main__ : ====================================================================== #
# __main__ : #
# __main__ : Current sys.path: #
# __main__ :   [0] C:\Users\hallo\Documents\augment-projects\dcc_webview\python #
# __main__ :   [1] C:\Users\hallo\Documents\augment-projects\dcc_webview\target\release #
# __main__ : #
# __main__ : ✓ Successfully imported auroraview._core #
# __main__ :   Location: C:\Users\hallo\Documents\augment-projects\dcc_webview\target\release\auroraview_core.pyd #
# __main__ : #
# __main__ : ✓ All imports successful! #
```

### 步骤 2：验证编译版本

检查 `auroraview_core.pyd` 的修改时间：

```bash
# 在 PowerShell 中运行
Get-Item "target/release/auroraview_core.pyd" | Select-Object LastWriteTime
```

**应该显示最近的时间戳**（在你运行 `cargo build --release` 之后）

### 步骤 3：测试事件循环修复

1. 打开 Maya 2022
2. 打开脚本编辑器（Ctrl + Shift + E）
3. 切换到 Python 标签
4. 复制 `examples/test_event_loop_fix.py` 的全部内容
5. 粘贴到脚本编辑器
6. 执行（Ctrl + Enter）

**预期结果：**
- ✓ WebView 窗口出现
- ✓ 显示 "Event Loop Fix Verified!" 消息
- ✓ **没有 PanicException 错误**
- ✓ Maya 保持响应

**关键日志：**
```
# __main__ : ✓ WebView started successfully! #
# __main__ : ✓ Maya is responsive! #
# __main__ : ✓ Test completed successfully! #
# __main__ : ✓ Event loop fix is working correctly! #
```

## 🔍 故障排除

### 问题 1：仍然看到 PanicException

**症状：**
```
pyo3_runtime.PanicException: Initializing the event loop outside of the main thread 
is a significant cross-platform compatibility hazard.
```

**解决方案：**

1. **确保代码已重新编译**
   ```bash
   cargo clean
   cargo build --release
   ```

2. **检查 .pyd 文件的修改时间**
   ```bash
   Get-Item "target/release/auroraview_core.pyd" | Select-Object LastWriteTime
   ```
   应该显示最近的时间戳

3. **关闭 Maya 并重新打开**
   - Maya 可能锁定了旧的 DLL 文件
   - 重新打开 Maya 会强制重新加载模块

4. **清理 Python 缓存**
   ```bash
   Get-ChildItem -Path "python/auroraview" -Filter "*.pyc" -Recurse | Remove-Item -Force
   Get-ChildItem -Path "python/auroraview" -Filter "__pycache__" -Recurse | Remove-Item -Force -Recurse
   ```

5. **运行诊断脚本**
   - 使用 `examples/diagnose_module_loading.py` 检查加载的模块位置
   - 确保加载的是 `target/release/auroraview_core.pyd`

### 问题 2：WebView 窗口不显示

**症状：**
- 脚本执行完成，但没有 WebView 窗口出现

**解决方案：**

1. 检查脚本编辑器的错误消息
2. 查看日志输出中是否有错误
3. 尝试使用 `examples/maya_quick_test.py` 而不是 `test_event_loop_fix.py`
4. 检查 Windows 任务栏中是否有隐藏的窗口

### 问题 3：Maya 冻结

**症状：**
- Maya 无响应，需要强制关闭

**解决方案：**

1. 这表示修复没有生效
2. 检查是否执行了 `cargo clean && cargo build --release`
3. 重启计算机以清除所有缓存
4. 检查 Rust 代码中是否正确使用了 `EventLoopBuilderExtWindows::with_any_thread(true)`

## 📊 验证修复

### 检查清单

- [ ] 运行 `cargo clean && cargo build --release`
- [ ] 验证 `target/release/auroraview_core.pyd` 的修改时间是最近的
- [ ] 运行 `uv run pytest tests/ -v` - 所有 45 个测试通过
- [ ] 在 Maya 中运行 `examples/diagnose_module_loading.py`
- [ ] 在 Maya 中运行 `examples/test_event_loop_fix.py`
- [ ] WebView 窗口出现，没有错误

### 成功的迹象

✓ WebView 窗口出现
✓ Maya 主线程保持响应
✓ 没有 PanicException 错误
✓ 脚本编辑器显示成功消息
✓ 可以与 WebView 交互

## 📝 完整测试流程

```bash
# 1. 重新编译
cargo clean
cargo build --release

# 2. 运行单元测试
uv run pytest tests/ -v

# 3. 在 Maya 中测试
# - 打开 Maya 2022
# - 运行 examples/diagnose_module_loading.py
# - 运行 examples/test_event_loop_fix.py
# - 验证 WebView 出现且没有错误
```

## 🎉 成功后

一旦验证修复有效，你可以：

1. **使用独立窗口模式**
   ```python
   from auroraview import WebView
   webview = WebView(title="My Tool")
   webview.load_html(html)
   webview.show_async()  # ✓ 现在可以工作了！
   ```

2. **创建生产工具**
   - 基于 `examples/maya_workspace_control.py` 创建
   - 添加自定义 HTML UI
   - 实现事件处理

## 📚 相关文件

- `examples/diagnose_module_loading.py` - 诊断脚本
- `examples/test_event_loop_fix.py` - 验证脚本
- `examples/maya_quick_test.py` - 快速测试
- `REBUILD_INSTRUCTIONS.md` - 重新编译说明
- `EVENT_LOOP_FIX.md` - 技术细节

---

**现在就开始吧！** 按照上面的步骤在 Maya 中测试修复。

