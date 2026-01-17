# -*- coding: utf-8 -*-
"""Integration tests for AuroraView Inspector with real CDP connection.

These tests require a running AuroraView/browser instance with CDP enabled.
Run with: pytest tests/python/integration/test_inspector_cdp.py -v

Set CDP endpoint via environment variable:
    AURORAVIEW_TEST_CDP_ENDPOINT=http://localhost:9222 pytest ...
"""

from __future__ import annotations

import json
import os
import urllib.request
from urllib.error import URLError

import pytest


# Get test endpoint from environment or use default
def get_test_endpoint() -> str:
    return os.environ.get("AURORAVIEW_TEST_CDP_ENDPOINT", "http://localhost:9222")


def is_cdp_available() -> bool:
    """Check if CDP endpoint is available."""
    endpoint = get_test_endpoint()
    try:
        with urllib.request.urlopen(f"{endpoint}/json", timeout=2) as resp:
            data = json.loads(resp.read().decode())
            return isinstance(data, list)
    except (URLError, TimeoutError, json.JSONDecodeError):
        return False


# Skip all tests if CDP not available
pytestmark = pytest.mark.skipif(
    not is_cdp_available(),
    reason=f"CDP endpoint not available at {get_test_endpoint()}"
)


class TestInspectorConnection:
    """Tests for Inspector connection."""

    def test_connect_success(self):
        """Test successful connection to CDP endpoint."""
        from auroraview.testing import Inspector

        inspector = Inspector.connect(get_test_endpoint())
        try:
            # Should be connected
            # Note: Python implementation doesn't have is_connected method
            # but we can verify by taking a snapshot
            snap = inspector.snapshot()
            assert snap is not None
        finally:
            inspector.close()

    def test_connect_context_manager(self):
        """Test connection with context manager."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()
            assert snap is not None
            assert snap.url  # URL should not be empty


class TestInspectorSnapshot:
    """Tests for Inspector snapshot functionality."""

    def test_snapshot_basic(self):
        """Test basic snapshot."""
        from auroraview.testing import Inspector, Snapshot

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()

            assert isinstance(snap, Snapshot)
            assert snap.url
            assert isinstance(snap.viewport, tuple)
            assert len(snap.viewport) == 2
            assert snap.viewport[0] > 0
            assert snap.viewport[1] > 0

    def test_snapshot_refs(self):
        """Test snapshot refs."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()

            # refs is a dict
            assert isinstance(snap.refs, dict)

            # If page has interactive elements, there should be refs
            if snap.ref_count() > 0:
                # Get first ref
                first_key = next(iter(snap.refs.keys()))
                ref = snap.refs[first_key]

                # Check ref properties
                assert ref.ref_id
                assert ref.role
                # name might be empty

    def test_snapshot_find(self):
        """Test snapshot find functionality."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()

            # Find should return a list
            results = snap.find("button")  # Common element type
            assert isinstance(results, list)

            # get_ref with invalid ID should return None
            assert snap.get_ref("@999999") is None

    def test_snapshot_to_text(self):
        """Test snapshot text format."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()

            text = snap.to_text()
            assert isinstance(text, str)
            assert "Page:" in text
            assert "Viewport:" in text

    def test_snapshot_to_json(self):
        """Test snapshot JSON format."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()

            json_str = snap.to_json()
            assert isinstance(json_str, str)

            # Should be valid JSON
            data = json.loads(json_str)
            assert "title" in data
            assert "url" in data
            assert "viewport" in data

    def test_snapshot_str(self):
        """Test snapshot string representation."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            snap = inspector.snapshot()

            # str(snap) should work
            s = str(snap)
            assert isinstance(s, str)
            assert len(s) > 0


class TestInspectorScreenshot:
    """Tests for Inspector screenshot functionality."""

    def test_screenshot_png(self):
        """Test taking PNG screenshot."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            data = inspector.screenshot()

            assert isinstance(data, bytes)
            assert len(data) > 0
            # Check PNG magic bytes
            assert data[:4] == b"\x89PNG"

    def test_screenshot_save(self, tmp_path):
        """Test saving screenshot to file."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            path = tmp_path / "screenshot.png"
            data = inspector.screenshot(str(path))

            assert path.exists()
            assert path.read_bytes() == data


class TestInspectorNavigation:
    """Tests for Inspector navigation."""

    def test_goto(self):
        """Test navigation."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            result = inspector.goto("about:blank")

            assert result.success
            assert "goto" in result.action

    def test_back_forward(self):
        """Test back/forward navigation."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            # Navigate somewhere first
            inspector.goto("about:blank")

            # Back (might not have history)
            inspector.back()
            # Result depends on history

            # Forward
            inspector.forward()
            # Result depends on history

    def test_reload(self):
        """Test page reload."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            result = inspector.reload()

            assert result.success
            assert "reload" in result.action


class TestInspectorInteraction:
    """Tests for Inspector element interaction."""

    def test_press_key(self):
        """Test pressing keys."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            # Press Escape (should always work)
            result = inspector.press("Escape")
            assert result.success

            # Press Tab
            result = inspector.press("Tab")
            assert result.success

    def test_scroll(self):
        """Test scrolling."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            # Scroll down
            result = inspector.scroll("down", 100)
            assert result.success

            # Scroll up
            result = inspector.scroll("up", 100)
            assert result.success

    def test_click_invalid_ref(self):
        """Test clicking invalid ref."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            # Take snapshot first to populate refs
            inspector.snapshot()

            # Click invalid ref
            result = inspector.click("@999999")
            assert not result.success
            assert result.error


class TestInspectorEval:
    """Tests for Inspector JavaScript evaluation."""

    def test_eval_simple(self):
        """Test simple evaluation."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            result = inspector.eval("1 + 1")
            assert result == 2

    def test_eval_string(self):
        """Test string evaluation."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            result = inspector.eval("'hello'")
            assert result == "hello"

    def test_eval_object(self):
        """Test object evaluation."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            result = inspector.eval("({a: 1, b: 2})")
            assert isinstance(result, dict)
            assert result.get("a") == 1
            assert result.get("b") == 2


class TestInspectorWait:
    """Tests for Inspector wait functionality."""

    def test_wait_idle(self):
        """Test waiting for network idle."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            # Navigate to blank page first
            inspector.goto("about:blank")

            result = inspector.wait("idle", timeout=5.0)
            assert result is True

    def test_wait_loaded(self):
        """Test waiting for DOM loaded."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            result = inspector.wait("loaded", timeout=5.0)
            assert result is True

    def test_wait_timeout(self):
        """Test wait timeout."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            # Wait for text that doesn't exist
            result = inspector.wait("text:__nonexistent_text_12345__", timeout=0.5)
            assert result is False


class TestInspectorProperties:
    """Tests for Inspector properties."""

    def test_url(self):
        """Test getting current URL."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            url = inspector.url
            assert isinstance(url, str)

    def test_title(self):
        """Test getting current title."""
        from auroraview.testing import Inspector

        with Inspector.connect(get_test_endpoint()) as inspector:
            title = inspector.title
            assert isinstance(title, str)


class TestActionResult:
    """Tests for ActionResult."""

    def test_action_result_success(self):
        """Test successful action result."""
        from auroraview.testing import ActionResult

        result = ActionResult(
            success=True,
            action="test action",
            changes=["change1", "change2"],
            duration_ms=100,
        )

        assert result.success
        assert result.action == "test action"
        assert len(result.changes) == 2
        assert result.error is None
        assert bool(result) is True
        assert "test action" in str(result)

    def test_action_result_failure(self):
        """Test failed action result."""
        from auroraview.testing import ActionResult

        result = ActionResult(
            success=False,
            action="test action",
            error="something went wrong",
        )

        assert not result.success
        assert result.error == "something went wrong"
        assert bool(result) is False
        assert "wrong" in str(result)


class TestRefInfo:
    """Tests for RefInfo."""

    def test_ref_info_str(self):
        """Test RefInfo string representation."""
        from auroraview.testing import RefInfo

        ref = RefInfo(
            ref_id="@1",
            role="button",
            name="Submit",
            description="Submit form",
        )

        assert ref.ref_id == "@1"
        assert ref.role == "button"
        assert ref.name == "Submit"
        assert ref.description == "Submit form"

        s = str(ref)
        assert "@1" in s
        assert "button" in s
        assert "Submit" in s
