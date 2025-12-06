"""Qt5/Qt6 compatibility test script.

This script demonstrates how to use the Qt configuration factory
and diagnostics tools to ensure consistent behavior across Qt versions.

Usage:
    # In Maya (PySide2/Qt5)
    python qt_compatibility_test.py

    # In Houdini (PySide6/Qt6)
    python qt_compatibility_test.py
"""

import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def test_qt_environment():
    """Test Qt environment detection and configuration."""
    print("\n" + "=" * 60)
    print("Qt Environment Test")
    print("=" * 60)

    from auroraview.integration.qt.diagnostics import (
        diagnose_qt_environment,
        print_diagnostics,
    )
    from auroraview.integration.qt.qt_config_factory import create_optimal_qt_config

    # Diagnose environment
    env_diag = diagnose_qt_environment()
    print_diagnostics(env_diag, "Qt Environment Diagnostics")

    # Test configuration factory
    print("\nConfiguration Factory Test:")
    print("-" * 60)

    dccs = ["maya", "houdini", "substancepainter", None]
    for dcc in dccs:
        config = create_optimal_qt_config(dcc)
        print(f"\n{dcc or 'generic'}:")
        print(f"  init_delay_ms: {config['init_delay_ms']}")
        print(f"  timer_interval_ms: {config['timer_interval_ms']}")
        print(f"  geometry_fix_delays: {config['geometry_fix_delays']}")
        print(f"  force_opaque_window: {config['force_opaque_window']}")
        print(f"  is_qt6: {config['is_qt6']}")


def test_dialog_configuration():
    """Test dialog configuration and diagnostics."""
    print("\n" + "=" * 60)
    print("Dialog Configuration Test")
    print("=" * 60)

    try:
        from qtpy.QtCore import Qt
        from qtpy.QtWidgets import QApplication, QDialog

        from auroraview.integration.qt._compat import is_qt6
        from auroraview.integration.qt.diagnostics import (
            diagnose_dialog,
            print_diagnostics,
        )
    except ImportError as e:
        print(f"Qt not available: {e}")
        return

    # Create QApplication if needed
    app = QApplication.instance()
    if not app:
        app = QApplication([])

    # Create test dialog
    dialog = QDialog()
    dialog.setWindowTitle("Qt Compatibility Test")

    # Apply Qt6 optimizations
    if is_qt6():
        print("\nApplying Qt6 optimizations...")
        dialog.setWindowFlags(
            Qt.Tool | Qt.WindowTitleHint | Qt.WindowCloseButtonHint
        )
        dialog.setAttribute(Qt.WA_OpaquePaintEvent, True)
        dialog.setAttribute(Qt.WA_TranslucentBackground, False)
        dialog.setAttribute(Qt.WA_NoSystemBackground, False)
    else:
        print("\nApplying Qt5 settings...")
        dialog.setWindowFlags(Qt.Window)

    # Diagnose dialog
    diag = diagnose_dialog(dialog)
    print_diagnostics(diag, "Dialog Diagnostics")

    # Show recommendations
    if diag["recommendations"]:
        print("\nRECOMMENDATIONS:")
        for rec in diag["recommendations"]:
            print(f"  ✓ {rec}")


def test_webview_integration():
    """Test WebView integration with Qt compatibility layer."""
    print("\n" + "=" * 60)
    print("WebView Integration Test")
    print("=" * 60)

    try:
        from qtpy.QtWidgets import QApplication, QDialog, QVBoxLayout

        from auroraview import QtWebView
        from auroraview.integration.qt.diagnostics import (
            diagnose_webview_container,
            print_diagnostics,
        )
        from auroraview.integration.qt.qt_config_factory import create_optimal_qt_config
    except ImportError as e:
        print(f"Required modules not available: {e}")
        return

    # Create QApplication if needed
    app = QApplication.instance()
    if not app:
        app = QApplication([])

    # Get optimal configuration
    config = create_optimal_qt_config("maya")  # Change to your DCC
    print(f"\nUsing configuration: {config}")

    # Create dialog
    dialog = QDialog()
    dialog.setWindowTitle("WebView Test")
    dialog.resize(800, 600)

    layout = QVBoxLayout(dialog)
    layout.setContentsMargins(0, 0, 0, 0)

    # Create WebView
    print("\nCreating QtWebView...")
    webview = QtWebView(parent=dialog, width=800, height=600)

    # Add to layout
    layout.addWidget(webview)

    # Diagnose container
    if hasattr(webview, "_container"):
        container_diag = diagnose_webview_container(webview._container)
        print_diagnostics(container_diag, "WebView Container Diagnostics")

    # Load test content
    webview.load_html(
        """
        <html>
        <body style="margin:0; padding:20px; font-family:sans-serif;">
            <h1>Qt Compatibility Test</h1>
            <p>If you can see this, the WebView is working!</p>
            <button onclick="alert('Click works!')">Test Click</button>
        </body>
        </html>
        """
    )

    dialog.show()
    print("\nDialog shown. Close the window to continue...")

    # Run event loop
    app.exec_()


if __name__ == "__main__":
    print("\n" + "=" * 60)
    print("AuroraView Qt5/Qt6 Compatibility Test Suite")
    print("=" * 60)

    # Run tests
    test_qt_environment()
    test_dialog_configuration()

    # Uncomment to test WebView integration
    # test_webview_integration()

    print("\n" + "=" * 60)
    print("Tests Complete!")
    print("=" * 60 + "\n")

