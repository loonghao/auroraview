调试 auroraview-dev MCP 服务器问题。

## 调试步骤

1. **检查 MCP 服务器状态**
   - 尝试调用任意 MCP 工具 (如 `get_project_info`)
   - 如果失败，说明 MCP 服务器未正确启动

2. **手动测试 MCP 服务器**
   ```bash
   cd packages/auroraview-mcp
   vx uv run auroraview-mcp
   ```

3. **检查依赖**
   ```bash
   cd packages/auroraview-mcp
   vx uv sync
   ```

4. **运行单元测试**
   ```bash
   just mcp-test
   ```

5. **使用 MCP Inspector 可视化调试**
   ```bash
   just mcp-inspector
   ```
   然后打开 http://localhost:5173

6. **检查 CDP 连接**
   - 确保 AuroraView 实例正在运行
   - 测试 CDP 端点: `curl http://localhost:9222/json`

## 常见问题

### MCP 服务器无法启动
- 检查 uv 是否安装: `vx uv --version`
- 检查 Python 版本: `python --version` (需要 3.10+)
- 检查依赖: `cd packages/auroraview-mcp && vx uv sync`

### 无法发现实例
- 确保 AuroraView 以 CDP 模式运行
- 检查端口配置 (默认 9222)
- 检查防火墙设置

### 连接失败
- 先运行 `discover_instances` 确认实例存在
- 检查 WebSocket 连接
- 确认没有其他程序占用端口

## 问题处理流程

调试完成后，如果发现问题，请按以下流程处理:

### 1. 总结问题

```
## Issue Summary

**Type**: [Bug/Missing Feature/Design Flaw/Performance/Documentation]
**Severity**: [Critical/High/Medium/Low]
**Component**: [affected file/module path]

**Problem**: [Brief description]

**Root Cause**: [Why this happens]

**Impact**: [What functionality is affected]

**Reproduction Steps**:
1. ...
2. ...

**Error Message** (if any):
```
error message here
```
```

### 2. 等待开发者决策

请选择如何处理此问题:

| 选项 | 适用场景 |
|------|---------|
| **1. 在当前分支修复** | 小改动 (<50行)，直接相关，低风险 |
| **2. 创建新分支** | 较大改动，需要隔离，可能影响其他功能 |
| **3. 创建 GitHub Issue** | 记录待办，不阻塞当前工作 |
| **4. 创建 RFC 提案** | 重大设计变更，需要团队讨论 |

**您的选择**: [1/2/3/4]

### 3. 执行决策

根据选择执行相应操作:

- **选项 1**: 直接在当前分支修改代码并提交
- **选项 2**: 使用 `git checkout -b fix/issue-name` 创建新分支
- **选项 3**: 使用 GitHub MCP 创建 Issue
- **选项 4**: 调用 `rfc-creator` skill 创建 RFC
