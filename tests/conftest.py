"""
Pytest configuration for AuroraView tests.

This module provides:
- Event loop configuration for Windows
- AI-powered testing fixtures (Midscene.js integration)
- Common test utilities
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    pass

# Fix for Playwright on Windows with pytest-asyncio
# Playwright's sync API needs ProactorEventLoop for subprocess support
if sys.platform == "win32":
    import asyncio

    # Set the default event loop policy to ProactorEventLoop
    # This is required for Playwright's subprocess spawning
    asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())

# Add project paths
PROJECT_ROOT = Path(__file__).parent.parent


def _add_sys_path(path: Path) -> None:
    path_str = str(path)
    if path_str not in sys.path:
        sys.path.insert(0, path_str)


_add_sys_path(PROJECT_ROOT / "python")
_add_sys_path(PROJECT_ROOT)
_add_sys_path(PROJECT_ROOT / "gallery")


@pytest.fixture(scope="session", autouse=True)
def setup_event_loop_policy():
    """Ensure correct event loop policy for Playwright."""
    if sys.platform == "win32":
        import asyncio

        asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    yield


# ─────────────────────────────────────────────────────────────────────────────
# AI Testing Fixtures (Midscene.js Integration)
# ─────────────────────────────────────────────────────────────────────────────


def has_ai_key() -> bool:
    """Check if AI API key is configured."""
    return bool(os.environ.get("OPENAI_API_KEY") or os.environ.get("MIDSCENE_MODEL_API_KEY"))


@pytest.fixture
def ai_config():
    """Provide Midscene configuration for AI tests.

    Usage:
        def test_with_ai(ai_config):
            from auroraview.testing.midscene import MidsceneAgent
            async with MidsceneAgent(page, ai_config) as agent:
                await agent.ai_act('click the button')
    """
    if not has_ai_key():
        pytest.skip("No AI API key configured")

    try:
        from auroraview.testing.midscene import MidsceneConfig

        return MidsceneConfig(
            debug=os.environ.get("MIDSCENE_DEBUG", "").lower() == "true",
            timeout=int(os.environ.get("MIDSCENE_TIMEOUT", "30000")),
            cacheable=True,
        )
    except ImportError:
        pytest.skip("Midscene module not available")


@pytest.fixture
def ai_agent_factory(ai_config):
    """Factory fixture for creating AI agents.

    Usage:
        async def test_with_ai(ai_agent_factory, page):
            async with ai_agent_factory(page) as agent:
                await agent.ai_act('click the button')
                await agent.ai_assert('the button was clicked')
    """
    from auroraview.testing.midscene import MidsceneAgent

    def create_agent(page):
        return MidsceneAgent(page, ai_config)

    return create_agent


@pytest.fixture
async def ai_browser():
    """Provide a Playwright browser for AI tests.

    Usage:
        async def test_with_browser(ai_browser, ai_agent_factory):
            page = await ai_browser.new_page()
            await page.goto('https://example.com')
            async with ai_agent_factory(page) as agent:
                await agent.ai_act('click login')
    """
    try:
        from playwright.async_api import async_playwright
    except ImportError:
        pytest.skip("Playwright not installed")

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        yield browser
        await browser.close()


@pytest.fixture
async def ai_page(ai_browser):
    """Provide a Playwright page for AI tests.

    Usage:
        async def test_with_page(ai_page, ai_agent_factory):
            await ai_page.goto('https://example.com')
            async with ai_agent_factory(ai_page) as agent:
                await agent.ai_act('click login')
    """
    page = await ai_browser.new_page(viewport={"width": 1200, "height": 800})
    yield page
    await page.close()


@pytest.fixture
async def ai_test_context(ai_page, ai_agent_factory):
    """Complete AI test context with page and agent.

    Usage:
        async def test_complete(ai_test_context):
            page, create_agent = ai_test_context
            await page.goto('https://example.com')
            async with create_agent(page) as agent:
                await agent.ai_act('click login')
    """
    return ai_page, ai_agent_factory


# ─────────────────────────────────────────────────────────────────────────────
# Test Markers
# ─────────────────────────────────────────────────────────────────────────────


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line("markers", "ai: AI-powered tests using Midscene.js")
    config.addinivalue_line("markers", "cdp: Tests using Chrome DevTools Protocol")
    config.addinivalue_line("markers", "e2e: End-to-end tests")
    config.addinivalue_line("markers", "slow: Slow tests")
    config.addinivalue_line("markers", "integration: Integration tests")


# ─────────────────────────────────────────────────────────────────────────────
# Mock AuroraView Bridge
# ─────────────────────────────────────────────────────────────────────────────


MOCK_BRIDGE_SCRIPT = """
window._apiCalls = [];
window._mockResponses = {};

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
"""


@pytest.fixture
def mock_bridge_script():
    """Provide the mock AuroraView bridge script.

    Usage:
        async def test_with_mock(ai_page, mock_bridge_script):
            await ai_page.add_init_script(mock_bridge_script)
            await ai_page.goto('file:///path/to/app.html')
    """
    return MOCK_BRIDGE_SCRIPT


async def inject_mock_bridge(page):
    """Inject mock AuroraView bridge into a page.

    Usage:
        await inject_mock_bridge(page)
    """
    await page.evaluate(MOCK_BRIDGE_SCRIPT)


@pytest.fixture
def inject_bridge():
    """Provide the inject_mock_bridge function.

    Usage:
        async def test_with_bridge(ai_page, inject_bridge):
            await ai_page.goto('file:///path/to/app.html')
            await inject_bridge(ai_page)
    """
    return inject_mock_bridge
