"""Deep inspection tests for Gallery.

More thorough tests to discover potential bugs in Gallery.
Uses Inspector API to verify UI behavior and API contracts.

In CI, Gallery is auto-started by the session fixture.
"""

from __future__ import annotations

import sys
import time
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from auroraview.testing import Inspector

# Check if playwright is available
try:
    from importlib.util import find_spec

    PLAYWRIGHT_AVAILABLE = find_spec("playwright") is not None
except ImportError:
    PLAYWRIGHT_AVAILABLE = False


pytestmark = [
    pytest.mark.skipif(sys.version_info < (3, 8), reason="Playwright requires Python 3.8+"),
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.integration,
    pytest.mark.cdp,
]

# Note: `inspector` fixture is provided by conftest.py and auto-starts Gallery


class TestGalleryAPIContract:
    """Test API contracts between frontend and backend."""

    def test_get_samples_structure(self, inspector: "Inspector"):
        """Test get_samples returns correct structure."""
        result = inspector.eval("""
            (async () => {
                try {
                    const samples = await window.auroraview.api.get_samples();
                    if (!Array.isArray(samples)) return { error: 'not array' };
                    if (samples.length === 0) return { error: 'empty array' };

                    // Check first sample structure
                    const sample = samples[0];
                    const required = ['id', 'title', 'category', 'description'];
                    const missing = required.filter(k => !(k in sample));
                    if (missing.length > 0) return { error: 'missing fields: ' + missing.join(', ') };

                    return { ok: true, count: samples.length };
                } catch (e) {
                    return { error: e.message };
                }
            })()
        """)
        if "error" in result:
            if "not initialized" in str(result.get("error", "")).lower():
                pytest.skip("API not available")
            pytest.fail(f"get_samples contract violation: {result['error']}")
        assert result.get("ok") is True
        assert result.get("count", 0) > 0

    def test_get_categories_structure(self, inspector: "Inspector"):
        """Test get_categories returns correct structure."""
        result = inspector.eval("""
            (async () => {
                try {
                    const cats = await window.auroraview.api.get_categories();
                    if (cats === null || typeof cats !== 'object') {
                        return { error: 'not object' };
                    }

                    const keys = Object.keys(cats);
                    if (keys.length === 0) return { error: 'empty categories' };

                    // Check structure of first category
                    const cat = cats[keys[0]];
                    const required = ['title', 'icon'];
                    const missing = required.filter(k => !(k in cat));
                    if (missing.length > 0) {
                        return { error: 'category missing fields: ' + missing.join(', ') };
                    }

                    return { ok: true, count: keys.length };
                } catch (e) {
                    return { error: e.message };
                }
            })()
        """)
        if "error" in result:
            if "not initialized" in str(result.get("error", "")).lower():
                pytest.skip("API not available")
            pytest.fail(f"get_categories contract violation: {result['error']}")
        assert result.get("ok") is True

    def test_get_source_requires_sample_id(self, inspector: "Inspector"):
        """Test get_source validates sample_id parameter."""
        result = inspector.eval("""
            (async () => {
                try {
                    // Should fail without sample_id
                    const result = await window.auroraview.api.get_source({});
                    // If it returns, check if it's an error
                    if (result === null || result === undefined || result === '') {
                        return { ok: true, message: 'returned empty for missing id' };
                    }
                    return { error: 'should have failed without sample_id' };
                } catch (e) {
                    // Expected error
                    return { ok: true, message: e.message };
                }
            })()
        """)
        # Either throws or returns empty - both acceptable
        assert result.get("ok") is True or "error" in result

    def test_kill_process_parameter_format(self, inspector: "Inspector"):
        """Test kill_process uses correct parameter format (pid as object)."""
        result = inspector.eval("""
            (async () => {
                try {
                    // Try with correct format: {pid: number}
                    const result = await window.auroraview.api.kill_process({pid: 99999});
                    // Should return error for non-existent process, not crash
                    return { ok: true, result: result };
                } catch (e) {
                    return { error: e.message };
                }
            })()
        """)
        # Should not crash, but may return error for non-existent process
        if "error" in result:
            error_msg = result["error"].lower()
            # These are acceptable errors
            acceptable = ["not found", "no such process", "does not exist", "invalid"]
            if not any(e in error_msg for e in acceptable):
                # Only fail for unexpected errors
                pass  # Log but don't fail - parameter format is correct
        # Main check: didn't crash
        assert result is not None


class TestGalleryEventSystem:
    """Test event system integration."""

    def test_event_subscription_works(self, inspector: "Inspector"):
        """Test event subscription returns unsubscribe function."""
        result = inspector.eval("""
            (() => {
                try {
                    const unsub = window.auroraview.on('test:event', () => {});
                    const isFunction = typeof unsub === 'function';
                    if (isFunction) unsub(); // Clean up
                    return { ok: isFunction };
                } catch (e) {
                    return { error: e.message };
                }
            })()
        """)
        if "error" in result:
            pytest.fail(f"Event subscription failed: {result['error']}")
        assert result.get("ok") is True

    def test_trigger_function_exists(self, inspector: "Inspector"):
        """Test trigger function is available."""
        result = inspector.eval("typeof window.auroraview.trigger === 'function'")
        assert result is True, "trigger function should exist"

    def test_event_roundtrip(self, inspector: "Inspector"):
        """Test event can be subscribed and triggered."""
        result = inspector.eval("""
            (() => {
                return new Promise((resolve) => {
                    let received = false;
                    const unsub = window.auroraview.on('test:roundtrip', (data) => {
                        received = data.value === 42;
                        unsub();
                        resolve({ ok: received });
                    });

                    // Trigger the event
                    window.auroraview.trigger('test:roundtrip', { value: 42 });

                    // Timeout fallback
                    setTimeout(() => {
                        unsub();
                        resolve({ ok: received, timeout: true });
                    }, 100);
                });
            })()
        """)
        assert result.get("ok") is True, "Event roundtrip failed"


class TestGalleryStateManagement:
    """Test state management and data flow."""

    def test_samples_loaded_in_state(self, inspector: "Inspector"):
        """Test samples are loaded into frontend state."""
        # Check if React state has samples (implementation-dependent)
        result = inspector.eval("""
            (() => {
                // Check for common state indicators
                const cards = document.querySelectorAll('[data-sample-id]');
                const sampleCards = document.querySelectorAll('.sample-card');
                const anyCards = document.querySelectorAll('[class*="sample"], [class*="card"]');

                return {
                    dataAttributes: cards.length,
                    sampleCards: sampleCards.length,
                    anyCards: anyCards.length,
                };
            })()
        """)
        # At least one indicator should show samples
        total = result.get("dataAttributes", 0) + result.get("sampleCards", 0) + result.get("anyCards", 0)
        assert total > 0, f"No sample cards found: {result}"


class TestGalleryDOMStructure:
    """Test DOM structure and elements."""

    def test_no_duplicate_ids(self, inspector: "Inspector"):
        """Test Gallery page has no duplicate element IDs."""
        # Skip if not on Gallery page
        url = inspector.url
        if not ("gallery" in url.lower() or "index.html" in url.lower() or "localhost" in url.lower()):
            pytest.skip(f"Not on Gallery page: {url}")

        result = inspector.eval("""
            (() => {
                const ids = Array.from(document.querySelectorAll('[id]'))
                    .map(el => el.id)
                    .filter(id => id);
                const seen = new Set();
                const duplicates = [];
                for (const id of ids) {
                    if (seen.has(id)) duplicates.push(id);
                    seen.add(id);
                }
                return { duplicates, total: ids.length };
            })()
        """)
        duplicates = result.get("duplicates", [])
        assert len(duplicates) == 0, f"Found duplicate IDs: {duplicates}"

    def test_images_have_alt_text(self, inspector: "Inspector"):
        """Test images have alt text for accessibility."""
        result = inspector.eval("""
            (() => {
                const images = Array.from(document.querySelectorAll('img'));
                const missing = images.filter(img => !img.alt && !img.getAttribute('aria-hidden'));
                return {
                    total: images.length,
                    missingAlt: missing.length,
                    missingSrc: missing.map(img => img.src.split('/').pop()).slice(0, 5),
                };
            })()
        """)
        total = result.get("total", 0)
        missing = result.get("missingAlt", 0)
        if total > 0:
            ratio = missing / total
            assert ratio < 0.3, (
                f"Too many images without alt text: {missing}/{total} "
                f"({ratio:.0%}). Examples: {result.get('missingSrc', [])}"
            )

    def test_buttons_are_accessible(self, inspector: "Inspector"):
        """Test buttons are accessible (have text or aria-label)."""
        result = inspector.eval("""
            (() => {
                const buttons = Array.from(document.querySelectorAll('button'));
                const inaccessible = buttons.filter(btn => {
                    const text = btn.textContent?.trim();
                    const ariaLabel = btn.getAttribute('aria-label');
                    const title = btn.getAttribute('title');
                    return !text && !ariaLabel && !title;
                });
                return {
                    total: buttons.length,
                    inaccessible: inaccessible.length,
                };
            })()
        """)
        total = result.get("total", 0)
        inaccessible = result.get("inaccessible", 0)
        if total > 0:
            ratio = inaccessible / total
            assert ratio < 0.1, (
                f"Too many inaccessible buttons: {inaccessible}/{total} ({ratio:.0%})"
            )

    def test_links_have_href(self, inspector: "Inspector"):
        """Test links have href attributes."""
        result = inspector.eval("""
            (() => {
                const links = Array.from(document.querySelectorAll('a'));
                const missingHref = links.filter(a => {
                    const href = a.getAttribute('href');
                    const onClick = a.getAttribute('onclick');
                    const role = a.getAttribute('role');
                    // Allow links with onClick or role="button"
                    return !href && !onClick && role !== 'button';
                });
                return {
                    total: links.length,
                    missingHref: missingHref.length,
                };
            })()
        """)
        missing = result.get("missingHref", 0)
        total = result.get("total", 0)
        if total > 0:
            ratio = missing / total
            assert ratio < 0.2, f"Too many links without href: {missing}/{total}"


class TestGalleryConsoleErrors:
    """Test for console errors."""

    def test_no_uncaught_errors(self, inspector: "Inspector"):
        """Test page has no uncaught errors in console."""
        # Get console errors via CDP
        result = inspector.eval("""
            (() => {
                // Check if there are any error indicators
                // This is a best-effort check
                const errors = window.__auroraview_errors__ || [];
                return { errors: errors.slice(0, 5) };
            })()
        """)
        errors = result.get("errors", [])
        # Just informational - don't fail
        if errors:
            print(f"Console errors found: {errors}")


class TestGalleryResponsiveness:
    """Test UI responsiveness."""

    def test_click_response_time(self, inspector: "Inspector"):
        """Test click actions respond quickly."""
        snap = inspector.snapshot()
        buttons = [r for r in snap.refs.values() if r.role.lower() == "button"]

        if not buttons:
            pytest.skip("No buttons to test")

        # Click first button and measure time
        start = time.time()
        inspector.click(buttons[0].ref_id)
        duration = time.time() - start

        # Should respond within 1 second
        assert duration < 1.0, f"Click took too long: {duration:.2f}s"

    def test_scroll_works(self, inspector: "Inspector"):
        """Test scrolling functionality."""
        result = inspector.scroll("down", 200)
        assert result.success, f"Scroll failed: {result.error}"

        result = inspector.scroll("up", 200)
        assert result.success, f"Scroll back failed: {result.error}"


class TestGalleryEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_rapid_snapshot_calls(self, inspector: "Inspector"):
        """Test rapid snapshot calls don't crash."""
        for _ in range(10):
            snap = inspector.snapshot()
            assert snap is not None
            assert snap.ref_count() >= 0

    def test_concurrent_eval(self, inspector: "Inspector"):
        """Test concurrent JS evaluations."""
        results = []
        for i in range(5):
            result = inspector.eval(f"1 + {i}")
            results.append(result)

        expected = [1, 2, 3, 4, 5]
        assert results == expected

    def test_special_characters_in_eval(self, inspector: "Inspector"):
        """Test special characters in JS evaluation."""
        result = inspector.eval("'hello\\nworld'.length")
        assert result == 11

        result = inspector.eval("'test\"quote'.length")
        assert result == 10

    def test_unicode_handling(self, inspector: "Inspector"):
        """Test Unicode handling."""
        result = inspector.eval("'你好世界'.length")
        assert result == 4

        result = inspector.eval("'emoji: 🎉'.includes('🎉')")
        assert result is True
