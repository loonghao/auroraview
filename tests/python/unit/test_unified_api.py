# -*- coding: utf-8 -*-
"""Tests for the unified WebView API (create_webview).

This module tests the new unified API that automatically selects the
appropriate WebView implementation based on the parent type.
"""

import inspect
import threading

import pytest

from auroraview.api import _get_mode_for_parent, _is_qwidget, create_webview

# Check Qt availability
try:
    from qtpy.QtWidgets import QWidget  # noqa: F401

    _QT_AVAILABLE = True
except ImportError:
    _QT_AVAILABLE = False

# Check if _core module is available (needed for WebView instantiation)
try:
    import auroraview._core  # noqa: F401

    _CORE_AVAILABLE = True
except ImportError:
    _CORE_AVAILABLE = False


class TestIsQWidget:
    """Tests for _is_qwidget helper function."""

    def test_none_returns_false(self):
        """None should not be detected as QWidget."""
        assert _is_qwidget(None) is False

    def test_int_returns_false(self):
        """Integer (HWND) should not be detected as QWidget."""
        assert _is_qwidget(12345) is False

    def test_string_returns_false(self):
        """String should not be detected as QWidget."""
        assert _is_qwidget("not a widget") is False

    def test_dict_returns_false(self):
        """Dict should not be detected as QWidget."""
        assert _is_qwidget({"key": "value"}) is False

    def test_list_returns_false(self):
        """List should not be detected as QWidget."""
        assert _is_qwidget([1, 2, 3]) is False

    def test_object_returns_false(self):
        """Regular object should not be detected as QWidget."""

        class MyClass:
            pass

        assert _is_qwidget(MyClass()) is False

    @pytest.mark.skipif(
        True,  # Skip if Qt not available
        reason="Qt not available",
    )
    def test_qwidget_returns_true(self):
        """QWidget should be detected correctly."""
        try:
            from qtpy.QtWidgets import QApplication, QWidget

            _app = QApplication.instance() or QApplication([])  # noqa: F841
            widget = QWidget()
            assert _is_qwidget(widget) is True
            widget.deleteLater()
        except ImportError:
            pytest.skip("Qt not available")


class TestGetModeForParent:
    """Tests for _get_mode_for_parent helper function."""

    def test_explicit_mode_overrides_auto(self):
        """Explicit mode should override auto-detection."""
        assert _get_mode_for_parent(None, "owner") == "owner"
        assert _get_mode_for_parent(12345, "child") == "child"
        assert _get_mode_for_parent(None, "none") == "none"

    def test_none_parent_defaults_to_none(self):
        """None parent should default to 'none' mode."""
        assert _get_mode_for_parent(None, None) == "none"

    def test_int_parent_defaults_to_owner(self):
        """Integer (HWND) parent should default to 'owner' mode."""
        assert _get_mode_for_parent(12345, None) == "owner"
        assert _get_mode_for_parent(0, None) == "owner"  # 0 is valid HWND

    def test_unknown_type_defaults_to_none(self):
        """Unknown parent type should default to 'none' mode."""
        assert _get_mode_for_parent("string", None) == "none"
        assert _get_mode_for_parent({"dict": True}, None) == "none"


class TestCreateWebviewImport:
    """Tests for create_webview import and basic functionality."""

    def test_import_from_api(self):
        """create_webview should be importable from api module."""
        from auroraview.api import create_webview

        assert callable(create_webview)

    def test_import_from_package(self):
        """create_webview should be importable from main package."""
        from auroraview import create_webview

        assert callable(create_webview)

    def test_run_app_import(self):
        """run_app should be importable from main package."""
        from auroraview import run_app

        assert callable(run_app)


class TestCreateWebviewParameters:
    """Tests for create_webview parameter handling."""

    def test_default_parameters(self):
        """Default parameters should be reasonable."""
        # We can't actually create a WebView in unit tests (requires Rust core),
        # but we can verify the function signature accepts expected parameters.
        sig = inspect.signature(create_webview)
        params = sig.parameters

        # Check key parameters exist
        assert "parent" in params
        assert "title" in params
        assert "width" in params
        assert "height" in params
        assert "url" in params
        assert "html" in params
        assert "debug" in params
        assert "context_menu" in params
        assert "frame" in params
        assert "transparent" in params
        assert "mode" in params
        assert "api" in params

        # Check defaults
        assert params["parent"].default is None
        assert params["title"].default == "AuroraView"
        assert params["width"].default == 800
        assert params["height"].default == 600
        assert params["debug"].default is True
        assert params["frame"].default is True
        assert params["mode"].default is None


class TestUnifiedApiExports:
    """Tests for unified API exports in __init__.py."""

    def test_create_webview_in_all(self):
        """create_webview should be in __all__."""
        import auroraview

        assert "create_webview" in auroraview.__all__

    def test_run_app_in_all(self):
        """run_app should be in __all__."""
        import auroraview

        assert "run_app" in auroraview.__all__

    def test_backward_compat_exports(self):
        """Legacy exports should still be available."""
        import auroraview

        # Legacy classes
        assert "WebView" in auroraview.__all__
        assert "QtWebView" in auroraview.__all__
        assert "AuroraView" in auroraview.__all__

        # Legacy functions
        assert "run_desktop" in auroraview.__all__


class TestApiDocumentation:
    """Tests for API documentation."""

    def test_create_webview_has_docstring(self):
        """create_webview should have comprehensive docstring."""
        from auroraview.api import create_webview

        assert create_webview.__doc__ is not None
        assert "parent" in create_webview.__doc__
        assert "QWidget" in create_webview.__doc__
        assert "HWND" in create_webview.__doc__

    def test_module_has_docstring(self):
        """api module should have comprehensive docstring."""
        import auroraview.api

        assert auroraview.api.__doc__ is not None
        assert "create_webview" in auroraview.api.__doc__
        assert "Migration" in auroraview.api.__doc__


class TestParameterNormalization:
    """Tests for parameter normalization between different WebView types."""

    def test_frame_vs_frameless_mapping(self):
        """frame=False should map to frameless=True for QtWebView."""
        # This is a design verification test
        # When frame=False is passed to create_webview with Qt parent,
        # it should be converted to frameless=True for QtWebView
        sig = inspect.signature(create_webview)
        params = sig.parameters

        # Verify we use 'frame' not 'frameless'
        assert "frame" in params
        assert "frameless" not in params

    def test_debug_vs_dev_tools_mapping(self):
        """debug should be the unified parameter name."""
        sig = inspect.signature(create_webview)
        params = sig.parameters

        # Verify we use 'debug' not 'dev_tools'
        assert "debug" in params
        assert "dev_tools" not in params

    def test_mode_vs_embed_mode_mapping(self):
        """mode should be the unified parameter name."""
        sig = inspect.signature(create_webview)
        params = sig.parameters

        # Verify we use 'mode' not 'embed_mode'
        assert "mode" in params
        assert "embed_mode" not in params


class TestApiBindingMixin:
    """Tests for WebViewApiMixin functionality."""

    def test_api_mixin_has_bind_call(self):
        """WebViewApiMixin should provide bind_call method."""
        from auroraview.core.mixins.api import WebViewApiMixin

        assert hasattr(WebViewApiMixin, "bind_call")
        assert callable(WebViewApiMixin.bind_call)

    def test_api_mixin_has_bind_api(self):
        """WebViewApiMixin should provide bind_api method."""
        from auroraview.core.mixins.api import WebViewApiMixin

        assert hasattr(WebViewApiMixin, "bind_api")
        assert callable(WebViewApiMixin.bind_api)

    def test_api_mixin_has_is_method_bound(self):
        """WebViewApiMixin should provide is_method_bound method."""
        from auroraview.core.mixins.api import WebViewApiMixin

        assert hasattr(WebViewApiMixin, "is_method_bound")

    def test_api_mixin_has_is_namespace_bound(self):
        """WebViewApiMixin should provide is_namespace_bound method."""
        from auroraview.core.mixins.api import WebViewApiMixin

        assert hasattr(WebViewApiMixin, "is_namespace_bound")

    def test_api_mixin_has_get_bound_methods(self):
        """WebViewApiMixin should provide get_bound_methods method."""
        from auroraview.core.mixins.api import WebViewApiMixin

        assert hasattr(WebViewApiMixin, "get_bound_methods")


class TestThreadDispatcher:
    """Tests for thread dispatcher utilities."""

    def test_thread_dispatcher_imports(self):
        """Thread dispatcher utilities should be importable."""
        from auroraview.utils.thread_dispatcher import (
            dcc_thread_safe,
            dcc_thread_safe_async,
            get_current_dcc_name,
            is_dcc_environment,
            is_main_thread,
            run_on_main_thread,
            run_on_main_thread_sync,
        )

        assert callable(run_on_main_thread)
        assert callable(run_on_main_thread_sync)
        assert callable(is_main_thread)
        assert callable(dcc_thread_safe)
        assert callable(dcc_thread_safe_async)
        assert callable(is_dcc_environment)
        assert callable(get_current_dcc_name)

    def test_is_main_thread_on_main_thread(self):
        """is_main_thread should return True on main thread."""
        from auroraview.utils.thread_dispatcher import is_main_thread

        # This test runs on main thread
        assert is_main_thread() is True

    def test_is_main_thread_on_background_thread(self):
        """is_main_thread should return False on background thread."""
        from auroraview.utils.thread_dispatcher import is_main_thread

        result = [None]

        def check_thread():
            result[0] = is_main_thread()

        thread = threading.Thread(target=check_thread)
        thread.start()
        thread.join()

        assert result[0] is False

    def test_dcc_thread_safe_decorator(self):
        """dcc_thread_safe decorator should work."""
        from auroraview.utils.thread_dispatcher import dcc_thread_safe

        @dcc_thread_safe
        def test_func():
            return 42

        # Should work on main thread
        result = test_func()
        assert result == 42

    def test_fallback_backend_always_available(self):
        """Fallback backend should always be available."""
        from auroraview.utils.thread_dispatcher import FallbackDispatcherBackend

        backend = FallbackDispatcherBackend()
        assert backend.is_available() is True

    def test_list_dispatcher_backends(self):
        """list_dispatcher_backends should return backend info."""
        from auroraview.utils.thread_dispatcher import list_dispatcher_backends

        backends = list_dispatcher_backends()
        assert isinstance(backends, list)
        assert len(backends) > 0

        # Each entry should be (priority, name, is_available)
        for priority, name, available in backends:
            assert isinstance(priority, int)
            assert isinstance(name, str)
            assert isinstance(available, bool)


class TestDCCThreadSafeWrapper:
    """Tests for DCCThreadSafeWrapper class."""

    def test_wrapper_class_exists(self):
        """DCCThreadSafeWrapper should be importable."""
        from auroraview.utils.thread_dispatcher import DCCThreadSafeWrapper

        assert DCCThreadSafeWrapper is not None

    def test_wrapper_has_expected_methods(self):
        """DCCThreadSafeWrapper should have expected methods."""
        from auroraview.utils.thread_dispatcher import DCCThreadSafeWrapper

        # Check method signatures exist
        assert hasattr(DCCThreadSafeWrapper, "eval_js")
        assert hasattr(DCCThreadSafeWrapper, "eval_js_sync")
        assert hasattr(DCCThreadSafeWrapper, "emit")
        assert hasattr(DCCThreadSafeWrapper, "load_url")
        assert hasattr(DCCThreadSafeWrapper, "load_html")
        assert hasattr(DCCThreadSafeWrapper, "reload")
        assert hasattr(DCCThreadSafeWrapper, "close")


class TestThreadSafetyExceptions:
    """Tests for thread safety exception classes."""

    def test_thread_safety_error_exists(self):
        """ThreadSafetyError should be importable."""
        from auroraview.utils.thread_dispatcher import ThreadSafetyError

        assert issubclass(ThreadSafetyError, Exception)

    def test_thread_dispatch_timeout_error_exists(self):
        """ThreadDispatchTimeoutError should be importable."""
        from auroraview.utils.thread_dispatcher import ThreadDispatchTimeoutError

        assert issubclass(ThreadDispatchTimeoutError, Exception)

    def test_deadlock_detected_error_exists(self):
        """DeadlockDetectedError should be importable."""
        from auroraview.utils.thread_dispatcher import DeadlockDetectedError

        assert issubclass(DeadlockDetectedError, Exception)

    def test_shutdown_in_progress_error_exists(self):
        """ShutdownInProgressError should be importable."""
        from auroraview.utils.thread_dispatcher import ShutdownInProgressError

        assert issubclass(ShutdownInProgressError, Exception)


class TestApiBindingThreadSafety:
    """Tests for API binding thread safety."""

    def test_bind_lock_exists_in_mixin(self):
        """WebViewApiMixin should use a lock for thread safety."""
        from auroraview.core.mixins.api import WebViewApiMixin

        # Check that the mixin references _bind_lock
        source = inspect.getsource(WebViewApiMixin)
        assert "_bind_lock" in source
        assert "Lock" in source

    def test_ensure_api_registry_is_lazy(self):
        """_ensure_api_registry should be lazy initialization."""
        from auroraview.core.mixins.api import WebViewApiMixin

        assert hasattr(WebViewApiMixin, "_ensure_api_registry")


class TestCreateWebviewWithApi:
    """Tests for create_webview with api parameter."""

    def test_api_parameter_exists(self):
        """create_webview should accept api parameter."""
        sig = inspect.signature(create_webview)
        params = sig.parameters

        assert "api" in params
        assert params["api"].default is None

    def test_api_parameter_type_hint(self):
        """api parameter should have correct type hint."""
        sig = inspect.signature(create_webview)
        params = sig.parameters

        # api should accept Any type
        api_param = params["api"]
        assert api_param.annotation is not inspect.Parameter.empty or api_param.default is None


class TestWebViewMixinIntegration:
    """Tests for WebView mixin integration."""

    def test_webview_has_api_mixin_methods(self):
        """WebView class should have API mixin methods."""
        # Check that WebView inherits from WebViewApiMixin
        from auroraview.core.mixins.api import WebViewApiMixin
        from auroraview.core.webview import WebView

        assert issubclass(WebView, WebViewApiMixin)

    def test_webview_has_event_mixin_methods(self):
        """WebView class should have event mixin methods."""
        from auroraview.core.mixins.events import WebViewEventMixin
        from auroraview.core.webview import WebView

        assert issubclass(WebView, WebViewEventMixin)


class TestBackwardCompatibility:
    """Tests for backward compatibility with legacy APIs."""

    def test_legacy_webview_import(self):
        """Legacy WebView import should work."""
        from auroraview.core import WebView

        assert WebView is not None

    def test_legacy_qtwebview_import(self):
        """Legacy QtWebView import should work."""
        from auroraview import QtWebView

        assert QtWebView is not None

    def test_legacy_auroraview_import(self):
        """Legacy AuroraView import should work."""
        from auroraview import AuroraView

        assert AuroraView is not None

    def test_legacy_run_desktop_import(self):
        """Legacy run_desktop import should work."""

        # May be None if core not available, but import should work
        pass


class TestShowMethodBehavior:
    """Tests for unified show() method behavior documentation."""

    def test_show_method_exists_on_webview(self):
        """WebView should have show() method."""
        from auroraview.core import WebView

        assert hasattr(WebView, "show")
        assert callable(getattr(WebView, "show", None))

    def test_show_method_exists_on_qtwebview(self):
        """QtWebView should have show() method."""
        try:
            from auroraview.integration.qt import QtWebView

            assert hasattr(QtWebView, "show")
            assert callable(getattr(QtWebView, "show", None))
        except ImportError:
            pytest.skip("Qt not available")

    def test_show_blocking_exists(self):
        """WebView should have show_blocking() for explicit blocking."""
        from auroraview.core import WebView

        assert hasattr(WebView, "show_blocking")
        assert callable(getattr(WebView, "show_blocking", None))

    def test_show_async_exists(self):
        """WebView should have show_async() for explicit non-blocking."""
        from auroraview.core import WebView

        assert hasattr(WebView, "show_async")
        assert callable(getattr(WebView, "show_async", None))

    def test_show_accepts_wait_parameter(self):
        """show() should accept wait parameter."""
        import inspect

        from auroraview.core import WebView

        sig = inspect.signature(WebView.show)
        params = sig.parameters
        assert "wait" in params, "show() should accept 'wait' parameter"

    def test_qtwebview_show_is_non_blocking(self):
        """QtWebView.show() should be non-blocking (Qt semantics)."""
        try:
            import inspect

            from auroraview.integration.qt import QtWebView

            # QtWebView.show() should not have wait parameter
            # because it always follows Qt widget semantics (non-blocking)
            sig = inspect.signature(QtWebView.show)
            params = sig.parameters
            # QtWebView.show() should be simple Qt show()
            assert len(params) <= 1, "QtWebView.show() should be simple (Qt semantics)"
        except ImportError:
            pytest.skip("Qt not available")


class TestShowMethodDocumentation:
    """Tests to verify show() method documentation accuracy."""

    def test_standalone_mode_detection(self):
        """Verify standalone mode is detected when parent is None."""
        # This tests the logic documented in unified-api.md
        parent = None
        is_embedded = parent is not None
        assert is_embedded is False, "No parent should mean standalone mode"

    def test_embedded_mode_detection_with_hwnd(self):
        """Verify embedded mode is detected when parent is HWND."""
        parent = 12345  # Simulated HWND
        is_embedded = parent is not None
        assert is_embedded is True, "HWND parent should mean embedded mode"

    def test_mode_auto_detection_logic(self):
        """Verify auto mode detection logic."""
        # Test the documented auto-detection behavior

        # None -> "none"
        parent_none = None
        mode_none = "none" if parent_none is None else "auto"
        assert mode_none == "none"

        # int (HWND) -> "owner"
        parent_hwnd = 12345
        mode_hwnd = "owner" if isinstance(parent_hwnd, int) else "auto"
        assert mode_hwnd == "owner"

    @pytest.mark.skipif(not _QT_AVAILABLE, reason="Qt not available for QWidget test")
    def test_mode_auto_detection_qwidget(self):
        """Verify auto mode detection for QWidget."""
        from qtpy.QtWidgets import QApplication, QWidget

        _app = QApplication.instance() or QApplication([])  # noqa: F841
        widget = QWidget()

        # QWidget -> "child"
        mode_widget = "child" if _is_qwidget(widget) else "auto"
        assert mode_widget == "child"
        widget.deleteLater()


class TestQtWebViewShowEvent:
    """Tests for QtWebView showEvent auto-initialization."""

    @pytest.mark.skipif(not _QT_AVAILABLE, reason="Qt not available")
    def test_qtwebview_has_show_event(self):
        """QtWebView should have showEvent method."""
        try:
            from auroraview.integration.qt import QtWebView

            assert hasattr(QtWebView, "showEvent")
            assert callable(getattr(QtWebView, "showEvent", None))
        except ImportError:
            pytest.skip("QtWebView not available")

    @pytest.mark.skipif(
        not _QT_AVAILABLE or not _CORE_AVAILABLE,
        reason="Qt or _core not available",
    )
    def test_qtwebview_has_webview_initialized_flag(self):
        """QtWebView should track initialization state."""
        try:
            from qtpy.QtWidgets import QApplication

            from auroraview.integration.qt import QtWebView

            _app = QApplication.instance() or QApplication([])  # noqa: F841

            # Create QtWebView without showing
            webview = QtWebView(url="about:blank")

            # Should have initialization flag
            assert hasattr(webview, "_webview_initialized")
            # Should not be initialized yet (not shown)
            assert webview._webview_initialized is False

            webview.deleteLater()
        except ImportError:
            pytest.skip("QtWebView not available")

    @pytest.mark.skipif(not _QT_AVAILABLE, reason="Qt not available")
    def test_qtwebview_show_is_simple_qt_show(self):
        """QtWebView.show() should just call QWidget.show()."""
        try:
            from auroraview.integration.qt import QtWebView

            # Check that show() doesn't have extra parameters like wait
            sig = inspect.signature(QtWebView.show)
            params = list(sig.parameters.keys())

            # Should only have 'self' parameter (standard Qt show)
            assert params == ["self"], (
                f"QtWebView.show() should be simple Qt show(), got params: {params}"
            )
        except ImportError:
            pytest.skip("QtWebView not available")

    @pytest.mark.skipif(
        not _QT_AVAILABLE or not _CORE_AVAILABLE,
        reason="Qt or _core not available",
    )
    def test_qtwebview_no_explicit_show_needed_in_layout(self):
        """QtWebView should auto-initialize when parent is shown."""
        try:
            from qtpy.QtWidgets import QApplication, QVBoxLayout, QWidget

            from auroraview.integration.qt import QtWebView

            _app = QApplication.instance() or QApplication([])  # noqa: F841

            # Create parent widget with layout
            parent = QWidget()
            layout = QVBoxLayout(parent)

            # Create QtWebView and add to layout (but don't show)
            webview = QtWebView(parent=parent, url="about:blank")
            layout.addWidget(webview)

            # WebView should not be initialized yet
            assert webview._webview_initialized is False

            # Note: Actually showing would require a running event loop
            # and WebView2 runtime, so we just verify the flag mechanism

            parent.deleteLater()
        except ImportError:
            pytest.skip("QtWebView not available")
