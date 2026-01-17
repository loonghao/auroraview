"""Configuration constants for AuroraView Gallery.

This module contains all configuration constants including:
- Path configurations
- Category definitions
- Icon mappings
- Category mappings
- Tag mappings
"""

from __future__ import annotations

import os
from pathlib import Path

# Path setup - __file__ is correctly set by runpy.run_path() in packed mode
PROJECT_ROOT = Path(__file__).parent.parent.parent
GALLERY_DIR = Path(__file__).parent.parent
DIST_DIR = GALLERY_DIR / "dist"

# Determine examples directory
# Priority: AURORAVIEW_EXAMPLES_DIR env > AURORAVIEW_RESOURCES_DIR/examples > project/examples
_examples_env = os.environ.get("AURORAVIEW_EXAMPLES_DIR")
_resources_env = os.environ.get("AURORAVIEW_RESOURCES_DIR")
if _examples_env:
    EXAMPLES_DIR = Path(_examples_env)
elif _resources_env:
    EXAMPLES_DIR = Path(_resources_env) / "examples"
else:
    EXAMPLES_DIR = PROJECT_ROOT / "examples"

# Category definitions
CATEGORIES = {
    "getting_started": {
        "title": "Getting Started",
        "icon": "rocket",
        "description": "Quick start examples and basic usage patterns",
    },
    "api_patterns": {
        "title": "API Patterns",
        "icon": "code",
        "description": "Different ways to use the AuroraView API",
    },
    "browser_features": {
        "title": "Browser & Tabs",
        "icon": "globe",
        "description": "Multi-tab browser, bookmarks, history, and navigation",
    },
    "window_features": {
        "title": "Window Features",
        "icon": "layout",
        "description": "Window styles, effects, and customization",
    },
    "desktop_features": {
        "title": "Desktop Features",
        "icon": "monitor",
        "description": "File dialogs, shell commands, and system integration",
    },
    "dcc_integration": {
        "title": "DCC Integration",
        "icon": "box",
        "description": "Maya, Houdini, Blender, and other DCC apps",
    },
}

# Icon mapping based on keywords in filename or docstring
ICON_MAPPING = {
    "browser": "globe",
    "tab": "layout-grid",
    "bookmark": "bookmark",
    "history": "clock",
    "download": "download",
    "decorator": "wand-2",
    "binding": "link",
    "event": "bell",
    "floating": "layers",
    "panel": "layers",
    "button": "circle",
    "logo": "circle",
    "tray": "inbox",
    "menu": "menu",
    "context": "menu",
    "desktop": "folder",
    "app": "folder",
    "asset": "image",
    "local": "image",
    "dcc": "box",
    "maya": "layers",
    "qt": "palette",
    "style": "palette",
    "window": "layout",
    "monitor": "monitor",
    "effect": "sparkles",
    "vibrancy": "sparkles",
    "blur": "sparkles",
    "acrylic": "sparkles",
    "mica": "sparkles",
    "click": "mouse-pointer",
    "transparent": "eye",
    "notification": "bell",
    "settings": "settings",
    "cookie": "cookie",
    "automation": "bot",
    "ai": "brain",
    "chat": "message-circle",
}

# Category mapping based on keywords
CATEGORY_MAPPING = {
    # Getting Started
    "simple": "getting_started",
    "decorator": "getting_started",
    "binding": "getting_started",
    "dynamic": "getting_started",
    # API Patterns
    "event": "api_patterns",
    "callback": "api_patterns",
    "signal": "api_patterns",
    "channel": "api_patterns",
    "command": "api_patterns",
    "ipc": "api_patterns",
    # Browser & Tabs (higher priority)
    "browser": "browser_features",
    "multi-tab": "browser_features",
    "tabcontainer": "browser_features",
    "tabbed": "browser_features",
    "agent_browser": "browser_features",
    "bookmark": "browser_features",
    "history": "browser_features",
    "cookie": "browser_features",
    "automation": "browser_features",
    # Window Features
    "floating": "window_features",
    "panel": "window_features",
    "button": "window_features",
    "logo": "window_features",
    "tray": "window_features",
    "menu": "window_features",
    "context": "window_features",
    "window": "window_features",
    "effect": "window_features",
    "vibrancy": "window_features",
    "blur": "window_features",
    "acrylic": "window_features",
    "mica": "window_features",
    "click-through": "window_features",
    "transparent": "window_features",
    "child": "window_features",
    "dock": "window_features",
    "toolbar": "window_features",
    "radial": "window_features",
    # Desktop Features
    "desktop": "desktop_features",
    "file": "desktop_features",
    "dialog": "desktop_features",
    "asset": "desktop_features",
    "local": "desktop_features",
    "dom": "desktop_features",
    # DCC Integration
    "dcc": "dcc_integration",
    "maya": "dcc_integration",
    "houdini": "dcc_integration",
    "blender": "dcc_integration",
    "qt": "dcc_integration",
    "integration": "dcc_integration",
}

# Tag mapping based on keywords
TAG_MAPPING = {
    "beginner": ["simple", "basic", "getting started", "quick"],
    "advanced": ["advanced", "complex", "plugin", "floating", "tray", "browser", "multi-tab", "automation"],
    "window": ["window", "panel", "frame", "transparent", "vibrancy", "blur", "mica", "browser", "tab"],
    "events": ["event", "callback", "lifecycle", "signal", "channel"],
    "qt": ["qt", "pyside", "maya", "houdini", "nuke"],
    "standalone": ["standalone", "desktop", "run_desktop"],
    "ui": ["ui", "style", "menu", "button", "panel", "browser", "navigation", "toolbar"],
    "api": ["api", "decorator", "binding", "call", "command"],
    "effects": ["effect", "vibrancy", "blur", "acrylic", "mica", "click-through"],
    "browser": ["browser", "tab", "bookmark", "history", "cookie", "navigation"],
}
