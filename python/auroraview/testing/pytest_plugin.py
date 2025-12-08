"""Pytest plugin for AuroraView headless testing.

This plugin provides pytest fixtures and markers for headless WebView testing.

Usage:
    ```python
    # In conftest.py
    pytest_plugins = ["auroraview.testing.pytest_plugin"]

    # Or install as entry point in pyproject.toml:
    # [project.entry-points.pytest11]
    # auroraview = "auroraview.testing.pytest_plugin"
    ```

Fixtures:
    - headless_webview: Auto-detected headless WebView
    - playwright_webview: Playwright-based WebView
    - xvfb_webview: Xvfb-based WebView (Linux only)

Markers:
    - @pytest.mark.webview: Mark test as WebView test
    - @pytest.mark.xvfb: Mark test to use Xvfb (Linux only)
    - @pytest.mark.playwright: Mark test to use Playwright

Example:
    ```python
    import pytest

    @pytest.mark.webview
    def test_button_click(headless_webview):
        headless_webview.load_html("<button id='btn'>Click</button>")
        headless_webview.click("#btn")

    @pytest.mark.playwright
    def test_with_playwright(playwright_webview):
        playwright_webview.goto("https://example.com")
        assert playwright_webview.text("h1") == "Example Domain"
    ```
"""

from __future__ import annotations

import os
import platform
from typing import TYPE_CHECKING, Generator

import pytest

if TYPE_CHECKING:
    from .headless_webview import HeadlessWebViewBase


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line("markers", "webview: mark test as WebView test")
    config.addinivalue_line("markers", "xvfb: mark test to use Xvfb virtual display (Linux only)")
    config.addinivalue_line("markers", "playwright: mark test to use Playwright browser")
    config.addinivalue_line("markers", "webview2_cdp: mark test to use WebView2 CDP connection")


def pytest_collection_modifyitems(config, items):
    """Skip tests based on platform and available tools."""
    skip_xvfb = pytest.mark.skip(reason="Xvfb only available on Linux")
    skip_no_xvfb = pytest.mark.skip(reason="Xvfb not installed")
    skip_no_playwright = pytest.mark.skip(reason="Playwright not installed")

    for item in items:
        # Skip Xvfb tests on non-Linux
        if "xvfb" in item.keywords:
            if platform.system() != "Linux":
                item.add_marker(skip_xvfb)
            elif not _has_xvfb():
                item.add_marker(skip_no_xvfb)

        # Skip Playwright tests if not installed
        if "playwright" in item.keywords:
            if not _has_playwright():
                item.add_marker(skip_no_playwright)


def _has_xvfb() -> bool:
    """Check if Xvfb is available."""
    import shutil

    return shutil.which("Xvfb") is not None


def _has_playwright() -> bool:
    """Check if Playwright is installed."""
    try:
        import importlib.util

        return importlib.util.find_spec("playwright") is not None
    except (ImportError, ModuleNotFoundError):
        return False


@pytest.fixture(scope="function")
def headless_webview() -> Generator["HeadlessWebViewBase", None, None]:
    """Fixture providing auto-detected headless WebView.

    This fixture automatically selects the best headless mode:
    1. WebView2 CDP if WEBVIEW2_CDP_URL is set
    2. Xvfb if on Linux with AURORAVIEW_USE_XVFB=1
    3. Playwright (default)

    Example:
        ```python
        def test_example(headless_webview):
            headless_webview.goto("https://example.com")
            assert headless_webview.text("h1") == "Example Domain"
        ```
    """
    from .headless_webview import HeadlessWebView

    webview = HeadlessWebView.auto()
    try:
        yield webview
    finally:
        webview.close()


@pytest.fixture(scope="function")
def playwright_webview() -> Generator["HeadlessWebViewBase", None, None]:
    """Fixture providing Playwright-based headless WebView.

    Example:
        ```python
        @pytest.mark.playwright
        def test_with_playwright(playwright_webview):
            playwright_webview.goto("https://example.com")
        ```
    """
    from .headless_webview import HeadlessWebView

    webview = HeadlessWebView.playwright()
    try:
        yield webview
    finally:
        webview.close()


@pytest.fixture(scope="function")
def xvfb_webview() -> Generator["HeadlessWebViewBase", None, None]:
    """Fixture providing Xvfb-based headless WebView (Linux only).

    Example:
        ```python
        @pytest.mark.xvfb
        def test_with_xvfb(xvfb_webview):
            xvfb_webview.load_html("<h1>Test</h1>")
        ```
    """
    from .headless_webview import HeadlessWebView

    webview = HeadlessWebView.virtual_display()
    try:
        yield webview
    finally:
        webview.close()


@pytest.fixture(scope="function")
def webview2_cdp_webview() -> Generator["HeadlessWebViewBase", None, None]:
    """Fixture providing WebView2 CDP-connected headless WebView.

    Requires WEBVIEW2_CDP_URL environment variable or a WebView2
    instance running with --remote-debugging-port=9222.

    Example:
        ```python
        @pytest.mark.webview2_cdp
        def test_with_webview2(webview2_cdp_webview):
            webview2_cdp_webview.goto("https://example.com")
        ```
    """
    from .headless_webview import HeadlessWebView

    cdp_url = os.environ.get("WEBVIEW2_CDP_URL", "http://localhost:9222")
    webview = HeadlessWebView.webview2_cdp(cdp_url)
    try:
        yield webview
    finally:
        webview.close()


# Session-scoped fixtures for performance


@pytest.fixture(scope="session")
def playwright_browser():
    """Session-scoped Playwright browser for performance.

    Use this when you need to run many tests and want to reuse
    the browser instance.

    Example:
        ```python
        def test_example(playwright_browser):
            page = playwright_browser.new_page()
            page.goto("https://example.com")
            page.close()
        ```
    """
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        pytest.skip("Playwright not installed")

    pw = sync_playwright().start()
    browser = pw.chromium.launch(headless=True)

    yield browser

    browser.close()
    pw.stop()


@pytest.fixture(scope="function")
def playwright_page(playwright_browser):
    """Function-scoped Playwright page using session browser.

    Example:
        ```python
        def test_example(playwright_page):
            playwright_page.goto("https://example.com")
            assert playwright_page.title() == "Example Domain"
        ```
    """
    context = playwright_browser.new_context()

    # Inject AuroraView bridge
    from .headless_webview import _get_bridge_script

    context.add_init_script(_get_bridge_script())

    page = context.new_page()

    yield page

    context.close()
