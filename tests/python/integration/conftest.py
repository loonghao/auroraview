"""
Pytest configuration for integration tests.

This module provides fixtures specific to integration testing,
including AI-powered testing with Midscene.js.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

# Project paths
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
GALLERY_DIR = PROJECT_ROOT / "gallery"
DIST_DIR = GALLERY_DIR / "dist"

sys.path.insert(0, str(PROJECT_ROOT / "python"))


# ─────────────────────────────────────────────────────────────────────────────
# Gallery Fixtures
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture
def gallery_dist_path():
    """Provide path to Gallery dist directory."""
    if not DIST_DIR.exists():
        pytest.skip("Gallery not built - run 'just gallery-build'")
    return DIST_DIR


@pytest.fixture
def gallery_url(gallery_dist_path):
    """Provide file URL to Gallery index.html."""
    index_path = gallery_dist_path / "index.html"
    if not index_path.exists():
        pytest.skip("Gallery index.html not found")
    return f"file://{index_path}"


# ─────────────────────────────────────────────────────────────────────────────
# AI Testing Fixtures
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture
async def gallery_with_ai(gallery_url, ai_agent_factory):
    """Provide Gallery page with AI agent ready.

    Usage:
        async def test_gallery(gallery_with_ai):
            page, agent = gallery_with_ai
            await agent.ai_act('click on a sample')
    """
    try:
        from playwright.async_api import async_playwright
    except ImportError:
        pytest.skip("Playwright not installed")

    from tests.conftest import inject_mock_bridge

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1200, "height": 800})

        await page.goto(gallery_url)
        await inject_mock_bridge(page)
        await page.wait_for_timeout(1000)

        async with ai_agent_factory(page) as agent:
            yield page, agent

        await browser.close()


@pytest.fixture
async def gallery_page_mock(gallery_url):
    """Provide Gallery page with mock bridge (no AI).

    Usage:
        async def test_gallery(gallery_page_mock):
            page = gallery_page_mock
            await page.click('button')
    """
    try:
        from playwright.async_api import async_playwright
    except ImportError:
        pytest.skip("Playwright not installed")

    from tests.conftest import inject_mock_bridge

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1200, "height": 800})

        await page.goto(gallery_url)
        await inject_mock_bridge(page)
        await page.wait_for_timeout(1000)

        yield page

        await browser.close()


# ─────────────────────────────────────────────────────────────────────────────
# CDP Fixtures
# ─────────────────────────────────────────────────────────────────────────────


CDP_URL = os.environ.get("WEBVIEW2_CDP_URL", "http://127.0.0.1:9222")


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


@pytest.fixture
async def cdp_page():
    """Connect to running Gallery via CDP.

    Usage:
        async def test_cdp(cdp_page):
            await cdp_page.click('button')
    """
    if not is_cdp_available():
        pytest.skip("CDP not available - start Gallery first")

    try:
        from playwright.async_api import async_playwright
    except ImportError:
        pytest.skip("Playwright not installed")

    async with async_playwright() as p:
        browser = await p.chromium.connect_over_cdp(CDP_URL)

        if browser.contexts and browser.contexts[0].pages:
            page = browser.contexts[0].pages[0]
        else:
            pytest.skip("No page found in Gallery")

        # Wait for AuroraView bridge
        await page.wait_for_function("typeof auroraview !== 'undefined'", timeout=30000)

        yield page

        await browser.close()


@pytest.fixture
async def cdp_with_ai(cdp_page, ai_agent_factory):
    """Connect to Gallery via CDP with AI agent.

    Usage:
        async def test_cdp_ai(cdp_with_ai):
            page, agent = cdp_with_ai
            await agent.ai_act('click on a sample')
    """
    async with ai_agent_factory(cdp_page) as agent:
        yield cdp_page, agent


# ─────────────────────────────────────────────────────────────────────────────
# Sample Mock Data
# ─────────────────────────────────────────────────────────────────────────────


MOCK_SAMPLES = [
    {
        "id": "simple_decorator",
        "title": "Simple Decorator",
        "category": "getting_started",
        "description": "Basic WebView example using decorators",
        "icon": "wand-2",
        "source_file": "simple_decorator.py",
        "tags": ["beginner", "decorator"],
    },
    {
        "id": "window_events",
        "title": "Window Events",
        "category": "window_management",
        "description": "Handle window lifecycle events",
        "icon": "layout",
        "source_file": "window_events.py",
        "tags": ["events", "window"],
    },
    {
        "id": "floating_panel",
        "title": "Floating Panel",
        "category": "window_effects",
        "description": "Create floating tool panels",
        "icon": "panel-top",
        "source_file": "floating_panel.py",
        "tags": ["panel", "floating"],
    },
]

MOCK_CATEGORIES = {
    "getting_started": {
        "title": "Getting Started",
        "icon": "rocket",
        "description": "Quick start examples",
    },
    "window_management": {
        "title": "Window Management",
        "icon": "layout",
        "description": "Window controls",
    },
    "window_effects": {
        "title": "Window Effects",
        "icon": "sparkles",
        "description": "Visual effects",
    },
}


@pytest.fixture
def mock_samples():
    """Provide mock sample data."""
    return MOCK_SAMPLES


@pytest.fixture
def mock_categories():
    """Provide mock category data."""
    return MOCK_CATEGORIES
