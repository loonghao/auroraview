# Photoshop 图层管理 - 快速开始指南

## 🚀 5 分钟快速开始

### 步骤 1: 启动 Python 后端

```bash
# Windows
.\examples\photoshop_layers_demo\start.ps1

# 或直接运行
python examples/photoshop_layers_demo/photoshop_layers_tool.py
```

**预期输出**:
```
✅ Found free port: 9001
📡 Bridge port: 9001
🔍 HTTP discovery: http://localhost:9000/discover
✅ WebView created with Bridge on port 9001
```

**重要**: 保持这个窗口打开！WebView 窗口会自动弹出。

---

### 步骤 2: 重新加载 UXP 插件

**为什么需要重新加载？**
我们修复了 DOM 访问的问题，需要重新加载插件才能生效。

**操作步骤**:

1. 打开 Photoshop
2. 打开 **UXP Developer Tool**:
   - 菜单: 插件 → 开发 → UXP Developer Tool
3. 找到 **"AuroraView Bridge (Minimal)"** 插件
4. 点击插件旁边的 **"..."** 按钮
5. 选择 **"Reload"**
6. 等待插件重新加载（应该没有错误）

**如果插件不在列表中**:
1. 点击 **"Add Plugin"**
2. 选择 `examples/photoshop_auroraview/uxp_plugin/manifest.json`
3. 点击 **"Load"**

---

### 步骤 3: 打开插件面板

1. 在 Photoshop 中，打开插件面板:
   - 菜单: 窗口 → 插件 → AuroraView (Minimal)
2. 插件面板应该显示:
   ```
   AuroraView Bridge
   [Disconnected]  (红色)
   [Connect to Python]
   Activity Log
   ```

---

### 步骤 4: 连接到 Python

**重要**: 确保 Photoshop 中有打开的文档！

1. 在 Photoshop 中创建或打开一个文档
2. 在插件面板中，点击 **"Connect to Python"** 按钮
3. 等待连接（1-2 秒）

**成功连接的标志**:
- 插件面板显示: `✅ Connected to Python` (绿色)
- WebView 窗口显示: `✅ Connected to Photoshop`
- Activity Log 显示连接日志

---

### 步骤 5: 开始使用

现在你可以在 WebView 窗口中：

1. **创建图层**:
   - 输入图层名称（例如: "My Layer"）
   - 点击 **"➕ Create Layer"**
   - 图层会在 Photoshop 中创建

2. **查看图层列表**:
   - 点击 **"🔄 Refresh Layers"**
   - 所有图层会显示在列表中

3. **编辑图层**:
   - 点击 ✏️ 重命名图层
   - 点击 🗑️ 删除图层

4. **查看文档信息**:
   - 点击 **"📄 Get Document Info"**
   - 文档信息会显示在顶部

---

## 🔍 故障排除

### 问题 1: UXP 插件加载失败

**错误**: "Plugin Load Failed"

**解决方案**:
1. 确保已重新加载插件（步骤 2）
2. 检查 UXP Developer Tool 的 "Logs" 标签
3. 如果有错误，尝试完全移除并重新添加插件

### 问题 2: 连接失败

**症状**: 点击 "Connect to Python" 后仍然显示 "Disconnected"

**检查清单**:
- [ ] Python 后端正在运行
- [ ] WebView 窗口已打开
- [ ] 端口 9001 未被占用
- [ ] 防火墙允许 Python 连接

**解决方案**:
1. 检查 Python 日志，确认 Bridge 已启动
2. 检查 UXP 插件的 Activity Log
3. 尝试重启 Python 后端

### 问题 3: WebView 显示 "Waiting for Photoshop..."

**原因**: UXP 插件未连接到 Python

**解决方案**:
1. 打开 Photoshop 中的 UXP 插件面板
2. 点击 **"Connect to Python"** 按钮
3. 等待连接成功

### 问题 4: 命令无响应

**症状**: 点击按钮没有反应

**检查清单**:
- [ ] Photoshop 中有打开的文档
- [ ] UXP 插件显示 "Connected"
- [ ] WebView 显示 "Connected to Photoshop"

**解决方案**:
1. 确保 Photoshop 中有打开的文档
2. 检查 Python 日志是否有错误
3. 检查 UXP Activity Log 是否有错误

---

## 📊 连接状态检查

### Python 后端

**正常状态**:
```
✅ Found free port: 9001
✅ Bridge initialized: localhost:9001
✅ WebView created with Bridge on port 9001
✅ Bridge background thread started
```

### UXP 插件

**正常状态**:
```
AuroraView Bridge
✅ Connected to Python  (绿色)
[Disconnect]

Activity Log
[23:27:59] AuroraView Bridge initialized
[23:28:05] Connecting to Python backend...
[23:28:05] ✅ Connected to Python backend
[23:28:05] 📨 Received: handshake_ack
```

### WebView

**正常状态**:
```
🎨 Photoshop Layers
✅ Connected to Photoshop  (绿色)

Document Info
Name: Untitled-1
Size: 1920 × 1080
Layers: 1
Mode: RGB
```

---

## 🎯 完整流程示例

### 创建图层

1. 在 WebView 中输入: "Background Layer"
2. 点击 **"➕ Create Layer"**
3. 观察:
   - Photoshop 中创建了新图层
   - UXP Activity Log: `📨 Received: execute_command`
   - Python 日志: `🎨 Layer created: Background Layer`
   - WebView 显示通知: `✅ Layer created: Background Layer`

### 获取图层列表

1. 点击 **"🔄 Refresh Layers"**
2. 观察:
   - UXP Activity Log: `📨 Received: execute_command`
   - Python 日志: `📋 Received 1 layers`
   - WebView 显示图层列表

---

## 📖 相关文档

- [完整使用指南](README_zh.md)
- [故障排除指南](../photoshop_auroraview/uxp_plugin/TROUBLESHOOTING.md)
- [实现总结](IMPLEMENTATION_SUMMARY.md)

---

## ✅ 成功标志

当一切正常时，你应该看到：

1. **Python 后端**: 日志显示 Bridge 已启动
2. **UXP 插件**: 显示 "✅ Connected to Python"
3. **WebView**: 显示 "✅ Connected to Photoshop"
4. **功能**: 可以创建图层、获取列表、编辑图层

**现在开始使用吧！** 🚀

