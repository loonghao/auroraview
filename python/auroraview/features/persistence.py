"""Persistence mixin for feature managers.

This module provides a base mixin for JSON-based persistence used by
BookmarkManager, HistoryManager, DownloadManager, etc.
"""

from __future__ import annotations

import json
import logging
from abc import abstractmethod
from pathlib import Path
from typing import Any, Dict, Generic, Type, TypeVar

logger = logging.getLogger(__name__)

T = TypeVar("T")


class PersistenceMixin(Generic[T]):
    """Mixin for JSON-based persistence.

    Provides save/load/export/import functionality for managers that store
    data in JSON files.

    Subclasses must define:
    - _storage_path: Path to storage file
    - _data_dir: Directory for data storage
    - A dataclass type for items (e.g., Bookmark, HistoryEntry, DownloadItem)
    """

    def __init__(self, data_dir: Path, storage_filename: str):
        """Initialize persistence.

        Args:
            data_dir: Directory for storing data
            storage_filename: Name of storage file (e.g., "bookmarks.json")
        """
        self._data_dir = Path(data_dir)
        self._storage_path = self._data_dir / storage_filename

    def _save(self, data: Dict[str, T]) -> None:
        """Save data to disk.

        Args:
            data: Dictionary of items to save
        """
        self._data_dir.mkdir(parents=True, exist_ok=True)
        serialized = {k: self._item_to_dict(v) for k, v in data.items()}
        self._storage_path.write_text(json.dumps(serialized, indent=2), encoding="utf-8")
        logger.debug("Saved %d items to %s", len(data), self._storage_path)

    def _load(
        self,
        item_cls: Type[T],
        on_load_item: Any | None = None,
    ) -> Dict[str, T]:
        """Load data from disk.

        Args:
            item_cls: Dataclass type with from_dict classmethod
            on_load_item: Optional callback to process each loaded item

        Returns:
            Dictionary of loaded items
        """
        if not self._storage_path.exists():
            return {}

        try:
            text = self._storage_path.read_text(encoding="utf-8")
            data = json.loads(text)
            result = {}
            for k, v in data.items():
                try:
                    item = item_cls.from_dict(v)
                    if on_load_item:
                        on_load_item(item)
                    result[k] = item
                except Exception as e:
                    logger.warning("Failed to load item %s from %s: %s", k, self._storage_path, e)
            logger.debug("Loaded %d items from %s", len(result), self._storage_path)
            return result
        except (json.JSONDecodeError, KeyError) as e:
            logger.error("Failed to load from %s: %s", self._storage_path, e)
            return {}

    def export_json(self, data: Dict[str, T]) -> str:
        """Export data to JSON string.

        Args:
            data: Dictionary of items to export

        Returns:
            JSON string
        """
        serialized = [self._item_to_dict(v) for v in data.values()]
        return json.dumps(serialized, indent=2)

    def import_json(
        self,
        json_str: str,
        item_cls: Type[T],
        existing_data: Dict[str, T],
        on_import_item: Any | None = None,
    ) -> int:
        """Import data from JSON string.

        Args:
            json_str: JSON string to import
            item_cls: Dataclass type with from_dict classmethod
            existing_data: Existing data dictionary (will be updated)
            on_import_item: Optional callback to process each imported item

        Returns:
            Number of imported items
        """
        data = json.loads(json_str)
        count = 0
        for item_data in data:
            try:
                item = item_cls.from_dict(item_data)
                existing_data[item.id] = item
                if on_import_item:
                    on_import_item(item)
                count += 1
            except Exception as e:
                logger.warning("Failed to import item: %s", e)
        return count

    @abstractmethod
    def _item_to_dict(self, item: T) -> Dict[str, Any]:
        """Convert item to dictionary.

        Args:
            item: Item to convert

        Returns:
            Dictionary representation
        """
        pass
