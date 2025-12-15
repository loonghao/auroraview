"""Pytest configuration for CDP-based testing.

This module provides fixtures for testing AuroraView applications
via Chrome DevTools Protocol (CDP) connection.

Usage:
    # Copy or import into your conftest.py
    from tests.conftest_cdp import *

    # Or use directly in tests:
    pytest tests/test_examples_cdp.py --confcutdir=tests
"""

from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Generator, Optional

import pytest

# Add project paths
PROJECT_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))


# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

CDP_URL = os.environ.get("WEBVIEW2_CDP_URL", "http://127.0.0.1:9222")
CDP_PORT = int(os.environ.get("WEBVIEW2_CDP_PORT", "9222"))


def pytest_configure(config):
    """Register custom markers for CDP testing."""
    config.addinivalue_line("markers", "cdp: mark test as requiring CDP connection")
    config.addinivalue_line("markers", "requires_gallery: mark test as requiring gallery process")


def pytest_collection_modifyitems(config, items):
    """Skip CDP tests if CDP is not available."""

    skip_cdp = pytest.mark.skip(reason="CDP endpoint not available")

    for item in items:
        if "cdp" in item.keywords or "requires_gallery" in item.keywords:
            # Check if CDP is available
            if not _is_cdp_available():
                item.add_marker(skip_cdp)


def _is_cdp_available(timeout: float = 2.0) -> bool:
    """Check if CDP endpoint is available."""
    import urllib.error
    import urllib.request

    try:
        req = urllib.request.urlopen(f"{CDP_URL}/json/version", timeout=timeout)
        req.close()
        return True
    except (urllib.error.URLError, OSError):
        return False


def _wait_for_cdp(timeout: float = 30.0) -> bool:
    """Wait for CDP endpoint to become available."""
    start = time.time()
    while time.time() - start < timeout:
        if _is_cdp_available(timeout=2.0):
            return True
        time.sleep(0.5)
    return False


# ─────────────────────────────────────────────────────────────────────────────
# Fixtures
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture(scope="session")
def cdp_url() -> str:
    """Get CDP URL for testing."""
    return CDP_URL


@pytest.fixture(scope="session")
def cdp_available() -> bool:
    """Check if CDP is available."""
    return _is_cdp_available()


@pytest.fixture(scope="module")
def webview2_cdp():
    """Create a WebView2 CDP connection for testing.

    This fixture connects to an existing WebView2 instance running
    with remote debugging enabled.

    Example:
        def test_example(webview2_cdp):
            webview2_cdp.goto("https://example.com")
            assert webview2_cdp.text("h1") == "Example Domain"
    """
    from auroraview.testing import HeadlessWebView

    if not _is_cdp_available():
        pytest.skip("CDP endpoint not available")

    webview = HeadlessWebView.webview2_cdp(CDP_URL, timeout=30)
    try:
        yield webview
    finally:
        webview.close()


@pytest.fixture(scope="function")
def cdp_page(webview2_cdp):
    """Get the Playwright page from CDP connection.

    Provides direct access to the Playwright Page object for
    advanced operations.

    Example:
        def test_example(cdp_page):
            cdp_page.goto("https://example.com")
            assert cdp_page.title() == "Example Domain"
    """
    return webview2_cdp.page


@pytest.fixture(scope="module")
def gallery_samples(webview2_cdp) -> list:
    """Get all samples from the gallery.

    Returns a list of sample dictionaries with id, title, category, etc.
    """
    # Wait for gallery to be ready
    time.sleep(1)

    # Get samples via API
    samples = webview2_cdp.evaluate("""
        (async () => {
            if (typeof auroraview === 'undefined') {
                return [];
            }
            try {
                return await auroraview.api.get_samples();
            } catch (e) {
                console.error('Failed to get samples:', e);
                return [];
            }
        })()
    """)

    return samples if isinstance(samples, list) else []


# ─────────────────────────────────────────────────────────────────────────────
# Gallery Process Fixture
# ─────────────────────────────────────────────────────────────────────────────


class GalleryProcessManager:
    """Manage gallery process lifecycle for testing."""

    def __init__(self, exe_path: Optional[Path] = None):
        self.exe_path = exe_path or (
            PROJECT_ROOT / "gallery" / "pack-output" / "auroraview-gallery.exe"
        )
        self.process: Optional[subprocess.Popen] = None
        self._started_by_us = False

    def ensure_running(self, timeout: float = 30.0) -> bool:
        """Ensure gallery is running, starting it if necessary."""
        # Check if already running
        if _is_cdp_available():
            return True

        # Start gallery
        if not self.exe_path.exists():
            return False

        self.process = subprocess.Popen(
            [str(self.exe_path)],
            cwd=str(self.exe_path.parent),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        self._started_by_us = True

        return _wait_for_cdp(timeout)

    def stop(self):
        """Stop gallery if we started it."""
        if self._started_by_us and self.process:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
            self.process = None
            self._started_by_us = False


@pytest.fixture(scope="session")
def gallery_manager() -> Generator[GalleryProcessManager, None, None]:
    """Session-scoped gallery process manager.

    Starts gallery if not already running, and stops it at session end
    if we started it.
    """
    manager = GalleryProcessManager()

    if not manager.ensure_running():
        pytest.skip("Gallery process not available")

    yield manager

    manager.stop()


# ─────────────────────────────────────────────────────────────────────────────
# Example Running Fixtures
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def run_example(webview2_cdp):
    """Factory fixture to run examples via gallery API.

    Example:
        def test_simple_decorator(run_example):
            result = run_example("simple_decorator")
            assert result["ok"], f"Failed: {result.get('error')}"
    """

    def _run_example(sample_id: str, show_console: bool = False) -> dict:
        """Run an example and return the result."""
        result = webview2_cdp.evaluate(f"""
            (async () => {{
                try {{
                    return await auroraview.api.run_sample({{
                        sample_id: "{sample_id}",
                        show_console: {str(show_console).lower()}
                    }});
                }} catch (e) {{
                    return {{ ok: false, error: e.message }};
                }}
            }})()
        """)
        return result if isinstance(result, dict) else {"ok": False, "error": "Invalid response"}

    return _run_example


@pytest.fixture(scope="module")
def kill_process(webview2_cdp):
    """Factory fixture to kill processes via gallery API.

    Example:
        def test_example(run_example, kill_process):
            result = run_example("simple_decorator")
            if result.get("pid"):
                kill_process(result["pid"])
    """

    def _kill_process(pid: int) -> dict:
        """Kill a process by PID."""
        result = webview2_cdp.evaluate(f"""
            (async () => {{
                try {{
                    return await auroraview.api.kill_process({{ pid: {pid} }});
                }} catch (e) {{
                    return {{ ok: false, error: e.message }};
                }}
            }})()
        """)
        return result if isinstance(result, dict) else {"ok": False, "error": "Invalid response"}

    return _kill_process


@pytest.fixture(scope="module")
def list_processes(webview2_cdp):
    """Factory fixture to list running processes.

    Example:
        def test_processes(list_processes):
            result = list_processes()
            assert result["ok"]
            print(result["processes"])
    """

    def _list_processes() -> dict:
        """List all running processes."""
        result = webview2_cdp.evaluate("""
            (async () => {
                try {
                    return await auroraview.api.list_processes();
                } catch (e) {
                    return { ok: false, error: e.message };
                }
            })()
        """)
        return result if isinstance(result, dict) else {"ok": False, "error": "Invalid response"}

    return _list_processes
