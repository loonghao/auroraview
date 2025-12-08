"""
Tests for PlaywrightBrowser - Playwright-based testing for AuroraView.

This module tests the PlaywrightBrowser class which provides two modes:
1. Native mode: Uses Playwright's built-in Chromium (requires `playwright install`)
2. CDP mode: Connects to our WebView2 via Chrome DevTools Protocol

The Native mode tests are skipped in CI since they require browser installation.
The CDP mode tests verify the actual WebView2 integration.
"""

import os
import sys

import pytest

# Check if playwright is available
try:
    from importlib.util import find_spec

    PLAYWRIGHT_AVAILABLE = find_spec("playwright") is not None
except ImportError:
    PLAYWRIGHT_AVAILABLE = False

# Check if running in CI
IN_CI = os.environ.get("CI") == "true"

# Common skip conditions
pytestmark = [
    pytest.mark.skipif(sys.version_info < (3, 8), reason="Playwright requires Python 3.8+"),
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.integration,
]


class TestPlaywrightBrowserImport:
    """Test PlaywrightBrowser module imports."""

    def test_import(self):
        """Test that PlaywrightBrowser can be imported."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        assert PlaywrightBrowser is not None

    def test_import_options(self):
        """Test that PlaywrightBrowserOptions can be imported."""
        from auroraview.testing.auroratest.playwright_browser import PlaywrightBrowserOptions

        options = PlaywrightBrowserOptions()
        assert options.headless is True
        assert options.inject_bridge is True

    def test_bridge_script_exists(self):
        """Test that the AuroraView bridge script is defined."""
        from auroraview.testing.auroratest.playwright_browser import AURORAVIEW_BRIDGE_SCRIPT

        assert AURORAVIEW_BRIDGE_SCRIPT is not None
        assert "window.auroraview" in AURORAVIEW_BRIDGE_SCRIPT
        assert "auroraviewready" in AURORAVIEW_BRIDGE_SCRIPT


class TestPlaywrightBrowserNative:
    """
    Tests for PlaywrightBrowser Native mode (uses Playwright's Chromium).

    These tests are skipped in CI because they require `playwright install chromium`.
    They test the testing tool itself, not core AuroraView functionality.
    """

    # Skip in CI - these require browser installation
    pytestmark = pytest.mark.skipif(
        IN_CI,
        reason="Native mode tests skipped in CI (requires `playwright install chromium`)"
    )

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_launch_and_close(self):
        """Test launching and closing PlaywrightBrowser."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        browser = PlaywrightBrowser.launch(headless=True)
        assert browser is not None
        assert browser.browser is not None

        browser.close()

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_new_page(self):
        """Test creating a new page."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            assert page is not None

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_navigate_to_url(self):
        """Test navigating to a URL."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Hello World</h1>")

            content = page.content()
            assert "Hello World" in content

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_locator_and_click(self):
        """Test using locators and clicking elements."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<button id='btn'>Click me</button>")

            btn = page.locator("#btn")
            assert btn.text_content() == "Click me"
            btn.click()

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_evaluate_javascript(self):
        """Test evaluating JavaScript in page."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            result = page.evaluate("1 + 1")
            assert result == 2

            title = page.evaluate("document.querySelector('h1').textContent")
            assert title == "Test"

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_screenshot(self):
        """Test taking screenshots."""
        import tempfile

        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Screenshot Test</h1>")

            with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
                screenshot_path = f.name

            try:
                page.screenshot(path=screenshot_path)
                assert os.path.exists(screenshot_path)
                assert os.path.getsize(screenshot_path) > 0
            finally:
                if os.path.exists(screenshot_path):
                    os.unlink(screenshot_path)


class TestPlaywrightBrowserAdvanced:
    """Advanced PlaywrightBrowser tests (Native mode)."""

    # Skip in CI
    pytestmark = pytest.mark.skipif(
        IN_CI,
        reason="Native mode tests skipped in CI (requires `playwright install chromium`)"
    )

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_fill_form(self):
        """Test filling form inputs."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        html = """
        <form>
            <input type="text" id="name" />
            <input type="email" id="email" />
            <textarea id="message"></textarea>
        </form>
        """

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto(f"data:text/html,{html}")

            page.locator("#name").fill("John Doe")
            page.locator("#email").fill("john@example.com")
            page.locator("#message").fill("Hello World")

            assert page.locator("#name").input_value() == "John Doe"
            assert page.locator("#email").input_value() == "john@example.com"
            assert page.locator("#message").input_value() == "Hello World"

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_wait_for_selector(self):
        """Test waiting for selectors."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        html = """
        <script>
            setTimeout(() => {
                const div = document.createElement('div');
                div.id = 'delayed';
                div.textContent = 'Loaded';
                document.body.appendChild(div);
            }, 100);
        </script>
        """

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto(f"data:text/html,{html}")

            element = page.wait_for_selector("#delayed", timeout=5000)
            assert element is not None
            assert element.text_content() == "Loaded"

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_multiple_pages(self):
        """Test creating multiple pages."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page1 = browser.new_page()
            page2 = browser.new_page()

            page1.goto("data:text/html,<h1>Page 1</h1>")
            page2.goto("data:text/html,<h1>Page 2</h1>")

            assert "Page 1" in page1.content()
            assert "Page 2" in page2.content()


class TestAuroraViewBridge:
    """Tests for AuroraView bridge injection (Native mode)."""

    # Skip in CI
    pytestmark = pytest.mark.skipif(
        IN_CI,
        reason="Native mode tests skipped in CI (requires `playwright install chromium`)"
    )

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_bridge_injected(self):
        """Test that AuroraView bridge is injected."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            has_bridge = page.evaluate("typeof window.auroraview !== 'undefined'")
            assert has_bridge is True

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_bridge_test_mode(self):
        """Test that bridge is in test mode."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            test_mode = page.evaluate("window.auroraview._testMode")
            assert test_mode is True

            platform = page.evaluate("window.auroraview._platform")
            assert platform == "playwright-test"

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_bridge_api_proxy(self):
        """Test that bridge API proxy works."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            # API proxy should exist
            has_api = page.evaluate("typeof window.auroraview.api !== 'undefined'")
            assert has_api is True

            # API calls should work (mock mode)
            result = page.evaluate("window.auroraview.api.test_method('arg1', 'arg2')")
            # In test mode, calls resolve to undefined
            assert result is None

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_bridge_events(self):
        """Test that bridge event system works."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            # Set up event handler and trigger event
            result = page.evaluate("""
                (function() {
                    let received = null;
                    window.auroraview.on('test_event', (data) => {
                        received = data;
                    });
                    window.auroraview.trigger('test_event', {message: 'hello'});
                    return received;
                })()
            """)

            assert result == {"message": "hello"}

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_bridge_disabled(self):
        """Test that bridge can be disabled."""
        from auroraview.testing.auroratest import PlaywrightBrowser

        with PlaywrightBrowser.launch(headless=True, inject_bridge=False) as browser:
            page = browser.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            has_bridge = page.evaluate("typeof window.auroraview !== 'undefined'")
            assert has_bridge is False
