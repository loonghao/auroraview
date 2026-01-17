"""Download management for AuroraView.

This module provides download functionality:
- Track download progress
- Manage download queue
- Resume/pause downloads
- Download history
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


class DownloadState(Enum):
    """Download state."""

    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    PAUSED = "paused"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class DownloadItem:
    """A download item."""

    id: str
    url: str
    filename: str
    save_path: str
    state: DownloadState = DownloadState.PENDING
    total_bytes: int = 0
    received_bytes: int = 0
    start_time: datetime = field(default_factory=datetime.now)
    end_time: Optional[datetime] = None
    error: Optional[str] = None
    mime_type: Optional[str] = None

    @property
    def progress(self) -> float:
        """Get download progress (0-100)."""
        if self.total_bytes <= 0:
            return 0.0
        return min(100.0, (self.received_bytes / self.total_bytes) * 100)

    @property
    def is_active(self) -> bool:
        """Check if download is active."""
        return self.state in (DownloadState.PENDING, DownloadState.IN_PROGRESS)

    @property
    def can_resume(self) -> bool:
        """Check if download can be resumed."""
        return self.state == DownloadState.PAUSED

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "url": self.url,
            "filename": self.filename,
            "save_path": self.save_path,
            "state": self.state.value,
            "total_bytes": self.total_bytes,
            "received_bytes": self.received_bytes,
            "start_time": self.start_time.isoformat(),
            "end_time": self.end_time.isoformat() if self.end_time else None,
            "error": self.error,
            "mime_type": self.mime_type,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "DownloadItem":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            url=data["url"],
            filename=data["filename"],
            save_path=data["save_path"],
            state=DownloadState(data.get("state", "pending")),
            total_bytes=data.get("total_bytes", 0),
            received_bytes=data.get("received_bytes", 0),
            start_time=datetime.fromisoformat(data["start_time"])
            if "start_time" in data
            else datetime.now(),
            end_time=datetime.fromisoformat(data["end_time"]) if data.get("end_time") else None,
            error=data.get("error"),
            mime_type=data.get("mime_type"),
        )


# Type alias for progress callback
ProgressCallback = Callable[[DownloadItem], None]


class DownloadManager:
    """Manages downloads with persistence."""

    DEFAULT_DOWNLOAD_DIR = "Downloads"

    def __init__(
        self,
        data_dir: Optional[Path] = None,
        download_dir: Optional[Path] = None,
    ):
        """Initialize download manager.

        Args:
            data_dir: Directory for storing download metadata.
            download_dir: Default download directory.
        """
        self._downloads: Dict[str, DownloadItem] = {}
        self._callbacks: Dict[str, List[ProgressCallback]] = {}

        if data_dir is None:
            data_dir = Path(os.environ.get("APPDATA", Path.home())) / "AuroraView"
        self._data_dir = Path(data_dir)
        self._storage_path = self._data_dir / "downloads.json"

        if download_dir is None:
            download_dir = Path.home() / self.DEFAULT_DOWNLOAD_DIR
        self._download_dir = Path(download_dir)

        self._load()

    @property
    def download_dir(self) -> Path:
        """Get default download directory."""
        return self._download_dir

    @download_dir.setter
    def download_dir(self, path: Path) -> None:
        """Set default download directory."""
        self._download_dir = Path(path)

    def create(
        self,
        url: str,
        filename: Optional[str] = None,
        save_path: Optional[str] = None,
        mime_type: Optional[str] = None,
    ) -> DownloadItem:
        """Create a new download.

        Args:
            url: Download URL
            filename: Optional filename (extracted from URL if not provided)
            save_path: Optional save path (uses download_dir if not provided)
            mime_type: Optional MIME type

        Returns:
            Created download item
        """
        if filename is None:
            # Extract filename from URL
            from urllib.parse import unquote, urlparse

            parsed = urlparse(url)
            filename = unquote(parsed.path.split("/")[-1]) or "download"

        if save_path is None:
            save_path = str(self._download_dir / filename)

        download = DownloadItem(
            id=str(uuid.uuid4())[:8],
            url=url,
            filename=filename,
            save_path=save_path,
            mime_type=mime_type,
        )
        self._downloads[download.id] = download
        self._save()
        return download

    def get(self, download_id: str) -> Optional[DownloadItem]:
        """Get download by ID."""
        return self._downloads.get(download_id)

    def start(self, download_id: str) -> bool:
        """Start a download."""
        download = self._downloads.get(download_id)
        if not download:
            return False
        if download.state not in (DownloadState.PENDING, DownloadState.PAUSED):
            return False

        download.state = DownloadState.IN_PROGRESS
        download.start_time = datetime.now()
        self._save()
        self._notify(download)
        return True

    def pause(self, download_id: str) -> bool:
        """Pause a download."""
        download = self._downloads.get(download_id)
        if not download:
            return False
        if download.state != DownloadState.IN_PROGRESS:
            return False

        download.state = DownloadState.PAUSED
        self._save()
        self._notify(download)
        return True

    def resume(self, download_id: str) -> bool:
        """Resume a paused download."""
        return self.start(download_id)

    def cancel(self, download_id: str) -> bool:
        """Cancel a download."""
        download = self._downloads.get(download_id)
        if not download:
            return False
        if download.state == DownloadState.COMPLETED:
            return False

        download.state = DownloadState.CANCELLED
        download.end_time = datetime.now()
        self._save()
        self._notify(download)
        return True

    def update_progress(
        self,
        download_id: str,
        received_bytes: int,
        total_bytes: Optional[int] = None,
    ) -> None:
        """Update download progress."""
        download = self._downloads.get(download_id)
        if not download:
            return

        download.received_bytes = received_bytes
        if total_bytes is not None:
            download.total_bytes = total_bytes

        self._notify(download)

    def complete(self, download_id: str) -> bool:
        """Mark download as completed."""
        download = self._downloads.get(download_id)
        if not download:
            return False

        download.state = DownloadState.COMPLETED
        download.end_time = datetime.now()
        download.received_bytes = download.total_bytes
        self._save()
        self._notify(download)
        return True

    def fail(self, download_id: str, error: str) -> bool:
        """Mark download as failed."""
        download = self._downloads.get(download_id)
        if not download:
            return False

        download.state = DownloadState.FAILED
        download.end_time = datetime.now()
        download.error = error
        self._save()
        self._notify(download)
        return True

    def remove(self, download_id: str, delete_file: bool = False) -> bool:
        """Remove download from list."""
        download = self._downloads.get(download_id)
        if not download:
            return False

        if delete_file and download.state == DownloadState.COMPLETED:
            try:
                Path(download.save_path).unlink(missing_ok=True)
            except OSError:
                pass

        del self._downloads[download_id]
        self._callbacks.pop(download_id, None)
        self._save()
        return True

    def clear_completed(self) -> int:
        """Clear completed downloads. Returns count removed."""
        to_remove = [
            did for did, d in self._downloads.items() if d.state == DownloadState.COMPLETED
        ]
        for did in to_remove:
            del self._downloads[did]
            self._callbacks.pop(did, None)
        if to_remove:
            self._save()
        return len(to_remove)

    # Callbacks

    def on_progress(self, download_id: str, callback: ProgressCallback) -> None:
        """Register progress callback."""
        if download_id not in self._callbacks:
            self._callbacks[download_id] = []
        self._callbacks[download_id].append(callback)

    def off_progress(self, download_id: str, callback: ProgressCallback) -> None:
        """Unregister progress callback."""
        if download_id in self._callbacks:
            try:
                self._callbacks[download_id].remove(callback)
            except ValueError:
                pass

    def _notify(self, download: DownloadItem) -> None:
        """Notify callbacks."""
        for callback in self._callbacks.get(download.id, []):
            try:
                callback(download)
            except Exception:
                pass

    # Query methods

    def all(self) -> List[DownloadItem]:
        """Get all downloads."""
        return list(self._downloads.values())

    def active(self) -> List[DownloadItem]:
        """Get active downloads."""
        return [d for d in self._downloads.values() if d.is_active]

    def completed(self) -> List[DownloadItem]:
        """Get completed downloads."""
        return [d for d in self._downloads.values() if d.state == DownloadState.COMPLETED]

    def recent(self, limit: int = 20) -> List[DownloadItem]:
        """Get recent downloads."""
        downloads = list(self._downloads.values())
        downloads.sort(key=lambda d: d.start_time, reverse=True)
        return downloads[:limit]

    @property
    def count(self) -> int:
        """Get download count."""
        return len(self._downloads)

    @property
    def active_count(self) -> int:
        """Get active download count."""
        return len(self.active())

    # Persistence

    def _save(self) -> None:
        """Save downloads to disk."""
        self._data_dir.mkdir(parents=True, exist_ok=True)
        data = {k: v.to_dict() for k, v in self._downloads.items()}
        self._storage_path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def _load(self) -> None:
        """Load downloads from disk."""
        if not self._storage_path.exists():
            return
        try:
            data = json.loads(self._storage_path.read_text(encoding="utf-8"))
            self._downloads = {k: DownloadItem.from_dict(v) for k, v in data.items()}
            # Reset in-progress downloads to paused
            for download in self._downloads.values():
                if download.state == DownloadState.IN_PROGRESS:
                    download.state = DownloadState.PAUSED
        except (json.JSONDecodeError, KeyError):
            pass
