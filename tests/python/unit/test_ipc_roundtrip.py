# -*- coding: utf-8 -*-
"""IPC roundtrip unit tests.

Tests the IPC test fixture handlers (echo, ping, collector) and verifies
data serialization/deserialization through the Rust JSON bridge.
"""
from __future__ import annotations

import threading

import pytest

# Import test fixtures
from tests.python.fixtures import CollectorHandler, EchoHandler, PingHandler


class TestEchoHandler:
    """Test EchoHandler fixture behavior."""

    def test_basic_echo(self):
        handler = EchoHandler()
        handler({"message": "hello"})
        assert handler.call_count == 1
        assert handler.calls[0] == {"message": "hello"}

    def test_multiple_calls(self):
        handler = EchoHandler()
        handler("first")
        handler("second")
        handler("third")
        assert handler.call_count == 3
        assert handler.calls == ["first", "second", "third"]

    def test_last_call(self):
        handler = EchoHandler()
        assert handler.last_call is None
        handler({"a": 1})
        handler({"b": 2})
        assert handler.last_call == {"b": 2}

    def test_reset(self):
        handler = EchoHandler()
        handler("data")
        assert handler.call_count == 1
        handler.reset()
        assert handler.call_count == 0
        assert handler.calls == []

    def test_thread_safety(self):
        handler = EchoHandler()
        threads = []
        for i in range(10):
            t = threading.Thread(target=handler, args=({"id": i},))
            threads.append(t)
            t.start()
        for t in threads:
            t.join()
        assert handler.call_count == 10

    def test_complex_data(self):
        handler = EchoHandler()
        complex_data = {
            "unicode": "Hello 你好 🌍",
            "nested": {"level1": {"level2": [1, 2, 3]}},
            "null": None,
            "bool": True,
            "number": 3.14,
        }
        handler(complex_data)
        assert handler.last_call == complex_data


class TestPingHandler:
    """Test PingHandler fixture behavior."""

    def test_basic_ping(self):
        handler = PingHandler()
        handler({"timestamp": 12345})
        assert handler.call_count == 1
        assert handler.calls[0]["received"] == {"timestamp": 12345}
        assert handler.calls[0]["response"] == "pong"

    def test_multiple_pings(self):
        handler = PingHandler()
        handler({"seq": 1})
        handler({"seq": 2})
        assert handler.call_count == 2
        assert handler.calls[0]["received"]["seq"] == 1
        assert handler.calls[1]["received"]["seq"] == 2

    def test_reset(self):
        handler = PingHandler()
        handler({})
        handler.reset()
        assert handler.call_count == 0


class TestCollectorHandler:
    """Test CollectorHandler fixture behavior."""

    def test_basic_collection(self):
        handler = CollectorHandler()
        handler({"event": "test"})
        assert handler.event_count == 1
        assert handler.events[0]["data"] == {"event": "test"}

    def test_wait_for_event(self):
        handler = CollectorHandler()

        def send_later():
            import time

            time.sleep(0.1)
            handler({"delayed": True})

        t = threading.Thread(target=send_later)
        t.start()
        result = handler.wait_for_event(timeout=2.0)
        t.join()
        assert result is True
        assert handler.event_count == 1

    def test_wait_timeout(self):
        handler = CollectorHandler()
        result = handler.wait_for_event(timeout=0.1)
        assert result is False

    def test_reset(self):
        handler = CollectorHandler()
        handler({"data": 1})
        assert handler.event_count == 1
        handler.reset()
        assert handler.event_count == 0


class TestRustJsonRoundtrip:
    """Test data roundtrip through the Rust JSON bridge.

    These tests verify that Python objects survive serialization through
    the Rust core's json_loads/json_dumps functions.
    """

    @pytest.fixture(autouse=True)
    def check_core(self):
        """Skip if Rust core is not available."""
        try:
            from auroraview import json_dumps, json_loads

            self.json_loads = json_loads
            self.json_dumps = json_dumps
        except (ImportError, TypeError):
            pytest.skip("Rust core not available")

    def test_basic_roundtrip(self):
        original = {"key": "value", "number": 42, "flag": True}
        serialized = self.json_dumps(original)
        deserialized = self.json_loads(serialized)
        assert deserialized == original

    def test_unicode_roundtrip(self):
        original = {"text": "Hello 你好 こんにちは 🌍 Ñoño"}
        serialized = self.json_dumps(original)
        deserialized = self.json_loads(serialized)
        assert deserialized == original

    def test_nested_roundtrip(self):
        original = {
            "level1": {
                "level2": {"level3": {"value": [1, "two", None, True, 3.14]}}
            }
        }
        serialized = self.json_dumps(original)
        deserialized = self.json_loads(serialized)
        assert deserialized == original

    def test_array_roundtrip(self):
        original = [
            {"name": "item1", "tags": ["a", "b"]},
            {"name": "item2", "tags": []},
        ]
        serialized = self.json_dumps(original)
        deserialized = self.json_loads(serialized)
        assert deserialized == original

    def test_special_values_roundtrip(self):
        original = {
            "null_val": None,
            "empty_str": "",
            "empty_obj": {},
            "empty_arr": [],
            "zero": 0,
            "false": False,
        }
        serialized = self.json_dumps(original)
        deserialized = self.json_loads(serialized)
        assert deserialized == original
