"""Settings management for AuroraView.

This module provides user settings functionality:
- Type-safe settings access
- Nested settings with dot notation
- Default values
- Persistence
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

SettingValue = Union[str, int, float, bool, List[Any], Dict[str, Any], None]


class SettingType(Enum):
    """Setting value types."""

    STRING = "string"
    INTEGER = "integer"
    FLOAT = "float"
    BOOLEAN = "boolean"
    LIST = "list"
    OBJECT = "object"


@dataclass
class Setting:
    """A setting definition."""

    key: str
    value: SettingValue
    default: SettingValue
    type: SettingType
    label: str = ""
    description: str = ""
    category: str = "general"
    options: Optional[List[SettingValue]] = None  # For enum-like settings

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "key": self.key,
            "value": self.value,
            "default": self.default,
            "type": self.type.value,
            "label": self.label,
            "description": self.description,
            "category": self.category,
            "options": self.options,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Setting":
        """Create from dictionary."""
        return cls(
            key=data["key"],
            value=data["value"],
            default=data["default"],
            type=SettingType(data.get("type", "string")),
            label=data.get("label", ""),
            description=data.get("description", ""),
            category=data.get("category", "general"),
            options=data.get("options"),
        )


class SettingsManager:
    """Manages application settings with persistence."""

    # Default settings schema
    DEFAULT_SETTINGS: Dict[str, Dict[str, Any]] = {
        "appearance.theme": {
            "default": "system",
            "type": "string",
            "label": "Theme",
            "description": "Application color theme",
            "category": "appearance",
            "options": ["system", "light", "dark"],
        },
        "appearance.font_size": {
            "default": 14,
            "type": "integer",
            "label": "Font Size",
            "description": "Default font size in pixels",
            "category": "appearance",
        },
        "appearance.show_bookmarks_bar": {
            "default": True,
            "type": "boolean",
            "label": "Show Bookmarks Bar",
            "description": "Show bookmarks bar below address bar",
            "category": "appearance",
        },
        "browser.homepage": {
            "default": "about:blank",
            "type": "string",
            "label": "Homepage",
            "description": "Default homepage URL",
            "category": "browser",
        },
        "browser.search_engine": {
            "default": "google",
            "type": "string",
            "label": "Search Engine",
            "description": "Default search engine",
            "category": "browser",
            "options": ["google", "bing", "duckduckgo", "baidu"],
        },
        "browser.new_tab_page": {
            "default": "blank",
            "type": "string",
            "label": "New Tab Page",
            "description": "What to show on new tab",
            "category": "browser",
            "options": ["blank", "homepage", "recent"],
        },
        "privacy.save_history": {
            "default": True,
            "type": "boolean",
            "label": "Save Browsing History",
            "description": "Save pages you visit to history",
            "category": "privacy",
        },
        "privacy.send_dnt": {
            "default": False,
            "type": "boolean",
            "label": "Send Do Not Track",
            "description": "Request websites not to track you",
            "category": "privacy",
        },
        "downloads.default_path": {
            "default": "",
            "type": "string",
            "label": "Download Location",
            "description": "Default download location",
            "category": "downloads",
        },
        "downloads.ask_location": {
            "default": False,
            "type": "boolean",
            "label": "Ask for Download Location",
            "description": "Always ask where to save files",
            "category": "downloads",
        },
        "advanced.developer_mode": {
            "default": False,
            "type": "boolean",
            "label": "Developer Mode",
            "description": "Enable developer features",
            "category": "advanced",
        },
        "advanced.enable_devtools": {
            "default": True,
            "type": "boolean",
            "label": "Enable DevTools",
            "description": "Allow opening developer tools",
            "category": "advanced",
        },
    }

    def __init__(self, data_dir: Optional[Path] = None):
        """Initialize settings manager.

        Args:
            data_dir: Directory for storing settings.
        """
        self._settings: Dict[str, Setting] = {}
        self._modified_at: Optional[datetime] = None

        if data_dir is None:
            data_dir = Path(os.environ.get("APPDATA", Path.home())) / "AuroraView"
        self._data_dir = Path(data_dir)
        self._storage_path = self._data_dir / "settings.json"

        self._init_defaults()
        self._load()

    def _init_defaults(self) -> None:
        """Initialize default settings."""
        for key, schema in self.DEFAULT_SETTINGS.items():
            self._settings[key] = Setting(
                key=key,
                value=schema["default"],
                default=schema["default"],
                type=SettingType(schema.get("type", "string")),
                label=schema.get("label", key),
                description=schema.get("description", ""),
                category=schema.get("category", "general"),
                options=schema.get("options"),
            )

    def get(self, key: str, default: SettingValue = None) -> SettingValue:
        """Get a setting value.

        Args:
            key: Setting key (supports dot notation)
            default: Default value if not found

        Returns:
            Setting value
        """
        setting = self._settings.get(key)
        if setting:
            return setting.value
        return default

    def set(self, key: str, value: SettingValue) -> None:
        """Set a setting value.

        Args:
            key: Setting key
            value: Setting value
        """
        if key in self._settings:
            self._settings[key].value = value
        else:
            # Create new setting with inferred type
            setting_type = self._infer_type(value)
            self._settings[key] = Setting(
                key=key,
                value=value,
                default=value,
                type=setting_type,
            )
        self._modified_at = datetime.now()
        self._save()

    def _infer_type(self, value: SettingValue) -> SettingType:
        """Infer setting type from value."""
        if isinstance(value, bool):
            return SettingType.BOOLEAN
        if isinstance(value, int):
            return SettingType.INTEGER
        if isinstance(value, float):
            return SettingType.FLOAT
        if isinstance(value, list):
            return SettingType.LIST
        if isinstance(value, dict):
            return SettingType.OBJECT
        return SettingType.STRING

    def reset(self, key: str) -> None:
        """Reset a setting to default."""
        if key in self._settings:
            self._settings[key].value = self._settings[key].default
            self._modified_at = datetime.now()
            self._save()

    def reset_all(self) -> None:
        """Reset all settings to defaults."""
        for setting in self._settings.values():
            setting.value = setting.default
        self._modified_at = datetime.now()
        self._save()

    def all(self) -> Dict[str, SettingValue]:
        """Get all settings as a dictionary."""
        return {k: s.value for k, s in self._settings.items()}

    def by_category(self, category: str) -> List[Setting]:
        """Get settings by category."""
        return [s for s in self._settings.values() if s.category == category]

    def categories(self) -> List[str]:
        """Get all categories."""
        return sorted({s.category for s in self._settings.values()})

    def get_setting(self, key: str) -> Optional[Setting]:
        """Get setting object (with metadata)."""
        return self._settings.get(key)

    def define(
        self,
        key: str,
        default: SettingValue,
        type: Optional[SettingType] = None,
        label: str = "",
        description: str = "",
        category: str = "general",
        options: Optional[List[SettingValue]] = None,
    ) -> None:
        """Define a new setting.

        Args:
            key: Setting key
            default: Default value
            type: Value type (inferred if not provided)
            label: Display label
            description: Setting description
            category: Setting category
            options: Valid options (for enum-like settings)
        """
        if type is None:
            type = self._infer_type(default)

        # Keep existing value if setting already exists
        existing_value = self._settings.get(key)
        value = existing_value.value if existing_value else default

        self._settings[key] = Setting(
            key=key,
            value=value,
            default=default,
            type=type,
            label=label,
            description=description,
            category=category,
            options=options,
        )

    def has(self, key: str) -> bool:
        """Check if setting exists."""
        return key in self._settings

    def remove(self, key: str) -> bool:
        """Remove a custom setting."""
        if key in self._settings and key not in self.DEFAULT_SETTINGS:
            del self._settings[key]
            self._save()
            return True
        return False

    # Persistence

    def _save(self) -> None:
        """Save settings to disk."""
        self._data_dir.mkdir(parents=True, exist_ok=True)

        # Only save non-default values
        data = {}
        for key, setting in self._settings.items():
            if setting.value != setting.default:
                data[key] = setting.value

        self._storage_path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def _load(self) -> None:
        """Load settings from disk."""
        if not self._storage_path.exists():
            return
        try:
            data = json.loads(self._storage_path.read_text(encoding="utf-8"))
            for key, value in data.items():
                if key in self._settings:
                    self._settings[key].value = value
                else:
                    # Unknown setting - create it
                    self._settings[key] = Setting(
                        key=key,
                        value=value,
                        default=value,
                        type=self._infer_type(value),
                    )
        except (json.JSONDecodeError, KeyError):
            pass

    def export_json(self) -> str:
        """Export all settings to JSON."""
        data = {k: s.to_dict() for k, s in self._settings.items()}
        return json.dumps(data, indent=2)

    def import_json(self, json_str: str) -> int:
        """Import settings from JSON. Returns count of imported items."""
        data = json.loads(json_str)
        count = 0
        for key, s_data in data.items():
            if isinstance(s_data, dict) and "value" in s_data:
                # Full setting object
                self._settings[key] = Setting.from_dict(s_data)
            else:
                # Just value
                self.set(key, s_data)
            count += 1
        self._save()
        return count
