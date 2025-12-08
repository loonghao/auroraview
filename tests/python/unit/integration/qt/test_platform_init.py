"""Tests for platform backend initialization and factory functions.

This module tests the platform detection and singleton pattern
implemented in the platforms package __init__.py.

Note: These tests require qtpy to be installed because the parent
qt package imports it during package initialization.
"""

import sys
from unittest.mock import patch

import pytest

# Skip entire module if qtpy is not available
pytest.importorskip("qtpy", reason="Qt tests require qtpy")

from auroraview.integration.qt.platforms import (
    NullPlatformBackend,
    PlatformBackend,
    get_backend,
    get_platform_backend,
    reset_backend,
)


class TestGetPlatformBackend:
    """Tests for get_platform_backend factory function."""

    def test_returns_platform_backend_instance(self):
        """Test that get_platform_backend returns a PlatformBackend."""
        backend = get_platform_backend()
        assert isinstance(backend, PlatformBackend)

    @pytest.mark.skipif(sys.platform != "win32", reason="Windows only test")
    def test_returns_windows_backend_on_windows(self):
        """Test that WindowsPlatformBackend is returned on Windows."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = get_platform_backend()
        assert isinstance(backend, WindowsPlatformBackend)

    @pytest.mark.skipif(sys.platform == "win32", reason="Non-Windows only test")
    def test_returns_null_backend_on_non_windows(self):
        """Test that NullPlatformBackend is returned on non-Windows."""
        backend = get_platform_backend()
        assert isinstance(backend, NullPlatformBackend)

    def test_returns_null_backend_for_darwin(self):
        """Test that NullPlatformBackend is returned for macOS."""
        with patch.object(sys, "platform", "darwin"):
            backend = get_platform_backend()
            assert isinstance(backend, NullPlatformBackend)

    def test_returns_null_backend_for_linux(self):
        """Test that NullPlatformBackend is returned for Linux."""
        with patch.object(sys, "platform", "linux"):
            backend = get_platform_backend()
            assert isinstance(backend, NullPlatformBackend)

    def test_returns_null_backend_for_unknown_platform(self):
        """Test that NullPlatformBackend is returned for unknown platforms."""
        with patch.object(sys, "platform", "unknown"):
            backend = get_platform_backend()
            assert isinstance(backend, NullPlatformBackend)

    def test_creates_new_instance_each_call(self):
        """Test that get_platform_backend creates new instances."""
        backend1 = get_platform_backend()
        backend2 = get_platform_backend()
        assert backend1 is not backend2


class TestGetBackendSingleton:
    """Tests for get_backend singleton function."""

    def setup_method(self):
        """Reset backend before each test."""
        reset_backend()

    def teardown_method(self):
        """Reset backend after each test."""
        reset_backend()

    def test_returns_platform_backend_instance(self):
        """Test that get_backend returns a PlatformBackend."""
        backend = get_backend()
        assert isinstance(backend, PlatformBackend)

    def test_returns_same_instance(self):
        """Test that get_backend returns the same instance on subsequent calls."""
        backend1 = get_backend()
        backend2 = get_backend()
        backend3 = get_backend()
        assert backend1 is backend2
        assert backend2 is backend3

    def test_creates_instance_on_first_call(self):
        """Test that the singleton is created on first call."""
        # After reset, _backend should be None
        reset_backend()
        backend = get_backend()
        assert backend is not None

    @pytest.mark.skipif(sys.platform != "win32", reason="Windows only test")
    def test_singleton_is_windows_backend_on_windows(self):
        """Test that singleton is WindowsPlatformBackend on Windows."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = get_backend()
        assert isinstance(backend, WindowsPlatformBackend)


class TestResetBackend:
    """Tests for reset_backend function."""

    def test_clears_singleton(self):
        """Test that reset_backend clears the singleton."""
        # Get a backend to create the singleton
        backend1 = get_backend()

        # Reset
        reset_backend()

        # Get again - should be a new instance
        backend2 = get_backend()

        # While instances may be equal in value, they should be different objects
        # after reset (though on same platform they're the same type)
        assert isinstance(backend1, PlatformBackend)
        assert isinstance(backend2, PlatformBackend)

    def test_reset_allows_new_singleton(self):
        """Test that reset allows creating a fresh singleton."""
        backend1 = get_backend()
        reset_backend()
        backend2 = get_backend()
        # New singleton should be created
        assert backend1 is not backend2

    def test_reset_multiple_times(self):
        """Test that reset can be called multiple times safely."""
        reset_backend()
        reset_backend()
        reset_backend()
        # Should not raise and should still work
        backend = get_backend()
        assert backend is not None


class TestModuleExports:
    """Tests for module __all__ exports."""

    def test_all_exports_available(self):
        """Test that all expected symbols are exported."""
        from auroraview.integration.qt import platforms

        assert hasattr(platforms, "PlatformBackend")
        assert hasattr(platforms, "NullPlatformBackend")
        assert hasattr(platforms, "get_backend")
        assert hasattr(platforms, "get_platform_backend")
        assert hasattr(platforms, "reset_backend")

    def test_all_list_contents(self):
        """Test that __all__ contains expected exports."""
        from auroraview.integration.qt import platforms

        expected = [
            "PlatformBackend",
            "NullPlatformBackend",
            "get_backend",
            "get_platform_backend",
            "reset_backend",
        ]
        for name in expected:
            assert name in platforms.__all__
