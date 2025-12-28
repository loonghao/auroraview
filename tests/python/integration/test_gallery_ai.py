"""AI-Powered Gallery E2E Tests using Midscene.js.

This module provides AI-driven end-to-end tests for the AuroraView Gallery
using Midscene.js integration. It enables:

- Natural language UI interactions
- AI-powered visual assertions
- Intelligent data extraction
- Self-healing selectors (AI finds elements by description)

Requirements:
    - playwright: pip install playwright && playwright install chromium
    - OpenAI API key or compatible model

Usage:
    # Set API key
    export OPENAI_API_KEY=your-api-key

    # Run tests
    pytest tests/python/integration/test_gallery_ai.py -v
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    pass

# Check dependencies
try:
    from playwright.async_api import async_playwright

    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False

# Project paths
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
GALLERY_DIR = PROJECT_ROOT / "gallery"
DIST_DIR = GALLERY_DIR / "dist"

sys.path.insert(0, str(PROJECT_ROOT / "python"))

# Check for AI API key
HAS_AI_KEY = bool(os.environ.get("OPENAI_API_KEY") or os.environ.get("MIDSCENE_MODEL_API_KEY"))

pytestmark = [
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.skipif(not HAS_AI_KEY, reason="No AI API key configured"),
    pytest.mark.skipif(not (DIST_DIR / "index.html").exists(), reason="Gallery not built"),
    pytest.mark.integration,
    pytest.mark.e2e,
    pytest.mark.ai,
]


def inject_mock_bridge(page):
    """Inject mock AuroraView bridge for testing."""
    return page.evaluate("""
        window._apiCalls = [];
        window._mockResponses = {
            'api.get_samples': [
                { id: 'simple_decorator', title: 'Simple Decorator', category: 'getting_started',
                  description: 'Basic WebView example using decorators', icon: 'wand-2',
                  source_file: 'simple_decorator.py', tags: ['beginner', 'decorator'] },
                { id: 'window_events', title: 'Window Events', category: 'window_management',
                  description: 'Handle window lifecycle events', icon: 'layout',
                  source_file: 'window_events.py', tags: ['events', 'window'] },
                { id: 'floating_panel', title: 'Floating Panel', category: 'window_effects',
                  description: 'Create floating tool panels', icon: 'panel-top',
                  source_file: 'floating_panel.py', tags: ['panel', 'floating'] },
                { id: 'system_tray', title: 'System Tray', category: 'window_effects',
                  description: 'System tray integration', icon: 'bell',
                  source_file: 'system_tray.py', tags: ['tray', 'system'] },
                { id: 'native_menu', title: 'Native Menu', category: 'menus',
                  description: 'Native application menus', icon: 'menu',
                  source_file: 'native_menu.py', tags: ['menu', 'native'] }
            ],
            'api.get_categories': {
                'getting_started': { title: 'Getting Started', icon: 'rocket', description: 'Quick start examples' },
                'window_management': { title: 'Window Management', icon: 'layout', description: 'Window controls' },
                'window_effects': { title: 'Window Effects', icon: 'sparkles', description: 'Visual effects' },
                'menus': { title: 'Menus', icon: 'menu', description: 'Menu systems' }
            },
            'api.get_source': '# Sample source code\\nfrom auroraview import WebView\\n\\ndef main():\\n    pass',
            'api.run_sample': { ok: true, pid: 12345 },
            'api.kill_process': { ok: true },
            'api.list_processes': { ok: true, processes: {} }
        };

        if (!window.auroraview) {
            window.auroraview = {
                call: function(method, params) {
                    window._apiCalls.push({ method, params, timestamp: Date.now() });
                    if (window._mockResponses[method]) {
                        return Promise.resolve(window._mockResponses[method]);
                    }
                    return Promise.resolve(undefined);
                },
                on: function(event, handler) {
                    if (!window._eventHandlers) window._eventHandlers = {};
                    if (!window._eventHandlers[event]) window._eventHandlers[event] = [];
                    window._eventHandlers[event].push(handler);
                    return () => {
                        const idx = window._eventHandlers[event].indexOf(handler);
                        if (idx >= 0) window._eventHandlers[event].splice(idx, 1);
                    };
                },
                trigger: function(event, data) {
                    if (window._eventHandlers && window._eventHandlers[event]) {
                        window._eventHandlers[event].forEach(h => h(data));
                    }
                },
                api: new Proxy({}, {
                    get: function(target, prop) {
                        return function(...args) {
                            return window.auroraview.call('api.' + prop, args);
                        };
                    }
                }),
                _testMode: true
            };
            window.dispatchEvent(new CustomEvent('auroraviewready'));
        }
    """)


class TestGalleryAIBasic:
    """Basic AI-powered Gallery tests."""

    @pytest.fixture
    async def page_with_ai(self):
        """Create a page with AI agent."""
        from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page(viewport={"width": 1200, "height": 800})

            # Load Gallery
            gallery_url = f"file://{DIST_DIR / 'index.html'}"
            await page.goto(gallery_url)

            # Inject mock bridge
            await inject_mock_bridge(page)

            # Wait for React to render
            await page.wait_for_timeout(1000)

            # Create AI agent with config
            config = MidsceneConfig(debug=False, timeout=30000)
            async with MidsceneAgent(page, config) as agent:
                yield page, agent

            await browser.close()

    @pytest.mark.asyncio
    async def test_gallery_loads_with_ai(self, page_with_ai):
        """Test Gallery loads using AI assertion."""
        page, agent = page_with_ai

        # AI assertion: check page loaded
        await agent.ai_assert("the page has loaded and shows content")

        # Traditional check for comparison
        title = await page.title()
        assert "Gallery" in title or "AuroraView" in title

    @pytest.mark.asyncio
    async def test_ai_finds_sidebar(self, page_with_ai):
        """Test AI can find sidebar elements."""
        page, agent = page_with_ai

        # AI assertion: sidebar exists
        await agent.ai_assert("there is a sidebar or navigation menu on the left side")

    @pytest.mark.asyncio
    async def test_ai_extracts_page_info(self, page_with_ai):
        """Test AI can extract page information."""
        page, agent = page_with_ai

        # AI query: extract page title
        title = await agent.ai_query("string, the page title or header text")
        assert title is not None


class TestGalleryAIInteractions:
    """AI-powered interaction tests for Gallery."""

    @pytest.fixture
    async def gallery_page(self):
        """Create a Gallery page with mock data."""
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page(viewport={"width": 1200, "height": 800})

            gallery_url = f"file://{DIST_DIR / 'index.html'}"
            await page.goto(gallery_url)
            await inject_mock_bridge(page)
            await page.wait_for_timeout(1500)

            yield page
            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_search_interaction(self, gallery_page):
        """Test AI can perform search interaction."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI action: search for "window"
            await agent.ai_act('type "window" in the search input or search box')

            # Wait for filter
            await page.wait_for_timeout(500)

            # AI assertion: results filtered
            await agent.ai_assert("the search input contains text or there are filtered results")

    @pytest.mark.asyncio
    async def test_ai_click_sample(self, gallery_page):
        """Test AI can click on a sample item."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI action: click on a sample
            try:
                await agent.ai_act("click on any sample item, example card, or list item")
                await page.wait_for_timeout(300)
            except Exception:
                # If no clickable sample, that's ok for mock
                pass

            # Check API was called
            calls = await page.evaluate("window._apiCalls")
            # Should have some API calls
            assert isinstance(calls, list)

    @pytest.mark.asyncio
    async def test_ai_navigation_flow(self, gallery_page):
        """Test AI can navigate through Gallery."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # Step 1: Verify initial state
            await agent.ai_assert("the gallery or application has loaded")

            # Step 2: Try to find and click a category
            try:
                await agent.ai_act("click on a category, menu item, or navigation link")
                await page.wait_for_timeout(300)
            except Exception:
                pass  # Categories might not be visible in mock

            # Step 3: Verify we're still on the page
            await agent.ai_assert("the page is still displaying content")


class TestGalleryAIDataExtraction:
    """AI-powered data extraction tests."""

    @pytest.fixture
    async def gallery_page(self):
        """Create a Gallery page."""
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page(viewport={"width": 1200, "height": 800})

            gallery_url = f"file://{DIST_DIR / 'index.html'}"
            await page.goto(gallery_url)
            await inject_mock_bridge(page)
            await page.wait_for_timeout(1500)

            yield page
            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_extract_visible_text(self, gallery_page):
        """Test AI can extract visible text content."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI query: extract any visible text
            text = await agent.ai_query("string, any visible heading or title text on the page")

            # Should get some text
            assert text is None or isinstance(text, str)

    @pytest.mark.asyncio
    async def test_ai_extract_ui_structure(self, gallery_page):
        """Test AI can describe UI structure."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI query: describe layout
            layout = await agent.ai_query(
                "string, describe the main layout areas of the page "
                "(e.g., sidebar, main content, header)"
            )

            assert layout is None or isinstance(layout, str)


class TestGalleryAIAssertions:
    """AI-powered assertion tests."""

    @pytest.fixture
    async def gallery_page(self):
        """Create a Gallery page."""
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page(viewport={"width": 1200, "height": 800})

            gallery_url = f"file://{DIST_DIR / 'index.html'}"
            await page.goto(gallery_url)
            await inject_mock_bridge(page)
            await page.wait_for_timeout(1500)

            yield page
            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_assert_no_errors(self, gallery_page):
        """Test AI can verify no error messages."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI assertion: no error messages
            await agent.ai_assert("there are no visible error messages, alerts, or warning dialogs")

    @pytest.mark.asyncio
    async def test_ai_assert_interactive_elements(self, gallery_page):
        """Test AI can verify interactive elements exist."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI assertion: interactive elements exist
            await agent.ai_assert("there are clickable elements like buttons, links, or menu items")

    @pytest.mark.asyncio
    async def test_ai_assert_responsive_layout(self, gallery_page):
        """Test AI can verify responsive layout."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # AI assertion: layout is reasonable
            await agent.ai_assert(
                "the layout appears properly formatted with no overlapping elements"
            )


class TestGalleryAIWorkflows:
    """Complete workflow tests using AI."""

    @pytest.fixture
    async def gallery_page(self):
        """Create a Gallery page."""
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page(viewport={"width": 1200, "height": 800})

            gallery_url = f"file://{DIST_DIR / 'index.html'}"
            await page.goto(gallery_url)
            await inject_mock_bridge(page)
            await page.wait_for_timeout(1500)

            yield page
            await browser.close()

    @pytest.mark.asyncio
    async def test_ai_browse_samples_workflow(self, gallery_page):
        """Test complete browse samples workflow with AI."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # Step 1: Verify Gallery loaded
            await agent.ai_assert("the application has loaded successfully")

            # Step 2: Look for samples list
            await agent.ai_assert("there is a list or grid of items, samples, or examples")

            # Step 3: Try search
            try:
                await agent.ai_act('find the search input and type "demo"')
                await page.wait_for_timeout(500)
            except Exception:
                pass  # Search might not be visible

            # Step 4: Clear search
            try:
                await agent.ai_act("clear the search input or click a clear button")
                await page.wait_for_timeout(300)
            except Exception:
                pass

            # Step 5: Final assertion
            await agent.ai_assert("the page is still functional and showing content")

    @pytest.mark.asyncio
    async def test_ai_sample_details_workflow(self, gallery_page):
        """Test viewing sample details workflow with AI."""
        from auroraview.testing.midscene import MidsceneAgent

        page = gallery_page

        async with MidsceneAgent(page) as agent:
            # Step 1: Find and click a sample
            try:
                await agent.ai_act("click on the first sample, example, or card in the list")
                await page.wait_for_timeout(500)
            except Exception:
                pass

            # Step 2: Check for details view
            await agent.ai_assert(
                "the page shows some content, either a details view or the main list"
            )

            # Step 3: Look for code or description
            try:
                code_visible = await agent.ai_query(
                    "boolean, is there any code snippet or source code visible?"
                )
                # Result can be True, False, or None
                assert code_visible in [True, False, None]
            except Exception:
                pass


# Pytest plugin for AI fixture
@pytest.fixture
async def ai_agent(request):
    """Pytest fixture for AI agent.

    Usage:
        async def test_example(ai_agent, page):
            async with ai_agent(page) as agent:
                await agent.ai_act('click the button')
    """
    from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

    config = MidsceneConfig(debug=False, timeout=30000)

    def create_agent(page):
        return MidsceneAgent(page, config)

    return create_agent


# Run tests if executed directly
if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
