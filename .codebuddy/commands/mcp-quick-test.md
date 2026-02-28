快速测试 auroraview-dev MCP 服务器的所有核心功能。

## 测试流程

请依次执行以下测试:

1. **获取项目信息** - 调用 `get_project_info` 验证 MCP 服务器正常运行
2. **获取可用示例** - 调用 `get_samples` 列出所有可用示例
3. **发现实例** - 调用 `discover_instances` 查看运行中的 AuroraView 实例
4. **启动 Gallery** - 如果没有实例，调用 `run_gallery` 启动 Gallery (端口 9222)
5. **连接实例** - 调用 `connect(port=9222)` 连接到 Gallery
6. **截图验证** - 调用 `take_screenshot` 截图确认连接成功
7. **获取页面快照** - 调用 `get_snapshot` 获取页面结构
8. **测试 API** - 调用 `list_api_methods` 和 `call_api` 测试 Python API 桥接

## 测试报告

测试完成后，生成测试报告:

| 步骤 | 工具 | 状态 | 结果/问题 |
|------|------|------|----------|
| 1 | get_project_info | ? | |
| 2 | get_samples | ? | |
| ... | ... | ... | |

## 问题处理

如果发现问题，请按以下格式总结:

```
## Issue Summary

**Type**: Bug / Missing Feature / Design Flaw
**Severity**: Critical / High / Medium / Low
**Component**: affected file/module

**Problem**: Brief description

**Root Cause**: Why this happens

**Impact**: What is affected
```

然后**等待开发者确认**如何处理:

1. **在当前分支修复** - 小改动，直接修复
2. **创建新分支** - 较大改动，需要隔离 (基于 remote main)
3. **创建 GitHub Issue** - 记录待办，暂不处理
4. **创建 RFC 提案** - 设计变更，需要讨论

请选择: [1/2/3/4]

### 创建新分支流程 (选项 2)

```bash
# 1. 获取最新的 remote main
git fetch origin main

# 2. 基于 remote main 创建新分支
git checkout -b fix/issue-name origin/main

# 3. 修改代码...

# 4. 提交并推送
git add -A
git commit -m "fix: description"
git push -u origin fix/issue-name

# 5. 创建 PR (可选)
gh pr create --title "fix: description" --body "..."
```
