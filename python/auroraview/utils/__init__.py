# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView Utilities Module.

This module contains utility functions and classes:
- EventTimer: Timer for periodic event processing
- TimerBackends: Pluggable timer backend system
- FileProtocol: File URL and local asset utilities
- Automation: Browser automation utilities
- Logging: Configurable logging for DCC environments

Example:
    >>> from auroraview.utils import EventTimer, path_to_file_url
    >>> timer = EventTimer(webview, interval=100)
    >>> url = path_to_file_url("/path/to/file.html")
"""

from __future__ import annotations

from .automation import (
    Automation,
    BrowserBackend,
    LocalWebViewBackend,
    SteelBrowserBackend,
)
from .event_timer import EventTimer
from .file_protocol import (
    file_url_to_auroraview_url,
    get_auroraview_entry_url,
    path_to_auroraview_url,
    path_to_file_url,
    prepare_html_with_local_assets,
)
from .logging import configure_logging, get_logger, is_verbose_enabled
from .thread_dispatcher import (
    ThreadDispatcherBackend,
    defer_to_main_thread,
    ensure_main_thread,
    get_dispatcher_backend,
    is_main_thread,
    list_dispatcher_backends,
    register_dispatcher_backend,
    run_on_main_thread,
    run_on_main_thread_sync,
    unregister_dispatcher_backend,
)
from .timer_backends import (
    QtTimerBackend,
    ThreadTimerBackend,
    TimerBackend,
    get_available_backend,
    list_registered_backends,
    register_timer_backend,
)

# Import submodules for attribute access
from . import automation as automation
from . import event_timer as event_timer
from . import file_protocol as file_protocol
from . import logging as logging
from . import thread_dispatcher as thread_dispatcher
from . import timer_backends as timer_backends

__all__ = [
    # Event Timer
    "EventTimer",
    # Timer Backends
    "TimerBackend",
    "ThreadTimerBackend",
    "QtTimerBackend",
    "get_available_backend",
    "register_timer_backend",
    "list_registered_backends",
    # File Protocol
    "path_to_file_url",
    "path_to_auroraview_url",
    "file_url_to_auroraview_url",
    "get_auroraview_entry_url",
    "prepare_html_with_local_assets",
    # Automation
    "Automation",
    "BrowserBackend",
    "LocalWebViewBackend",
    "SteelBrowserBackend",
    # Logging
    "configure_logging",
    "get_logger",
    "is_verbose_enabled",
    # Thread Dispatcher
    "ThreadDispatcherBackend",
    "register_dispatcher_backend",
    "unregister_dispatcher_backend",
    "get_dispatcher_backend",
    "list_dispatcher_backends",
    "run_on_main_thread",
    "run_on_main_thread_sync",
    "is_main_thread",
    "ensure_main_thread",
    "defer_to_main_thread",
    # Submodules
    "automation",
    "event_timer",
    "file_protocol",
    "logging",
    "thread_dispatcher",
    "timer_backends",
]
