# 🎨 Maya 快速开始 - AuroraView 非阻塞集成

## ⚡ 5 分钟快速测试

### 1️⃣ 安装

```bash
pip install auroraview
```

### 2️⃣ 打开 Maya

```bash
C:\Program Files\Autodesk\Maya2022\bin\maya.exe
```

### 3️⃣ 打开脚本编辑器

- 菜单：`Windows > General Editors > Script Editor`
- 快捷键：`Ctrl + Shift + E`

### 4️⃣ 选择 Python 标签

在脚本编辑器底部点击 **Python** 标签

### 5️⃣ 复制脚本

从 `examples/maya_quick_test.py` 复制全部代码

### 6️⃣ 执行

点击 **Execute** 或按 `Ctrl + Enter`

### 7️⃣ 验证

✓ WebView 窗口出现（1-2 秒内）
✓ Maya 保持响应
✓ 可以创建对象
✓ 对象出现在 Maya 中

---

## 📋 测试清单

### 基本测试
- [ ] WebView 窗口打开
- [ ] Maya 可以响应（移动视图、选择对象）
- [ ] WebView 按钮可以点击
- [ ] 创建的对象出现在 Maya 中

### 功能测试
- [ ] 创建立方体
- [ ] 创建球体
- [ ] 获取场景信息
- [ ] 删除对象
- [ ] 清空场景

### 性能测试
- [ ] WebView 启动 < 2 秒
- [ ] 事件响应 < 100 ms
- [ ] Maya 响应 < 50 ms
- [ ] 没有崩溃

---

## 🎯 两个测试脚本

### 快速测试（推荐）
📄 `examples/maya_quick_test.py`
- ⏱️ 5 分钟
- 🎯 基本功能
- 📝 最小代码

**复制这个脚本到 Maya 脚本编辑器执行**

### 完整测试
📄 `examples/maya_test_tool.py`
- ⏱️ 10-15 分钟
- 🎯 完整功能
- 📊 详细 UI

**用于深入测试所有功能**

---

## 🔧 常见问题

### ❌ WebView 不显示？

**解决方案：**
```bash
# 使用 Maya 的 Python 安装
C:\Program Files\Autodesk\Maya2022\bin\mayapy.exe -m pip install auroraview
```

### ❌ 看到 PanicException 错误？

**这已经修复了！** 之前的版本有线程安全问题。最新版本：
- ✓ 在主线程中创建 WebView
- ✓ 在后台线程中创建新实例
- ✓ 没有 "unsendable" 错误

**如果仍然看到错误，请更新：**
```bash
pip install --upgrade auroraview
```

### ❌ Maya 仍然冻结？

检查 Python 版本：
```python
import sys
print(sys.version)  # 应该是 3.7+
```

### ❌ 导入错误？

在脚本编辑器中运行：
```python
import auroraview
print(auroraview.__file__)
```

---

## 📚 详细文档

- **完整测试指南：** `docs/MAYA_TESTING_GUIDE.md`
- **异步集成指南：** `docs/ASYNC_DCC_INTEGRATION.md`
- **示例说明：** `examples/README_MAYA_TESTING.md`

---

## 🔧 技术细节 - 线程安全修复

### 问题（已解决）
之前的实现尝试在后台线程中发送 Rust 的 `AuroraView` 对象，但 Rust 核心被标记为 `unsendable`，导致：
```
PanicException: auroraview_core::webview::AuroraView is unsendable, but sent to another thread
```

### 解决方案
现在的实现：
1. **主线程** - 创建 WebView，加载 HTML/URL，保存内容
2. **后台线程** - 创建新的 WebView 实例，加载保存的内容，运行事件循环
3. **结果** - Maya 主线程完全不被阻塞 ✓

### 代码示例
```python
# 主线程
webview = WebView(title="My Tool")
webview.load_html(html)  # 内容被保存
webview.show_async()     # 立即返回

# 后台线程（自动处理）
# - 创建新的 WebView 实例
# - 加载保存的 HTML
# - 运行事件循环
# - Maya 保持响应
```

---

## 🎉 预期结果

### 脚本输出
```
======================================================================
Starting WebView in background thread...
======================================================================

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

## 🚀 核心功能

### show_async() - 非阻塞启动

```python
from auroraview import WebView

webview = WebView(title="My Tool")
webview.load_html(html)
webview.show_async()  # ✓ 立即返回，Maya 保持响应
```

### wait() - 等待关闭

```python
webview.wait()  # 等待用户关闭窗口
```

### 事件通信

```python
@webview.on("my_event")
def handle_event(data):
    print(f"Received: {data}")

webview.emit("response", {"status": "ok"})
```

---

## 📊 性能指标

| 指标 | 预期值 |
|------|--------|
| WebView 启动 | < 2 秒 |
| 事件响应 | < 100 ms |
| Maya 响应 | < 50 ms |
| 内存占用 | 50-100 MB |

---

## ✨ 关键特性

✓ **非阻塞** - Maya 主线程不被冻结
✓ **线程安全** - 后台线程正常工作
✓ **事件驱动** - WebView 和 Maya 可以通信
✓ **稳定可靠** - 没有崩溃或错误
✓ **高性能** - 快速响应

---

## 🔗 链接

- **GitHub：** https://github.com/loonghao/auroraview
- **PR #4：** https://github.com/loonghao/auroraview/pull/4

---

## 📞 需要帮助？

1. 查看 `docs/MAYA_TESTING_GUIDE.md` 中的故障排除
2. 检查脚本编辑器的错误消息
3. 查看示例代码
4. 提交 Issue 到 GitHub

---

**祝你测试顺利！** 🎉

**下一步：** 复制 `examples/maya_quick_test.py` 到 Maya 脚本编辑器并执行！

