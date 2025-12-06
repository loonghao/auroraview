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
    # Submodules
    "automation",
    "event_timer",
    "file_protocol",
    "logging",
    "timer_backends",
]
