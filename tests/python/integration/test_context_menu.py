"""Test context menu configuration.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import os
import sys

import pytest

# Skip entire module if not on Windows (WebView2 is Windows-only)
if sys.platform != "win32":
    pytest.skip("WebView2 context menu tests require Windows", allow_module_level=True)

# Skip in CI (requires display environment)
if os.environ.get("CI") == "true":
    pytest.skip("WebView tests require display environment, skipped in CI", allow_module_level=True)

from auroraview import WebView
from auroraview._core import WebView as _CoreWebView


def test_context_menu_enabled_by_default():
    """Test that context menu is enabled by default."""
    webview = WebView(
        title="Test",
        width=800,
        height=600,
    )
    # WebView should be created successfully with default context_menu=True
    assert webview is not None
    webview.close()


def test_context_menu_can_be_disabled():
    """Test that context menu can be disabled."""
    webview = WebView(
        title="Test",
        width=800,
        height=600,
        context_menu=False,
    )
    # WebView should be created successfully with context_menu=False
    assert webview is not None
    webview.close()


def test_context_menu_in_create_method():
    """Test context_menu parameter in create() method."""
    webview = WebView.create(
        title="Test",
        width=800,
        height=600,
        context_menu=False,
    )
    assert webview is not None
    webview.close()


def test_context_menu_in_run_embedded():
    """Test context_menu parameter in run_embedded() method."""
    # Note: This test doesn't actually embed in a parent window
    # It just verifies the parameter is accepted
    webview = WebView.create(
        title="Test",
        width=800,
        height=600,
        context_menu=False,
        mode="owner",  # Use owner mode without actual parent
    )
    assert webview is not None
    webview.close()


@pytest.mark.qt
def test_qt_webview_context_menu():
    """Test context_menu parameter in QtWebView."""
    from qtpy.QtWidgets import QApplication

    from auroraview import QtWebView

    _ = QApplication.instance() or QApplication([])

    # Test with context menu disabled
    webview = QtWebView(
        title="Test",
        width=800,
        height=600,
        context_menu=False,
    )
    assert webview is not None
    webview.close()

    # Test with context menu enabled (default)
    webview2 = QtWebView(
        title="Test",
        width=800,
        height=600,
        context_menu=True,
    )
    assert webview2 is not None
    webview2.close()


def test_core_webview_context_menu_parameter():
    """Test that _core.WebView accepts context_menu parameter.

    This is the critical test that verifies the Rust core has been compiled
    with the context_menu parameter support.
    """
    # Test with context_menu=False
    webview = _CoreWebView(
        title="Core Test",
        width=800,
        height=600,
        context_menu=False,
    )
    assert webview is not None
    del webview  # Clean up

    # Test with context_menu=True (default)
    webview2 = _CoreWebView(
        title="Core Test 2",
        width=800,
        height=600,
        context_menu=True,
    )
    assert webview2 is not None
    del webview2  # Clean up


def test_core_webview_context_menu_default():
    """Test that _core.WebView has context_menu=True by default."""
    # Create without specifying context_menu
    webview = _CoreWebView(
        title="Core Test Default",
        width=800,
        height=600,
    )
    assert webview is not None
    del webview  # Clean up


def test_core_webview_all_parameters_with_context_menu():
    """Test _core.WebView with all parameters including context_menu."""
    webview = _CoreWebView(
        title="Full Test",
        width=1024,
        height=768,
        url=None,
        html="<h1>Test</h1>",
        dev_tools=True,
        context_menu=False,  # The key parameter we're testing
        resizable=True,
        decorations=True,
        parent_hwnd=None,
        parent_mode=None,
    )
    assert webview is not None
    del webview  # Clean up
