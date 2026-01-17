"""History management for AuroraView.

This module provides browsing history functionality:
- Record page visits
- Search history
- Get recent/frequent pages
- Clear history
"""

from __future__ import annotations

import json
import os
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, Dict, List, Optional
from urllib.parse import urlparse


@dataclass
class HistoryEntry:
    """A history entry."""

    id: str
    url: str
    title: str
    visit_count: int = 1
    typed_count: int = 0  # Times user typed URL directly
    last_visit: datetime = field(default_factory=datetime.now)
    first_visit: datetime = field(default_factory=datetime.now)
    favicon: Optional[str] = None

    def domain(self) -> Optional[str]:
        """Get domain from URL."""
        try:
            parsed = urlparse(self.url)
            return parsed.netloc
        except Exception:
            return None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "url": self.url,
            "title": self.title,
            "visit_count": self.visit_count,
            "typed_count": self.typed_count,
            "last_visit": self.last_visit.isoformat(),
            "first_visit": self.first_visit.isoformat(),
            "favicon": self.favicon,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "HistoryEntry":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            url=data["url"],
            title=data["title"],
            visit_count=data.get("visit_count", 1),
            typed_count=data.get("typed_count", 0),
            last_visit=datetime.fromisoformat(data["last_visit"])
            if "last_visit" in data
            else datetime.now(),
            first_visit=datetime.fromisoformat(data["first_visit"])
            if "first_visit" in data
            else datetime.now(),
            favicon=data.get("favicon"),
        )


class HistoryManager:
    """Manages browsing history with persistence."""

    DEFAULT_MAX_ENTRIES = 10000

    def __init__(
        self,
        data_dir: Optional[Path] = None,
        max_entries: int = DEFAULT_MAX_ENTRIES,
    ):
        """Initialize history manager.

        Args:
            data_dir: Directory for storing history. If None, uses default.
            max_entries: Maximum number of entries to keep.
        """
        self._entries: Dict[str, HistoryEntry] = {}
        self._max_entries = max_entries

        if data_dir is None:
            data_dir = Path(os.environ.get("APPDATA", Path.home())) / "AuroraView"
        self._data_dir = Path(data_dir)
        self._storage_path = self._data_dir / "history.json"

        self._load()

    def visit(
        self,
        url: str,
        title: str,
        favicon: Optional[str] = None,
        typed: bool = False,
    ) -> HistoryEntry:
        """Record a page visit.

        Args:
            url: Page URL
            title: Page title
            favicon: Optional favicon
            typed: Whether user typed URL directly

        Returns:
            History entry
        """
        # Check if entry exists
        existing = self._find_by_url(url)
        if existing:
            existing.visit_count += 1
            if typed:
                existing.typed_count += 1
            existing.last_visit = datetime.now()
            if title:
                existing.title = title
            if favicon:
                existing.favicon = favicon
            self._save()
            return existing

        # Create new entry
        entry = HistoryEntry(
            id=str(uuid.uuid4())[:8],
            url=url,
            title=title or url,
            typed_count=1 if typed else 0,
            favicon=favicon,
        )
        self._entries[entry.id] = entry

        # Enforce max entries
        self._enforce_max_entries()
        self._save()
        return entry

    def get(self, entry_id: str) -> Optional[HistoryEntry]:
        """Get entry by ID."""
        return self._entries.get(entry_id)

    def _find_by_url(self, url: str) -> Optional[HistoryEntry]:
        """Find entry by URL."""
        for entry in self._entries.values():
            if entry.url == url:
                return entry
        return None

    def get_by_url(self, url: str) -> Optional[HistoryEntry]:
        """Get entry by URL."""
        return self._find_by_url(url)

    def delete(self, entry_id: str) -> bool:
        """Delete a history entry."""
        if entry_id in self._entries:
            del self._entries[entry_id]
            self._save()
            return True
        return False

    def delete_url(self, url: str) -> bool:
        """Delete all entries for a URL."""
        to_delete = [eid for eid, e in self._entries.items() if e.url == url]
        for eid in to_delete:
            del self._entries[eid]
        if to_delete:
            self._save()
        return len(to_delete) > 0

    def recent(self, limit: int = 20) -> List[HistoryEntry]:
        """Get recent history entries."""
        entries = list(self._entries.values())
        entries.sort(key=lambda e: e.last_visit, reverse=True)
        return entries[:limit]

    def frequent(self, limit: int = 10) -> List[HistoryEntry]:
        """Get frequently visited sites."""
        entries = list(self._entries.values())
        entries.sort(key=lambda e: e.visit_count, reverse=True)
        return entries[:limit]

    def search(self, query: str, limit: int = 50) -> List[HistoryEntry]:
        """Search history."""
        query = query.lower()
        results = []
        for entry in self._entries.values():
            if query in entry.title.lower() or query in entry.url.lower():
                results.append(entry)
        # Sort by relevance (visit count + recency)
        results.sort(
            key=lambda e: (
                e.visit_count * 0.3 + (datetime.now() - e.last_visit).total_seconds() * -0.00001
            ),
            reverse=True,
        )
        return results[:limit]

    def by_domain(self, domain: str) -> List[HistoryEntry]:
        """Get entries by domain."""
        return [e for e in self._entries.values() if e.domain() == domain]

    def today(self) -> List[HistoryEntry]:
        """Get today's history."""
        cutoff = datetime.now() - timedelta(days=1)
        entries = [e for e in self._entries.values() if e.last_visit >= cutoff]
        entries.sort(key=lambda e: e.last_visit, reverse=True)
        return entries

    def this_week(self) -> List[HistoryEntry]:
        """Get this week's history."""
        cutoff = datetime.now() - timedelta(weeks=1)
        entries = [e for e in self._entries.values() if e.last_visit >= cutoff]
        entries.sort(key=lambda e: e.last_visit, reverse=True)
        return entries

    def this_month(self) -> List[HistoryEntry]:
        """Get this month's history."""
        cutoff = datetime.now() - timedelta(days=30)
        entries = [e for e in self._entries.values() if e.last_visit >= cutoff]
        entries.sort(key=lambda e: e.last_visit, reverse=True)
        return entries

    def delete_older_than(self, days: int) -> int:
        """Delete entries older than specified days."""
        cutoff = datetime.now() - timedelta(days=days)
        to_delete = [eid for eid, e in self._entries.items() if e.last_visit < cutoff]
        for eid in to_delete:
            del self._entries[eid]
        if to_delete:
            self._save()
        return len(to_delete)

    def delete_domain(self, domain: str) -> int:
        """Delete all entries for a domain."""
        to_delete = [eid for eid, e in self._entries.items() if e.domain() == domain]
        for eid in to_delete:
            del self._entries[eid]
        if to_delete:
            self._save()
        return len(to_delete)

    def clear(self) -> None:
        """Clear all history."""
        self._entries.clear()
        self._save()

    def all(self) -> List[HistoryEntry]:
        """Get all entries."""
        return list(self._entries.values())

    @property
    def count(self) -> int:
        """Get entry count."""
        return len(self._entries)

    def _enforce_max_entries(self) -> None:
        """Enforce max entries limit."""
        if len(self._entries) <= self._max_entries:
            return
        # Remove oldest entries
        entries = list(self._entries.values())
        entries.sort(key=lambda e: e.last_visit)
        to_remove = len(self._entries) - self._max_entries
        for entry in entries[:to_remove]:
            del self._entries[entry.id]

    # Persistence

    def _save(self) -> None:
        """Save history to disk."""
        self._data_dir.mkdir(parents=True, exist_ok=True)
        data = {k: v.to_dict() for k, v in self._entries.items()}
        self._storage_path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def _load(self) -> None:
        """Load history from disk."""
        if not self._storage_path.exists():
            return
        try:
            data = json.loads(self._storage_path.read_text(encoding="utf-8"))
            self._entries = {k: HistoryEntry.from_dict(v) for k, v in data.items()}
        except (json.JSONDecodeError, KeyError):
            pass

    def export_json(self) -> str:
        """Export history to JSON."""
        return json.dumps([e.to_dict() for e in self._entries.values()], indent=2)

    def import_json(self, json_str: str) -> int:
        """Import history from JSON. Returns count of imported items."""
        data = json.loads(json_str)
        count = 0
        for e_data in data:
            entry = HistoryEntry.from_dict(e_data)
            self._entries[entry.id] = entry
            count += 1
        self._save()
        return count
