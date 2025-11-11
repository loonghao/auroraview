# Photoshop 集成步骤指南

本文档提供详细的 Photoshop 与 AuroraView 集成步骤说明。

## 目录

1. [环境准备](#环境准备)
2. [安装 UXP 开发者工具](#安装-uxp-开发者工具)
3. [配置 Rust 开发环境](#配置-rust-开发环境)
4. [部署 WebSocket 服务器](#部署-websocket-服务器)
5. [加载 UXP 插件](#加载-uxp-插件)
6. [测试集成](#测试集成)
7. [生产环境部署](#生产环境部署)
8. [常见问题](#常见问题)

## 环境准备

### 系统要求

| 组件 | 最低版本 | 推荐版本 |
|------|---------|---------|
| Adobe Photoshop | 24.0 (2023) | 26.0+ (2025) |
| Rust | 1.70 | 1.75+ |
| Node.js (可选) | 16.x | 20.x+ |
| 操作系统 | Windows 10, macOS 11 | Windows 11, macOS 14+ |

### 检查 Photoshop 版本

1. 打开 Photoshop
2. 前往 **帮助 → 关于 Photoshop**
3. 确认版本号 ≥ 24.0

## 安装 UXP 开发者工具

### 方法 1: Creative Cloud Desktop

1. 打开 **Creative Cloud Desktop**
2. 前往 **所有应用**
3. 搜索 "UXP Developer Tool"
4. 点击 **安装**

### 方法 2: 直接下载

1. 访问 [Adobe Developer Console](https://developer.adobe.com/console)
2. 下载 **UXP Developer Tool**
3. 运行安装程序

### 验证安装

```bash
# 启动 UXP Developer Tool
# Windows: 从开始菜单搜索 "UXP Developer Tool"
# macOS: 从应用程序文件夹启动
```

## 配置 Rust 开发环境

### 安装 Rust

```bash
# Windows (PowerShell)
winget install Rustlang.Rustup

# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 验证安装

```bash
rustc --version
cargo --version
```

### 安装项目依赖

```bash
cd examples/photoshop_examples
cargo build --release
```

## 部署 WebSocket 服务器

### 开发模式

```bash
# 启动开发服务器 (带日志)
RUST_LOG=info cargo run --bin websocket_server
```

### 生产模式

```bash
# 编译优化版本
cargo build --release

# 运行
./target/release/websocket_server
```

### 配置服务器端口

编辑 `websocket_server.rs`:

```rust
let addr = "127.0.0.1:9001"; // 修改端口号
```

### 验证服务器运行

你应该看到:
```
🚀 AuroraView WebSocket Server listening on: 127.0.0.1:9001
📡 Waiting for Photoshop UXP plugin to connect...
```

## 加载 UXP 插件

### 步骤 1: 打开 UXP Developer Tool

1. 启动 **UXP Developer Tool**
2. 确保 Photoshop 正在运行

### 步骤 2: 添加插件

1. 点击 **Add Plugin** 按钮
2. 选择 **manifest.json** 文件路径:
   ```
   examples/photoshop_examples/uxp_plugin/manifest.json
   ```
3. 插件应出现在列表中

### 步骤 3: 加载插件

1. 在插件列表中找到 "AuroraView Bridge"
2. 点击 **Load** 按钮
3. 状态应变为 "Loaded"

### 步骤 4: 在 Photoshop 中打开插件

1. 在 Photoshop 中,前往 **插件 → AuroraView**
2. 插件面板应该出现

## 测试集成

### 测试 1: 连接测试

1. 确认 WebSocket 服务器正在运行
2. 在插件面板中,点击 **Connect**
3. 状态应变为 "Connected" (绿色)
4. 服务器控制台应显示:
   ```
   ✅ New connection from: 127.0.0.1:xxxxx
   🔗 WebSocket connection established
   🤝 Handshake from Photoshop
   ```

### 测试 2: 图层创建

1. 在 Photoshop 中创建或打开一个文档
2. 点击 **Create New Layer** 按钮
3. 检查:
   - Photoshop 中应创建新图层
   - 插件日志显示 "Layer created successfully"
   - 服务器控制台显示 "🎨 Layer created"

### 测试 3: 文档信息

1. 点击 **Get Document Info** 按钮
2. 检查:
   - 插件日志显示 "Document info retrieved"
   - 服务器控制台显示文档详细信息

### 测试 4: 选区信息

1. 在 Photoshop 中使用选框工具创建选区
2. 点击 **Get Selection Info** 按钮
3. 检查服务器控制台显示选区边界

## 生产环境部署

### 安全配置

#### 1. 启用 WSS (安全 WebSocket)

**生成 SSL 证书**:
```bash
# 使用 OpenSSL
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

**修改服务器代码** (添加 TLS 支持):
```rust
// 需要添加 tokio-native-tls 依赖
use tokio_native_tls::TlsAcceptor;
```

**更新 manifest.json**:
```json
{
  "requiredPermissions": {
    "network": {
      "domains": ["wss://your-domain.com:9001"]
    }
  }
}
```

#### 2. 添加身份验证

在 `websocket_server.rs` 中添加 token 验证:

```rust
fn handle_photoshop_message(msg: &WsMessage, peer_map: &PeerMap, sender_addr: &SocketAddr) {
    // Verify authentication token
    if let Some(token) = msg.data.get("auth_token") {
        if !verify_token(token) {
            send_error(peer_map, sender_addr, "Invalid token");
            return;
        }
    }
    // ... rest of the logic
}
```

### 性能优化

#### 1. 消息批处理

```rust
// 批量处理消息,减少网络开销
let mut message_buffer = Vec::new();
// ... collect messages
send_batch(peer_map, sender_addr, &message_buffer);
```

#### 2. 连接池管理

```rust
// 限制最大连接数
const MAX_CONNECTIONS: usize = 100;

if peer_map.lock().unwrap().len() >= MAX_CONNECTIONS {
    eprintln!("❌ Max connections reached");
    return;
}
```

### 日志和监控

#### 配置日志级别

```bash
# 开发环境
RUST_LOG=debug cargo run

# 生产环境
RUST_LOG=info cargo run
```

#### 添加监控指标

```rust
// 记录连接统计
struct Metrics {
    total_connections: AtomicUsize,
    messages_sent: AtomicUsize,
    messages_received: AtomicUsize,
}
```

## 常见问题

### Q1: 插件无法连接到服务器

**症状**: 点击 Connect 后状态仍为 "Disconnected"

**解决方案**:
1. 检查服务器是否运行: `netstat -an | findstr 9001` (Windows) 或 `lsof -i :9001` (macOS/Linux)
2. 检查防火墙设置
3. 确认 URL 正确: `ws://localhost:9001`
4. 查看浏览器控制台错误 (UXP Developer Tool → Debug)

### Q2: 网络权限错误

**症状**: UXP 抛出 "Network access denied"

**解决方案**:
确认 `manifest.json` 包含正确的网络权限:
```json
{
  "requiredPermissions": {
    "network": {
      "domains": ["ws://localhost:*"]
    }
  }
}
```

### Q3: 插件加载失败

**症状**: UXP Developer Tool 显示 "Failed to load"

**解决方案**:
1. 验证 `manifest.json` 语法 (使用 JSON 验证器)
2. 检查 Photoshop 版本兼容性
3. 查看 UXP Developer Tool 控制台详细错误
4. 确认所有文件路径正确

### Q4: 消息发送失败

**症状**: 服务器未收到消息

**解决方案**:
1. 检查 WebSocket 连接状态
2. 验证消息格式 (必须是有效 JSON)
3. 查看插件日志
4. 使用浏览器开发者工具调试

### Q5: 性能问题

**症状**: 消息延迟或丢失

**解决方案**:
1. 启用消息批处理
2. 减少消息频率
3. 使用异步处理
4. 检查网络带宽

## 下一步

完成集成后,你可以:

1. **扩展功能**: 添加更多 Photoshop API 调用
2. **集成 AuroraView**: 连接到 AuroraView 核心模块
3. **开发 UI**: 创建更丰富的插件界面
4. **自动化工作流**: 实现批处理和自动化任务
5. **发布插件**: 准备发布到 Adobe Exchange

## 参考资源

- [Adobe UXP 官方文档](https://developer.adobe.com/photoshop/uxp/)
- [Photoshop API 参考](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/)
- [tokio-tungstenite 文档](https://docs.rs/tokio-tungstenite/)
- [WebSocket 协议规范](https://datatracker.ietf.org/doc/html/rfc6455)

## 支持

如有问题,请:
1. 查看本文档的常见问题部分
2. 查看项目 GitHub Issues
3. 联系 AuroraView 团队

