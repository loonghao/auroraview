"""Notification system for AuroraView.

This module provides notification functionality:
- Desktop notifications
- In-app notifications
- Notification permissions
- Notification history
"""

from __future__ import annotations

import json
import os
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional


class NotificationType(Enum):
    """Notification types."""

    INFO = "info"
    SUCCESS = "success"
    WARNING = "warning"
    ERROR = "error"
    PROGRESS = "progress"


class NotificationPriority(Enum):
    """Notification priority levels."""

    LOW = "low"
    NORMAL = "normal"
    HIGH = "high"
    URGENT = "urgent"


class PermissionState(Enum):
    """Permission state."""

    DEFAULT = "default"
    GRANTED = "granted"
    DENIED = "denied"


@dataclass
class NotificationAction:
    """A notification action button."""

    id: str
    label: str
    icon: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "label": self.label,
            "icon": self.icon,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "NotificationAction":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            label=data["label"],
            icon=data.get("icon"),
        )


@dataclass
class Notification:
    """A notification."""

    id: str
    title: str
    body: str
    type: NotificationType = NotificationType.INFO
    priority: NotificationPriority = NotificationPriority.NORMAL
    icon: Optional[str] = None
    image: Optional[str] = None
    tag: Optional[str] = None  # Group notifications
    timestamp: datetime = field(default_factory=datetime.now)
    actions: List[NotificationAction] = field(default_factory=list)
    data: Optional[Dict[str, Any]] = None
    read: bool = False
    auto_close: Optional[int] = None  # Auto close after N milliseconds

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "title": self.title,
            "body": self.body,
            "type": self.type.value,
            "priority": self.priority.value,
            "icon": self.icon,
            "image": self.image,
            "tag": self.tag,
            "timestamp": self.timestamp.isoformat(),
            "actions": [a.to_dict() for a in self.actions],
            "data": self.data,
            "read": self.read,
            "auto_close": self.auto_close,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Notification":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            title=data["title"],
            body=data["body"],
            type=NotificationType(data.get("type", "info")),
            priority=NotificationPriority(data.get("priority", "normal")),
            icon=data.get("icon"),
            image=data.get("image"),
            tag=data.get("tag"),
            timestamp=datetime.fromisoformat(data["timestamp"])
            if "timestamp" in data
            else datetime.now(),
            actions=[NotificationAction.from_dict(a) for a in data.get("actions", [])],
            data=data.get("data"),
            read=data.get("read", False),
            auto_close=data.get("auto_close"),
        )


# Type alias for notification callback
NotificationCallback = Callable[[Notification], None]
ActionCallback = Callable[[Notification, str], None]  # notification, action_id


class NotificationManager:
    """Manages notifications with persistence."""

    DEFAULT_MAX_HISTORY = 100

    def __init__(
        self,
        data_dir: Optional[Path] = None,
        max_history: int = DEFAULT_MAX_HISTORY,
    ):
        """Initialize notification manager.

        Args:
            data_dir: Directory for storing notifications.
            max_history: Maximum number of notifications to keep.
        """
        self._notifications: Dict[str, Notification] = {}
        self._permissions: Dict[str, PermissionState] = {}
        self._max_history = max_history
        self._callbacks: List[NotificationCallback] = []
        self._action_callbacks: List[ActionCallback] = []

        if data_dir is None:
            data_dir = Path(os.environ.get("APPDATA", Path.home())) / "AuroraView"
        self._data_dir = Path(data_dir)
        self._storage_path = self._data_dir / "notifications.json"

        self._load()

    def notify(
        self,
        title: str,
        body: str,
        type: NotificationType = NotificationType.INFO,
        priority: NotificationPriority = NotificationPriority.NORMAL,
        icon: Optional[str] = None,
        image: Optional[str] = None,
        tag: Optional[str] = None,
        actions: Optional[List[NotificationAction]] = None,
        data: Optional[Dict[str, Any]] = None,
        auto_close: Optional[int] = None,
    ) -> Notification:
        """Create and show a notification.

        Args:
            title: Notification title
            body: Notification body
            type: Notification type
            priority: Notification priority
            icon: Optional icon URL
            image: Optional image URL
            tag: Optional tag for grouping
            actions: Optional action buttons
            data: Optional custom data
            auto_close: Auto close after N milliseconds

        Returns:
            Created notification
        """
        notification = Notification(
            id=str(uuid.uuid4())[:8],
            title=title,
            body=body,
            type=type,
            priority=priority,
            icon=icon,
            image=image,
            tag=tag,
            actions=actions or [],
            data=data,
            auto_close=auto_close,
        )

        # Replace existing notification with same tag
        if tag:
            existing = self._find_by_tag(tag)
            if existing:
                del self._notifications[existing.id]

        self._notifications[notification.id] = notification
        self._enforce_max_history()
        self._save()

        # Notify callbacks
        for callback in self._callbacks:
            try:
                callback(notification)
            except Exception:
                pass

        return notification

    def _find_by_tag(self, tag: str) -> Optional[Notification]:
        """Find notification by tag."""
        for n in self._notifications.values():
            if n.tag == tag:
                return n
        return None

    # Convenience methods

    def info(self, title: str, body: str, **kwargs: Any) -> Notification:
        """Show info notification."""
        return self.notify(title, body, type=NotificationType.INFO, **kwargs)

    def success(self, title: str, body: str, **kwargs: Any) -> Notification:
        """Show success notification."""
        return self.notify(title, body, type=NotificationType.SUCCESS, **kwargs)

    def warning(self, title: str, body: str, **kwargs: Any) -> Notification:
        """Show warning notification."""
        return self.notify(title, body, type=NotificationType.WARNING, **kwargs)

    def error(self, title: str, body: str, **kwargs: Any) -> Notification:
        """Show error notification."""
        return self.notify(title, body, type=NotificationType.ERROR, **kwargs)

    def progress(
        self,
        title: str,
        body: str,
        tag: Optional[str] = None,
        **kwargs: Any,
    ) -> Notification:
        """Show progress notification."""
        return self.notify(
            title,
            body,
            type=NotificationType.PROGRESS,
            tag=tag or str(uuid.uuid4())[:8],
            **kwargs,
        )

    def update_progress(self, notification_id: str, body: str) -> Optional[Notification]:
        """Update progress notification body."""
        notification = self._notifications.get(notification_id)
        if notification and notification.type == NotificationType.PROGRESS:
            notification.body = body
            self._save()
            return notification
        return None

    def get(self, notification_id: str) -> Optional[Notification]:
        """Get notification by ID."""
        return self._notifications.get(notification_id)

    def mark_read(self, notification_id: str) -> bool:
        """Mark notification as read."""
        notification = self._notifications.get(notification_id)
        if notification:
            notification.read = True
            self._save()
            return True
        return False

    def mark_all_read(self) -> int:
        """Mark all notifications as read. Returns count."""
        count = 0
        for notification in self._notifications.values():
            if not notification.read:
                notification.read = True
                count += 1
        if count > 0:
            self._save()
        return count

    def dismiss(self, notification_id: str) -> bool:
        """Dismiss (remove) a notification."""
        if notification_id in self._notifications:
            del self._notifications[notification_id]
            self._save()
            return True
        return False

    def dismiss_all(self) -> int:
        """Dismiss all notifications. Returns count."""
        count = len(self._notifications)
        self._notifications.clear()
        self._save()
        return count

    def trigger_action(self, notification_id: str, action_id: str) -> bool:
        """Trigger a notification action."""
        notification = self._notifications.get(notification_id)
        if not notification:
            return False

        # Check if action exists
        if not any(a.id == action_id for a in notification.actions):
            return False

        # Notify action callbacks
        for callback in self._action_callbacks:
            try:
                callback(notification, action_id)
            except Exception:
                pass

        return True

    # Callbacks

    def on_notification(self, callback: NotificationCallback) -> None:
        """Register notification callback."""
        self._callbacks.append(callback)

    def off_notification(self, callback: NotificationCallback) -> None:
        """Unregister notification callback."""
        try:
            self._callbacks.remove(callback)
        except ValueError:
            pass

    def on_action(self, callback: ActionCallback) -> None:
        """Register action callback."""
        self._action_callbacks.append(callback)

    def off_action(self, callback: ActionCallback) -> None:
        """Unregister action callback."""
        try:
            self._action_callbacks.remove(callback)
        except ValueError:
            pass

    # Query methods

    def all(self) -> List[Notification]:
        """Get all notifications."""
        notifications = list(self._notifications.values())
        notifications.sort(key=lambda n: n.timestamp, reverse=True)
        return notifications

    def unread(self) -> List[Notification]:
        """Get unread notifications."""
        return [n for n in self.all() if not n.read]

    def by_type(self, type: NotificationType) -> List[Notification]:
        """Get notifications by type."""
        return [n for n in self.all() if n.type == type]

    def by_tag(self, tag: str) -> List[Notification]:
        """Get notifications by tag."""
        return [n for n in self.all() if n.tag == tag]

    @property
    def count(self) -> int:
        """Get notification count."""
        return len(self._notifications)

    @property
    def unread_count(self) -> int:
        """Get unread notification count."""
        return len(self.unread())

    # Permissions

    def request_permission(self, origin: str = "*") -> PermissionState:
        """Request notification permission.

        In AuroraView, this always grants permission.
        This method exists for Web API compatibility.
        """
        self._permissions[origin] = PermissionState.GRANTED
        return PermissionState.GRANTED

    def get_permission(self, origin: str = "*") -> PermissionState:
        """Get permission state for origin."""
        return self._permissions.get(origin, PermissionState.DEFAULT)

    def revoke_permission(self, origin: str) -> None:
        """Revoke permission for origin."""
        self._permissions[origin] = PermissionState.DENIED

    # Persistence

    def _enforce_max_history(self) -> None:
        """Enforce max history limit."""
        if len(self._notifications) <= self._max_history:
            return
        # Remove oldest notifications
        notifications = list(self._notifications.values())
        notifications.sort(key=lambda n: n.timestamp)
        to_remove = len(self._notifications) - self._max_history
        for notification in notifications[:to_remove]:
            del self._notifications[notification.id]

    def _save(self) -> None:
        """Save notifications to disk."""
        self._data_dir.mkdir(parents=True, exist_ok=True)
        data = {
            "notifications": {k: v.to_dict() for k, v in self._notifications.items()},
            "permissions": {k: v.value for k, v in self._permissions.items()},
        }
        self._storage_path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def _load(self) -> None:
        """Load notifications from disk."""
        if not self._storage_path.exists():
            return
        try:
            data = json.loads(self._storage_path.read_text(encoding="utf-8"))
            self._notifications = {
                k: Notification.from_dict(v) for k, v in data.get("notifications", {}).items()
            }
            self._permissions = {
                k: PermissionState(v) for k, v in data.get("permissions", {}).items()
            }
        except (json.JSONDecodeError, KeyError):
            pass

    def clear_history(self) -> None:
        """Clear notification history."""
        self._notifications.clear()
        self._save()
