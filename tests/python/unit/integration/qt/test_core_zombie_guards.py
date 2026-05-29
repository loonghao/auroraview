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
