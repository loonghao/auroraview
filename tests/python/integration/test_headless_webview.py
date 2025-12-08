"""Tests for headless WebView testing framework.

These tests verify the headless testing infrastructure works correctly.
"""

import os
import platform

import pytest

# Skip all tests if playwright is not installed
pytest.importorskip("playwright")


def _playwright_browser_available() -> bool:
    """Check if Playwright browser is available."""
    try:
        from playwright.sync_api import sync_playwright

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            browser.close()
        return True
    except Exception:
        return False


# Skip tests if Playwright browser not installed
pytestmark = pytest.mark.skipif(
    not _playwright_browser_available(),
    reason="Playwright browser not installed. Run: playwright install chromium",
)


class TestPlaywrightHeadlessWebView:
    """Tests for Playwright-based headless WebView."""

    def test_launch_and_close(self):
        """Test basic launch and close."""
        from auroraview.testing.headless_webview import HeadlessWebView

        webview = HeadlessWebView.playwright()
        assert not webview._closed
        webview.close()
        assert webview._closed

    def test_context_manager(self):
        """Test context manager usage."""
        from auroraview.testing.headless_webview import HeadlessWebView

        with HeadlessWebView.playwright() as webview:
            assert not webview._closed
        # Should be closed after context exit
        assert webview._closed

    def test_goto_and_text(self):
        """Test navigation and text extraction."""
        from auroraview.testing.headless_webview import HeadlessWebView

        with HeadlessWebView.playwright() as webview:
            webview.goto("https://example.com")
            text = webview.text("h1")
            assert "Example" in text

    def test_load_html(self):
        """Test loading HTML content."""
        from auroraview.testing.headless_webview import HeadlessWebView

        html = "<html><body><h1 id='title'>Test Title</h1></body></html>"

        with HeadlessWebView.playwright() as webview:
            webview.load_html(html)
            text = webview.text("#title")
            assert text == "Test Title"

    def test_click(self):
        """Test clicking elements."""
        from auroraview.testing.headless_webview import HeadlessWebView

        html = """
        <html><body>
            <button id="btn" onclick="document.getElementById('result').textContent='clicked'">
                Click Me
            </button>
            <div id="result"></div>
        </body></html>
        """

        with HeadlessWebView.playwright() as webview:
            webview.load_html(html)
            webview.click("#btn")
            text = webview.text("#result")
            assert text == "clicked"

    def test_fill(self):
        """Test filling input elements."""
        from auroraview.testing.headless_webview import HeadlessWebView

        html = """
        <html><body>
            <input id="email" type="email">
        </body></html>
        """

        with HeadlessWebView.playwright() as webview:
            webview.load_html(html)
            webview.fill("#email", "test@example.com")
            value = webview.evaluate("document.getElementById('email').value")
            assert value == "test@example.com"

    def test_evaluate(self):
        """Test JavaScript evaluation."""
        from auroraview.testing.headless_webview import HeadlessWebView

        with HeadlessWebView.playwright() as webview:
            webview.load_html("<html><body></body></html>")
            result = webview.evaluate("1 + 2")
            assert result == 3

    def test_wait_for_selector(self):
        """Test waiting for elements."""
        from auroraview.testing.headless_webview import HeadlessWebView

        html = """
        <html><body>
            <script>
                setTimeout(() => {
                    const div = document.createElement('div');
                    div.id = 'delayed';
                    div.textContent = 'Loaded';
                    document.body.appendChild(div);
                }, 100);
            </script>
        </body></html>
        """

        with HeadlessWebView.playwright(timeout=5.0) as webview:
            webview.load_html(html)
            webview.wait_for_selector("#delayed")
            text = webview.text("#delayed")
            assert text == "Loaded"

    def test_screenshot(self, tmp_path):
        """Test taking screenshots."""
        from auroraview.testing.headless_webview import HeadlessWebView

        screenshot_path = tmp_path / "test.png"

        with HeadlessWebView.playwright() as webview:
            webview.load_html("<html><body><h1>Screenshot Test</h1></body></html>")
            webview.screenshot(str(screenshot_path))

        assert screenshot_path.exists()
        assert screenshot_path.stat().st_size > 0

    def test_bridge_injection(self):
        """Test AuroraView bridge is injected."""
        from auroraview.testing.headless_webview import HeadlessWebView

        with HeadlessWebView.playwright(inject_bridge=True) as webview:
            webview.load_html("<html><body></body></html>")

            # Check bridge exists
            has_bridge = webview.evaluate("typeof window.auroraview !== 'undefined'")
            assert has_bridge

            # Check test mode flag
            test_mode = webview.evaluate("window.auroraview._testMode")
            assert test_mode is True

            # Check API proxy exists
            has_api = webview.evaluate("typeof window.auroraview.api === 'object'")
            assert has_api

    def test_bridge_not_injected(self):
        """Test bridge can be disabled."""
        from auroraview.testing.headless_webview import HeadlessWebView

        with HeadlessWebView.playwright(inject_bridge=False) as webview:
            webview.load_html("<html><body></body></html>")

            has_bridge = webview.evaluate("typeof window.auroraview !== 'undefined'")
            assert not has_bridge

    def test_page_access(self):
        """Test direct Playwright page access."""
        from auroraview.testing.headless_webview import HeadlessWebView

        with HeadlessWebView.playwright() as webview:
            webview.load_html("<html><body><h1>Test</h1></body></html>")

            # Access underlying Playwright page
            page = webview.page
            assert page is not None

            # Use Playwright API directly
            locator = page.locator("h1")
            assert locator.text_content() == "Test"


class TestHeadlessWebViewAuto:
    """Tests for auto-detection mode."""

    def test_auto_selects_playwright(self):
        """Test auto mode selects Playwright by default."""
        from auroraview.testing.headless_webview import (
            HeadlessWebView,
            PlaywrightHeadlessWebView,
        )

        # Clear environment variables that might affect detection
        os.environ.pop("WEBVIEW2_CDP_URL", None)
        os.environ.pop("AURORAVIEW_USE_XVFB", None)

        webview = HeadlessWebView.auto()
        try:
            assert isinstance(webview, PlaywrightHeadlessWebView)
        finally:
            webview.close()


class TestHeadlessWebViewContextManager:
    """Tests for context manager function."""

    def test_headless_webview_function(self):
        """Test headless_webview context manager function."""
        from auroraview.testing.headless_webview import headless_webview

        with headless_webview(mode="playwright") as webview:
            webview.load_html("<h1>Test</h1>")
            text = webview.text("h1")
            assert text == "Test"

    def test_headless_webview_auto_mode(self):
        """Test auto mode via function."""
        from auroraview.testing.headless_webview import headless_webview

        with headless_webview(mode="auto") as webview:
            webview.load_html("<h1>Auto Test</h1>")
            text = webview.text("h1")
            assert text == "Auto Test"


@pytest.mark.skipif(platform.system() != "Linux", reason="Xvfb only available on Linux")
class TestVirtualDisplayWebView:
    """Tests for Xvfb-based WebView (Linux only)."""

    @pytest.mark.skipif(not os.path.exists("/usr/bin/Xvfb"), reason="Xvfb not installed")
    def test_virtual_display_launch(self):
        """Test Xvfb WebView launch."""
        from auroraview.testing.headless_webview import HeadlessWebView

        # Note: This test requires actual WebView to work
        # It may fail if WebView dependencies are not installed
        try:
            webview = HeadlessWebView.virtual_display()
            webview.close()
        except Exception as e:
            pytest.skip(f"Virtual display test skipped: {e}")


# These tests don't require Playwright browser
@pytest.mark.skipif(False, reason="Always run")
class TestHeadlessOptions:
    """Tests for HeadlessOptions configuration."""

    def test_default_options(self):
        """Test default options values."""
        from auroraview.testing.headless_webview import HeadlessOptions

        options = HeadlessOptions()
        assert options.timeout == 30.0
        assert options.width == 1280
        assert options.height == 720
        assert options.inject_bridge is True
        assert options.screenshot_on_failure is True

    def test_custom_options(self):
        """Test custom options."""
        from auroraview.testing.headless_webview import HeadlessOptions

        options = HeadlessOptions(
            timeout=60.0,
            width=1920,
            height=1080,
            inject_bridge=False,
            slow_mo=100,
        )

        assert options.timeout == 60.0
        assert options.width == 1920
        assert options.height == 1080
        assert options.inject_bridge is False
        assert options.slow_mo == 100
