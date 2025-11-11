# 🔄 完全重新安装 UXP 插件

## 问题

即使修改了 `manifest.json`，UXP 仍然显示权限错误。这是因为 UXP 缓存了旧的 manifest 配置。

---

## ✅ 解决方案：完全重新安装

### 步骤 1: 卸载旧插件

1. 打开 **UXP Developer Tool**
2. 找到 **"AuroraView Bridge (Minimal)"**
3. 点击 **"..."** → **"Remove"** (移除)
4. 确认移除

### 步骤 2: 关闭 Photoshop

**重要**: 完全关闭 Photoshop，确保所有进程都结束。

### 步骤 3: 清除 UXP 缓存（可选但推荐）

**Windows**:
```powershell
# 删除 UXP 插件缓存
Remove-Item -Path "$env:APPDATA\Adobe\UXP\PluginsStorage\*" -Recurse -Force
```

**macOS**:
```bash
# 删除 UXP 插件缓存
rm -rf ~/Library/Application\ Support/Adobe/UXP/PluginsStorage/*
```

### 步骤 4: 重新启动 Photoshop

启动 Photoshop 并打开一个文档。

### 步骤 5: 重新加载插件

1. 打开 **UXP Developer Tool** (插件 → 开发 → UXP Developer Tool)
2. 点击 **"Add Plugin..."**
3. 浏览到插件目录并选择 `manifest.json`:
   ```
   C:\Users\hallo\Documents\augment-projects\dcc_webview\examples\photoshop_auroraview\uxp_plugin\manifest.json
   ```
4. 点击 **"Load"**

### 步骤 6: 验证权限

在 UXP Developer Tool 中：

1. 选择插件
2. 查看 **"Details"** 标签
3. 检查 **"Network Permissions"** 部分

**应该显示**:
```
Network Permissions:
- ws://localhost:9001
- wss://localhost:9001
- http://localhost:9000
```

**如果显示**:
```
- ws://localhost:*
- wss://localhost:*
```

说明缓存没有清除，需要重复步骤 2-5。

### 步骤 7: 打开插件面板

1. 窗口 → 插件 → **AuroraView (Minimal)**
2. 检查日志区域是否显示 "AuroraView Bridge initialized"

### 步骤 8: 连接到 Python

1. 确认 Python Bridge 正在运行
2. 点击 **"Connect to Python"**

---

## 🔍 验证成功

### 检查 1: UXP Developer Tool Logs

应该看到：
```
AuroraView Bridge initialized
Connecting to Python backend...
✅ Connected to Python backend
📨 Received: handshake_ack
```

**不应该看到**:
```
❌ Permission denied to the url ws://localhost:9001
```

### 检查 2: 插件面板

应该显示：
```
✅ Connected to Python (绿色)
```

### 检查 3: Python 日志

应该看到：
```
🤝 Photoshop connected: {'client': 'photoshop', ...}
```

---

## 🚨 如果还是失败

### 方案 A: 更改插件 ID

修改 `manifest.json` 中的 ID（强制 UXP 识别为新插件）:

```json
{
  "id": "com.auroraview.photoshop.minimal.v2",  // 添加 .v2
  "name": "AuroraView Bridge (Minimal) v2",
  ...
}
```

然后重复步骤 1-8。

### 方案 B: 使用不同的端口

如果端口 9001 被其他应用占用：

1. **修改 Python 脚本**:
   ```python
   bridge = Bridge(port=9002)  # 改为 9002
   ```

2. **修改 manifest.json**:
   ```json
   "domains": [
     "ws://localhost:9002",
     "wss://localhost:9002",
     "http://localhost:9000"
   ]
   ```

3. **修改 index.js**:
   ```javascript
   const BRIDGE_URL = 'ws://localhost:9002';  // 改为 9002
   ```

### 方案 C: 检查防火墙

确保 Windows 防火墙允许 Python 连接：

```powershell
# 以管理员身份运行
New-NetFirewallRule -DisplayName "Python WebSocket" -Direction Inbound -Protocol TCP -LocalPort 9001 -Action Allow
```

---

## 📝 完整检查清单

在尝试连接前，确保：

- [ ] 旧插件已完全移除
- [ ] Photoshop 已重启
- [ ] UXP 缓存已清除（可选）
- [ ] 插件已重新加载
- [ ] manifest.json 显示正确的端口（9001）
- [ ] Python Bridge 正在运行
- [ ] Photoshop 中有打开的文档
- [ ] 防火墙允许连接

---

## 🎯 快速重新安装脚本

**Windows PowerShell**:

```powershell
# 1. 关闭 Photoshop（手动）

# 2. 清除缓存
Remove-Item -Path "$env:APPDATA\Adobe\UXP\PluginsStorage\*" -Recurse -Force -ErrorAction SilentlyContinue

# 3. 重启 Photoshop（手动）

# 4. 在 UXP Developer Tool 中重新加载插件（手动）
```

---

**现在请按照步骤 1-8 完全重新安装插件！** 🔄

