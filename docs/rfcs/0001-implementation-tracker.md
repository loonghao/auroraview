# RFC 0001: AuroraView MCP Server - Implementation Tracker

## 总体进度

| Phase | 状态 | 完成度 | 目标版本 | 预计时间 |
|-------|------|--------|----------|----------|
| Phase 1: Python SDK 核心 | 待开始 | 0% | v0.3.0 | 2 周 |
| Phase 2: Gallery 集成 | 待开始 | 0% | v0.3.1 | 1 周 |
| Phase 3: DCC 支持 | 待开始 | 0% | v0.4.0 | 2 周 |
| Phase 4: Node.js SDK | 待开始 | 0% | v0.4.1 | 2 周 |
| Phase 5: Midscene 集成 | 待开始 | 0% | v0.5.0 | 2 周 |
| Phase 6: 高级功能 | 待开始 | 0% | v0.6.0 | 2 周 |

## 详细进度

### Phase 1: Python SDK 核心功能

#### 项目结构搭建
- [ ] 创建 `packages/auroraview-mcp/` 目录
- [ ] 配置 `pyproject.toml`
- [ ] 设置开发环境
- [ ] 添加到 monorepo 工作区

#### 实例发现 (`discover_instances`)
- [ ] 设计
- [ ] 实现端口扫描
- [ ] 实现 AuroraView 特征检测
- [ ] 测试
- [ ] 文档

#### 连接管理 (`connect`, `disconnect`)
- [ ] 设计
- [ ] 实现 CDP WebSocket 连接
- [ ] 实现连接池
- [ ] 测试
- [ ] 文档

#### 页面操作 (`list_pages`, `select_page`, `get_page_info`)
- [ ] 设计
- [ ] 实现页面列表
- [ ] 实现页面选择（过滤 about:blank）
- [ ] 实现页面信息获取
- [ ] 测试
- [ ] 文档

#### 基本 API 调用 (`call_api`, `list_api_methods`)
- [ ] 设计
- [ ] 实现 JS 执行桥接
- [ ] 实现异步调用支持
- [ ] 测试
- [ ] 文档

#### 截图 (`take_screenshot`)
- [ ] 设计
- [ ] 实现 CDP 截图
- [ ] 支持元素截图
- [ ] 支持全页面截图
- [ ] 测试
- [ ] 文档

### Phase 2: Gallery 集成

#### Gallery 工具
- [ ] `get_samples` - 获取示例列表
- [ ] `run_sample` - 运行示例
- [ ] `stop_sample` - 停止示例
- [ ] `get_sample_source` - 获取源码
- [ ] `list_processes` - 列出进程

#### 资源提供者
- [ ] `auroraview://instances`
- [ ] `auroraview://page/{id}`
- [ ] `auroraview://samples`
- [ ] `auroraview://sample/{name}/source`

### Phase 3: DCC 支持

#### DCC 实例发现
- [ ] `list_dcc_instances` - 发现 DCC 实例
- [ ] Maya 适配
- [ ] Blender 适配
- [ ] Houdini 适配
- [ ] Unreal 适配（预留）

#### DCC 上下文
- [ ] `get_dcc_context` - 获取 DCC 上下文
- [ ] 场景信息
- [ ] 选择状态
- [ ] 当前帧

#### DCC 命令
- [ ] `execute_dcc_command` - 执行 DCC 命令
- [ ] Maya cmds 支持
- [ ] Blender bpy 支持
- [ ] Houdini hou 支持

#### 选择同步
- [ ] `sync_selection` - 同步选择状态

### Phase 4: Node.js SDK

#### 基础设施
- [ ] 创建 `packages/auroraview-mcp-node/` 目录
- [ ] 配置 `package.json` 和 `tsconfig.json`
- [ ] 设置构建流程

#### MCP Server
- [ ] 实现 TypeScript MCP Server
- [ ] CDP 连接管理
- [ ] 工具注册

#### 核心工具移植
- [ ] 发现工具
- [ ] 页面工具
- [ ] API 工具
- [ ] UI 工具

#### 类型定义
- [ ] 完整 TypeScript 类型
- [ ] 导出类型声明

### Phase 5: Midscene 集成

#### Midscene Agent
- [ ] `AuroraViewMidsceneAgent` 类
- [ ] 上下文增强
- [ ] 错误处理

#### AI 工具
- [ ] `ai_act` - 自然语言 UI 操作
- [ ] `ai_query` - 数据提取
- [ ] `ai_assert` - 断言
- [ ] `ai_wait_for` - 等待条件

#### DCC AI 工具
- [ ] `ai_dcc_action` - DCC 环境 AI 操作
- [ ] DCC 上下文注入

#### 提示模板
- [ ] AuroraView 通用模板
- [ ] Gallery 专用模板
- [ ] DCC 专用模板

### Phase 6: 高级功能

#### 调试工具
- [ ] `get_console_logs` - 控制台日志
- [ ] `get_network_requests` - 网络请求
- [ ] `get_backend_status` - 后端状态
- [ ] `reload_page` - 重载页面

#### UI 交互
- [ ] `get_snapshot` - 页面快照
- [ ] `click` - 点击元素
- [ ] `fill` - 填充输入
- [ ] `evaluate` - 执行 JS

#### 事件系统
- [ ] `emit_event` - 触发事件

#### 高级功能
- [ ] 多实例管理
- [ ] 性能监控工具
- [ ] SSE 传输支持
- [ ] 提示模板 (Prompts)

## 测试计划

### 单元测试
- [ ] `test_discovery.py` - 实例发现测试
- [ ] `test_connection.py` - 连接管理测试
- [ ] `test_api_bridge.py` - API 桥接测试
- [ ] `test_tools.py` - 工具函数测试
- [ ] `test_dcc.py` - DCC 工具测试

### 集成测试
- [ ] 与 Gallery 集成测试
- [ ] 与 Claude Desktop 集成测试
- [ ] 与 CodeBuddy 集成测试
- [ ] Maya 集成测试
- [ ] Blender 集成测试

### E2E 测试
- [ ] 完整工作流测试
- [ ] DCC 工作流测试
- [ ] Midscene AI 测试
- [ ] 错误处理测试
- [ ] 性能测试

## 文档更新

### Python SDK
- [ ] README.md
- [ ] 安装指南
- [ ] 配置指南
- [ ] API 参考
- [ ] 使用示例
- [ ] 故障排除

### Node.js SDK
- [ ] README.md
- [ ] 安装指南
- [ ] Midscene 集成指南
- [ ] API 参考
- [ ] TypeScript 类型文档

### DCC 集成
- [ ] Maya 集成指南
- [ ] Blender 集成指南
- [ ] Houdini 集成指南
- [ ] DCC 最佳实践

## 里程碑

| 里程碑 | 目标日期 | 状态 |
|--------|----------|------|
| Python SDK Alpha | TBD | 待开始 |
| Gallery 集成完成 | TBD | 待开始 |
| DCC 支持 Beta | TBD | 待开始 |
| Node.js SDK Alpha | TBD | 待开始 |
| Midscene 集成完成 | TBD | 待开始 |
| v1.0 稳定版 | TBD | 待开始 |

## 更新日志

| 日期 | 变更 |
|------|------|
| 2024-12-30 | 创建跟踪文档 |
| 2024-12-30 | 添加 DCC、Node.js、Midscene 阶段 |
