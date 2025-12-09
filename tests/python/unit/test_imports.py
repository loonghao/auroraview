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

    # Test core module - skip if native module not built
    try:
        from auroraview import _core

        assert hasattr(_core, "WebView")
    except ImportError:
        # Native module not available (e.g., CI without maturin build)
        import pytest

        pytest.skip("Native _core module not available")


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
            error_str = str(e).lower()
            # Skip known optional dependencies and platform-specific errors
            skip_patterns = [
                "qtpy",
                "pyside",
                "windll",  # Windows-only ctypes attribute
                "win32",  # Windows-only modules
            ]
            if not any(pattern in error_str for pattern in skip_patterns):
                errors.append(f"{modname}: {e}")
        except AttributeError as e:
            error_str = str(e).lower()
            # Skip Windows-only ctypes.windll AttributeError
            if "windll" not in error_str:
                errors.append(f"{modname}: {e}")

    # Report all errors at once
    if errors:
        pytest.fail("Failed to import modules:\n" + "\n".join(errors))


def test_webview_module():
    """Test webview module imports."""
    from auroraview.core import webview

    assert hasattr(webview, "WebView")


def test_bridge_module():
    """Test bridge module imports."""
    from auroraview.integration import bridge

    assert hasattr(bridge, "Bridge")


def test_event_timer_module():
    """Test event_timer module imports."""
    from auroraview.utils import event_timer

    assert hasattr(event_timer, "EventTimer")


def test_framework_module():
    """Test framework module imports."""
    from auroraview.integration import framework

    assert hasattr(framework, "AuroraView")


@pytest.mark.skipif(
    "qtpy" not in sys.modules and "PySide6" not in sys.modules,
    reason="Qt dependencies not available",
)
def test_qt_integration_module():
    """Test Qt integration module imports (if Qt is available)."""
    from auroraview.integration import qt

    assert hasattr(qt, "QtWebView")


def test_testing_module():
    """Test testing framework module imports."""
    from auroraview import testing

    # New headless testing API
    assert hasattr(testing, "HeadlessWebView")
    assert hasattr(testing, "HeadlessOptions")
    assert hasattr(testing, "headless_webview")
    assert hasattr(testing, "DomAssertions")

    # AuroraTest submodule
    assert hasattr(testing, "auroratest")
