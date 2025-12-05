# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Unit tests for the Signal-Slot system."""

from __future__ import annotations

import threading
from typing import Any, List

from auroraview.core.signals import (
    ConnectionGuard,
    ConnectionId,
    Signal,
    SignalRegistry,
    WebViewSignals,
)


class TestSignal:
    """Test the Signal class."""

    def test_connect_and_emit(self) -> None:
        """Test basic connect and emit functionality."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        conn_id = signal.connect(lambda x: received.append(x))

        signal.emit(42)
        signal.emit(100)

        assert received == [42, 100]
        assert isinstance(conn_id, ConnectionId)

    def test_disconnect(self) -> None:
        """Test disconnecting a handler."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        conn_id = signal.connect(lambda x: received.append(x))
        signal.emit(1)

        result = signal.disconnect(conn_id)
        assert result is True

        signal.emit(2)  # Should not be received
        assert received == [1]

    def test_disconnect_nonexistent(self) -> None:
        """Test disconnecting a non-existent handler."""
        signal: Signal[int] = Signal()
        fake_id = ConnectionId("fake-id")

        result = signal.disconnect(fake_id)
        assert result is False

    def test_multiple_handlers(self) -> None:
        """Test multiple handlers on same signal."""
        signal: Signal[int] = Signal()
        results1: List[int] = []
        results2: List[int] = []

        signal.connect(lambda x: results1.append(x))
        signal.connect(lambda x: results2.append(x * 2))

        signal.emit(5)

        assert results1 == [5]
        assert results2 == [10]

    def test_connect_once(self) -> None:
        """Test one-time handler."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        signal.connect_once(lambda x: received.append(x))

        signal.emit(1)
        signal.emit(2)  # Should not trigger

        assert received == [1]

    def test_disconnect_all(self) -> None:
        """Test disconnecting all handlers."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        signal.connect(lambda x: received.append(x))
        signal.connect(lambda x: received.append(x * 2))

        count = signal.disconnect_all()
        assert count == 2

        signal.emit(1)
        assert received == []

    def test_handler_count(self) -> None:
        """Test handler count property."""
        signal: Signal[int] = Signal()

        assert signal.handler_count == 0
        assert signal.is_connected is False

        conn1 = signal.connect(lambda x: None)
        assert signal.handler_count == 1
        assert signal.is_connected is True

        _ = signal.connect(lambda x: None)  # conn2 unused intentionally
        assert signal.handler_count == 2

        signal.disconnect(conn1)
        assert signal.handler_count == 1

    def test_emit_no_handlers(self) -> None:
        """Test emitting with no handlers."""
        signal: Signal[int] = Signal()
        count = signal.emit(42)
        assert count == 0

    def test_callable_emit(self) -> None:
        """Test calling signal directly to emit."""
        signal: Signal[str] = Signal()
        received: List[str] = []

        signal.connect(lambda x: received.append(x))
        signal("hello")  # Using __call__

        assert received == ["hello"]

    def test_thread_safety(self) -> None:
        """Test thread-safe operations."""
        signal: Signal[int] = Signal()
        results: List[int] = []
        lock = threading.Lock()

        def handler(x: int) -> None:
            with lock:
                results.append(x)

        signal.connect(handler)

        def emit_value(val: int) -> None:
            signal.emit(val)

        threads = [threading.Thread(target=emit_value, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(results) == 10
        assert sorted(results) == list(range(10))


class TestConnectionGuard:
    """Test the ConnectionGuard class."""

    def test_auto_disconnect_on_del(self) -> None:
        """Test automatic disconnection when guard is deleted."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        conn_id = signal.connect(lambda x: received.append(x))
        guard = ConnectionGuard(signal, conn_id)

        signal.emit(1)
        assert received == [1]

        del guard  # Should disconnect

        signal.emit(2)
        assert received == [1]  # Not received

    def test_detach(self) -> None:
        """Test detaching the guard."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        conn_id = signal.connect(lambda x: received.append(x))
        guard = ConnectionGuard(signal, conn_id)
        guard.detach()

        del guard  # Should NOT disconnect

        signal.emit(1)
        assert received == [1]

    def test_manual_disconnect(self) -> None:
        """Test manual disconnection via guard."""
        signal: Signal[int] = Signal()
        received: List[int] = []

        conn_id = signal.connect(lambda x: received.append(x))
        guard = ConnectionGuard(signal, conn_id)

        signal.emit(1)
        result = guard.disconnect()
        assert result is True

        signal.emit(2)
        assert received == [1]


class TestSignalRegistry:
    """Test the SignalRegistry class."""

    def test_get_or_create(self) -> None:
        """Test getting or creating signals."""
        registry = SignalRegistry()

        sig1 = registry.get_or_create("test_event")
        sig2 = registry.get_or_create("test_event")

        assert sig1 is sig2  # Same instance

    def test_get_nonexistent(self) -> None:
        """Test getting non-existent signal."""
        registry = SignalRegistry()
        assert registry.get("nonexistent") is None

    def test_contains(self) -> None:
        """Test checking if signal exists."""
        registry = SignalRegistry()

        assert "test" not in registry
        registry.get_or_create("test")
        assert "test" in registry

    def test_connect_and_emit(self) -> None:
        """Test connecting and emitting via registry."""
        registry = SignalRegistry()
        received: List[Any] = []

        registry.connect("my_event", lambda x: received.append(x))
        registry.emit("my_event", {"data": 123})

        assert received == [{"data": 123}]

    def test_disconnect(self) -> None:
        """Test disconnecting via registry."""
        registry = SignalRegistry()
        received: List[Any] = []

        conn_id = registry.connect("my_event", lambda x: received.append(x))
        registry.emit("my_event", 1)

        result = registry.disconnect("my_event", conn_id)
        assert result is True

        registry.emit("my_event", 2)
        assert received == [1]

    def test_remove_signal(self) -> None:
        """Test removing a signal."""
        registry = SignalRegistry()
        registry.get_or_create("test")

        assert registry.remove("test") is True
        assert "test" not in registry
        assert registry.remove("test") is False

    def test_names(self) -> None:
        """Test getting signal names."""
        registry = SignalRegistry()
        registry.get_or_create("event1")
        registry.get_or_create("event2")

        names = registry.names()
        assert set(names) == {"event1", "event2"}

    def test_dict_access(self) -> None:
        """Test dictionary-style access."""
        registry = SignalRegistry()
        signal = registry["my_event"]

        assert signal is registry.get_or_create("my_event")


class TestWebViewSignals:
    """Test the WebViewSignals class."""

    def test_lifecycle_signals(self) -> None:
        """Test lifecycle signals."""
        signals = WebViewSignals()
        loaded = False

        def on_loaded() -> None:
            nonlocal loaded
            loaded = True

        signals.page_loaded.connect(on_loaded)
        signals.page_loaded.emit()

        assert loaded is True

    def test_custom_signals(self) -> None:
        """Test custom signals via registry."""
        signals = WebViewSignals()
        received: List[Any] = []

        signals.on("my_event", lambda x: received.append(x))
        signals.emit_custom("my_event", {"key": "value"})

        assert received == [{"key": "value"}]

    def test_get_custom(self) -> None:
        """Test getting custom signal."""
        signals = WebViewSignals()

        sig1 = signals.get_custom("test")
        sig2 = signals.custom["test"]

        assert sig1 is sig2

    def test_disconnect_all(self) -> None:
        """Test disconnecting all signals."""
        signals = WebViewSignals()
        count = 0

        def inc() -> None:
            nonlocal count
            count += 1

        signals.page_loaded.connect(inc)
        signals.closing.connect(inc)
        signals.on("custom", lambda _: inc())

        signals.disconnect_all()

        signals.page_loaded.emit()
        signals.closing.emit()
        signals.emit_custom("custom", None)

        assert count == 0  # Nothing received
