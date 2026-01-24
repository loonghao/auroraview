# -*- coding: utf-8 -*-
"""Qt event processor for AuroraView.

This module provides the QtEventProcessor class that handles event processing
for Qt-integrated WebViews by processing both Qt events and WebView message queue.
"""

import logging
import os

try:
    from qtpy.QtCore import QCoreApplication
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

from auroraview.core.webview import WebView

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)


class QtEventProcessor:
    """Event processor for Qt integration (strategy pattern).

    This class handles event processing for Qt-integrated WebViews by:
    1. Processing Qt events (QCoreApplication.processEvents())
    2. Processing WebView message queue (webview._core.process_events())

    This ensures both Qt and WebView events are handled correctly.

    Architecture:
        WebView (base class)
            ↓ uses
        QtEventProcessor (strategy)
            ↓ processes
        Qt events + WebView events

    Example:
        >>> webview = WebView()
        >>> processor = QtEventProcessor(webview)
        >>> webview.set_event_processor(processor)
        >>>
        >>> # Now emit() and eval_js() automatically process Qt + WebView events
        >>> webview.emit("my_event", {"data": 123})
    """

    def __init__(self, webview: WebView):
        """Initialize Qt event processor.

        Args:
            webview: WebView instance to process events for
        """
        self._webview = webview
        self._process_count = 0

    def process(self) -> None:
        """Process Qt events and WebView message queue.

        This is the core method called by WebView._auto_process_events().

        Following main branch design:
        1. Process Qt events first (QCoreApplication.processEvents())
        2. Process AuroraView message queue (WebView.process_events())

        Without step 2, JavaScript code sent via eval_js() or emit() will
        remain in the message queue and never execute, causing Promises to hang.
        """
        self._process_count += 1

        try:
            # Step 1: Process Qt events first
            QCoreApplication.processEvents()

            # Step 2: Process AuroraView message queue
            # This is CRITICAL - without this, eval_js/emit messages stay in queue
            self._webview.process_events()
        except Exception as e:  # pragma: no cover - best-effort only
            if _VERBOSE_LOGGING:
                logger.debug(f"QtEventProcessor: Event processing failed: {e}")


__all__ = ["QtEventProcessor"]
