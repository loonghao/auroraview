"""
Pytest configuration for AuroraView tests.

This module provides:
- Event loop configuration for Windows
- Common test utilities
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

# Fix for Playwright on Windows with pytest-asyncio
# Playwright's sync API needs ProactorEventLoop for subprocess support
if sys.platform == "win32":
    import asyncio

    # Set the default event loop policy to ProactorEventLoop
    # This is required for Playwright's subprocess spawning
    asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())

# Add project paths
# In CI, the wheel is installed to site-packages, so we should use that.
# Only add source paths for local development when the package is not installed.
PROJECT_ROOT = Path(__file__).parent.parent


def _add_sys_path(path: Path) -> None:
    path_str = str(path)
    if path_str not in sys.path:
        sys.path.insert(0, path_str)


def _is_package_installed() -> bool:
    """Check if auroraview is installed as a package (with _core module)."""
    try:
        # Try to import from site-packages first
        import importlib.util

        spec = importlib.util.find_spec("auroraview._core")
        return spec is not None
    except (ImportError, ModuleNotFoundError):
        return False


# Only add source paths if the package is not installed
# This allows CI to use the installed wheel while local dev uses source
if not _is_package_installed():
    _add_sys_path(PROJECT_ROOT / "python")
    _add_sys_path(PROJECT_ROOT)

# Always add gallery path for gallery-related tests
_add_sys_path(PROJECT_ROOT / "gallery")


@pytest.fixture(scope="session", autouse=True)
def setup_event_loop_policy():
    """Ensure correct event loop policy for Playwright."""
    if sys.platform == "win32":
        import asyncio

        asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    yield


# ─────────────────────────────────────────────────────────────────────────────
# Test Markers
# ─────────────────────────────────────────────────────────────────────────────


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line("markers", "cdp: Tests using Chrome DevTools Protocol")
    config.addinivalue_line("markers", "e2e: End-to-end tests")
    config.addinivalue_line("markers", "slow: Slow tests")
    config.addinivalue_line("markers", "integration: Integration tests")
