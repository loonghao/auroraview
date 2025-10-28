# Maya 测试脚本 - AuroraView 非阻塞集成

本目录包含用于在 Maya 中测试 AuroraView 非阻塞 WebView 功能的脚本。

## 📁 文件说明

### 1. `maya_quick_test.py` ⭐ 推荐

**最简单的测试脚本 - 适合快速验证**

- 最小化的代码
- 快速启动
- 基本功能演示
- 适合初次测试

**功能：**
- 创建立方体
- 获取场景信息
- 实时 UI 更新

**使用时间：** 5 分钟

### 2. `maya_test_tool.py` 🎯 完整功能

**功能完整的测试工具 - 适合深入测试**

- 完整的 UI 设计
- 多种操作选项
- 详细的日志输出
- 生产级别的代码质量

**功能：**
- 场景信息显示
- 创建立方体和球体
- 删除选中对象
- 清空场景
- 实时状态更新

**使用时间：** 10-15 分钟

### 3. `maya_integration.py`

**原始集成示例**

- 展示基本集成方式
- 使用 `show_async()` 方法
- 适合学习参考

### 4. `maya_integration_async.py`

**异步集成示例**

- 专门为异步模式设计
- 详细的注释
- 最佳实践示例

## 🚀 快速开始

### 步骤 1：安装 AuroraView

```bash
pip install auroraview
```

### 步骤 2：打开 Maya

```bash
C:\Program Files\Autodesk\Maya2022\bin\maya.exe
```

### 步骤 3：打开脚本编辑器

- 菜单：`Windows > General Editors > Script Editor`
- 快捷键：`Ctrl + Shift + E`

### 步骤 4：选择 Python 标签

在脚本编辑器底部，确保选择了 **Python** 标签

### 步骤 5：复制脚本

从 `maya_quick_test.py` 复制全部代码到脚本编辑器

### 步骤 6：执行

点击 **Execute** 或按 `Ctrl + Enter`

### 步骤 7：验证

- ✓ WebView 窗口应该在 1-2 秒内出现
- ✓ Maya 应该保持响应
- ✓ 你可以在 WebView 中创建对象
- ✓ 对象应该出现在 Maya 中

## 📊 测试清单

### 基本功能测试

- [ ] WebView 窗口正常打开
- [ ] Maya 保持响应（可以移动视图、选择对象等）
- [ ] WebView 中的按钮可以点击
- [ ] 创建的对象出现在 Maya 中
- [ ] 场景信息正确显示

### 高级功能测试

- [ ] 快速连续点击多个按钮
- [ ] 同时在 Maya 和 WebView 中操作
- [ ] 创建多个对象
- [ ] 删除对象
- [ ] 清空场景
- [ ] 没有错误或崩溃

### 性能测试

- [ ] WebView 启动时间 < 2 秒
- [ ] 事件响应时间 < 100 ms
- [ ] Maya 响应延迟 < 50 ms
- [ ] 内存占用合理（< 200 MB）

## 🔍 预期输出

### 脚本编辑器输出

```
======================================================================
Starting WebView in background thread...
======================================================================

✓ WebView started!
✓ Maya is responsive!

The WebView window should appear shortly.
You can now:
  • Use the UI to create objects
  • Continue working in Maya
  • Close the window when done
```

### WebView 窗口

- 绿色标题栏
- 响应式 UI
- 实时状态更新
- 关闭按钮

## ⚠️ 常见问题

### Q: WebView 窗口不显示？

A: 
1. 检查脚本编辑器的错误消息
2. 确保 AuroraView 已安装：`pip list | grep auroraview`
3. 尝试重启 Maya
4. 检查是否有其他窗口被隐藏

### Q: Maya 仍然被冻结？

A:
1. 这表示 `show_async()` 没有工作
2. 检查 Python 版本（应该是 3.7+）
3. 查看脚本编辑器的错误消息

### Q: 导入错误？

A:
```bash
# 使用 Maya 的 Python 安装 AuroraView
C:\Program Files\Autodesk\Maya2022\bin\mayapy.exe -m pip install auroraview
```

### Q: 事件不工作？

A:
1. 检查脚本编辑器的日志
2. 确保事件处理器在 `show_async()` 前注册
3. 检查 JavaScript 中的事件名称

## 📚 详细文档

更多信息请查看：

- **完整测试指南：** `docs/MAYA_TESTING_GUIDE.md`
- **异步集成指南：** `docs/ASYNC_DCC_INTEGRATION.md`
- **API 参考：** `python/auroraview/webview.py`

## 🎯 测试目标

这些脚本的目标是验证：

1. ✓ **非阻塞性** - Maya 主线程不被冻结
2. ✓ **线程安全** - 后台线程正常工作
3. ✓ **事件通信** - WebView 和 Maya 可以通信
4. ✓ **稳定性** - 没有崩溃或错误
5. ✓ **性能** - 响应时间快

## 💡 提示

### 调试技巧

1. **查看日志**
   - 脚本编辑器会显示所有日志消息
   - 时间戳显示操作执行时间

2. **使用浏览器开发者工具**
   - 在 WebView 中按 F12 打开开发者工具
   - 查看 JavaScript 控制台

3. **检查 Maya 脚本编辑器**
   - 所有 Python 输出都会显示在这里
   - 错误消息会显示在这里

### 性能优化

- 第一次运行会比较慢（编译 Rust 代码）
- 后续运行会更快
- 关闭其他应用可以提高性能

## 🔗 相关资源

- **GitHub 仓库：** https://github.com/loonghao/auroraview
- **AuroraView 文档：** https://github.com/loonghao/auroraview#readme
- **Maya 文档：** https://help.autodesk.com/view/MAYA/2022/ENU/

## 📝 反馈

如果你发现任何问题或有改进建议，请：

1. 查看 `docs/MAYA_TESTING_GUIDE.md` 中的故障排除部分
2. 检查脚本编辑器的错误消息
3. 提交 Issue 到 GitHub

---

**祝你测试顺利！** 🎉

有任何问题，请参考详细的测试指南：`docs/MAYA_TESTING_GUIDE.md`

