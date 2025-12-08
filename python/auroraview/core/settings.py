"""WebView settings management.

This module provides a unified interface for managing WebView settings,
inspired by Qt WebView's QWebViewSettings pattern.

Example:
    >>> from auroraview import WebView
    >>> webview = WebView.create("My App")
    >>> webview.settings.javascript_enabled = True
    >>> webview.settings.dev_tools_enabled = True
    >>> print(webview.settings.user_agent)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class WebViewSettings:
    """WebView settings configuration.

    This class provides a unified interface for managing WebView settings
    across different backends. Settings can be modified before or after
    WebView creation.

    Attributes:
        javascript_enabled: Enable JavaScript execution (default: True)
        local_storage_enabled: Enable local storage (default: True)
        dev_tools_enabled: Enable developer tools (default: True)
        context_menu_enabled: Enable native context menu (default: True)
        allow_file_access: Allow file:// protocol access (default: False)
        user_agent: Custom user agent string (default: None)
        background_color: Background color in hex format (default: None)
        zoom_level: Zoom level as percentage (default: 100)
        minimum_font_size: Minimum font size in pixels (default: 0)
        default_font_size: Default font size in pixels (default: 16)
        default_encoding: Default text encoding (default: "UTF-8")
    """

    javascript_enabled: bool = True
    local_storage_enabled: bool = True
    dev_tools_enabled: bool = True
    context_menu_enabled: bool = True
    allow_file_access: bool = False
    user_agent: Optional[str] = None
    background_color: Optional[str] = None
    zoom_level: int = 100
    minimum_font_size: int = 0
    default_font_size: int = 16
    default_encoding: str = "UTF-8"

    # Additional settings for advanced use cases
    _extra: dict = field(default_factory=dict)

    def set(self, key: str, value: object) -> None:
        """Set a custom setting value.

        Args:
            key: Setting key
            value: Setting value
        """
        if hasattr(self, key):
            setattr(self, key, value)
        else:
            self._extra[key] = value

    def get(self, key: str, default: object = None) -> object:
        """Get a setting value.

        Args:
            key: Setting key
            default: Default value if key not found

        Returns:
            Setting value or default
        """
        if hasattr(self, key) and key != "_extra":
            return getattr(self, key)
        return self._extra.get(key, default)

    def to_dict(self) -> dict:
        """Convert settings to dictionary.

        Returns:
            Dictionary of all settings
        """
        result = {
            "javascript_enabled": self.javascript_enabled,
            "local_storage_enabled": self.local_storage_enabled,
            "dev_tools_enabled": self.dev_tools_enabled,
            "context_menu_enabled": self.context_menu_enabled,
            "allow_file_access": self.allow_file_access,
            "user_agent": self.user_agent,
            "background_color": self.background_color,
            "zoom_level": self.zoom_level,
            "minimum_font_size": self.minimum_font_size,
            "default_font_size": self.default_font_size,
            "default_encoding": self.default_encoding,
        }
        result.update(self._extra)
        return result

    @classmethod
    def from_dict(cls, data: dict) -> "WebViewSettings":
        """Create settings from dictionary.

        Args:
            data: Dictionary of settings

        Returns:
            WebViewSettings instance
        """
        known_keys = {
            "javascript_enabled",
            "local_storage_enabled",
            "dev_tools_enabled",
            "context_menu_enabled",
            "allow_file_access",
            "user_agent",
            "background_color",
            "zoom_level",
            "minimum_font_size",
            "default_font_size",
            "default_encoding",
        }
        known = {k: v for k, v in data.items() if k in known_keys}
        extra = {k: v for k, v in data.items() if k not in known_keys}
        settings = cls(**known)
        settings._extra = extra
        return settings

    def copy(self) -> "WebViewSettings":
        """Create a copy of settings.

        Returns:
            New WebViewSettings instance with same values
        """
        return WebViewSettings.from_dict(self.to_dict())


# Default settings instance
DEFAULT_SETTINGS = WebViewSettings()


__all__ = [
    "WebViewSettings",
    "DEFAULT_SETTINGS",
]
