# Maya WebView Integration - Testing Instructions

## 快速开始（5 分钟）

### 方式 1：独立窗口（最简单）

1. **打开 Maya 2022**
   ```bash
   C:\Program Files\Autodesk\Maya2022\bin\maya.exe
   ```

2. **打开脚本编辑器**
   - 菜单：`Windows > General Editors > Script Editor`
   - 快捷键：`Ctrl + Shift + E`

3. **切换到 Python 标签**
   - 在脚本编辑器底部点击 **Python** 标签

4. **复制脚本**
   - 打开文件：`examples/maya_quick_test.py`
   - 复制全部代码

5. **粘贴并执行**
   - 粘贴到脚本编辑器
   - 点击 **Execute** 或按 `Ctrl + Enter`

6. **验证**
   - ✓ WebView 窗口出现
   - ✓ Maya 保持响应
   - ✓ 可以创建立方体
   - ✓ 可以查看场景信息

---

## 推荐方式（15 分钟）

### 方式 2：嵌入式模式（完整集成）

1. **打开 Maya 2022**
   ```bash
   C:\Program Files\Autodesk\Maya2022\bin\maya.exe
   ```

2. **打开脚本编辑器**
   - 菜单：`Windows > General Editors > Script Editor`
   - 快捷键：`Ctrl + Shift + E`

3. **切换到 Python 标签**
   - 在脚本编辑器底部点击 **Python** 标签

4. **复制脚本**
   - 打开文件：`examples/maya_workspace_control.py`
   - 复制全部代码

5. **粘贴并执行**
   - 粘贴到脚本编辑器
   - 点击 **Execute** 或按 `Ctrl + Enter`

6. **验证**
   - ✓ WebView 作为可停靠面板出现
   - ✓ 可以停靠到 Maya 的任何位置
   - ✓ 可以创建立方体和球体
   - ✓ 可以删除选中对象
   - ✓ 可以查看场景信息
   - ✓ Maya 完全响应

---

## 详细测试清单

### 基本功能测试

- [ ] WebView 窗口/面板出现
- [ ] Maya 保持响应（可以移动视图）
- [ ] 可以选择 Maya 中的对象
- [ ] WebView 中的按钮可以点击
- [ ] 创建的对象出现在 Maya 中

### 创建对象测试

- [ ] 创建立方体（默认大小）
- [ ] 创建立方体（自定义大小）
- [ ] 创建球体（默认半径）
- [ ] 创建球体（自定义半径）
- [ ] 对象在 Maya 中可见

### 删除对象测试

- [ ] 在 Maya 中选择对象
- [ ] 点击 "Delete Selected" 按钮
- [ ] 对象从 Maya 中删除
- [ ] 状态消息显示删除数量

### 场景信息测试

- [ ] 点击 "Refresh Info" 按钮
- [ ] 显示正确的节点数
- [ ] 显示正确的网格数
- [ ] 显示正确的摄像机数
- [ ] 显示正确的灯光数

### 事件通信测试

- [ ] Python 可以发送事件到 WebView
- [ ] WebView 可以发送事件到 Python
- [ ] 状态消息实时更新
- [ ] 没有错误或异常

### 性能测试

- [ ] WebView 启动 < 2 秒
- [ ] 事件响应 < 100 ms
- [ ] Maya 响应 < 50 ms
- [ ] 没有内存泄漏
- [ ] 没有崩溃

---

## 故障排除

### 问题：WebView 不显示

**解决方案：**
1. 检查 Python 版本：`import sys; print(sys.version)`
2. 检查 AuroraView 安装：`import auroraview; print(auroraview.__file__)`
3. 查看脚本编辑器的错误消息
4. 尝试重启 Maya

### 问题：看到 PanicException 错误

**解决方案：**
1. 这已经修复了！更新 AuroraView：`pip install --upgrade auroraview`
2. 确保使用最新版本
3. 检查是否使用了 `create_embedded()` 而不是 `show()`

### 问题：Maya 仍然冻结

**解决方案：**
1. 确保使用 `show_async()` 或 `create_embedded()`
2. 不要使用 `show()`（这是阻塞的）
3. 检查脚本中是否有其他阻塞操作

### 问题：事件不工作

**解决方案：**
1. 检查事件名称是否匹配（Python 和 JavaScript）
2. 确保事件处理器在发送事件前注册
3. 查看脚本编辑器的日志输出
4. 使用 `logger.info()` 调试

---

## 三种测试方式对比

| 方式 | 文件 | 复杂度 | 集成度 | 推荐用途 |
|------|------|--------|--------|---------|
| 独立窗口 | `maya_quick_test.py` | ⭐ 简单 | ✗ 无 | 快速原型 |
| 嵌入式基础 | `maya_embedded_integration.py` | ⭐⭐ 中等 | ✓ 部分 | 基础集成 |
| 嵌入式完整 | `maya_workspace_control.py` | ⭐⭐⭐ 复杂 | ✓ 完全 | 生产工具 |

---

## 预期输出

### 脚本编辑器日志

```
auroraview.webview : Loading HTML (4790 bytes)
auroraview.webview : Showing WebView in background thread: AuroraView - Quick Test
auroraview.webview : WebView background thread started

✓ WebView started!
✓ Maya is responsive!

The WebView window should appear shortly.
```

### WebView 窗口

- 绿色标题栏
- 响应式 UI
- 实时更新
- 关闭按钮

### Maya 行为

- ✓ 完全响应
- ✓ 可以继续工作
- ✓ 对象正确创建
- ✓ 没有卡顿

---

## 下一步

1. ✓ 测试基本功能
2. ✓ 测试事件通信
3. ✓ 测试性能
4. ⏳ 自定义 HTML UI
5. ⏳ 添加更多事件处理
6. ⏳ 部署到生产环境

---

## 相关文档

- `MAYA_QUICK_START.md` - 快速开始指南
- `MAYA_INTEGRATION_SUMMARY.md` - 集成总结
- `docs/MAYA_EMBEDDED_INTEGRATION.md` - 嵌入式集成详细指南
- `docs/MAYA_TESTING_GUIDE.md` - 完整测试指南

---

## 获取帮助

1. 查看脚本编辑器的错误消息
2. 检查相关文档
3. 查看示例代码
4. 提交 Issue 到 GitHub

---

**准备好了吗？** 现在就试试吧！

**推荐开始：** 复制 `examples/maya_workspace_control.py` 到 Maya 脚本编辑器并执行！

