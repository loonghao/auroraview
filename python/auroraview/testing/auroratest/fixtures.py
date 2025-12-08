"""
Pytest fixtures for AuroraTest.

Provides ready-to-use fixtures for testing AuroraView applications.
"""

from __future__ import annotations

import logging
import os
from typing import TYPE_CHECKING, Generator

import pytest

if TYPE_CHECKING:
    from .browser import Browser, BrowserContext
    from .page import Page

logger = logging.getLogger(__name__)


@pytest.fixture(scope="session")
def browser() -> Generator["Browser", None, None]:
    """
    Session-scoped browser fixture.

    Creates a single browser instance for all tests in the session.

    Example:
        ```python
        def test_example(browser):
            page = browser.new_page()
            await page.goto("https://example.com")
        ```
    """
    from .browser import Browser

    # Check for CI environment
    is_ci = os.environ.get("CI", "").lower() in ("true", "1", "yes")

    browser = Browser.launch(
        headless=is_ci or os.environ.get("HEADLESS", "").lower() in ("true", "1", "yes"),
        slow_mo=float(os.environ.get("SLOW_MO", "0")),
    )

    yield browser

    browser.close()


@pytest.fixture(scope="function")
def context(browser: "Browser") -> Generator["BrowserContext", None, None]:
    """
    Function-scoped browser context fixture.

    Creates a new isolated context for each test.

    Example:
        ```python
        def test_isolated(context):
            page = context.new_page()
            # This test has isolated cookies/storage
        ```
    """
    context = browser.new_context()

    yield context

    context.close()


@pytest.fixture(scope="function")
def page(browser: "Browser") -> Generator["Page", None, None]:
    """
    Function-scoped page fixture.

    Creates a new page for each test.

    Example:
        ```python
        async def test_navigation(page):
            await page.goto("https://example.com")
            await expect(page).to_have_title("Example")
        ```
    """
    page = browser.new_page()

    yield page

    page.close()


@pytest.fixture(scope="function")
def page_with_context(context: "BrowserContext") -> Generator["Page", None, None]:
    """
    Function-scoped page fixture with isolated context.

    Example:
        ```python
        async def test_with_context(page_with_context):
            # Page with isolated cookies/storage
            await page_with_context.goto("https://example.com")
        ```
    """
    page = context.new_page()

    yield page

    page.close()


# ========== Async Fixtures ==========


@pytest.fixture(scope="session")
async def async_browser() -> "Browser":
    """
    Async session-scoped browser fixture.

    For use with pytest-asyncio.
    """
    from .browser import Browser

    is_ci = os.environ.get("CI", "").lower() in ("true", "1", "yes")

    browser = Browser.launch(
        headless=is_ci or os.environ.get("HEADLESS", "").lower() in ("true", "1", "yes"),
    )

    yield browser

    browser.close()


@pytest.fixture(scope="function")
async def async_page(async_browser: "Browser") -> "Page":
    """
    Async function-scoped page fixture.

    For use with pytest-asyncio.
    """
    page = async_browser.new_page()

    yield page

    page.close()


# ========== Configuration ==========


def pytest_addoption(parser):
    """Add AuroraTest command line options."""
    group = parser.getgroup("auroratest")

    group.addoption(
        "--headed",
        action="store_true",
        default=False,
        help="Run tests in headed mode (show browser window)",
    )

    group.addoption(
        "--slow-mo",
        type=float,
        default=0,
        help="Slow down operations by specified milliseconds",
    )

    group.addoption(
        "--screenshot-on-failure",
        action="store_true",
        default=False,
        help="Take screenshot on test failure",
    )

    group.addoption(
        "--video",
        action="store_true",
        default=False,
        help="Record video of test execution",
    )

    group.addoption(
        "--trace",
        action="store_true",
        default=False,
        help="Record trace for debugging",
    )


@pytest.fixture(scope="session")
def browser_options(request) -> dict:
    """
    Get browser options from command line.

    Example:
        ```python
        def test_with_options(browser_options):
            print(f"Headless: {browser_options['headless']}")
        ```
    """
    return {
        "headless": not request.config.getoption("--headed"),
        "slow_mo": request.config.getoption("--slow-mo"),
    }


# ========== Hooks ==========


@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_makereport(item, call):
    """
    Hook to capture screenshots on test failure.
    """
    outcome = yield
    report = outcome.get_result()

    if report.when == "call" and report.failed:
        # Check if screenshot-on-failure is enabled
        if item.config.getoption("--screenshot-on-failure", default=False):
            # Try to get page fixture
            page = item.funcargs.get("page") or item.funcargs.get("async_page")
            if page and not page.is_closed():
                try:
                    import asyncio

                    screenshot_dir = item.config.rootdir / "test-results" / "screenshots"
                    screenshot_dir.mkdir(parents=True, exist_ok=True)

                    screenshot_path = screenshot_dir / f"{item.name}.png"

                    # Run async screenshot
                    loop = asyncio.get_event_loop()
                    loop.run_until_complete(page.screenshot(path=str(screenshot_path)))

                    logger.info(f"Screenshot saved: {screenshot_path}")
                except Exception as e:
                    logger.warning(f"Failed to capture screenshot: {e}")
