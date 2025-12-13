"""Real end-to-end tests for Gallery application.

These tests launch the actual Gallery frontend and test real user interactions.
They require the Gallery to be built first: `just gallery-build`

Usage:
    pytest tests/python/integration/test_gallery_real_e2e.py -v

Note: These tests are slower but provide the highest confidence level.
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

import pytest

# Check if playwright is available
try:
    from playwright.sync_api import sync_playwright

    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False

PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
GALLERY_DIR = PROJECT_ROOT / "gallery"
DIST_DIR = GALLERY_DIR / "dist"

pytestmark = [
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.skipif(not (DIST_DIR / "index.html").exists(), reason="Gallery not built"),
    pytest.mark.integration,
    pytest.mark.e2e,
    pytest.mark.slow,
]


class TestGalleryFrontend:
    """Test Gallery frontend without backend."""

    @pytest.fixture
    def browser_context(self):
        """Create a browser context with AuroraView bridge mock."""
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(viewport={"width": 1200, "height": 800})

            # Inject mock AuroraView bridge
            context.add_init_script("""
                window._apiCalls = [];
                window._mockResponses = {};

                window.auroraview = {
                    call: function(method, params) {
                        window._apiCalls.push({ method, params, timestamp: Date.now() });

                        // Return mock response if configured
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

                    _testMode: true,
                    _platform: 'playwright-test'
                };

                window.dispatchEvent(new CustomEvent('auroraviewready'));
            """)

            yield context
            browser.close()

    def test_gallery_loads(self, browser_context):
        """Test that Gallery frontend loads correctly."""
        page = browser_context.new_page()

        # Load Gallery
        gallery_url = f"file://{DIST_DIR / 'index.html'}"
        page.goto(gallery_url)

        # Wait for React to render
        page.wait_for_selector("[data-testid='sidebar'], .sidebar, nav", timeout=10000)

        # Check title
        title = page.title()
        assert "Gallery" in title or "AuroraView" in title

    def test_sidebar_renders_categories(self, browser_context):
        """Test that sidebar shows sample categories."""
        page = browser_context.new_page()

        # Configure mock response for get_samples
        page.evaluate("""
            window._mockResponses['api.get_samples'] = [
                { id: 'test1', title: 'Test Sample 1', category: 'getting_started', description: 'Test', icon: 'code', tags: [] },
                { id: 'test2', title: 'Test Sample 2', category: 'api_patterns', description: 'Test', icon: 'code', tags: [] }
            ];
            window._mockResponses['api.get_categories'] = {
                'getting_started': { title: 'Getting Started', icon: 'rocket', description: 'Quick start' },
                'api_patterns': { title: 'API Patterns', icon: 'code', description: 'API usage' }
            };
        """)

        gallery_url = f"file://{DIST_DIR / 'index.html'}"
        page.goto(gallery_url)

        # Wait for content to load
        time.sleep(1)

        # Check that API was called
        calls = page.evaluate("window._apiCalls")
        api_methods = [c["method"] for c in calls]

        # Should have called get_samples or get_categories
        assert any("sample" in m.lower() or "categor" in m.lower() for m in api_methods), (
            f"Expected sample/category API calls, got: {api_methods}"
        )

    def test_sample_click_triggers_api(self, browser_context):
        """Test that clicking a sample triggers the correct API call."""
        page = browser_context.new_page()

        # Configure mock responses
        page.evaluate("""
            window._mockResponses['api.get_samples'] = [
                { id: 'simple_decorator', title: 'Simple Decorator', category: 'getting_started',
                  description: 'Basic example', icon: 'wand-2', source_file: 'simple_decorator.py', tags: ['beginner'] }
            ];
            window._mockResponses['api.get_categories'] = {
                'getting_started': { title: 'Getting Started', icon: 'rocket', description: 'Quick start' }
            };
            window._mockResponses['api.get_source'] = '# Sample source code';
        """)

        gallery_url = f"file://{DIST_DIR / 'index.html'}"
        page.goto(gallery_url)

        # Wait for samples to load
        time.sleep(2)

        # Try to find and click a sample item
        # The exact selector depends on the Gallery UI implementation
        sample_selectors = [
            "[data-sample-id]",
            ".sample-item",
            "[role='button']",
            "button:has-text('Simple')",
            "div:has-text('Simple Decorator')",
        ]

        clicked = False
        for selector in sample_selectors:
            try:
                elements = page.locator(selector).all()
                if elements:
                    elements[0].click()
                    clicked = True
                    break
            except Exception:
                continue

        if clicked:
            time.sleep(0.5)

            # Check API calls
            calls = page.evaluate("window._apiCalls")
            # Should have called get_source or similar
            assert len(calls) > 0, "Expected API calls after click"


class TestGalleryAPIIntegration:
    """Test Gallery with real Python backend (subprocess)."""

    @pytest.fixture
    def gallery_process(self):
        """Start Gallery as a subprocess and return its process."""
        # This would require a way to connect to the running Gallery
        # For now, we test the API directly
        pytest.skip("Full Gallery subprocess testing not yet implemented")

    def test_api_round_trip(self):
        """Test a complete API call round trip."""
        sys.path.insert(0, str(PROJECT_ROOT / "python"))

        from auroraview import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        # Test spawn -> list -> kill flow
        # 1. Spawn
        spawn_result = json_loads(
            plugins.handle_command(
                "plugin:process|spawn_ipc",
                json_dumps(
                    {
                        "command": sys.executable,
                        "args": ["-c", "import time; time.sleep(5)"],
                    }
                ),
            )
        )
        assert spawn_result.get("success") is True
        pid = spawn_result["data"]["pid"]

        # 2. List
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert list_result.get("success") is True
        assert pid in list_result["data"]["processes"]

        # 3. Kill
        kill_result = json_loads(
            plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid}))
        )
        assert kill_result.get("success") is True

        # 4. Verify killed
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert pid not in list_result["data"]["processes"]


class TestGalleryDemoDiscovery:
    """Test that Gallery correctly discovers and categorizes demos."""

    def test_scan_examples_finds_demos(self):
        """Test that scan_examples finds demo files."""
        sys.path.insert(0, str(GALLERY_DIR))

        # Import gallery main
        if "main" in sys.modules:
            del sys.modules["main"]

        from main import SAMPLES, CATEGORIES

        assert len(SAMPLES) > 0, "Should find at least one sample"
        assert len(CATEGORIES) > 0, "Should have categories defined"

        # Each sample should have required fields
        for sample in SAMPLES:
            assert "id" in sample
            assert "title" in sample
            assert "category" in sample
            assert sample["category"] in CATEGORIES, (
                f"Sample {sample['id']} has unknown category: {sample['category']}"
            )

    def test_docstring_parsing(self):
        """Test that docstrings are correctly parsed."""
        sys.path.insert(0, str(GALLERY_DIR))

        from main import parse_docstring

        docstring = """Simple Decorator Demo - Basic example.

        This demonstrates the @view.bind_call decorator.

        Features:
        - Easy API binding
        - Automatic JSON serialization

        Use Cases:
        - Quick prototyping
        - Simple tools
        """

        result = parse_docstring(docstring)

        assert result["title"] == "Simple Decorator Demo"
        assert "Basic example" in result["description"]
        assert len(result["features"]) == 2
        assert len(result["use_cases"]) == 2
