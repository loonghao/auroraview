"""Unit tests for WebView.close() behavior.

These tests must not require the native `_core` extension module.
"""

from __future__ import annotations

import threading


def test_close_prefers_async_core_when_present():
    from auroraview.core.webview import WebView

    calls = []

    class DummyCore:
        def __init__(self, name: str):
            self._name = name

        def close(self):
            calls.append(self._name)

    # Bypass __init__ so we don't need the native module.
    webview = WebView.__new__(WebView)
    webview._core = DummyCore("core")
    webview._async_core = DummyCore("async")
    webview._async_core_lock = threading.Lock()
    webview._show_thread = None
    webview._close_requested = False

    # Ensure singleton cleanup doesn't affect other tests.
    old_registry = WebView._singleton_registry
    WebView._singleton_registry = {}
    try:
        webview.close()
    finally:
        WebView._singleton_registry = old_registry

    assert "async" in calls
