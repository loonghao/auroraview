# -*- coding: utf-8 -*-
"""Global window registry for managing multiple WebView instances.

This module provides a centralized way to track, access, and manage
multiple WebView windows across an application.
"""

from __future__ import annotations

import logging
import weakref
from threading import Lock
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional
from uuid import uuid4

if TYPE_CHECKING:
    from .webview import WebView

logger = logging.getLogger(__name__)


class WindowManager:
    """Global window registry for managing multiple WebView instances.

    Features:
    - Singleton pattern for global access
    - Thread-safe operations
    - Weak references to prevent memory leaks
    - Change notification callbacks
    - Event broadcasting to all windows

    Example:
        >>> from auroraview.core.window_manager import get_window_manager
        >>> wm = get_window_manager()
        >>> wm.register(webview)
        'wv_a1b2c3d4'
        >>> wm.get_active()
        <WebView object>
        >>> wm.get_all()
        [<WebView>, <WebView>]
    """

    _instance: Optional["WindowManager"] = None
    _class_lock = Lock()

    def __new__(cls) -> "WindowManager":
        if cls._instance is None:
            with cls._class_lock:
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
                    cls._instance._init()
        return cls._instance

    def _init(self) -> None:
        """Initialize instance state."""
        self._windows: Dict[str, weakref.ref] = {}
        self._active_id: Optional[str] = None
        self._on_change_callbacks: List[Callable[[], None]] = []
        self._lock = Lock()
        logger.debug("WindowManager initialized")

    def register(self, window: "WebView", uid: Optional[str] = None) -> str:
        """Register a window and return its unique ID.

        Args:
            window: The WebView instance to register
            uid: Optional custom unique ID. Auto-generated if not provided.

        Returns:
            The unique ID assigned to this window
        """
        with self._lock:
            if uid is None:
                uid = f"wv_{uuid4().hex[:8]}"

            # Check if already registered
            if uid in self._windows:
                existing = self._windows[uid]()
                if existing is window:
                    logger.debug(f"Window already registered: {uid}")
                    return uid
                logger.warning(f"Replacing existing window with uid: {uid}")

            self._windows[uid] = weakref.ref(window, lambda _: self._on_window_gc(uid))

            if self._active_id is None:
                self._active_id = uid

            logger.info(f"Window registered: {uid} (total: {len(self._windows)})")
            self._notify_change()
            return uid

    def unregister(self, uid: str) -> bool:
        """Unregister a window by ID.

        Args:
            uid: The window's unique ID

        Returns:
            True if window was found and removed
        """
        with self._lock:
            if uid not in self._windows:
                return False

            del self._windows[uid]

            if self._active_id == uid:
                self._active_id = next(iter(self._windows.keys()), None)

            logger.info(f"Window unregistered: {uid} (remaining: {len(self._windows)})")
            self._notify_change()
            return True

    def get(self, uid: str) -> Optional["WebView"]:
        """Get a window by ID.

        Args:
            uid: The window's unique ID

        Returns:
            The WebView instance, or None if not found
        """
        ref = self._windows.get(uid)
        return ref() if ref else None

    def get_active(self) -> Optional["WebView"]:
        """Get the currently active window."""
        if self._active_id:
            return self.get(self._active_id)
        return None

    def get_active_id(self) -> Optional[str]:
        """Get the ID of the currently active window."""
        return self._active_id

    def set_active(self, uid: str) -> bool:
        """Set the active window by ID.

        Args:
            uid: The window's unique ID

        Returns:
            True if window exists and was set as active
        """
        with self._lock:
            if uid not in self._windows:
                return False

            self._active_id = uid
            logger.debug(f"Active window set: {uid}")
            self._notify_change()
            return True

    def get_all(self) -> List["WebView"]:
        """Get all registered windows."""
        with self._lock:
            return [ref() for ref in self._windows.values() if ref() is not None]

    def get_all_ids(self) -> List[str]:
        """Get all registered window IDs."""
        with self._lock:
            return list(self._windows.keys())

    def count(self) -> int:
        """Get the number of registered windows."""
        with self._lock:
            return len(self._windows)

    def has(self, uid: str) -> bool:
        """Check if a window with the given ID exists.

        Args:
            uid: The window's unique ID

        Returns:
            True if window exists
        """
        return uid in self._windows and self._windows[uid]() is not None

    def on_change(self, callback: Callable[[], None]) -> Callable[[], None]:
        """Register a callback for window changes.

        Args:
            callback: Function to call when windows change

        Returns:
            A function to unregister the callback
        """
        self._on_change_callbacks.append(callback)

        def unsubscribe():
            if callback in self._on_change_callbacks:
                self._on_change_callbacks.remove(callback)

        return unsubscribe

    def broadcast(self, event: str, data: Any = None) -> int:
        """Broadcast an event to all windows.

        Args:
            event: Event name
            data: Event payload

        Returns:
            Number of windows that received the event
        """
        count = 0
        for window in self.get_all():
            try:
                window.emit(event, data)
                count += 1
            except Exception as e:
                logger.warning(f"Failed to broadcast to window: {e}")
        logger.debug(f"Broadcast '{event}' to {count} windows")
        return count

    def close_all(self) -> int:
        """Close all registered windows.

        Returns:
            Number of windows closed
        """
        count = 0
        for window in self.get_all():
            try:
                window.close()
                count += 1
            except Exception as e:
                logger.warning(f"Failed to close window: {e}")
        logger.info(f"Closed {count} windows")
        return count

    def find_by_title(self, title: str) -> Optional["WebView"]:
        """Find a window by its title.

        Args:
            title: Window title to search for

        Returns:
            The first matching WebView, or None
        """
        for window in self.get_all():
            try:
                if window.title == title:
                    return window
            except Exception:
                pass
        return None

    def _on_window_gc(self, uid: str) -> None:
        """Handle window being garbage collected."""
        with self._lock:
            if uid in self._windows:
                del self._windows[uid]
                logger.debug(f"Window garbage collected: {uid}")
                if self._active_id == uid:
                    self._active_id = next(iter(self._windows.keys()), None)

    def _notify_change(self) -> None:
        """Notify all change callbacks."""
        for cb in self._on_change_callbacks:
            try:
                cb()
            except Exception as e:
                logger.warning(f"Change callback error: {e}")

    def reset(self) -> None:
        """Reset the WindowManager (for testing).

        Warning: This will clear all registered windows without closing them.
        """
        with self._lock:
            self._windows.clear()
            self._active_id = None
            self._on_change_callbacks.clear()
            logger.info("WindowManager reset")


# Global accessors


def get_window_manager() -> WindowManager:
    """Get the global WindowManager instance."""
    return WindowManager()


def get_windows() -> List["WebView"]:
    """Get all registered WebView windows."""
    return get_window_manager().get_all()


def get_active_window() -> Optional["WebView"]:
    """Get the currently active WebView window."""
    return get_window_manager().get_active()


def broadcast_event(event: str, data: Any = None) -> int:
    """Broadcast an event to all windows.

    Args:
        event: Event name
        data: Event payload

    Returns:
        Number of windows that received the event
    """
    return get_window_manager().broadcast(event, data)
