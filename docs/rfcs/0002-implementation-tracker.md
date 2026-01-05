# RFC 0002: 嵌入式 MCP Server 实现跟踪

## 总体进度

| Phase | 状态 | 完成度 | 目标版本 |
|-------|------|--------|----------|
| Phase 1: 核心实现 | 待开始 | 0% | v0.5.0 |
| Phase 2: 测试框架 | 待开始 | 0% | v0.5.1 |
| Phase 3: DCC 集成 | 待开始 | 0% | v0.5.2 |
| Phase 4: 高级功能 | 待开始 | 0% | v0.6.0 |

## 详细进度

### Phase 1: 核心实现

#### MCPConfig 配置类
- [ ] 设计
- [ ] 实现
- [ ] 测试
- [ ] 文档

#### EmbeddedMCPServer 基础实现
- [ ] HTTP Server 框架
- [ ] SSE 端点实现
- [ ] JSON-RPC 消息处理
- [ ] 工具注册机制
- [ ] 资源注册机制

#### 自动 API 发现
- [ ] 从 bind_call 自动发现
- [ ] 函数签名解析
- [ ] 文档字符串提取
- [ ] 参数 Schema 生成

#### 内置工具
- [ ] take_screenshot
- [ ] evaluate
- [ ] get_page_info
- [ ] emit_event
- [ ] click
- [ ] fill
- [ ] get_snapshot
- [ ] get_console_logs

#### WebView 参数扩展
- [ ] mcp 参数
- [ ] mcp_port 参数
- [ ] mcp_name 参数
- [ ] 启动时自动启动 MCP Server

### Phase 2: 测试框架

#### MCPTestClient
- [ ] SSE 连接
- [ ] 工具调用
- [ ] 资源读取
- [ ] 事件订阅

#### pytest 集成
- [ ] fixture 设计
- [ ] 异步测试支持
- [ ] 报告生成

#### 示例测试
- [ ] 基础 API 测试
- [ ] UI 交互测试
- [ ] 事件测试

### Phase 3: DCC 集成

#### Maya 环境
- [ ] 线程安全验证
- [ ] Maya API 暴露
- [ ] 示例工具

#### Blender 环境
- [ ] 线程安全验证
- [ ] Blender API 暴露
- [ ] 示例工具

#### 其他 DCC
- [ ] Houdini
- [ ] Nuke
- [ ] 3ds Max

### Phase 4: 高级功能

#### 认证支持
- [ ] Token 认证
- [ ] CORS 配置

#### 多实例管理
- [ ] 实例发现
- [ ] 实例切换

#### 性能监控
- [ ] 内存监控
- [ ] 请求统计

## 测试计划

### 单元测试

- [ ] MCPConfig 配置解析
- [ ] 工具注册和调用
- [ ] SSE 消息格式
- [ ] JSON-RPC 处理

### 集成测试

- [ ] WebView + MCP Server 启动
- [ ] AI 助手连接
- [ ] 工具调用完整流程

### E2E 测试

- [ ] 开发调试场景
- [ ] 自动化测试场景
- [ ] DCC 环境场景

## 文档更新

- [ ] API 参考文档
- [ ] 使用指南
- [ ] 示例代码
- [ ] 迁移指南

## 更新日志

| 日期 | 变更 |
|------|------|
| 2025-12-31 | 创建跟踪文档 |
