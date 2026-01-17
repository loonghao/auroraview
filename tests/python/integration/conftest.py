"""
Pytest configuration for integration tests.

Provides fixtures for Gallery CDP testing that work in both:
- Local development: Connect to existing Gallery
- CI environment: Auto-start Gallery, run tests, cleanup
"""

from __future__ import annotations

import atexit
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

import pytest

# Project paths
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
GALLERY_DIR = PROJECT_ROOT / "gallery"
DIST_DIR = GALLERY_DIR / "dist"

# CDP configuration
CDP_PORT = int(os.environ.get("WEBVIEW2_CDP_PORT", "9222"))
CDP_URL = os.environ.get("WEBVIEW2_CDP_URL", f"http://127.0.0.1:{CDP_PORT}")

# Timeouts
GALLERY_STARTUP_TIMEOUT = 60  # seconds
GALLERY_READY_TIMEOUT = 30  # seconds


def _is_package_installed() -> bool:
    """Check if auroraview is installed as a package (with _core module)."""
    try:
        import importlib.util

        spec = importlib.util.find_spec("auroraview._core")
        return spec is not None
    except (ImportError, ModuleNotFoundError):
        return False


# Only add source paths if the package is not installed
# This allows CI to use the installed wheel while local dev uses source
if not _is_package_installed():
    sys.path.insert(0, str(PROJECT_ROOT / "python"))


# ─────────────────────────────────────────────────────────────────────────────
# CDP Utilities
# ─────────────────────────────────────────────────────────────────────────────


def is_cdp_available(url: str = CDP_URL, timeout: float = 2.0) -> bool:
    """Check if CDP endpoint is available."""
    import urllib.error
    import urllib.request

    try:
        req = urllib.request.urlopen(f"{url}/json/version", timeout=timeout)
        req.close()
        return True
    except (urllib.error.URLError, OSError):
        return False


def wait_for_cdp(url: str = CDP_URL, timeout: int = GALLERY_STARTUP_TIMEOUT) -> bool:
    """Wait for CDP endpoint to become available."""
    start = time.time()
    while time.time() - start < timeout:
        if is_cdp_available(url, timeout=2.0):
            return True
        time.sleep(0.5)
    return False


# ─────────────────────────────────────────────────────────────────────────────
# Gallery Process Manager
# ─────────────────────────────────────────────────────────────────────────────


class GalleryProcess:
    """Manage Gallery process for testing."""

    _instance: Optional["GalleryProcess"] = None
    _started_by_us: bool = False

    def __init__(self):
        self.process: Optional[subprocess.Popen] = None
        self._cleanup_registered = False

    @classmethod
    def get_instance(cls) -> "GalleryProcess":
        """Get singleton instance."""
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def start(self, timeout: int = GALLERY_STARTUP_TIMEOUT) -> bool:
        """Start Gallery with CDP enabled.

        Returns True if Gallery is ready (either already running or started).
        """
        # Check if already running
        if is_cdp_available():
            print("[GalleryProcess] CDP already available, using existing Gallery")
            return True

        # Find Gallery executable or script
        gallery_exe = self._find_gallery_executable()
        if not gallery_exe:
            print("[GalleryProcess] No Gallery executable found")
            return False

        print(f"[GalleryProcess] Starting Gallery: {gallery_exe}")

        # Set environment for CDP
        env = os.environ.copy()
        env["AURORAVIEW_CDP_PORT"] = str(CDP_PORT)

        # Start process
        if gallery_exe.suffix == ".exe":
            # Packed Gallery
            self.process = subprocess.Popen(
                [str(gallery_exe)],
                cwd=str(gallery_exe.parent),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
            )
        else:
            # Python script
            self.process = subprocess.Popen(
                [sys.executable, str(gallery_exe)],
                cwd=str(PROJECT_ROOT),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
            )

        GalleryProcess._started_by_us = True
        print(f"[GalleryProcess] Started with PID {self.process.pid}")

        # Register cleanup
        if not self._cleanup_registered:
            atexit.register(self.stop)
            self._cleanup_registered = True

        # Wait for CDP
        print(f"[GalleryProcess] Waiting for CDP on port {CDP_PORT}...")
        if not wait_for_cdp(timeout=timeout):
            print("[GalleryProcess] CDP not available after timeout")
            self.stop()
            return False

        print("[GalleryProcess] CDP is ready!")
        return True

    def stop(self):
        """Stop Gallery if we started it."""
        if not GalleryProcess._started_by_us:
            return

        if self.process and self.process.poll() is None:
            print("[GalleryProcess] Stopping Gallery...")
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                print("[GalleryProcess] Force killing Gallery...")
                self.process.kill()
                self.process.wait(timeout=5)

        self.process = None
        GalleryProcess._started_by_us = False
        print("[GalleryProcess] Gallery stopped")

    def _find_gallery_executable(self) -> Optional[Path]:
        """Find Gallery executable or script."""
        # Priority order:
        # 1. Packed executable (CI)
        packed_exe = GALLERY_DIR / "pack-output" / "auroraview-gallery.exe"
        if packed_exe.exists():
            return packed_exe

        # 2. Python script (local dev)
        gallery_script = GALLERY_DIR / "main.py"
        if gallery_script.exists():
            # Check if dist is built
            if (DIST_DIR / "index.html").exists():
                return gallery_script
            print("[GalleryProcess] Gallery dist not built, run 'just gallery-build'")

        return None

    def is_running(self) -> bool:
        """Check if Gallery process is running."""
        if self.process and self.process.poll() is None:
            return True
        return is_cdp_available()


# ─────────────────────────────────────────────────────────────────────────────
# Session-scoped Gallery Fixture
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture(scope="session")
def gallery_cdp_session():
    """Session-scoped fixture that ensures Gallery is running with CDP.

    This fixture:
    - Connects to existing Gallery if CDP is available
    - Starts Gallery automatically if not running
    - Stops Gallery on session end if we started it

    Usage:
        def test_something(gallery_cdp_session, inspector):
            # Gallery is guaranteed to be running
            pass
    """
    manager = GalleryProcess.get_instance()

    if not manager.start():
        pytest.skip("Failed to start Gallery with CDP")

    yield manager

    # Cleanup is handled by atexit


@pytest.fixture(scope="session")
def cdp_url(gallery_cdp_session) -> str:
    """Provide CDP URL after Gallery is started."""
    return CDP_URL


# ─────────────────────────────────────────────────────────────────────────────
# Inspector Fixtures
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture
def inspector(gallery_cdp_session):
    """Create Inspector connected to Gallery.

    Usage:
        def test_gallery(inspector):
            snap = inspector.snapshot()
            inspector.click("@button-id")
    """
    from auroraview.testing import Inspector

    inspector = Inspector.connect(CDP_URL)

    # Ensure we're on Gallery page
    _ensure_gallery_page(inspector)

    yield inspector
    inspector.close()


@pytest.fixture(scope="class")
def inspector_class(gallery_cdp_session):
    """Class-scoped Inspector for test classes.

    Usage:
        class TestGallery:
            def test_one(self, inspector_class):
                pass
            def test_two(self, inspector_class):
                pass
    """
    from auroraview.testing import Inspector

    inspector = Inspector.connect(CDP_URL)
    _ensure_gallery_page(inspector)

    yield inspector
    inspector.close()


def _ensure_gallery_page(inspector) -> None:
    """Ensure Inspector is on Gallery page, not external content."""
    url = inspector.url or ""

    # Check if on Gallery page
    if any(x in url.lower() for x in ["gallery", "index.html", "localhost", "file://"]):
        # Wait for auroraview bridge
        try:
            inspector.wait("js", "typeof window.auroraview !== 'undefined'", timeout=10)
        except Exception:
            pass
        return

    # Navigate to Gallery
    gallery_index = DIST_DIR / "index.html"
    if gallery_index.exists():
        inspector.goto(f"file:///{gallery_index}")
        inspector.wait("idle", timeout=10)


# ─────────────────────────────────────────────────────────────────────────────
# Gallery Fixtures (Non-CDP)
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
# Legacy CDP Fixture (async)
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture
async def cdp_page(gallery_cdp_session):
    """Connect to running Gallery via CDP using Playwright.

    Usage:
        async def test_cdp(cdp_page):
            await cdp_page.click('button')
    """
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
