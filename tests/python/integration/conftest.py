"""
Pytest configuration for AuroraView integration tests.
"""

import os
import sys

import pytest

# Ensure auroraview is importable
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "python"))

# Import fixtures from auroraview.testing to make them available to tests
from auroraview.testing.fixtures import (  # noqa: E402, F401
    draggable_window_html,
    form_html,
    headless_webview,
    playwright_webview,
    test_html,
)


def pytest_configure(config):
    """Configure pytest for AuroraView integration tests."""
    config.addinivalue_line("markers", "asyncio: mark test as async")
    config.addinivalue_line("markers", "slow: mark test as slow running")
    config.addinivalue_line("markers", "ui: mark test as requiring UI")
    config.addinivalue_line("markers", "webview: mark test as requiring WebView")
    config.addinivalue_line("markers", "playwright: mark test as using Playwright")


@pytest.fixture(scope="session")
def event_loop_policy():
    """Use default event loop policy."""
    import asyncio

    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    return asyncio.get_event_loop_policy()
