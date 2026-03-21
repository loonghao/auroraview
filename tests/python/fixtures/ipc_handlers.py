# -*- coding: utf-8 -*-
"""Reusable IPC test handlers for roundtrip testing.

These handlers simulate real application scenarios for verifying
the complete Python ↔ Rust ↔ JS IPC communication chain.

Usage:
    echo = EchoHandler()
    webview.on("echo", echo)
    # After JS sends event "echo" with data {"msg": "hi"}:
    assert echo.calls == [{"msg": "hi"}]

    ping = PingHandler()
    webview.on("ping", ping)
    # After JS sends event "ping":
    assert ping.calls[0]["response"] == "pong"
"""
from __future__ import annotations

import threading


class EchoHandler:
    """Echo handler - records received data for later inspection.

    Thread-safe: all state mutations are protected by a lock.
    """

    def __init__(self):
        # type: () -> None
        self._calls = []  # type: List[Any]
        self._lock = threading.Lock()

    def __call__(self, data):
        # type: (Any) -> None
        """Handle incoming IPC event by recording the data."""
        with self._lock:
            self._calls.append(data)

    @property
    def calls(self):
        # type: () -> List[Any]
        """Return a copy of all received calls."""
        with self._lock:
            return list(self._calls)

    @property
    def call_count(self):
        # type: () -> int
        """Return the number of calls received."""
        with self._lock:
            return len(self._calls)

    @property
    def last_call(self):
        # type: () -> Optional[Any]
        """Return the last received data, or None."""
        with self._lock:
            return self._calls[-1] if self._calls else None

    def reset(self):
        # type: () -> None
        """Clear all recorded calls."""
        with self._lock:
            self._calls.clear()


class PingHandler:
    """Ping handler - responds with 'pong' and records request metadata.

    Simulates a request-response pattern over IPC events.
    """

    def __init__(self):
        # type: () -> None
        self._calls = []  # type: List[Dict[str, Any]]
        self._lock = threading.Lock()

    def __call__(self, data):
        # type: (Any) -> None
        """Handle ping request, record it with pong response metadata."""
        with self._lock:
            self._calls.append({
                "received": data,
                "response": "pong",
            })

    @property
    def calls(self):
        # type: () -> List[Dict[str, Any]]
        """Return a copy of all ping/pong records."""
        with self._lock:
            return list(self._calls)

    @property
    def call_count(self):
        # type: () -> int
        with self._lock:
            return len(self._calls)

    def reset(self):
        # type: () -> None
        with self._lock:
            self._calls.clear()


class CollectorHandler:
    """General-purpose event collector - records all events with metadata.

    Useful for testing event ordering, timing, and multi-event scenarios.
    """

    def __init__(self):
        # type: () -> None
        self._events = []  # type: List[Dict[str, Any]]
        self._lock = threading.Lock()
        self._event_received = threading.Event()

    def __call__(self, data):
        # type: (Any) -> None
        """Record an incoming event."""
        with self._lock:
            self._events.append({"data": data})
            self._event_received.set()

    @property
    def events(self):
        # type: () -> List[Dict[str, Any]]
        """Return a copy of all collected events."""
        with self._lock:
            return list(self._events)

    @property
    def event_count(self):
        # type: () -> int
        with self._lock:
            return len(self._events)

    def wait_for_event(self, timeout=5.0):
        # type: (float) -> bool
        """Wait until at least one event is received.

        Args:
            timeout: Maximum seconds to wait.

        Returns:
            True if an event was received, False if timed out.
        """
        return self._event_received.wait(timeout)

    def reset(self):
        # type: () -> None
        """Clear all collected events and reset the wait flag."""
        with self._lock:
            self._events.clear()
            self._event_received.clear()
