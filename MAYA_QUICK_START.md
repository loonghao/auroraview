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

```bash
# 使用 Maya 的 Python 安装
C:\Program Files\Autodesk\Maya2022\bin\mayapy.exe -m pip install auroraview
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

