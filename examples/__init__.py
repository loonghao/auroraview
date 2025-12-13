"""AuroraView examples package.

This package contains example scripts demonstrating AuroraView usage in various DCC applications.

Quick Start:
    # Run the interactive gallery
    python gallery/main.py
    # or via just:
    just gallery

    # Or run individual examples
    python examples/simple_decorator.py
    python examples/floating_panel_demo.py

Available Examples:
    Core Features:
    - simple_decorator.py: Basic API binding with decorators
    - dynamic_binding.py: Runtime API binding and plugin system
    - window_events_demo.py: Window lifecycle events

    Desktop Features:
    - desktop_app_demo.py: File dialogs, shell commands, environment
    - desktop_events_demo.py: File drag-drop, plugin invoke
    - system_tray_demo.py: System tray icon and menu
    - floating_panel_demo.py: Transparent, frameless windows

    DOM & UI:
    - dom_manipulation_demo.py: Element operations from Python
    - custom_context_menu_demo.py: Custom right-click menus
    - native_menu_demo.py: Application menu bar with shortcuts

    Advanced:
    - custom_protocol_demo.py: Register custom URL schemes
    - multi_window_demo.py: Multiple windows with communication
    - signals_advanced_demo.py: Qt-style signal-slot system
    - cookie_management_demo.py: Session and persistent cookies

    DCC Integration:
    - maya_qt_echo_demo.py: Maya integration with QtWebView
    - qt_style_tool.py: Qt-style class inheritance pattern
    - qt_custom_menu_demo.py: Qt environment custom menus
    - dcc_integration_example.py: Non-blocking event loop
    - local_assets_example.py: file:// protocol support
    - logo_button_demo.py: Transparent floating button
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

