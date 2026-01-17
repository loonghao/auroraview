"""Features API for AuroraView Gallery.

This module provides API handlers for browser-like features:
- Bookmarks management
- History tracking
- Downloads management
- Settings management
- Notifications
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any, Optional, TYPE_CHECKING

from auroraview.features import (
    BookmarkManager,
    Bookmark,
    BookmarkFolder,
    HistoryManager,
    HistoryEntry,
    DownloadManager,
    DownloadItem,
    DownloadState,
    SettingsManager,
    NotificationManager,
    NotificationType,
)

if TYPE_CHECKING:
    from auroraview import WebView

# Module-level managers (initialized on first use)
_bookmark_manager: Optional[BookmarkManager] = None
_history_manager: Optional[HistoryManager] = None
_download_manager: Optional[DownloadManager] = None
_settings_manager: Optional[SettingsManager] = None
_notification_manager: Optional[NotificationManager] = None

# Data directory
_data_dir: Optional[Path] = None


def _get_data_dir() -> Path:
    """Get data directory for storing feature data."""
    global _data_dir
    if _data_dir is None:
        # Use user's app data directory or local data
        if os.name == 'nt':
            base = Path(os.environ.get('LOCALAPPDATA', '~'))
        else:
            base = Path.home() / '.local' / 'share'
        _data_dir = base.expanduser() / 'auroraview' / 'gallery'
        _data_dir.mkdir(parents=True, exist_ok=True)
    return _data_dir


def get_bookmark_manager() -> BookmarkManager:
    """Get or create bookmark manager."""
    global _bookmark_manager
    if _bookmark_manager is None:
        _bookmark_manager = BookmarkManager(data_dir=_get_data_dir())
    return _bookmark_manager


def get_history_manager() -> HistoryManager:
    """Get or create history manager."""
    global _history_manager
    if _history_manager is None:
        _history_manager = HistoryManager(data_dir=_get_data_dir())
    return _history_manager


def get_download_manager() -> DownloadManager:
    """Get or create download manager."""
    global _download_manager
    if _download_manager is None:
        _download_manager = DownloadManager(data_dir=_get_data_dir())
    return _download_manager


def get_settings_manager() -> SettingsManager:
    """Get or create settings manager."""
    global _settings_manager
    if _settings_manager is None:
        _settings_manager = SettingsManager(data_dir=_get_data_dir())
    return _settings_manager


def get_notification_manager() -> NotificationManager:
    """Get or create notification manager."""
    global _notification_manager
    if _notification_manager is None:
        _notification_manager = NotificationManager()
    return _notification_manager


# ============================================================================
# Bookmarks API
# ============================================================================

def get_bookmarks() -> dict[str, Any]:
    """Get all bookmarks."""
    manager = get_bookmark_manager()
    bookmarks = manager.all()  # Use correct method name
    folders = manager.all_folders()  # Use correct method name
    return {
        "ok": True,
        "bookmarks": [_bookmark_to_dict(b) for b in bookmarks],
        "folders": [_folder_to_dict(f) for f in folders],
    }


def add_bookmark(url: str, title: str, folder_id: Optional[str] = None) -> dict[str, Any]:
    """Add a new bookmark."""
    manager = get_bookmark_manager()
    bookmark = manager.add(url=url, title=title, folder_id=folder_id)  # Use correct method name
    return {
        "ok": True,
        "bookmark": _bookmark_to_dict(bookmark),
    }


def remove_bookmark(bookmark_id: str) -> dict[str, Any]:
    """Remove a bookmark."""
    manager = get_bookmark_manager()
    success = manager.remove(bookmark_id)  # Use correct method name
    return {"ok": success}


def update_bookmark(bookmark_id: str, title: Optional[str] = None, 
                   url: Optional[str] = None, folder_id: Optional[str] = None) -> dict[str, Any]:
    """Update a bookmark."""
    manager = get_bookmark_manager()
    bookmark = manager.get(bookmark_id)  # Use correct method name
    if bookmark is None:
        return {"ok": False, "error": "Bookmark not found"}
    
    # Use the update method
    manager.update(bookmark_id, title=title, url=url)
    
    # Handle folder change if needed
    if folder_id is not None:
        manager.move_to_folder(bookmark_id, folder_id)
    
    updated = manager.get(bookmark_id)
    return {"ok": True, "bookmark": _bookmark_to_dict(updated) if updated else None}


def create_folder(name: str, parent_id: Optional[str] = None) -> dict[str, Any]:
    """Create a bookmark folder."""
    manager = get_bookmark_manager()
    folder = manager.create_folder(name=name, parent_id=parent_id)
    return {"ok": True, "folder": _folder_to_dict(folder)}


def remove_folder(folder_id: str) -> dict[str, Any]:
    """Remove a bookmark folder."""
    manager = get_bookmark_manager()
    success = manager.delete_folder(folder_id)  # Use correct method name
    return {"ok": success}


def is_bookmarked(url: str) -> dict[str, Any]:
    """Check if a URL is bookmarked."""
    manager = get_bookmark_manager()
    bookmark = manager.find_by_url(url)
    return {
        "ok": True,
        "bookmarked": bookmark is not None,
        "bookmark": _bookmark_to_dict(bookmark) if bookmark else None,
    }


def _bookmark_to_dict(b: Bookmark) -> dict[str, Any]:
    """Convert bookmark to dict."""
    return {
        "id": b.id,
        "url": b.url,
        "title": b.title,
        "favicon": b.favicon,
        "folderId": b.parent_id,  # Use correct field name
        "createdAt": b.created_at.isoformat() if b.created_at else None,
    }


def _folder_to_dict(f: BookmarkFolder) -> dict[str, Any]:
    """Convert folder to dict."""
    manager = get_bookmark_manager()
    bookmarks_in_folder = manager.in_folder(f.id)
    return {
        "id": f.id,
        "name": f.name,
        "parentId": f.parent_id,
        "bookmarks": [_bookmark_to_dict(b) for b in bookmarks_in_folder],
    }


# ============================================================================
# History API
# ============================================================================

def get_history(limit: int = 100, offset: int = 0) -> dict[str, Any]:
    """Get browsing history."""
    manager = get_history_manager()
    entries = manager.recent(limit=limit)  # Use correct method name
    return {
        "ok": True,
        "entries": [_history_to_dict(e) for e in entries],
        "total": manager.count,
    }


def add_history(url: str, title: str) -> dict[str, Any]:
    """Add a history entry."""
    manager = get_history_manager()
    entry = manager.visit(url=url, title=title)  # Use correct method name
    return {"ok": True, "entry": _history_to_dict(entry)}


def remove_history(entry_id: str) -> dict[str, Any]:
    """Remove a history entry."""
    manager = get_history_manager()
    success = manager.delete(entry_id)  # Use correct method name
    return {"ok": success}


def clear_history() -> dict[str, Any]:
    """Clear all history."""
    manager = get_history_manager()
    manager.clear()  # Use correct method name
    return {"ok": True}


def search_history(query: str, limit: int = 50) -> dict[str, Any]:
    """Search history entries."""
    manager = get_history_manager()
    entries = manager.search(query, limit=limit)
    return {
        "ok": True,
        "entries": [_history_to_dict(e) for e in entries],
    }


def _history_to_dict(e: HistoryEntry) -> dict[str, Any]:
    """Convert history entry to dict."""
    return {
        "id": e.id,
        "url": e.url,
        "title": e.title,
        "visitCount": e.visit_count,
        "lastVisit": e.last_visit.isoformat() if e.last_visit else None,
        "favicon": e.favicon,
    }


# ============================================================================
# Downloads API
# ============================================================================

def get_downloads() -> dict[str, Any]:
    """Get all downloads."""
    manager = get_download_manager()
    downloads = manager.all()
    return {
        "ok": True,
        "downloads": [_download_to_dict(d) for d in downloads],
    }


def pause_download(download_id: str) -> dict[str, Any]:
    """Pause a download."""
    manager = get_download_manager()
    success = manager.pause(download_id)
    return {"ok": success}


def resume_download(download_id: str) -> dict[str, Any]:
    """Resume a download."""
    manager = get_download_manager()
    success = manager.resume(download_id)
    return {"ok": success}


def cancel_download(download_id: str) -> dict[str, Any]:
    """Cancel a download."""
    manager = get_download_manager()
    success = manager.cancel(download_id)
    return {"ok": success}


def retry_download(download_id: str) -> dict[str, Any]:
    """Retry a failed download."""
    manager = get_download_manager()
    download = manager.get(download_id)
    if download is None:
        return {"ok": False, "error": "Download not found"}
    
    # Reset failed download to pending state
    if download.state == DownloadState.FAILED:
        download.state = DownloadState.PENDING
        download.error = None
        return {"ok": True}
    return {"ok": False, "error": "Download is not in failed state"}


def remove_download(download_id: str) -> dict[str, Any]:
    """Remove a download from list."""
    manager = get_download_manager()
    success = manager.remove(download_id)
    return {"ok": success}


def clear_completed_downloads() -> dict[str, Any]:
    """Clear completed downloads."""
    manager = get_download_manager()
    manager.clear_completed()
    return {"ok": True}


def _download_to_dict(d: DownloadItem) -> dict[str, Any]:
    """Convert download item to dict."""
    return {
        "id": d.id,
        "url": d.url,
        "filename": d.filename,
        "path": d.save_path,  # Use correct field name
        "size": d.total_bytes,  # Use correct field name
        "downloadedBytes": d.received_bytes,  # Use correct field name
        "state": d.state.value if isinstance(d.state, DownloadState) else d.state,
        "error": d.error,
        "startedAt": d.start_time.isoformat() if d.start_time else None,  # Use correct field name
        "completedAt": d.end_time.isoformat() if d.end_time else None,  # Use correct field name
        "speed": None,  # Not tracked in current implementation
    }


# ============================================================================
# Settings API
# ============================================================================

def get_settings() -> dict[str, Any]:
    """Get all settings."""
    manager = get_settings_manager()
    settings = manager.all()  # Returns dict[str, SettingValue]
    return {
        "ok": True,
        "settings": settings,
    }


def get_setting(key: str) -> dict[str, Any]:
    """Get a specific setting."""
    manager = get_settings_manager()
    value = manager.get(key)
    return {"ok": True, "key": key, "value": value}


def set_setting(key: str, value: Any) -> dict[str, Any]:
    """Set a setting value."""
    manager = get_settings_manager()
    manager.set(key, value)
    return {"ok": True}


def reset_settings() -> dict[str, Any]:
    """Reset all settings to defaults."""
    manager = get_settings_manager()
    manager.reset_all()
    return {"ok": True}


# ============================================================================
# Notifications API
# ============================================================================

def show_notification(
    title: str,
    message: str,
    type: str = "info",
    timeout: Optional[int] = None,
) -> dict[str, Any]:
    """Show a notification."""
    manager = get_notification_manager()
    # Map type string to NotificationType enum
    type_map = {
        "info": NotificationType.INFO,
        "success": NotificationType.SUCCESS,
        "warning": NotificationType.WARNING,
        "error": NotificationType.ERROR,
    }
    notification_type = type_map.get(type, NotificationType.INFO)
    
    notification = manager.notify(
        title=title,
        body=message,  # Use 'body' parameter
        type=notification_type,
        auto_close=timeout,
    )
    return {"ok": True, "id": notification.id}


def dismiss_notification(notification_id: str) -> dict[str, Any]:
    """Dismiss a notification."""
    manager = get_notification_manager()
    success = manager.dismiss(notification_id)
    return {"ok": success}


def get_notifications() -> dict[str, Any]:
    """Get all active notifications."""
    manager = get_notification_manager()
    notifications = manager.all()
    return {
        "ok": True,
        "notifications": [
            {
                "id": n.id,
                "title": n.title,
                "message": n.body,  # Use 'body' field
                "type": n.type.value,
            }
            for n in notifications
        ],
    }


# ============================================================================
# Register APIs with WebView
# ============================================================================

def register_features_api(webview: "WebView") -> None:
    """Register all features APIs with a WebView instance."""
    # Bookmarks
    webview.bind_call("features.get_bookmarks", lambda: get_bookmarks())
    webview.bind_call("features.add_bookmark", add_bookmark)
    webview.bind_call("features.remove_bookmark", remove_bookmark)
    webview.bind_call("features.update_bookmark", update_bookmark)
    webview.bind_call("features.create_folder", create_folder)
    webview.bind_call("features.remove_folder", remove_folder)
    webview.bind_call("features.is_bookmarked", is_bookmarked)
    
    # History
    webview.bind_call("features.get_history", get_history)
    webview.bind_call("features.add_history", add_history)
    webview.bind_call("features.remove_history", remove_history)
    webview.bind_call("features.clear_history", clear_history)
    webview.bind_call("features.search_history", search_history)
    
    # Downloads
    webview.bind_call("features.get_downloads", get_downloads)
    webview.bind_call("features.pause_download", pause_download)
    webview.bind_call("features.resume_download", resume_download)
    webview.bind_call("features.cancel_download", cancel_download)
    webview.bind_call("features.retry_download", retry_download)
    webview.bind_call("features.remove_download", remove_download)
    webview.bind_call("features.clear_completed_downloads", clear_completed_downloads)
    
    # Settings
    webview.bind_call("features.get_settings", get_settings)
    webview.bind_call("features.get_setting", get_setting)
    webview.bind_call("features.set_setting", set_setting)
    webview.bind_call("features.reset_settings", reset_settings)
    
    # Notifications
    webview.bind_call("features.show_notification", show_notification)
    webview.bind_call("features.dismiss_notification", dismiss_notification)
    webview.bind_call("features.get_notifications", get_notifications)
