"""
AuroraView Playwright-like Testing Framework

A Playwright-inspired testing framework for AuroraView WebView applications.
Supports headless testing, screenshots, network interception, and CI/CD integration.

Example:
    ```python
    from auroraview.testing.playwright import Browser, expect

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
"""

from .browser import Browser, BrowserContext
from .page import Page
from .locator import Locator
from .expect import expect
from .network import Route, Request, Response
from .fixtures import browser, page, context

__all__ = [
    # Core classes
    "Browser",
    "BrowserContext", 
    "Page",
    "Locator",
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
