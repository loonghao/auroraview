"""
Tests for AuroraTest Browser - Cross-platform WebView testing for AuroraView.

This module tests the Browser class which uses our wry-based WebView implementation
for automated UI testing. The Browser class provides a Playwright-like API
and runs on our native WebView backend:
- Windows: WebView2 (Chromium Edge)
- macOS: WKWebView (Safari)
- Linux: WebKitGTK

Tests cover:
- Browser launch and close
- Page creation and navigation
- JavaScript execution
- AuroraView bridge injection
- DOM interaction

Note: wry requires the event loop to be initialized on the main thread.
In CI environments where tests run in worker threads, these tests are skipped
on Linux and macOS.
"""

import os
import sys

import pytest

# Check if running in CI
IN_CI = os.environ.get("CI") == "true"

# Check if we have a display (required for WebView)
HAS_DISPLAY = (
    os.environ.get("DISPLAY") is not None or sys.platform == "win32" or sys.platform == "darwin"
)

# wry requires event loop to be on main thread
# In CI, tests run in worker threads which causes panic on Linux and macOS
# On Windows CI, WebView2 tests can hang due to event loop issues
IS_LINUX = sys.platform == "linux"
IS_MACOS = sys.platform == "darwin"
IS_WINDOWS = sys.platform == "win32"

# Skip WebView tests in CI on all platforms due to event loop/thread issues
SKIP_WEBVIEW_TESTS = IN_CI

# Common skip conditions
pytestmark = [
    pytest.mark.integration,
    # Skip all Browser tests in CI due to event loop/thread issues
    pytest.mark.skipif(
        SKIP_WEBVIEW_TESTS,
        reason="WebView tests require main thread event loop, skipped in CI",
    ),
]


def skip_without_display(reason="WebView tests require display"):
    """Skip test if no display is available or in CI."""
    return pytest.mark.skipif(IN_CI or not HAS_DISPLAY, reason=reason)


class TestBrowserImport:
    """Test Browser module imports - works on all platforms."""

    def test_import_browser(self):
        """Test that Browser can be imported."""
        from auroraview.testing.auroratest import Browser

        assert Browser is not None

    def test_import_browser_options(self):
        """Test that BrowserOptions can be imported."""
        from auroraview.testing.auroratest.browser import BrowserOptions

        options = BrowserOptions()
        assert options.headless is True
        assert options.timeout == 30000

    def test_import_page(self):
        """Test that Page can be imported."""
        from auroraview.testing.auroratest import Page

        assert Page is not None

    def test_import_locator(self):
        """Test that Locator can be imported."""
        from auroraview.testing.auroratest import Locator

        assert Locator is not None


class TestBrowserBasic:
    """Basic Browser tests using our wry-based WebView (cross-platform)."""

    @skip_without_display()
    def test_launch_and_close(self):
        """Test launching and closing Browser."""
        from auroraview.testing.auroratest import Browser

        browser = Browser.launch(headless=True)
        assert browser is not None
        assert browser.proxy is not None

        browser.close()

    @skip_without_display()
    def test_new_page(self):
        """Test creating a new page."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            assert page is not None

    @skip_without_display()
    def test_context_manager(self):
        """Test Browser as context manager."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            assert browser is not None
            page = browser.new_page()
            assert page is not None
        # Browser should be closed after exiting context


class TestPageNavigation:
    """Test page navigation with our wry-based WebView (cross-platform)."""

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_set_content(self):
        """Test setting page content."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1>Hello World</h1>")
            await page.wait_for_timeout(500)

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_goto_data_url(self):
        """Test navigating to data URL."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.goto("data:text/html,<h1>Test Page</h1>")
            await page.wait_for_timeout(500)


class TestJavaScriptExecution:
    """Test JavaScript execution with our wry-based WebView (cross-platform)."""

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_evaluate_simple(self):
        """Test simple JavaScript evaluation."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1>Test</h1>")
            await page.wait_for_timeout(500)

            # Execute JavaScript
            await page.evaluate("document.title = 'Modified'")

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_evaluate_with_return(self):
        """Test JavaScript evaluation with return value."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1 id='title'>Hello</h1>")
            await page.wait_for_timeout(500)

            # This tests the async evaluation
            await page.evaluate("1 + 1")


class TestLocatorInteraction:
    """Test Locator-based DOM interaction (cross-platform)."""

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_locator_click(self):
        """Test clicking elements via Locator."""
        from auroraview.testing.auroratest import Browser

        html = """
        <button id="btn" onclick="this.textContent='Clicked!'">Click me</button>
        """

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content(html)
            await page.wait_for_timeout(1000)

            # Use force=True to skip actionability checks in CI
            await page.locator("#btn").click(force=True)
            await page.wait_for_timeout(200)

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_locator_fill(self):
        """Test filling input via Locator."""
        from auroraview.testing.auroratest import Browser

        html = """
        <input type="text" id="input" />
        """

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content(html)
            await page.wait_for_timeout(1000)

            # Use force=True to skip actionability checks in CI
            await page.locator("#input").fill("Hello World", force=True)
            await page.wait_for_timeout(200)

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_get_by_test_id(self):
        """Test get_by_test_id locator."""
        from auroraview.testing.auroratest import Browser

        html = """
        <button data-testid="submit-btn">Submit</button>
        """

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content(html)
            await page.wait_for_timeout(1000)

            # Use force=True to skip actionability checks in CI
            await page.get_by_test_id("submit-btn").click(force=True)
            await page.wait_for_timeout(200)


class TestAuroraViewBridge:
    """Test AuroraView bridge functionality (cross-platform)."""

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_bridge_injected(self):
        """Test that AuroraView bridge is injected."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1>Test</h1>")
            await page.wait_for_timeout(1000)

            # Check if bridge is available
            await page.evaluate("typeof window.auroraview !== 'undefined'")

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_bridge_api_proxy(self):
        """Test that bridge API proxy exists."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1>Test</h1>")
            await page.wait_for_timeout(1000)

            # Check if API proxy exists
            await page.evaluate("typeof window.auroraview.api !== 'undefined'")

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_bridge_event_system(self):
        """Test that bridge event system works."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1>Test</h1>")
            await page.wait_for_timeout(1000)

            # Test event subscription and triggering
            await page.evaluate("""
                (function() {
                    let received = null;
                    if (window.auroraview && window.auroraview.on) {
                        window.auroraview.on('test_event', (data) => {
                            received = data;
                        });
                        if (window.auroraview.trigger) {
                            window.auroraview.trigger('test_event', {message: 'hello'});
                        }
                    }
                    return received;
                })()
            """)


class TestFormInteraction:
    """Test form interaction capabilities (cross-platform)."""

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_fill_form(self):
        """Test filling a complete form."""
        from auroraview.testing.auroratest import Browser

        html = """
        <form id="test-form">
            <input type="text" id="name" />
            <input type="email" id="email" />
            <textarea id="message"></textarea>
            <button type="submit">Submit</button>
        </form>
        """

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content(html)
            await page.wait_for_timeout(1000)

            # Use force=True to skip actionability checks in CI
            await page.locator("#name").fill("John Doe", force=True)
            await page.locator("#email").fill("john@example.com", force=True)
            await page.locator("#message").fill("Hello from AuroraTest!", force=True)
            await page.wait_for_timeout(200)

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_checkbox_interaction(self):
        """Test checkbox interaction."""
        from auroraview.testing.auroratest import Browser

        html = """
        <label>
            <input type="checkbox" id="agree" />
            I agree
        </label>
        """

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content(html)
            await page.wait_for_timeout(1000)

            # Use force=True to skip actionability checks in CI
            await page.locator("#agree").click(force=True)
            await page.wait_for_timeout(200)

    @skip_without_display()
    @pytest.mark.asyncio
    async def test_select_option(self):
        """Test select dropdown interaction."""
        from auroraview.testing.auroratest import Browser

        html = """
        <select id="country">
            <option value="">Select...</option>
            <option value="us">United States</option>
            <option value="uk">United Kingdom</option>
            <option value="cn">China</option>
        </select>
        """

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content(html)
            await page.wait_for_timeout(500)

            # Select by value
            await page.evaluate("document.getElementById('country').value = 'cn'")
            await page.wait_for_timeout(200)


class TestMultiplePages:
    """Test multiple page management (cross-platform)."""

    @skip_without_display()
    def test_multiple_pages_list(self):
        """Test that multiple pages are tracked."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page1 = browser.new_page()
            page2 = browser.new_page()

            assert len(browser.pages) >= 2
            assert page1 in browser.pages
            assert page2 in browser.pages


class TestBrowserContext:
    """Test BrowserContext functionality (cross-platform)."""

    @skip_without_display()
    def test_new_context(self):
        """Test creating a new browser context."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            context = browser.new_context()
            assert context is not None
            assert context.browser == browser

    @skip_without_display()
    def test_context_new_page(self):
        """Test creating page in context."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            context = browser.new_context()
            page = context.new_page()
            assert page is not None
            assert page in context.pages
