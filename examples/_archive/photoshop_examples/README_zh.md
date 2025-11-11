# Photoshop 集成示例

[![English Docs](https://img.shields.io/badge/docs-English-blue)](./README.md)

本示例演示了如何使用 WebSocket 协议实现 Adobe Photoshop 与 AuroraView 之间的双向通信。

## 架构设计

```
┌─────────────────────────────────────────────────────────┐
│              Adobe Photoshop 2025+                      │
│                                                         │
│  ┌───────────────────────────────────────────────┐    │
│  │         UXP 插件 (JavaScript)                 │    │
│  │  - WebSocket 客户端                           │    │
│  │  - Photoshop 图像 API                         │    │
│  │  - 图层管理                                   │    │
│  └──────────────────┬────────────────────────────┘    │
└─────────────────────┼──────────────────────────────────┘
                      │
                      │ WebSocket (ws://localhost:9001)
                      │
┌─────────────────────▼──────────────────────────────────┐
│         Rust WebSocket 服务器                          │
│  - tokio-tungstenite                                   │
│  - 消息路由                                            │
│  - 多客户端支持                                        │
└─────────────────────┬──────────────────────────────────┘
                      │
                      │ IPC / API
                      │
┌─────────────────────▼──────────────────────────────────┐
│              AuroraView 核心                           │
│  - DCC 集成                                            │
│  - 资产管理                                            │
└────────────────────────────────────────────────────────┘
```

## 功能特性

- ✅ 实时双向通信
- ✅ 图层创建和管理
- ✅ 选区信息获取
- ✅ 文档元数据访问
- ✅ 自动重连机制
- ✅ 多客户端广播支持

## 环境要求

### Photoshop 端
- Adobe Photoshop 2024 或更高版本 (v24.0+)
- [UXP 开发者工具](https://developer.adobe.com/photoshop/uxp/2022/guides/devtool/)

### 服务器端
- Rust 1.70+ 及 Cargo
- tokio 运行时

## 快速开始

### 1. 启动 WebSocket 服务器

```bash
cd examples/photoshop_examples
cargo run --bin websocket_server
```

你应该看到:
```
🚀 AuroraView WebSocket Server listening on: 127.0.0.1:9001
📡 Waiting for Photoshop UXP plugin to connect...
```

### 2. 在 Photoshop 中加载 UXP 插件

1. 打开 **UXP Developer Tool**
2. 点击 **Add Plugin**
3. 导航到 `examples/photoshop_examples/uxp_plugin/manifest.json`
4. 点击 **Load**
5. 在 Photoshop 中,前往 **插件 → AuroraView**

### 3. 连接插件到服务器

1. 在 AuroraView 面板中,确认服务器 URL: `ws://localhost:9001`
2. 点击 **Connect** 按钮
3. 状态应变为 "Connected" (绿色)

### 4. 测试通信

点击任意操作按钮:
- **Create New Layer**: 创建图层并发送信息到服务器
- **Get Selection Info**: 获取当前选区边界
- **Get Document Info**: 发送文档元数据

查看服务器控制台接收到的消息!

## 消息协议

### 消息格式

所有消息使用 JSON 格式:

```json
{
  "type": "request|response|event",
  "id": "unique-message-id",
  "action": "action_name",
  "data": {
    // 特定操作的数据载荷
  },
  "timestamp": 1704067200000
}
```

### 支持的操作

#### Photoshop → 服务器

| 操作 | 描述 | 数据 |
|------|------|------|
| `handshake` | 初始连接握手 | `{ client, version, app, appVersion }` |
| `layer_created` | 图层创建事件 | `{ name, id }` |
| `selection_info` | 选区数据 | `{ hasSelection, bounds, documentName }` |
| `document_info` | 文档元数据 | `{ name, width, height, resolution, colorMode }` |
| `command_result` | 命令执行结果 | `{ command, success, result/error }` |

#### 服务器 → Photoshop

| 操作 | 描述 | 数据 |
|------|------|------|
| `handshake_ack` | 握手确认 | `{ server, version, status }` |
| `execute_command` | 执行 Photoshop 命令 | `{ command, params }` |

## 项目结构

```
photoshop_examples/
├── Cargo.toml                 # Rust 依赖配置
├── websocket_server.rs        # WebSocket 服务器实现
├── README.md                  # 英文文档
├── README_zh.md              # 本文件
└── uxp_plugin/               # Photoshop UXP 插件
    ├── manifest.json         # 插件清单 (v5)
    ├── index.html            # 插件 UI
    └── index.js              # 插件逻辑
```

## 开发指南

### 修改服务器逻辑

编辑 `websocket_server.rs` 并自定义 `handle_photoshop_message` 函数:

```rust
fn handle_photoshop_message(msg: &WsMessage, peer_map: &PeerMap, sender_addr: &SocketAddr) {
    match msg.action.as_str() {
        "your_custom_action" => {
            // 你的自定义逻辑
        }
        _ => {}
    }
}
```

### 添加新的 Photoshop 操作

编辑 `uxp_plugin/index.js`:

```javascript
async function yourCustomAction() {
    try {
        // 使用 Photoshop API
        const result = await app.batchPlay([...], {});
        
        // 发送到服务器
        sendMessage('your_custom_action', { result });
    } catch (error) {
        log(`错误: ${error.message}`, 'error');
    }
}
```

## 故障排除

### 连接失败

**问题**: 插件显示 "Disconnected" 状态

**解决方案**:
1. 确认服务器正在运行: `cargo run --bin websocket_server`
2. 检查防火墙设置 (允许端口 9001)
3. 确保 URL 是 `ws://localhost:9001` (不是 `wss://`)

### 网络权限错误

**问题**: UXP 抛出网络权限错误

**解决方案**: 确认 `manifest.json` 包含:
```json
{
  "requiredPermissions": {
    "network": {
      "domains": ["ws://localhost:*"]
    }
  }
}
```

### 插件无法加载

**问题**: UXP Developer Tool 显示错误

**解决方案**:
1. 检查 Photoshop 版本 (必须是 24.0+)
2. 验证 `manifest.json` 语法
3. 查看 UXP Developer Tool 控制台错误信息

## 下一步计划

- [ ] 实现安全 WebSocket (wss://)
- [ ] 添加身份验证机制
- [ ] 与 AuroraView 核心集成
- [ ] 添加批处理操作支持
- [ ] 实现资产导出功能

## 参考资料

- [Adobe UXP 文档](https://developer.adobe.com/photoshop/uxp/)
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/)
- [Photoshop Imaging API](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/)

## 许可证

本示例是 AuroraView 项目的一部分。

