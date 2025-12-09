"""Qt diagnostics for troubleshooting Qt5/Qt6 compatibility issues.

This module provides diagnostic tools to identify and troubleshoot
Qt version-specific issues in DCC integrations.
"""

from __future__ import annotations

import logging
from typing import Any, Dict

from auroraview.integration.qt._compat import get_qt_info, is_qt6

logger = logging.getLogger(__name__)


def diagnose_qt_environment() -> Dict[str, Any]:
    """Diagnose Qt environment and detect potential issues.

    Returns:
        Dict with diagnostic information and recommendations.
    """
    binding, version = get_qt_info()
    is_qt6_env = is_qt6()

    info = {
        "qt_binding": binding,
        "qt_version": version,
        "is_qt6": is_qt6_env,
        "issues": [],
        "warnings": [],
        "recommendations": [],
    }

    # Check if Qt is available
    try:
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app:
            info["app_name"] = app.applicationName()
            info["app_version"] = app.applicationVersion()
        else:
            info["warnings"].append("No QApplication instance found")
    except ImportError:
        info["issues"].append("Qt not available - qtpy import failed")
        return info

    # Qt6-specific checks
    if is_qt6_env:
        info["recommendations"].append("Use opaque windows for better performance")
        info["recommendations"].append("Increase init_delay_ms to 100ms or more")
        info["recommendations"].append("Use Qt.Tool flag for tool windows")

    return info


def diagnose_dialog(dialog: Any) -> Dict[str, Any]:
    """Diagnose QDialog configuration for Qt compatibility issues.

    Args:
        dialog: The QDialog to diagnose.

    Returns:
        Dict with diagnostic information.
    """
    try:
        from qtpy.QtCore import Qt
    except ImportError:
        return {"error": "Qt not available"}

    info = {
        "qt_info": get_qt_info(),
        "is_qt6": is_qt6(),
        "issues": [],
        "warnings": [],
        "recommendations": [],
        "attributes": {},
        "flags": {},
    }

    # Check window attributes
    attributes_to_check = [
        ("WA_TranslucentBackground", Qt.WA_TranslucentBackground),
        ("WA_OpaquePaintEvent", Qt.WA_OpaquePaintEvent),
        ("WA_NoSystemBackground", Qt.WA_NoSystemBackground),
        ("WA_NativeWindow", Qt.WA_NativeWindow),
        ("WA_InputMethodEnabled", Qt.WA_InputMethodEnabled),
    ]

    for name, attr in attributes_to_check:
        try:
            info["attributes"][name] = dialog.testAttribute(attr)
        except Exception as e:
            info["warnings"].append(f"Failed to check {name}: {e}")

    # Check window flags
    flags = dialog.windowFlags()
    info["flags"]["is_tool"] = bool(flags & Qt.Tool)
    info["flags"]["is_window"] = bool(flags & Qt.Window)
    info["flags"]["is_frameless"] = bool(flags & Qt.FramelessWindowHint)

    # Qt6-specific checks
    if is_qt6():
        # Check for performance issues
        if info["attributes"].get("WA_TranslucentBackground"):
            info["issues"].append("PERFORMANCE: Translucent background is slow in Qt6")
            info["recommendations"].append("Set WA_TranslucentBackground to False")

        if not info["attributes"].get("WA_OpaquePaintEvent"):
            info["warnings"].append("OpaquePaintEvent not set (recommended for Qt6)")
            info["recommendations"].append("Set WA_OpaquePaintEvent to True for better performance")

        if not info["flags"]["is_tool"]:
            info["warnings"].append("Not using Qt.Tool flag (window may not stay on top)")
            info["recommendations"].append("Use Qt.Tool flag for tool windows in Qt6")

    return info


def diagnose_webview_container(container: Any) -> Dict[str, Any]:
    """Diagnose WebView container widget for Qt compatibility issues.

    Args:
        container: The container QWidget.

    Returns:
        Dict with diagnostic information.
    """
    try:
        from qtpy.QtCore import Qt
    except ImportError:
        return {"error": "Qt not available"}

    info = {
        "qt_info": get_qt_info(),
        "is_qt6": is_qt6(),
        "issues": [],
        "warnings": [],
        "recommendations": [],
        "attributes": {},
        "size_policy": {},
    }

    # Check attributes
    attributes_to_check = [
        ("WA_NativeWindow", Qt.WA_NativeWindow),
        ("WA_OpaquePaintEvent", Qt.WA_OpaquePaintEvent),
        ("WA_InputMethodEnabled", Qt.WA_InputMethodEnabled),
    ]

    for name, attr in attributes_to_check:
        try:
            info["attributes"][name] = container.testAttribute(attr)
        except Exception as e:
            info["warnings"].append(f"Failed to check {name}: {e}")

    # Check size policy
    try:
        policy = container.sizePolicy()
        info["size_policy"]["horizontal"] = policy.horizontalPolicy()
        info["size_policy"]["vertical"] = policy.verticalPolicy()
    except Exception as e:
        info["warnings"].append(f"Failed to check size policy: {e}")

    # Qt6-specific checks
    if is_qt6():
        if not info["attributes"].get("WA_NativeWindow"):
            info["warnings"].append("WA_NativeWindow not set (may cause issues in Qt6)")
            info["recommendations"].append("Set WA_NativeWindow to True for Qt6 containers")

    return info


def print_diagnostics(diag: Dict[str, Any], title: str = "Diagnostics") -> None:
    """Print diagnostic information in a readable format.

    Args:
        diag: Diagnostic dict from diagnose_* functions.
        title: Title for the diagnostic output.
    """
    print(f"\n{'=' * 60}")
    print(f"{title}")
    print(f"{'=' * 60}")

    for key, value in diag.items():
        if key in ("issues", "warnings", "recommendations"):
            if value:
                print(f"\n{key.upper()}:")
                for item in value:
                    print(f"  - {item}")
        elif isinstance(value, dict):
            print(f"\n{key.upper()}:")
            for k, v in value.items():
                print(f"  {k}: {v}")
        else:
            print(f"{key}: {value}")

    print(f"{'=' * 60}\n")
