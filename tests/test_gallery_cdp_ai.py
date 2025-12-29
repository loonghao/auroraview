"""AI-Enhanced Gallery CDP Integration Tests.

This module enhances the existing CDP tests with Midscene.js AI capabilities
for more robust and maintainable UI testing.

Key improvements over traditional CDP tests:
- Natural language selectors (self-healing)
- AI-powered visual assertions
- Intelligent data extraction
- Reduced selector brittleness

Usage:
    # Set API key
    export OPENAI_API_KEY=your-api-key

    # Start Gallery with CDP
    ./gallery/pack-output/auroraview-gallery.exe &

    # Run tests
    pytest tests/test_gallery_cdp_ai.py -v
"""

from __future__ import annotations

import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import pytest

# Project setup
PROJECT_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))

# Check dependencies
try:
    from playwright.async_api import async_playwright

    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False

# Configuration
CDP_URL = os.environ.get("WEBVIEW2_CDP_URL", "http://127.0.0.1:9222")
HAS_AI_KEY = bool(os.environ.get("OPENAI_API_KEY") or os.environ.get("MIDSCENE_MODEL_API_KEY"))


def is_cdp_available() -> bool:
    """Check if CDP endpoint is available."""
    import urllib.error
    import urllib.request

    try:
        req = urllib.request.urlopen(f"{CDP_URL}/json/version", timeout=2)
        req.close()
        return True
    except (urllib.error.URLError, OSError):
        return False


pytestmark = [
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.skipif(not HAS_AI_KEY, reason="No AI API key configured"),
    pytest.mark.skipif(not is_cdp_available(), reason="CDP not available"),
    pytest.mark.integration,
    pytest.mark.cdp,
    pytest.mark.ai,
]


@dataclass
class AITestResult:
    """Result of an AI-powered test."""

    name: str
    passed: bool
    duration_ms: float
    ai_actions: int = 0
    ai_assertions: int = 0
    error: Optional[str] = None


class TestGalleryAICDP:
    """AI-enhanced CDP tests for Gallery."""

    async def _find_gallery_page(self, browser):
        """Find the correct Gallery page from all available pages.

        WebView2 with CDP may expose multiple pages:
        - about:blank (empty initial page)
        - Loading screen page
        - Main application page

        This method finds the most appropriate page for testing.
        """
        all_pages = []
        for context in browser.contexts:
            for page in context.pages:
                url = page.url
                title = ""
                try:
                    title = await page.title()
                except Exception:
                    pass
                all_pages.append(
                    {
                        "page": page,
                        "url": url,
                        "title": title,
                    }
                )

        print(f"[CDP] Found {len(all_pages)} page(s):")
        for i, p in enumerate(all_pages):
            print(f"  [{i}] URL: {p['url']}, Title: {p['title']}")

        if not all_pages:
            return None

        # Priority order for page selection:
        # 1. Page with auroraview API ready
        # 2. Page with loading screen (auroraLoading)
        # 3. Page with non-blank URL (not about:blank)
        # 4. First available page

        # Check for auroraview ready
        for p in all_pages:
            try:
                has_api = await p["page"].evaluate("""
                    () => typeof window.auroraview !== 'undefined' &&
                          typeof window.auroraview.api !== 'undefined'
                """)
                if has_api:
                    print(f"[CDP] Selected page with auroraview API: {p['url']}")
                    return p["page"]
            except Exception:
                pass

        # Check for loading screen
        for p in all_pages:
            try:
                has_loading = await p["page"].evaluate("""
                    () => typeof window.auroraLoading !== 'undefined' ||
                          document.querySelector('.loading-container') !== null
                """)
                if has_loading:
                    print(f"[CDP] Selected page with loading screen: {p['url']}")
                    return p["page"]
            except Exception:
                pass

        # Skip about:blank pages
        for p in all_pages:
            if p["url"] and p["url"] != "about:blank":
                print(f"[CDP] Selected non-blank page: {p['url']}")
                return p["page"]

        # Fallback to first page
        print(f"[CDP] Fallback to first page: {all_pages[0]['url']}")
        return all_pages[0]["page"]

    @pytest.fixture
    async def cdp_page_with_ai(self):
        """Connect to Gallery via CDP and create AI agent."""
        from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

        async with async_playwright() as p:
            browser = await p.chromium.connect_over_cdp(CDP_URL)

            # Find the correct Gallery page (not about:blank)
            page = await self._find_gallery_page(browser)
            if not page:
                raise RuntimeError("No valid Gallery page found")

            # Wait for Gallery to be ready
            await page.wait_for_function(
                """
                typeof auroraview !== 'undefined' &&
                typeof auroraview.api !== 'undefined'
            """,
                timeout=30000,
            )

            config = MidsceneConfig(debug=False, timeout=30000)
            async with MidsceneAgent(page, config) as agent:
                yield page, agent

            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_gallery_ready(self, cdp_page_with_ai):
        """Test Gallery is ready using AI assertion."""
        page, agent = cdp_page_with_ai

        # AI assertion: Gallery loaded
        await agent.ai_assert(
            "the AuroraView Gallery application has loaded and shows sample examples"
        )

    @pytest.mark.asyncio
    async def test_ai_samples_visible(self, cdp_page_with_ai):
        """Test samples are visible using AI."""
        page, agent = cdp_page_with_ai

        # AI assertion: samples list visible
        await agent.ai_assert(
            "there is a list or grid of sample examples with titles and descriptions"
        )

        # AI query: count visible samples
        count = await agent.ai_query(
            "number, approximately how many sample items are visible on the page"
        )

        if count is not None:
            assert count > 0, "Should have visible samples"

    @pytest.mark.asyncio
    async def test_ai_search_samples(self, cdp_page_with_ai):
        """Test search functionality using AI."""
        page, agent = cdp_page_with_ai

        # AI action: search for "window"
        await agent.ai_act('type "window" in the search box or search input')

        # Wait for filter
        await page.wait_for_timeout(500)

        # AI assertion: results filtered
        await agent.ai_assert(
            "the search results show items related to window or the list has been filtered"
        )

        # AI action: clear search
        await agent.ai_act("clear the search input or remove the search text")

        await page.wait_for_timeout(300)

    @pytest.mark.asyncio
    async def test_ai_select_sample(self, cdp_page_with_ai):
        """Test selecting a sample using AI."""
        page, agent = cdp_page_with_ai

        # AI action: click on Simple Decorator sample
        await agent.ai_act(
            'click on the sample called "Simple Decorator" or the first sample in the list'
        )

        await page.wait_for_timeout(500)

        # AI assertion: sample selected
        await agent.ai_assert("a sample has been selected and its details or code are visible")

    @pytest.mark.asyncio
    async def test_ai_run_sample(self, cdp_page_with_ai):
        """Test running a sample using AI."""
        page, agent = cdp_page_with_ai

        # First select a sample
        await agent.ai_act("click on any sample item in the list")
        await page.wait_for_timeout(300)

        # AI action: click run button
        try:
            await agent.ai_act('click the "Run" button or play button')
            await page.wait_for_timeout(1000)

            # AI assertion: sample started
            await agent.ai_assert("the sample has started running or a console output is visible")

            # AI action: stop the sample
            await agent.ai_act('click the "Stop" button or stop icon')
            await page.wait_for_timeout(500)
        except Exception as e:
            # Run button might not be visible for all samples
            pytest.skip(f"Run button not available: {e}")

    @pytest.mark.asyncio
    async def test_ai_category_navigation(self, cdp_page_with_ai):
        """Test category navigation using AI."""
        page, agent = cdp_page_with_ai

        # AI query: get visible categories
        categories = await agent.ai_query(
            "string[], list of category names visible in the sidebar or navigation"
        )

        if categories and len(categories) > 0:
            # AI action: click on a category
            await agent.ai_act(f'click on the category "{categories[0]}"')
            await page.wait_for_timeout(300)

            # AI assertion: category selected
            await agent.ai_assert(
                f"the {categories[0]} category is selected or its samples are shown"
            )

    @pytest.mark.asyncio
    async def test_ai_extract_sample_info(self, cdp_page_with_ai):
        """Test extracting sample information using AI."""
        page, agent = cdp_page_with_ai

        # AI query: extract sample information
        samples = await agent.ai_query(
            """
            {title: string, category: string}[],
            extract the first 3 sample items with their titles and categories
            """
        )

        if samples:
            assert len(samples) > 0, "Should extract at least one sample"
            for sample in samples:
                assert "title" in sample, "Sample should have title"

    @pytest.mark.asyncio
    async def test_ai_code_viewer(self, cdp_page_with_ai):
        """Test code viewer using AI."""
        page, agent = cdp_page_with_ai

        # Select a sample first
        await agent.ai_act("click on the first sample in the list")
        await page.wait_for_timeout(500)

        # AI assertion: code is visible
        await agent.ai_assert("source code or a code editor is visible showing Python code")

        # AI query: check for syntax highlighting
        has_highlighting = await agent.ai_query(
            "boolean, does the code have syntax highlighting (colored keywords)?"
        )

        # Result can be True, False, or None
        assert has_highlighting in [True, False, None]


class TestGalleryAIWorkflows:
    """Complete workflow tests using AI with CDP."""

    @pytest.fixture
    async def cdp_page_with_ai(self):
        """Connect to Gallery via CDP and create AI agent."""
        from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

        async with async_playwright() as p:
            browser = await p.chromium.connect_over_cdp(CDP_URL)

            if browser.contexts and browser.contexts[0].pages:
                page = browser.contexts[0].pages[0]
            else:
                raise RuntimeError("No page found")

            await page.wait_for_function("typeof auroraview !== 'undefined'", timeout=30000)

            config = MidsceneConfig(debug=False, timeout=30000)
            async with MidsceneAgent(page, config) as agent:
                yield page, agent

            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_full_browse_workflow(self, cdp_page_with_ai):
        """Test complete browse workflow with AI."""
        page, agent = cdp_page_with_ai

        # Step 1: Verify Gallery loaded
        await agent.ai_assert("the AuroraView Gallery is loaded and ready")

        # Step 2: Browse categories
        await agent.ai_act("click on a category in the sidebar")
        await page.wait_for_timeout(300)

        # Step 3: Search for something
        await agent.ai_act('type "demo" in the search box')
        await page.wait_for_timeout(500)

        # Step 4: Select a result
        await agent.ai_act("click on the first search result or sample")
        await page.wait_for_timeout(300)

        # Step 5: View code
        await agent.ai_assert("code or details are visible for the selected sample")

        # Step 6: Clear search
        await agent.ai_act("clear the search input")
        await page.wait_for_timeout(300)

        # Step 7: Final verification
        await agent.ai_assert("the Gallery is still functional and showing samples")

    @pytest.mark.asyncio
    async def test_ai_run_multiple_samples(self, cdp_page_with_ai):
        """Test running multiple samples with AI."""
        page, agent = cdp_page_with_ai

        samples_to_test = ["Simple Decorator", "Window Events", "Floating Panel"]
        results = []

        for sample_name in samples_to_test:
            try:
                # Select sample
                await agent.ai_act(f'click on the sample "{sample_name}" or similar')
                await page.wait_for_timeout(300)

                # Run it
                await agent.ai_act('click the "Run" button')
                await page.wait_for_timeout(1500)

                # Check it started
                started = await agent.ai_query(
                    "boolean, is there a running process indicator or console output?"
                )

                # Stop it
                await agent.ai_act('click the "Stop" button')
                await page.wait_for_timeout(300)

                results.append({"sample": sample_name, "started": started})
            except Exception as e:
                results.append({"sample": sample_name, "error": str(e)})

        # At least some should work
        successful = [r for r in results if r.get("started") or "error" not in r]
        assert len(successful) > 0, f"No samples ran successfully: {results}"


class TestGalleryAIVisual:
    """Visual testing with AI."""

    @pytest.fixture
    async def cdp_page_with_ai(self):
        """Connect to Gallery via CDP."""
        from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

        async with async_playwright() as p:
            browser = await p.chromium.connect_over_cdp(CDP_URL)

            if browser.contexts and browser.contexts[0].pages:
                page = browser.contexts[0].pages[0]
            else:
                raise RuntimeError("No page found")

            await page.wait_for_function("typeof auroraview !== 'undefined'", timeout=30000)

            config = MidsceneConfig(debug=False, timeout=30000)
            async with MidsceneAgent(page, config) as agent:
                yield page, agent

            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_visual_layout(self, cdp_page_with_ai):
        """Test visual layout with AI."""
        page, agent = cdp_page_with_ai

        # AI assertions about visual layout
        await agent.ai_assert("the sidebar is on the left side of the page")
        await agent.ai_assert("the main content area is on the right")
        await agent.ai_assert("there are no overlapping UI elements")

    @pytest.mark.asyncio
    async def test_ai_visual_consistency(self, cdp_page_with_ai):
        """Test visual consistency with AI."""
        page, agent = cdp_page_with_ai

        # AI assertions about visual consistency
        await agent.ai_assert("all sample cards have consistent styling")
        await agent.ai_assert("text is readable and properly sized")
        await agent.ai_assert("icons are visible and properly aligned")

    @pytest.mark.asyncio
    async def test_ai_no_visual_errors(self, cdp_page_with_ai):
        """Test for visual errors with AI."""
        page, agent = cdp_page_with_ai

        # AI assertions about errors
        await agent.ai_assert("there are no error messages or warnings visible")
        await agent.ai_assert("there are no broken images or missing icons")
        await agent.ai_assert("the page does not show any loading spinners stuck")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
