"""Standalone test for context_menu parameter.

This test can be run directly without pytest to verify the context_menu
parameter is working correctly in the Rust core.

Usage:
    python tests/test_context_menu_standalone.py

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import os
import sys

import pytest

# Skip in CI (requires display environment)
pytestmark = pytest.mark.skipif(
    os.environ.get("CI") == "true",
    reason="WebView tests require display environment, skipped in CI",
)


def test_core_webview_context_menu():
    """Test that _core.WebView accepts context_menu parameter."""
    from auroraview._core import WebView as _CoreWebView

    print("\n" + "=" * 80)
    print("Testing _core.WebView context_menu parameter")
    print("=" * 80)

    # Test 1: context_menu=False
    print("\n[TEST 1] Creating WebView with context_menu=False...")
    try:
        webview1 = _CoreWebView(
            title="Test context_menu=False",
            width=800,
            height=600,
            context_menu=False,
        )
        print("✅ SUCCESS: WebView created with context_menu=False")
        del webview1
    except TypeError as e:
        pytest.fail(f"Failed to create WebView with context_menu=False: {e}")

    # Test 2: context_menu=True
    print("\n[TEST 2] Creating WebView with context_menu=True...")
    try:
        webview2 = _CoreWebView(
            title="Test context_menu=True",
            width=800,
            height=600,
            context_menu=True,
        )
        print("✅ SUCCESS: WebView created with context_menu=True")
        del webview2
    except TypeError as e:
        pytest.fail(f"Failed to create WebView with context_menu=True: {e}")

    # Test 3: Default (should be context_menu=True)
    print("\n[TEST 3] Creating WebView without context_menu parameter (default)...")
    try:
        webview3 = _CoreWebView(
            title="Test default",
            width=800,
            height=600,
        )
        print("✅ SUCCESS: WebView created with default context_menu")
        del webview3
    except TypeError as e:
        pytest.fail(f"Failed to create WebView with default context_menu: {e}")

    # Test 4: All parameters
    print("\n[TEST 4] Creating WebView with all parameters...")
    try:
        webview4 = _CoreWebView(
            title="Test all params",
            width=1024,
            height=768,
            url=None,
            html="<h1>Test</h1>",
            dev_tools=True,
            context_menu=False,
            resizable=True,
            decorations=True,
            parent_hwnd=None,
            parent_mode=None,
        )
        print("✅ SUCCESS: WebView created with all parameters")
        del webview4
    except TypeError as e:
        pytest.fail(f"Failed to create WebView with all parameters: {e}")

    print("\n" + "=" * 80)
    print("✅ ALL TESTS PASSED!")
    print("=" * 80)


def test_high_level_api():
    """Test that high-level WebView API accepts context_menu parameter."""
    from auroraview import WebView

    print("\n" + "=" * 80)
    print("Testing high-level WebView API")
    print("=" * 80)

    # Test 1: WebView.__init__ with context_menu=False
    print("\n[TEST 1] Creating WebView with context_menu=False...")
    try:
        webview1 = WebView(
            title="High-level test",
            width=800,
            height=600,
            context_menu=False,
        )
        print("✅ SUCCESS: WebView created with context_menu=False")
        webview1.close()
    except TypeError as e:
        pytest.fail(f"Failed to create WebView with context_menu=False: {e}")

    # Test 2: WebView.create with context_menu=False
    print("\n[TEST 2] Creating WebView with WebView.create()...")
    try:
        webview2 = WebView.create(
            title="High-level create test",
            width=800,
            height=600,
            context_menu=False,
        )
        print("✅ SUCCESS: WebView.create() works with context_menu=False")
        webview2.close()
    except TypeError as e:
        pytest.fail(f"Failed WebView.create() with context_menu=False: {e}")

    print("\n" + "=" * 80)
    print("✅ ALL HIGH-LEVEL API TESTS PASSED!")
    print("=" * 80)


if __name__ == "__main__":
    success = True

    # Run core tests
    if not test_core_webview_context_menu():
        success = False

    # Run high-level API tests
    if not test_high_level_api():
        success = False

    # Exit with appropriate code
    sys.exit(0 if success else 1)
