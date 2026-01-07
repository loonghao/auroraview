# MCP API 优化总结

## 优化概述

本次优化基于 [MCP Best Practices](https://modelcontextprotocol.io/sitemap.xml) 和 [RFC 0003: API Design Guidelines](./rfcs/0003-api-design-guidelines.md) 进行,旨在提升 AuroraView MCP Server 的质量和开发体验。

## 已完成的优化

### 1. MCP 标准注解支持

为 `Tool` 和 `ToolDefinition` 结构添加了 MCP 标准注解字段:

- `readOnlyHint`: 标记工具是否只读
- `destructiveHint`: 标记工具是否可能执行破坏性操作
- `idempotentHint`: 标记工具是否幂等
- `openWorldHint`: 标记工具是否与外部实体交互

**新增 API**:
```rust
pub fn with_output_schema(mut self, schema: Value) -> Self
pub fn read_only(mut self) -> Self
pub fn destructive(mut self) -> Self
pub fn idempotent(mut self) -> Self
pub fn open_world(mut self) -> Self
```

**使用示例**:
```rust
let tool = Tool::new("auroraview_get_user", "Get user information")
    .with_param("id", "string", "User ID")
    .read_only()
    .with_output_schema(json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "name": {"type": "string"}
        }
    }));
```

### 2. Output Schema 支持

为工具添加了 `output_schema` 字段,用于定义工具输出的 JSON Schema。这有助于 AI 助手更好地理解工具返回的数据结构。

**协议更新**:
```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,  // 新增
    pub read_only_hint: Option<bool>,
    pub destructive_hint: Option<bool>,
    pub idempotent_hint: Option<bool>,
    pub open_world_hint: Option<bool>,
}
```

### 3. 改进的错误处理

重构了错误类型,提供更详细和可操作的错误消息:

**错误消息格式**:
- 包含具体错误信息
- 提供建议的下一步操作
- 避免暴露内部实现细节

**新错误结构**:
```rust
pub enum McpError {
    #[error("Tool '{name}' not found. Available tools: {available}. Suggestion: Check the tool name spelling or call tools/list to see all available tools.")]
    ToolNotFound { name: String, available: String },

    #[error("Invalid arguments for tool '{tool}': {reason}. Suggestion: {suggestion}")]
    InvalidArguments { tool: String, reason: String, suggestion: String },

    #[error("Tool '{tool}' execution failed: {reason}. Suggestion: {suggestion}")]
    ToolExecutionFailed { tool: String, reason: String, suggestion: String },

    #[error("JSON serialization error: {0}. Suggestion: Check that the data structure is valid and contains only JSON-serializable types.")]
    Json(#[from] serde_json::Error),
}
```

**错误消息示例**:
```
Tool 'api.get_user' not found. Available tools: [api.get_user, api.list_users, ...]. 
Suggestion: Check the tool name spelling or call tools/list to see all available tools.
```

### 4. Python Docstrings 自动提取

Python 绑定已经支持从函数的 `__doc__` 属性自动提取工具描述:

```python
@view.bind_call("api.get_user")
def get_user(user_id: str) -> dict:
    """Get user information by ID.

    Args:
        user_id: The user ID to look up

    Returns:
        Dictionary containing user details
    """
    return {"id": user_id, "name": "John Doe"}
```

这个 docstring 会自动被提取并用作 MCP 工具的描述。

### 5. MCP 控制参数 (NEW!)

为 `bind_call` 添加了精细的 MCP 暴露控制:

**新增参数**:
- `mcp: bool = True` - 控制是否将方法暴露给 MCP 客户端
- `mcp_name: Optional[str] = None` - 为 MCP 工具提供自定义名称

**使用示例**:

```python
# 1. 默认行为 - 暴露到 MCP (mcp=True)
@view.bind_call("api.get_user")
def get_user(user_id: str) -> dict:
    return {"name": "Alice"}

# 2. 隐藏内部方法 - 仅 JavaScript 可用
@view.bind_call("api._internal_debug", mcp=False)
def internal_debug() -> dict:
    return {"debug": "info"}

# 3. 自定义 MCP 名称 - 对 AI 助手更友好
@view.bind_call("api.create_user_record", mcp_name="create_user")
def create_user_record(name: str) -> dict:
    return {"name": name}

# 4. 显式控制 - 代码更清晰
@view.bind_call("api.delete_user", mcp=True)
def delete_user(user_id: str) -> dict:
    return {"deleted": True}
```

**优势**:
- **选择性暴露**: 精确控制哪些 API 需要暴露给 MCP
- **自定义命名**: 为 MCP 客户端提供更友好的工具名
- **向后兼容**: 默认 `mcp=True`,不影响现有代码
- **灵活性**: 开发者可以精细控制 MCP 暴露的 API

**参见**: [MCP Control Guide](mcp-control-guide.md) 了解更多详细信息和最佳实践。

## 待完成的优化

### 3. 工具命名约定

**目标**: 将工具名称从 `api.xxx` 改为 `auroraview_{action}_{resource}` 格式。

**当前状态**: 待讨论

**原因**:
- 当前命名约定 `api.xxx` 是 AuroraView 框架内部的约定
- 更改命名可能会影响到现有的前端代码和 API 使用方式
- 需要与团队讨论变更的影响范围和迁移策略

**建议方案**:

1. **保持向后兼容**: 同时支持新旧两种命名约定
   ```python
   # 旧命名 (保留用于内部调用)
   @view.bind_call("api.get_user")
   def get_user(user_id: str) -> dict:
       ...

   # 新命名 (用于 MCP 暴露)
   @view.mcp_tool("auroraview_get_user")
   def get_user_mcp(user_id: str) -> dict:
       return get_user(user_id)
   ```

2. **自动名称映射**: 在 MCP 工具注册时自动进行名称转换
   ```python
   # api.get_source -> auroraview_get_source
   # api.run_sample -> auroraview_run_sample
   ```

3. **渐进式迁移**: 先在新工具中使用新命名,旧工具保持不变

## 文件修改清单

### Rust 代码
- `crates/auroraview-mcp/src/protocol.rs`: 添加注解和 output_schema 字段
- `crates/auroraview-mcp/src/tool.rs`: 添加注解支持方法和更新错误处理
- `crates/auroraview-mcp/src/error.rs`: 重构错误类型,提供可操作消息
- `crates/auroraview-mcp/src/server.rs`: 更新错误使用
- `crates/auroraview-mcp/src/python.rs`: 更新错误处理

### Python 代码
- `python/auroraview/core/webview.py`: 已支持 docstring 提取

### 6. MCP Prompts 支持 (NEW!)

为 MCP Server 添加了完整的 Prompts 功能支持：

**新增功能**:
- `PromptRegistry` - 管理 prompt 定义
- `Prompt` - Prompt 结构，包含 name、description、arguments
- `PromptHandler` - 异步 prompt 执行器
- `GetPromptResult` - 返回 prompt 定义和可选消息

**Python API**:
- `@mcp_server.prompt()` - 装饰器模式注册 prompt
- `mcp_server.register_prompt()` - 手动注册 prompt
- `mcp_server.list_prompts()` - 列出所有 prompts

**Rust 实现**:
- `ServerCapabilities::enable_prompts()` - 启用 prompts 能力
- `list_prompts()` - 列出所有 prompts
- `get_prompt()` - 获取并执行指定 prompt

**使用示例**:

```python
# 注册简单 prompt
@view.mcp_server.prompt()
def code_review() -> str:
    """Generate code review feedback."""
    return "Please review the following code for:\n" \
           "1. Code quality and readability\n" \
           "2. Performance considerations\n" \
           "3. Security vulnerabilities\n" \
           "4. Best practices adherence\n" \
           "Provide specific, actionable feedback for each issue found."

# 带 custom 名称的 prompt
@view.mcp_server.prompt(name="bug_report")
def generate_bug_report() -> str:
    """Generate a structured bug report template."""
    return "Please generate a bug report with the following structure:\n" \
           "**Summary**: Brief description of the issue\n" \
           "**Steps to Reproduce**: Detailed steps to reproduce\n" \
           "**Expected Behavior**: What should happen\n" \
           "**Actual Behavior**: What actually happened\n"

# 返回消息的 prompt
@view.mcp_server.prompt()
def greet_user() -> str:
    def greet_user(self) -> str:
        return GetPromptResult::new(...)
            .with_system_message("You are a helpful assistant.")
            .with_user_message("Hello! How can I help you today?")
```

**优势**:
- **模板重用**: AI 助手可以重复使用高质量 prompt
- **一致性**: 确保所有 AI 响应遵循相同指南
- **上下文管理**: Prompts 可以包含预定义的消息和系统指令
- **灵活性**: 支持 prompt 参数化以适应不同场景

**参见**: [MCP Prompts Guide](mcp-prompts-guide.md) 了解更多详细信息和用例。

## 下一步

1. **完成命名约定迁移**: 与团队讨论并确定命名迁移策略
2. **添加单元测试**: 为新的注解、错误处理和 prompts 添加测试
3. **更新文档**: 更新 Gallery 示例以使用新的注解
4. **性能优化**: 考虑添加工具执行时间限制和缓存

## 参考资料

- [MCP Best Practices](https://modelcontextprotocol.io/sitemap.xml)
- [MCP Protocol Specification](https://modelcontextprotocol.io/specification/draft.md)
- [AuroraView RFC 0002](./rfcs/0002-embedded-mcp-server.md)
- [AuroraView RFC 0003](./rfcs/0003-api-design-guidelines.md)

---

**文档版本**: 1.0  
**创建日期**: 2025-01-03  
**作者**: AuroraView Team
