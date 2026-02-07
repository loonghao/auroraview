# -*- coding: utf-8 -*-
"""WebView container embedding for Qt integration.

This module provides the EmbeddingMixin class that handles the complex logic
of embedding a WebView native window into a Qt widget. It supports two modes:

1. Direct embedding (Qt6 preferred): Uses Win32 SetParent() directly
2. createWindowContainer (Qt5 fallback): Uses Qt's native container mechanism
"""

import logging
import os
import sys
from typing import TYPE_CHECKING, Optional

try:
    from qtpy.QtCore import QTimer
    from qtpy.QtGui import QWindow
    from qtpy.QtWidgets import QVBoxLayout, QWidget
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

from auroraview.integration.qt._compat import (
    apply_clip_styles_to_parent,
    create_container_widget,
    embed_window_directly,
    get_qt_info,
    is_qt6,
    post_container_setup,
    prepare_hwnd_for_container,
    supports_direct_embedding,
    update_embedded_window_geometry,
)

if TYPE_CHECKING:
    from auroraview.core.webview import WebView

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)


class EmbeddingMixin:
    """Mixin class providing WebView container embedding functionality.

    This mixin provides methods for embedding a native WebView window into
    a Qt widget hierarchy. It handles the complexity of:

    - Detecting the best embedding mode (direct vs createWindowContainer)
    - Setting up the native window as a child of the Qt widget
    - Synchronizing geometry between Qt and the native window
    - Handling WebView2 child window issues on Qt6

    The mixin expects the host class to have:
    - self._webview: WebView instance
    - self._webview_container: QWidget for the container
    - self._webview_qwindow: QWindow wrapper
    - self._webview_page_layout: QVBoxLayout for the webview page
    - self.winId(): method to get Qt widget's native handle
    - self.size(): method to get Qt widget size
    """

    # Type hints for expected attributes
    _webview: "WebView"
    _webview_container: Optional[QWidget]
    _webview_qwindow: Optional[QWindow]
    _webview_page: QWidget
    _webview_page_layout: "QVBoxLayout"  # type: ignore[name-defined]
    _using_direct_embed: bool
    _direct_embed_hwnd: Optional[int]

    def _create_webview_container(self, core, hwnd: Optional[int] = None) -> None:
        """Create Qt container for WebView after WebView is initialized.

        This method supports two embedding modes:
        1. Direct embedding (Qt6 preferred): Uses SetParent() directly, bypassing
           createWindowContainer which has known issues on Qt6.
        2. createWindowContainer (Qt5 fallback): Uses Qt's native container mechanism.

        The mode is automatically selected based on:
        - Qt version (Qt6 prefers direct embedding)
        - Platform support (Windows required for direct embedding)
        - Environment variable override: AURORAVIEW_USE_DIRECT_EMBED=1/0

        Args:
            core: The core WebView object (Rust binding)
            hwnd: Optional pre-fetched HWND. If None, will call core.get_hwnd()
        """
        try:
            if hwnd is not None:
                webview_hwnd = hwnd
            else:
                get_hwnd = getattr(core, "get_hwnd", None)
                webview_hwnd = get_hwnd() if callable(get_hwnd) else None

            if not webview_hwnd:
                logger.warning("[EmbeddingMixin] No HWND available for container")
                return

            # Log Qt version info for debugging
            qt_binding, qt_version = get_qt_info()
            if _VERBOSE_LOGGING:
                logger.debug(
                    f"[EmbeddingMixin] Creating container for HWND=0x{webview_hwnd:X} "
                    f"(Qt binding: {qt_binding}, version: {qt_version})"
                )

            # Determine embedding mode
            env_direct = os.environ.get("AURORAVIEW_USE_DIRECT_EMBED", "").lower()
            if env_direct in ("1", "true", "yes", "on"):
                use_direct_embed = True
            elif env_direct in ("0", "false", "no", "off"):
                use_direct_embed = False
            else:
                # Auto-detect: Use direct embedding on Qt6 if platform supports it
                use_direct_embed = is_qt6() and supports_direct_embedding()

            if use_direct_embed:
                self._create_container_direct(webview_hwnd)
            else:
                self._create_container_qt(webview_hwnd)

        except Exception as e:
            logger.exception(f"[EmbeddingMixin] Failed to create container: {e}")
            self._webview_container = None

    def _create_container_direct(self, webview_hwnd: int) -> None:
        """Create container using direct SetParent embedding.

        This mode is preferred for Qt6 where createWindowContainer has known issues
        with WebView2. It uses Win32 SetParent() directly to establish the parent-child
        relationship.

        Args:
            webview_hwnd: The WebView's native window handle.
        """
        logger.info(f"[EmbeddingMixin] Using DIRECT embedding mode for HWND=0x{webview_hwnd:X}")

        # Get our widget's HWND
        parent_hwnd = int(self.winId())  # type: ignore[attr-defined]
        if not parent_hwnd:
            logger.error("[EmbeddingMixin] Failed to get parent widget HWND")
            self._create_container_qt(webview_hwnd)  # Fallback
            return

        # Get initial size
        size = self.size()  # type: ignore[attr-defined]
        width = size.width() if size.width() > 0 else 800
        height = size.height() if size.height() > 0 else 600

        # Use direct embedding via platform backend
        success = embed_window_directly(webview_hwnd, parent_hwnd, width, height)
        if not success:
            logger.warning(
                "[EmbeddingMixin] Direct embedding failed, falling back to createWindowContainer"
            )
            self._create_container_qt(webview_hwnd)
            return

        # Mark that we're using direct embedding mode
        self._using_direct_embed = True
        self._direct_embed_hwnd = webview_hwnd

        # Create a placeholder widget to participate in Qt layout
        self._webview_container = QWidget(self)  # type: ignore[arg-type]
        self._webview_container.setStyleSheet(
            "border: none; margin: 0; padding: 0; background-color: transparent;"
        )
        self._webview_page.setStyleSheet("background-color: #0d0d0d;")

        # Add placeholder to layout
        self._webview_page_layout.addWidget(self._webview_container, 1)

        # Apply clip styles to parent
        apply_clip_styles_to_parent(parent_hwnd)

        # Finalize anti-flicker optimizations
        core = getattr(self._webview, "_core", None)
        if core is not None:
            finalize_fn = getattr(core, "finalize_container_embedding", None)
            if callable(finalize_fn):
                try:
                    finalize_fn()
                except Exception:
                    pass

        # Fix WebView2 child windows
        self._schedule_child_window_fixes(webview_hwnd)

        logger.info(f"[EmbeddingMixin] Direct embedding successful: HWND=0x{webview_hwnd:X}")

    def _schedule_child_window_fixes(self, webview_hwnd: int) -> None:
        """Schedule multiple attempts to fix WebView2 child windows.

        WebView2 creates child windows (Chrome_WidgetWin_0, etc.) asynchronously
        after the main window is created. We need to fix them multiple times
        to catch all of them as they're created.

        Args:
            webview_hwnd: The WebView's native window handle.
        """
        from auroraview.integration.qt.platforms import get_backend

        def fix_children():
            """Fix all child windows."""
            try:
                backend = get_backend()
                if hasattr(backend, "_fix_all_child_windows_recursive"):
                    count = backend._fix_all_child_windows_recursive(webview_hwnd)
                    if count > 0:
                        logger.info(f"[EmbeddingMixin] Fixed {count} WebView2 child windows")
            except Exception as e:
                if _VERBOSE_LOGGING:
                    logger.debug(f"[EmbeddingMixin] fix_children failed: {e}")

        # Fix immediately
        fix_children()

        # Schedule delayed fixes to catch asynchronously created child windows
        delays = [50, 100, 200, 500, 1000, 2000]
        for delay in delays:
            QTimer.singleShot(delay, fix_children)

    def _create_container_qt(self, webview_hwnd: int) -> None:
        """Create container using Qt's createWindowContainer.

        This is the traditional embedding mode that works well on Qt5.
        On Qt6, it may have issues with WebView2 (white frames, dragging problems).

        Args:
            webview_hwnd: The WebView's native window handle.
        """
        logger.info(
            f"[EmbeddingMixin] Using createWindowContainer mode for HWND=0x{webview_hwnd:X}"
        )

        self._using_direct_embed = False

        # Step 1: Prepare HWND using compat layer
        prepare_hwnd_for_container(webview_hwnd)

        # Step 2: Wrap the native HWND as a QWindow
        self._webview_qwindow = QWindow.fromWinId(webview_hwnd)
        if self._webview_qwindow is None:
            logger.error("[EmbeddingMixin] QWindow.fromWinId returned None")
            return

        if _VERBOSE_LOGGING:
            logger.debug("[EmbeddingMixin] QWindow created from HWND")

        # Step 3: Create container using compat layer
        self._webview_container = create_container_widget(
            self._webview_qwindow,
            self,  # type: ignore[arg-type]
            focus_policy="strong",
        )
        if self._webview_container is None:
            logger.error("[EmbeddingMixin] create_container_widget returned None")
            return

        # Ensure container has minimal styling
        self._webview_container.setStyleSheet(
            "border: none; margin: 0; padding: 0; background-color: #0d0d0d;"
        )
        self._webview_page.setStyleSheet("background-color: #0d0d0d;")

        # Step 4: Add container to webview page layout
        self._webview_page_layout.addWidget(self._webview_container, 1)

        # Step 5: Apply clip styles to QtWebView widget
        if sys.platform == "win32":
            self_hwnd = int(self.winId())  # type: ignore[attr-defined]
            if self_hwnd:
                apply_clip_styles_to_parent(self_hwnd)

        # Step 6: Finalize anti-flicker optimizations
        core = getattr(self._webview, "_core", None)
        if core is not None:
            finalize_fn = getattr(core, "finalize_container_embedding", None)
            if callable(finalize_fn):
                try:
                    finalize_fn()
                    if _VERBOSE_LOGGING:
                        logger.debug("[EmbeddingMixin] Anti-flicker optimizations removed")
                except Exception as e:
                    if _VERBOSE_LOGGING:
                        logger.debug(f"[EmbeddingMixin] finalize_container_embedding failed: {e}")

        # Step 7: Post-container setup
        post_container_setup(self._webview_container, webview_hwnd)

        # Step 8: Force container to fill parent layout immediately
        self._force_container_geometry()

        # Step 9: Fix WebView2 child windows for Qt6 compatibility
        if sys.platform == "win32":
            try:
                import auroraview

                fix_fn = getattr(auroraview, "fix_webview2_child_windows", None)
                if callable(fix_fn):
                    fix_fn(webview_hwnd)
                    if _VERBOSE_LOGGING:
                        logger.debug(
                            f"[EmbeddingMixin] Fixed WebView2 child windows for HWND=0x{webview_hwnd:X}"
                        )
            except Exception as e:
                if _VERBOSE_LOGGING:
                    logger.debug(f"[EmbeddingMixin] fix_webview2_child_windows failed: {e}")

        if _VERBOSE_LOGGING:
            logger.debug(
                "[EmbeddingMixin] Container created successfully for HWND=0x%X", webview_hwnd
            )

    def _sync_embedded_geometry(self) -> None:
        """Resize the embedded native WebView window to match this QWidget.

        When using createWindowContainer, Qt handles geometry automatically,
        so this method only needs to remove window borders on first call.
        """
        try:
            if sys.platform != "win32":
                return
            # When using createWindowContainer, Qt handles geometry automatically
            pass
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug("[EmbeddingMixin] failed to sync embedded geometry: %s", e)

    def _sync_webview2_controller_bounds(self, force_width: int = 0, force_height: int = 0) -> None:
        """Sync WebView2 controller bounds with container size.

        This is needed because createWindowContainer only handles the native
        window position/size, but WebView2's controller may need explicit
        bounds update to render content correctly.

        Args:
            force_width: If > 0, use this width instead of container size.
            force_height: If > 0, use this height instead of container size.
        """
        try:
            container = getattr(self, "_webview_container", None)
            if container is None:
                logger.debug("[EmbeddingMixin] _sync_webview2_controller_bounds: container is None")
                return

            # Get container size or use forced size
            if force_width > 0 and force_height > 0:
                width = force_width
                height = force_height
            else:
                container_size = container.size()
                width = container_size.width()
                height = container_size.height()

            if width <= 0 or height <= 0:
                logger.debug(
                    f"[EmbeddingMixin] _sync_webview2_controller_bounds: invalid size {width}x{height}"
                )
                return

            logger.info(
                f"[EmbeddingMixin] _sync_webview2_controller_bounds: syncing to {width}x{height}"
            )

            # Try to sync WebView2 controller bounds via Rust API
            core = getattr(self._webview, "_core", None)
            if core is not None:
                # Prefer sync_bounds for Qt6 compatibility
                sync_bounds = getattr(core, "sync_bounds", None)
                if callable(sync_bounds):
                    try:
                        sync_bounds(width, height)
                        logger.info(
                            f"[EmbeddingMixin] sync_bounds({width}, {height}) called successfully"
                        )
                        return
                    except Exception as e:
                        logger.warning(f"[EmbeddingMixin] sync_bounds failed: {e}")
                else:
                    logger.warning("[EmbeddingMixin] sync_bounds not available on core")

                # Fallback to set_size
                set_size = getattr(core, "set_size", None)
                if callable(set_size):
                    try:
                        set_size(width, height)
                        logger.info(
                            f"[EmbeddingMixin] Synced WebView2 bounds via set_size: {width}x{height}"
                        )
                    except Exception as e:
                        logger.warning(f"[EmbeddingMixin] set_size failed: {e}")
            else:
                logger.warning("[EmbeddingMixin] _sync_webview2_controller_bounds: core is None")

        except Exception as e:
            logger.warning(f"[EmbeddingMixin] _sync_webview2_controller_bounds failed: {e}")

    def _force_container_geometry(self) -> None:
        """Force container to fill parent layout immediately."""
        try:
            from qtpy.QtWidgets import QApplication

            container = getattr(self, "_webview_container", None)
            if container is None:
                return

            # Get our size
            our_size = self.size()  # type: ignore[attr-defined]
            width = our_size.width()
            height = our_size.height()

            if width <= 0 or height <= 0:
                return

            # Force container to fill our size
            container.setGeometry(0, 0, width, height)
            container.resize(width, height)

            # Also resize the QWindow if available
            qwindow = getattr(self, "_webview_qwindow", None)
            if qwindow is not None:
                try:
                    qwindow.resize(width, height)
                except Exception:
                    pass

            # Qt5-style: single processEvents
            QApplication.processEvents()

            # Sync WebView2 controller bounds
            self._sync_webview2_controller_bounds(width, height)

            if _VERBOSE_LOGGING:
                logger.debug(f"[EmbeddingMixin] Forced container geometry: {width}x{height}")

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[EmbeddingMixin] _force_container_geometry failed: {e}")

    def _handle_resize_for_embedding(self, width: int, height: int) -> None:
        """Handle resize event for embedded WebView.

        Called from resizeEvent to update embedded WebView geometry.

        Args:
            width: New width
            height: New height
        """
        # Handle direct embedding mode
        if getattr(self, "_using_direct_embed", False):
            direct_hwnd = getattr(self, "_direct_embed_hwnd", None)
            if direct_hwnd:
                update_embedded_window_geometry(direct_hwnd, 0, 0, width, height)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[EmbeddingMixin] Direct embed resize: {width}x{height}")

        # Force container to fill parent and sync WebView2 controller bounds
        container = getattr(self, "_webview_container", None)
        if container is not None:
            container.setGeometry(0, 0, width, height)
            self._sync_webview2_controller_bounds()


__all__ = ["EmbeddingMixin"]
