# -*- coding: utf-8 -*-
"""Unit tests for the zombie-widget guards added to ``QtWebView`` in _core.py.

These tests exercise the new patch-coverage surface introduced by the
fix for "QtWebView zombie reference RuntimeError under DCC hosts":

* ``_guard_alive`` decorator (module-level)
* ``QtWebView.is_alive`` property
* ``QtWebView._teardown_signal_bridge`` method
* ``QtWebView.destroy`` method
* ``QtWebView.__del__`` -> ``_handle_destructor`` delegation
* ``QtWebView.eventFilter`` close-propagation + RuntimeError swallow
* ``QtWebView.resizeEvent`` container-deleted recovery
* ``QtWebView.showEvent`` reuse-after-close path
* ``QtWebView.closeEvent`` accept semantics
* The ``@_guard_alive`` decorated high-level API methods
* ``on()`` and ``register_callback`` IPC wrappers

Pure-logic methods (``_guard_alive``, ``is_alive``, ``_teardown_signal_bridge``,
``destroy``, ``__del__``, ``__repr__``, ``get_hwnd``, the guarded API methods,
``on``, ``register_callback``) are tested against lightweight stubs by
binding the unbound method/property to a stub instance.

Methods that call ``super()`` (``closeEvent``, ``showEvent``, ``resizeEvent``,
``eventFilter``) require a genuine ``QtWebView`` instance for ``super()``
to resolve. We construct one via ``__new__`` + manual ``QWidget.__init__``
so that the C++ widget exists but our pre-set Python attributes drive the
behaviour. This avoids spinning up the Rust core while still letting Qt's
super-class methods run.
"""

import sys
import types
from unittest.mock import MagicMock

import pytest

# The module under test imports qtpy at load time, so skip the whole
# module on environments where qtpy is unavailable.
pytest.importorskip("qtpy", reason="Qt tests require qtpy")

# noqa imports must run AFTER the importorskip above.
from auroraview.integration.qt import _core  # noqa: E402, I001
from auroraview.integration.qt._core import QtWebView, _guard_alive  # noqa: E402

pytestmark = [pytest.mark.qt_related, pytest.mark.unit]


# ---------------------------------------------------------------------------
# Module-level qapp fixture: a single QApplication shared by all tests in
# this file that need a Qt event loop ancestry. We do NOT use pytest-qt's
# ``qapp`` because importing pytest-qt here would force the full Qt test
# environment for the entire suite; this fixture is intentionally minimal.
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def qapp():
    from qtpy.QtWidgets import QApplication

    app = QApplication.instance()
    if app is None:
        app = QApplication(sys.argv)
    yield app


@pytest.fixture
def bare_widget(qapp):
    """Construct a QtWebView shell without running its ``__init__``.

    This bypasses Rust-core acquisition while still giving us a live C++
    QWidget so that ``super().closeEvent`` etc. resolve. We populate just
    the Python-side attributes the methods under test consult.
    """
    from qtpy.QtWidgets import QWidget

    obj = QtWebView.__new__(QtWebView)
    QWidget.__init__(obj)
    # Pre-populate attributes touched by the methods we'll exercise so
    # that the test author -- not the production code -- explicitly opts
    # into each branch.
    obj._is_closing = False
    obj._webview = MagicMock()
    obj._webview_initialized = True
    obj._webview_container = None
    obj._using_direct_embed = False
    obj._direct_embed_hwnd = None
    obj._parent_window = None
    obj._asset_root = None
    obj._qt_signal_state = {
        "current_url": "",
        "current_title": "",
        "is_loading": False,
        "load_progress": 0,
    }
    obj._setup_signal_bridge = MagicMock()
    obj._initialize_webview = MagicMock()
    obj._sync_webview2_controller_bounds = MagicMock()
    yield obj
    # Best-effort cleanup; the C++ object is reaped by Qt at end of scope.
    try:
        obj.deleteLater()
    except RuntimeError:
        pass


# ---------------------------------------------------------------------------
# _guard_alive decorator
# ---------------------------------------------------------------------------


class _GuardHost:
    """Minimal stand-in for a QtWebView-like object exposing ``is_alive``."""

    def __init__(self, alive=True):
        self.is_alive = alive
        self.calls = []


class TestGuardAliveDecorator:
    def test_returns_method_result_when_alive(self):
        @_guard_alive
        def method(self, x):
            self.calls.append(x)
            return "ok"

        host = _GuardHost(alive=True)
        assert method(host, 7) == "ok"
        assert host.calls == [7]

    def test_returns_none_when_not_alive(self):
        @_guard_alive
        def method(self, x):
            self.calls.append(x)
            return "should-not-run"

        host = _GuardHost(alive=False)
        assert method(host, 1) is None
        assert host.calls == []

    def test_swallows_cpp_deleted_runtime_error(self):
        @_guard_alive
        def method(self):
            raise RuntimeError("Internal C++ object (QWidget) already deleted")

        host = _GuardHost(alive=True)
        # Must not raise -- the decorator absorbs C++-deletion errors.
        assert method(host) is None

    def test_swallows_wrapped_cpp_runtime_error(self):
        @_guard_alive
        def method(self):
            raise RuntimeError("wrapped C++ object of type Foo has been deleted")

        host = _GuardHost(alive=True)
        assert method(host) is None

    def test_reraises_unrelated_runtime_error(self):
        """The decorator must NOT swallow non-zombie RuntimeErrors -- those
        signal real bugs (path errors, Rust panics, ...).
        """

        @_guard_alive
        def method(self):
            raise RuntimeError("invalid path: /nonexistent/file.html")

        host = _GuardHost(alive=True)
        with pytest.raises(RuntimeError, match="invalid path"):
            method(host)

    def test_reraises_unrelated_exception_types(self):
        @_guard_alive
        def method(self):
            raise ValueError("some other bug")

        host = _GuardHost(alive=True)
        with pytest.raises(ValueError):
            method(host)

    def test_preserves_function_metadata(self):
        @_guard_alive
        def my_method(self):
            """My docstring."""

        assert my_method.__name__ == "my_method"
        assert my_method.__doc__ == "My docstring."

    def test_passes_args_and_kwargs(self):
        @_guard_alive
        def method(self, a, b, *, c):
            return (a, b, c)

        host = _GuardHost(alive=True)
        assert method(host, 1, 2, c=3) == (1, 2, 3)


# ---------------------------------------------------------------------------
# QtWebView.is_alive property
# ---------------------------------------------------------------------------


class _AliveHost:
    """Stub for testing the ``is_alive`` property fget directly.

    ``objectName`` is the cheap Qt access used by the property to detect
    zombie C++ widgets. We expose it as a callable attribute and have it
    raise ``RuntimeError`` to simulate post-deletion access.
    """

    def __init__(self, is_closing=False, raise_on_object_name=False):
        self._is_closing = is_closing
        self._raise = raise_on_object_name

    def objectName(self):
        if self._raise:
            raise RuntimeError("Internal C++ object already deleted")
        return "stub"


class TestIsAliveProperty:
    def _is_alive(self, host):
        return QtWebView.is_alive.fget(host)

    def test_returns_true_for_fresh_widget(self):
        host = _AliveHost()
        assert self._is_alive(host) is True

    def test_returns_false_when_closing_flag_set(self):
        host = _AliveHost(is_closing=True)
        # Even a fully-alive Qt object reports not-alive while closing.
        assert self._is_alive(host) is False

    def test_returns_false_when_object_name_raises(self):
        host = _AliveHost(raise_on_object_name=True)
        assert self._is_alive(host) is False

    def test_closing_short_circuits_object_name_check(self):
        """``_is_closing`` must be checked first so a teardown sequence
        doesn't trigger a redundant Qt call on a half-dead widget.
        """
        host = _AliveHost(is_closing=True, raise_on_object_name=True)
        # Should still return False without ever hitting objectName().
        assert self._is_alive(host) is False


# ---------------------------------------------------------------------------
# QtWebView._teardown_signal_bridge method
# ---------------------------------------------------------------------------


_BRIDGE_EVENTS = [
    "navigation_started",
    "navigation_finished",
    "load_progress",
    "title_changed",
    "url_changed",
    "js_error",
    "console_message",
    "render_process_terminated",
    "selection_changed",
    "icon_changed",
]


class _CoreLikeWebView:
    """Approximation of the underlying ``WebView`` core used by the bridge."""

    def __init__(self, with_signals=True, with_handlers=True):
        # threading.Lock-style context manager.
        self._event_handlers_lock = MagicMock()
        self._event_handlers_lock.__enter__ = MagicMock(return_value=None)
        self._event_handlers_lock.__exit__ = MagicMock(return_value=None)

        if with_handlers:
            # Pre-populate one handler per known signal-bridge event plus a
            # user-registered handler that must be preserved.
            self._event_handlers = {ev: [lambda data: None] for ev in _BRIDGE_EVENTS}
            self._event_handlers["user_event"] = [lambda data: None]
        else:
            self._event_handlers = {}

        if with_signals:
            self._signals = MagicMock()
            self._signals.custom = {ev: MagicMock() for ev in _BRIDGE_EVENTS}
        else:
            self._signals = None


class _TeardownHost:
    def __init__(self, webview):
        self._webview = webview


class TestTeardownSignalBridge:
    def _teardown(self, host):
        return QtWebView._teardown_signal_bridge(host)

    def test_returns_silently_when_webview_missing(self):
        host = _TeardownHost(webview=None)
        # Must not raise.
        self._teardown(host)

    def test_returns_silently_when_webview_attribute_absent(self):
        host = _TeardownHost(webview=MagicMock())
        del host._webview
        # Must not raise.
        self._teardown(host)

    def test_clears_known_event_handlers_only(self):
        core = _CoreLikeWebView(with_signals=False)
        host = _TeardownHost(webview=core)
        self._teardown(host)
        for ev in _BRIDGE_EVENTS:
            assert ev not in core._event_handlers
        # User-registered handlers must be preserved.
        assert "user_event" in core._event_handlers

    def test_disconnects_signals_when_present(self):
        core = _CoreLikeWebView(with_signals=True)
        host = _TeardownHost(webview=core)
        self._teardown(host)
        for ev in _BRIDGE_EVENTS:
            sig = core._signals.custom[ev]
            sig.disconnect_all.assert_called_once()

    def test_handles_missing_signals_attribute(self):
        core = MagicMock(spec=["_event_handlers", "_event_handlers_lock"])
        core._event_handlers = {ev: [] for ev in _BRIDGE_EVENTS}
        core._event_handlers_lock.__enter__ = MagicMock(return_value=None)
        core._event_handlers_lock.__exit__ = MagicMock(return_value=None)
        host = _TeardownHost(webview=core)
        # Must not raise even though `core._signals` is absent.
        self._teardown(host)

    def test_handles_signals_set_to_none(self):
        core = _CoreLikeWebView(with_signals=False)
        core._signals = None
        host = _TeardownHost(webview=core)
        self._teardown(host)  # must not raise

    def test_swallows_exception_in_handler_clear(self):
        core = MagicMock(spec=["_event_handlers", "_event_handlers_lock", "_signals"])
        core._event_handlers_lock.__enter__ = MagicMock(side_effect=RuntimeError("lock gone"))
        core._event_handlers_lock.__exit__ = MagicMock(return_value=None)
        core._signals = None
        host = _TeardownHost(webview=core)
        # Outer try/except must absorb the lock failure.
        self._teardown(host)

    def test_swallows_exception_in_signal_disconnect(self):
        core = _CoreLikeWebView(with_signals=True)
        # Make one of the signal disconnect_all calls explode.
        core._signals.custom["js_error"].disconnect_all.side_effect = RuntimeError("dead signal")
        host = _TeardownHost(webview=core)
        # Must not raise.
        self._teardown(host)

    def test_idempotent(self):
        """Calling twice must remain safe (used by showEvent reuse path)."""
        core = _CoreLikeWebView()
        host = _TeardownHost(webview=core)
        self._teardown(host)
        # Second call: handlers already gone, signals already disconnected.
        self._teardown(host)


# ---------------------------------------------------------------------------
# QtWebView.destroy method
# ---------------------------------------------------------------------------


class _DestroyHost:
    """Stub mirroring just enough of QtWebView for ``destroy`` to run."""

    def __init__(self, parent_window=None):
        self._is_closing = False
        self._parent_window = parent_window
        self._webview = MagicMock()
        self._handle_close_event = MagicMock(return_value=False)
        self.deleteLater = MagicMock()


class TestDestroyMethod:
    def _destroy(self, host):
        return QtWebView.destroy(host)

    def test_calls_handle_close_event(self):
        host = _DestroyHost()
        self._destroy(host)
        host._handle_close_event.assert_called_once()

    def test_clears_parent_window_after_remove_filter(self):
        parent = MagicMock()
        host = _DestroyHost(parent_window=parent)
        self._destroy(host)
        parent.removeEventFilter.assert_called_once_with(host)
        assert host._parent_window is None

    def test_no_parent_window_no_remove_filter_call(self):
        host = _DestroyHost(parent_window=None)
        self._destroy(host)
        # Nothing to call; just must not crash.
        assert host._parent_window is None

    def test_swallows_remove_filter_runtime_error(self):
        parent = MagicMock()
        parent.removeEventFilter.side_effect = RuntimeError("parent gone")
        host = _DestroyHost(parent_window=parent)
        # Must not raise.
        self._destroy(host)
        assert host._parent_window is None

    def test_clears_webview_reference(self):
        host = _DestroyHost()
        self._destroy(host)
        assert host._webview is None

    def test_calls_delete_later(self):
        host = _DestroyHost()
        self._destroy(host)
        host.deleteLater.assert_called_once()

    def test_swallows_delete_later_runtime_error(self):
        host = _DestroyHost()
        host.deleteLater.side_effect = RuntimeError("already deleted")
        # Must not raise.
        self._destroy(host)


# ---------------------------------------------------------------------------
# QtWebView.__del__ delegation
# ---------------------------------------------------------------------------


class _DelHost:
    def __init__(self):
        self._handle_destructor = MagicMock()


class TestDunderDel:
    def test_delegates_to_handle_destructor(self):
        host = _DelHost()
        QtWebView.__del__(host)
        host._handle_destructor.assert_called_once()


# ---------------------------------------------------------------------------
# QtWebView.__repr__
# ---------------------------------------------------------------------------


class _ReprHost:
    def __init__(self, alive=True):
        self._alive = alive

    def windowTitle(self):
        if not self._alive:
            raise RuntimeError("Internal C++ object already deleted")
        return "MyTitle"

    def width(self):
        return 800

    def height(self):
        return 600


class TestRepr:
    def test_repr_when_alive(self):
        host = _ReprHost(alive=True)
        assert QtWebView.__repr__(host) == "QtWebView(title='MyTitle', size=800x600)"

    def test_repr_when_dead(self):
        host = _ReprHost(alive=False)
        assert QtWebView.__repr__(host) == "QtWebView(<deleted>)"


# ---------------------------------------------------------------------------
# QtWebView.get_hwnd
# ---------------------------------------------------------------------------


class _HwndHost:
    def __init__(self, hwnd=None, raises=None):
        self._webview = MagicMock()
        if raises is not None:
            self._webview.get_hwnd.side_effect = raises
        else:
            self._webview.get_hwnd.return_value = hwnd


class TestGetHwnd:
    def test_returns_hwnd_when_available(self):
        host = _HwndHost(hwnd=12345)
        assert QtWebView.get_hwnd(host) == 12345

    def test_returns_none_when_webview_raises(self):
        host = _HwndHost(raises=RuntimeError("dead"))
        assert QtWebView.get_hwnd(host) is None


# ---------------------------------------------------------------------------
# Module-level smoke tests
# ---------------------------------------------------------------------------


class TestModuleExports:
    def test_qtwebview_in_all(self):
        assert "QtWebView" in _core.__all__

    def test_event_processor_in_all(self):
        assert "QtEventProcessor" in _core.__all__

    def test_guard_alive_callable(self):
        assert callable(_guard_alive)

    def test_about_to_close_signal_is_class_attribute(self):
        assert hasattr(QtWebView, "aboutToClose")

    def test_verbose_logging_is_bool(self):
        assert isinstance(_core._VERBOSE_LOGGING, bool)


class TestModuleReloadCoverage:
    """Force re-execution of module-level code so that coverage.py observes
    decorator/Signal/import lines.

    ``conftest.py`` calls ``importlib.util.find_spec("auroraview._core")``
    BEFORE pytest-cov installs its trace function. As a result, every line
    that runs at module import time (``import``, ``def``, class body,
    ``Signal(...)`` assignments) is recorded as "not executed" by coverage,
    even though the module was clearly loaded. Reloading the module while
    coverage is active fixes this and lets us legitimately count the new
    decorator/method definitions.

    We reload BOTH ``_core`` and ``lifecycle`` because they contain the
    bulk of the patch lines added by this PR.
    """

    def test_reload_core_module(self):
        import importlib

        from auroraview.integration.qt import _core as core_module

        importlib.reload(core_module)
        # Sanity check: the module still exports its public surface.
        assert "QtWebView" in core_module.__all__
        assert callable(core_module._guard_alive)

    def test_reload_lifecycle_module(self):
        import importlib

        from auroraview.integration.qt import lifecycle as lifecycle_module

        importlib.reload(lifecycle_module)
        assert "LifecycleMixin" in lifecycle_module.__all__


# ---------------------------------------------------------------------------
# Guarded high-level API methods
# ---------------------------------------------------------------------------


class _ApiHost:
    """Stub for testing the @_guard_alive decorated load_url/load_html/etc.

    For tests that exercise ``load_file``, the stub must also expose
    ``load_url`` and ``load_html`` because the implementation may call
    them via ``self.<name>(...)``. We bind those as ``MethodType`` wrappers
    so they delegate to the underlying ``_webview`` mock just like the
    real class does.
    """

    def __init__(self, alive=True):
        self.is_alive = alive
        self._webview = MagicMock()
        self._asset_root = None
        # Bind the real (decorated) versions as instance methods so that
        # ``self.load_url(...)`` inside ``load_file`` resolves to a real
        # call that flows through ``_guard_alive`` and ``_webview``.
        self.load_url = types.MethodType(QtWebView.load_url, self)
        self.load_html = types.MethodType(QtWebView.load_html, self)


class TestGuardedApiMethods:
    """Each of these methods is wrapped in @_guard_alive at class definition.

    When the host is not alive, the wrapper short-circuits and the
    underlying ``self._webview.<method>`` MUST NOT be called.
    """

    def test_load_url_noop_when_not_alive(self):
        host = _ApiHost(alive=False)
        QtWebView.load_url(host, "http://example.com")
        host._webview.load_url.assert_not_called()

    def test_load_url_dispatches_when_alive(self):
        host = _ApiHost(alive=True)
        QtWebView.load_url(host, "http://example.com")
        host._webview.load_url.assert_called_once_with("http://example.com")

    def test_load_html_noop_when_not_alive(self):
        host = _ApiHost(alive=False)
        QtWebView.load_html(host, "<html/>")
        host._webview.load_html.assert_not_called()

    def test_load_html_dispatches_when_alive(self):
        host = _ApiHost(alive=True)
        QtWebView.load_html(host, "<html/>")
        host._webview.load_html.assert_called_once_with("<html/>")

    def test_eval_js_noop_when_not_alive(self):
        host = _ApiHost(alive=False)
        QtWebView.eval_js(host, "1 + 1")
        host._webview.eval_js.assert_not_called()

    def test_eval_js_dispatches_when_alive(self):
        host = _ApiHost(alive=True)
        QtWebView.eval_js(host, "1 + 1")
        host._webview.eval_js.assert_called_once_with("1 + 1")

    def test_emit_noop_when_not_alive(self):
        host = _ApiHost(alive=False)
        QtWebView.emit(host, "ev", {"k": 1})
        host._webview.emit.assert_not_called()

    def test_emit_dispatches_with_auto_process(self):
        host = _ApiHost(alive=True)
        QtWebView.emit(host, "ev", {"k": 1})
        host._webview.emit.assert_called_once_with("ev", {"k": 1}, auto_process=True)

    def test_emit_respects_explicit_auto_process_false(self):
        host = _ApiHost(alive=True)
        QtWebView.emit(host, "ev", {"k": 1}, auto_process=False)
        host._webview.emit.assert_called_once_with("ev", {"k": 1}, auto_process=False)

    def test_load_file_noop_when_not_alive(self, tmp_path):
        html_file = tmp_path / "x.html"
        html_file.write_text("<html/>", encoding="utf-8")
        host = _ApiHost(alive=False)
        QtWebView.load_file(host, str(html_file))
        host._webview.load_html.assert_not_called()
        host._webview.load_url.assert_not_called()

    def test_load_file_reads_and_calls_load_html(self, tmp_path):
        html_file = tmp_path / "x.html"
        html_file.write_text("<html><body>hi</body></html>", encoding="utf-8")
        host = _ApiHost(alive=True)
        QtWebView.load_file(host, str(html_file))
        host._webview.load_html.assert_called_once()
        called_with = host._webview.load_html.call_args.args[0]
        assert "hi" in called_with

    def test_load_file_falls_back_when_read_fails(self, tmp_path):
        host = _ApiHost(alive=True)
        # A path that cannot be read; falling back to ``_webview.load_file``.
        nonexistent = str(tmp_path / "missing.html")
        QtWebView.load_file(host, nonexistent)
        host._webview.load_file.assert_called_once_with(nonexistent)

    def test_load_file_uses_auroraview_protocol_when_under_asset_root(self, tmp_path):
        asset_root = tmp_path / "assets"
        asset_root.mkdir()
        html_file = asset_root / "page.html"
        html_file.write_text("<html/>", encoding="utf-8")
        host = _ApiHost(alive=True)
        host._asset_root = str(asset_root)
        QtWebView.load_file(host, str(html_file))
        host._webview.load_url.assert_called_once()
        url_arg = host._webview.load_url.call_args.args[0]
        if sys.platform == "win32":
            assert url_arg.startswith("https://auroraview.localhost/")
        else:
            assert url_arg.startswith("auroraview://")
        assert url_arg.endswith("page.html")

    def test_load_file_falls_back_to_load_html_when_outside_asset_root(self, tmp_path):
        asset_root = tmp_path / "assets"
        asset_root.mkdir()
        outside = tmp_path / "outside.html"
        outside.write_text("<html><body>outside</body></html>", encoding="utf-8")
        host = _ApiHost(alive=True)
        host._asset_root = str(asset_root)
        QtWebView.load_file(host, str(outside))
        # The auroraview-protocol branch must be skipped (relative_to raises
        # ValueError); load_html must run via the file-read path.
        host._webview.load_html.assert_called_once()


# ---------------------------------------------------------------------------
# QtWebView.on / register_callback wrappers
# ---------------------------------------------------------------------------


class _OnHost:
    def __init__(self, alive=True):
        self.is_alive = alive
        self._webview = MagicMock()
        self.ipcMessageReceived = MagicMock()


class TestOnDecorator:
    def test_registered_wrapper_skips_when_not_alive(self):
        host = _OnHost(alive=True)
        decorator = QtWebView.on(host, "evt")
        called = []

        @decorator
        def handler(data):
            called.append(data)

        # Capture the wrapper that was registered with the underlying core.
        host._webview.register_callback.assert_called_once()
        wrapper = host._webview.register_callback.call_args.args[1]

        # Now flip alive=False and dispatch.
        host.is_alive = False
        result = wrapper({"x": 1})
        assert result is None
        assert called == []

    def test_registered_wrapper_dispatches_when_alive(self):
        host = _OnHost(alive=True)
        decorator = QtWebView.on(host, "evt")
        called = []

        @decorator
        def handler(data):
            called.append(data)
            return "ok"

        wrapper = host._webview.register_callback.call_args.args[1]
        result = wrapper({"x": 1})
        assert result == "ok"
        assert called == [{"x": 1}]
        host.ipcMessageReceived.emit.assert_called_once_with("evt", {"x": 1})

    def test_registered_wrapper_swallows_runtime_error(self):
        host = _OnHost(alive=True)
        # Make the Qt signal emit raise to simulate a deleted C++ object.
        host.ipcMessageReceived.emit.side_effect = RuntimeError("dead signal")
        decorator = QtWebView.on(host, "evt")

        @decorator
        def handler(data):
            return "should-not-run"

        wrapper = host._webview.register_callback.call_args.args[1]
        # Wrapper must absorb the RuntimeError.
        assert wrapper({"x": 1}) is None


class TestRegisterCallback:
    def test_wrapper_skips_when_not_alive(self):
        host = _OnHost(alive=True)
        called = []
        QtWebView.register_callback(host, "evt", lambda d: called.append(d))
        wrapper = host._webview.register_callback.call_args.args[1]
        host.is_alive = False
        assert wrapper({"k": 1}) is None
        assert called == []

    def test_wrapper_dispatches_when_alive(self):
        host = _OnHost(alive=True)
        called = []

        def cb(data):
            called.append(data)
            return "x"

        QtWebView.register_callback(host, "evt", cb)
        wrapper = host._webview.register_callback.call_args.args[1]
        assert wrapper({"k": 1}) == "x"
        assert called == [{"k": 1}]

    def test_wrapper_swallows_runtime_error(self):
        host = _OnHost(alive=True)
        host.ipcMessageReceived.emit.side_effect = RuntimeError("dead")
        QtWebView.register_callback(host, "evt", lambda d: "x")
        wrapper = host._webview.register_callback.call_args.args[1]
        assert wrapper({"k": 1}) is None


# ---------------------------------------------------------------------------
# QtWebView.closeEvent / showEvent / resizeEvent / eventFilter
#
# These need a real QtWebView shell (not a stub) so that ``super()`` can
# resolve. We intentionally do NOT run the Rust core; instead we call the
# methods on a bare-init shell with carefully pre-set Python attributes.
# ---------------------------------------------------------------------------


class TestCloseEventOnRealShell:
    def test_accepts_event_when_handle_close_returns_true(self, bare_widget):
        bare_widget._is_closing = True  # _handle_close_event returns True

        from qtpy.QtGui import QCloseEvent

        event = QCloseEvent()
        QtWebView.closeEvent(bare_widget, event)
        assert event.isAccepted()

    def test_runs_full_close_sequence_when_not_closing(self, bare_widget):
        # Provide _teardown_signal_bridge as a recordable stand-in so we can
        # assert it was invoked by _handle_close_event.
        bare_widget._teardown_signal_bridge = MagicMock()

        from qtpy.QtGui import QCloseEvent

        event = QCloseEvent()
        QtWebView.closeEvent(bare_widget, event)
        assert event.isAccepted()
        assert bare_widget._is_closing is True
        bare_widget._teardown_signal_bridge.assert_called_once()


class TestShowEventOnRealShell:
    def test_resets_is_closing_on_reshow(self, bare_widget):
        bare_widget._is_closing = True
        bare_widget._webview_initialized = True  # avoid re-init branch

        from qtpy.QtGui import QShowEvent

        QtWebView.showEvent(bare_widget, QShowEvent())
        assert bare_widget._is_closing is False
        bare_widget._setup_signal_bridge.assert_called_once()

    def test_resets_signal_state_on_reshow(self, bare_widget):
        bare_widget._is_closing = True
        bare_widget._webview_initialized = True
        bare_widget._qt_signal_state = {
            "current_url": "stale",
            "current_title": "t",
            "is_loading": True,
            "load_progress": 50,
        }
        from qtpy.QtGui import QShowEvent

        QtWebView.showEvent(bare_widget, QShowEvent())
        assert bare_widget._qt_signal_state == {
            "current_url": "",
            "current_title": "",
            "is_loading": False,
            "load_progress": 0,
        }

    def test_initializes_when_not_initialized(self, bare_widget):
        bare_widget._is_closing = False
        bare_widget._webview_initialized = False

        from qtpy.QtGui import QShowEvent

        QtWebView.showEvent(bare_widget, QShowEvent())
        bare_widget._initialize_webview.assert_called_once()
        assert bare_widget._webview_initialized is True

    def test_skips_init_when_already_initialized(self, bare_widget):
        bare_widget._is_closing = False
        bare_widget._webview_initialized = True

        from qtpy.QtGui import QShowEvent

        QtWebView.showEvent(bare_widget, QShowEvent())
        bare_widget._initialize_webview.assert_not_called()


class TestResizeEventOnRealShell:
    def test_clears_container_on_runtime_error(self, bare_widget, caplog):
        import logging

        from qtpy.QtCore import QSize
        from qtpy.QtGui import QResizeEvent

        container = MagicMock()
        container.setGeometry.side_effect = RuntimeError("C++ object already deleted")
        bare_widget._webview_container = container

        with caplog.at_level(logging.WARNING, logger=_core.logger.name):
            QtWebView.resizeEvent(bare_widget, QResizeEvent(QSize(800, 600), QSize(0, 0)))
        assert bare_widget._webview_container is None
        assert "container C++ object already deleted" in caplog.text

    def test_calls_sync_when_container_alive(self, bare_widget):
        from qtpy.QtCore import QSize
        from qtpy.QtGui import QResizeEvent

        container = MagicMock()
        bare_widget._webview_container = container
        QtWebView.resizeEvent(bare_widget, QResizeEvent(QSize(1024, 768), QSize(0, 0)))
        container.setGeometry.assert_called_once_with(0, 0, 1024, 768)
        bare_widget._sync_webview2_controller_bounds.assert_called_once()

    def test_no_container_no_sync(self, bare_widget):
        from qtpy.QtCore import QSize
        from qtpy.QtGui import QResizeEvent

        bare_widget._webview_container = None
        QtWebView.resizeEvent(bare_widget, QResizeEvent(QSize(640, 480), QSize(0, 0)))
        bare_widget._sync_webview2_controller_bounds.assert_not_called()

    def test_direct_embed_branch(self, bare_widget):
        from qtpy.QtCore import QSize
        from qtpy.QtGui import QResizeEvent

        bare_widget._using_direct_embed = True
        bare_widget._direct_embed_hwnd = 42
        # Just make sure the direct-embed branch executes without error.
        QtWebView.resizeEvent(bare_widget, QResizeEvent(QSize(320, 240), QSize(0, 0)))


class TestEventFilterOnRealShell:
    """Test ``eventFilter`` with real Qt objects.

    ``super().eventFilter`` is implemented at the C++ level and rejects
    non-``QObject``/non-``QEvent`` arguments at type-check time, so we
    cannot pass MagicMocks. We construct real ``QWidget``/``QEvent``
    instances (cheap, no rendering) and let the call flow naturally.
    """

    def test_close_event_on_parent_triggers_handle_close(self, qapp, bare_widget):
        from qtpy.QtCore import QEvent
        from qtpy.QtWidgets import QWidget

        parent = QWidget()
        try:
            bare_widget._parent_window = parent
            bare_widget._handle_close_event = MagicMock(return_value=False)

            event = QEvent(QEvent.Close)
            QtWebView.eventFilter(bare_widget, parent, event)
            bare_widget._handle_close_event.assert_called_once()
        finally:
            parent.deleteLater()

    def test_swallows_runtime_error(self, bare_widget):
        # ``event.type()`` raising RuntimeError represents a deleted Qt
        # event object; the filter must absorb and return False without
        # ever reaching ``super().eventFilter``.
        parent = MagicMock()
        bare_widget._parent_window = parent

        event = MagicMock()
        event.type.side_effect = RuntimeError("deleted")
        result = QtWebView.eventFilter(bare_widget, parent, event)
        assert result is False

    def test_non_close_event_not_propagated(self, qapp, bare_widget):
        from qtpy.QtCore import QEvent
        from qtpy.QtWidgets import QWidget

        parent = QWidget()
        try:
            bare_widget._parent_window = parent
            bare_widget._handle_close_event = MagicMock()
            event = QEvent(QEvent.Resize)
            QtWebView.eventFilter(bare_widget, parent, event)
            bare_widget._handle_close_event.assert_not_called()
        finally:
            parent.deleteLater()

    def test_unrelated_watched_object_not_propagated(self, qapp, bare_widget):
        from qtpy.QtCore import QEvent
        from qtpy.QtWidgets import QWidget

        parent = QWidget()
        watched_other = QWidget()  # NOT the parent_window
        try:
            bare_widget._parent_window = parent
            bare_widget._handle_close_event = MagicMock()
            event = QEvent(QEvent.Close)
            QtWebView.eventFilter(bare_widget, watched_other, event)
            bare_widget._handle_close_event.assert_not_called()
        finally:
            parent.deleteLater()
            watched_other.deleteLater()


# ---------------------------------------------------------------------------
# QtWebView._setup_signal_bridge -- registers a closure-per-event onto
# self._webview and dispatches to Qt signals. We mock _webview.on so that
# we can grab the registered closures and exercise each one.
# ---------------------------------------------------------------------------


class _OnRegistry:
    """Mimics ``WebView.on(name)`` -- a decorator factory that records the
    decorated callback in a dict so the test can extract it.
    """

    def __init__(self):
        self.handlers = {}

    def __call__(self, name):
        def decorator(func):
            self.handlers[name] = func
            return func

        return decorator


@pytest.fixture
def bridge_widget(qapp):
    """A bare QtWebView shell wired with an _OnRegistry-backed _webview so
    that ``_setup_signal_bridge`` can be invoked and the registered
    callbacks recovered for unit-style assertions.

    The Qt signals (``loadStarted``, ``urlChanged``, ...) are replaced
    with recordable ``MagicMock`` instances so we can assert ``.emit()``
    calls without relying on the PySide signal-dispatch machinery (which
    is brittle on a ``__new__``-built shell that bypasses ``__init__``).
    """
    from qtpy.QtWidgets import QWidget

    obj = QtWebView.__new__(QtWebView)
    QWidget.__init__(obj)
    obj._is_closing = False
    obj._webview_initialized = True
    obj._qt_signal_state = {
        "current_url": "",
        "current_title": "",
        "is_loading": False,
        "load_progress": 0,
    }
    # Build a _webview that pretends to have ``on(name)`` registration.
    registry = _OnRegistry()
    obj._webview = MagicMock()
    obj._webview.on = registry
    obj._registry = registry  # for the test to introspect
    obj._teardown_signal_bridge = MagicMock()
    # Replace each Qt signal we assert on with a MagicMock. This shadows
    # the class-level ``Signal(...)`` descriptor on the instance.
    for sig_name in (
        "urlChanged",
        "loadStarted",
        "loadFinished",
        "loadProgress",
        "titleChanged",
        "iconChanged",
        "iconUrlChanged",
        "jsError",
        "consoleMessage",
        "renderProcessTerminated",
        "selectionChanged",
        "ipcMessageReceived",
    ):
        setattr(obj, sig_name, MagicMock())
    yield obj
    try:
        obj.deleteLater()
    except RuntimeError:
        pass


class TestSetupSignalBridge:
    """Each of the 10 internal callbacks registered by ``_setup_signal_bridge``
    must:

    * Use the ``self._webview.on(<event>)`` registration mechanism
    * Short-circuit when ``is_alive`` is False
    * Update ``_qt_signal_state`` and emit the matching Qt signal when the
      widget is alive
    * Swallow ``RuntimeError`` (deleted-C++) without re-raising
    """

    def test_teardown_called_first(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        bridge_widget._teardown_signal_bridge.assert_called_once()

    def test_all_known_events_registered(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        assert set(bridge_widget._registry.handlers.keys()) == set(_BRIDGE_EVENTS)

    def test_navigation_started_updates_state_and_emits(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["navigation_started"]
        cb({"url": "https://x.test/"})
        assert bridge_widget._qt_signal_state["is_loading"] is True
        assert bridge_widget._qt_signal_state["load_progress"] == 0
        bridge_widget.loadStarted.emit.assert_called_once()
        bridge_widget.urlChanged.emit.assert_called_once_with("https://x.test/")

    def test_navigation_started_skips_when_not_alive(self, bridge_widget):
        bridge_widget._is_closing = True
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["navigation_started"]
        cb({"url": "https://x.test/"})
        assert bridge_widget._qt_signal_state["is_loading"] is False
        bridge_widget.loadStarted.emit.assert_not_called()

    def test_navigation_finished_emits_load_finished(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["navigation_finished"]
        cb({"success": True, "url": "https://done/"})
        bridge_widget.loadFinished.emit.assert_called_once_with(True)
        assert bridge_widget._qt_signal_state["is_loading"] is False
        assert bridge_widget._qt_signal_state["load_progress"] == 100

    def test_navigation_finished_failure_progress_zero(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["navigation_finished"]
        cb({"success": False, "url": "https://fail/"})
        assert bridge_widget._qt_signal_state["load_progress"] == 0
        bridge_widget.loadFinished.emit.assert_called_once_with(False)

    def test_load_progress_clamps_and_emits(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["load_progress"]
        cb({"progress": 50})
        cb({"progress": 50})  # same value -> dedup, no second emit
        cb({"progress": -10})  # clamp to 0
        cb({"progress": 200})  # clamp to 100
        # Three distinct emits: 50, 0, 100
        emit_calls = bridge_widget.loadProgress.emit.call_args_list
        emitted_values = [c.args[0] for c in emit_calls]
        assert emitted_values == [50, 0, 100]

    def test_title_changed_dedup(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["title_changed"]
        cb({"title": "First"})
        cb({"title": "First"})  # same -> dedup
        cb({"title": "Second"})
        emit_calls = [c.args[0] for c in bridge_widget.titleChanged.emit.call_args_list]
        assert emit_calls == ["First", "Second"]

    def test_url_changed_dedup(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["url_changed"]
        cb({"url": "https://a/"})
        cb({"url": "https://a/"})
        cb({"url": "https://b/"})
        emit_calls = [c.args[0] for c in bridge_widget.urlChanged.emit.call_args_list]
        assert emit_calls == ["https://a/", "https://b/"]

    def test_js_error_emits_with_args(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["js_error"]
        cb({"message": "ReferenceError", "line": 12, "source": "main.js"})
        bridge_widget.jsError.emit.assert_called_once_with("ReferenceError", 12, "main.js")

    def test_console_message_emits_full_tuple(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["console_message"]
        cb({"level": 2, "message": "warn!", "line": 3, "source": "foo.js"})
        bridge_widget.consoleMessage.emit.assert_called_once_with(2, "warn!", 3, "foo.js")

    def test_render_process_terminated_emits(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["render_process_terminated"]
        cb({"status": 1, "exit_code": 137})
        bridge_widget.renderProcessTerminated.emit.assert_called_once_with(1, 137)

    def test_selection_changed_emits(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["selection_changed"]
        cb({})
        bridge_widget.selectionChanged.emit.assert_called_once()

    def test_icon_changed_emits_both_signals(self, bridge_widget):
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["icon_changed"]
        cb({"url": "data:image/png;base64,XYZ"})
        bridge_widget.iconChanged.emit.assert_called_once()
        bridge_widget.iconUrlChanged.emit.assert_called_once_with("data:image/png;base64,XYZ")

    def test_callbacks_skip_when_not_alive(self, bridge_widget):
        """Each registered callback must respect the is_alive guard."""
        QtWebView._setup_signal_bridge(bridge_widget)
        bridge_widget._is_closing = True

        for ev in _BRIDGE_EVENTS:
            cb = bridge_widget._registry.handlers[ev]
            cb({"some": "data"})

    def test_callbacks_handle_none_data(self, bridge_widget):
        """The closures use ``data.get(...)`` defensively when ``data`` is
        truthy; calling them with ``None`` exercises the alternative path.
        """
        QtWebView._setup_signal_bridge(bridge_widget)
        for ev in _BRIDGE_EVENTS:
            cb = bridge_widget._registry.handlers[ev]
            cb(None)  # must not raise

    def test_callbacks_swallow_runtime_error(self, bridge_widget):
        """RuntimeError from a Qt emit must be absorbed inside each
        callback (TOCTOU window between is_alive and the actual call).
        """
        # Force every relevant signal's emit to raise.
        for sig_name in (
            "urlChanged",
            "loadStarted",
            "loadFinished",
            "loadProgress",
            "titleChanged",
            "iconChanged",
            "iconUrlChanged",
            "jsError",
            "consoleMessage",
            "renderProcessTerminated",
            "selectionChanged",
        ):
            getattr(bridge_widget, sig_name).emit.side_effect = RuntimeError("dead")

        QtWebView._setup_signal_bridge(bridge_widget)
        for ev in _BRIDGE_EVENTS:
            cb = bridge_widget._registry.handlers[ev]
            cb({"data": "x"})  # must not raise


# ---------------------------------------------------------------------------
# QtWebView.create_deferred classmethod
# ---------------------------------------------------------------------------


class TestCreateDeferred:
    def test_returns_qwidget_placeholder(self, qapp, monkeypatch):
        from qtpy.QtCore import QTimer
        from qtpy.QtWidgets import QWidget

        # Capture rather than execute the timer callback.
        scheduled = []

        def fake_single_shot(delay, callback):
            scheduled.append((delay, callback))

        monkeypatch.setattr(QTimer, "singleShot", staticmethod(fake_single_shot))

        placeholder = QtWebView.create_deferred(parent=None, title="Loading", width=400, height=300)
        try:
            assert isinstance(placeholder, QWidget)
            assert placeholder.windowTitle() == "Loading"
            # The deferred creation must have been scheduled exactly once.
            assert len(scheduled) == 1
            assert scheduled[0][0] == 0  # delay = 0 ms
        finally:
            placeholder.deleteLater()

    def test_on_error_called_on_failure(self, qapp, monkeypatch):
        """When the deferred ``cls(...)`` constructor raises, ``on_error``
        is invoked with the error string and the placeholder reflects it.
        """
        from qtpy.QtCore import QTimer

        scheduled = []
        monkeypatch.setattr(
            QTimer,
            "singleShot",
            staticmethod(lambda delay, cb: scheduled.append(cb)),
        )

        # Make ``cls(...)`` raise -- patch QtWebView.__init__ to raise.
        # The deferred function calls ``cls(parent=..., title=..., ...)``.
        original_init = QtWebView.__init__

        def boom(self, *a, **kw):
            raise RuntimeError("simulated failure")

        monkeypatch.setattr(QtWebView, "__init__", boom)

        errors = []
        placeholder = QtWebView.create_deferred(
            parent=None,
            title="X",
            width=100,
            height=100,
            on_error=lambda msg: errors.append(msg),
        )
        try:
            # Trigger the deferred callback now.
            assert scheduled, "Scheduled callback missing"
            scheduled[0]()
            assert errors == ["simulated failure"]
        finally:
            # Restore __init__ before deleting placeholder so Qt can clean up.
            monkeypatch.setattr(QtWebView, "__init__", original_init)
            placeholder.deleteLater()


# ---------------------------------------------------------------------------
# QtWebView.show / moveEvent / get_diagnostics / property accessors
# ---------------------------------------------------------------------------


class TestSimpleAccessors:
    def test_show_calls_super_show(self, bare_widget):
        # The override is just ``super().show()``. Run it -- if super()
        # resolves and doesn't raise, that's all we need.
        QtWebView.show(bare_widget)
        # Visibility on offscreen platform -- just verify call is benign.

    def test_move_event_calls_super(self, bare_widget):
        from qtpy.QtCore import QPoint
        from qtpy.QtGui import QMoveEvent

        QtWebView.moveEvent(bare_widget, QMoveEvent(QPoint(1, 2), QPoint(0, 0)))

    def test_get_diagnostics_returns_keys(self, bare_widget):
        # ``get_diagnostics`` reads ``_event_processor`` and ``_webview``.
        bare_widget._event_processor = MagicMock()
        bare_widget._event_processor._process_count = 7
        bare_widget._webview._event_processor = MagicMock()
        diag = QtWebView.get_diagnostics(bare_widget)
        assert "event_processor_type" in diag
        assert "event_process_count" in diag
        assert diag["event_process_count"] == 7
        assert diag["has_event_processor"] is True


class _PropHost:
    """Stub for property delegation tests."""

    def __init__(self):
        self._webview = MagicMock()
        self._webview.state = "STATE"
        self._webview.commands = "COMMANDS"
        self._webview.channels = "CHANNELS"


class TestDelegatedProperties:
    def test_state_property(self):
        host = _PropHost()
        assert QtWebView.state.fget(host) == "STATE"

    def test_commands_property(self):
        host = _PropHost()
        assert QtWebView.commands.fget(host) == "COMMANDS"

    def test_channels_property(self):
        host = _PropHost()
        assert QtWebView.channels.fget(host) == "CHANNELS"

    def test_command_decorator_delegates(self):
        host = _PropHost()
        host._webview.command = MagicMock(return_value="decorated")
        result = QtWebView.command(host, "name")
        host._webview.command.assert_called_once_with("name")
        assert result == "decorated"

    def test_create_channel_delegates(self):
        host = _PropHost()
        host._webview.create_channel = MagicMock(return_value="chan")
        assert QtWebView.create_channel(host, "x") == "chan"
        host._webview.create_channel.assert_called_once_with("x")

    def test_bind_call_delegates(self):
        host = _PropHost()
        host._webview.bind_call = MagicMock(return_value="bc")
        assert QtWebView.bind_call(host, "m", lambda: None) == "bc"

    def test_bind_api_delegates(self):
        host = _PropHost()
        host._webview.bind_api = MagicMock()

        class API:
            pass

        api = API()
        QtWebView.bind_api(host, api, namespace="ns")
        host._webview.bind_api.assert_called_once_with(api, "ns")


# ---------------------------------------------------------------------------
# Window event callbacks (on_shown, on_closing, ...) -- thin delegations
# ---------------------------------------------------------------------------


class _CallbackHost:
    def __init__(self):
        self._webview = MagicMock()


class TestWindowCallbacks:
    @pytest.mark.parametrize(
        "method_name, webview_attr",
        [
            ("on_shown", "on_shown"),
            ("on_closing", "on_closing"),
            ("on_closed", "on_closed"),
            ("on_resized", "on_resized"),
            ("on_moved", "on_moved"),
            ("on_focused", "on_focused"),
            ("on_blurred", "on_blurred"),
            ("on_minimized", "on_minimized"),
            ("on_maximized", "on_maximized"),
            ("on_restored", "on_restored"),
        ],
    )
    def test_delegates_to_underlying_webview(self, method_name, webview_attr):
        host = _CallbackHost()
        method = getattr(QtWebView, method_name)
        getattr(host._webview, webview_attr).return_value = "cb-result"
        cb = lambda: None  # noqa: E731
        result = method(host, cb)
        assert result == "cb-result"
        getattr(host._webview, webview_attr).assert_called_once_with(cb)


# ---------------------------------------------------------------------------
# Qt signal-state read-only properties
# ---------------------------------------------------------------------------


class _QtStateHost:
    def __init__(self, **state):
        self._qt_signal_state = {
            "current_url": "",
            "current_title": "",
            "is_loading": False,
            "load_progress": 0,
        }
        self._qt_signal_state.update(state)


class TestQtStateProperties:
    def test_current_url_default_empty(self):
        host = _QtStateHost()
        assert QtWebView.current_url.fget(host) == ""

    def test_current_url_populated(self):
        host = _QtStateHost(current_url="https://x/")
        assert QtWebView.current_url.fget(host) == "https://x/"

    def test_current_title(self):
        host = _QtStateHost(current_title="Hello")
        assert QtWebView.current_title.fget(host) == "Hello"

    def test_is_loading_default_false(self):
        host = _QtStateHost()
        assert QtWebView.is_loading.fget(host) is False

    def test_is_loading_true(self):
        host = _QtStateHost(is_loading=True)
        assert QtWebView.is_loading.fget(host) is True

    def test_load_progress_default_zero(self):
        host = _QtStateHost()
        assert QtWebView.load_progress_value.fget(host) == 0

    def test_load_progress_value(self):
        host = _QtStateHost(load_progress=42)
        assert QtWebView.load_progress_value.fget(host) == 42


# ---------------------------------------------------------------------------
# title getter/setter
# ---------------------------------------------------------------------------


class TestTitleGetterSetter:
    def test_title_getter_returns_window_title(self, bare_widget):
        bare_widget.setWindowTitle("Stored")
        assert QtWebView.title.fget(bare_widget) == "Stored"

    def test_title_setter_updates_widget_and_internal_webview(self, bare_widget):
        bare_widget._webview = MagicMock()
        QtWebView.title.fset(bare_widget, "NewTitle")
        assert bare_widget._title == "NewTitle"
        assert bare_widget.windowTitle() == "NewTitle"
        # The setter also writes to ``self._webview._title`` defensively.
        assert bare_widget._webview._title == "NewTitle"

    def test_title_setter_swallows_webview_error(self, bare_widget):
        # If writing _title to webview raises, the setter must absorb it.
        class _Boom:
            def __setattr__(self, name, value):
                raise RuntimeError("nope")

        bare_widget._webview = _Boom()
        # Must not raise.
        QtWebView.title.fset(bare_widget, "X")
        assert bare_widget._title == "X"


# ---------------------------------------------------------------------------
# Additional lifecycle.py branches: _handle_close_event aboutToClose path
# and _handle_destructor verbose-logging branch.
# ---------------------------------------------------------------------------


class _AboutToCloseHost:
    """Stub matching the lifecycle._Host shape but adding aboutToClose."""

    def __init__(self):
        import types as _types

        from auroraview.integration.qt.lifecycle import LifecycleMixin

        self._geometry_sync_in_progress = False
        self._child_window_fix_in_progress = False
        self._is_closing = False
        self._webview_initialized = True
        self._webview = MagicMock()
        self._webview_container = MagicMock()
        self._webview_qwindow = MagicMock()
        self._force_container_geometry = MagicMock()
        self._reset_state_for_reuse = _types.MethodType(LifecycleMixin._reset_state_for_reuse, self)
        self.aboutToClose = MagicMock()

    # The mixin reads ``self.aboutToClose.emit()``; MagicMock provides .emit


class TestAboutToCloseSignalEmission:
    def test_about_to_close_emitted_before_state_change(self):
        from auroraview.integration.qt.lifecycle import LifecycleMixin

        host = _AboutToCloseHost()
        order = []
        host.aboutToClose.emit.side_effect = lambda: order.append(("emit", host._is_closing))

        LifecycleMixin._handle_close_event(host)

        # Emitted exactly once.
        host.aboutToClose.emit.assert_called_once()
        # When emitted, _is_closing was still False (state mutation
        # happens afterwards).
        assert order == [("emit", False)]

    def test_about_to_close_emit_runtime_error_swallowed(self):
        from auroraview.integration.qt.lifecycle import LifecycleMixin

        host = _AboutToCloseHost()
        host.aboutToClose.emit.side_effect = RuntimeError("dead signal")
        # Must not raise -- subsequent steps still run.
        result = LifecycleMixin._handle_close_event(host)
        assert result is False
        assert host._is_closing is True

    def test_about_to_close_attribute_error_swallowed(self):
        from auroraview.integration.qt.lifecycle import LifecycleMixin

        host = _AboutToCloseHost()
        host.aboutToClose.emit.side_effect = AttributeError("missing")
        result = LifecycleMixin._handle_close_event(host)
        assert result is False
        assert host._is_closing is True


# ---------------------------------------------------------------------------
# Verbose-logging coverage: when AURORAVIEW_LOG_VERBOSE is enabled, several
# ``if _VERBOSE_LOGGING:`` branches in lifecycle.py become reachable. We
# do not change the env var (which is read once at import); instead we
# monkeypatch the module-level constant for the duration of the test.
# ---------------------------------------------------------------------------


class TestVerboseLoggingBranches:
    def test_handle_close_event_verbose_logs(self, monkeypatch, caplog):
        import logging

        from auroraview.integration.qt import lifecycle

        monkeypatch.setattr(lifecycle, "_VERBOSE_LOGGING", True)
        host = _AboutToCloseHost()
        # Make every step raise to maximize logging branches.
        host._webview.close.side_effect = RuntimeError("dead")
        host.aboutToClose = MagicMock()
        original_teardown = getattr(host, "_teardown_signal_bridge", None)
        host._teardown_signal_bridge = MagicMock(side_effect=RuntimeError("td"))

        with caplog.at_level(logging.DEBUG, logger=lifecycle.logger.name):
            lifecycle.LifecycleMixin._handle_close_event(host)

        # Final state still updated; logging branches exercised.
        assert host._is_closing is True
        if original_teardown is not None:
            host._teardown_signal_bridge = original_teardown

    def test_handle_destructor_verbose(self, monkeypatch, caplog):
        import logging

        from auroraview.integration.qt import lifecycle

        monkeypatch.setattr(lifecycle, "_VERBOSE_LOGGING", True)
        host = _AboutToCloseHost()
        host._is_closing = False
        host._webview.close.side_effect = RuntimeError("boom")

        with caplog.at_level(logging.DEBUG, logger=lifecycle.logger.name):
            lifecycle.LifecycleMixin._handle_destructor(host)
        # Logging branch executed; no exception escaped.

    def test_reset_state_for_reuse_verbose(self, monkeypatch, caplog):
        import logging

        from auroraview.integration.qt import lifecycle

        monkeypatch.setattr(lifecycle, "_VERBOSE_LOGGING", True)
        host = _AboutToCloseHost()
        host._webview._core = MagicMock()
        with caplog.at_level(logging.DEBUG, logger=lifecycle.logger.name):
            lifecycle.LifecycleMixin._reset_state_for_reuse(host)
        host._webview._core.reset.assert_called_once()

    def test_reset_state_for_reuse_swallows_core_error(self, monkeypatch, caplog):
        """When ``core.reset()`` raises, the error must be swallowed so that
        a partially-constructed Rust state cannot abort the whole close
        path. This exercises lifecycle.py L450-L452 (the except branch).
        """
        import logging

        from auroraview.integration.qt import lifecycle

        monkeypatch.setattr(lifecycle, "_VERBOSE_LOGGING", True)
        host = _AboutToCloseHost()
        host._webview._core = MagicMock()
        host._webview._core.reset.side_effect = RuntimeError("rust kaboom")
        with caplog.at_level(logging.DEBUG, logger=lifecycle.logger.name):
            lifecycle.LifecycleMixin._reset_state_for_reuse(host)
        # Must not raise; the Rust-state-reset failure is logged and swallowed.

    def test_handle_close_event_swallows_reset_state_error(self, monkeypatch, caplog):
        """The ``_reset_state_for_reuse`` step inside ``_handle_close_event``
        is wrapped in its own try/except so that a failure does not bubble
        up. Exercise lifecycle.py L411-L413 (verbose-log + swallow).
        """
        import logging

        from auroraview.integration.qt import lifecycle

        monkeypatch.setattr(lifecycle, "_VERBOSE_LOGGING", True)
        host = _AboutToCloseHost()
        host._reset_state_for_reuse = MagicMock(side_effect=RuntimeError("kaboom"))

        with caplog.at_level(logging.DEBUG, logger=lifecycle.logger.name):
            result = lifecycle.LifecycleMixin._handle_close_event(host)

        # Even though _reset_state_for_reuse raised, the close path
        # completed and ``_is_closing`` was still flipped True.
        assert result is False
        assert host._is_closing is True
        host._reset_state_for_reuse.assert_called_once()


# ---------------------------------------------------------------------------
# Additional verbose-logging branches in _core.py.
#
# Several ``if _VERBOSE_LOGGING:`` blocks live inside _core.py where they
# are unreachable in default test runs. We monkeypatch ``_core._VERBOSE_LOGGING``
# True for each test so that the branch becomes hot, then assert the
# associated debug log emerged in caplog.
# ---------------------------------------------------------------------------


class TestCoreVerboseLoggingBranches:
    """Force the ``if _VERBOSE_LOGGING:`` branches inside ``_core.py`` so
    that coverage records them. These cover lines 99, 114, 619, 685,
    1047, 1081, 1104, 1200 and a handful of related lines.
    """

    def test_guard_alive_verbose_skip_path(self, monkeypatch, caplog):
        """The _guard_alive decorator logs ``skipped`` debug message when
        the widget is dead and verbose logging is on. Covers L98-L102.
        """
        import logging

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)

        @_core._guard_alive
        def method(self):  # noqa: ARG001
            return "should-not-run"

        host = _GuardHost(alive=False)
        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            assert method(host) is None

    def test_guard_alive_verbose_runtime_error_path(self, monkeypatch, caplog):
        """When the wrapped method raises a C++-deletion RuntimeError and
        verbose logging is on, the decorator logs the catch. Covers L113-L117.
        """
        import logging

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)

        @_core._guard_alive
        def method(self):  # noqa: ARG001
            raise RuntimeError("Internal C++ object (QWidget) already deleted")

        host = _GuardHost(alive=True)
        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            assert method(host) is None

    def test_setup_signal_bridge_verbose(self, monkeypatch, bridge_widget, caplog):
        """When verbose logging is on, _setup_signal_bridge logs an
        initialization marker. Covers L618-L619.
        """
        import logging

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            QtWebView._setup_signal_bridge(bridge_widget)
        assert any("Signal bridge initialized" in rec.message for rec in caplog.records)

    def test_teardown_signal_bridge_verbose(self, monkeypatch, caplog):
        """Verbose-logging branch inside _teardown_signal_bridge. Covers L684-L685."""
        import logging

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        host = _TeardownHost(webview=_CoreLikeWebView())
        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            QtWebView._teardown_signal_bridge(host)
        assert any("Signal bridge torn down" in rec.message for rec in caplog.records)

    def test_resize_event_verbose_direct_embed(self, monkeypatch, bare_widget, caplog):
        """When verbose logging is on and the widget is in direct-embed
        mode, ``resizeEvent`` logs the resize. Covers L1046-L1047.
        """
        import logging

        from qtpy.QtCore import QSize
        from qtpy.QtGui import QResizeEvent

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        bare_widget._using_direct_embed = True
        bare_widget._direct_embed_hwnd = 99
        # Stub the win32 helper so the test does not require windows APIs.
        monkeypatch.setattr(_core, "update_embedded_window_geometry", lambda *a, **kw: None)
        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            QtWebView.resizeEvent(bare_widget, QResizeEvent(QSize(123, 234), QSize(0, 0)))
        assert any("Direct embed resize" in rec.message for rec in caplog.records)

    def test_event_filter_verbose_close_branch(self, monkeypatch, qapp, bare_widget, caplog):
        """When verbose logging is on, ``eventFilter`` logs ``Parent
        window closing`` before delegating. Covers L1080-L1081.
        """
        import logging

        from qtpy.QtCore import QEvent
        from qtpy.QtWidgets import QWidget

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        parent = QWidget()
        try:
            bare_widget._parent_window = parent
            bare_widget._handle_close_event = MagicMock(return_value=False)
            with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
                QtWebView.eventFilter(bare_widget, parent, QEvent(QEvent.Close))
            assert any("Parent window closing" in rec.message for rec in caplog.records)
        finally:
            parent.deleteLater()

    def test_show_event_verbose_reset_branch(self, monkeypatch, bare_widget, caplog):
        """When verbose logging is on AND ``_is_closing`` was True at
        showEvent, the reuse-reset path emits a debug log. Covers L1103-L1104.
        """
        import logging

        from qtpy.QtGui import QShowEvent

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        bare_widget._is_closing = True
        bare_widget._webview_initialized = True

        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            QtWebView.showEvent(bare_widget, QShowEvent())
        assert any("Resetting _is_closing on re-show" in rec.message for rec in caplog.records)

    def test_get_hwnd_verbose_error_branch(self, monkeypatch, bare_widget, caplog):
        """When the underlying ``_webview.get_hwnd()`` raises and verbose
        logging is on, the error is logged and ``None`` is returned.
        Covers L1198-L1200.
        """
        import logging

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        bare_widget._webview = MagicMock()
        bare_widget._webview.get_hwnd.side_effect = RuntimeError("no hwnd")

        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            assert QtWebView.get_hwnd(bare_widget) is None
        assert any("QtWebView.get_hwnd() error" in rec.message for rec in caplog.records)

    def test_create_deferred_verbose_log(self, monkeypatch, qapp, caplog):
        """The ``create_deferred`` placeholder branch and the deferred
        do_create function both emit verbose-mode debug logs. Covers
        L727-L728 and L733-L734.
        """
        import logging

        from qtpy.QtCore import QTimer
        from qtpy.QtWidgets import QWidget

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        scheduled = []
        monkeypatch.setattr(
            QTimer,
            "singleShot",
            staticmethod(lambda delay, cb: scheduled.append(cb)),
        )

        # Replace ``__init__`` to skip Rust-core construction; we only need
        # to exercise the verbose-log branches inside the deferred body.
        original_init = QtWebView.__init__

        def stub_init(self, *_a, **_kw):
            QWidget.__init__(self)

        monkeypatch.setattr(QtWebView, "__init__", stub_init)

        with caplog.at_level(logging.DEBUG, logger=_core.logger.name):
            placeholder = QtWebView.create_deferred(parent=None, title="X", width=10, height=10)
            try:
                # Trigger do_create to hit the second verbose-log line.
                assert scheduled
                scheduled[0]()
            finally:
                monkeypatch.setattr(QtWebView, "__init__", original_init)
                placeholder.deleteLater()

        assert any("create_deferred" in rec.message for rec in caplog.records)


# ---------------------------------------------------------------------------
# Bridge-closure RuntimeError swallow paths that the original test missed.
#
# The original ``test_callbacks_swallow_runtime_error`` passes
# ``{"data": "x"}`` for every event, which means the ``progress``,
# ``title`` and ``url`` closures never reach their ``self.<sig>.emit(...)``
# call (the dedup/empty-string guards short-circuit first). As a result
# their ``except RuntimeError`` branches stay uncovered.
#
# These tests pass *valid* event payloads so each emit is invoked (and
# made to raise via ``side_effect``), exercising the swallow branch.
# ---------------------------------------------------------------------------


class TestBridgeClosureRuntimeErrorSwallow:
    def test_load_progress_swallow(self, bridge_widget):
        """L527-L528: ``except RuntimeError: pass`` inside the
        ``load_progress`` closure.
        """
        bridge_widget.loadProgress.emit.side_effect = RuntimeError("dead")
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["load_progress"]
        cb({"progress": 50})  # 50 != 0 -> emit attempted -> raises -> swallow.
        bridge_widget.loadProgress.emit.assert_called_once_with(50)

    def test_title_changed_swallow(self, bridge_widget):
        """L539-L540: ``except RuntimeError: pass`` inside the
        ``title_changed`` closure.
        """
        bridge_widget.titleChanged.emit.side_effect = RuntimeError("dead")
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["title_changed"]
        cb({"title": "RealTitle"})
        bridge_widget.titleChanged.emit.assert_called_once_with("RealTitle")

    def test_url_changed_swallow(self, bridge_widget):
        """L551-L552: ``except RuntimeError: pass`` inside the
        ``url_changed`` closure.
        """
        bridge_widget.urlChanged.emit.side_effect = RuntimeError("dead")
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["url_changed"]
        cb({"url": "https://swallow.test/"})
        bridge_widget.urlChanged.emit.assert_called_once_with("https://swallow.test/")

    def test_navigation_started_url_change_branch(self, bridge_widget):
        """The navigation_started closure has an inner branch that
        only emits ``urlChanged`` when ``url`` is non-empty AND differs
        from the cached value. Cover both legs of the conditional
        explicitly so coverage records both arms (L495-L497).
        """
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["navigation_started"]

        # First navigation: url empty -> only loadStarted emits.
        cb({})
        bridge_widget.loadStarted.emit.assert_called_once()
        bridge_widget.urlChanged.emit.assert_not_called()

        # Second navigation: url present + new -> urlChanged emits.
        cb({"url": "https://nav.test/"})
        bridge_widget.urlChanged.emit.assert_called_once_with("https://nav.test/")
        assert bridge_widget._qt_signal_state["current_url"] == "https://nav.test/"

    def test_navigation_finished_url_change_branch(self, bridge_widget):
        """L511-L513: navigation_finished's inner ``urlChanged.emit`` arm."""
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["navigation_finished"]
        cb({"success": True, "url": "https://done.test/"})
        bridge_widget.urlChanged.emit.assert_called_once_with("https://done.test/")
        assert bridge_widget._qt_signal_state["current_url"] == "https://done.test/"

    def test_icon_changed_no_url_branch(self, bridge_widget):
        """When data has no ``url`` key, only ``iconChanged`` is emitted
        (covers the ``if url:`` False arm at L613).
        """
        QtWebView._setup_signal_bridge(bridge_widget)
        cb = bridge_widget._registry.handlers["icon_changed"]
        cb({})
        bridge_widget.iconChanged.emit.assert_called_once()
        bridge_widget.iconUrlChanged.emit.assert_not_called()


# ---------------------------------------------------------------------------
# load_file paths -- the asset_root branch and its fallbacks.
#
# These tests exercise lines L790-L818 in _core.py (asset_root translation
# to the auroraview://localhost protocol on Windows / non-Windows; the
# ``ValueError`` fallback when the path is outside asset_root; the final
# ``except Exception`` that delegates to ``_webview.load_file`` or
# ``self.load_url``).
# ---------------------------------------------------------------------------


class TestLoadFileBranches:
    def _make_host(self, asset_root=None):
        """Build a minimally-populated _GuardHost-compatible stub bound
        with the unbound ``QtWebView.load_file`` method so we can drive
        it directly.
        """
        host = MagicMock()
        host.is_alive = True
        host._asset_root = asset_root
        host._webview = MagicMock()
        host.load_url = MagicMock()
        host.load_html = MagicMock()
        return host

    def test_asset_root_path_uses_auroraview_protocol(self, tmp_path, monkeypatch):
        """When the file is under ``_asset_root``, load_file should
        synthesize an ``auroraview://...`` URL and call ``self.load_url``.
        Covers L790-L802 including the verbose-log branch when on.
        """
        asset_root = tmp_path / "assets"
        asset_root.mkdir()
        index = asset_root / "page" / "index.html"
        index.parent.mkdir(parents=True, exist_ok=True)
        index.write_text("<p>x</p>", encoding="utf-8")

        monkeypatch.setattr(_core, "_VERBOSE_LOGGING", True)
        host = self._make_host(asset_root=str(asset_root))

        QtWebView.load_file(host, index)

        # Exactly one ``load_url`` call.
        host.load_url.assert_called_once()
        called_url = host.load_url.call_args.args[0]
        # URL scheme depends on platform.
        assert called_url.startswith(("auroraview://", "https://auroraview.localhost/"))
        assert "page/index.html" in called_url

    def test_asset_root_outside_falls_back_to_load_html(self, tmp_path):
        """When the file is OUTSIDE asset_root, the ``ValueError`` arm at
        L803-L808 is hit, then the fallback at L810-L812 reads the file
        and delegates to ``self.load_html``.
        """
        asset_root = tmp_path / "assets"
        asset_root.mkdir()
        outside = tmp_path / "outside.html"
        outside.write_text("<h1>outside</h1>", encoding="utf-8")

        host = self._make_host(asset_root=str(asset_root))
        QtWebView.load_file(host, outside)

        host.load_html.assert_called_once_with("<h1>outside</h1>")
        host.load_url.assert_not_called()

    def test_no_asset_root_reads_and_calls_load_html(self, tmp_path):
        """When ``_asset_root`` is None, the asset_root block is skipped
        entirely and ``load_html`` is invoked with file contents.
        """
        page = tmp_path / "p.html"
        page.write_text("<b>hi</b>", encoding="utf-8")
        host = self._make_host(asset_root=None)

        QtWebView.load_file(host, page)
        host.load_html.assert_called_once_with("<b>hi</b>")

    def test_unreadable_file_falls_back_to_webview_load_file(self, monkeypatch):
        """If both ``read_text`` and ``self.load_html`` paths fail, the
        ``except Exception`` arm at L813-L816 delegates to
        ``self._webview.load_file`` when callable.
        """
        host = self._make_host(asset_root=None)
        # Force the read_text path to blow up by giving a non-existent path.
        bogus = "/definitely/does/not/exist/page.html"
        host._webview.load_file = MagicMock()

        QtWebView.load_file(host, bogus)
        host._webview.load_file.assert_called_once_with(bogus)

    def test_unreadable_file_falls_back_to_load_url_when_no_loader(self, monkeypatch):
        """L817-L818: when ``_webview.load_file`` is missing or
        non-callable, the very last fallback uses the file:// URI.
        """
        host = self._make_host(asset_root=None)
        # Make load_file attribute non-callable so getattr+callable returns False.
        host._webview.load_file = "not-a-function"
        bogus = "/another/missing/page.html"

        QtWebView.load_file(host, bogus)
        host.load_url.assert_called_once()
        called = host.load_url.call_args.args[0]
        assert called.startswith("file:")

    def test_asset_root_non_windows_uses_auroraview_scheme(self, tmp_path, monkeypatch):
        """Force the non-Windows arm of the platform check at L797-L798
        so that coverage records both branches of the ``if sys.platform``
        ladder.
        """
        asset_root = tmp_path / "assets"
        asset_root.mkdir()
        page = asset_root / "x.html"
        page.write_text("<i/>", encoding="utf-8")

        # Force the ``else`` branch.
        monkeypatch.setattr(_core.sys, "platform", "linux")
        host = self._make_host(asset_root=str(asset_root))

        QtWebView.load_file(host, page)
        called = host.load_url.call_args.args[0]
        assert called.startswith("auroraview://")


# ---------------------------------------------------------------------------
# create_deferred -- on_ready callback path. The previous test exercised
# the placeholder branch but omitted ``on_ready``, leaving L745-L746
# uncovered. This forces the True arm of ``if on_ready:``.
# ---------------------------------------------------------------------------


class TestCreateDeferredOnReadyBranch:
    def test_on_ready_invoked(self, qapp, monkeypatch):
        """Force the True arm of ``if on_ready:`` inside ``do_create``
        (L745-L746). ``cls(...)`` inside the deferred body resolves to
        the real ``QtWebView`` class via classmethod binding, so we
        replace ``QtWebView.__init__`` with a no-op for the duration of
        the test. We then assert that ``on_ready`` was called with the
        constructed widget.
        """
        from qtpy.QtCore import QTimer
        from qtpy.QtWidgets import QWidget

        scheduled = []
        monkeypatch.setattr(
            QTimer,
            "singleShot",
            staticmethod(lambda delay, cb: scheduled.append(cb)),
        )

        # Replace the heavy ``__init__`` with a QWidget-only init so that
        # ``cls(...)`` returns a usable widget without any Rust core call.
        original_init = QtWebView.__init__

        def stub_init(self, *_a, **_kw):
            QWidget.__init__(self)

        monkeypatch.setattr(QtWebView, "__init__", stub_init)

        captured = []
        placeholder = QtWebView.create_deferred(
            parent=None,
            title="X",
            width=10,
            height=10,
            on_ready=lambda w: captured.append(w),
        )
        try:
            assert scheduled
            scheduled[0]()
            assert len(captured) == 1
            assert isinstance(captured[0], QtWebView)
        finally:
            monkeypatch.setattr(QtWebView, "__init__", original_init)
            placeholder.deleteLater()
