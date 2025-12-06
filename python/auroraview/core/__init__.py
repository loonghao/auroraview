# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView Core Module.

This module contains the core WebView functionality:
- WebView: The main WebView class
- Backend: Backend abstraction layer
- Settings: WebView configuration
- Cookies: Cookie management

Example:
    >>> from auroraview.core import WebView, WebViewSettings
    >>> webview = WebView(title="My App")
    >>> webview.show()
"""

from __future__ import annotations

from .backend import (
    BackendType,
    get_available_backends,
    get_backend_type,
    get_default_backend,
    is_backend_available,
    set_backend_type,
)
from .channel import Channel, ChannelManager
from .commands import CommandError, CommandErrorCode, CommandRegistry
from .cookies import Cookie
from .event_emitter import (
    EventEmitter,
    LoadEvent,
    NavigationEvent,
    WindowEvent as WindowEventData2,
    deprecated,
)
from .events import EventHandler, WindowEvent, WindowEventData
from .settings import DEFAULT_SETTINGS, WebViewSettings
from .signals import ConnectionGuard, ConnectionId, Signal, SignalRegistry, WebViewSignals
from .state import State
from .webview import WebView

# Import submodules for attribute access
from . import backend as backend
from . import channel as channel
from . import commands as commands
from . import cookies as cookies
from . import event_emitter as event_emitter
from . import events as events
from . import settings as settings
from . import state as state
from . import webview as webview

__all__ = [
    # WebView
    "WebView",
    # Backend abstraction
    "BackendType",
    "get_backend_type",
    "set_backend_type",
    "get_default_backend",
    "get_available_backends",
    "is_backend_available",
    # Settings
    "WebViewSettings",
    "DEFAULT_SETTINGS",
    # Cookie management
    "Cookie",
    # Events
    "WindowEvent",
    "WindowEventData",
    "EventHandler",
    # EventEmitter pattern
    "EventEmitter",
    "NavigationEvent",
    "LoadEvent",
    "deprecated",
    # State
    "State",
    # Signals (Qt-inspired)
    "Signal",
    "SignalRegistry",
    "ConnectionId",
    "ConnectionGuard",
    "WebViewSignals",
    # Commands
    "CommandRegistry",
    "CommandError",
    "CommandErrorCode",
    # Channels
    "Channel",
    "ChannelManager",
    # Submodules
    "backend",
    "channel",
    "commands",
    "cookies",
    "events",
    "event_emitter",
    "settings",
    "state",
    "webview",
]
