"""Maya + QtWebView shelf demo using auroraview.api.rename_selected.

Usage inside Maya Script Editor::

    import examples.maya_qt_echo_demo as demo
    demo.show_auroraview_maya_dialog()

This requires:
    - auroraview installed with Qt extras: `mayapy -m pip install auroraview[qt]`
    - qtpy + a supported Qt binding (PySide2 / PySide6 / PyQt5 / PyQt6)

The example demonstrates:
    - QtWebView automatic event processing (no manual process_events() needed)
    - High-level interaction events (`viewport.*` / `ui.view.*`)
    - QtWebView.load_file() helper for loading external HTML files
    - Best practices for Qt-based DCC integration

Note:
    This example uses QtWebView which automatically handles event processing.
    You don't need to manually call process_events() or create scriptJobs.
    See docs/QT_BEST_PRACTICES.md for more information.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Optional

import maya.OpenMayaUI as omui
from qtpy.QtWidgets import QDialog, QVBoxLayout, QWidget
from shiboken2 import wrapInstance

from auroraview import AuroraView, QtWebView


def _maya_main_window() -> QWidget:
    """Return Maya main window as a QWidget.

    This uses shiboken2 + qtpy to stay agnostic to the actual Qt binding.
    """

    ptr = omui.MQtUtil.mainWindow()
    if ptr is None:
        raise RuntimeError("Cannot find Maya main window")
    return wrapInstance(int(ptr), QWidget)


class _ShelfAPI:
    """API object exposed to `auroraview.api.*` for a Maya shelf-style demo.

    Methods on this class become `auroraview.api.<name>` on the JS side
    when bound via :class:`AuroraView` / ``bind_api``.
    """

    def rename_selected(self, prefix: str = "av_") -> dict[str, Any]:
        """Rename the currently selected Maya objects and print to Script Editor.

        Args:
            prefix: Base prefix for the new object names (e.g. "av_", "char_").

        Returns:
            A dictionary with summary information for debugging in DevTools.
        """

        import maya.cmds as cmds

        sel = cmds.ls(selection=True, long=False) or []
        if not sel:
            msg = "[AuroraView] No objects selected to rename."
            print(msg)
            return {"ok": False, "message": msg, "renamed": []}

        renamed: list[dict[str, str]] = []
        for index, obj in enumerate(sel, start=1):
            new_name = f"{prefix}{index:02d}"
            try:
                actual_new = cmds.rename(obj, new_name)
                renamed.append({"old": obj, "new": actual_new})
            except Exception as exc:  # pragma: no cover - runs only inside Maya
                print(f"[AuroraView] Failed to rename {obj}: {exc}")

        msg = f"[AuroraView] Renamed {len(renamed)}/{len(sel)} selected objects."
        print(msg)
        return {"ok": True, "message": msg, "renamed": renamed}


class AuroraViewMayaDialog(QDialog):
    """Qt dialog embedding a QtWebView inside Maya.

    The dialog hosts a QtWebView and exposes a rename API so that the
    front-end can call `auroraview.api.rename_selected({...})` and receive a result.

    Best Practices Demonstrated:
        - Uses QtWebView for automatic event processing
        - No manual process_events() calls needed
        - No scriptJob required for event handling
        - Clean integration with Maya's Qt event loop

    See Also:
        - docs/QT_BEST_PRACTICES.md for detailed guide
        - docs/CHANGELOG_QT_IMPROVEMENTS.md for technical details
    """

    def __init__(self, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)
        self.setWindowTitle("AuroraView Maya Shelf (Rename Selection)")
        self.resize(800, 600)
        # Enable the standard Qt size grip so the user can resize the dialog
        # without interfering with the embedded WebView content.
        self.setSizeGripEnabled(True)
        # Use a dark background so the Qt frame around the WebView looks
        # consistent with the HTML content and does not show a bright strip.
        self.setStyleSheet("background-color: #383838;")

        layout = QVBoxLayout(self)
        # Leave a more generous margin so the dialog's resize grip and borders
        # are clearly separated from the embedded WebView. This reduces the
        # chance of accidentally grabbing the WebView when the user intends to
        # resize the Qt dialog itself.
        layout.setContentsMargins(14, 14, 14, 14)

        # Create QtWebView as child widget. Disable dev tools here to reduce
        # startup overhead in production/demo scenarios.
        #
        # ✨ Event processing is automatic with QtWebView!
        # No need to call process_events() or create scriptJobs.
        self.webview = QtWebView(self, dev_tools=False)
        layout.addWidget(self.webview)

        # Bind Python API to `auroraview.api.*` via AuroraView wrapper, which
        # also keeps this dialog alive through its internal registry.
        self.api = _ShelfAPI()
        self.auroraview = AuroraView(
            parent=self,
            api=self.api,
            _view=self.webview,
            _keep_alive_root=self,
        )

        # Demo handlers for high-level interaction events.
        # In a real tool you would map these to Maya camera/viewport operations
        # instead of just printing.
        def _log_event(name: str, payload: Any) -> None:
            print(f"[AuroraView Demo] {name}: {payload!r}")

        def _handle_viewport_orbit(data: Any) -> None:
            _log_event("viewport.orbit", data)

        def _handle_viewport_zoom(data: Any) -> None:
            _log_event("viewport.zoom", data)

        def _handle_ui_pan(data: Any) -> None:
            _log_event("ui.view.pan", data)

        def _handle_ui_zoom(data: Any) -> None:
            _log_event("ui.view.zoom", data)

        self.webview.on("viewport.orbit")(_handle_viewport_orbit)
        self.webview.on("viewport.zoom")(_handle_viewport_zoom)
        self.webview.on("ui.view.pan")(_handle_ui_pan)
        self.webview.on("ui.view.zoom")(_handle_ui_zoom)

        # Load HTML from an external file next to this module and feed it
        # via load_html() so we avoid `file://` restrictions in embedded
        # WebView2 inside DCC hosts like Maya.
        html_path = Path(__file__).with_suffix(".html")
        self.webview.load_file(html_path)
        self.webview.show()


def show_auroraview_maya_dialog() -> None:
    """Show the AuroraView Qt echo dialog inside Maya.

    This helper can be called directly from Maya's Script Editor::

        import examples.maya_qt_echo_demo as demo
        demo.show_auroraview_maya_dialog()
    """

    parent = _maya_main_window()
    dlg = AuroraViewMayaDialog(parent)
    dlg.setObjectName("AuroraViewMayaEchoDialog")
    dlg.show()


# Convenience function for direct execution
def maya_qt_echo_demo() -> None:
    """Convenience function to show the Maya Qt echo demo.

    This can be called as::

        from examples import maya_qt_echo_demo
        maya_qt_echo_demo()
    """
    show_auroraview_maya_dialog()
