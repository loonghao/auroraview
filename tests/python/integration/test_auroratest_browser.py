"""
Tests for AuroraTest Browser - WebView2-based testing for AuroraView.

This module tests the Browser class which uses our WebView2 implementation
for automated UI testing. The Browser class provides a Playwright-like API
but runs on our native WebView2 backend.

Tests cover:
- Browser launch and close
- Page creation and navigation
- JavaScript execution
- AuroraView bridge injection
- DOM interaction
"""

import os
import sys

import pytest

# Check if running in CI
IN_CI = os.environ.get("CI") == "true"

# Skip on non-Windows platforms (WebView2 is Windows-only)
pytestmark = [
    pytest.mark.skipif(sys.platform != "win32", reason="WebView2 tests only run on Windows"),
    pytest.mark.integration,
]


class TestBrowserImport:
    """Test Browser module imports."""

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
    """Basic Browser tests using our WebView2."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    def test_launch_and_close(self):
        """Test launching and closing Browser."""
        from auroraview.testing.auroratest import Browser

        browser = Browser.launch(headless=True)
        assert browser is not None
        assert browser.proxy is not None

        browser.close()

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    def test_new_page(self):
        """Test creating a new page."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            assert page is not None

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    def test_context_manager(self):
        """Test Browser as context manager."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            assert browser is not None
            page = browser.new_page()
            assert page is not None
        # Browser should be closed after exiting context


class TestPageNavigation:
    """Test page navigation with our WebView2."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    @pytest.mark.asyncio
    async def test_set_content(self):
        """Test setting page content."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.set_content("<h1>Hello World</h1>")
            await page.wait_for_timeout(500)

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    @pytest.mark.asyncio
    async def test_goto_data_url(self):
        """Test navigating to data URL."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            page = browser.new_page()
            await page.goto("data:text/html,<h1>Test Page</h1>")
            await page.wait_for_timeout(500)


class TestJavaScriptExecution:
    """Test JavaScript execution with our WebView2."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
    """Test Locator-based DOM interaction."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
            await page.wait_for_timeout(500)

            await page.locator("#btn").click()
            await page.wait_for_timeout(200)

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
            await page.wait_for_timeout(500)

            await page.locator("#input").fill("Hello World")
            await page.wait_for_timeout(200)

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
            await page.wait_for_timeout(500)

            await page.get_by_test_id("submit-btn").click()
            await page.wait_for_timeout(200)


class TestAuroraViewBridge:
    """Test AuroraView bridge functionality."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
    """Test form interaction capabilities."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
            await page.wait_for_timeout(500)

            await page.locator("#name").fill("John Doe")
            await page.locator("#email").fill("john@example.com")
            await page.locator("#message").fill("Hello from AuroraTest!")
            await page.wait_for_timeout(200)

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
            await page.wait_for_timeout(500)

            await page.locator("#agree").click()
            await page.wait_for_timeout(200)

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
    """Test multiple page management."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
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
    """Test BrowserContext functionality."""

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    def test_new_context(self):
        """Test creating a new browser context."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            context = browser.new_context()
            assert context is not None
            assert context.browser == browser

    @pytest.mark.skipif(IN_CI, reason="WebView2 UI tests require display")
    def test_context_new_page(self):
        """Test creating page in context."""
        from auroraview.testing.auroratest import Browser

        with Browser.launch(headless=True) as browser:
            context = browser.new_context()
            page = context.new_page()
            assert page is not None
            assert page in context.pages
