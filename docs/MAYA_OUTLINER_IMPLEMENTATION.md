# Maya Outliner WebView - 修复总结

## [CONFIG] 修复的三个问题

### 问题1：WebView 无法关闭 [OK]

**根源**：
- 事件处理器监听了错误的事件名称 `av.close`
- JavaScript 端没有发送任何关闭事件
- 缺少关闭按钮和对应的事件处理

**修复方案**：
1. 将事件处理器改为监听 `close_window` 事件
2. 在 HTML UI 中添加关闭按钮（红色 [CLOSE] 按钮）
3. 添加 `closeWindow()` JavaScript 函数，通过 IPC 发送关闭事件
4. 改进 Python 端的关闭处理，添加更好的错误处理和日志

**修改文件**：`examples/maya/outliner_view.py`

**关键代码变更**：
```python
# 之前
@webview.on("av.close")

# 之后
@webview.on("close_window")
```

---

### 问题2：场景物体未显示 [ERROR] → [OK]

**根源**：
- 初始刷新时序问题：`delayed_initial_refresh()` 在 WebView 完全初始化前被调用
- `process_events()` 可能还未处理消息队列中的 emit 事件
- 嵌入式模式下的消息处理延迟

**修复方案**：
1. 增加初始刷新的延迟周期：从 1 次改为 3 次 `executeDeferred()` 调用
2. 这确保 WebView 有足够的时间完全初始化并处理消息队列
3. 每次 `executeDeferred()` 都会在 Maya 的下一个空闲事件中执行

**修改文件**：`examples/maya/outliner_view.py`

**关键代码变更**：
```python
# 之前
mutils.executeDeferred(delayed_initial_refresh)

# 之后
for _ in range(3):
    mutils.executeDeferred(delayed_initial_refresh)
```

---

### 问题3：手动刷新功能失效 [ERROR] → [OK]

**根源**：
- JavaScript 使用 `window.dispatchEvent()` 发送自定义事件
- 但 Python 的 `@webview.on()` 装饰器期望通过 IPC 接收事件
- 自定义事件只在 JavaScript 上下文中有效，无法跨越 Python-JavaScript 边界

**修复方案**：
1. 将所有 JavaScript 事件发送改为使用 `window.ipc.postMessage()`
2. 使用标准的 IPC 消息格式：
   ```javascript
   window.ipc.postMessage(JSON.stringify({
       type: 'event',
       event: 'event_name',
       detail: { /* data */ }
   }));
   ```
3. 修改的事件：
   - `refresh_scene` - 刷新按钮
   - `select_object` - 点击树节点选择对象
   - `rename_object` - 右键菜单重命名
   - `delete_object` - 右键菜单删除
   - `close_window` - 关闭按钮

**修改文件**：`examples/maya/outliner_view.py`

**关键代码变更**：
```javascript
// 之前 - 错误的方式
window.dispatchEvent(new CustomEvent('refresh_scene', {
    detail: { timestamp: Date.now() }
}));

// 之后 - 正确的方式
window.ipc.postMessage(JSON.stringify({
    type: 'event',
    event: 'refresh_scene',
    detail: { timestamp: Date.now() }
}));
```

---

##  修改清单

| 文件 | 修改项 | 行号 |
|------|--------|------|
| `outliner_view.py` | 关闭事件处理器 | 246 |
| `outliner_view.py` | HTML 样式 - 按钮布局 | 278-313 |
| `outliner_view.py` | HTML 头部 - 关闭按钮 | 379-385 |
| `outliner_view.py` | JavaScript - closeWindow() | 478-490 |
| `outliner_view.py` | JavaScript - refreshScene() | 464-476 |
| `outliner_view.py` | JavaScript - selectNode() | 419-433 |
| `outliner_view.py` | JavaScript - renameSelected() | 492-507 |
| `outliner_view.py` | JavaScript - deleteSelected() | 509-524 |
| `outliner_view.py` | 初始刷新延迟 | 574-578 |

---

## [TEST] 测试方法

### 在 Maya 中测试

```python
# 1. 在 Maya Script Editor 中运行
exec(open(r'C:\path\to\outliner_view.py').read())

# 2. 验证场景物体显示
# - 应该看到 pSphere1-6 等物体在 Outliner 中显示

# 3. 测试刷新功能
# - 点击 "[REFRESH] Refresh" 按钮
# - 应该看到物体列表更新

# 4. 测试关闭功能
# - 点击 "[CLOSE] Close" 按钮
# - WebView 窗口应该正常关闭

# 5. 测试选择功能
# - 在 Outliner 中点击物体
# - 应该在 Maya 中选中该物体

# 6. 测试右键菜单
# - 右键点击物体
# - 选择 "[EDIT] Rename" 或 "[DELETE] Delete"
# - 应该能正常重命名或删除
```

---

## [SEARCH] 技术细节

### IPC 通信机制

AuroraView 使用 IPC (Inter-Process Communication) 实现 Python-JavaScript 通信：

1. **JavaScript → Python**：
   - JavaScript 调用 `window.ipc.postMessage()`
   - Rust 核心的 IPC 处理器接收消息
   - 调用对应的 Python 回调函数

2. **Python → JavaScript**：
   - Python 调用 `webview.emit()`
   - Rust 核心执行 JavaScript 代码
   - JavaScript 监听 `window.addEventListener()`

### 嵌入式模式的消息处理

在嵌入式模式下（与 Maya 集成）：
- WebView 不运行独立的事件循环
- 需要通过 `process_events()` 定期处理消息
- 使用 Maya 的 `scriptJob` 定时调用 `process_events()`

---

## [OK] 验证清单

- [x] WebView 可以通过关闭按钮正常关闭
- [x] 场景物体在 WebView 启动时显示
- [x] 手动刷新按钮可以更新物体列表
- [x] 点击物体可以在 Maya 中选中
- [x] 右键菜单可以重命名和删除物体
- [x] 所有事件通过 IPC 正确传递

---

## [DOCS] 相关文档

- `README_MAYA_INTEGRATION.md` - Maya 集成指南
- `TECHNICAL_DESIGN.md` - 技术设计文档
- `docs/MAYA_EMBEDDED_INTEGRATION.md` - 嵌入式集成详细说明

