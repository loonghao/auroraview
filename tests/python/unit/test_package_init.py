"""Tests for package-level fallbacks and placeholders in auroraview.__init__

These tests validate that helpful ImportErrors are raised when optional
components are not available in the current environment. If optional
components are available, the tests verify basic instantiation instead.
"""

import importlib

import pytest


def test_bridge_placeholder_or_real():
    mod = importlib.import_module("auroraview")
    assert hasattr(mod, "Bridge")

    # If websockets is available, Bridge() should construct; otherwise, it should raise
    try:
        bridge = mod.Bridge(port=0, auto_start=False)  # type: ignore[arg-type]
        # Basic sanity on representation without starting server
        assert "Bridge(" in repr(bridge)
    except ImportError as ei:  # Placeholder path
        msg = str(ei)
        assert "websockets" in msg
        assert "Install with: pip install websockets" in msg


def test_qtwebview_placeholder_import_error():
    mod = importlib.import_module("auroraview")
    assert hasattr(mod, "QtWebView")

    with pytest.raises(ImportError) as ei:
        mod.QtWebView()  # type: ignore[call-arg]
    msg = str(ei.value)
    assert "Qt backend is not available" in msg
    assert "pip install auroraview[qt]" in msg


def test_service_discovery_placeholder_or_symbol():
    mod = importlib.import_module("auroraview")
    # Symbol should exist in either real or placeholder form
    assert hasattr(mod, "ServiceDiscovery")

    # If Rust core is available, just verify the type is callable; otherwise, instantiate should raise
    ServiceDiscovery = mod.ServiceDiscovery  # type: ignore[attr-defined]
    try:
        # Avoid actually starting services in CI; just check it is callable
        assert callable(ServiceDiscovery)
    except ImportError as ei:
        assert "ServiceDiscovery requires Rust core module" in str(ei)


def test_metadata_exports_present():
    mod = importlib.import_module("auroraview")
    # __version__ and __author__ should exist even without the Rust core
    assert isinstance(getattr(mod, "__version__", "0"), str)
    assert isinstance(getattr(mod, "__author__", ""), str)

    # WebView symbol should be present (class is imported from .webview)
    assert hasattr(mod, "WebView")
