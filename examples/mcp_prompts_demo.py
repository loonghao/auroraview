"""MCP Prompts Demo - Demonstrates MCP Prompt functionality.

This example shows how to register and use MCP prompts
to provide reusable prompt templates for AI assistants.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from auroraview import AuroraView, McpConfig, ok


def main():
    # Create MCP server config with auto_expose_api enabled
    mcp_config = McpConfig(
        name="gallery",
        host="127.0.0.1",
        port=0,  # Auto-port
        auto_expose_api=True,  # Automatically expose bind_call methods as MCP tools
    )

    # Create WebView
    view = AuroraView.create(
        "MCP Prompts Demo",
        width=1200,
        height=800,
        mcp_config=mcp_config,
        mcp_enabled=True,
    )

    # Example 1: Simple text prompt
    @view.mcp_server.prompt()
    def code_review() -> str:
        """Generate code review feedback.

        This prompt can be used by AI assistants to perform code reviews
        with consistent guidelines and feedback structure.
        """
        return "Please review the following code for:\n" \
               "1. Code quality and readability\n" \
               "2. Performance considerations\n" \
               "3. Security vulnerabilities\n" \
               "4. Best practices adherence\n\n" \
               "Provide specific, actionable feedback for each issue found."

    # Example 2: Prompt with user message template
    @view.mcp_server.prompt(name="bug_report")
    def generate_bug_report() -> str:
        """Generate a structured bug report template.

        This prompt provides a consistent format for reporting bugs
        with all necessary fields for effective debugging.
        """
        return "Please generate a bug report with the following structure:\n" \
               "**Summary**: Brief description of the issue\n" \
               "**Steps to Reproduce**: Detailed steps to reproduce\n" \
               "**Expected Behavior**: What should happen\n" \
               "**Actual Behavior**: What actually happened\n" \
               "**Environment**: OS, browser version, AuroraView version\n" \
               "**Error Messages**: Any error logs or stack traces"

    # Example 3: Prompt with parameters
    @view.mcp_server.prompt(name="analyze_image", description="Analyze image content for context")
    def analyze_image_prompt() -> str:
        """Generate a prompt for image analysis.

        This prompt helps AI assistants understand how to analyze
        and describe images with appropriate technical language.
        """
        return "Please analyze this image and provide:\n" \
               "1. Visual description of the image content\n" \
               "2. Technical details about composition, lighting, style\n" \
               "3. Any text or data visible in the image\n" \
               "4. Suggested use cases or applications\n" \
               "Be specific and use appropriate technical terminology."

    # Example 4: Documentation prompt
    @view.mcp_server.prompt(name="write_docs")
    def write_documentation() -> str:
        """Generate documentation for code features.

        Use this prompt to create consistent, well-structured
        documentation that follows project standards.
        """
        return "Please write documentation for this feature including:\n" \
               "1. **Purpose**: Clear description of what the feature does\n" \
               "2. **Parameters**: List of parameters with types and descriptions\n" \
               "3. **Return Value**: Structure and type of return value\n" \
               "4. **Example Usage**: Code showing how to use the feature\n" \
               "5. **Notes**: Any important considerations or edge cases\n\n" \
               "Use markdown formatting and follow the project documentation style guide."

    # Example 5: Debugging prompt
    @view.mcp_server.prompt(name="debug_info", description="Get system debug information")
    def get_debug_prompt() -> str:
        """Generate a prompt for debugging system issues.

        This prompt helps gather relevant debugging information
        from the system state and context.
        """
        return "Please help debug this issue. Provide:\n" \
               "1. Current system state and configuration\n" \
               "2. Recent error messages or exceptions\n" \
               "3. Steps already taken to resolve the issue\n" \
               "4. Relevant logs or traces\n" \
               "5. Suggested next steps for investigation\n" \
               "Be specific about the error and context."

    # Example 6: Testing prompt
    @view.mcp_server.prompt(name="test_case", description="Generate test case for a feature")
    def generate_test_case() -> str:
        """Create a comprehensive test case structure.

        This prompt ensures test cases include all necessary elements
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

    print("\n=== MCP Prompts Demo ===")
    print("Registered prompts:")
    print("  ✓ code_review")
    print("  ✓ bug_report")
    print("  ✓ analyze_image")
    print("  ✓ write_docs")
    print("  ✓ debug_info")
    print("  ✓ test_case")
    print("\nThese prompts can be used by AI assistants to:")
    print("  - Provide consistent review guidelines")
    print("  - Generate structured reports")
    print("  - Analyze visual content")
    print("  - Create documentation")
    print("  - Debug system issues")
    print("  - Generate test cases")
    print("\nStarting WebView...\n")

    # Show WebView
    view.show()


if __name__ == "__main__":
    main()
