# 快速测试指南 - 窗口关闭按钮修复

## 测试前准备

### 1. 构建项目

```bash
cd C:\Users\hallo\Documents\augment-projects\dcc_webview
cargo build --release
```

### 2. 启动开发服务器

```bash
cd examples\maya-outliner
npm run dev
```

等待看到: `Local: http://localhost:5173/`

---

## 在 Maya 中测试

### 方法 1: 使用测试脚本（推荐）

在 Maya Script Editor 中执行:

```python
import codecs
with codecs.open(r"C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\maya-outliner\test_in_maya.py", 'r', 'utf-8') as f:
    exec(f.read())
```

### 方法 2: 手动导入

```python
import sys
sys.path.insert(0, r"C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\maya-outliner")

from maya_integration import maya_outliner_v2

outliner = maya_outliner_v2.main(
    url="http://localhost:5173",
    singleton=True
)
```

---

## 测试步骤

### 测试 1: 基本关闭功能

1. **点击窗口的 X 按钮**
2. **预期结果**:
   - 窗口立即消失（<100ms）
   - 控制台输出:
     ```
     [WindowsWindowManager] Close message detected: 0x0010
     [WindowsWindowManager] Window destroyed successfully (WM_CLOSE)
     [MayaOutlinerV2] Close signal detected from lifecycle manager
     ```

3. **失败标志**:
   - 窗口冻结不消失
   - 控制台显示 "DestroyWindow failed"
   - 没有 "Window destroyed successfully" 消息

### 测试 2: 程序化关闭

```python
outliner.close()
```

**预期**: 窗口正常关闭，无错误

### 测试 3: 多次开关

```python
# 循环测试
for i in range(3):
    print("\n[TEST] Round %d" % (i+1))
    outliner = maya_outliner_v2.main()
    # 点击 X 按钮关闭
    # 等待 1 秒
```

**预期**: 每次都能正常打开和关闭

### 测试 4: 生命周期状态

```python
# 打开窗口
outliner = maya_outliner_v2.main()

# 检查状态
state = outliner._webview._core.get_lifecycle_state()
print("State: %s" % state)  # 应该是 "Active"

# 点击 X 按钮后
# 状态应该变为 "CloseRequested" 或 "Destroyed"
```

---

## 使用 Spy++ 验证

### 快速步骤

1. **启动 Spy++**:
   ```
   C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\spyxx_amd64.exe
   ```

2. **找到窗口**:
   - 按 `Ctrl+F`
   - 拖动靶心图标到 Outliner 窗口
   - 记录 HWND (例如: 0x00120ABC)

3. **监控消息**:
   - 右键窗口 → `Messages...`
   - 勾选:
     - WM_CLOSE (0x0010)
     - WM_DESTROY (0x0002)
     - WM_NCDESTROY (0x0082)
     - WM_SYSCOMMAND (0x0112)
   - 点击 OK

4. **测试关闭**:
   - 点击 X 按钮
   - 观察消息序列

5. **验证结果**:

   **正确的消息序列**:
   ```
   <00001> 00120ABC S WM_CLOSE
   <00002> 00120ABC S WM_DESTROY
   <00003> 00120ABC S WM_NCDESTROY
   ```

   **错误的消息序列**（修复前）:
   ```
   <00001> 00120ABC S WM_CLOSE
   # 没有后续消息 <- 问题！
   ```

---

## 预期控制台输出

### 成功的关闭序列

```
[INFO] Launching Maya Outliner V2...
[SUCCESS] Outliner launched!
[OK] Lifecycle management is ACTIVE

# 用户点击 X 按钮

[WindowsWindowManager] Close message detected: 0x0010
[WindowsWindowManager] Window destroyed successfully (WM_CLOSE)
[MayaOutlinerV2] Close signal detected from lifecycle manager
[MayaOutlinerV2] Invoking cleanup...
[MayaOutliner] Closing WebView...
[MayaOutliner] EventTimer stopped and cleaned up
[MayaOutliner] Maya callbacks removed
[MayaOutliner] WebView cleanup complete
```

### 失败的输出（不应该看到）

```
[ERROR] DestroyWindow failed
[WARNING] Window still valid after close
[ERROR] Access violation
[ERROR] Timeout waiting for close
```

---

## 故障排除

### 问题 1: 窗口不关闭

**症状**: 点击 X 按钮，窗口冻结

**检查**:
1. 查看控制台是否有 "Window destroyed successfully"
2. 使用 Spy++ 检查是否收到 WM_DESTROY
3. 确认使用的是最新构建: `cargo build --release`

**解决**:
```bash
# 重新构建
cargo clean
cargo build --release

# 重启 Maya
```

### 问题 2: GBK 编码错误

**症状**: `UnicodeDecodeError: 'gbk' codec can't decode...`

**解决**: 使用 codecs 打开文件:
```python
import codecs
with codecs.open(r"path\to\script.py", 'r', 'utf-8') as f:
    exec(f.read())
```

### 问题 3: 导入失败

**症状**: `ImportError: No module named 'auroraview'`

**检查 PYTHONPATH**:
```python
import sys
print(sys.path)

# 应该包含:
# C:\Users\hallo\Documents\augment-projects\dcc_webview\python
```

**解决**:
```python
import sys
sys.path.insert(0, r"C:\Users\hallo\Documents\augment-projects\dcc_webview\python")
```

### 问题 4: Dev server 未运行

**症状**: 窗口显示 "Failed to load"

**解决**:
```bash
cd examples\maya-outliner
npm install  # 首次运行
npm run dev
```

---

## 验证清单

测试完成后，确认:

- [ ] 点击 X 按钮，窗口立即消失
- [ ] 控制台显示 "Window destroyed successfully"
- [ ] Spy++ 显示完整的 WM_CLOSE → WM_DESTROY 序列
- [ ] 无错误消息
- [ ] Maya 保持稳定
- [ ] 可以重新打开窗口
- [ ] 程序化 `outliner.close()` 也能工作
- [ ] 生命周期状态正确转换

---

## 相关文档

- [SPY_PLUS_PLUS_GUIDE.md](./SPY_PLUS_PLUS_GUIDE.md) - Spy++ 详细使用指南
- [WINDOW_CLOSE_FIX_SUMMARY.md](./WINDOW_CLOSE_FIX_SUMMARY.md) - 修复总结
- [WINDOW_CLOSE_ROOT_CAUSE_ANALYSIS.md](./WINDOW_CLOSE_ROOT_CAUSE_ANALYSIS.md) - 根本原因分析

---

## 快速命令参考

```python
# 启动
import codecs
with codecs.open(r"C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\maya-outliner\test_in_maya.py", 'r', 'utf-8') as f:
    exec(f.read())

# 关闭
outliner.close()

# 检查状态
outliner._webview._core.get_lifecycle_state()

# 重新打开
maya_outliner_v2.main()
```

---

**测试愉快！**

