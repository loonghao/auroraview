快速测试 Gallery 嵌入式 MCP Server（Rust Streamable HTTP）。

## 前置条件

1. **启动 Gallery**
   ```bash
   # 使用固定端口启动（推荐）
   just gallery-mcp 27168

   # 或默认启动（端口随机）
   just gallery
   ```

2. **确认 MCP 端口**
   查看控制台输出：
   ```
   [Python] MCP auto-selected free port 27168
   ```

## 测试流程

### 方式 1: 使用 just 命令

```bash
# 健康检查
just gallery-mcp-health 27168

# 列出所有工具
just gallery-mcp-list 27168

# 运行完整测试
just gallery-mcp-test 27168

# 调用特定工具
just gallery-mcp-call 27168 "api.get_samples" "{}"
just gallery-mcp-call 27168 "api.get_source" '{"sample_id":"hello_world"}'
```

### 方式 2: 使用 IDE MCP 客户端

配置 `~/.codebuddy/mcp.json`：
```json
{
  "mcpServers": {
    "auroraview-gallery": {
      "url": "http://127.0.0.1:27168/mcp",
      "transportType": "streamable-http"
    }
  }
}
```

然后测试以下工具：

| # | 工具 | 预期结果 |
|---|------|---------|
| 1 | `api.get_samples` | 返回示例数组 |
| 2 | `api.get_categories` | 返回分类字典 |
| 3 | `api.get_mcp_info` | 返回 MCP 服务器信息 |
| 4 | `api.get_source(sample_id="hello_world")` | 返回 Python 源代码 |
| 5 | `api.list_processes` | 返回进程列表 |
| 6 | `api.get_children` | 返回子窗口列表 |
| 7 | `api.list_webview_extensions` | 返回扩展列表 |

## 测试报告模板

| 步骤 | 工具 | 状态 | 结果/问题 |
|------|------|------|----------|
| 1 | api.get_samples | ✅/❌ | |
| 2 | api.get_categories | ✅/❌ | |
| 3 | api.get_mcp_info | ✅/❌ | |
| 4 | api.get_source | ✅/❌ | |
| 5 | api.list_processes | ✅/❌ | |

## 常见问题

### MCP 客户端无法连接
1. 确认 Gallery 正在运行
2. 检查端口：`netstat -an | findstr 27168`
3. 测试健康端点：`curl http://127.0.0.1:27168/health`

### 工具调用返回空结果
1. 检查 Gallery 控制台输出
2. 查看 `[AuroraView DEBUG]` 日志
3. 确认工具名称正确（区分大小写）

### 端口冲突
```bash
# 使用其他端口
just gallery-mcp 27200
```

## 问题处理

发现问题后，按以下格式总结：

```
## Issue Summary

**Type**: Bug / Missing Feature / Design Flaw
**Severity**: Critical / High / Medium / Low
**Component**: affected file/module

**Problem**: Brief description

**Root Cause**: Why this happens

**Impact**: What is affected
```

然后**等待开发者确认**如何处理：

1. **在当前分支修复** - 小改动，直接修复
2. **创建新分支** - 较大改动，基于 remote main
3. **创建 GitHub Issue** - 记录待办
4. **创建 RFC 提案** - 设计变更

请选择: [1/2/3/4]
