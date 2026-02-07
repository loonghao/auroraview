# -*- coding: utf-8 -*-
"""Lifecycle management mixin for QtWebView.

This module provides the LifecycleMixin class that handles WebView initialization,
show/hide logic, close events, and state reset for reuse.
"""

import logging
import os
import sys
import time
from typing import TYPE_CHECKING, Optional

try:
    from qtpy.QtCore import Qt, QTimer
    from qtpy.QtWidgets import QApplication
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

from auroraview.integration.qt._compat import (
    hide_window_for_init,
    show_window_after_init,
)

if TYPE_CHECKING:
    from qtpy.QtGui import QWindow
    from qtpy.QtWidgets import QStackedWidget, QWidget

    from auroraview.core.webview import WebView

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)


class LifecycleMixin:
    """Mixin class providing WebView lifecycle management.

    This mixin handles:
    - WebView initialization on first show
    - Anti-flicker hiding during init (Windows)
    - Close event handling and cleanup
    - State reset for widget reuse in DCC environments

    The mixin expects the host class to have:
    - self._webview: WebView instance
    - self._webview_initialized: bool flag
    - self._is_closing: bool flag
    - self._stack: QStackedWidget for page management
    - self._initial_url: Optional initial URL
    - self._initial_html: Optional initial HTML
    - self._embed_mode: Embedding mode string
    - self._create_webview_container(): method for container creation
    - self._force_container_geometry(): method for geometry sync
    - self._sync_webview2_controller_bounds(): method for bounds sync
    - self.winId(): method to get Qt widget's native handle
    - self.setAttribute(): Qt widget method
    """

    # Type hints for expected attributes
    _webview: "WebView"
    _webview_initialized: bool
    _is_closing: bool
    _stack: "QStackedWidget"
    _initial_url: Optional[str]
    _initial_html: Optional[str]
    _embed_mode: str
    _webview_container: Optional["QWidget"]  # type: ignore[name-defined]
    _webview_qwindow: Optional["QWindow"]  # type: ignore[name-defined]
    _pre_show_hidden: bool
    _show_start_time: float

    def _initialize_webview(self) -> None:
        """Initialize the WebView and load initial content.

        This is called automatically on first show. It handles:
        1. Anti-flicker hiding (on Windows)
        2. WebView creation and container setup
        3. Loading initial URL or HTML
        4. Starting the event timer

        Due to Rust WebView limitations (!Send), we must create and run
        the WebView on the main thread. We use progressive initialization
        with QApplication.processEvents() to keep the UI responsive.
        """
        self._show_start_time = time.time()
        if _VERBOSE_LOGGING:
            logger.debug("[LifecycleMixin] _initialize_webview() started with anti-flicker")

        # Step 1: Hide the window before initialization (anti-flicker)
        self._pre_show_hidden = False
        if sys.platform == "win32":
            # Ensure native window handle exists
            self.setAttribute(Qt.WA_NativeWindow, True)  # type: ignore[attr-defined]
            qt_hwnd = int(self.winId())  # type: ignore[attr-defined]
            if qt_hwnd:
                self._pre_show_hidden = hide_window_for_init(qt_hwnd)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[LifecycleMixin] Pre-show hidden applied: HWND=0x{qt_hwnd:X}")

        # Ensure QStackedWidget shows loading page
        self._stack.setCurrentIndex(0)

        # Process events to ensure the widget geometry is established
        QApplication.processEvents()

        # Step 2: Initialize WebView with progressive event processing
        self._init_webview_progressive()

    def _init_webview_progressive(self) -> None:
        """Initialize WebView on main thread with progressive event processing.

        This keeps the DCC UI responsive by processing Qt events between
        initialization steps.
        """
        start_time = getattr(self, "_show_start_time", time.time())

        # Step 1: Get the core WebView object
        core = getattr(self._webview, "_core", None)
        if core is None:
            logger.warning("[LifecycleMixin] No core WebView available, using fallback")
            self._webview.show()
            return

        # Process events to keep UI responsive
        QApplication.processEvents()

        # Step 2: Create and show the embedded WebView
        embed_mode = getattr(self, "_embed_mode", None)
        show_embedded = getattr(core, "show_embedded", None)

        # Setup callback for event-driven initialization
        setup_via_callback = False
        if hasattr(core, "set_on_hwnd_created"):

            def on_hwnd_created(hwnd):
                if _VERBOSE_LOGGING:
                    logger.debug(f"[LifecycleMixin] Rust callback: HWND created 0x{hwnd:X}")
                # Initialize container immediately
                self._create_webview_container(core, hwnd=hwnd)  # type: ignore[attr-defined]

            try:
                core.set_on_hwnd_created(on_hwnd_created)
                setup_via_callback = True
                if _VERBOSE_LOGGING:
                    logger.debug("[LifecycleMixin] set_on_hwnd_created callback registered")
            except Exception as e:
                logger.warning(f"[LifecycleMixin] Failed to set on_hwnd_created callback: {e}")

        try:
            if callable(show_embedded):
                core_show_start = time.time()
                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[LifecycleMixin] Calling core.show_embedded() for embed_mode={embed_mode!r}"
                    )
                show_embedded()
                core_show_time = (time.time() - core_show_start) * 1000
                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[LifecycleMixin] core.show_embedded() returned in {core_show_time:.1f}ms"
                    )
            else:
                core_show_start = time.time()
                logger.warning(
                    "[LifecycleMixin] core.show_embedded() not available; "
                    "falling back to core.show() (may block DCC UI!)"
                )
                core.show()
                core_show_time = (time.time() - core_show_start) * 1000
                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[LifecycleMixin] core.show() fallback returned in {core_show_time:.1f}ms"
                    )
        except Exception as exc:
            logger.warning(
                f"[LifecycleMixin] core.show_embedded()/core.show() failed ({exc}); "
                "falling back to WebView.show()"
            )
            self._webview.show()
            return

        # Process events after blocking operation
        QApplication.processEvents()

        # Step 3: Create Qt container for WebView
        if not setup_via_callback:
            self._create_webview_container(core)  # type: ignore[attr-defined]

        QApplication.processEvents()

        # Step 4: Ensure WebView is visible after container creation
        try:
            core.set_visible(True)
            core.process_events()
            if _VERBOSE_LOGGING:
                logger.debug("[LifecycleMixin] WebView visibility ensured after container creation")
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[LifecycleMixin] Failed to set visibility: {e}")

        QApplication.processEvents()

        # Step 5: Switch from loading page to webview page
        self._stack.setCurrentIndex(1)

        # Step 6: Restore window visibility (anti-flicker completion)
        if getattr(self, "_pre_show_hidden", False) and sys.platform == "win32":
            qt_hwnd = int(self.winId())  # type: ignore[attr-defined]
            if qt_hwnd:
                show_window_after_init(qt_hwnd)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[LifecycleMixin] Restored visibility: HWND=0x{qt_hwnd:X}")
            self._pre_show_hidden = False

        QApplication.processEvents()

        # Step 7: Schedule delayed geometry sync for DCC apps
        def delayed_geometry_sync() -> None:
            """Sync geometry after layout has stabilized."""
            try:
                self._force_container_geometry()  # type: ignore[attr-defined]
                self._sync_webview2_controller_bounds()  # type: ignore[attr-defined]
                if _VERBOSE_LOGGING:
                    logger.debug("[LifecycleMixin] Delayed geometry sync completed")
            except Exception:
                pass

        # Schedule multiple syncs at different intervals
        for delay in [50, 100, 250, 500, 1000]:
            QTimer.singleShot(delay, delayed_geometry_sync)

        # Step 8: Load initial content
        if self._initial_url:
            if _VERBOSE_LOGGING:
                logger.debug(f"[LifecycleMixin] Loading initial URL: {self._initial_url}")
            self._webview.load_url(self._initial_url)
        elif self._initial_html:
            if _VERBOSE_LOGGING:
                logger.debug(
                    f"[LifecycleMixin] Loading initial HTML ({len(self._initial_html)} bytes)"
                )
            self._webview.load_html(self._initial_html)

        # Step 9: Start EventTimer for message processing
        timer = getattr(self._webview, "_auto_timer", None)
        if timer is not None:
            try:
                timer.start()
                total_time = (time.time() - start_time) * 1000
                if _VERBOSE_LOGGING:
                    logger.debug(f"[LifecycleMixin] Ready in {total_time:.1f}ms")
                return
            except Exception as exc:
                logger.warning(f"[LifecycleMixin] EventTimer failed ({exc}), using fallback")

        # Fallback
        self._webview.show()

    def _handle_close_event(self) -> bool:
        """Handle close event logic.

        Returns:
            True if close was already in progress (should accept and return early)
        """
        if self._is_closing:
            return True

        if _VERBOSE_LOGGING:
            logger.debug("[LifecycleMixin] closeEvent")
        self._is_closing = True

        try:
            # Close the WebView
            try:
                self._webview.close()
            except Exception as e:  # pragma: no cover
                if _VERBOSE_LOGGING:
                    logger.debug("[LifecycleMixin] error closing embedded WebView: %s", e)

            # Reset initialization state for potential reuse
            self._reset_state_for_reuse()
        except Exception:
            pass

        return False

    def _reset_state_for_reuse(self) -> None:
        """Reset internal state so the widget can be shown again.

        This is called during closeEvent to prepare the widget for potential reuse.
        In DCC environments, users may close and reopen tool panels multiple times.
        Without resetting state, the WebView would not reinitialize on subsequent shows.
        """
        if _VERBOSE_LOGGING:
            logger.debug("[LifecycleMixin] Resetting state for reuse")

        # Reset initialization flag so showEvent will reinitialize
        self._webview_initialized = False

        # Note: We intentionally do NOT reset _is_closing here.
        # The _is_closing flag should remain True after closeEvent to indicate
        # the widget is in a closing state. It will be reset by showEvent
        # if the widget is shown again.

        # Clear container references (will be recreated on next show)
        self._webview_container = None
        self._webview_qwindow = None

        # Reset the Rust WebView state if the method exists
        try:
            core = getattr(self._webview, "_core", None)
            if core is not None and hasattr(core, "reset"):
                core.reset()
                if _VERBOSE_LOGGING:
                    logger.debug("[LifecycleMixin] Rust WebView state reset")
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug("[LifecycleMixin] Failed to reset Rust state: %s", e)

    def _handle_destructor(self) -> None:
        """Handle destructor cleanup.

        Called from __del__ to ensure cleanup if the widget is GC'ed unexpectedly.
        """
        try:
            if not getattr(self, "_is_closing", False) and hasattr(self, "_webview"):
                self._webview.close()
        except Exception as e:  # pragma: no cover
            if _VERBOSE_LOGGING:
                logger.debug("[LifecycleMixin] __del__ error: %s", e)


__all__ = ["LifecycleMixin"]
