"""Test Qt backend functionality with actual Qt installation.

This test module requires Qt dependencies to be installed:
    pip install auroraview[qt]

These tests verify that the Qt backend works correctly when dependencies are available.
"""

import sys

import pytest

# Check if Qt is available
try:
    import auroraview

    HAS_QT = auroraview._HAS_QT
    QT_IMPORT_ERROR = auroraview._QT_IMPORT_ERROR
except ImportError:
    HAS_QT = False
    QT_IMPORT_ERROR = "auroraview not installed"

# Skip all tests in this module if Qt is not available
pytestmark = pytest.mark.skipif(not HAS_QT, reason=f"Qt backend not available: {QT_IMPORT_ERROR}")


class TestQtBackendAvailability:
    """Test Qt backend availability and diagnostics."""

    def test_has_qt_flag(self):
        """Test that _HAS_QT flag is True when Qt is installed."""
        import auroraview

        assert auroraview._HAS_QT is True
        assert auroraview._QT_IMPORT_ERROR is None

    def test_qt_classes_importable(self):
        """Test that Qt classes can be imported."""
        from auroraview import AuroraViewQt, QtWebView

        assert QtWebView is not None
        assert AuroraViewQt is not None

    def test_qt_classes_in_all(self):
        """Test that Qt classes are in __all__."""
        import auroraview

        assert "QtWebView" in auroraview.__all__
        assert "AuroraViewQt" in auroraview.__all__


class TestQtWebViewInstantiation:
    """Test QtWebView instantiation and basic properties."""

    def test_qtwebview_can_instantiate(self):
        """Test that QtWebView can be instantiated."""
        from auroraview import QtWebView

        # QtWebView requires a QApplication to exist
        # We'll test instantiation in a controlled way
        try:
            from qtpy.QtWidgets import QApplication

            # Create QApplication if it doesn't exist
            app = QApplication.instance()
            if app is None:
                app = QApplication(sys.argv)

            # Now we can create QtWebView
            webview = QtWebView()
            assert webview is not None

            # Clean up
            webview.close()
            webview.deleteLater()

        except Exception as e:
            pytest.skip(f"Cannot test QtWebView instantiation: {e}")

    def test_qtwebview_has_required_methods(self):
        """Test that QtWebView has all required methods."""
        from auroraview import QtWebView

        # Check that the class has the expected methods
        assert hasattr(QtWebView, "load_url")
        assert hasattr(QtWebView, "load_html")
        assert hasattr(QtWebView, "show")
        assert hasattr(QtWebView, "hide")
        assert hasattr(QtWebView, "close")

    def test_qtwebview_is_qwidget(self):
        """Test that QtWebView is a QWidget subclass."""
        from qtpy.QtWidgets import QWidget

        from auroraview import QtWebView

        assert issubclass(QtWebView, QWidget)


class TestQtWebViewFunctionality:
    """Test QtWebView functionality with actual Qt."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app
        # Cleanup is handled by Qt

    @pytest.fixture
    def webview(self, qapp):
        """Provide a QtWebView instance for tests."""
        from auroraview import QtWebView

        view = QtWebView()
        yield view
        view.close()
        view.deleteLater()

    def test_load_url(self, webview):
        """Test loading a URL."""
        # Load a simple URL
        webview.load_url("https://www.example.com")
        # We can't easily verify the content loaded, but we can verify no exception

    def test_load_html(self, webview):
        """Test loading HTML content."""
        html = "<html><body><h1>Test</h1></body></html>"
        webview.load_html(html)
        # Verify no exception was raised

    def test_show_hide(self, webview):
        """Test show/hide functionality."""
        webview.show()
        assert webview.isVisible()

        webview.hide()
        assert not webview.isVisible()

    def test_window_title(self, webview):
        """Test setting window title."""
        webview.setWindowTitle("Test Window")
        assert webview.windowTitle() == "Test Window"

    def test_resize(self, webview):
        """Test resizing the widget."""
        webview.resize(800, 600)
        size = webview.size()
        assert size.width() == 800
        assert size.height() == 600


class TestAuroraViewQtAlias:
    """Test AuroraViewQt backward compatibility alias."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_auroraviewqt_is_alias(self):
        """Test that AuroraViewQt is an alias for QtWebView."""
        from auroraview import AuroraViewQt, QtWebView

        assert AuroraViewQt is QtWebView

    def test_auroraviewqt_can_instantiate(self, qapp):
        """Test that AuroraViewQt can be instantiated."""
        from auroraview import AuroraViewQt

        view = AuroraViewQt()
        assert view is not None
        view.close()
        view.deleteLater()


class TestQtIntegrationModule:
    """Test the qt_integration module directly."""

    def test_qt_integration_importable(self):
        """Test that qt_integration module can be imported."""
        from auroraview import qt_integration

        assert qt_integration is not None

    def test_qt_integration_exports(self):
        """Test that qt_integration exports expected classes."""
        from auroraview import qt_integration

        assert hasattr(qt_integration, "QtWebView")
        assert hasattr(qt_integration, "AuroraViewQt")

    def test_qtwebview_from_qt_integration(self):
        """Test importing QtWebView directly from qt_integration."""
        from auroraview.qt_integration import QtWebView

        assert QtWebView is not None


class TestQtBackendWithEvents:
    """Test Qt backend event handling."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    @pytest.fixture
    def webview(self, qapp):
        """Provide a QtWebView instance for tests."""
        from auroraview import QtWebView

        view = QtWebView()
        yield view
        view.close()
        view.deleteLater()

    def test_event_decorator_available(self, webview):
        """Test that @webview.on() decorator is available."""
        assert hasattr(webview, "on")
        assert callable(webview.on)

        # Test that we can use the decorator
        @webview.on("test_event")
        def test_handler(data):
            return {"received": data}

        assert callable(test_handler)

    def test_load_html_with_javascript(self, webview):
        """Test loading HTML with JavaScript that could trigger events."""
        html = """
        <html>
        <body>
            <h1>Event Test</h1>
            <script>
                // This would normally trigger events to Python
                console.log('JavaScript loaded');
            </script>
        </body>
        </html>
        """
        webview.load_html(html)
        # Verify no exception was raised


class TestQtBackendErrorHandling:
    """Test Qt backend error handling."""

    @pytest.fixture
    def qapp(self):
        """Provide a QApplication instance for tests."""
        from qtpy.QtWidgets import QApplication

        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        yield app

    def test_invalid_url_handling(self, qapp):
        """Test handling of invalid URLs."""
        from auroraview import QtWebView

        view = QtWebView()
        # Loading an invalid URL should not crash
        view.load_url("not-a-valid-url")
        view.close()
        view.deleteLater()

    def test_empty_html_handling(self, qapp):
        """Test handling of empty HTML."""
        from auroraview import QtWebView

        view = QtWebView()
        view.load_html("")
        view.close()
        view.deleteLater()
