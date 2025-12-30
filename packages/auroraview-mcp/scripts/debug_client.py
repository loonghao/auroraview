#!/usr/bin/env python
"""Debug client for testing AuroraView MCP Server locally.

Usage:
    # Run from packages/auroraview-mcp directory
    uv run python scripts/debug_client.py

    # Or with specific test
    uv run python scripts/debug_client.py --test discover
"""

from __future__ import annotations

import argparse
import asyncio
import json


async def test_with_client():
    """Test MCP server using FastMCP's built-in Client."""
    from fastmcp import Client

    from auroraview_mcp.server import mcp

    print("=" * 60)
    print("Testing AuroraView MCP Server with FastMCP Client")
    print("=" * 60)

    async with Client(mcp) as client:
        # List all tools
        print("\n[Tools]")
        tools = await client.list_tools()
        for tool in tools:
            print(f"  - {tool.name}: {tool.description[:60]}...")

        # List all resources
        print("\n[Resources]")
        resources = await client.list_resources()
        for resource in resources:
            print(f"  - {resource.uri}: {resource.name}")

        # Test discover_instances
        print("\n[Test: discover_instances]")
        result = await client.call_tool("discover_instances", {})
        print(f"  Result: {json.dumps(result, indent=2, default=str)[:200]}...")

        print("\n" + "=" * 60)
        print("All tests passed!")


async def test_discover():
    """Test instance discovery."""
    from fastmcp import Client

    from auroraview_mcp.server import mcp

    print("Testing instance discovery...")

    async with Client(mcp) as client:
        result = await client.call_tool("discover_instances", {})
        print(f"Found instances: {json.dumps(result, indent=2, default=str)}")


async def test_connect(port: int = 9222):
    """Test connection to a specific port."""
    from fastmcp import Client

    from auroraview_mcp.server import mcp

    print(f"Testing connection to port {port}...")

    async with Client(mcp) as client:
        result = await client.call_tool("connect", {"port": port})
        print(f"Connection result: {json.dumps(result, indent=2, default=str)}")


async def interactive_mode():
    """Interactive testing mode."""
    from fastmcp import Client

    from auroraview_mcp.server import mcp

    print("=" * 60)
    print("AuroraView MCP Interactive Debug Mode")
    print("=" * 60)
    print("Commands: tools, resources, call <tool> <json_args>, quit")
    print()

    async with Client(mcp) as client:
        while True:
            try:
                cmd = input("mcp> ").strip()
                if not cmd:
                    continue

                if cmd == "quit" or cmd == "exit":
                    break
                elif cmd == "tools":
                    tools = await client.list_tools()
                    for tool in tools:
                        print(f"  {tool.name}")
                elif cmd == "resources":
                    resources = await client.list_resources()
                    for resource in resources:
                        print(f"  {resource.uri}")
                elif cmd.startswith("call "):
                    parts = cmd[5:].split(" ", 1)
                    tool_name = parts[0]
                    args = json.loads(parts[1]) if len(parts) > 1 else {}
                    result = await client.call_tool(tool_name, args)
                    print(json.dumps(result, indent=2, default=str))
                else:
                    print(f"Unknown command: {cmd}")
            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"Error: {e}")


def main():
    parser = argparse.ArgumentParser(description="Debug AuroraView MCP Server")
    parser.add_argument(
        "--test",
        choices=["all", "discover", "connect", "interactive"],
        default="all",
        help="Test to run",
    )
    parser.add_argument("--port", type=int, default=9222, help="Port for connect test")
    args = parser.parse_args()

    if args.test == "all":
        asyncio.run(test_with_client())
    elif args.test == "discover":
        asyncio.run(test_discover())
    elif args.test == "connect":
        asyncio.run(test_connect(args.port))
    elif args.test == "interactive":
        asyncio.run(interactive_mode())


if __name__ == "__main__":
    main()
