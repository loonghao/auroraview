# Photoshop 图层管理示例

这是一个完整的 Photoshop 集成示例，展示如何在 AuroraView WebView 中创建和管理 Photoshop 图层。

## 功能特性

### ✅ 图层操作
- **创建图层**: 在 WebView 中点击按钮创建新图层
- **获取图层列表**: 实时显示所有图层信息
- **删除图层**: 删除指定图层
- **重命名图层**: 修改图层名称
- **文档信息**: 获取文档尺寸、图层数量等信息

### ✅ 技术亮点
- **服务发现**: 自动端口分配，避免冲突
- **双向通信**: WebView ↔ Bridge ↔ UXP ↔ Photoshop
- **实时更新**: 图层变化自动同步到 UI
- **美观界面**: 现代化的渐变 UI 设计

## 快速开始

### 1. 安装依赖

```bash
# 确保已安装 websockets
pip install websockets

# 重新编译 AuroraView（包含服务发现）
maturin develop --release
```

### 2. 加载 UXP 插件

**重要**: 使用 v2 版本的插件（已修复权限问题）

1. 打开 Photoshop
2. 打开 UXP Developer Tool (插件 → 开发)
3. 点击 "Add Plugin..."
4. 选择 `examples/photoshop_auroraview/uxp_plugin/manifest.json`
5. 点击 "Load" 加载插件

**验证插件信息**:
- 名称: `AuroraView Bridge v2`
- ID: `com.auroraview.photoshop.bridge.v2`
- 版本: `2.0.0`

**如果之前安装过旧版本**:
1. 在 UXP Developer Tool 中移除旧插件
2. 重新加载新版本插件

### 3. 运行示例

```bash
python examples/photoshop_layers_demo/photoshop_layers_tool.py
```

### 4. 连接 Photoshop

1. 在 Photoshop 中打开 UXP 插件面板:
   - 窗口 → 插件 → **AuroraView Bridge v2**
2. 确保 Photoshop 中有打开的文档
3. 点击 **"Connect to Python"** 按钮
4. 看到 **"✅ Connected to Python"** (绿色) 表示连接成功

**Activity Log 应该显示**:
```
[HH:MM:SS] AuroraView Bridge initialized
[HH:MM:SS] Connecting to Python backend...
[HH:MM:SS] ✅ Connected to Python backend
[HH:MM:SS] 📨 Received: handshake_ack
```

### 5. 开始使用

- 在 WebView 窗口中输入图层名称
- 点击 "➕ Create Layer" 创建图层
- 点击 "🔄 Refresh Layers" 刷新图层列表
- 点击图层旁边的 ✏️ 重命名或 🗑️ 删除

## 架构说明

### 通信流程

```
┌─────────────────────────────────────────────────────────────┐
│                    完整通信流程                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  WebView UI          Bridge           UXP Plugin  Photoshop │
│  (HTML/JS)          (Python)          (JavaScript)  (API)   │
│                                                              │
│  ┌──────────┐      ┌──────────┐      ┌──────────┐ ┌──────┐│
│  │ 点击按钮  │─1──>│ emit()   │      │          │ │      ││
│  │          │      │          │      │          │ │      ││
│  │          │      │ @on()    │─2──>│ WebSocket│ │      ││
│  │          │      │          │      │          │ │      ││
│  │          │      │          │      │ execute  │─3─>│ API││
│  │          │      │          │      │ command  │ │      ││
│  │          │      │          │      │          │ │      ││
│  │          │      │          │<─4───│ send     │<─5─│ 结果││
│  │          │      │          │      │ message  │ │      ││
│  │          │      │ emit()   │      │          │ │      ││
│  │ 更新UI   │<─6───│ bridge:  │      │          │ │      ││
│  │          │      │ event    │      │          │ │      ││
│  └──────────┘      └──────────┘      └──────────┘ └──────┘│
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 代码结构

```
photoshop_layers_demo/
├── photoshop_layers_tool.py    # Python 主程序
├── ui.html                      # WebView UI 界面
└── README_zh.md                 # 本文档
```

## 代码示例

### Python 端 (photoshop_layers_tool.py)

```python
from auroraview import Bridge, WebView

# 创建 Bridge（自动端口分配）
bridge = Bridge(
    port=0,                    # 自动分配
    service_discovery=True,    # 启用服务发现
)

# 注册事件处理器
@bridge.on('layer_created')
async def handle_layer_created(data, client):
    logger.info(f"🎨 Layer created: {data}")
    
    # 通知 WebView
    if webview:
        webview.emit('bridge:layer_created', data)
    
    return None

# 创建 WebView 并关联 Bridge
webview = WebView.create(
    title="Photoshop Layers Demo",
    html=html_content,
    bridge=bridge  # 自动连接
)

webview.show()
```

### JavaScript 端 (ui.html)

```javascript
// 监听 Bridge 事件
window.aurora.on('bridge:layer_created', (data) => {
    console.log('Layer created:', data);
    showNotification(`✅ Layer created: ${data.name}`);
    refreshLayers();
});

// 发送命令到 Photoshop
function createLayer() {
    const name = document.getElementById('layerName').value;
    
    window.aurora.emit('send_to_bridge', {
        action: 'execute_command',
        data: {
            command: 'create_layer',
            params: { name: name }
        }
    });
}
```

### UXP 插件端 (index.js)

```javascript
// 创建图层
async function createLayer(params) {
    await app.batchPlay([{
        _obj: 'make',
        _target: [{ _ref: 'layer' }],
        using: {
            _obj: 'layer',
            name: params.name || 'New Layer'
        }
    }], {});
    
    const layer = app.activeDocument.activeLayers[0];
    
    // 发送结果到 Python
    sendMessage('layer_created', {
        name: layer.name,
        id: layer.id
    });
}
```

## 支持的命令

### 从 WebView 发送到 Photoshop

| 命令 | 参数 | 说明 |
|------|------|------|
| `create_layer` | `{ name: string }` | 创建新图层 |
| `get_layers` | `{}` | 获取所有图层 |
| `delete_layer` | `{ id: number }` | 删除图层 |
| `rename_layer` | `{ id: number, newName: string }` | 重命名图层 |
| `get_document_info` | `{}` | 获取文档信息 |

### 从 Photoshop 发送到 WebView

| 事件 | 数据 | 说明 |
|------|------|------|
| `layer_created` | `{ name, id, bounds }` | 图层已创建 |
| `layers_list` | `{ count, layers[] }` | 图层列表 |
| `layer_deleted` | `{ id, name }` | 图层已删除 |
| `layer_renamed` | `{ id, oldName, newName }` | 图层已重命名 |
| `document_info` | `{ name, width, height, ... }` | 文档信息 |

## 故障排除

### 连接失败

1. **检查端口**: 确保 Bridge 端口未被占用
2. **检查 UXP 插件**: 确保插件已正确加载
3. **查看日志**: 检查 Python 和 UXP 插件的日志输出

### 命令无响应

1. **检查连接状态**: 确保 Photoshop 已连接
2. **检查文档**: 确保 Photoshop 中有打开的文档
3. **查看错误**: 检查 UXP 插件的错误日志

## 扩展功能

### 添加新命令

1. **在 UXP 插件中添加命令处理**:
```javascript
case 'my_command':
    await myCommand(params);
    break;
```

2. **在 Python 中添加事件处理**:
```python
@bridge.on('my_event')
async def handle_my_event(data, client):
    # 处理事件
    pass
```

3. **在 UI 中调用**:
```javascript
window.aurora.emit('send_to_bridge', {
    action: 'execute_command',
    data: {
        command: 'my_command',
        params: { ... }
    }
});
```

## 下一步

- 添加图层样式编辑
- 实现图层预览
- 支持批量操作
- 添加撤销/重做功能

## 相关文档

- [服务发现文档](../../docs/SERVICE_DISCOVERY_IMPLEMENTATION.md)
- [Bridge API 文档](../../docs/BRIDGE_DESIGN.md)
- [Photoshop UXP 文档](https://developer.adobe.com/photoshop/uxp/)

