"""AuroraView examples package.

This package contains example scripts demonstrating AuroraView usage in various DCC applications.
"""

from __future__ import annotations

# Maya Qt integration example
try:
    from .maya_qt_echo_demo import maya_qt_echo_demo, show_auroraview_maya_dialog
except ImportError:
    # Maya/Qt dependencies not available
    def maya_qt_echo_demo() -> None:
        """Placeholder for maya_qt_echo_demo when dependencies are not available."""
        raise ImportError(
            "Maya Qt demo requires Maya and Qt dependencies. "
            "Install with: pip install auroraview[qt]"
        )

    def show_auroraview_maya_dialog() -> None:
        """Placeholder for show_auroraview_maya_dialog when dependencies are not available."""
        raise ImportError(
            "Maya Qt demo requires Maya and Qt dependencies. "
            "Install with: pip install auroraview[qt]"
        )


__all__ = [
    "maya_qt_echo_demo",
    "show_auroraview_maya_dialog",
]

