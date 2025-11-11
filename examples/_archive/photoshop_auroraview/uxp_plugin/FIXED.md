# ✅ 问题已修复！

## 🐛 问题原因

**错误信息**:
```
Permission denied to the url ws://localhost:9001. 
Manifest entry not found.
```

**根本原因**:
UXP 的网络权限配置不支持端口通配符 `*`。

**错误配置**:
```json
"domains": [
  "ws://localhost:*",    // ❌ 不支持通配符
  "wss://localhost:*"
]
```

**正确配置**:
```json
"domains": [
  "ws://localhost:9001",   // ✅ 明确指定端口
  "wss://localhost:9001",
  "http://localhost:9000"
]
```

---

## 🔧 已修复的文件

### `manifest.json`

**修改内容**:
- 移除端口通配符 `*`
- 明确指定 WebSocket 端口: `9001`
- 明确指定 HTTP 发现端口: `9000`

---

## 📋 现在请执行

### 步骤 1: 重新加载插件

**重要**: 修改 `manifest.json` 后必须重新加载插件！

1. 打开 **UXP Developer Tool**
2. 找到 **"AuroraView Bridge (Minimal)"**
3. 点击 **"..."** → **"Reload"**

### 步骤 2: 打开插件面板

1. 窗口 → 插件 → **AuroraView (Minimal)**
2. 确保 Photoshop 中有打开的文档

### 步骤 3: 连接到 Python

1. 确认 Python Bridge 正在运行（Terminal 148）
2. 点击 **"Connect to Python"**

---

## ✅ 预期结果

### UXP 插件面板

```
AuroraView Bridge
┌─────────────────────────────────┐
│ ✅ Connected to Python (绿色)   │
└─────────────────────────────────┘

[Disconnect]

Activity Log
┌─────────────────────────────────┐
│ [23:35:19] AuroraView Bridge    │
│            initialized          │
│ [23:35:25] Connecting to Python │
│            backend...           │
│ [23:35:25] ✅ Connected to      │
│            Python backend       │
│ [23:35:25] 📨 Received:         │
│            handshake_ack        │
└─────────────────────────────────┘
```

### Python 日志 (Terminal 148)

```
2025-11-09 23:35:25 - websockets.server - INFO - connection open
2025-11-09 23:35:25 - auroraview.bridge - INFO - 🤝 Photoshop connected: {
    'client': 'photoshop',
    'app': 'Photoshop',
    'version': '26.5.0'
}
```

---

## 🎯 测试连接

连接成功后，在 UXP 插件面板中测试：

### 测试 1: 发送测试消息

在 UXP Developer Tool 的 Console 中运行：

```javascript
sendMessage('test', { message: 'Hello from Photoshop!' });
```

**预期**: Python 日志显示收到消息

### 测试 2: 创建图层

在 Console 中运行：

```javascript
createLayer({ name: 'Test Layer' });
```

**预期**: 
- Photoshop 中创建新图层
- Python 日志显示 "🎨 Layer created: Test Layer"

---

## 🔍 如果还有问题

### 检查清单

- [ ] Python Bridge 正在运行（Terminal 148）
- [ ] Bridge 监听在端口 9001
- [ ] UXP 插件已重新加载
- [ ] Photoshop 中有打开的文档
- [ ] manifest.json 中的端口号正确

### 查看日志

**UXP Developer Tool → Logs**:
- 应该看到 "AuroraView Bridge initialized"
- 应该看到 "Connecting to Python backend..."
- 应该看到 "✅ Connected to Python backend"

**Python Terminal**:
- 应该看到 "🤝 Photoshop connected"

---

## 📖 相关文档

- [快速开始指南](../../photoshop_layers_demo/QUICK_START.md)
- [调试指南](DEBUG.md)
- [故障排除](TROUBLESHOOTING.md)

---

**现在重新加载插件并尝试连接！** 🚀

