# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Mixin classes for modular functionality.

This module provides Mixin classes that implement different aspects of
WebView functionality. These are combined by the main WebView class.

Currently exposed mixins (pure capability bundles, no shared-state coupling):

    WebViewWindowMixin:    Window control methods (move, resize, minimize, ...)
    WebViewContentMixin:   Content loading methods (load_url, load_html, ...)
    WebViewJSMixin:        JavaScript interaction methods (eval_js, ...)
    WebViewEventMixin:     Event system methods (emit, on, register_callback, ...)
    WebViewApiMixin:       API binding methods (bind_call, bind_api, ...)
    WebViewDOMMixin:       DOM manipulation methods (dom, dom_all, ...)
    WebViewTelemetryMixin: Telemetry / observability hooks

Note:
    The earlier Lifecycle / State / Commands / Channels / Factory / Bridge
    mixins were inlined directly into ``WebView`` because they coupled too
    tightly to ``_core`` / ``_async_core`` private state and to the concrete
    class itself (factory return-types, lifecycle close paths, etc.). See the
    "Mixin layout note" comment in ``auroraview/core/webview.py`` for the
    full rationale.
"""

from auroraview.core.mixins.api import WebViewApiMixin
from auroraview.core.mixins.content import WebViewContentMixin
from auroraview.core.mixins.dom import WebViewDOMMixin
from auroraview.core.mixins.events import WebViewEventMixin
from auroraview.core.mixins.javascript import WebViewJSMixin
from auroraview.core.mixins.telemetry import WebViewTelemetryMixin
from auroraview.core.mixins.window import WebViewWindowMixin

# ``__all__`` mirrors the MRO order used by ``WebView`` in ``webview.py``
# (Window -> Content -> JS -> Event -> Api -> DOM -> Telemetry). Keeping
# the two in sync removes a layer of indirection for readers comparing
# this module against the class definition.
__all__ = [
    "WebViewWindowMixin",
    "WebViewContentMixin",
    "WebViewJSMixin",
    "WebViewEventMixin",
    "WebViewApiMixin",
    "WebViewDOMMixin",
    "WebViewTelemetryMixin",
]
