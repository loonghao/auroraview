# MCP Prompts Guide

This guide explains how to use MCP Prompts in AuroraView.

## Overview

MCP Prompts allow you to provide reusable prompt templates to AI assistants. Unlike Tools (which execute operations), Prompts generate text content that can be used as messages or templates.

## Key Differences: Prompts vs Tools

| Feature | Tools | Prompts |
|----------|-------|---------|
| **Purpose** | Execute operations | Generate text/templates |
| **Behavior** | Return structured data | Return text content |
| **Usage** | Call for actions | Use as message/template |
| **Example** | `get_user()` | `code_review_prompt()` |
| **Stateful** | No | Can include messages/context |

## Registering Prompts

### Method 1: Decorator Pattern (Recommended)

Use the `@mcp_server.prompt()` decorator:

```python
@view.mcp_server.prompt()
def code_review() -> str:
    """Generate code review feedback.

    This prompt can be used by AI assistants to perform
    code reviews with consistent guidelines.
    """
    return "Please review the following code for:\n" \
           "1. Code quality and readability\n" \
           "2. Performance considerations\n" \
           "3. Security vulnerabilities\n" \
           "4. Best practices adherence\n" \
           "Provide specific, actionable feedback for each issue found."
```

### Method 2: Named Prompt

Provide a custom name:

```python
@view.mcp_server.prompt(name="bug_report")
def generate_bug_report() -> str:
    """Generate a structured bug report template."""
    return "Please generate a bug report with following structure:\n" \
           "**Summary**: Brief description of issue\n" \
           "**Steps to Reproduce**: Detailed steps to reproduce\n" \
           "**Expected Behavior**: What should happen\n" \
           "**Actual Behavior**: What actually happened\n"
```

### Method 3: Custom Description

Override the docstring:

```python
@view.mcp_server.prompt(description="Analyze image content for context")
def analyze_image_prompt() -> str:
    def analyze_image_prompt(self) -> str:
        # Implementation
        pass
```

## Using Prompts in AI Assistants

When an AI assistant connects to your MCP server, it can:

1. **List available prompts** via `prompts/list`
2. **Get a specific prompt** via `prompts/get`
3. **Execute a prompt** and get the generated text

### Prompt Execution Flow

1. **Request**: AI assistant calls `prompts/get` with a prompt name and optional arguments
2. **Execute**: Server runs the prompt function with provided arguments
3. **Response**: Returns `GetPromptResult` containing:
   - `prompt`: The prompt definition (name, description, arguments)
   - `messages`: Optional array of message templates (user, system, assistant)

## Prompt Messages

Prompts can include pre-defined messages in their response:

```python
@view.mcp_server.prompt()
def greet_user() -> str:
    """Greet a new user with a personalized message."""
    
    # The prompt can return messages
    return GreetUser()
        .with_system_message("You are a helpful assistant for AuroraView Gallery")
        .with_user_message("Hello! How can I help you today?")
```

## Use Cases

### 1. Code Review Template

Provide consistent review guidelines:

```python
@view.mcp_server.prompt()
def code_review() -> str:
    """Generate code review feedback.

    Ensures all code reviews follow consistent standards
    and provide actionable, specific feedback.
    """
    return "Please review the following code for:\n" \
           "1. Code quality and readability\n" \
           "2. Performance considerations\n" \
           "3. Security vulnerabilities\n" \
           "4. Best practices adherence\n" \
           "Provide specific, actionable feedback for each issue found."
```

### 2. Bug Report Template

Structured bug reporting with all required fields:

```python
@view.mcp_server.prompt(name="bug_report")
def generate_bug_report() -> str:
    """Generate a structured bug report template.

    Ensures all bug reports contain necessary information
    for effective debugging and tracking.
    """
    return "Please generate a bug report with following structure:\n" \
           "**Summary**: Brief description of issue\n" \
           "**Steps to Reproduce**: Detailed steps to reproduce\n" \
           "**Expected Behavior**: What should happen\n" \
           "**Actual Behavior**: What actually happened\n" \
           "**Environment**: OS, browser version, AuroraView version\n" \
           "**Error Messages**: Any error logs or stack traces"
```

### 3. Documentation Template

Consistent documentation structure:

```python
@view.mcp_server.prompt(name="write_docs")
def write_documentation() -> str:
    """Generate documentation for code features."""
    
    return "Please write documentation for this feature including:\n" \
           "1. **Purpose**: Clear description of what the feature does\n" \
           "2. **Parameters**: List of parameters with types and descriptions\n" \
           "3. **Return Value**: Structure and type of return value\n" \
           "4. **Example Usage**: Code showing how to use the feature\n" \
           "5. **Notes**: Any important considerations or edge cases\n" \
           "Use markdown formatting and follow project documentation style guide."
```

### 4. Debugging Template

Structured debugging assistance:

```python
@view.mcp_server.prompt(name="debug_info", description="Get system debug information")
def get_debug_prompt() -> str:
    """Generate a prompt for debugging system issues.

    Helps gather relevant debugging information from
    system state and context.
    """
    return "Please help debug this issue. Provide:\n" \
           "1. Current system state and configuration\n" \
           "2. Recent error messages or exceptions\n" \
           "3. Steps already taken to resolve the issue\n" \
           "4. Relevant logs or traces\n" \
           "5. Suggested next steps for investigation\n" \
           "Be specific about the error and context."
```

### 5. Test Case Generation

Comprehensive test case structure:

```python
@view.mcp_server.prompt(name="test_case", description="Generate test case for a feature")
def generate_test_case() -> str:
    """Create a comprehensive test case structure.

    Ensures test cases include all necessary elements
    for thorough testing and reproducibility.
    """
    return "Please generate a test case that includes:\n" \
           "1. **Test Case Name**: Clear, descriptive title\n" \
           "2. **Description**: What the test verifies\n" \
           "3. **Preconditions**: State required before running the test\n" \
           "4. **Test Steps**: Detailed steps to execute\n" \
           "5. **Expected Results**: What should happen\n" \
           "6. **Acceptance Criteria**: How to determine pass/fail\n" \
           "Include edge cases and error scenarios."
```

## Advanced Usage

### Prompt with Arguments

You can define prompts that accept parameters:

```python
@view.mcp_server.prompt(name="analyze_content")
def analyze_content_prompt(content_type: str, depth: int) -> str:
    """Analyze content with specified parameters.
    
    Args:
        content_type: Type of content (image, text, code)
        depth: Analysis depth (basic, detailed, comprehensive)
    """
    if content_type == "image":
        return f"Please analyze this {depth} image..."
    else:
        return f"Please analyze this {depth} text content..."
```

### Prompts vs Tools Integration

Use prompts to guide AI to use your tools:

```python
# Tool that performs an action
@view.bind_call("api.delete_user")
def delete_user(user_id: str) -> dict:
    return {"deleted": True, "user_id": user_id}

# Prompt that guides how to use the tool
@view.mcp_server.prompt(name="delete_user_guide")
def delete_user_guide() -> str:
    """Guide AI assistants on how to use delete_user tool."""
    return """
To delete a user account safely:

1. First, verify the user_id is correct
2. Check for any active sessions or data dependencies
3. Use the api.delete_user tool with the user_id
4. Verify deletion was successful
5. Log the deletion for audit trail

Always ensure you have proper authorization before deleting.
"""
```

## Best Practices

### 1. Clear Descriptions

```python
# Good: Specific and actionable
@view.mcp_server.prompt()
def code_review() -> str:
    """Generate code review feedback.

    Focus on security, performance, and best practices.
    Provide line-by-line feedback when issues are found.
    """

# Avoid: Vague descriptions
@view.mcp_server.prompt()
def helper_prompt() -> str:
    """Help with something."""
```

### 2. Use Markdown Formatting

```python
@view.mcp_server.prompt()
def generate_documentation() -> str:
    """Generate documentation for code features."""
    return """
# Feature Name

**Purpose**: Description here

## Usage

```python
result = api.some_function(param1, param2)
```

## Parameters

| Name | Type | Required | Description |
|-------|------|----------|-------------|
| param1 | string | Yes | Description here |

## Returns

```json
{
    "result": "value"
}
```

## Notes

Any important considerations.
"""
```

### 3. Consistent Naming

```python
# Use action_verb naming pattern
@view.mcp_server.prompt(name="create_user")
@view.mcp_server.prompt(name="delete_user")
@view.mcp_server.prompt(name="update_user")

# Avoid generic names
# Bad: helper_prompt, do_something, process_data
# Good: code_review, bug_report, generate_docs
```

### 4. Context Awareness

Include relevant context in prompts:

```python
@view.mcp_server.prompt(name="context_aware_guide")
def get_context_prompt() -> str:
    """Generate context-aware guidance."""
    return """
You are helping with the AuroraView Gallery application.
The Gallery allows users to:
- Launch and manage example windows
- Run MCP tools and prompts
- Manage child windows and process communication

Current context:
- User is authenticated
- Multiple example windows may be open
- MCP server is available for AI assistants

Keep responses concise and actionable.
"""
```

## Debugging

### Check Registered Prompts

```python
# List all registered prompts
prompts = view.mcp_server.list_prompts()
print(f"Registered prompts: {prompts}")
```

### Test Prompt Execution

```python
# Manually test a prompt
import auroraview

server = auroraview.McpServer(...)
server.register_prompt("test_prompt", handler, "Test prompt")

# Test execution
result = server.prompts().execute("test_prompt", {"arg": "value"})
print(f"Result: {result}")
```

## Examples

See `examples/mcp_prompts_demo.py` for a complete working example demonstrating:
- Simple text prompts
- Prompts with custom names
- Prompts with descriptions
- Various use cases (code review, bug reports, documentation, etc.)

## Comparison with Tools

| Aspect | Tools | Prompts |
|--------|-------|---------|
| **Returns** | Structured data (JSON) | Text content |
| **State** | Stateless | Can include message history |
| **Purpose** | Execute operations | Generate/guide text |
| **Use Case** | Get data, delete user | Code review, bug reporting |
| **AI Usage** | "Call `api.get_user`" | "Get `bug_report` prompt" |

## API Reference

### McpServer.prompt()

```python
@mcp_server.prompt(name: Optional[str] = None, description: Optional[str] = None)
def prompt_decorator(func: Callable) -> Callable:
    """Decorator to register a prompt.
    
    Args:
        name: Optional custom prompt name (defaults to function name)
        description: Custom description (defaults to docstring)
        
    Returns:
        Decorator function
        
    Example:
        @view.mcp_server.prompt(name="review_code")
        def code_review() -> str:
            return "Please review..."
    """
```

### McpServer.register_prompt()

```python
def register_prompt(
    name: str,
    handler: Callable,
    description: str = ""
) -> None:
    """Manually register a prompt.
    
    Args:
        name: Prompt name
        handler: Python callable that returns prompt text
        description: Prompt description
        
    Example:
        def generate_prompt():
            return "Generate a report..."
        
        view.mcp_server.register_prompt("report", generate_prompt, "Bug report generator")
    """
```

## Migration from Tools to Prompts

When you have tools that should be prompts:

### Step 1: Identify Text Generation

If a tool returns formatted text, convert to prompt:

```python
# Before (Tool)
@view.bind_call("api.generate_code")
def generate_code(spec: str) -> dict:
    return {"code": f"Generated code for: {spec}"}

# After (Prompt)
@view.mcp_server.prompt()
def generate_code_prompt() -> str:
    """Generate code based on specification."""
    return f"Please generate code that: {spec}"
```

### Step 2: Add Prompt with Tool Guide

```python
# Tool
@view.bind_call("api.analyze_data")
def analyze_data(data: dict) -> dict:
    return {"analysis": result}

# Prompt that guides to use the tool
@view.mcp_server.prompt(name="analyze_data_guide")
def analyze_data_guide() -> str:
    """
To analyze data effectively:

1. Use the api.analyze_data tool with your data
2. Review the analysis results
3. Apply insights to your use case
4. Consider any limitations or edge cases
"""
```

## See Also

- [MCP Usage Guide](mcp-usage-guide.md)
- [MCP Control Guide](mcp-control-guide.md)
- [MCP Optimization Summary](mcp-optimization-summary.md)
- [RFC 0002: Embedded MCP Server](rfcs/0002-embedded-mcp-server.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io/specification/)
