# -*- coding: utf-8 -*-
"""Test decorators for AuroraView testing framework.

This module provides decorators to simplify test setup and conditional skipping
based on environment requirements.

Example:
    >>> from auroraview.testing.decorators import requires_qt, requires_cdp, slow_test
    >>>
    >>> @requires_qt
    >>> def test_qt_webview():
    ...     # This test only runs if Qt is available
    ...     pass
    >>>
    >>> @requires_cdp
    >>> def test_cdp_connection():
    ...     # This test only runs if CDP endpoint is available
    ...     pass
    >>>
    >>> @slow_test
    >>> def test_performance():
    ...     # This test is marked as slow
    ...     pass
"""

from __future__ import annotations

import os
import sys
from typing import Any, Callable, Optional, TypeVar

import pytest

F = TypeVar("F", bound=Callable[..., Any])


def _check_qt_available() -> bool:
    """Check if Qt (PySide6 or PySide2) is available."""
    try:
        import PySide6  # noqa: F401

        return True
    except ImportError:
        pass

    try:
        import PySide2  # noqa: F401

        return True
    except ImportError:
        pass

    return False


def _check_cdp_available(url: str = "http://127.0.0.1:9222") -> bool:
    """Check if CDP endpoint is available."""
    import urllib.error
    import urllib.request

    try:
        req = urllib.request.urlopen(f"{url}/json/version", timeout=2)
        req.close()
        return True
    except (urllib.error.URLError, OSError):
        return False


def _check_gallery_available() -> bool:
    """Check if packed gallery is available."""
    from pathlib import Path

    gallery_path = Path(__file__).parent.parent.parent.parent.parent / "gallery" / "pack-output"
    exe_name = "auroraview-gallery.exe" if sys.platform == "win32" else "auroraview-gallery"
    return (gallery_path / exe_name).exists()


def _check_playwright_available() -> bool:
    """Check if Playwright is available."""
    try:
        import playwright  # noqa: F401

        return True
    except ImportError:
        return False


def _check_webview2_available() -> bool:
    """Check if WebView2 runtime is available (Windows only)."""
    if sys.platform != "win32":
        return False

    try:
        import winreg

        # Check for WebView2 runtime in registry
        key_paths = [
            r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
            r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        ]

        for key_path in key_paths:
            try:
                key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, key_path)
                winreg.CloseKey(key)
                return True
            except WindowsError:
                continue

        return False
    except ImportError:
        return False


# ─────────────────────────────────────────────────────────────────────────────
# Conditional Skip Decorators
# ─────────────────────────────────────────────────────────────────────────────


def requires_qt(func: F) -> F:
    """Skip test if Qt (PySide6/PySide2) is not available.

    Example:
        >>> @requires_qt
        >>> def test_qt_webview():
        ...     from auroraview.integration.qt import QtWebView
        ...     # Test Qt functionality
    """
    return pytest.mark.skipif(
        not _check_qt_available(), reason="Qt (PySide6/PySide2) not available"
    )(func)


def requires_cdp(url: str = "http://127.0.0.1:9222") -> Callable[[F], F]:
    """Skip test if CDP endpoint is not available.

    Args:
        url: CDP endpoint URL to check

    Example:
        >>> @requires_cdp()
        >>> def test_cdp_connection():
        ...     # Test CDP functionality
        ...
        >>> @requires_cdp("http://localhost:9333")
        >>> def test_custom_cdp():
        ...     # Test with custom CDP port
    """

    def decorator(func: F) -> F:
        return pytest.mark.skipif(
            not _check_cdp_available(url), reason=f"CDP endpoint not available at {url}"
        )(func)

    return decorator


def requires_gallery(func: F) -> F:
    """Skip test if packed gallery is not available.

    Example:
        >>> @requires_gallery
        >>> def test_gallery_api():
        ...     # Test gallery functionality
    """
    return pytest.mark.skipif(
        not _check_gallery_available(), reason="Packed gallery not available"
    )(func)


def requires_playwright(func: F) -> F:
    """Skip test if Playwright is not available.

    Example:
        >>> @requires_playwright
        >>> def test_playwright_browser():
        ...     from playwright.sync_api import sync_playwright
        ...     # Test Playwright functionality
    """
    return pytest.mark.skipif(
        not _check_playwright_available(),
        reason="Playwright not available (pip install playwright)",
    )(func)


def requires_webview2(func: F) -> F:
    """Skip test if WebView2 runtime is not available (Windows only).

    Example:
        >>> @requires_webview2
        >>> def test_webview2_features():
        ...     # Test WebView2-specific functionality
    """
    return pytest.mark.skipif(
        not _check_webview2_available(), reason="WebView2 runtime not available"
    )(func)


def requires_windows(func: F) -> F:
    """Skip test if not running on Windows.

    Example:
        >>> @requires_windows
        >>> def test_windows_specific():
        ...     # Test Windows-specific functionality
    """
    return pytest.mark.skipif(sys.platform != "win32", reason="Windows only test")(func)


def requires_linux(func: F) -> F:
    """Skip test if not running on Linux.

    Example:
        >>> @requires_linux
        >>> def test_linux_specific():
        ...     # Test Linux-specific functionality
    """
    return pytest.mark.skipif(sys.platform != "linux", reason="Linux only test")(func)


def requires_macos(func: F) -> F:
    """Skip test if not running on macOS.

    Example:
        >>> @requires_macos
        >>> def test_macos_specific():
        ...     # Test macOS-specific functionality
    """
    return pytest.mark.skipif(sys.platform != "darwin", reason="macOS only test")(func)


def requires_env(var_name: str, expected_value: Optional[str] = None) -> Callable[[F], F]:
    """Skip test if environment variable is not set or doesn't match expected value.

    Args:
        var_name: Environment variable name
        expected_value: Expected value (if None, just checks existence)

    Example:
        >>> @requires_env("CI")
        >>> def test_ci_only():
        ...     # Only run in CI environment
        ...
        >>> @requires_env("TEST_MODE", "integration")
        >>> def test_integration_mode():
        ...     # Only run when TEST_MODE=integration
    """

    def decorator(func: F) -> F:
        actual = os.environ.get(var_name)
        if expected_value is None:
            skip = actual is None
            reason = f"Environment variable {var_name} not set"
        else:
            skip = actual != expected_value
            reason = f"Environment variable {var_name}={expected_value} required (got {actual})"

        return pytest.mark.skipif(skip, reason=reason)(func)

    return decorator


# ─────────────────────────────────────────────────────────────────────────────
# Test Category Markers
# ─────────────────────────────────────────────────────────────────────────────


def slow_test(func: F) -> F:
    """Mark test as slow (may be skipped in quick test runs).

    Example:
        >>> @slow_test
        >>> def test_performance_benchmark():
        ...     # Long-running performance test
    """
    return pytest.mark.slow(func)


def integration_test(func: F) -> F:
    """Mark test as integration test.

    Example:
        >>> @integration_test
        >>> def test_full_workflow():
        ...     # End-to-end integration test
    """
    return pytest.mark.integration(func)


def unit_test(func: F) -> F:
    """Mark test as unit test.

    Example:
        >>> @unit_test
        >>> def test_single_function():
        ...     # Isolated unit test
    """
    return pytest.mark.unit(func)


def smoke_test(func: F) -> F:
    """Mark test as smoke test (quick sanity check).

    Example:
        >>> @smoke_test
        >>> def test_basic_import():
        ...     import auroraview
        ...     assert auroraview is not None
    """
    return pytest.mark.smoke(func)


def flaky_test(reruns: int = 3, reruns_delay: int = 1) -> Callable[[F], F]:
    """Mark test as flaky with automatic reruns.

    Args:
        reruns: Number of times to retry on failure
        reruns_delay: Delay between retries in seconds

    Example:
        >>> @flaky_test(reruns=3)
        >>> def test_network_dependent():
        ...     # Test that may fail due to network issues
    """

    def decorator(func: F) -> F:
        # Use pytest-rerunfailures if available
        marked = pytest.mark.flaky(reruns=reruns, reruns_delay=reruns_delay)(func)
        return marked

    return decorator


# ─────────────────────────────────────────────────────────────────────────────
# Test Setup Decorators
# ─────────────────────────────────────────────────────────────────────────────


def with_timeout(seconds: int) -> Callable[[F], F]:
    """Set timeout for a test.

    Args:
        seconds: Maximum time allowed for test execution

    Example:
        >>> @with_timeout(30)
        >>> def test_long_operation():
        ...     # Test must complete within 30 seconds
    """

    def decorator(func: F) -> F:
        return pytest.mark.timeout(seconds)(func)

    return decorator


def parametrize_examples(example_ids: list) -> Callable[[F], F]:
    """Parametrize test with example IDs.

    Args:
        example_ids: List of example IDs to test

    Example:
        >>> @parametrize_examples(["simple_decorator", "dynamic_binding"])
        >>> def test_example_startup(example_id):
        ...     # Test runs for each example
    """

    def decorator(func: F) -> F:
        return pytest.mark.parametrize("example_id", example_ids)(func)

    return decorator


def serial_test(func: F) -> F:
    """Mark test to run serially (not in parallel).

    Requires pytest-xdist with --dist=loadfile or serial_test plugin.

    Example:
        >>> @serial_test
        >>> def test_global_state():
        ...     # Test that modifies global state
    """
    try:
        from pytest import mark

        return mark.serial(func)
    except AttributeError:
        # Fallback if serial marker not available
        return func


# ─────────────────────────────────────────────────────────────────────────────
# Utility Functions
# ─────────────────────────────────────────────────────────────────────────────


def skip_if(condition: bool, reason: str) -> Callable[[F], F]:
    """Skip test if condition is True.

    Args:
        condition: Condition to check
        reason: Reason for skipping

    Example:
        >>> @skip_if(sys.version_info < (3, 9), "Requires Python 3.9+")
        >>> def test_new_feature():
        ...     # Test using Python 3.9+ features
    """

    def decorator(func: F) -> F:
        return pytest.mark.skipif(condition, reason=reason)(func)

    return decorator


def xfail_if(condition: bool, reason: str) -> Callable[[F], F]:
    """Mark test as expected failure if condition is True.

    Args:
        condition: Condition to check
        reason: Reason for expected failure

    Example:
        >>> @xfail_if(sys.platform == "win32", "Known issue on Windows")
        >>> def test_unix_specific():
        ...     # Test that's expected to fail on Windows
    """

    def decorator(func: F) -> F:
        return pytest.mark.xfail(condition, reason=reason)(func)

    return decorator


__all__ = [
    # Skip decorators
    "requires_qt",
    "requires_cdp",
    "requires_gallery",
    "requires_playwright",
    "requires_webview2",
    "requires_windows",
    "requires_linux",
    "requires_macos",
    "requires_env",
    # Category markers
    "slow_test",
    "integration_test",
    "unit_test",
    "smoke_test",
    "flaky_test",
    # Setup decorators
    "with_timeout",
    "parametrize_examples",
    "serial_test",
    # Utility functions
    "skip_if",
    "xfail_if",
]
