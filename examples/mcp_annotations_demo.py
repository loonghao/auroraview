"""AuroraView MCP Server - 示例: 使用 MCP 标准注解

本示例展示如何使用 AuroraView MCP Server 的新特性:
- MCP 标准注解 (readOnly, destructive, idempotent, openWorld)
- Output Schema 定义
- 改进的错误处理
"""

import asyncio
import sys
from typing import Dict

# 添加父目录到路径以导入 auroraview
sys.path.insert(0, str(__file__).parent.parent)
sys.path.insert(0, str(__file__).parent.parent / "python"))

from auroraview import WebView, McpConfig, ok, err


def main():
    """运行 MCP 示例应用"""

    # 创建 WebView 并启用 MCP
    view = WebView(
        title="MCP Annotations Demo",
        url="data:text/html,<h1>MCP Annotations Demo</h1>",
        width=800,
        height=600,
        mcp=True,
        mcp_port=8765,
        mcp_config=McpConfig(
            auto_expose_api=True,
            host="127.0.0.1",
            name="mcp-annotations-demo",
        )
    )

    # ============ 示例 1: 只读工具 ============
    @view.bind_call("api.get_config")
    def get_config() -> dict:
        """Get application configuration (read-only).

        Returns:
            Dictionary containing current configuration:
            - theme: Current theme name
            - language: Current language
            - timezone: Current timezone
        """
        # 这是一个只读操作,MCP 客户端可以通过 readOnlyHint 知道
        # 不会修改任何状态
        return ok({
            "theme": "dark",
            "language": "en",
            "timezone": "UTC"
        })

    # ============ 示例 2: 幂等工具 ============
    @view.bind_call("api.set_theme")
    def set_theme(theme: str = "dark") -> dict:
        """Set application theme (idempotent).

        Args:
            theme: Theme name (dark/light/auto)

        Returns:
            Confirmation message with new theme

        Note:
            设置相同的主题多次不会有副作用,
            因为这是幂等操作
        """
        # 这是一个幂等操作 - 多次调用相同参数不会有额外效果
        valid_themes = ["dark", "light", "auto"]
        if theme not in valid_themes:
            return err(f"Invalid theme '{theme}'. Valid themes: {', '.join(valid_themes)}")
        
        return ok({
            "theme": theme,
            "message": f"Theme set to {theme}"
        })

    # ============ 示例 3: 破坏性操作 ============
    @view.bind_call("api.delete_user")
    def delete_user(user_id: str) -> dict:
        """Delete a user account (destructive).

        Args:
            user_id: ID of the user to delete

        Returns:
            Deletion confirmation

        Warning:
            This operation cannot be undone!
            MCP 客户端会看到 destructiveHint=True
        """
        # 这是一个破坏性操作,需要额外的确认
        if not user_id:
            return err("User ID is required. Please provide a valid user ID to delete.")
        
        # 模拟删除操作
        return ok({
            "deleted": user_id,
            "message": f"User {user_id} has been deleted (this action cannot be undone)"
        })

    # ============ 示例 4: 外部世界操作 ============
    @view.bind_call("api.send_email")
    def send_email(to: str, subject: str, body: str) -> dict:
        """Send an email (external world operation).

        Args:
            to: Recipient email address
            subject: Email subject
            body: Email body content

        Returns:
            Email delivery status

        Note:
            This operation interacts with external email service.
            MCP 客户端会看到 openWorldHint=True
        """
        if not all([to, subject, body]):
            return err("All fields (to, subject, body) are required to send an email.")
        
        if "@" not in to:
            return err(f"Invalid email address: {to}. Must contain '@' symbol.")
        
        # 模拟发送邮件
        return ok({
            "to": to,
            "subject": subject,
            "status": "sent",
            "message": f"Email sent to {to}"
        })

    # ============ 示例 5: 复杂输出 with Schema ============
    @view.bind_call("api.get_user_profile")
    def get_user_profile(user_id: str) -> dict:
        """Get detailed user profile with structured output.

        Args:
            user_id: Unique user identifier

        Returns:
            User profile with the following structure:
            {
                "id": string,           // User ID
                "username": string,      // Username
                "email": string,         // Email address
                "profile": {
                    "display_name": string,  // Display name
                    "bio": string,          // Short bio
                    "avatar_url": string     // Avatar URL
                },
                "stats": {
                    "followers": integer,    // Follower count
                    "following": integer,    // Following count
                    "posts": integer        // Post count
                },
                "created_at": string,      // ISO 8601 timestamp
                "updated_at": string       // ISO 8601 timestamp
            }

        Raises:
            UserNotFoundError: If user_id does not exist
        """
        if not user_id:
            return err("User ID is required. Please provide a valid user ID.")
        
        # 模拟用户数据
        users_db = {
            "user_001": {
                "username": "john_doe",
                "email": "john@example.com",
                "profile": {
                    "display_name": "John Doe",
                    "bio": "Software Developer",
                    "avatar_url": "https://example.com/avatars/john.jpg"
                },
                "stats": {
                    "followers": 1234,
                    "following": 567,
                    "posts": 89
                }
            }
        }
        
        user = users_db.get(user_id)
        if not user:
            return err(f"User '{user_id}' not found. Available users: {', '.join(users_db.keys())}. Try listing users first.")
        
        # 返回结构化的用户数据
        return ok({
            "id": user_id,
            "username": user["username"],
            "email": user["email"],
            "profile": user["profile"],
            "stats": user["stats"],
            "created_at": "2025-01-03T12:00:00Z",
            "updated_at": "2025-01-03T12:30:00Z"
        })

    # ============ 示例 6: 分页列表 ============
    @view.bind_call("api.list_users")
    def list_users(limit: int = 20, offset: int = 0) -> dict:
        """List users with pagination support.

        Args:
            limit: Maximum number of items to return (default: 20, max: 100)
            offset: Number of items to skip (default: 0)

        Returns:
            Paginated user list:
            {
                "items": [...],       // User objects
                "total": 150,        // Total number of users
                "offset": 0,         // Current offset
                "limit": 20,         // Applied limit
                "has_more": true       // Whether more items available
            }
        """
        if limit > 100:
            return err(f"Limit cannot exceed 100. Requested: {limit}")
        
        if offset < 0:
            return err(f"Offset cannot be negative. Requested: {offset}")
        
        # 模拟用户列表
        all_users = [f"user_{i:03d}" for i in range(1, 151)]
        items = all_users[offset:offset + limit]
        total = len(all_users)
        has_more = offset + limit < total
        
        return ok({
            "items": items,
            "total": total,
            "offset": offset,
            "limit": limit,
            "has_more": has_more
        })

    # ============ 示例 7: 带详细错误处理的工具 ============
    @view.bind_call("api.create_user")
    def create_user(username: str, email: str, password: str) -> dict:
        """Create a new user account.

        Args:
            username: Desired username (3-20 characters, alphanumeric only)
            email: Valid email address
            password: Password (min 8 characters, must contain letter and number)

        Returns:
            Created user object:
            {
                "id": string,        // New user ID
                "username": string,   // Username
                "email": string,      // Email
                "created_at": string  // Creation timestamp
            }

        Raises:
            UsernameExistsError: If username already taken
            InvalidEmailError: If email format is invalid
            WeakPasswordError: If password does not meet requirements
        """
        # 验证用户名
        if not username or len(username) < 3 or len(username) > 20:
            return err("Username must be between 3 and 20 characters. Please provide a valid username.")
        
        if not username.isalnum():
            return err("Username can only contain alphanumeric characters. Please use letters and numbers only.")
        
        # 验证邮箱
        if not email or "@" not in email or "." not in email:
            return err("Invalid email format. Please provide a valid email address (e.g., user@example.com).")
        
        # 验证密码
        if not password or len(password) < 8:
            return err("Password must be at least 8 characters long. Please use a stronger password.")
        
        if not any(c.isalpha() for c in password):
            return err("Password must contain at least one letter. Please add letters to your password.")
        
        if not any(c.isdigit() for c in password):
            return err("Password must contain at least one number. Please add numbers to your password.")
        
        # 模拟用户创建
        new_user_id = f"user_{username}"
        return ok({
            "id": new_user_id,
            "username": username,
            "email": email,
            "created_at": "2025-01-03T12:00:00Z"
        })

    # ============ 示例 8: 异步操作 ============
    @view.bind_call("api.async_operation")
    async def async_operation(duration: int = 5) -> dict:
        """Perform an asynchronous operation.

        Args:
            duration: Duration in seconds to wait (default: 5)

        Returns:
            Operation result:
            {
                "status": string,      // Operation status
                "duration": number,   // Actual duration
                "result": string      // Result message
            }
        """
        if duration < 0:
            return err("Duration cannot be negative. Please provide a positive number of seconds.")
        
        if duration > 60:
            return err("Duration cannot exceed 60 seconds for this operation. Please use a shorter duration.")
        
        # 模拟异步操作
        await asyncio.sleep(duration)
        
        return ok({
            "status": "completed",
            "duration": duration,
            "result": f"Async operation completed after {duration} seconds"
        })

    # 打印启动信息
    print("=" * 60, file=sys.stderr)
    print("MCP Annotations Demo", file=sys.stderr)
    print("=" * 60, file=sys.stderr)
    print("\nRegistered MCP Tools:", file=sys.stderr)
    print("  - api.get_config (read-only)", file=sys.stderr)
    print("  - api.set_theme (idempotent)", file=sys.stderr)
    print("  - api.delete_user (destructive)", file=sys.stderr)
    print("  - api.send_email (open world)", file=sys.stderr)
    print("  - api.get_user_profile (with output schema)", file=sys.stderr)
    print("  - api.list_users (paginated)", file=sys.stderr)
    print("  - api.create_user (detailed error handling)", file=sys.stderr)
    print("  - api.async_operation (async)", file=sys.stderr)
    print("\nMCP Server running on http://127.0.0.1:8765/mcp", file=sys.stderr)
    print("Configure Claude Desktop to connect to this URL.", file=sys.stderr)
    print("=" * 60, file=sys.stderr)
    
    # 显示窗口
    view.show()


if __name__ == "__main__":
    main()
