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
    config.addinivalue_line("markers", "qt: mark test as requiring Qt")
    config.addinivalue_line("markers", "e2e: mark test as end-to-end test")


@pytest.fixture(scope="session")
def event_loop_policy():
    """Use default event loop policy."""
    import asyncio

    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    return asyncio.get_event_loop_policy()


# Qt testing fixtures (only available when pytest-qt is installed)
@pytest.fixture
def qt_webview(qtbot):
    """Provide a QtWebView instance for testing.

    This fixture requires pytest-qt to be installed.
    Usage:
        def test_qt_feature(qt_webview):
            qt_webview.load_html("<h1>Test</h1>")
    """
    from auroraview.testing.qt import create_qt_webview

    webview = create_qt_webview(qtbot)
    yield webview
    webview.close()


@pytest.fixture
def qt_helper(qtbot):
    """Provide a QtWebViewTestHelper for testing.

    This fixture requires pytest-qt to be installed.
    Usage:
        def test_with_helper(qt_helper):
            webview = qt_helper.create_webview(html="<h1>Test</h1>")
            qt_helper.wait_loaded(webview)
    """
    from auroraview.testing.qt import QtWebViewTestHelper

    helper = QtWebViewTestHelper(qtbot)
    yield helper
    helper.cleanup()
