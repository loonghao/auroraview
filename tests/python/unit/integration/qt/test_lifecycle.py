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
