"""Entry point for auroraview-mcp CLI."""

import asyncio
import sys


def main() -> int:
    """Run the AuroraView MCP Server."""
    from auroraview_mcp.server import mcp

    asyncio.run(mcp.run())
    return 0


if __name__ == "__main__":
    sys.exit(main())
