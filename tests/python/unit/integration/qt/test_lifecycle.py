# -*- coding: utf-8 -*-
"""Tests for auroraview.integration.qt.lifecycle.LifecycleMixin.

These tests target the close-event / state-reset / destructor paths and the
``delayed_geometry_sync`` retry chain.  ``QTimer.singleShot`` is patched so
scheduled callbacks can be inspected synchronously without a running Qt
event loop, which keeps the suite usable on headless CI runners.
"""

import sys
import types
from unittest.mock import MagicMock

import pytest

# The mixin imports qtpy at module load time, so skip the whole module on
# environments where qtpy is unavailable.
pytest.importorskip("qtpy", reason="Qt tests require qtpy")

# The imports below must run AFTER pytest.importorskip("qtpy") above, so we
# cannot move them up into the top-level import block.  ruff's isort rule
# (I001) does not understand this constraint, so we suppress it here in
# addition to the per-import E402 noqas.
from auroraview.integration.qt import lifecycle  # noqa: E402, I001
from auroraview.integration.qt.lifecycle import (  # noqa: E402
    LifecycleMixin,
    _DELAYED_SYNC_RETRY_INTERVAL_MS,
    _delayed_geometry_sync,
)

pytestmark = [pytest.mark.qt_related, pytest.mark.unit]


class _Host:
    """Minimal stand-in for ``QtWebView`` exposing the attributes the mixin
    reads via ``getattr`` / ``setattr``.

    The mixin methods under test call sibling mixin methods through ``self``
    (e.g. ``_handle_close_event`` invokes ``self._reset_state_for_reuse()``).
    To exercise those branches without bringing up a real ``QtWebView``, we
    bind the relevant mixin methods onto the stub instance via
    :func:`types.MethodType`.
    """

    def __init__(self):
        # Flags backing the cross-task mutex
        self._geometry_sync_in_progress = False
        self._child_window_fix_in_progress = False
        self._is_closing = False
        self._webview_initialized = True
        # Webview / container plumbing
        self._webview = MagicMock()
        self._webview_container = MagicMock()
        self._webview_qwindow = MagicMock()
        # Methods the mixin delegates to
        self._force_container_geometry = MagicMock()
        # Bind mixin methods so ``self._reset_state_for_reuse()`` in the
        # production code resolves to the real implementation on this stub.
        self._reset_state_for_reuse = types.MethodType(LifecycleMixin._reset_state_for_reuse, self)


@pytest.fixture
def captured_timers(monkeypatch):
    """Record ``QTimer.singleShot`` calls without dispatching them.

    Returns a list of ``(delay_ms, callback)`` tuples in invocation order;
    tests can pop items off the list and call them by hand.

    IMPORTANT: Callbacks are NEVER dispatched automatically.  Tests that
    need to exercise scheduled work must invoke ``captured_timers[i][1]()``
    explicitly.  Do not change this fixture to auto-dispatch — multiple
    tests rely on deterministic manual invocation to verify retry chains.
    """
    calls = []

    class _FakeQTimer:
        @staticmethod
        def singleShot(delay, callback):
            calls.append((delay, callback))

    monkeypatch.setattr(lifecycle, "QTimer", _FakeQTimer)
    return calls


# ---------------------------------------------------------------------------
# delayed_geometry_sync — the most subtle path in the module: it must not
# deadlock against the child-window fixer, must reschedule itself when the
# fixer holds its flag, and must give up after a bounded number of retries.
# ---------------------------------------------------------------------------


class TestDelayedGeometrySync:
    """Behavioural tests for the delayed geometry-sync function."""

    def test_skips_when_is_closing(self, captured_timers):
        host = _Host()
        host._is_closing = True
        _delayed_geometry_sync(host)
        host._force_container_geometry.assert_not_called()
        assert captured_timers == []

    def test_runs_force_geometry_when_acquired(self, captured_timers):
        host = _Host()
        _delayed_geometry_sync(host)
        host._force_container_geometry.assert_called_once()
        # Flag must be released on exit
        assert host._geometry_sync_in_progress is False
        assert captured_timers == []

    def test_blocked_by_child_window_fix_reschedules(self, captured_timers):
        host = _Host()
        host._child_window_fix_in_progress = True
        _delayed_geometry_sync(host, retries_left=3)
        # Did not run the work
        host._force_container_geometry.assert_not_called()
        # Did schedule a retry at _DELAYED_SYNC_RETRY_INTERVAL_MS
        assert len(captured_timers) == 1
        delay, cb = captured_timers[0]
        assert delay == _DELAYED_SYNC_RETRY_INTERVAL_MS
        assert callable(cb)

    def test_retry_chain_decrements_until_zero(self, captured_timers):
        host = _Host()
        host._child_window_fix_in_progress = True

        _delayed_geometry_sync(host, retries_left=3)  # retries_left=3 → schedule retry with 2
        assert len(captured_timers) == 1

        captured_timers[0][1]()  # retries_left=2 → schedule retry with 1
        assert len(captured_timers) == 2

        captured_timers[1][1]()  # retries_left=1 → schedule retry with 0
        assert len(captured_timers) == 3

        captured_timers[2][1]()  # retries_left=0 → no further schedule
        assert len(captured_timers) == 3
        host._force_container_geometry.assert_not_called()

    def test_retry_succeeds_when_peer_releases(self, captured_timers):
        host = _Host()
        host._child_window_fix_in_progress = True
        _delayed_geometry_sync(host, retries_left=3)  # blocked, schedules retry

        # Peer releases its flag before the retry fires
        host._child_window_fix_in_progress = False
        captured_timers[0][1]()

        host._force_container_geometry.assert_called_once()
        assert host._geometry_sync_in_progress is False

    def test_swallows_force_geometry_exception(self, captured_timers):
        host = _Host()
        host._force_container_geometry.side_effect = RuntimeError("boom")
        _delayed_geometry_sync(host)
        # Must not propagate
        # Flag still released even after an exception
        assert host._geometry_sync_in_progress is False


# ---------------------------------------------------------------------------
# _handle_close_event
# ---------------------------------------------------------------------------


class TestHandleCloseEvent:
    def test_returns_true_when_already_closing(self):
        host = _Host()
        host._is_closing = True
        result = LifecycleMixin._handle_close_event(host)
        assert result is True
        host._webview.close.assert_not_called()

    def test_sets_is_closing_flag_on_first_call(self):
        host = _Host()
        LifecycleMixin._handle_close_event(host)
        assert host._is_closing is True

    def test_calls_webview_close(self):
        host = _Host()
        LifecycleMixin._handle_close_event(host)
        host._webview.close.assert_called_once()

    def test_continues_after_webview_close_exception(self):
        host = _Host()
        host._webview.close.side_effect = RuntimeError("already gone")
        # Must not raise
        result = LifecycleMixin._handle_close_event(host)
        assert result is False
        # State reset still happens
        assert host._webview_initialized is False

    def test_calls_reset_state_for_reuse(self):
        host = _Host()
        LifecycleMixin._handle_close_event(host)
        # _reset_state_for_reuse side effects observable on the host
        assert host._webview_initialized is False
        assert host._webview_container is None
        assert host._webview_qwindow is None

    def test_returns_false_on_first_close(self):
        host = _Host()
        result = LifecycleMixin._handle_close_event(host)
        assert result is False


# ---------------------------------------------------------------------------
# _reset_state_for_reuse
# ---------------------------------------------------------------------------


class TestResetStateForReuse:
    def test_resets_initialized_flag(self):
        host = _Host()
        LifecycleMixin._reset_state_for_reuse(host)
        assert host._webview_initialized is False

    def test_clears_container_refs(self):
        host = _Host()
        LifecycleMixin._reset_state_for_reuse(host)
        assert host._webview_container is None
        assert host._webview_qwindow is None

    def test_keeps_is_closing_unchanged(self):
        """Documented contract: ``_is_closing`` must NOT be reset here.

        showEvent is responsible for clearing it on next show.
        """
        host = _Host()
        host._is_closing = True
        LifecycleMixin._reset_state_for_reuse(host)
        assert host._is_closing is True

    def test_calls_core_reset_when_available(self):
        host = _Host()
        core = MagicMock()
        host._webview._core = core
        LifecycleMixin._reset_state_for_reuse(host)
        core.reset.assert_called_once()

    def test_handles_missing_core(self):
        host = _Host()
        host._webview._core = None
        # Must not raise
        LifecycleMixin._reset_state_for_reuse(host)

    def test_handles_core_without_reset(self):
        host = _Host()
        # spec=[] makes hasattr return False
        host._webview._core = MagicMock(spec=[])
        # Must not raise
        LifecycleMixin._reset_state_for_reuse(host)

    def test_swallows_core_reset_exception(self):
        host = _Host()
        core = MagicMock()
        core.reset.side_effect = RuntimeError("core gone")
        host._webview._core = core
        # Must not raise
        LifecycleMixin._reset_state_for_reuse(host)


# ---------------------------------------------------------------------------
# _handle_destructor
# ---------------------------------------------------------------------------


class TestHandleDestructor:
    def test_does_nothing_when_already_closing(self):
        host = _Host()
        host._is_closing = True
        LifecycleMixin._handle_destructor(host)
        host._webview.close.assert_not_called()

    def test_closes_webview_when_not_closing(self):
        host = _Host()
        LifecycleMixin._handle_destructor(host)
        host._webview.close.assert_called_once()

    def test_swallows_exception(self):
        host = _Host()
        host._webview.close.side_effect = RuntimeError("boom")
        # Must not raise
        LifecycleMixin._handle_destructor(host)

    def test_safe_when_webview_attribute_missing(self):
        host = _Host()
        del host._webview
        # Must not raise
        LifecycleMixin._handle_destructor(host)


# ---------------------------------------------------------------------------
# Module-level smoke tests
# ---------------------------------------------------------------------------


class TestModuleExports:
    def test_lifecycle_mixin_exported(self):
        assert "LifecycleMixin" in lifecycle.__all__

    def test_verbose_logging_constant_is_bool(self):
        assert isinstance(lifecycle._VERBOSE_LOGGING, bool)


@pytest.mark.skipif(sys.platform != "win32", reason="Windows-only optimisation")
class TestPlatformGuards:
    """Platform-specific behaviour is gated by ``sys.platform``; just check
    the constant the module relies on is present and importable.
    """

    def test_qapplication_importable(self):
        from qtpy.QtWidgets import QApplication

        assert QApplication is not None


# ---------------------------------------------------------------------------
# _initialize_webview / _init_webview_progressive
# ---------------------------------------------------------------------------
#
# These two methods drive the WebView creation handshake and account for the
# bulk of the lines in ``lifecycle.py``.  They are normally invoked from
# ``QtWebView.showEvent`` against a fully constructed Qt widget, but the
# logic is mostly bookkeeping around (a) the Rust core, (b) Win32 anti-
# flicker helpers, (c) ``QApplication.processEvents`` and (d) ``QTimer``
# scheduling.  Patching those four things at the module level lets us
# reach every branch with a stub host.


class _ProgressiveHost(_Host):
    """Stub host with the extra surface ``_init_webview_progressive`` needs."""

    def __init__(self):
        super().__init__()
        self._stack = MagicMock()
        self._initial_url = None
        self._initial_html = None
        self._embed_mode = "child"
        self._using_direct_embed = False
        self._direct_embed_hwnd = None
        self._pre_show_hidden = False
        self._show_start_time = 0.0
        self._create_webview_container = MagicMock()

    def setAttribute(self, *args, **kwargs):  # noqa: N802 - Qt naming
        pass

    def winId(self):  # noqa: N802 - Qt naming
        return 0xABCDEF

    def size(self):
        s = MagicMock()
        s.width.return_value = 1024
        s.height.return_value = 768
        return s


@pytest.fixture
def stub_qapp(monkeypatch):
    """Replace ``QApplication`` with a stub whose ``processEvents`` is a no-op."""
    fake_app = MagicMock()
    monkeypatch.setattr(lifecycle, "QApplication", fake_app)
    return fake_app


@pytest.fixture
def stub_anti_flicker(monkeypatch):
    """Stub the Windows-only anti-flicker helpers and return them as mocks."""
    hide = MagicMock(return_value=True)
    show = MagicMock()
    monkeypatch.setattr(lifecycle, "hide_window_for_init", hide)
    monkeypatch.setattr(lifecycle, "show_window_after_init", show)
    return hide, show


class TestInitializeWebView:
    """``_initialize_webview`` is the showEvent entry point."""

    def test_calls_progressive_init(self, monkeypatch, stub_qapp, stub_anti_flicker):
        host = _ProgressiveHost()
        called = []
        host._init_webview_progressive = types.MethodType(lambda self_: called.append(self_), host)
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._initialize_webview(host)
        assert called == [host]
        assert host._stack.setCurrentIndex.called
        assert stub_qapp.processEvents.called

    def test_anti_flicker_runs_on_windows(self, monkeypatch, stub_qapp, stub_anti_flicker):
        host = _ProgressiveHost()
        host._init_webview_progressive = types.MethodType(lambda self_: None, host)
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="win32"))
        LifecycleMixin._initialize_webview(host)
        hide, _show = stub_anti_flicker
        hide.assert_called_once()
        assert host._pre_show_hidden is True

    def test_skips_anti_flicker_when_hwnd_zero(self, monkeypatch, stub_qapp, stub_anti_flicker):
        host = _ProgressiveHost()
        host._init_webview_progressive = types.MethodType(lambda self_: None, host)
        host.winId = lambda: 0  # type: ignore[assignment]
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="win32"))
        LifecycleMixin._initialize_webview(host)
        hide, _show = stub_anti_flicker
        hide.assert_not_called()
        assert host._pre_show_hidden is False


class TestInitWebViewProgressive:
    """``_init_webview_progressive`` is the meat of the file (~130 LOC).

    It handles Rust-callback registration, the ``show_embedded`` happy path
    plus two fallback paths, the post-show NC-strip, anti-flicker
    completion and final ``QTimer.singleShot`` scheduling.
    """

    @staticmethod
    def _make_core(*, has_set_on_hwnd=True, show_embedded=True):
        core = MagicMock()
        if not has_set_on_hwnd:
            del core.set_on_hwnd_created
        if not show_embedded:
            del core.show_embedded
        return core

    def test_no_core_uses_webview_show_fallback(self, captured_timers, stub_qapp):
        host = _ProgressiveHost()
        host._webview._core = None
        LifecycleMixin._init_webview_progressive(host)
        host._webview.show.assert_called_once()
        assert captured_timers == []

    def test_show_embedded_happy_path_schedules_two_syncs(
        self, captured_timers, stub_qapp, monkeypatch
    ):
        host = _ProgressiveHost()
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        core.show_embedded.assert_called_once()
        core.show.assert_not_called()
        core.set_on_hwnd_created.assert_called_once()
        delays = sorted(d for d, _ in captured_timers)
        assert delays == [150, 500]
        host._webview._auto_timer.start.assert_called_once()
        core.set_visible.assert_called_once_with(True)

    def test_falls_back_to_core_show_when_show_embedded_missing(
        self, captured_timers, stub_qapp, monkeypatch
    ):
        host = _ProgressiveHost()
        core = self._make_core(show_embedded=False)
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        core.show.assert_called_once()

    def test_show_embedded_exception_falls_back_to_webview_show(self, captured_timers, stub_qapp):
        host = _ProgressiveHost()
        core = self._make_core()
        core.show_embedded.side_effect = RuntimeError("boom")
        host._webview._core = core
        LifecycleMixin._init_webview_progressive(host)
        host._webview.show.assert_called_once()
        assert captured_timers == []

    def test_set_on_hwnd_created_failure_is_swallowed(
        self, captured_timers, stub_qapp, monkeypatch
    ):
        host = _ProgressiveHost()
        core = self._make_core()
        core.set_on_hwnd_created.side_effect = RuntimeError("no callbacks")
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._create_webview_container.assert_called()

    def test_callback_creates_container_when_invoked(self, captured_timers, stub_qapp, monkeypatch):
        host = _ProgressiveHost()
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        cb = core.set_on_hwnd_created.call_args[0][0]
        cb(0xFEED)
        host._create_webview_container.assert_any_call(core, hwnd=0xFEED)

    def test_loads_initial_url(self, captured_timers, stub_qapp, monkeypatch):
        host = _ProgressiveHost()
        host._initial_url = "https://example.com/"
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._webview.load_url.assert_called_once_with("https://example.com/")
        host._webview.load_html.assert_not_called()

    def test_loads_initial_html_when_no_url(self, captured_timers, stub_qapp, monkeypatch):
        host = _ProgressiveHost()
        host._initial_url = None
        host._initial_html = "<html></html>"
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._webview.load_html.assert_called_once_with("<html></html>")
        host._webview.load_url.assert_not_called()

    def test_no_initial_content_no_load_calls(self, captured_timers, stub_qapp, monkeypatch):
        host = _ProgressiveHost()
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._webview.load_url.assert_not_called()
        host._webview.load_html.assert_not_called()

    def test_auto_timer_failure_falls_back_to_webview_show(
        self, captured_timers, stub_qapp, monkeypatch
    ):
        host = _ProgressiveHost()
        core = self._make_core()
        host._webview._core = core
        timer = MagicMock()
        timer.start.side_effect = RuntimeError("timer dead")
        host._webview._auto_timer = timer
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._webview.show.assert_called_once()

    def test_no_auto_timer_falls_back_to_webview_show(
        self, captured_timers, stub_qapp, monkeypatch
    ):
        host = _ProgressiveHost()
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = None
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._webview.show.assert_called_once()

    def test_anti_flicker_restore_runs_on_win32(
        self, captured_timers, stub_qapp, stub_anti_flicker, monkeypatch
    ):
        host = _ProgressiveHost()
        host._pre_show_hidden = True
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="win32"))
        LifecycleMixin._init_webview_progressive(host)
        _hide, show = stub_anti_flicker
        show.assert_called_once()
        assert host._pre_show_hidden is False

    def test_set_visible_failure_is_swallowed(self, captured_timers, stub_qapp, monkeypatch):
        host = _ProgressiveHost()
        core = self._make_core()
        core.set_visible.side_effect = RuntimeError("visibility broken")
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._webview._auto_timer.start.assert_called_once()

    def test_stack_switches_to_webview_page(self, captured_timers, stub_qapp, monkeypatch):
        host = _ProgressiveHost()
        core = self._make_core()
        host._webview._core = core
        host._webview._auto_timer = MagicMock()
        monkeypatch.setattr(lifecycle, "sys", types.SimpleNamespace(platform="linux"))
        LifecycleMixin._init_webview_progressive(host)
        host._stack.setCurrentIndex.assert_called_with(1)
