"""MCP Control Demo - Demonstrates mcp and mcp_name parameters.

This example shows how to control which methods are exposed to MCP
and how to customize MCP tool names.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from auroraview import AuroraView, McpConfig, ok, err


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
        "MCP Control Demo",
        width=1200,
        height=800,
        mcp_config=mcp_config,
        mcp_enabled=True,
    )

    # Example 1: Default behavior - exposed to MCP with original name
    @view.bind_call("api.get_user")
    def get_user(user_id: str) -> dict:
        """Get user information by ID.

        This will be registered as MCP tool "api.get_user".
        """
        # Simulate database lookup
        users = {
            "1": {"id": "1", "name": "Alice", "role": "admin"},
            "2": {"id": "2", "name": "Bob", "role": "user"},
        }
        return users.get(user_id, err(f"User {user_id} not found"))

    # Example 2: Hidden from MCP - only available to JavaScript
    @view.bind_call("api._internal_debug", mcp=False)
    def internal_debug() -> dict:
        """Internal debugging function - not exposed to MCP.

        This will NOT be registered as an MCP tool because mcp=False.
        Only available via auroraview.call() from JavaScript.
        """
        return ok({
            "message": "This is internal - AI assistants cannot see this",
            "debug_info": {"version": "1.0.0", "env": "production"},
        })

    # Example 3: Custom MCP name - user-friendly for AI assistants
    @view.bind_call(
        "api.create_user_record",
        mcp_name="create_user"  # Simpler name for MCP
    )
    def create_user_record(name: str, email: str) -> dict:
        """Create a new user record in the system.

        This will be registered as MCP tool "create_user" instead of
        "api.create_user_record", making it more natural for AI assistants.
        """
        return ok({
            "id": "3",
            "name": name,
            "email": email,
            "created_at": "2025-01-03T10:00:00Z",
        })

    # Example 4: Destructive operation with custom MCP name
    @view.bind_call(
        "api._delete_user_by_id",
        mcp_name="delete_user",  # Clearer name for MCP
        mcp=True,  # Explicitly exposed (same as default)
    )
    def delete_user_by_id(user_id: str) -> dict:
        """Delete a user by ID (destructive operation).

        This will be registered as MCP tool "delete_user".
        The mcp=True explicitly enables MCP exposure (redundant but clear).
        """
        return ok({
            "deleted": True,
            "user_id": user_id,
            "warning": "This operation cannot be undone",
        })

    # Example 5: Multiple decorators with different MCP settings
    @view.bind_call("api.get_stats")
    def get_stats() -> dict:
        """Get system statistics.

        Default: exposed to MCP as "api.get_stats".
        """
        return ok({
            "total_users": 100,
            "active_sessions": 42,
            "uptime_seconds": 3600,
        })

    # Example 6: Decorator with custom name only
    @view.bind_call("api._system_health_check", mcp_name="health_check")
    def system_health_check() -> dict:
        """System health check endpoint.

        Registered as MCP tool "health_check" instead of
        "api._system_health_check" - much cleaner!
        """
        return ok({
            "status": "healthy",
            "checks": {
                "database": "ok",
                "cache": "ok",
                "api": "ok",
            },
        })

    print("\n=== MCP Control Demo ===")
    print("The following MCP tools will be exposed:")
    print("  ✓ api.get_user        (default name)")
    print("  ✗ api._internal_debug (hidden: mcp=False)")
    print("  ✓ create_user         (custom name from api.create_user_record)")
    print("  ✓ delete_user         (custom name from api._delete_user_by_id)")
    print("  ✓ api.get_stats       (default name)")
    print("  ✓ health_check        (custom name from api._system_health_check)")
    print("\nInternal methods hidden from MCP:")
    print("  ✗ api._internal_debug (mcp=False)")
    print("\nMethods with custom MCP names:")
    print("  api.create_user_record  → create_user")
    print("  api._delete_user_by_id → delete_user")
    print("  api._system_health_check → health_check")
    print("\nStarting WebView...\n")

    # Show the WebView
    view.show()


if __name__ == "__main__":
    main()
