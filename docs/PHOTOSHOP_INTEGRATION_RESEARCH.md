# Adobe Photoshop 集成技术调研报告

## 执行摘要

本报告评估了 Adobe Photoshop 与 AuroraView 项目的集成可能性。基于 2025 年最新技术栈,推荐使用 **UXP (Unified Extensibility Platform) + WebSocket** 方案实现双向通信。

## 1. 技术背景

### 1.1 Adobe 扩展平台演进

| 平台 | 状态 | 技术栈 | 适用场景 |
|------|------|--------|---------|
| **CEP** (Common Extensibility Platform) | 🔴 即将废弃 | Chromium + ExtendScript | 遗留项目 |
| **UXP** (Unified Extensibility Platform) | ✅ 当前主流 (2025) | 现代 JavaScript + 原生 API | 新项目推荐 |
| **Generator** | ⚠️ 特定用途 | Node.js + WebSocket | 实时资产生成 |

### 1.2 UXP 核心优势

- **轻量级**: 不使用完整 Chromium,资源占用低
- **直接通信**: 无需 ExtendScript 中间层
- **现代 JavaScript**: 支持 ES6+ 特性
- **网络支持**: 原生支持 WebSocket、fetch、XHR
- **跨应用**: Photoshop、InDesign、XD 统一平台

## 2. 技术方案对比

### 方案 A: UXP + WebSocket (推荐)

**架构**:
```
Photoshop UXP Plugin <--WebSocket--> AuroraView Rust Server <--> DCC Tools
```

**优势**:
- ✅ 2025 年官方推荐技术栈
- ✅ 双向实时通信
- ✅ 支持跨域通信
- ✅ 易于调试和维护
- ✅ 与 AuroraView 现有架构兼容

**劣势**:
- ⚠️ 需要在 manifest.json 配置网络权限
- ⚠️ 用户需要手动安装 UXP 插件

**可行性**: ⭐⭐⭐⭐⭐ (强烈推荐)

### 方案 B: CEP + WebSocket (不推荐)

**架构**:
```
Photoshop CEP Extension <--WebSocket--> AuroraView Rust Server
```

**优势**:
- ✅ 成熟的技术栈
- ✅ 丰富的社区资源

**劣势**:
- 🔴 Adobe 官方即将废弃
- 🔴 资源占用高 (完整 Chromium)
- 🔴 需要 ExtendScript 中间层
- 🔴 不适合长期维护

**可行性**: ⭐⭐ (不推荐新项目)

### 方案 C: Generator Plugin (特定场景)

**架构**:
```
Photoshop Generator <--IPC/WebSocket--> Node.js Server <--> AuroraView
```

**优势**:
- ✅ 专为资产生成优化
- ✅ 支持 WebSocket

**劣势**:
- ⚠️ 功能受限,主要用于资产导出
- ⚠️ 需要额外的 Node.js 层
- ⚠️ 文档较少

**可行性**: ⭐⭐⭐ (特定场景)

## 3. 推荐方案详细设计

### 3.1 技术栈

**Photoshop 端**:
- UXP Plugin (Manifest v5)
- JavaScript ES6+
- WebSocket API
- Photoshop Imaging API

**AuroraView 端**:
- Rust WebSocket Server (tokio-tungstenite)
- JSON 消息协议
- 异步事件处理

### 3.2 通信协议

**消息格式** (JSON):
```json
{
  "type": "request|response|event",
  "id": "unique-message-id",
  "action": "create_layer|get_selection|export_image",
  "data": {
    // Action-specific payload
  },
  "timestamp": 1704067200000
}
```

### 3.3 网络权限配置

**manifest.json** (UXP Plugin):
```json
{
  "manifestVersion": 5,
  "id": "com.auroraview.photoshop",
  "name": "AuroraView Bridge",
  "version": "1.0.0",
  "requiredPermissions": {
    "network": {
      "domains": [
        "ws://localhost:*",
        "wss://localhost:*"
      ]
    }
  }
}
```

## 4. 关键技术点

### 4.1 WebSocket 连接管理

**Photoshop UXP 端**:
- 支持原生 WebSocket API
- 需要处理重连逻辑
- 支持 wss:// (安全连接)

**Rust 服务器端**:
- 使用 `tokio-tungstenite` 库
- 支持多客户端连接
- 消息广播和路由

### 4.2 安全性考虑

- ✅ 使用 wss:// 加密连接 (生产环境)
- ✅ Token 认证机制
- ✅ 域名白名单限制
- ✅ 消息签名验证

### 4.3 错误处理

- 连接断开自动重连
- 消息超时处理
- 错误日志记录
- 用户友好的错误提示

## 5. 实现路线图

### Phase 1: POC 验证 (1-2 周)
- [ ] 创建基础 UXP 插件
- [ ] 实现 Rust WebSocket 服务器
- [ ] 验证双向通信
- [ ] 测试基本图层操作

### Phase 2: 核心功能 (2-3 周)
- [ ] 完整的消息协议
- [ ] 图层创建/修改/删除
- [ ] 选区操作
- [ ] 文档信息获取

### Phase 3: 高级功能 (2-3 周)
- [ ] 图像导出
- [ ] 批处理操作
- [ ] 插件 UI 界面
- [ ] 性能优化

### Phase 4: 生产就绪 (1-2 周)
- [ ] 安全加固
- [ ] 错误处理完善
- [ ] 用户文档
- [ ] 自动化测试

## 6. 参考资料

### 官方文档
- [Adobe UXP for Photoshop](https://developer.adobe.com/photoshop/uxp/)
- [UXP Network APIs](https://developer.adobe.com/indesign/uxp/resources/recipes/network/)
- [UXP Manifest v5](https://developer.adobe.com/photoshop/uxp/2022/guides/uxp_for_you/uxp_for_cep_devs/)

### Rust 库
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/) - WebSocket 库
- [serde_json](https://docs.rs/serde_json/) - JSON 序列化

### 社区资源
- [Adobe CEP Samples](https://github.com/Adobe-CEP/Samples) - WebSocket 示例
- [UXP Developer Tool](https://developer.adobe.com/photoshop/uxp/2022/guides/devtool/)

## 7. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| UXP API 变更 | 中 | 关注官方更新,版本锁定 |
| 网络连接不稳定 | 中 | 实现重连机制,离线队列 |
| 性能瓶颈 | 低 | 异步处理,消息批处理 |
| 用户安装复杂 | 中 | 提供详细文档,自动化脚本 |

## 8. 结论

**推荐方案**: UXP + WebSocket

**理由**:
1. 符合 Adobe 2025 年技术方向
2. 与 AuroraView 架构完美契合
3. 性能优秀,资源占用低
4. 易于维护和扩展
5. 社区支持良好

**下一步行动**:
1. 创建 POC 示例代码
2. 验证核心功能可行性
3. 编写集成文档
4. 开始 Phase 1 开发

