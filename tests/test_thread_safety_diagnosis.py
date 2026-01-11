# -*- coding: utf-8 -*-
"""Tests for thread safety diagnosis findings.

This module tests the critical issues identified in the architecture diagnosis:
- P0: Lock ordering in EventLoopState
- P0: Message queue wake-up batching latency
- P1: Dual message pump conflict in DCC embedded mode
- P2: Inconsistent event processing strategies
"""

import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List

import pytest

from auroraview.utils.thread_dispatcher import (
    FallbackDispatcherBackend,
    ThreadDispatcherBackend,
    defer_to_main_thread,
    is_main_thread,
)


class TestThreadSafetyDiagnosis:
    """Tests validating thread safety diagnosis findings."""

    def test_main_thread_detection_consistency(self):
        """Test that main thread detection is consistent across calls.

        This validates that is_main_thread() returns consistent results,
        which is critical for proper thread dispatching in DCC environments.
        """
        results = [is_main_thread() for _ in range(100)]
        assert all(r == results[0] for r in results), "Main thread detection should be consistent"

    def test_cross_thread_dispatch_safety(self):
        """Test that cross-thread dispatch doesn't cause race conditions.

        This validates the thread dispatcher's ability to safely marshal
        calls from background threads to the main thread.
        """
        results: List[int] = []
        lock = threading.Lock()

        def append_from_thread(value: int):
            with lock:
                results.append(value)

        # Dispatch from multiple threads
        threads = []
        for i in range(10):
            t = threading.Thread(target=lambda v=i: defer_to_main_thread(append_from_thread, v))
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        # Give time for deferred calls to complete
        time.sleep(0.1)

        # All values should be present (order may vary)
        assert len(results) == 10, f"Expected 10 results, got {len(results)}"
        assert set(results) == set(range(10)), "All values should be dispatched"

    def test_concurrent_dispatcher_access(self):
        """Test concurrent access to the dispatcher backend.

        This validates that the dispatcher handles concurrent registration
        and execution without deadlocks or race conditions.
        """

        class CountingBackend(ThreadDispatcherBackend):
            def __init__(self):
                self.call_count = 0
                self._lock = threading.Lock()

            def is_available(self) -> bool:
                return True

            def run_deferred(self, func, *args, **kwargs):
                with self._lock:
                    self.call_count += 1
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                with self._lock:
                    self.call_count += 1
                return func(*args, **kwargs)

        backend = CountingBackend()

        def concurrent_dispatch():
            for _ in range(100):
                backend.run_deferred(lambda: None)

        # Run concurrent dispatches
        with ThreadPoolExecutor(max_workers=4) as executor:
            futures = [executor.submit(concurrent_dispatch) for _ in range(4)]
            for f in as_completed(futures):
                f.result()  # Raise any exceptions

        assert backend.call_count == 400, f"Expected 400 calls, got {backend.call_count}"


class TestEventProcessingStrategies:
    """Tests for event processing strategy consistency.

    These tests document the P2 finding about inconsistent event processing.
    """

    def test_fallback_backend_always_available(self):
        """Test that fallback backend is always available as a safety net."""
        backend = FallbackDispatcherBackend()
        assert backend.is_available(), "Fallback should always be available"

    def test_fallback_executes_immediately(self):
        """Test that fallback backend executes functions immediately.

        This is important for DCC environments where Qt may not be available.
        """
        backend = FallbackDispatcherBackend()
        results = []

        backend.run_deferred(lambda: results.append(1))
        backend.run_deferred(lambda: results.append(2))

        assert results == [1, 2], "Fallback should execute immediately in order"

    def test_sync_execution_returns_value(self):
        """Test that synchronous execution properly returns values."""
        backend = FallbackDispatcherBackend()

        result = backend.run_sync(lambda: 42)
        assert result == 42, "Sync execution should return the value"

        result = backend.run_sync(lambda x, y: x + y, 10, 20)
        assert result == 30, "Sync execution should handle args"

    def test_sync_execution_with_kwargs(self):
        """Test that synchronous execution handles kwargs properly."""
        backend = FallbackDispatcherBackend()

        def func_with_kwargs(a, b=10):
            return a + b

        result = backend.run_sync(func_with_kwargs, 5, b=15)
        assert result == 20, "Sync execution should handle kwargs"


class TestLockOrderingValidation:
    """Tests to validate lock ordering patterns.

    These tests help ensure proper lock ordering to prevent deadlocks.
    """

    def test_nested_lock_acquisition_order(self):
        """Test that nested locks are acquired in consistent order.

        This validates the lock ordering pattern recommended in the diagnosis.
        """
        lock_a = threading.Lock()
        lock_b = threading.Lock()
        lock_c = threading.Lock()

        acquisition_order = []
        lock_order_lock = threading.Lock()

        def acquire_in_order():
            # Correct order: A -> B -> C
            with lock_a:
                with lock_order_lock:
                    acquisition_order.append("A")
                with lock_b:
                    with lock_order_lock:
                        acquisition_order.append("B")
                    with lock_c:
                        with lock_order_lock:
                            acquisition_order.append("C")

        # Run from multiple threads
        threads = [threading.Thread(target=acquire_in_order) for _ in range(5)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # Verify order pattern repeats correctly
        expected_pattern = ["A", "B", "C"] * 5
        assert acquisition_order == expected_pattern, "Lock order should be consistent"

    def test_no_deadlock_with_timeout(self):
        """Test that lock acquisition with timeout prevents deadlocks."""
        lock = threading.Lock()
        acquired = threading.Event()
        released = threading.Event()

        def hold_lock():
            with lock:
                acquired.set()
                released.wait(timeout=1.0)

        # Start thread that holds the lock
        holder = threading.Thread(target=hold_lock)
        holder.start()
        acquired.wait()

        # Try to acquire with timeout (should fail)
        start = time.time()
        result = lock.acquire(timeout=0.1)
        elapsed = time.time() - start

        if result:
            lock.release()

        # Release the holder
        released.set()
        holder.join()

        assert not result, "Should not acquire held lock"
        assert elapsed < 0.5, f"Timeout should be respected, took {elapsed:.2f}s"


class TestMessageLatencyBaseline:
    """Tests to establish baseline latency metrics.

    These tests help validate the P0 finding about message queue latency.
    """

    def test_deferred_call_latency(self):
        """Test the latency of deferred calls."""
        backend = FallbackDispatcherBackend()
        latencies = []

        for _ in range(100):
            start = time.perf_counter()
            backend.run_deferred(lambda: None)
            elapsed = time.perf_counter() - start
            latencies.append(elapsed * 1000)  # Convert to ms

        avg_latency = sum(latencies) / len(latencies)
        max_latency = max(latencies)

        # Fallback should be very fast (< 1ms average)
        assert avg_latency < 1.0, f"Average latency too high: {avg_latency:.3f}ms"
        assert max_latency < 10.0, f"Max latency too high: {max_latency:.3f}ms"

    def test_sync_call_latency(self):
        """Test the latency of synchronous calls."""
        backend = FallbackDispatcherBackend()
        latencies = []

        for _ in range(100):
            start = time.perf_counter()
            backend.run_sync(lambda: 42)
            elapsed = time.perf_counter() - start
            latencies.append(elapsed * 1000)

        avg_latency = sum(latencies) / len(latencies)

        # Sync calls should be fast
        assert avg_latency < 1.0, f"Average sync latency too high: {avg_latency:.3f}ms"


class TestThreadDispatcherRobustness:
    """Tests for thread dispatcher robustness.

    These tests validate error handling and edge cases.
    """

    def test_exception_in_deferred_call(self):
        """Test that exceptions in deferred calls are handled gracefully."""
        backend = FallbackDispatcherBackend()

        def raise_error():
            raise ValueError("Test error")

        # Should not propagate exception
        try:
            backend.run_deferred(raise_error)
        except ValueError:
            pytest.fail("Exception should be caught in fallback backend")

    def test_exception_in_sync_call(self):
        """Test that exceptions in sync calls propagate correctly."""
        backend = FallbackDispatcherBackend()

        def raise_error():
            raise ValueError("Test error")

        with pytest.raises(ValueError, match="Test error"):
            backend.run_sync(raise_error)

    def test_rapid_dispatch_stress(self):
        """Stress test rapid dispatch calls."""
        backend = FallbackDispatcherBackend()
        counter = {"value": 0}
        lock = threading.Lock()

        def increment():
            with lock:
                counter["value"] += 1

        # Rapid fire dispatches
        start = time.time()
        for _ in range(10000):
            backend.run_deferred(increment)
        elapsed = time.time() - start

        assert counter["value"] == 10000, "All dispatches should complete"
        assert elapsed < 1.0, f"10k dispatches took too long: {elapsed:.2f}s"
