"""
AuroraView Testing Framework

A comprehensive testing framework for AuroraView WebView applications.

This module provides multiple testing approaches:

1. **HeadlessWebView** (recommended): Unified headless testing with multiple backends
   - Playwright mode (fast, cross-platform)
   - Xvfb mode (real WebView on Linux)
   - WebView2 CDP mode (real WebView2 on Windows)

2. **AuroraTest**: Playwright-like API for our native WebView
   - Browser, Page, Locator classes
   - expect() assertions

3. **PlaywrightBrowser**: Direct Playwright Chromium for frontend testing

Example (HeadlessWebView - recommended):
    ```python
    from auroraview.testing import HeadlessWebView

    with HeadlessWebView.playwright() as webview:
        webview.goto("https://example.com")
        webview.click("#button")
        assert webview.text("#result") == "Success"
    ```

Example (AuroraTest - Playwright-like API):
    ```python
    from auroraview.testing.auroratest import Browser, expect

    async def test_login():
        browser = Browser.launch(headless=True)
        page = browser.new_page()

        await page.goto("https://example.com/login")
        await page.locator("#email").fill("test@example.com")
        await page.get_by_role("button", name="Login").click()

        await expect(page.locator(".welcome")).to_have_text("Welcome!")

        browser.close()
    ```

Example (Pytest fixture):
    ```python
    def test_example(headless_webview):
        headless_webview.load_html("<h1>Test</h1>")
        assert headless_webview.text("h1") == "Test"
    ```
"""

from .dom_assertions import DomAssertions
from .headless_webview import (
    HeadlessOptions,
    HeadlessWebView,
    HeadlessWebViewBase,
    PlaywrightHeadlessWebView,
    VirtualDisplayWebView,
    WebView2CDPWebView,
    headless_webview,
)

# Import auroratest submodule for Playwright-like API
from . import auroratest

__all__ = [
    # Playwright-like testing (auroratest submodule)
    "auroratest",
    # Headless WebView testing (multiple backends)
    "HeadlessWebView",
    "HeadlessOptions",
    "HeadlessWebViewBase",
    "PlaywrightHeadlessWebView",
    "VirtualDisplayWebView",
    "WebView2CDPWebView",
    "headless_webview",
    # DOM assertions
    "DomAssertions",
]
