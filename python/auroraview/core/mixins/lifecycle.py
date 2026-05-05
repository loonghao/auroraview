# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Lifecycle Mixin.

This module provides lifecycle methods for the WebView class.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from typing import Callable

logger = logging.getLogger(__name__)


class WebViewLifecycleMixin:
    """Mixin providing lifecycle methods.

    Provides methods for controlling the WebView lifecycle:
    - show: Show the WebView window (smart mode)
    - show_async: Show window in non-blocking mode
    - show_blocking: Show window and block until closed
    - wait: Wait for window to close
    - close: Close the WebView
    """

    # Type hints for attributes from main class
    _core: Any
    _ready_events: Any
    _event_thread: Optional[Any]
    _is_blocking: bool
    _title: str
    _url: Optional[str]
    _html: Optional[str]

    def show(self, *, wait: Optional[bool] = None) -> None:
        """Show the WebView window (smart mode).

        Automatically detects standalone/embedded/packed mode and chooses the best behavior:
        - Packed mode: Runs as headless API server (no window, JSON-RPC via stdin/stdout)
        - Standalone window: Blocks until closed (unless wait=False)
        - Embedded window: Non-blocking, auto-starts timer if available

        Args:
            wait: Whether to wait for window to close
                - None: Auto-detect (standalone=True, embedded=False)
                - True: Block until window closes
                - False: Return immediately (background thread)
        """
        # Check for packed mode first - transparent to developers
        from .packed import is_packed_mode, run_api_server

        if is_packed_mode():
            logger.info("Packed mode detected: running as API server")
            run_api_server(self)
            return

        # Detect mode
        is_embedded = self._core is not None and hasattr(self._core, "_is_embedded") and self._core._is_embedded

        if wait is None:
            wait = not is_embedded  # Standalone=blocking, Embedded=non-blocking

        if wait:
            self.show_blocking()
        else:
            self.show_async()

    def show_async(self) -> None:
        """Show window in non-blocking mode (for embedded DCC)."""
        self._show_non_blocking()
        logger.info("[show_async] WebView shown (non-blocking)")

    def _show_non_blocking(self) -> None:
        """Internal: show without blocking (embedded mode)."""
        if self._core is None:
            logger.error("Cannot show: core is None")
            return

        # Show the window (non-blocking)
        if hasattr(self._core, "show"):
            self._core.show()

        # Auto-start event processor if available
        if hasattr(self, "_event_processor") and self._event_processor:
            self._event_processor.start()

        logger.info("[show] WebView shown (non-blocking)")

    def show_blocking(self) -> None:
        """Show window and block until closed (standalone mode)."""
        if self._core is None:
            logger.error("Cannot show: core is None")
            return

        # Load content before showing
        if self._url:
            self._core.load_url(self._url)
        elif self._html:
            self._core.load_html(self._html)

        # Show and block
        if hasattr(self._core, "show"):
            self._core.show()

        self._is_blocking = True
        logger.info("[show_blocking] WebView shown (blocking)")

        # In standalone mode, main thread is blocked by GUI
        # No need to manually wait (the GUI event loop blocks)

    def wait(self, timeout: Optional[float] = None) -> bool:
        """Wait for window to close.

        Args:
            timeout: Max seconds to wait (None = forever)

        Returns:
            True if window closed, False if timeout
        """
        if self._event_thread and self._event_thread.is_alive():
            self._event_thread.join(timeout)
            return not self._event_thread.is_alive()
        return True

    def close(self) -> None:
        """Close the WebView."""
        logger.info("[close] Closing WebView")

        # Stop event processor
        if hasattr(self, "_event_processor") and self._event_processor:
            self._event_processor.stop()

        # Close core
        if self._core and hasattr(self._core, "close"):
            try:
                self._core.close()
            except Exception as e:
                logger.warning(f"[close] Error closing core: {e}")

        # Wait for event thread to finish
        if self._event_thread and self._event_thread.is_alive():
            self._event_thread.join(timeout=5.0)

        logger.info("[close] WebView closed")
