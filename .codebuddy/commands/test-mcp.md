测试本地开发版本的 AuroraView MCP Server。

## 配置步骤

1. 确保 `~/.codebuddy/mcp.json` 包含以下配置:

```json
{
  "mcpServers": {
    "auroraview-dev": {
      "command": "vx",
      "args": [
        "uv",
        "--directory",
        "C:/Users/hallo/Documents/augment-projects/dcc_webview/packages/auroraview-mcp",
        "run",
        "auroraview-mcp"
      ],
      "env": {
        "AURORAVIEW_DEFAULT_PORT": "9222",
        "AURORAVIEW_PROJECT_ROOT": "C:/Users/hallo/Documents/augment-projects/dcc_webview"
      }
    }
  }
}
```

2. 重启 CodeBuddy 以加载新的 MCP 配置

## 测试流程

请按以下顺序测试 MCP 功能:

### 1. 基础连接测试
- 调用 `discover_instances` 发现运行中的 AuroraView 实例
- 如果没有实例，调用 `run_gallery` 启动 Gallery
- 调用 `connect` 连接到实例

### 2. 页面操作测试
- 调用 `list_pages` 列出所有页面
- 调用 `get_page_info` 获取页面详情
- 调用 `take_screenshot` 截图验证

### 3. API 调用测试
- 调用 `list_api_methods` 列出可用 API
- 调用 `call_api` 测试 API 调用

### 4. UI 自动化测试
- 调用 `get_snapshot` 获取页面结构
- 调用 `click` 或 `fill` 测试 UI 操作

### 5. 清理
- 调用 `stop_gallery` 停止 Gallery
- 调用 `disconnect` 断开连接

## 调试命令

如果遇到问题，可以使用以下命令调试:

```bash
# 手动启动 MCP 服务器查看输出
cd packages/auroraview-mcp && vx uv run auroraview-mcp

# 运行单元测试
just mcp-test

# 使用 MCP Inspector 可视化调试
just mcp-inspector

# 运行 debug client
just mcp-debug
```

## 预期结果

- `discover_instances`: 返回实例列表 (port, pid, title)
- `connect`: 返回连接成功信息
- `take_screenshot`: 返回 base64 编码的截图
- `get_snapshot`: 返回页面 accessibility tree
- `call_api`: 返回 API 调用结果

## 问题处理流程

测试过程中如果发现问题，请按以下流程处理:

### Step 1: 总结问题

```
## Issue Summary

**Type**: Bug / Missing Feature / Design Flaw / Performance
**Severity**: Critical / High / Medium / Low
**Component**: affected file/module

**Problem**: Brief description of the issue

**Root Cause**: Analysis of why this happens

**Impact**: What functionality is affected

**Proposed Fix**: High-level solution approach
```

### Step 2: 等待开发者决策

请选择如何处理:

1. **在当前分支修复** - 小改动，直接相关，低风险
2. **创建新分支** - 较大改动，需要隔离测试
3. **创建 GitHub Issue** - 记录待办，暂不处理
4. **创建 RFC 提案** - 重大设计变更，需要讨论

**您的选择**: [1/2/3/4]

### Step 3: 执行决策

根据选择执行:

| 选项 | 操作 |
|------|------|
| 1 | 直接修改代码，`git commit -m "fix: ..."` |
| 2 | `git checkout -b fix/issue-name`，修改后 `git push -u origin fix/...` |
| 3 | 使用 GitHub MCP `issue_write` 创建 Issue |
| 4 | 调用 `rfc-creator` skill 创建 RFC 文档 |

### 多问题处理

如果发现多个问题，创建汇总表:

| # | 问题 | 严重性 | 处理方式 | 状态 |
|---|------|--------|---------|------|
| 1 | ... | High | 当前分支 | Pending |
| 2 | ... | Medium | Issue | Created |
| 3 | ... | Low | RFC | Pending |

按优先级 (Critical > High > Medium > Low) 依次处理。
