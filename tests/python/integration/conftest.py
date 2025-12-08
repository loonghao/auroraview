"""
Pytest configuration for AuroraTest integration tests.
"""

import os
import sys

import pytest

# Ensure auroraview is importable
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "python"))


def pytest_configure(config):
    """Configure pytest for AuroraTest integration tests."""
    config.addinivalue_line("markers", "asyncio: mark test as async")
    config.addinivalue_line("markers", "slow: mark test as slow running")


@pytest.fixture(scope="session")
def event_loop_policy():
    """Use default event loop policy."""
    import asyncio

    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    return asyncio.get_event_loop_policy()
