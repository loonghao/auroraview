# 🚀 安装 AuroraView Bridge v2

## ✅ 已修复的问题

**问题**: UXP 缓存了旧的 manifest.json，导致权限错误
**解决方案**: 更改插件 ID，让 UXP 识别为全新插件

---

## 📋 安装步骤

### 步骤 1: 移除旧插件（如果存在）

1. 打开 **UXP Developer Tool**
2. 找到 **"AuroraView Bridge (Minimal)"** 或任何旧版本
3. 点击 **"..."** → **"Remove"**

### 步骤 2: 加载新插件

1. 在 UXP Developer Tool 中点击 **"Add Plugin..."**
2. 选择 manifest.json:
   ```
   C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\photoshop_auroraview\uxp_plugin\manifest.json
   ```
3. 点击 **"Load"**

### 步骤 3: 验证插件信息

在 UXP Developer Tool 中，你应该看到：

```
Name: AuroraView Bridge v2
ID: com.auroraview.photoshop.bridge.v2
Version: 2.0.0
Status: Loaded
```

### 步骤 4: 检查网络权限

1. 在 UXP Developer Tool 中选择插件
2. 查看 **"Details"** 或 **"Manifest"** 标签
3. 确认网络权限包含：
   ```
   - ws://localhost:9001
   - wss://localhost:9001
   - http://localhost:9000
   ```

### 步骤 5: 测试权限

在 **UXP Developer Tool Console** 中运行：

```javascript
// 测试 WebSocket 权限
try {
    const ws = new WebSocket('ws://localhost:9001');
    console.log('✅ WebSocket permission OK - Ready to connect!');
    ws.close();
} catch (e) {
    console.error('❌ Still permission denied:', e.message);
}
```

**预期输出**: `✅ WebSocket permission OK - Ready to connect!`

### 步骤 6: 打开插件面板

1. 窗口 → 插件 → **AuroraView Bridge v2**
2. 面板应该显示：
   ```
   AuroraView Bridge
   [Disconnected]
   [Connect to Python]
   Activity Log
   [HH:MM:SS] AuroraView Bridge initialized
   ```

### 步骤 7: 启动 Python Bridge

如果还没有运行，启动 Python 服务：

```bash
python examples/photoshop_layers_demo/test_bridge_only.py
```

**预期输出**:
```
✅ Found free port: 9001
✅ Bridge started on port 9001
✅ WebSocket server listening on ws://localhost:9001
📡 Waiting for clients to connect...
```

### 步骤 8: 连接到 Python

1. 确保 Photoshop 中有打开的文档
2. 在插件面板中点击 **"Connect to Python"**
3. 观察日志

---

## ✅ 成功标志

### UXP 插件面板

```
AuroraView Bridge
✅ Connected to Python (绿色)
[Disconnect]

Activity Log
[23:45:19] AuroraView Bridge initialized
[23:45:25] Connecting to Python backend...
[23:45:25] ✅ Connected to Python backend
[23:45:25] 📨 Received: handshake_ack
```

### Python 日志

```
2025-11-09 23:45:25 - websockets.server - INFO - connection open
2025-11-09 23:45:25 - auroraview.bridge - INFO - 🤝 Photoshop connected: {
    'client': 'photoshop',
    'app': 'Photoshop',
    'version': '26.5.0'
}
```

### UXP Developer Tool Logs

```
AuroraView Bridge initialized
Connecting to Python backend...
✅ Connected to Python backend
📨 Received: handshake_ack
```

---

## 🎯 测试连接

连接成功后，在 UXP Developer Tool Console 中测试：

```javascript
// 测试发送消息
sendMessage('test', { message: 'Hello from Photoshop!' });

// 测试创建图层
createLayer({ name: 'Test Layer from Console' });

// 测试获取图层列表
getLayers();
```

---

## 🔍 故障排除

### 问题 1: 还是显示 "Permission denied"

**原因**: 插件没有完全重新加载

**解决方案**:
1. 完全关闭 Photoshop
2. 删除缓存: `%APPDATA%\Adobe\UXP\PluginsStorage`
3. 重启 Photoshop
4. 重新加载插件

### 问题 2: 找不到插件面板

**位置**: 窗口 → 插件 → **AuroraView Bridge v2**

**注意**: 名称已从 "AuroraView (Minimal)" 改为 "AuroraView Bridge v2"

### 问题 3: 连接失败但没有权限错误

**检查清单**:
- [ ] Python Bridge 正在运行
- [ ] Bridge 监听在 9001 端口
- [ ] Photoshop 中有打开的文档
- [ ] 防火墙允许连接

---

## 📖 下一步

连接成功后，你可以：

1. **测试图层操作**:
   - 创建图层
   - 获取图层列表
   - 删除图层
   - 重命名图层

2. **启动完整的 WebView 示例**:
   ```bash
   python examples/photoshop_layers_demo/photoshop_layers_tool.py
   ```

3. **开发自己的功能**:
   - 在 `index.js` 中添加新命令
   - 在 Python 中添加新的事件处理器

---

## 🎉 总结

**v2 的改进**:
- ✅ 新的插件 ID（避免缓存问题）
- ✅ 明确的网络权限配置
- ✅ 更大的面板尺寸（600px）
- ✅ 更好的日志显示（400px）
- ✅ 改进的初始化逻辑

**现在开始使用吧！** 🚀

