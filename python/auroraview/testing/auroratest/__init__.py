"""
AuroraTest - Playwright-like Testing Framework for AuroraView

A Playwright-inspired testing framework for AuroraView WebView applications.
Supports headless testing, screenshots, network interception, and CI/CD integration.

Future PyPI package: auroraview-test

Example:
    ```python
    from auroraview.testing.auroratest import Browser, expect

    async def test_login():
        browser = Browser.launch(headless=True)
        page = browser.new_page()

        await page.goto("https://auroraview.localhost/login.html")
        await page.locator("#email").fill("test@example.com")
        await page.locator("#password").fill("secret")
        await page.get_by_role("button", name="Login").click()

        await expect(page.locator(".welcome")).to_have_text("Welcome!")

        browser.close()
    ```

For full Playwright API access (recommended for testing):
    ```python
    from auroraview.testing.auroratest import PlaywrightBrowser

    browser = PlaywrightBrowser.launch(headless=True)
    page = browser.new_page()

    page.goto("https://example.com")
    page.locator("#button").click()
    page.screenshot(path="screenshot.png")

    browser.close()
    ```
"""

from .browser import Browser, BrowserContext, BrowserOptions
from .page import Page
from .locator import Locator
from .expect import expect
from .network import Route, Request, Response
from .fixtures import browser, page, context

# Playwright CDP-based browser (recommended for testing)
try:
    from .playwright_browser import PlaywrightBrowser, PlaywrightBrowserOptions
except ImportError:
    # Playwright not installed
    PlaywrightBrowser = None  # type: ignore
    PlaywrightBrowserOptions = None  # type: ignore

__all__ = [
    # Core classes
    "Browser",
    "BrowserContext",
    "BrowserOptions",
    "Page",
    "Locator",
    # Playwright CDP browser (recommended)
    "PlaywrightBrowser",
    "PlaywrightBrowserOptions",
    # Assertions
    "expect",
    # Network
    "Route",
    "Request",
    "Response",
    # Fixtures
    "browser",
    "page",
    "context",
]
