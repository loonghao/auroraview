# AuroraView Bridge 设计方案

## 📋 概述

将 WebSocket Bridge 功能内置到 AuroraView 框架核心,提供通用的 DCC 工具和 Web 应用集成能力。

## 🎯 设计目标

1. **易用性**: 开发者用几行代码即可启动 Bridge
2. **通用性**: 支持任何 WebSocket 客户端 (Photoshop, Maya, Blender, Web 应用等)
3. **集成性**: 与 WebView 深度集成,自动双向通信
4. **可扩展性**: 支持自定义消息协议和处理器
5. **向后兼容**: 不破坏现有 API

## 🏗️ 架构设计

### 核心组件

```
┌─────────────────────────────────────────────────────────┐
│                  AuroraView Framework                   │
│                                                         │
│  ┌──────────────┐         ┌──────────────┐            │
│  │   WebView    │◄────────┤    Bridge    │            │
│  │              │         │              │            │
│  │  - UI Layer  │         │  - WebSocket │            │
│  │  - IPC       │         │  - Routing   │            │
│  └──────────────┘         │  - Handlers  │            │
│                           └──────┬───────┘            │
└──────────────────────────────────┼──────────────────────┘
                                   │ WebSocket
                    ┌──────────────┼──────────────┐
                    │              │              │
              ┌─────▼─────┐  ┌────▼─────┐  ┌────▼─────┐
              │ Photoshop │  │   Maya   │  │   Web    │
              │    UXP    │  │  Script  │  │   App    │
              └───────────┘  └──────────┘  └──────────┘
```

### 模块划分

1. **`bridge.py`**: 核心 Bridge 类
   - WebSocket 服务器
   - 消息路由
   - 连接管理
   - 错误处理

2. **`webview.py`**: 扩展 WebView 类
   - 添加 `bridge` 参数
   - 自动绑定 Bridge 事件到 WebView
   - 提供便捷方法

3. **`__init__.py`**: 导出 Bridge 类
   - 添加到 `__all__`
   - 保持向后兼容

## 💡 API 设计

### 理想的使用方式

```python
from auroraview import WebView, Bridge

# 方式 1: 装饰器风格 (推荐)
bridge = Bridge(port=9001)

@bridge.on('layer_created')
async def handle_layer(data, client):
    print(f"Layer created: {data}")
    return {"status": "ok"}

@bridge.on('handshake')
async def handle_handshake(data, client):
    print(f"Client connected: {data}")
    return {"server": "auroraview", "version": "1.0.0"}

# 创建 WebView 并关联 Bridge
webview = WebView.create(
    title="My Tool",
    url="http://localhost:5173",
    bridge=bridge  # 自动绑定
)

# 启动 (自动启动 Bridge 和 WebView)
webview.show()
```

```python
# 方式 2: 手动注册
bridge = Bridge(port=9001)

async def handle_layer(data, client):
    return {"status": "ok"}

bridge.register_handler('layer_created', handle_layer)

webview = WebView.create("My Tool", bridge=bridge)
webview.show()
```

```python
# 方式 3: 内联 Lambda
bridge = Bridge(port=9001)

bridge.on('ping')(lambda data, client: {"pong": True})

webview = WebView.create("My Tool", bridge=bridge)
webview.show()
```

```python
# 方式 4: WebView 快捷方式
webview = WebView.create(
    title="Photoshop Tool",
    url="http://localhost:5173",
    bridge=True  # 自动创建 Bridge (默认端口 9001)
)

# 通过 webview.bridge 访问
@webview.bridge.on('layer_created')
async def handle_layer(data, client):
    print(f"Layer: {data}")
```

### Bridge 类 API

```python
class Bridge:
    """WebSocket Bridge for DCC and Web application integration."""
    
    def __init__(
        self,
        host: str = "localhost",
        port: int = 9001,
        *,
        auto_start: bool = False,
        protocol: str = "json",  # 'json' or 'msgpack'
    ):
        """Initialize Bridge.
        
        Args:
            host: WebSocket server host
            port: WebSocket server port
            auto_start: Auto-start server on creation
            protocol: Message protocol ('json' or 'msgpack')
        """
        
    def on(self, action: str) -> Callable:
        """Decorator to register message handler.
        
        Args:
            action: Action name (e.g., 'layer_created')
            
        Returns:
            Decorator function
            
        Example:
            @bridge.on('layer_created')
            async def handle_layer(data, client):
                return {"status": "ok"}
        """
        
    def register_handler(self, action: str, handler: Callable):
        """Register message handler.
        
        Args:
            action: Action name
            handler: Async function(data, client) -> response
        """
        
    async def start(self):
        """Start WebSocket server."""
        
    async def stop(self):
        """Stop WebSocket server."""
        
    async def send(self, client, data: Dict[str, Any]):
        """Send message to specific client."""
        
    async def broadcast(self, data: Dict[str, Any]):
        """Broadcast message to all clients."""
        
    def execute_command(self, command: str, params: Dict = None):
        """Send command to all clients (non-blocking)."""
        
    @property
    def clients(self) -> Set:
        """Get connected clients."""
        
    @property
    def is_running(self) -> bool:
        """Check if server is running."""
```

### WebView 集成

```python
class WebView:
    def __init__(
        self,
        ...,
        bridge: Union[Bridge, bool, None] = None,
    ):
        """
        Args:
            bridge: Bridge instance, True (auto-create), or None
        """
        
    @property
    def bridge(self) -> Optional[Bridge]:
        """Get associated Bridge instance."""
        
    def send_to_bridge(self, action: str, data: Dict):
        """Send message to Bridge clients."""
```

## 🔄 自动双向通信

### Bridge → WebView

```python
# Bridge 收到消息后自动通知 WebView
@bridge.on('layer_created')
async def handle_layer(data, client):
    # 处理完成后,自动触发 WebView 事件
    # webview.emit('bridge:layer_created', data)
    return {"status": "ok"}
```

### WebView → Bridge

```python
# JavaScript 调用 Python,Python 转发到 Bridge
// JavaScript
window.auroraview.call('send_to_photoshop', {
    command: 'create_layer',
    params: {name: 'New Layer'}
});

# Python (自动绑定)
@webview.on('send_to_photoshop')
def handle_send(data):
    webview.bridge.execute_command(
        data['command'],
        data['params']
    )
```

## 📦 实现计划

### Phase 1: 核心 Bridge 类
- [ ] 创建 `python/auroraview/bridge.py`
- [ ] 实现 WebSocket 服务器
- [ ] 实现消息路由系统
- [ ] 实现装饰器 API
- [ ] 添加连接管理
- [ ] 添加错误处理和日志

### Phase 2: WebView 集成
- [ ] 扩展 `WebView.__init__` 添加 `bridge` 参数
- [ ] 实现自动启动 Bridge
- [ ] 实现双向事件绑定
- [ ] 添加便捷方法

### Phase 3: 文档和示例
- [ ] 更新 Photoshop 示例使用新 API
- [ ] 创建 API 文档
- [ ] 创建使用指南
- [ ] 添加单元测试

### Phase 4: 高级功能 (可选)
- [ ] 支持 MessagePack 协议
- [ ] 支持 SSL/TLS (WSS)
- [ ] 支持认证机制
- [ ] 支持消息压缩
- [ ] 性能优化

## 🎯 向后兼容性

- 现有 `WebView` API 完全不变
- `bridge` 参数为可选,默认 `None`
- 不影响现有代码

## 📊 对比现有实现

| 特性 | 现有 (photoshop_bridge.py) | 新设计 (内置 Bridge) |
|------|---------------------------|---------------------|
| 位置 | 示例代码 | 框架核心 |
| 使用 | 手动创建和管理 | 自动集成 |
| API | 基础类方法 | 装饰器 + 便捷方法 |
| WebView 集成 | 手动绑定 | 自动双向通信 |
| 代码量 | ~200 行 | ~300 行 (含集成) |
| 复用性 | 需要复制代码 | 直接导入使用 |

## ✅ 优势

1. **开发效率**: 从 ~50 行代码减少到 ~10 行
2. **代码复用**: 不需要为每个项目复制 Bridge 代码
3. **统一体验**: 所有 DCC 集成使用相同 API
4. **自动化**: Bridge 和 WebView 自动协同工作
5. **可维护性**: 框架统一维护,bug 修复惠及所有用户

## 🚀 下一步

1. 实现核心 `Bridge` 类
2. 集成到 `WebView`
3. 更新示例代码
4. 编写文档和测试

