"""
Pytest configuration and fixtures for AuroraView tests

This module registers all test fixtures and configures pytest for AuroraView testing.
"""

import os
import sys

import pytest

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

# Import fixtures from auroraview.testing
from auroraview.testing import (
    draggable_window_html,
    headless_webview,
    test_html,
    webview,
    webview_bot,
    webview_with_html,
)

# Re-export fixtures so pytest can discover them
__all__ = [
    "webview",
    "webview_bot",
    "webview_with_html",
    "headless_webview",
    "test_html",
    "draggable_window_html",
]


def pytest_configure(config):
    """
    Pytest configuration hook.

    Registers custom markers and configures test environment.
    """
    # Register custom markers
    config.addinivalue_line("markers", "ui: mark test as UI test (requires display)")
    config.addinivalue_line("markers", "slow: mark test as slow running")
    config.addinivalue_line("markers", "maya: mark test as requiring Maya")
    config.addinivalue_line("markers", "headless: mark test as headless (no display required)")


def pytest_collection_modifyitems(config, items):
    """
    Pytest hook to modify test collection.

    Automatically marks tests based on their names and locations.
    """
    for item in items:
        # Auto-mark UI tests
        if "test_window" in item.nodeid or "test_ui" in item.nodeid:
            item.add_marker(pytest.mark.ui)

        # Auto-mark Maya tests
        if "maya" in item.nodeid.lower():
            item.add_marker(pytest.mark.maya)


@pytest.fixture(scope="session", autouse=True)
def setup_test_environment():
    """
    Session-level fixture to setup test environment.

    Runs once before all tests.
    """
    print("\n" + "=" * 80)
    print("Setting up AuroraView test environment...")
    print("=" * 80)

    # Setup code here

    yield

    # Teardown code here
    print("\n" + "=" * 80)
    print("Tearing down AuroraView test environment...")
    print("=" * 80)


@pytest.fixture(autouse=True)
def test_logger(request):
    """
    Auto-use fixture that logs test start/end.

    Provides better visibility into test execution.
    """
    test_name = request.node.name
    print(f"\n{'=' * 80}")
    print(f"[START] test: {test_name}")
    print(f"{'=' * 80}")

    yield

    print(f"\n{'=' * 80}")
    print(f"[END] test: {test_name}")
    print(f"{'=' * 80}")
