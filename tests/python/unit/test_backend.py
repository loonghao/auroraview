"""Tests for the backend abstraction module."""

import os
import sys
from unittest import mock

import pytest

from auroraview.core.backend import (
    ENV_BACKEND,
    BackendType,
    get_available_backends,
    get_backend_type,
    get_default_backend,
    is_backend_available,
    set_backend_type,
)


class TestBackendType:
    """Tests for BackendType enum."""

    def test_backend_type_values(self):
        """Test that all backend types have correct values."""
        assert BackendType.WRY.value == "wry"
        assert BackendType.WEBVIEW2.value == "webview2"
        assert BackendType.WKWEBVIEW.value == "wkwebview"
        assert BackendType.WEBKITGTK.value == "webkitgtk"

    def test_from_string_valid(self):
        """Test parsing valid backend names."""
        assert BackendType.from_string("wry") == BackendType.WRY
        assert BackendType.from_string("WRY") == BackendType.WRY
        assert BackendType.from_string("webview2") == BackendType.WEBVIEW2
        assert BackendType.from_string("wv2") == BackendType.WEBVIEW2
        assert BackendType.from_string("wkwebview") == BackendType.WKWEBVIEW
        assert BackendType.from_string("webkit") == BackendType.WKWEBVIEW
        assert BackendType.from_string("webkitgtk") == BackendType.WEBKITGTK
        assert BackendType.from_string("gtk") == BackendType.WEBKITGTK

    def test_from_string_invalid(self):
        """Test parsing invalid backend names raises ValueError."""
        with pytest.raises(ValueError, match="Unknown backend"):
            BackendType.from_string("invalid")

        with pytest.raises(ValueError, match="Unknown backend"):
            BackendType.from_string("")


class TestGetDefaultBackend:
    """Tests for get_default_backend function."""

    def test_default_is_wry(self):
        """Test that default backend is WRY."""
        assert get_default_backend() == BackendType.WRY


class TestGetAvailableBackends:
    """Tests for get_available_backends function."""

    def test_wry_always_available(self):
        """Test that WRY is always in available backends."""
        backends = get_available_backends()
        assert BackendType.WRY in backends

    @mock.patch.object(sys, "platform", "win32")
    def test_windows_backends(self):
        """Test available backends on Windows."""
        backends = get_available_backends()
        assert BackendType.WRY in backends
        assert BackendType.WEBVIEW2 in backends

    @mock.patch.object(sys, "platform", "darwin")
    def test_macos_backends(self):
        """Test available backends on macOS."""
        backends = get_available_backends()
        assert BackendType.WRY in backends
        assert BackendType.WKWEBVIEW in backends

    @mock.patch.object(sys, "platform", "linux")
    def test_linux_backends(self):
        """Test available backends on Linux."""
        backends = get_available_backends()
        assert BackendType.WRY in backends
        assert BackendType.WEBKITGTK in backends


class TestGetBackendType:
    """Tests for get_backend_type function."""

    def test_default_without_env(self):
        """Test that default is returned when env var not set."""
        with mock.patch.dict(os.environ, {}, clear=True):
            # Remove the env var if it exists
            os.environ.pop(ENV_BACKEND, None)
            assert get_backend_type() == BackendType.WRY

    def test_env_override(self):
        """Test that environment variable overrides default."""
        with mock.patch.dict(os.environ, {ENV_BACKEND: "webview2"}):
            assert get_backend_type() == BackendType.WEBVIEW2

    def test_env_invalid_raises(self):
        """Test that invalid env value raises ValueError."""
        with mock.patch.dict(os.environ, {ENV_BACKEND: "invalid"}):
            with pytest.raises(ValueError):
                get_backend_type()


class TestSetBackendType:
    """Tests for set_backend_type function."""

    def test_set_backend(self):
        """Test setting backend type via function."""
        original = os.environ.get(ENV_BACKEND)
        try:
            set_backend_type(BackendType.WEBVIEW2)
            assert os.environ[ENV_BACKEND] == "webview2"
        finally:
            # Restore original value
            if original is None:
                os.environ.pop(ENV_BACKEND, None)
            else:
                os.environ[ENV_BACKEND] = original


class TestIsBackendAvailable:
    """Tests for is_backend_available function."""

    def test_wry_always_available(self):
        """Test that WRY is always available."""
        assert is_backend_available(BackendType.WRY) is True

    @mock.patch.object(sys, "platform", "win32")
    def test_webview2_on_windows(self):
        """Test WebView2 availability on Windows."""
        assert is_backend_available(BackendType.WEBVIEW2) is True
        assert is_backend_available(BackendType.WKWEBVIEW) is False

    @mock.patch.object(sys, "platform", "darwin")
    def test_wkwebview_on_macos(self):
        """Test WKWebView availability on macOS."""
        assert is_backend_available(BackendType.WKWEBVIEW) is True
        assert is_backend_available(BackendType.WEBVIEW2) is False
