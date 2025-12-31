# -*- coding: utf-8 -*-
"""Tests for thread_dispatcher module.

This module tests the thread dispatcher functionality for DCC applications.
"""

import os
import threading
import time

import pytest

from auroraview.utils.thread_dispatcher import (
    FallbackDispatcherBackend,
    ThreadDispatcherBackend,
    clear_dispatcher_backends,
    defer_to_main_thread,
    ensure_main_thread,
    get_dispatcher_backend,
    is_main_thread,
    list_dispatcher_backends,
    register_dispatcher_backend,
    run_on_main_thread,
    run_on_main_thread_sync,
    unregister_dispatcher_backend,
)


class TestThreadDispatcherBackend:
    """Tests for ThreadDispatcherBackend abstract class."""

    def test_get_name_removes_backend_suffix(self):
        """Test that get_name() removes 'Backend' suffix."""

        class TestBackend(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return True

            def run_deferred(self, func, *args, **kwargs):
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                return func(*args, **kwargs)

        backend = TestBackend()
        assert backend.get_name() == "Test"

    def test_get_name_without_backend_suffix(self):
        """Test that get_name() works without 'Backend' suffix."""

        class MyDispatcher(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return True

            def run_deferred(self, func, *args, **kwargs):
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                return func(*args, **kwargs)

        backend = MyDispatcher()
        assert backend.get_name() == "MyDispatcher"

    def test_is_main_thread_default_implementation(self):
        """Test default is_main_thread() implementation."""

        class TestBackend(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return True

            def run_deferred(self, func, *args, **kwargs):
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                return func(*args, **kwargs)

        backend = TestBackend()
        # Should return True when called from main thread
        assert backend.is_main_thread() is True


class TestFallbackDispatcherBackend:
    """Tests for FallbackDispatcherBackend."""

    def test_is_available_always_true(self):
        """Test that fallback backend is always available."""
        backend = FallbackDispatcherBackend()
        assert backend.is_available() is True

    def test_run_deferred_executes_function(self):
        """Test that run_deferred executes the function."""
        backend = FallbackDispatcherBackend()
        result = []

        def append_value():
            result.append(42)

        backend.run_deferred(append_value)
        assert result == [42]

    def test_run_deferred_with_args(self):
        """Test run_deferred with arguments."""
        backend = FallbackDispatcherBackend()
        result = []

        def append_values(a, b, c=None):
            result.extend([a, b, c])

        backend.run_deferred(append_values, 1, 2, c=3)
        assert result == [1, 2, 3]

    def test_run_sync_returns_value(self):
        """Test that run_sync returns the function's return value."""
        backend = FallbackDispatcherBackend()

        def get_value():
            return 42

        result = backend.run_sync(get_value)
        assert result == 42

    def test_run_sync_with_args(self):
        """Test run_sync with arguments."""
        backend = FallbackDispatcherBackend()

        def add(a, b, c=0):
            return a + b + c

        result = backend.run_sync(add, 1, 2, c=3)
        assert result == 6

    def test_run_sync_propagates_exception(self):
        """Test that run_sync propagates exceptions."""
        backend = FallbackDispatcherBackend()

        def raise_error():
            raise ValueError("test error")

        with pytest.raises(ValueError, match="test error"):
            backend.run_sync(raise_error)


class TestBackendRegistration:
    """Tests for backend registration system."""

    def test_register_custom_backend(self):
        """Test registering a custom backend."""

        class CustomBackend(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return False  # Not available for testing

            def run_deferred(self, func, *args, **kwargs):
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                return func(*args, **kwargs)

        # Register with high priority
        register_dispatcher_backend(CustomBackend, priority=1000)

        # Check it's in the list
        backends = list_dispatcher_backends()
        custom_found = False
        for priority, name, available in backends:
            if name == "Custom":
                custom_found = True
                assert priority == 1000
                assert available is False
                break

        assert custom_found, "Custom backend should be in the list"

    def test_register_backend_with_name(self):
        """Test registering a backend with a custom name."""

        class MyBackend(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return False

            def run_deferred(self, func, *args, **kwargs):
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                return func(*args, **kwargs)

        register_dispatcher_backend(MyBackend, priority=999, name="CustomName")

        backends = list_dispatcher_backends()
        found = False
        for priority, name, _available in backends:
            if name == "CustomName":
                found = True
                assert priority == 999
                break

        assert found, "Backend with custom name should be in the list"

    def test_unregister_backend(self):
        """Test unregistering a backend."""

        class TempBackend(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return False

            def run_deferred(self, func, *args, **kwargs):
                func(*args, **kwargs)

            def run_sync(self, func, *args, **kwargs):
                return func(*args, **kwargs)

        register_dispatcher_backend(TempBackend, priority=998)

        # Verify it's registered
        backends = list_dispatcher_backends()
        assert any(name == "Temp" for _, name, _ in backends)

        # Unregister
        result = unregister_dispatcher_backend(TempBackend)
        assert result is True

        # Verify it's gone
        backends = list_dispatcher_backends()
        assert not any(name == "Temp" for _, name, _ in backends)

    def test_unregister_nonexistent_backend(self):
        """Test unregistering a backend that doesn't exist."""

        class NonExistentBackend(ThreadDispatcherBackend):
            def is_available(self) -> bool:
                return False

            def run_deferred(self, func, *args, **kwargs):
                pass

            def run_sync(self, func, *args, **kwargs):
                pass

        result = unregister_dispatcher_backend(NonExistentBackend)
        assert result is False

    def test_clear_dispatcher_backends(self):
        """Test clearing all backends."""
        # Clear all
        clear_dispatcher_backends()

        # Should be empty now
        backends = list_dispatcher_backends()
        # After clearing, built-in backends are re-registered on first access
        # So we just verify the cache was cleared
        assert len(backends) >= 1  # At least fallback should be registered

    def test_list_dispatcher_backends(self):
        """Test listing all registered backends."""
        backends = list_dispatcher_backends()

        # Should have at least the built-in backends
        assert len(backends) >= 1

        # Each entry should be a tuple of (priority, name, is_available)
        for entry in backends:
            assert len(entry) == 3
            priority, name, available = entry
            assert isinstance(priority, int)
            assert isinstance(name, str)
            assert isinstance(available, bool)

    def test_backends_sorted_by_priority(self):
        """Test that backends are sorted by priority (highest first)."""
        backends = list_dispatcher_backends()

        priorities = [p for p, _, _ in backends]
        assert priorities == sorted(priorities, reverse=True)

    def test_string_based_registration(self):
        """Test registering a backend using string path."""
        # Register using string path
        register_dispatcher_backend(
            "auroraview.utils.thread_dispatcher:FallbackDispatcherBackend",
            priority=1,
            name="StringFallback",
        )

        backends = list_dispatcher_backends()
        found = False
        for priority, name, available in backends:
            if name == "StringFallback":
                found = True
                assert priority == 1
                assert available is True  # Fallback is always available
                break

        assert found, "String-registered backend should be in the list"

    def test_invalid_string_registration(self):
        """Test that invalid string paths are handled gracefully."""
        # Register with invalid module path
        register_dispatcher_backend(
            "nonexistent.module:SomeBackend",
            priority=1,
            name="Invalid",
        )

        # Should be in the list but not available
        backends = list_dispatcher_backends()
        found = False
        for _priority, name, available in backends:
            if name == "Invalid":
                found = True
                assert available is False
                break

        assert found, "Invalid backend should still be in the list"


class TestConvenienceFunctions:
    """Tests for convenience functions."""

    def test_run_on_main_thread(self):
        """Test run_on_main_thread function."""
        result = []

        def append_value():
            result.append(42)

        run_on_main_thread(append_value)
        # Give some time for deferred execution
        time.sleep(0.1)
        assert 42 in result

    def test_run_on_main_thread_with_args(self):
        """Test run_on_main_thread with arguments."""
        result = []

        def append_values(a, b, c=None):
            result.extend([a, b, c])

        run_on_main_thread(append_values, 1, 2, c=3)
        time.sleep(0.1)
        assert result == [1, 2, 3]

    def test_run_on_main_thread_sync(self):
        """Test run_on_main_thread_sync function."""

        def get_value():
            return 42

        result = run_on_main_thread_sync(get_value)
        assert result == 42

    def test_run_on_main_thread_sync_with_args(self):
        """Test run_on_main_thread_sync with arguments."""

        def add(a, b, c=0):
            return a + b + c

        result = run_on_main_thread_sync(add, 1, 2, c=3)
        assert result == 6

    def test_run_on_main_thread_sync_propagates_exception(self):
        """Test that run_on_main_thread_sync propagates exceptions."""

        def raise_error():
            raise ValueError("test error")

        with pytest.raises(ValueError, match="test error"):
            run_on_main_thread_sync(raise_error)

    def test_is_main_thread(self):
        """Test is_main_thread function."""
        # Should return True when called from main thread
        assert is_main_thread() is True

    def test_is_main_thread_from_background_thread(self):
        """Test is_main_thread from a background thread."""
        result = []

        def check_main_thread():
            result.append(is_main_thread())

        thread = threading.Thread(target=check_main_thread)
        thread.start()
        thread.join()

        # Should return False from background thread
        assert result == [False]


class TestDecorators:
    """Tests for decorator functions."""

    def test_ensure_main_thread_from_main_thread(self):
        """Test ensure_main_thread decorator when already on main thread."""
        call_count = [0]

        @ensure_main_thread
        def increment():
            call_count[0] += 1
            return call_count[0]

        result = increment()
        assert result == 1
        assert call_count[0] == 1

    def test_ensure_main_thread_preserves_return_value(self):
        """Test that ensure_main_thread preserves return values."""

        @ensure_main_thread
        def get_value(x, y=0):
            return x + y

        result = get_value(10, y=5)
        assert result == 15

    def test_ensure_main_thread_preserves_exception(self):
        """Test that ensure_main_thread preserves exceptions."""

        @ensure_main_thread
        def raise_error():
            raise ValueError("test error")

        with pytest.raises(ValueError, match="test error"):
            raise_error()

    def test_defer_to_main_thread(self):
        """Test defer_to_main_thread decorator."""
        result = []

        @defer_to_main_thread
        def append_value(x):
            result.append(x)

        # Should return None immediately
        ret = append_value(42)
        assert ret is None

        # Give some time for deferred execution
        time.sleep(0.1)
        assert 42 in result


class TestCrossThreadExecution:
    """Tests for cross-thread execution scenarios."""

    def test_run_sync_from_background_thread(self):
        """Test run_on_main_thread_sync from a background thread."""
        result = []

        def get_thread_info():
            return threading.current_thread().name

        def background_task():
            # Call run_on_main_thread_sync from background thread
            main_thread_name = run_on_main_thread_sync(get_thread_info)
            result.append(main_thread_name)

        thread = threading.Thread(target=background_task)
        thread.start()
        thread.join()

        # The function should have been executed
        assert len(result) == 1

    def test_concurrent_deferred_calls(self):
        """Test multiple concurrent deferred calls."""
        result = []
        lock = threading.Lock()

        def append_value(x):
            with lock:
                result.append(x)

        # Queue multiple deferred calls
        for i in range(10):
            run_on_main_thread(append_value, i)

        # Give time for all to execute
        time.sleep(0.5)

        # All values should be present
        assert len(result) == 10
        assert set(result) == set(range(10))


class TestMockDCCBackends:
    """Tests for DCC backend behavior using mocks."""

    def test_maya_backend_detection(self):
        """Test Maya backend detection with mock."""
        from auroraview.utils.thread_dispatcher import MayaDispatcherBackend

        backend = MayaDispatcherBackend()

        # Without Maya installed, should return False
        assert backend.is_available() is False

    def test_houdini_backend_detection(self):
        """Test Houdini backend detection with mock."""
        from auroraview.utils.thread_dispatcher import HoudiniDispatcherBackend

        backend = HoudiniDispatcherBackend()

        # Without Houdini installed, should return False
        assert backend.is_available() is False

    def test_blender_backend_detection(self):
        """Test Blender backend detection with mock."""
        from auroraview.utils.thread_dispatcher import BlenderDispatcherBackend

        backend = BlenderDispatcherBackend()

        # Without Blender installed, should return False
        assert backend.is_available() is False

    def test_nuke_backend_detection(self):
        """Test Nuke backend detection with mock."""
        from auroraview.utils.thread_dispatcher import NukeDispatcherBackend

        backend = NukeDispatcherBackend()

        # Without Nuke installed, should return False
        assert backend.is_available() is False

    def test_max_backend_detection(self):
        """Test 3ds Max backend detection with mock."""
        from auroraview.utils.thread_dispatcher import MaxDispatcherBackend

        backend = MaxDispatcherBackend()

        # Without 3ds Max installed, should return False
        assert backend.is_available() is False

    def test_unreal_backend_detection(self):
        """Test Unreal Engine backend detection with mock."""
        from auroraview.utils.thread_dispatcher import UnrealDispatcherBackend

        backend = UnrealDispatcherBackend()

        # Without Unreal installed, should return False
        assert backend.is_available() is False


class TestQtBackend:
    """Tests for Qt backend."""

    def test_qt_backend_without_qt_app(self):
        """Test Qt backend when no Qt app is running."""
        from auroraview.utils.thread_dispatcher import QtDispatcherBackend

        backend = QtDispatcherBackend()

        # Without a Qt app instance, should return False
        # (unless tests are running in a Qt environment)
        # This test just verifies the check doesn't crash
        available = backend.is_available()
        assert isinstance(available, bool)


class TestEdgeCases:
    """Tests for edge cases and error handling."""

    def test_run_deferred_with_no_args(self):
        """Test run_deferred with a function that takes no arguments."""
        result = []

        def no_args():
            result.append("called")

        run_on_main_thread(no_args)
        time.sleep(0.1)
        assert result == ["called"]

    def test_run_sync_with_none_return(self):
        """Test run_sync with a function that returns None."""

        def return_none():
            return None

        result = run_on_main_thread_sync(return_none)
        assert result is None

    def test_run_sync_with_complex_return(self):
        """Test run_sync with complex return types."""

        def return_complex():
            return {"key": [1, 2, 3], "nested": {"a": "b"}}

        result = run_on_main_thread_sync(return_complex)
        assert result == {"key": [1, 2, 3], "nested": {"a": "b"}}

    def test_get_dispatcher_backend_caching(self):
        """Test that get_dispatcher_backend caches the result."""
        backend1 = get_dispatcher_backend()
        backend2 = get_dispatcher_backend()

        # Should return the same instance
        assert backend1 is backend2

    def test_decorator_preserves_function_metadata(self):
        """Test that decorators preserve function metadata."""

        @ensure_main_thread
        def my_function():
            """My docstring."""
            pass

        assert my_function.__name__ == "my_function"
        assert my_function.__doc__ == "My docstring."

        @defer_to_main_thread
        def another_function():
            """Another docstring."""
            pass

        assert another_function.__name__ == "another_function"
        assert another_function.__doc__ == "Another docstring."


class TestEnvironmentVariableOverride:
    """Tests for environment variable backend override."""

    def test_env_var_override_fallback(self):
        """Test forcing fallback backend via environment variable."""
        # Clear cache first
        clear_dispatcher_backends()

        # Set environment variable
        old_value = os.environ.get("AURORAVIEW_DISPATCHER")
        try:
            os.environ["AURORAVIEW_DISPATCHER"] = "fallback"

            backend = get_dispatcher_backend()
            # FallbackDispatcherBackend.get_name() returns "FallbackDispatcher"
            # but we register it with name="Fallback"
            assert "Fallback" in backend.get_name()
        finally:
            # Restore original value
            if old_value is None:
                os.environ.pop("AURORAVIEW_DISPATCHER", None)
            else:
                os.environ["AURORAVIEW_DISPATCHER"] = old_value
            # Clear cache
            clear_dispatcher_backends()

    def test_env_var_case_insensitive(self):
        """Test that environment variable is case-insensitive."""
        clear_dispatcher_backends()

        old_value = os.environ.get("AURORAVIEW_DISPATCHER")
        try:
            os.environ["AURORAVIEW_DISPATCHER"] = "FALLBACK"

            backend = get_dispatcher_backend()
            assert "Fallback" in backend.get_name()
        finally:
            if old_value is None:
                os.environ.pop("AURORAVIEW_DISPATCHER", None)
            else:
                os.environ["AURORAVIEW_DISPATCHER"] = old_value
            clear_dispatcher_backends()

    def test_env_var_invalid_backend(self):
        """Test behavior with invalid environment variable value."""
        clear_dispatcher_backends()

        old_value = os.environ.get("AURORAVIEW_DISPATCHER")
        try:
            os.environ["AURORAVIEW_DISPATCHER"] = "nonexistent_backend"

            # Should fall back to normal priority-based selection
            backend = get_dispatcher_backend()
            # Should get some backend (not crash)
            assert backend is not None
        finally:
            if old_value is None:
                os.environ.pop("AURORAVIEW_DISPATCHER", None)
            else:
                os.environ["AURORAVIEW_DISPATCHER"] = old_value
            clear_dispatcher_backends()
