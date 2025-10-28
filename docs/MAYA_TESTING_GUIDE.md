# Maya 测试指南 - AuroraView 非阻塞集成

本指南将帮助你在 Maya 2022 中测试 AuroraView 的非阻塞 WebView 功能。

## 前置条件

- **Maya 2022** 已安装（路径：`C:\Program Files\Autodesk\Maya2022\bin\maya.exe`）
- **Python 3.7+** 环境
- **AuroraView** 已安装

### 安装 AuroraView

```bash
pip install auroraview
```

## 快速测试（5 分钟）

### 方法 1：使用快速测试脚本

这是最简单的方式，适合快速验证功能。

#### 步骤 1：打开 Maya

```bash
C:\Program Files\Autodesk\Maya2022\bin\maya.exe
```

#### 步骤 2：打开脚本编辑器

在 Maya 中：
- 菜单：`Windows > General Editors > Script Editor`
- 或快捷键：`Ctrl + Shift + E`

#### 步骤 3：切换到 Python 标签

在脚本编辑器底部，点击 **Python** 标签（不是 MEL）

#### 步骤 4：复制并运行脚本

复制 `examples/maya_quick_test.py` 的全部内容到 Python 标签中。

点击 **Execute** 或按 `Ctrl + Enter`

#### 步骤 5：验证结果

你应该看到：

1. **Maya 脚本编辑器输出：**
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

2. **WebView 窗口出现**（通常在 1-2 秒内）

3. **关键测试点：**
   - ✓ Maya 窗口仍然可以点击和操作
   - ✓ 你可以在 Maya 中创建新对象
   - ✓ WebView 中的按钮可以点击
   - ✓ 创建的对象会出现在 Maya 中

### 方法 2：使用完整测试工具

这个版本有更多功能和详细的 UI。

#### 步骤 1-3：同上

#### 步骤 4：复制并运行脚本

复制 `examples/maya_test_tool.py` 的全部内容到 Python 标签中。

点击 **Execute** 或按 `Ctrl + Enter`

#### 步骤 5：使用完整功能

WebView 窗口会显示以下功能：

**📊 场景信息**
- 点击 "Refresh Scene Info" 获取当前场景信息
- 显示节点数、网格数、摄像机数、灯光数

**🎲 创建对象**
- 输入立方体大小，点击 "Create Cube"
- 输入球体半径，点击 "Create Sphere"
- 对象会立即在 Maya 中创建

**🗑️ 场景管理**
- "Delete Selected" - 删除选中的对象
- "Clear Scene" - 清空整个场景

## 详细测试步骤

### 测试 1：验证非阻塞行为

**目标：** 确认 Maya 主线程不被阻塞

**步骤：**

1. 运行快速测试脚本
2. WebView 窗口打开后，立即尝试：
   - 在 Maya 中移动视图（中键拖动）
   - 在 Maya 中选择对象
   - 在 Maya 中创建新对象（菜单或快捷键）
   - 打开其他 Maya 窗口

**预期结果：**
- ✓ 所有操作都能立即响应
- ✓ 没有卡顿或延迟
- ✓ Maya 完全正常工作

### 测试 2：验证事件通信

**目标：** 确认 WebView 和 Maya 之间的通信正常

**步骤：**

1. 运行完整测试工具
2. 点击 "Refresh Scene Info" 按钮
3. 观察场景信息是否更新
4. 在 WebView 中创建一个立方体
5. 检查 Maya 中是否出现了立方体

**预期结果：**
- ✓ 场景信息正确显示
- ✓ 创建的对象出现在 Maya 中
- ✓ 没有错误消息

### 测试 3：验证多个操作

**目标：** 确认可以执行多个连续操作

**步骤：**

1. 运行完整测试工具
2. 创建 3 个不同大小的立方体
3. 创建 2 个不同半径的球体
4. 点击 "Refresh Scene Info" 查看总节点数
5. 选择一些对象并点击 "Delete Selected"
6. 再次刷新场景信息

**预期结果：**
- ✓ 所有对象都被创建
- ✓ 场景信息正确更新
- ✓ 删除操作正常工作
- ✓ 没有崩溃或错误

### 测试 4：验证线程安全性

**目标：** 确认在后台线程中运行是安全的

**步骤：**

1. 运行完整测试工具
2. 快速连续点击多个按钮
3. 同时在 Maya 中进行操作
4. 观察是否有任何错误或不稳定

**预期结果：**
- ✓ 没有崩溃
- ✓ 没有错误消息
- ✓ 所有操作都被正确处理

## 故障排除

### 问题 1：WebView 窗口不显示

**症状：** 脚本运行但没有 WebView 窗口出现

**解决方案：**
1. 检查 Maya 脚本编辑器的输出是否有错误
2. 确保 AuroraView 已正确安装：`pip list | grep auroraview`
3. 尝试重启 Maya
4. 检查是否有其他窗口被隐藏

### 问题 2：Maya 仍然被冻结

**症状：** WebView 打开后 Maya 无法响应

**解决方案：**
1. 这表示 `show_async()` 没有正常工作
2. 检查 Python 版本：`python --version`（应该是 3.7+）
3. 检查 threading 模块是否可用
4. 查看脚本编辑器的错误消息

### 问题 3：事件不工作

**症状：** 点击按钮但 Maya 中没有反应

**解决方案：**
1. 检查脚本编辑器的输出日志
2. 确保事件处理器已注册（应该在 WebView 启动前）
3. 检查 JavaScript 中的事件名称是否匹配
4. 查看 Maya 脚本编辑器中的错误消息

### 问题 4：导入错误

**症状：** `ImportError: No module named 'auroraview'`

**解决方案：**
1. 在 Maya 的 Python 环境中安装 AuroraView：
   ```bash
   # 找到 Maya 的 Python 路径
   # 通常在 C:\Program Files\Autodesk\Maya2022\bin\mayapy.exe
   
   # 使用 mayapy 安装
   C:\Program Files\Autodesk\Maya2022\bin\mayapy.exe -m pip install auroraview
   ```

2. 或者在脚本中添加路径：
   ```python
   import sys
   sys.path.insert(0, 'C:\\path\\to\\auroraview')
   ```

## 性能指标

### 预期性能

- **WebView 启动时间：** < 2 秒
- **事件响应时间：** < 100 ms
- **Maya 响应延迟：** < 50 ms
- **内存占用：** ~ 50-100 MB

### 监控性能

在脚本编辑器中查看日志：
- 时间戳显示每个操作的执行时间
- 日志级别：INFO（正常）、WARNING（警告）、ERROR（错误）

## 下一步

### 集成到你的工具

一旦验证了功能正常，你可以：

1. **创建自定义工具**
   - 基于 `maya_test_tool.py` 修改
   - 添加你自己的功能

2. **创建 Maya 插件**
   - 将脚本包装成 Maya 插件
   - 添加到 Maya 菜单

3. **生产部署**
   - 添加错误处理
   - 添加配置选项
   - 添加日志记录

## 参考资源

- **AuroraView 文档：** `docs/ASYNC_DCC_INTEGRATION.md`
- **示例代码：** `examples/maya_integration_async.py`
- **API 参考：** `python/auroraview/webview.py`

## 支持

如果遇到问题：

1. 检查日志输出
2. 查看故障排除部分
3. 查看示例代码
4. 提交 Issue 到 GitHub

---

**祝你测试顺利！** 🎉

