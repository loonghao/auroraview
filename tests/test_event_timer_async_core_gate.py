# -*- coding: utf-8 -*-

import threading


from auroraview.utils.event_timer import EventTimer
from auroraview.utils.timer_backends import ThreadTimerBackend


class _FakeThread:
    def __init__(self, alive):
        self._alive = alive

    def is_alive(self):
        return self._alive


class _StubWebView:
    def __init__(self):
        self.process_events_called = 0
        self.process_events_ipc_only_called = 0

        # EventTimer expects these locks/fields to exist in some modes.
        self._async_core_lock = threading.Lock()
        self._async_core = None

    def process_events(self):
        self.process_events_called += 1
        return False

    def process_events_ipc_only(self):
        self.process_events_ipc_only_called += 1
        return False


def _make_timer(webview):
    # Use the thread backend so EventTimer chooses process_events() path.
    timer = EventTimer(
        webview,
        interval_ms=1,
        check_window_validity=False,
        backend=ThreadTimerBackend(),
    )
    # We don't actually start a real timer thread in unit tests.
    timer._running = True
    return timer


def test_tick_does_not_gate_when_no_show_thread_even_if_async_core_none():
    webview = _StubWebView()
    # Explicitly ensure no show thread attribute is set.
    assert getattr(webview, "_show_thread", None) is None

    timer = _make_timer(webview)
    timer._tick()

    assert webview.process_events_called == 1


def test_tick_gates_when_show_thread_alive_and_async_core_none():
    webview = _StubWebView()
    webview._show_thread = _FakeThread(True)

    timer = _make_timer(webview)
    timer._tick()

    assert webview.process_events_called == 0


def test_tick_runs_when_show_thread_alive_and_async_core_ready():
    webview = _StubWebView()
    webview._show_thread = _FakeThread(True)
    with webview._async_core_lock:
        webview._async_core = object()

    timer = _make_timer(webview)
    timer._tick()

    assert webview.process_events_called == 1
