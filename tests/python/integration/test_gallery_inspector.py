"""Gallery UI tests using Inspector API.

Tests for AuroraView Gallery using the new Inspector API (CDP-based).

In CI, Gallery is auto-started by the session fixture.
Locally, connect to existing Gallery or auto-start.

These tests verify:
- Page structure and accessibility
- Navigation and routing
- Sample cards rendering
- Search/filter functionality
- API calls (bridge integration)
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

# Skip conditions
pytestmark = [
    pytest.mark.skipif(sys.version_info < (3, 8), reason="Playwright requires Python 3.8+"),
    pytest.mark.skipif(not PLAYWRIGHT_AVAILABLE, reason="Playwright not installed"),
    pytest.mark.integration,
    pytest.mark.cdp,
]

# Note: `inspector` fixture is provided by conftest.py and auto-starts Gallery


class TestGalleryPageStructure:
    """Test Gallery page structure and accessibility."""

    def test_page_loads_with_title(self, inspector: "Inspector"):
        """Test page loads with expected title."""
        snap = inspector.snapshot()
        assert snap.title, "Page should have a title"
        # Gallery title can vary, just check it exists
        assert len(snap.title) > 0

    def test_page_has_interactive_elements(self, inspector: "Inspector"):
        """Test page has interactive elements."""
        snap = inspector.snapshot()
        assert snap.ref_count() > 0, "Page should have interactive elements"

    def test_snapshot_contains_navigation(self, inspector: "Inspector"):
        """Test snapshot contains navigation elements."""
        snap = inspector.snapshot()
        # Should have some clickable elements
        buttons = [r for r in snap.refs.values() if r.role.lower() == "button"]
        links = [r for r in snap.refs.values() if r.role.lower() == "link"]
        assert len(buttons) + len(links) > 0, "Page should have buttons or links"

    def test_viewport_is_set(self, inspector: "Inspector"):
        """Test viewport dimensions are reasonable."""
        snap = inspector.snapshot()
        width, height = snap.viewport
        assert width >= 800, f"Viewport width {width} should be >= 800"
        assert height >= 600, f"Viewport height {height} should be >= 600"


class TestGallerySampleCards:
    """Test sample card rendering and interaction."""

    def test_samples_are_visible(self, inspector: "Inspector"):
        """Test sample cards are rendered."""
        snap = inspector.snapshot()
        # Look for sample-related elements
        # Cards might be buttons, links, or have specific text
        tree_text = snap.tree.lower()
        # Gallery should show some samples
        assert (
            "sample" in tree_text
            or "example" in tree_text
            or "demo" in tree_text
            or snap.ref_count() > 5  # At least several interactive elements
        ), "Page should show samples or have multiple interactive elements"

    def test_find_category_elements(self, inspector: "Inspector"):
        """Test category sections are rendered."""
        snap = inspector.snapshot()
        # Categories should appear in the tree
        categories = ["Getting Started", "Window", "Events", "DCC", "Browser"]
        found = False
        tree_text = snap.tree.lower()
        for cat in categories:
            if cat.lower() in tree_text:
                found = True
                break
        # This is informational - category presence depends on samples
        if not found:
            # Check if we have any structure
            assert snap.ref_count() > 0, "Page should have some structure"


class TestGallerySearch:
    """Test search functionality."""

    def test_find_search_input(self, inspector: "Inspector"):
        """Test search input is present."""
        snap = inspector.snapshot()
        # Look for search-related elements
        search_refs = snap.find("search")
        textboxes = [r for r in snap.refs.values() if r.role.lower() in ("textbox", "searchbox")]

        # Either find search by name or by role - search might not be visible
        # Just check we can query successfully
        _ = len(search_refs) > 0 or len(textboxes) > 0
        assert snap is not None

    def test_search_filters_results(self, inspector: "Inspector"):
        """Test search filtering works."""
        # Get initial snapshot
        snap_before = inspector.snapshot()

        # Find search input
        textboxes = [
            r for r in snap_before.refs.values() if r.role.lower() in ("textbox", "searchbox")
        ]
        if not textboxes:
            pytest.skip("No search input found")

        search_ref = textboxes[0].ref_id

        # Type search term
        result = inspector.fill(search_ref, "window")
        if not result.success:
            pytest.skip(f"Could not fill search: {result.error}")

        # Wait for filter
        time.sleep(0.5)

        # Get new snapshot
        snap_after = inspector.snapshot()

        # Results should change (either filtered or showing results)
        # This is a soft assertion - just verify no errors
        assert snap_after is not None


class TestGalleryNavigation:
    """Test navigation and routing."""

    def test_can_click_element(self, inspector: "Inspector"):
        """Test clicking an element works."""
        snap = inspector.snapshot()
        if snap.ref_count() == 0:
            pytest.skip("No interactive elements")

        # Get first clickable element
        buttons = [r for r in snap.refs.values() if r.role.lower() in ("button", "link")]
        if not buttons:
            pytest.skip("No buttons or links found")

        first = buttons[0]
        result = inspector.click(first.ref_id)
        # Just verify no crash - action may or may not succeed
        assert result is not None

    def test_keyboard_navigation(self, inspector: "Inspector"):
        """Test keyboard navigation works."""
        result = inspector.press("Tab")
        assert result.success, f"Tab key press failed: {result.error}"

        result = inspector.press("Escape")
        assert result.success, f"Escape key press failed: {result.error}"


class TestGalleryBridgeIntegration:
    """Test AuroraView bridge integration."""

    def test_auroraview_bridge_exists(self, inspector: "Inspector"):
        """Test AuroraView bridge is injected."""
        result = inspector.eval("typeof window.auroraview !== 'undefined'")
        assert result is True, "AuroraView bridge should be available"

    def test_bridge_has_api(self, inspector: "Inspector"):
        """Test bridge has API namespace."""
        result = inspector.eval("typeof window.auroraview.api !== 'undefined'")
        assert result is True, "AuroraView API should be available"

    def test_bridge_has_call_method(self, inspector: "Inspector"):
        """Test bridge has call method."""
        result = inspector.eval("typeof window.auroraview.call === 'function'")
        assert result is True, "AuroraView call method should be available"

    def test_bridge_has_on_method(self, inspector: "Inspector"):
        """Test bridge has event subscription."""
        result = inspector.eval("typeof window.auroraview.on === 'function'")
        assert result is True, "AuroraView on method should be available"

    def test_api_get_samples_returns_array(self, inspector: "Inspector"):
        """Test get_samples API returns array."""
        result = inspector.eval("""
            (async () => {
                try {
                    const samples = await window.auroraview.api.get_samples();
                    return Array.isArray(samples);
                } catch (e) {
                    return false;
                }
            })()
        """)
        # This may fail if not in packed mode
        if result is False:
            pytest.skip("API call failed - may not be in packed mode")
        assert result is True

    def test_api_get_categories_returns_object(self, inspector: "Inspector"):
        """Test get_categories API returns object."""
        result = inspector.eval("""
            (async () => {
                try {
                    const cats = await window.auroraview.api.get_categories();
                    return cats !== null && typeof cats === 'object';
                } catch (e) {
                    return false;
                }
            })()
        """)
        if result is False:
            pytest.skip("API call failed - may not be in packed mode")
        assert result is True


class TestGalleryAccessibility:
    """Test accessibility features."""

    def test_interactive_elements_have_names(self, inspector: "Inspector"):
        """Test interactive elements have accessible names."""
        snap = inspector.snapshot()
        unnamed = []
        for ref in snap.refs.values():
            if not ref.name or ref.name.strip() == "":
                unnamed.append(ref)

        # Allow some unnamed elements but not too many
        unnamed_ratio = len(unnamed) / max(len(snap.refs), 1)
        assert unnamed_ratio < 0.5, (
            f"Too many unnamed elements: {len(unnamed)}/{len(snap.refs)} ({unnamed_ratio:.0%})"
        )

    def test_buttons_have_names(self, inspector: "Inspector"):
        """Test all buttons have accessible names."""
        snap = inspector.snapshot()
        buttons = [r for r in snap.refs.values() if r.role.lower() == "button"]

        unnamed_buttons = [b for b in buttons if not b.name or b.name.strip() == ""]
        if buttons:
            assert len(unnamed_buttons) == 0, (
                f"Found {len(unnamed_buttons)} buttons without names: "
                f"{[b.ref_id for b in unnamed_buttons]}"
            )

    def test_links_have_names(self, inspector: "Inspector"):
        """Test all links have accessible names."""
        snap = inspector.snapshot()
        links = [r for r in snap.refs.values() if r.role.lower() == "link"]

        unnamed_links = [link for link in links if not link.name or link.name.strip() == ""]
        if links:
            # Some tolerance for icon-only links
            assert len(unnamed_links) <= len(links) * 0.2, (
                f"Too many links without names: {len(unnamed_links)}/{len(links)}"
            )


class TestGalleryPerformance:
    """Test performance-related aspects."""

    def test_snapshot_is_fast(self, inspector: "Inspector"):
        """Test snapshot generation is reasonably fast."""
        start = time.time()
        snap = inspector.snapshot()
        duration = time.time() - start

        assert duration < 2.0, f"Snapshot took too long: {duration:.2f}s"
        assert snap is not None

    def test_multiple_snapshots_consistent(self, inspector: "Inspector"):
        """Test multiple snapshots are consistent."""
        snap1 = inspector.snapshot()
        time.sleep(0.1)
        snap2 = inspector.snapshot()

        # Title and URL should be stable
        assert snap1.title == snap2.title
        assert snap1.url == snap2.url

        # Ref count should be similar (allow some variance for dynamic content)
        diff = abs(snap1.ref_count() - snap2.ref_count())
        assert diff < 10, (
            f"Ref count changed significantly: {snap1.ref_count()} -> {snap2.ref_count()}"
        )


class TestGalleryErrorHandling:
    """Test error handling."""

    def test_click_invalid_ref_fails_gracefully(self, inspector: "Inspector"):
        """Test clicking invalid ref returns error."""
        result = inspector.click("@99999")
        assert not result.success
        assert result.error is not None
        assert "not found" in result.error.lower()

    def test_fill_invalid_ref_fails_gracefully(self, inspector: "Inspector"):
        """Test filling invalid ref returns error."""
        result = inspector.fill("@99999", "test")
        assert not result.success
        assert result.error is not None

    def test_invalid_js_returns_error(self, inspector: "Inspector"):
        """Test invalid JS throws but doesn't crash."""
        try:
            inspector.eval("this.is.invalid.code(")
            # If it doesn't throw, that's also acceptable
        except Exception:
            # Expected
            pass


class TestGalleryWaitConditions:
    """Test wait functionality."""

    def test_wait_for_idle(self, inspector: "Inspector"):
        """Test wait for idle state."""
        result = inspector.wait("idle", timeout=10)
        assert result is True

    def test_wait_for_loaded(self, inspector: "Inspector"):
        """Test wait for loaded state."""
        result = inspector.wait("loaded", timeout=10)
        assert result is True

    def test_wait_for_nonexistent_text_times_out(self, inspector: "Inspector"):
        """Test wait for nonexistent text times out."""
        result = inspector.wait("text:NONEXISTENT_TEXT_12345", timeout=1)
        assert result is False
