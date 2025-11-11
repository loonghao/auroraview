# 🔌 端口配置说明

## 问题

UXP 插件硬编码了 WebSocket 地址 `ws://localhost:9001`，但 Python 脚本使用了自动端口分配 (`port=0`)，导致连接失败。

---

## ✅ 解决方案

### 已修复

`photoshop_layers_tool.py` 现在使用**固定端口 9001**：

```python
self.bridge = Bridge(
    port=9001,                 # 固定端口（UXP 插件期望 9001）
    service_discovery=True,    # 启用服务发现
    discovery_port=9000,       # HTTP 发现端点
    enable_mdns=False,         # 禁用 mDNS
)
```

---

## 🔍 端口配置

### Bridge WebSocket 端口: 9001

**用途**: UXP 插件连接到 Python Bridge

**配置位置**:
- Python: `photoshop_layers_tool.py` → `Bridge(port=9001)`
- UXP: `index.js` → `new WebSocket('ws://localhost:9001')`
- Manifest: `manifest.json` → `"ws://localhost:9001"`

### HTTP 发现端口: 9000

**用途**: 服务发现 HTTP 端点（未来扩展用）

**配置位置**:
- Python: `photoshop_layers_tool.py` → `Bridge(discovery_port=9000)`

---

## 🔧 如果需要更改端口

### 场景 1: 端口 9001 被占用

如果端口 9001 被其他应用占用，需要同时修改 3 个地方：

#### 1. Python 脚本

**文件**: `photoshop_layers_tool.py`

```python
self.bridge = Bridge(
    port=9002,  # 改为 9002
    ...
)
```

#### 2. UXP 插件代码

**文件**: `examples/photoshop_auroraview/uxp_plugin/index.js`

```javascript
socket = new WebSocket('ws://localhost:9002');  // 改为 9002
```

#### 3. UXP 插件权限

**文件**: `examples/photoshop_auroraview/uxp_plugin/manifest.json`

```json
"requiredPermissions": {
  "network": {
    "domains": [
      "ws://localhost:9002",   // 改为 9002
      "wss://localhost:9002",  // 改为 9002
      "http://localhost:9000"
    ]
  }
}
```

#### 4. 重新加载插件

**重要**: 修改 `manifest.json` 后必须重新加载插件！

1. 在 UXP Developer Tool 中移除插件
2. 重新加载插件

---

## 🎯 推荐配置

### 开发环境

使用**固定端口**，便于调试：

```python
Bridge(port=9001)  # 固定端口
```

### 生产环境

如果需要支持多个实例，可以使用**动态端口 + 服务发现**：

```python
Bridge(
    port=0,                    # 自动分配
    service_discovery=True,    # 启用服务发现
)
```

然后 UXP 插件通过 HTTP 发现端点获取实际端口：

```javascript
// 1. 查询服务发现端点
const response = await fetch('http://localhost:9000/discover');
const data = await response.json();

// 2. 使用返回的端口连接
const socket = new WebSocket(`ws://localhost:${data.port}`);
```

---

## 📊 端口使用总结

| 端口 | 用途 | 协议 | 配置位置 |
|------|------|------|----------|
| 9001 | Bridge WebSocket | ws:// | Python + UXP |
| 9000 | HTTP 发现端点 | http:// | Python |

---

## 🔍 验证端口

### 检查端口是否被占用

**Windows PowerShell**:
```powershell
# 检查端口 9001
Get-NetTCPConnection -LocalPort 9001 -ErrorAction SilentlyContinue

# 如果有输出，说明端口被占用
```

**查看占用端口的进程**:
```powershell
netstat -ano | findstr :9001
```

### 测试连接

**在 UXP Developer Tool Console 中**:
```javascript
// 测试 WebSocket 连接
const ws = new WebSocket('ws://localhost:9001');
ws.onopen = () => console.log('✅ Connected to port 9001');
ws.onerror = (e) => console.error('❌ Connection failed:', e);
```

---

## 🚨 常见问题

### 问题 1: "Connection refused"

**原因**: Python Bridge 没有运行

**解决方案**:
```bash
python examples/photoshop_layers_demo/photoshop_layers_tool.py
```

### 问题 2: "Permission denied"

**原因**: manifest.json 中的端口配置不正确

**解决方案**:
1. 检查 `manifest.json` 中的端口号
2. 移除并重新加载插件

### 问题 3: "Port already in use"

**原因**: 端口 9001 被其他应用占用

**解决方案**:
1. 关闭占用端口的应用
2. 或者更改端口（见上方"如果需要更改端口"）

---

## 📖 相关文档

- [快速开始](QUICKSTART.md)
- [UXP 插件安装](../photoshop_auroraview/uxp_plugin/INSTALL_V2.md)
- [故障排除](../photoshop_auroraview/uxp_plugin/TROUBLESHOOTING.md)

---

**现在端口配置已修复，应该可以正常连接了！** 🚀

