# 服务发现示例

这个示例展示了 AuroraView 的新服务发现功能，解决了 WebSocket Bridge 的端口冲突问题。

## 功能特性

### 1. 动态端口分配 ✅
- **自动查找可用端口**：不再需要手动指定端口
- **避免端口冲突**：支持多个 Bridge 实例同时运行
- **简单易用**：只需设置 `port=0`

### 2. HTTP 发现端点 ✅
- **固定端口**：默认在 9000 端口提供 HTTP API
- **UXP 兼容**：Adobe Photoshop 插件可以通过 HTTP 发现 Bridge
- **CORS 支持**：允许跨域访问

### 3. mDNS 服务发现 ✅
- **Zeroconf/Bonjour**：自动服务广播和发现
- **DCC 工具集成**：Maya、Blender 等可以自动发现服务
- **跨平台**：Windows/macOS/Linux 统一支持

## 快速开始

### 运行示例

```bash
# 1. 确保已安装依赖
pip install websockets

# 2. 重新编译 Rust 扩展（包含服务发现模块）
pip install -e .

# 3. 运行示例
python examples/service_discovery_demo/bridge_with_discovery.py
```

### 预期输出

```
================================================================================
Service Discovery Demo
================================================================================

✅ Bridge created with auto-allocated port: 9001
📡 HTTP discovery endpoint: http://localhost:9000/discover
🔍 mDNS service: _auroraview._tcp.local.

🎉 Starting WebView and Bridge...
Try the buttons in the UI to test service discovery!
```

## 使用方法

### 基础用法

```python
from auroraview import Bridge

# 自动分配端口 + 服务发现
bridge = Bridge(
    port=0,                    # 0 = 自动分配
    service_discovery=True,    # 启用服务发现
    discovery_port=9000,       # HTTP 发现端点
    enable_mdns=True,          # 启用 mDNS
)

print(f"Bridge running on port: {bridge.port}")
```

### HTTP 发现 API

**端点**: `GET http://localhost:9000/discover`

**响应**:
```json
{
  "service": "AuroraView Bridge",
  "port": 9001,
  "protocol": "websocket",
  "version": "0.2.3",
  "timestamp": 1234567890
}
```

### UXP 插件集成

```javascript
// Photoshop UXP 插件
async function connectToBridge() {
    // 1. 发现 Bridge 端口
    const response = await fetch('http://localhost:9000/discover');
    const info = await response.json();
    
    console.log(`Found Bridge at port ${info.port}`);
    
    // 2. 连接 WebSocket
    const ws = new WebSocket(`ws://localhost:${info.port}`);
    
    ws.onopen = () => {
        console.log('Connected to AuroraView Bridge');
    };
    
    return ws;
}
```

### Maya/Blender 集成 (mDNS)

```python
# Maya/Blender Python 脚本
from auroraview import ServiceDiscovery

# 发现 AuroraView 服务
discovery = ServiceDiscovery()
services = discovery.discover_services(timeout_secs=5)

if services:
    service = services[0]
    print(f"Found: {service.name} at {service.host}:{service.port}")
    
    # 连接 WebSocket
    import websocket
    ws = websocket.create_connection(f"ws://{service.host}:{service.port}")
    ws.send('{"action": "handshake"}')
```

## 架构说明

### 通信流程

```
┌─────────────────────────────────────────────────────────────┐
│                    服务发现架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  UXP 插件              HTTP 发现           Bridge            │
│  (Photoshop)          (端口 9000)        (动态端口)          │
│                                                              │
│  ┌──────────┐         ┌──────────┐       ┌──────────┐      │
│  │ fetch()  │────1───>│ /discover│       │ WebSocket│      │
│  │          │<───2────│ endpoint │       │ Server   │      │
│  │          │         └──────────┘       │          │      │
│  │          │                             │ Port:    │      │
│  │ ws.send()│────3───────────────────────>│ 9001     │      │
│  └──────────┘                             └──────────┘      │
│                                                              │
│  DCC 工具              mDNS 发现           Bridge            │
│  (Maya/Blender)       (Zeroconf)         (动态端口)          │
│                                                              │
│  ┌──────────┐         ┌──────────┐       ┌──────────┐      │
│  │ discover │────1───>│ mDNS     │<──────│ Register │      │
│  │ services │<───2────│ Daemon   │       │ Service  │      │
│  │          │         └──────────┘       │          │      │
│  │ ws.send()│────3───────────────────────>│ Port:    │      │
│  └──────────┘                             │ 9002     │      │
└─────────────────────────────────────────────────────────────┘
```

### 端口分配策略

1. **Bridge 端口**：从 9001 开始查找可用端口
2. **HTTP 发现端口**：固定 9000（可配置）
3. **mDNS**：无需端口（使用多播 DNS）

## 测试

### 测试 HTTP 发现

```bash
# 使用 curl
curl http://localhost:9000/discover

# 使用 PowerShell
Invoke-WebRequest -Uri http://localhost:9000/discover | ConvertFrom-Json
```

### 测试 WebSocket

```javascript
// 浏览器控制台
const ws = new WebSocket('ws://localhost:9001');
ws.onopen = () => {
    ws.send(JSON.stringify({action: 'ping', timestamp: Date.now()}));
};
ws.onmessage = (e) => console.log('Response:', JSON.parse(e.data));
```

### 测试 mDNS

```python
from auroraview import ServiceDiscovery

discovery = ServiceDiscovery(enable_mdns=True)
services = discovery.discover_services(timeout_secs=5)

for service in services:
    print(f"Found: {service.name} at {service.host}:{service.port}")
```

## 故障排除

### 端口冲突

如果 HTTP 发现端口（9000）被占用：

```python
bridge = Bridge(
    port=0,
    service_discovery=True,
    discovery_port=9100,  # 使用其他端口
)
```

### mDNS 不工作

1. **Windows**: 确保 Bonjour 服务已安装
2. **Linux**: 确保 Avahi 守护进程正在运行
3. **macOS**: 内置支持，无需额外配置

### 防火墙问题

确保允许以下端口：
- **9000**: HTTP 发现端点
- **9001+**: Bridge WebSocket 端口（动态分配）
- **5353**: mDNS 多播端口

## 下一步

- 查看 [Photoshop 集成示例](../photoshop_auroraview/)
- 阅读 [服务发现设计文档](../../docs/SERVICE_DISCOVERY_DESIGN.md)
- 探索 [Bridge API 文档](../../docs/BRIDGE_DESIGN.md)

