# 🚀 快速开始 - Photoshop 图层管理

## 5 分钟快速上手

### 步骤 1: 安装 UXP 插件 (首次使用)

1. 打开 **Photoshop**
2. 打开 **UXP Developer Tool** (插件 → 开发)
3. 点击 **"Add Plugin..."**
4. 选择文件:
   ```
   examples/photoshop_auroraview/uxp_plugin/manifest.json
   ```
5. 点击 **"Load"**

**验证**: 应该看到 **"AuroraView Bridge v2"** 已加载

---

### 步骤 2: 启动 Python 后端

```bash
python examples/photoshop_layers_demo/photoshop_layers_tool.py
```

**预期输出**:
```
✅ Found free port: 9001
📡 Bridge port: 9001
✅ WebView created with Bridge on port 9001
```

**WebView 窗口会自动打开**

---

### 步骤 3: 连接 Photoshop

1. 在 Photoshop 中**创建或打开一个文档** (重要！)
2. 打开插件面板: **窗口 → 插件 → AuroraView Bridge v2**
3. 点击 **"Connect to Python"**

**成功标志**:
- UXP 插件显示: **"✅ Connected to Python"** (绿色)
- WebView 显示: **"✅ Connected to Photoshop"**

---

### 步骤 4: 开始使用

在 **WebView 窗口**中:

1. **创建图层**:
   - 输入图层名称 (例如: "Background")
   - 点击 **"➕ Create Layer"**
   - 图层会在 Photoshop 中创建

2. **查看图层**:
   - 点击 **"🔄 Refresh Layers"**
   - 所有图层会显示在列表中

3. **编辑图层**:
   - 点击 ✏️ 重命名图层
   - 点击 🗑️ 删除图层

4. **查看文档信息**:
   - 点击 **"📄 Get Document Info"**
   - 文档信息会显示在顶部

---

## 🎯 完整示例流程

### 示例 1: 创建多个图层

1. 创建图层 "Background"
2. 创建图层 "Main Content"
3. 创建图层 "Text Layer"
4. 点击 "🔄 Refresh Layers" 查看所有图层

### 示例 2: 管理图层

1. 点击 "Background" 旁边的 ✏️
2. 重命名为 "BG Layer"
3. 点击 "Text Layer" 旁边的 🗑️ 删除
4. 刷新查看更新后的列表

---

## 📊 界面说明

### WebView 窗口

```
┌─────────────────────────────────┐
│ 🎨 Photoshop Layers             │
├─────────────────────────────────┤
│ ✅ Connected to Photoshop       │ ← 连接状态
│                                 │
│ Document Info                   │
│ Name: Untitled-1                │ ← 文档信息
│ Size: 1920 × 1080               │
│ Layers: 3                       │
│                                 │
│ Create New Layer                │
│ ┌─────────────────────────────┐ │
│ │ Layer Name                  │ │ ← 输入框
│ └─────────────────────────────┘ │
│ [➕ Create Layer]               │ ← 创建按钮
│                                 │
│ Layers                          │
│ ┌─────────────────────────────┐ │
│ │ Background (ID: 123)        │ │
│ │ Visible: ✓  Opacity: 100%   │ │
│ │ [✏️] [🗑️]                    │ │ ← 操作按钮
│ ├─────────────────────────────┤ │
│ │ Main Content (ID: 124)      │ │
│ │ Visible: ✓  Opacity: 100%   │ │
│ │ [✏️] [🗑️]                    │ │
│ └─────────────────────────────┘ │
│                                 │
│ [🔄 Refresh Layers]             │ ← 刷新按钮
│ [📄 Get Document Info]          │
└─────────────────────────────────┘
```

### UXP 插件面板

```
┌─────────────────────────────────┐
│ AuroraView Bridge               │
├─────────────────────────────────┤
│ ✅ Connected to Python          │ ← 连接状态
│ [Disconnect]                    │
│                                 │
│ Activity Log                    │
│ ┌─────────────────────────────┐ │
│ │ [23:45:19] AuroraView       │ │
│ │            Bridge           │ │
│ │            initialized      │ │
│ │ [23:45:25] Connecting...    │ │
│ │ [23:45:25] ✅ Connected     │ │
│ │ [23:45:26] 📨 Received:     │ │ ← 日志
│ │            handshake_ack    │ │
│ │ [23:45:30] 📨 Received:     │ │
│ │            layer_created    │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘
```

---

## 🔍 故障排除

### 问题 1: 连接失败

**症状**: 点击 "Connect to Python" 后仍显示 "Disconnected"

**解决方案**:
1. 确认 Python 脚本正在运行
2. 确认 WebView 窗口已打开
3. 确认 Photoshop 中有打开的文档
4. 检查 UXP Developer Tool 的 Logs 标签是否有错误

### 问题 2: 权限错误

**症状**: "Permission denied to the url ws://localhost:9001"

**解决方案**:
1. 在 UXP Developer Tool 中移除旧插件
2. 重新加载插件
3. 确认插件名称是 "AuroraView Bridge v2"

### 问题 3: 命令无响应

**症状**: 点击按钮没有反应

**解决方案**:
1. 确认 Photoshop 中有打开的文档
2. 确认连接状态显示 "Connected"
3. 查看 Python 日志是否有错误

---

## 📖 更多信息

- [完整文档](README_zh.md)
- [UXP 插件安装指南](../photoshop_auroraview/uxp_plugin/INSTALL_V2.md)
- [实现总结](IMPLEMENTATION_SUMMARY.md)

---

**现在开始创建你的第一个图层吧！** 🎨

