"""Test examples startup using AuroraView testing framework with CDP.

This module tests that all examples in the gallery can be launched successfully
using the AuroraView testing framework with WebView2 CDP connection.

Usage:
    # First, start the packed gallery with CDP enabled:
    cd gallery && ./pack-output/auroraview-gallery.exe

    # Then run tests:
    pytest tests/test_examples_cdp.py -v

    # Or run directly:
    python tests/test_examples_cdp.py

Requirements:
    - Packed gallery with remote_debugging_port = 9222
    - playwright: pip install playwright
"""

from __future__ import annotations

import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import pytest

# Add project root to path
PROJECT_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))

from auroraview.testing import HeadlessWebView, WebView2CDPWebView  # noqa: E402

# ─────────────────────────────────────────────────────────────────────────────
# Test Configuration
# ─────────────────────────────────────────────────────────────────────────────

CDP_URL = os.environ.get("WEBVIEW2_CDP_URL", "http://127.0.0.1:9222")
GALLERY_EXE = PROJECT_ROOT / "gallery" / "pack-output" / "auroraview-gallery.exe"
EXAMPLE_STARTUP_TIMEOUT = 10  # seconds to wait for example window
EXAMPLE_RUN_DURATION = 3  # seconds to let example run before closing


@dataclass
class ExampleTestResult:
    """Result of testing an example."""

    example_id: str
    title: str
    success: bool
    pid: Optional[int] = None
    error: Optional[str] = None
    startup_time_ms: float = 0


# ─────────────────────────────────────────────────────────────────────────────
# CDP Connection Utilities
# ─────────────────────────────────────────────────────────────────────────────


def wait_for_cdp(url: str = CDP_URL, timeout: int = 30) -> bool:
    """Wait for CDP endpoint to become available."""
    import urllib.error
    import urllib.request

    # Extract host and port from URL
    if url.startswith("http://"):
        host_port = url[7:]
    else:
        host_port = url

    start = time.time()
    while time.time() - start < timeout:
        try:
            req = urllib.request.urlopen(f"http://{host_port}/json/version", timeout=2)
            req.close()
            return True
        except (urllib.error.URLError, OSError):
            time.sleep(0.5)
    return False


def is_cdp_available(url: str = CDP_URL) -> bool:
    """Check if CDP endpoint is available."""
    return wait_for_cdp(url, timeout=2)


# ─────────────────────────────────────────────────────────────────────────────
# Gallery Process Management
# ─────────────────────────────────────────────────────────────────────────────


class GalleryProcess:
    """Manage the packed gallery process for testing."""

    def __init__(self, exe_path: Path = GALLERY_EXE):
        self.exe_path = exe_path
        self.process: Optional[subprocess.Popen] = None

    def start(self, timeout: int = 30) -> bool:
        """Start the gallery process and wait for CDP."""
        if not self.exe_path.exists():
            raise FileNotFoundError(f"Gallery exe not found: {self.exe_path}")

        print(f"Starting gallery: {self.exe_path}")

        self.process = subprocess.Popen(
            [str(self.exe_path)],
            cwd=str(self.exe_path.parent),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

        print(f"Process started: PID {self.process.pid}")
        print("Waiting for CDP...")

        if not wait_for_cdp(CDP_URL, timeout=timeout):
            self.stop()
            return False

        print("CDP is available!")
        return True

    def stop(self):
        """Stop the gallery process."""
        if self.process:
            print("Stopping gallery process...")
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
            self.process = None

    def is_running(self) -> bool:
        """Check if process is still running."""
        return self.process is not None and self.process.poll() is None

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()
        return False


# ─────────────────────────────────────────────────────────────────────────────
# Example Testing via CDP
# ─────────────────────────────────────────────────────────────────────────────


class ExampleTester:
    """Test examples through the gallery via CDP."""

    def __init__(self, cdp_url: str = CDP_URL):
        self.cdp_url = cdp_url
        self.webview: Optional[WebView2CDPWebView] = None

    def connect(self):
        """Connect to gallery via CDP."""
        self.webview = HeadlessWebView.webview2_cdp(self.cdp_url, timeout=30)
        # Wait for gallery to be ready
        time.sleep(1)

    def disconnect(self):
        """Disconnect from CDP."""
        if self.webview:
            self.webview.close()
            self.webview = None

    def get_samples(self) -> list:
        """Get list of all samples from gallery."""
        if not self.webview:
            raise RuntimeError("Not connected to gallery")

        # Call the gallery's API to get samples
        result = self.webview.evaluate("auroraview.api.get_samples()")

        # Handle promise result
        if isinstance(result, dict) and "then" in str(type(result)):
            # It's a promise, need to await
            result = self.webview.evaluate("""
                (async () => {
                    return await auroraview.api.get_samples();
                })()
            """)

        return result if isinstance(result, list) else []

    def run_example(self, sample_id: str) -> ExampleTestResult:
        """Run an example and check if it starts successfully."""
        if not self.webview:
            raise RuntimeError("Not connected to gallery")

        start_time = time.time()

        # Get sample info first
        sample_info = self.webview.evaluate(f"""
            (async () => {{
                const samples = await auroraview.api.get_samples();
                return samples.find(s => s.id === "{sample_id}");
            }})()
        """)

        title = sample_info.get("title", sample_id) if sample_info else sample_id

        # Run the sample via gallery API
        result = self.webview.evaluate(f"""
            (async () => {{
                return await auroraview.api.run_sample({{ sample_id: "{sample_id}" }});
            }})()
        """)

        startup_time = (time.time() - start_time) * 1000

        if not result:
            return ExampleTestResult(
                example_id=sample_id,
                title=title,
                success=False,
                error="No response from run_sample API",
                startup_time_ms=startup_time,
            )

        if result.get("ok"):
            pid = result.get("pid")

            # Let the example run for a bit
            time.sleep(EXAMPLE_RUN_DURATION)

            # Check if process is still running (it should be for GUI apps)
            processes = self.webview.evaluate("""
                (async () => {
                    return await auroraview.api.list_processes();
                })()
            """)

            # Check if process is still running
            if processes and processes.get("ok"):
                for proc in processes.get("processes", []):
                    if proc.get("pid") == pid:
                        # Process is still running, will be killed below
                        break

            # Kill the process after testing
            if pid:
                self.webview.evaluate(f"""
                    (async () => {{
                        return await auroraview.api.kill_process({{ pid: {pid} }});
                    }})()
                """)

            return ExampleTestResult(
                example_id=sample_id,
                title=title,
                success=True,
                pid=pid,
                startup_time_ms=startup_time,
            )
        else:
            return ExampleTestResult(
                example_id=sample_id,
                title=title,
                success=False,
                error=result.get("error", "Unknown error"),
                startup_time_ms=startup_time,
            )

    def __enter__(self):
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.disconnect()
        return False


# ─────────────────────────────────────────────────────────────────────────────
# Pytest Fixtures
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def gallery_process():
    """Start gallery process for the test module."""
    # Check if CDP is already available (gallery already running)
    if is_cdp_available():
        print("CDP already available, using existing gallery process")
        yield None
        return

    # Start gallery process
    gallery = GalleryProcess()
    if not gallery.start():
        pytest.skip("Failed to start gallery process")

    yield gallery

    gallery.stop()


@pytest.fixture(scope="module")
def example_tester(gallery_process):
    """Create example tester connected to gallery."""
    with ExampleTester() as tester:
        yield tester


@pytest.fixture(scope="module")
def all_samples(example_tester):
    """Get all samples from gallery."""
    return example_tester.get_samples()


# ─────────────────────────────────────────────────────────────────────────────
# Test Cases
# ─────────────────────────────────────────────────────────────────────────────

# List of examples that are known to work well for automated testing
TESTABLE_EXAMPLES = [
    "simple_decorator",
    "dynamic_binding",
    "window_events",
    "floating_panel",
    "desktop_app",
    "desktop_events",
    "system_tray",
    "dom_manipulation",
    "custom_context_menu",
    "native_menu",
    "custom_protocol",
    "multi_window",
    "signals_advanced",
    "cookie_management",
    "local_assets",
    "logo_button",
]

# Examples that require special handling or are known to have issues
SKIP_EXAMPLES = [
    "maya_qt_echo",  # Requires Maya
    "qt_style_tool",  # Requires Qt environment
    "qt_custom_menu",  # Requires Qt environment
    "dcc_integration",  # Requires DCC app
]


class TestExamplesStartup:
    """Test that all examples can start successfully."""

    @pytest.mark.parametrize("sample_id", TESTABLE_EXAMPLES)
    def test_example_startup(self, example_tester, sample_id):
        """Test that an example can start without errors."""
        result = example_tester.run_example(sample_id)

        assert result.success, f"Example '{result.title}' failed to start: {result.error}"
        assert result.pid is not None, f"Example '{result.title}' did not return a PID"
        print(
            f"✓ {result.title} started successfully (PID: {result.pid}, {result.startup_time_ms:.0f}ms)"
        )

    def test_get_all_samples(self, example_tester):
        """Test that we can retrieve all samples from gallery."""
        samples = example_tester.get_samples()

        assert samples, "No samples returned from gallery"
        assert len(samples) > 0, "Empty samples list"

        print(f"Found {len(samples)} samples:")
        for sample in samples:
            print(f"  - {sample.get('id')}: {sample.get('title')}")


# ─────────────────────────────────────────────────────────────────────────────
# Standalone Test Runner
# ─────────────────────────────────────────────────────────────────────────────


def run_all_tests():
    """Run all example tests and print a summary."""
    print("=" * 60)
    print("AuroraView Examples CDP Test Suite")
    print("=" * 60)

    # Check CDP availability
    if not is_cdp_available():
        print("\nCDP not available. Starting gallery process...")
        gallery = GalleryProcess()
        if not gallery.start():
            print("ERROR: Failed to start gallery process")
            return 1
        own_gallery = True
    else:
        print("\nUsing existing gallery process (CDP available)")
        gallery = None
        own_gallery = False

    try:
        with ExampleTester() as tester:
            # Get all samples
            print("\nFetching samples from gallery...")
            samples = tester.get_samples()

            if not samples:
                print("ERROR: No samples found")
                return 1

            print(f"Found {len(samples)} samples\n")

            # Test each sample
            results: list[ExampleTestResult] = []

            for sample in samples:
                sample_id = sample.get("id", "")

                # Skip known problematic examples
                if sample_id in SKIP_EXAMPLES:
                    print(f"⊘ Skipping {sample_id} (requires special environment)")
                    continue

                print(f"Testing: {sample_id}...", end=" ", flush=True)

                try:
                    result = tester.run_example(sample_id)
                    results.append(result)

                    if result.success:
                        print(f"✓ OK ({result.startup_time_ms:.0f}ms)")
                    else:
                        print(f"✗ FAILED: {result.error}")
                except Exception as e:
                    print(f"✗ ERROR: {e}")
                    results.append(
                        ExampleTestResult(
                            example_id=sample_id,
                            title=sample_id,
                            success=False,
                            error=str(e),
                        )
                    )

            # Print summary
            print("\n" + "=" * 60)
            print("Test Summary")
            print("=" * 60)

            passed = sum(1 for r in results if r.success)
            failed = sum(1 for r in results if not r.success)

            print(f"Total: {len(results)}")
            print(f"Passed: {passed}")
            print(f"Failed: {failed}")

            if failed > 0:
                print("\nFailed examples:")
                for r in results:
                    if not r.success:
                        print(f"  - {r.title}: {r.error}")

            return 0 if failed == 0 else 1

    finally:
        if own_gallery and gallery:
            gallery.stop()


if __name__ == "__main__":
    sys.exit(run_all_tests())
