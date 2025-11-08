# Spy++ 使用指南 - 调试窗口关闭问题

## 什么是 Spy++？

**Spy++** (spyxx.exe) 是 Visual Studio 自带的 Windows 消息监控工具，可以：
- 查看所有窗口的层次结构
- 监控窗口消息（WM_CLOSE, WM_DESTROY 等）
- 检查窗口属性（HWND, 样式, 父窗口等）
- 验证窗口是否真正被销毁

## 安装位置

Spy++ 通常位于 Visual Studio 安装目录：

```
# Visual Studio 2022
C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\spyxx_amd64.exe

# Visual Studio 2019
C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\Common7\Tools\spyxx_amd64.exe

# 32位版本
spyxx.exe

# 64位版本（推荐）
spyxx_amd64.exe
```

**提示**: 使用 64 位版本 (spyxx_amd64.exe) 来监控 64 位应用程序（如 Maya）。

## 快速启动

### 方法 1: 从开始菜单

1. 按 `Win` 键
2. 搜索 "Spy++"
3. 选择 "Spy++ (x64)" 或 "Spy++"

### 方法 2: 从命令行

```cmd
# 添加到 PATH 或直接运行
"C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\spyxx_amd64.exe"
```

### 方法 3: 创建快捷方式

右键桌面 → 新建快捷方式 → 粘贴上面的路径

---

## 使用场景 1: 查找窗口 HWND

### 步骤

1. **启动 Spy++**
2. **打开 Window Finder**:
   - 菜单: `Search` → `Find Window...`
   - 快捷键: `Ctrl+F`

3. **拖动靶心图标**:
   - 看到 "Finder Tool" 窗口
   - 拖动靶心图标到目标窗口上
   - 释放鼠标

4. **查看窗口信息**:
   ```
   Handle:    0x00120ABC  <- 这是 HWND
   Caption:   Maya Outliner
   Class:     Qt5QWindowIcon
   Style:     96000000 (WS_VISIBLE | WS_CHILD | ...)
   ```

### 验证窗口是否存在

- 如果窗口在列表中 → 窗口仍然存在（未销毁）
- 如果窗口不在列表中 → 窗口已被销毁

---

## 使用场景 2: 监控窗口消息

### 步骤

1. **找到目标窗口**（使用 Window Finder）

2. **启动消息监控**:
   - 右键窗口 → `Messages...`
   - 或: 选中窗口 → 菜单 `Spy` → `Log Messages`

3. **配置消息过滤器**:
   ```
   [Messages] 标签页:
   
   勾选以下消息:
   ☑ WM_CLOSE       (0x0010)  <- 关闭按钮点击
   ☑ WM_DESTROY     (0x0002)  <- 窗口销毁
   ☑ WM_NCDESTROY   (0x0082)  <- 非客户区销毁
   ☑ WM_QUIT        (0x0012)  <- 退出消息
   ☑ WM_SYSCOMMAND  (0x0112)  <- 系统命令（SC_CLOSE）
   
   可选（调试用）:
   ☑ WM_NCLBUTTONDOWN (0x00A1)  <- 非客户区鼠标按下
   ☑ WM_NCLBUTTONUP   (0x00A2)  <- 非客户区鼠标释放
   ```

4. **点击 OK 开始监控**

5. **测试关闭按钮**:
   - 点击窗口的 X 按钮
   - 观察 Spy++ 消息日志

### 预期消息序列（修复后）

```
<00001> 00120ABC P WM_SYSCOMMAND nID:SC_CLOSE ...
<00002> 00120ABC S WM_CLOSE
<00003> 00120ABC S WM_DESTROY
<00004> 00120ABC S WM_NCDESTROY
```

**说明**:
- `P` = Posted message (异步)
- `S` = Sent message (同步)
- `R` = Returned from message

### 问题诊断

**如果只看到 WM_CLOSE 但没有 WM_DESTROY**:
```
<00001> 00120ABC S WM_CLOSE
# 没有后续消息 <- 问题！窗口未被销毁
```
→ 说明 `DestroyWindow()` 没有被调用

**如果看到完整序列**:
```
<00001> 00120ABC S WM_CLOSE
<00002> 00120ABC S WM_DESTROY
<00003> 00120ABC S WM_NCDESTROY
```
→ 说明窗口正确销毁 ✓

---

## 使用场景 3: 检查窗口层次结构

### 步骤

1. **打开 Windows 视图**:
   - 菜单: `Spy` → `Windows`
   - 快捷键: `Ctrl+W`

2. **查找目标窗口**:
   - 使用 Window Finder 定位
   - 或手动展开树形结构

3. **检查父子关系**:
   ```
   Desktop
   └─ Maya 2024 (HWND: 0x00010001)
      └─ Qt5QWindowIcon (HWND: 0x00020002)  <- Maya 主窗口
         └─ Qt5QWindowIcon (HWND: 0x00120ABC)  <- 我们的 webview
   ```

### 验证嵌入模式

**正确的嵌入窗口**:
- Style 包含 `WS_CHILD` (0x40000000)
- 有父窗口（Parent HWND 不为 NULL）

**独立窗口**:
- Style 包含 `WS_OVERLAPPEDWINDOW`
- 父窗口为 Desktop

---

## 使用场景 4: 检查窗口属性

### 步骤

1. **找到目标窗口**

2. **打开属性窗口**:
   - 右键窗口 → `Properties...`
   - 或: 选中窗口 → 菜单 `View` → `Properties`

3. **查看关键属性**:

#### General 标签页
```
Window Handle:  0x00120ABC
Caption:        Maya Outliner
Class Name:     Qt5QWindowIcon
```

#### Styles 标签页
```
Window Styles:
☑ WS_VISIBLE     (0x10000000)  <- 窗口可见
☑ WS_CHILD       (0x40000000)  <- 子窗口
☐ WS_DISABLED    (0x08000000)  <- 未禁用

Extended Styles:
☑ WS_EX_NOPARENTNOTIFY (0x00000004)
```

#### Windows 标签页
```
Parent Window:   0x00020002  <- 父窗口 HWND
Owner Window:    (none)
Next Window:     0x00030003
Previous Window: 0x00040004
```

---

## 实战示例: 调试窗口关闭问题

### 测试流程

1. **启动 Spy++** (spyxx_amd64.exe)

2. **启动 Maya 和 Outliner**:
   ```python
   # 在 Maya Script Editor 中
   exec(open(r"C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\maya-outliner\launch_simple.py", encoding='utf-8').read())
   ```

3. **找到 Outliner 窗口**:
   - `Ctrl+F` 打开 Window Finder
   - 拖动靶心到 Outliner 窗口
   - 记录 HWND (例如: 0x00120ABC)

4. **启动消息监控**:
   - 右键窗口 → `Messages...`
   - 勾选: WM_CLOSE, WM_DESTROY, WM_SYSCOMMAND
   - 点击 OK

5. **测试关闭按钮**:
   - 点击 Outliner 的 X 按钮
   - 观察 Spy++ 消息日志

6. **验证结果**:

   **修复前（问题）**:
   ```
   <00001> 00120ABC S WM_CLOSE
   # 没有 WM_DESTROY <- 窗口未销毁
   ```

   **修复后（正确）**:
   ```
   <00001> 00120ABC S WM_CLOSE
   <00002> 00120ABC S WM_DESTROY      <- 窗口被销毁
   <00003> 00120ABC S WM_NCDESTROY
   ```

7. **验证窗口已销毁**:
   - 刷新 Windows 视图 (`F5`)
   - 搜索之前的 HWND
   - 应该找不到（窗口已销毁）

---

## 常见问题

### Q1: Spy++ 找不到我的窗口

**原因**:
- 使用了 32 位 Spy++ 监控 64 位程序
- 窗口已经被销毁

**解决**:
- 使用 `spyxx_amd64.exe` (64 位版本)
- 确认窗口仍然存在

### Q2: 消息太多，看不清

**解决**:
- 使用消息过滤器，只勾选需要的消息
- 清空日志: 右键消息窗口 → `Clear`
- 暂停捕获: 菜单 `Messages` → `Stop Logging`

### Q3: 如何保存消息日志？

**步骤**:
1. 消息窗口 → 菜单 `File` → `Save As...`
2. 选择保存位置
3. 文件格式: `.log` 或 `.txt`

### Q4: 如何对比修复前后的差异？

**步骤**:
1. 修复前: 保存消息日志为 `before.log`
2. 修复后: 保存消息日志为 `after.log`
3. 使用文本对比工具（如 WinMerge, Beyond Compare）对比

---

## 快捷键参考

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+F` | Find Window (查找窗口) |
| `Ctrl+W` | Windows View (窗口视图) |
| `F5` | Refresh (刷新) |
| `Ctrl+C` | Copy (复制选中内容) |
| `Ctrl+A` | Select All (全选) |
| `Delete` | Clear (清空日志) |

---

## 总结

**Spy++ 的三大用途**:

1. **查找窗口** - 获取 HWND 和窗口属性
2. **监控消息** - 验证 WM_CLOSE → WM_DESTROY 序列
3. **检查层次** - 确认父子窗口关系

**调试窗口关闭问题的关键**:
- 监控 WM_CLOSE 和 WM_DESTROY 消息
- 验证消息序列完整性
- 确认窗口从窗口列表中消失

---

**相关文档**:
- [WINDOW_CLOSE_ROOT_CAUSE_ANALYSIS.md](./WINDOW_CLOSE_ROOT_CAUSE_ANALYSIS.md)
- [WINDOW_CLOSE_FIX_SUMMARY.md](./WINDOW_CLOSE_FIX_SUMMARY.md)

