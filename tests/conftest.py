"""
Pytest configuration for AuroraView tests.
"""

import sys

import pytest

# Fix for Playwright on Windows with pytest-asyncio
# Playwright's sync API needs ProactorEventLoop for subprocess support
if sys.platform == "win32":
    import asyncio

    # Set the default event loop policy to ProactorEventLoop
    # This is required for Playwright's subprocess spawning
    asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())


@pytest.fixture(scope="session", autouse=True)
def setup_event_loop_policy():
    """Ensure correct event loop policy for Playwright."""
    if sys.platform == "win32":
        import asyncio

        asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    yield
