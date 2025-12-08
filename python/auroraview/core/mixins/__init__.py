# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Mixin classes for modular functionality.

This module provides Mixin classes that implement different aspects of
WebView functionality. These are combined by the main WebView class.

Mixins:
    WebViewWindowMixin: Window control methods (move, resize, minimize, etc.)
    WebViewContentMixin: Content loading methods (load_url, load_html, etc.)
    WebViewJSMixin: JavaScript interaction methods (eval_js, eval_js_async, etc.)
    WebViewEventMixin: Event system methods (emit, on, register_callback, etc.)
    WebViewApiMixin: API binding methods (bind_call, bind_api, etc.)
    WebViewDOMMixin: DOM manipulation methods (dom, dom_all, etc.)
"""

from auroraview.core.mixins.window import WebViewWindowMixin
from auroraview.core.mixins.content import WebViewContentMixin
from auroraview.core.mixins.javascript import WebViewJSMixin
from auroraview.core.mixins.events import WebViewEventMixin
from auroraview.core.mixins.api import WebViewApiMixin
from auroraview.core.mixins.dom import WebViewDOMMixin

__all__ = [
    "WebViewWindowMixin",
    "WebViewContentMixin",
    "WebViewJSMixin",
    "WebViewEventMixin",
    "WebViewApiMixin",
    "WebViewDOMMixin",
]
