"""Feature modules for AuroraView.

This package provides browser-like features that can be used with WebView:
- bookmarks: Bookmark management
- history: Browsing history
- downloads: Download management
- settings: User settings
- notifications: Notification system
"""

from __future__ import annotations

from .bookmarks import BookmarkManager, Bookmark, BookmarkFolder
from .history import HistoryManager, HistoryEntry
from .downloads import DownloadManager, DownloadItem, DownloadState
from .settings import SettingsManager, Setting
from .notifications import NotificationManager, Notification, NotificationType

__all__ = [
    # Bookmarks
    "BookmarkManager",
    "Bookmark",
    "BookmarkFolder",
    # History
    "HistoryManager",
    "HistoryEntry",
    # Downloads
    "DownloadManager",
    "DownloadItem",
    "DownloadState",
    # Settings
    "SettingsManager",
    "Setting",
    # Notifications
    "NotificationManager",
    "Notification",
    "NotificationType",
]
