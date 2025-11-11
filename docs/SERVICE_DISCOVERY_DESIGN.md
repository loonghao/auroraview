# 服务发现架构设计

> **状态**: ✅ 已实现 | 查看 [实现总结](SERVICE_DISCOVERY_IMPLEMENTATION.md)

## 概述

为 AuroraView 框架添加基于 Rust 的服务发现功能，解决 Bridge WebSocket 端口冲突问题，并提供自动服务发现能力。

**实现状态**: ✅ 完成
- [x] 动态端口分配
- [x] mDNS 服务注册
- [x] HTTP 发现端点
- [x] Python 绑定
- [x] Bridge 集成
- [ ] 完整文档和测试

## 目标

1. **动态端口分配**: 自动查找可用端口，避免端口冲突
2. **mDNS 服务发现**: 支持 Zeroconf/Bonjour 协议，用于 Maya/Blender 等 DCC 工具
3. **HTTP 发现端点**: 为 Adobe UXP 插件提供服务发现（UXP 不支持 mDNS）
4. **跨平台支持**: Windows/macOS/Linux 统一 API

## 架构设计

### 模块结构

```
src/
├── lib.rs                      # 注册服务发现模块
├── service_discovery/
│   ├── mod.rs                  # 模块入口
│   ├── port_allocator.rs       # 动态端口分配
│   ├── mdns_service.rs         # mDNS 服务注册/发现
│   ├── http_discovery.rs       # HTTP 发现端点
│   └── python_bindings.rs      # PyO3 Python 绑定
```

### 核心组件

#### 1. 动态端口分配器 (PortAllocator)

```rust
pub struct PortAllocator {
    start_port: u16,
    max_attempts: u16,
}

impl PortAllocator {
    pub fn find_free_port(&self) -> Result<u16, Error>;
    pub fn is_port_available(port: u16) -> bool;
}
```

**功能**:
- 从指定范围查找可用端口
- 验证端口是否被占用
- 支持自定义起始端口和最大尝试次数

#### 2. mDNS 服务 (MdnsService)

```rust
pub struct MdnsService {
    daemon: ServiceDaemon,
    service_name: String,
}

impl MdnsService {
    pub fn new() -> Result<Self, Error>;
    pub fn register(&self, port: u16, metadata: HashMap<String, String>) -> Result<(), Error>;
    pub fn unregister(&self) -> Result<(), Error>;
    pub fn discover(service_type: &str) -> Result<Vec<ServiceInfo>, Error>;
}
```

**功能**:
- 注册 `_auroraview._tcp.local.` 服务
- 广播服务信息（端口、版本、元数据）
- 发现网络中的其他 AuroraView 实例
- 自动处理服务注销

**服务类型**: `_auroraview._tcp.local.`

**元数据**:
- `version`: AuroraView 版本
- `protocol`: WebSocket 协议版本
- `app`: 应用名称（可选）

#### 3. HTTP 发现端点 (HttpDiscovery)

```rust
pub struct HttpDiscovery {
    port: u16,
    bridge_port: u16,
    server_handle: Option<JoinHandle<()>>,
}

impl HttpDiscovery {
    pub fn new(discovery_port: u16, bridge_port: u16) -> Self;
    pub async fn start(&mut self) -> Result<(), Error>;
    pub fn stop(&mut self) -> Result<(), Error>;
}
```

**功能**:
- 在固定端口（默认 9000）启动 HTTP 服务器
- 提供 `/discover` 端点返回 Bridge 信息
- 支持 CORS（允许 UXP 插件跨域访问）

**API 端点**:

```
GET http://localhost:9000/discover

Response:
{
  "service": "AuroraView Bridge",
  "port": 9001,
  "protocol": "websocket",
  "version": "0.2.3",
  "timestamp": 1234567890
}
```

#### 4. Python 绑定 (ServiceDiscovery)

```python
from auroraview import ServiceDiscovery

# 创建服务发现实例
discovery = ServiceDiscovery(
    bridge_port=0,              # 0 = 自动分配
    discovery_port=9000,        # HTTP 发现端点
    enable_mdns=True,           # 启用 mDNS
    service_name="My App"       # 服务名称
)

# 获取分配的端口
port = discovery.bridge_port
print(f"Bridge running on port: {port}")

# 启动服务发现
discovery.start()

# 发现其他服务
services = discovery.discover_services()
for service in services:
    print(f"Found: {service.name} at {service.host}:{service.port}")

# 停止服务发现
discovery.stop()
```

## 集成方案

### 与 Bridge 集成

```python
from auroraview import Bridge, ServiceDiscovery

# 方案 1: Bridge 自动创建 ServiceDiscovery
bridge = Bridge(
    port=0,                    # 自动分配端口
    service_discovery=True,    # 启用服务发现
    discovery_port=9000        # HTTP 发现端口
)

print(f"Bridge port: {bridge.port}")  # 自动分配的端口

# 方案 2: 手动创建 ServiceDiscovery
discovery = ServiceDiscovery(bridge_port=0, enable_mdns=True)
bridge = Bridge(port=discovery.bridge_port)
discovery.start()
```

### UXP 插件使用

```javascript
// Photoshop UXP 插件
async function connectToBridge() {
    try {
        // 1. 通过 HTTP 发现 Bridge 端口
        const response = await fetch('http://localhost:9000/discover');
        const info = await response.json();
        
        console.log(`Found Bridge at port ${info.port}`);
        
        // 2. 连接 WebSocket
        const ws = new WebSocket(`ws://localhost:${info.port}`);
        
        ws.onopen = () => {
            console.log('Connected to AuroraView Bridge');
        };
        
        return ws;
    } catch (error) {
        console.error('Failed to discover Bridge:', error);
    }
}
```

### Maya/Blender 使用 (mDNS)

```python
# Maya/Blender Python 脚本
from auroraview import ServiceDiscovery

# 发现 AuroraView 服务
discovery = ServiceDiscovery()
services = discovery.discover_services()

if services:
    service = services[0]
    print(f"Connecting to {service.name} at {service.host}:{service.port}")
    
    # 连接 WebSocket
    import websocket
    ws = websocket.create_connection(f"ws://{service.host}:{service.port}")
    ws.send('{"action": "handshake"}')
```

## 依赖项

### Cargo.toml 新增依赖

```toml
[dependencies]
# Service discovery
mdns-sd = "0.11"                    # Pure Rust mDNS implementation
warp = "0.3"                        # HTTP server for discovery endpoint
tokio = { version = "1.42", features = ["full"] }  # Already exists
```

## 实现计划

1. ✅ 设计架构文档
2. ⏳ 实现动态端口分配器
3. ⏳ 实现 mDNS 服务
4. ⏳ 实现 HTTP 发现端点
5. ⏳ 创建 Python 绑定
6. ⏳ 集成到 Bridge
7. ⏳ 更新示例代码
8. ⏳ 编写测试和文档

## 安全考虑

1. **端口范围限制**: 只在用户端口范围（1024-65535）查找
2. **CORS 限制**: HTTP 端点仅允许 localhost 访问
3. **mDNS 范围**: 仅在本地网络广播
4. **元数据验证**: 验证服务元数据格式

## 性能考虑

1. **端口查找**: 使用快速 TCP 连接测试，超时 100ms
2. **mDNS 缓存**: 缓存发现结果，避免重复查询
3. **HTTP 服务器**: 使用 Warp 轻量级框架，低内存占用
4. **异步 I/O**: 所有网络操作使用 Tokio 异步运行时

