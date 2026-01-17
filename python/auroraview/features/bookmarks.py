"""Bookmark management for AuroraView.

This module provides bookmark functionality similar to browser bookmarks:
- Add, remove, update bookmarks
- Organize bookmarks in folders
- Search bookmarks
- Import/export bookmarks
"""

from __future__ import annotations

import json
import os
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional


@dataclass
class Bookmark:
    """A bookmark entry."""

    id: str
    title: str
    url: str
    favicon: Optional[str] = None
    parent_id: Optional[str] = None
    position: int = 0
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)
    tags: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "title": self.title,
            "url": self.url,
            "favicon": self.favicon,
            "parent_id": self.parent_id,
            "position": self.position,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
            "tags": self.tags,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Bookmark":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            title=data["title"],
            url=data["url"],
            favicon=data.get("favicon"),
            parent_id=data.get("parent_id"),
            position=data.get("position", 0),
            created_at=datetime.fromisoformat(data["created_at"])
            if "created_at" in data
            else datetime.now(),
            updated_at=datetime.fromisoformat(data["updated_at"])
            if "updated_at" in data
            else datetime.now(),
            tags=data.get("tags", []),
        )


@dataclass
class BookmarkFolder:
    """A bookmark folder."""

    id: str
    name: str
    parent_id: Optional[str] = None
    position: int = 0
    created_at: datetime = field(default_factory=datetime.now)
    is_special: bool = False  # For bookmarks bar, other bookmarks

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "name": self.name,
            "parent_id": self.parent_id,
            "position": self.position,
            "created_at": self.created_at.isoformat(),
            "is_special": self.is_special,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BookmarkFolder":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            name=data["name"],
            parent_id=data.get("parent_id"),
            position=data.get("position", 0),
            created_at=datetime.fromisoformat(data["created_at"])
            if "created_at" in data
            else datetime.now(),
            is_special=data.get("is_special", False),
        )


# Special folder IDs
BOOKMARKS_BAR_ID = "bookmarks_bar"
OTHER_BOOKMARKS_ID = "other_bookmarks"


class BookmarkManager:
    """Manages bookmarks with persistence."""

    def __init__(self, data_dir: Optional[Path] = None):
        """Initialize bookmark manager.

        Args:
            data_dir: Directory for storing bookmarks. If None, uses default.
        """
        self._bookmarks: Dict[str, Bookmark] = {}
        self._folders: Dict[str, BookmarkFolder] = {}

        if data_dir is None:
            data_dir = Path(os.environ.get("APPDATA", Path.home())) / "AuroraView"
        self._data_dir = Path(data_dir)
        self._storage_path = self._data_dir / "bookmarks.json"

        self._init_special_folders()
        self._load()

    def _init_special_folders(self) -> None:
        """Initialize special folders."""
        if BOOKMARKS_BAR_ID not in self._folders:
            self._folders[BOOKMARKS_BAR_ID] = BookmarkFolder(
                id=BOOKMARKS_BAR_ID,
                name="Bookmarks Bar",
                is_special=True,
            )
        if OTHER_BOOKMARKS_ID not in self._folders:
            self._folders[OTHER_BOOKMARKS_ID] = BookmarkFolder(
                id=OTHER_BOOKMARKS_ID,
                name="Other Bookmarks",
                is_special=True,
            )

    def add(
        self,
        url: str,
        title: str,
        folder_id: Optional[str] = None,
        favicon: Optional[str] = None,
        tags: Optional[List[str]] = None,
    ) -> Bookmark:
        """Add a bookmark.

        Args:
            url: Bookmark URL
            title: Bookmark title
            folder_id: Parent folder ID (defaults to bookmarks bar)
            favicon: Favicon URL/data URI
            tags: Optional tags

        Returns:
            Created bookmark
        """
        bookmark_id = str(uuid.uuid4())[:8]
        folder_id = folder_id or BOOKMARKS_BAR_ID

        # Get next position in folder
        siblings = self.in_folder(folder_id)
        position = max((b.position for b in siblings), default=-1) + 1

        bookmark = Bookmark(
            id=bookmark_id,
            title=title,
            url=url,
            favicon=favicon,
            parent_id=folder_id,
            position=position,
            tags=tags or [],
        )
        self._bookmarks[bookmark_id] = bookmark
        self._save()
        return bookmark

    def get(self, bookmark_id: str) -> Optional[Bookmark]:
        """Get a bookmark by ID."""
        return self._bookmarks.get(bookmark_id)

    def update(
        self,
        bookmark_id: str,
        title: Optional[str] = None,
        url: Optional[str] = None,
        favicon: Optional[str] = None,
        tags: Optional[List[str]] = None,
    ) -> Optional[Bookmark]:
        """Update a bookmark."""
        bookmark = self._bookmarks.get(bookmark_id)
        if not bookmark:
            return None

        if title is not None:
            bookmark.title = title
        if url is not None:
            bookmark.url = url
        if favicon is not None:
            bookmark.favicon = favicon
        if tags is not None:
            bookmark.tags = tags
        bookmark.updated_at = datetime.now()

        self._save()
        return bookmark

    def remove(self, bookmark_id: str) -> bool:
        """Remove a bookmark."""
        if bookmark_id in self._bookmarks:
            del self._bookmarks[bookmark_id]
            self._save()
            return True
        return False

    def is_bookmarked(self, url: str) -> bool:
        """Check if URL is bookmarked."""
        return any(b.url == url for b in self._bookmarks.values())

    def find_by_url(self, url: str) -> Optional[Bookmark]:
        """Find bookmark by URL."""
        for bookmark in self._bookmarks.values():
            if bookmark.url == url:
                return bookmark
        return None

    def all(self) -> List[Bookmark]:
        """Get all bookmarks."""
        return list(self._bookmarks.values())

    def in_folder(self, folder_id: str) -> List[Bookmark]:
        """Get bookmarks in a folder."""
        bookmarks = [b for b in self._bookmarks.values() if b.parent_id == folder_id]
        return sorted(bookmarks, key=lambda b: b.position)

    def search(self, query: str) -> List[Bookmark]:
        """Search bookmarks."""
        query = query.lower()
        results = []
        for bookmark in self._bookmarks.values():
            if (
                query in bookmark.title.lower()
                or query in bookmark.url.lower()
                or any(query in tag.lower() for tag in bookmark.tags)
            ):
                results.append(bookmark)
        return results

    def move_to_folder(self, bookmark_id: str, folder_id: str) -> bool:
        """Move bookmark to folder."""
        bookmark = self._bookmarks.get(bookmark_id)
        if not bookmark or folder_id not in self._folders:
            return False

        bookmark.parent_id = folder_id
        bookmark.updated_at = datetime.now()
        self._save()
        return True

    # Folder operations

    def create_folder(self, name: str, parent_id: Optional[str] = None) -> BookmarkFolder:
        """Create a folder."""
        folder_id = str(uuid.uuid4())[:8]
        folder = BookmarkFolder(
            id=folder_id,
            name=name,
            parent_id=parent_id,
        )
        self._folders[folder_id] = folder
        self._save()
        return folder

    def get_folder(self, folder_id: str) -> Optional[BookmarkFolder]:
        """Get a folder by ID."""
        return self._folders.get(folder_id)

    def all_folders(self) -> List[BookmarkFolder]:
        """Get all folders."""
        return list(self._folders.values())

    def delete_folder(self, folder_id: str, delete_contents: bool = False) -> bool:
        """Delete a folder."""
        folder = self._folders.get(folder_id)
        if not folder or folder.is_special:
            return False

        if delete_contents:
            # Delete all bookmarks in folder
            to_delete = [bid for bid, b in self._bookmarks.items() if b.parent_id == folder_id]
            for bid in to_delete:
                del self._bookmarks[bid]
        else:
            # Move contents to root
            for bookmark in self._bookmarks.values():
                if bookmark.parent_id == folder_id:
                    bookmark.parent_id = None

        del self._folders[folder_id]
        self._save()
        return True

    def rename_folder(self, folder_id: str, name: str) -> bool:
        """Rename a folder."""
        folder = self._folders.get(folder_id)
        if not folder or folder.is_special:
            return False
        folder.name = name
        self._save()
        return True

    # Persistence

    def _save(self) -> None:
        """Save bookmarks to disk."""
        self._data_dir.mkdir(parents=True, exist_ok=True)
        data = {
            "bookmarks": {k: v.to_dict() for k, v in self._bookmarks.items()},
            "folders": {k: v.to_dict() for k, v in self._folders.items()},
        }
        self._storage_path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def _load(self) -> None:
        """Load bookmarks from disk."""
        if not self._storage_path.exists():
            return
        try:
            data = json.loads(self._storage_path.read_text(encoding="utf-8"))
            self._bookmarks = {
                k: Bookmark.from_dict(v) for k, v in data.get("bookmarks", {}).items()
            }
            self._folders = {
                k: BookmarkFolder.from_dict(v) for k, v in data.get("folders", {}).items()
            }
            self._init_special_folders()
        except (json.JSONDecodeError, KeyError):
            pass

    def export_json(self) -> str:
        """Export bookmarks to JSON."""
        data = {
            "bookmarks": [b.to_dict() for b in self._bookmarks.values()],
            "folders": [f.to_dict() for f in self._folders.values()],
        }
        return json.dumps(data, indent=2)

    def import_json(self, json_str: str) -> int:
        """Import bookmarks from JSON. Returns count of imported items."""
        data = json.loads(json_str)
        count = 0
        for b_data in data.get("bookmarks", []):
            bookmark = Bookmark.from_dict(b_data)
            self._bookmarks[bookmark.id] = bookmark
            count += 1
        for f_data in data.get("folders", []):
            folder = BookmarkFolder.from_dict(f_data)
            if not folder.is_special:
                self._folders[folder.id] = folder
        self._save()
        return count

    def clear(self) -> None:
        """Clear all bookmarks (keeps special folders)."""
        self._bookmarks.clear()
        self._folders = {k: v for k, v in self._folders.items() if v.is_special}
        self._save()

    @property
    def count(self) -> int:
        """Get bookmark count."""
        return len(self._bookmarks)

    @property
    def folder_count(self) -> int:
        """Get folder count."""
        return len(self._folders)
