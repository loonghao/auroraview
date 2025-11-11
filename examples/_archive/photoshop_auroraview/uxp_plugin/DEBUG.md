# UXP Plugin Debug Guide

## 问题诊断

### 1. 检查插件是否加载

**症状**: 插件面板显示但没有日志

**检查步骤**:

1. 打开 **UXP Developer Tool**
2. 找到 **"AuroraView Bridge (Minimal)"**
3. 点击插件查看详情
4. 检查 **"Logs"** 标签页

**预期日志**:
```
AuroraView Bridge initialized
```

**如果没有日志**:
- JavaScript 可能有错误
- DOM 元素可能没有正确加载

---

### 2. 检查 DOM 元素

**在 UXP Developer Tool 的 Console 中运行**:

```javascript
// 检查元素是否存在
console.log('statusEl:', document.getElementById('status'));
console.log('logEl:', document.getElementById('log'));
console.log('connectBtn:', document.getElementById('connectBtn'));

// 检查 document.readyState
console.log('readyState:', document.readyState);
```

**预期输出**:
```
statusEl: <div id="status" class="status disconnected">
logEl: <div id="log" class="log">
connectBtn: <button id="connectBtn">
readyState: complete
```

---

### 3. 手动初始化

**如果自动初始化失败，在 Console 中手动运行**:

```javascript
// 手动获取元素
statusEl = document.getElementById('status');
logEl = document.getElementById('log');
connectBtn = document.getElementById('connectBtn');

// 手动添加日志
function log(message) {
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    logEl.appendChild(entry);
    logEl.scrollTop = logEl.scrollHeight;
}

// 测试日志
log('Manual initialization test');
```

---

### 4. 检查 WebSocket 连接

**在 Console 中运行**:

```javascript
// 测试 WebSocket 连接
const testSocket = new WebSocket('ws://localhost:9001');

testSocket.onopen = () => {
    console.log('✅ WebSocket connected');
    log('✅ WebSocket connected');
};

testSocket.onerror = (error) => {
    console.error('❌ WebSocket error:', error);
    log('❌ WebSocket error');
};

testSocket.onclose = () => {
    console.log('WebSocket closed');
    log('WebSocket closed');
};
```

---

### 5. 常见问题

#### 问题 1: "Loading..." 一直显示

**原因**: JavaScript 没有执行或初始化失败

**解决方案**:
1. 检查 UXP Developer Tool 的 Logs 标签
2. 查看是否有 JavaScript 错误
3. 尝试手动初始化（见步骤 3）

#### 问题 2: 点击 "Connect to Python" 没有反应

**原因**: 事件监听器没有绑定

**解决方案**:
```javascript
// 在 Console 中手动绑定
connectBtn.addEventListener('click', connect);
```

#### 问题 3: 连接失败

**原因**: Python Bridge 没有运行或端口错误

**检查清单**:
- [ ] Python 脚本正在运行
- [ ] Bridge 监听在 9001 端口
- [ ] 防火墙允许连接
- [ ] WebSocket URL 正确: `ws://localhost:9001`

---

### 6. 完整重置流程

如果一切都不工作，尝试完整重置：

1. **关闭 Photoshop**

2. **删除插件缓存**:
   - Windows: `%APPDATA%\Adobe\UXP\PluginsStorage`
   - macOS: `~/Library/Application Support/Adobe/UXP/PluginsStorage`

3. **重新启动 Photoshop**

4. **重新加载插件**:
   - UXP Developer Tool → Add Plugin
   - 选择 `manifest.json`
   - Load

5. **检查日志**:
   - UXP Developer Tool → Logs
   - 应该看到 "AuroraView Bridge initialized"

---

### 7. 验证修复

**成功的标志**:

1. **插件面板**:
   ```
   AuroraView Bridge
   [Disconnected] (红色)
   [Connect to Python]
   Activity Log
   [23:35:19] AuroraView Bridge initialized
   ```

2. **UXP Developer Tool Logs**:
   ```
   AuroraView Bridge initialized
   ```

3. **点击 Connect 后**:
   ```
   [23:35:20] Connecting to Python backend...
   [23:35:20] ✅ Connected to Python backend
   [23:35:20] 📨 Received: handshake_ack
   ```

---

## 当前修改

### 修改 1: 改进初始化逻辑

**文件**: `index.js`

**变更**:
- 添加了 `initialize()` 函数
- 检查 `document.readyState`
- 添加了 DOM 元素存在性检查
- 添加了错误日志

### 修改 2: 增加日志区域可见性

**文件**: `index.html`

**变更**:
- 添加 `min-height: 300px`
- 添加边框 `border: 1px solid #444`
- 添加初始 "Loading..." 文本

### 修改 3: 增加面板尺寸

**文件**: `manifest.json`

**变更**:
- 设置默认宽度: 350px
- 设置默认高度: 600px

---

## 下一步

1. **重新加载插件** (UXP Developer Tool → Reload)
2. **打开插件面板** (窗口 → 插件 → AuroraView)
3. **检查日志区域** - 应该看到 "AuroraView Bridge initialized"
4. **点击 Connect** - 观察连接过程

如果还有问题，请查看 UXP Developer Tool 的 Logs 标签并报告错误信息。

