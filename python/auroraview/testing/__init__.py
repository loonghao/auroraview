"""
AuroraView Testing Framework

A pytest-qt inspired testing framework for AuroraView WebView applications.

This module provides fixtures, utilities, and helpers for testing WebView-based
applications with a focus on UI automation and DOM verification.

Features:
    - **AuroraTest**: Playwright-like testing API (auroratest submodule)
    - **DomAssertions**: Real value verification using the DOM API
    - **HeadlessTestRunner**: Run tests without visible windows
    - **WebViewBot**: High-level automation API
    - **Pytest fixtures**: Ready-to-use fixtures for common patterns

Example (AuroraTest - Playwright-like API):
    ```python
    from auroraview.testing.auroratest import Browser, expect

    async def test_login():
        browser = Browser.launch(headless=True)
        page = browser.new_page()

        await page.goto("https://example.com/login")
        await page.locator("#email").fill("test@example.com")
        await page.locator("#password").fill("secret")
        await page.get_by_role("button", name="Login").click()

        await expect(page.locator(".welcome")).to_have_text("Welcome!")

        browser.close()
    ```

Example (DOM Assertions):
    ```python
    from auroraview.testing import DomAssertions

    def test_form(webview):
        assertions = DomAssertions(webview)
        assertions.assert_text("#title", "Welcome")
        assertions.assert_value("#email", "test@example.com")
        assertions.wait_for_visible("#modal", timeout=2)
    ```

Example (Headless Testing):
    ```python
    from auroraview.testing import headless_test

    def test_button_click():
        with headless_test(html="<button id='btn'>Click</button>") as runner:
            runner.click("#btn")
            runner.assertions.assert_has_class("#btn", "clicked")
    ```

Example (WebViewBot - Legacy):
    ```python
    from auroraview.testing import webview, webview_bot

    def test_window_dragging(webview, webview_bot):
        webview.load_html(test_html)
        webview_bot.wait_for_event('webview_ready', timeout=5)
        webview_bot.drag('.title-bar', offset=(100, 50))
    ```
"""

from .assertions import (
    assert_element_exists,
    assert_element_text,
    assert_event_emitted,
    assert_window_title,
)
from .dom_assertions import DomAssertions
from .fixtures import (
    draggable_window_html,
    headless_webview,
    test_html,
    webview,
    webview_bot,
    webview_with_html,
)
from .headless import HeadlessTestRunner, headless_test
from .webview_bot import EventRecord, WebViewBot

# Import auroratest submodule for Playwright-like API
from . import auroratest

__all__ = [
    # Playwright-like testing (auroratest submodule)
    "auroratest",
    # New DOM-based testing
    "DomAssertions",
    "HeadlessTestRunner",
    "headless_test",
    # Legacy WebViewBot
    "WebViewBot",
    "EventRecord",
    # Pytest fixtures
    "webview",
    "webview_bot",
    "webview_with_html",
    "headless_webview",
    "test_html",
    "draggable_window_html",
    # Legacy assertions
    "assert_event_emitted",
    "assert_element_exists",
    "assert_element_text",
    "assert_window_title",
]
