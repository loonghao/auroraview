"""Import tests for AuroraView package.

This test ensures all modules can be imported successfully,
providing broad coverage of the codebase.
"""

from __future__ import annotations

import importlib
import pkgutil
import sys

import pytest


def test_auroraview_core_imports():
    """Test that core AuroraView modules can be imported."""
    import auroraview

    # Test main package
    assert hasattr(auroraview, "__version__")
    assert hasattr(auroraview, "WebView")
    assert hasattr(auroraview, "AuroraView")

    # Test core module
    from auroraview import _core

    assert hasattr(_core, "WebView")


def test_auroraview_all_submodules():
    """Test importing all AuroraView submodules recursively."""
    import auroraview

    prefix = f"{auroraview.__name__}."
    errors = []

    # Walk through all submodules
    for _importer, modname, _ispkg in pkgutil.walk_packages(
        path=auroraview.__path__,
        prefix=prefix,
    ):
        # Skip private modules and version module
        if "._" in modname or modname.endswith("._version"):
            continue

        try:
            importlib.import_module(modname)
        except ImportError as e:
            # Some modules may have optional dependencies (Qt, etc.)
            # Only fail if it's not a known optional dependency
            if "qtpy" not in str(e).lower() and "pyside" not in str(e).lower():
                errors.append(f"{modname}: {e}")

    # Report all errors at once
    if errors:
        pytest.fail("Failed to import modules:\n" + "\n".join(errors))


def test_webview_module():
    """Test webview module imports."""
    from auroraview import webview

    assert hasattr(webview, "WebView")


def test_bridge_module():
    """Test bridge module imports."""
    from auroraview import bridge

    assert hasattr(bridge, "Bridge")


def test_event_timer_module():
    """Test event_timer module imports."""
    from auroraview import event_timer

    assert hasattr(event_timer, "EventTimer")


def test_framework_module():
    """Test framework module imports."""
    from auroraview import framework

    assert hasattr(framework, "AuroraView")


@pytest.mark.skipif(
    "qtpy" not in sys.modules and "PySide6" not in sys.modules,
    reason="Qt dependencies not available",
)
def test_qt_integration_module():
    """Test Qt integration module imports (if Qt is available)."""
    from auroraview import qt_integration

    assert hasattr(qt_integration, "QtWebView")


def test_testing_module():
    """Test testing framework module imports."""
    from auroraview import testing

    assert hasattr(testing, "WebViewBot")
    assert hasattr(testing, "EventRecord")
