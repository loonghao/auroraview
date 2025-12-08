"""Async event processor for WebView in DCC applications.

This module provides an asynchronous event processor that handles IPC
messages in a background thread, reducing main thread blocking.

Key features:
1. Background thread for message processing
2. Batch message handling to reduce GIL contention
3. Non-blocking callbacks using QTimer.singleShot

This is especially useful for Houdini and other DCCs where the main
thread is heavily loaded with cooking/rendering operations.

Example:
    >>> from auroraview import WebView
    >>> from auroraview.utils.async_event_processor import AsyncEventProcessor
    >>>
    >>> webview = WebView.create(parent=hwnd, mode="container")
    >>> processor = AsyncEventProcessor(webview, batch_size=10)
    >>> processor.start()
"""

import logging
import queue
import threading
import time
from typing import Any, Callable, Optional

logger = logging.getLogger(__name__)


class AsyncEventProcessor:
    """Async event processor for WebView IPC messages.

    Processes IPC messages in a background thread to avoid blocking
    the main DCC thread. Callbacks are dispatched to the main thread
    using Qt's signal/slot mechanism.

    Args:
        webview: WebView instance to process events for
        interval_ms: Polling interval in milliseconds (default: 8ms)
        batch_size: Max messages to process per batch (default: 10)
        use_qt_dispatch: Use Qt for main thread dispatch (default: True)
    """

    def __init__(
        self,
        webview,
        interval_ms: int = 8,
        batch_size: int = 10,
        use_qt_dispatch: bool = True,
    ):
        self._webview = webview
        self._interval_ms = interval_ms
        self._batch_size = batch_size
        self._use_qt_dispatch = use_qt_dispatch

        self._running = False
        self._thread: Optional[threading.Thread] = None
        self._callback_queue: queue.Queue = queue.Queue()

        # Statistics
        self._messages_processed = 0
        self._batches_processed = 0

        logger.debug(
            f"AsyncEventProcessor created: interval={interval_ms}ms, batch_size={batch_size}"
        )

    def start(self) -> None:
        """Start the async event processor."""
        if self._running:
            logger.warning("AsyncEventProcessor already running")
            return

        self._running = True
        self._thread = threading.Thread(
            target=self._process_loop,
            name="AsyncEventProcessor",
            daemon=True,
        )
        self._thread.start()

        # Start Qt dispatch timer if using Qt
        if self._use_qt_dispatch:
            self._start_qt_dispatch()

        logger.info("AsyncEventProcessor started")

    def stop(self) -> None:
        """Stop the async event processor."""
        if not self._running:
            return

        self._running = False

        if self._thread:
            self._thread.join(timeout=1.0)
            self._thread = None

        logger.info(
            f"AsyncEventProcessor stopped: "
            f"processed {self._messages_processed} messages in "
            f"{self._batches_processed} batches"
        )

    def _process_loop(self) -> None:
        """Background thread loop for processing messages."""
        interval_sec = self._interval_ms / 1000.0

        while self._running:
            try:
                # Get WebView core and process events
                core = getattr(self._webview, "_core", None)
                if core is None:
                    time.sleep(interval_sec)
                    continue

                # Process IPC messages only (not full event loop)
                if hasattr(core, "process_events_ipc_only"):
                    should_close = core.process_events_ipc_only()
                else:
                    should_close = core.process_events()

                if should_close:
                    logger.info("Close signal received from WebView")
                    self._running = False
                    break

                self._batches_processed += 1

            except Exception as e:
                logger.error(f"Error in async event processing: {e}")

            time.sleep(interval_sec)

    def _start_qt_dispatch(self) -> None:
        """Start Qt timer for dispatching callbacks to main thread."""
        try:
            from qtpy.QtCore import QTimer

            self._qt_timer = QTimer()
            self._qt_timer.timeout.connect(self._dispatch_callbacks)
            self._qt_timer.start(self._interval_ms)
            logger.debug("Qt dispatch timer started")
        except ImportError:
            logger.warning("Qt not available, callbacks will run in background thread")
            self._use_qt_dispatch = False

    def _dispatch_callbacks(self) -> None:
        """Dispatch queued callbacks to main thread (called by Qt timer)."""
        dispatched = 0
        while not self._callback_queue.empty() and dispatched < self._batch_size:
            try:
                callback, args, kwargs = self._callback_queue.get_nowait()
                callback(*args, **kwargs)
                dispatched += 1
            except queue.Empty:
                break
            except Exception as e:
                logger.error(f"Error dispatching callback: {e}")

    def queue_callback(
        self,
        callback: Callable,
        *args: Any,
        **kwargs: Any,
    ) -> None:
        """Queue a callback for execution on the main thread.

        Args:
            callback: Function to call
            *args: Positional arguments
            **kwargs: Keyword arguments
        """
        self._callback_queue.put((callback, args, kwargs))

    @property
    def is_running(self) -> bool:
        """Check if processor is running."""
        return self._running

    @property
    def stats(self) -> dict:
        """Get processing statistics."""
        return {
            "messages_processed": self._messages_processed,
            "batches_processed": self._batches_processed,
            "queue_size": self._callback_queue.qsize(),
        }


__all__ = ["AsyncEventProcessor"]
