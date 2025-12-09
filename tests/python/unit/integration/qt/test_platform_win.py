"""Tests for Windows platform backend implementation.

This module tests the WindowsPlatformBackend class using mocks
to avoid actual Win32 API calls.

Note: These tests require qtpy to be installed because the parent
qt package imports it during package initialization.
"""

import sys
from unittest.mock import MagicMock, patch

import pytest

# Skip entire module if qtpy is not available
pytest.importorskip("qtpy", reason="Qt tests require qtpy")

# Skip all tests if not on Windows
pytestmark = pytest.mark.skipif(sys.platform != "win32", reason="Windows only tests")


@pytest.fixture
def mock_win32_apis():
    """Fixture that mocks all Win32 API functions used by win.py.

    The win.py module creates module-level GetWindowLong/SetWindowLong
    aliases that point to either GetWindowLongPtrW (64-bit) or
    GetWindowLongW (32-bit). We need to mock these aliases, not the
    underlying user32 functions.
    """
    with patch("auroraview.integration.qt.platforms.win.GetWindowLong") as mock_get, patch(
        "auroraview.integration.qt.platforms.win.SetWindowLong"
    ) as mock_set, patch("auroraview.integration.qt.platforms.win.user32") as mock_user32:
        # Default return values
        mock_get.return_value = 0
        mock_set.return_value = 1
        mock_user32.SetWindowPos.return_value = 1
        mock_user32.SetParent.return_value = 1
        mock_user32.SetLayeredWindowAttributes.return_value = 1

        yield {
            "GetWindowLong": mock_get,
            "SetWindowLong": mock_set,
            "user32": mock_user32,
        }


class TestWindowsPlatformBackendImport:
    """Tests for WindowsPlatformBackend import and instantiation."""

    def test_can_import_on_windows(self):
        """Test that WindowsPlatformBackend can be imported on Windows."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        assert WindowsPlatformBackend is not None

    def test_is_platform_backend_subclass(self):
        """Test that WindowsPlatformBackend is a PlatformBackend subclass."""
        from auroraview.integration.qt.platforms.base import PlatformBackend
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        assert issubclass(WindowsPlatformBackend, PlatformBackend)

    def test_can_instantiate(self):
        """Test that WindowsPlatformBackend can be instantiated."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = WindowsPlatformBackend()
        assert backend is not None


class TestWindowsConstants:
    """Tests for Windows API constants defined in win.py."""

    def test_window_style_constants_exist(self):
        """Test that all window style constants are defined."""
        from auroraview.integration.qt.platforms import win

        # Basic styles
        assert hasattr(win, "WS_CHILD")
        assert hasattr(win, "WS_POPUP")
        assert hasattr(win, "WS_CAPTION")
        assert hasattr(win, "WS_THICKFRAME")
        assert hasattr(win, "WS_OVERLAPPEDWINDOW")

        # Extended styles
        assert hasattr(win, "WS_EX_LAYERED")
        assert hasattr(win, "WS_EX_APPWINDOW")
        assert hasattr(win, "WS_EX_TOOLWINDOW")

        # Clip styles
        assert hasattr(win, "WS_CLIPCHILDREN")
        assert hasattr(win, "WS_CLIPSIBLINGS")

    def test_window_style_values(self):
        """Test that window style constants have correct values."""
        from auroraview.integration.qt.platforms import win

        assert win.WS_CHILD == 0x40000000
        assert win.WS_POPUP == 0x80000000
        assert win.WS_EX_LAYERED == 0x00080000
        assert win.WS_CLIPCHILDREN == 0x02000000
        assert win.WS_CLIPSIBLINGS == 0x04000000

    def test_swp_flags_exist(self):
        """Test that SetWindowPos flags are defined."""
        from auroraview.integration.qt.platforms import win

        assert hasattr(win, "SWP_FRAMECHANGED")
        assert hasattr(win, "SWP_NOMOVE")
        assert hasattr(win, "SWP_NOSIZE")
        assert hasattr(win, "SWP_NOZORDER")
        assert hasattr(win, "SWP_NOACTIVATE")


class TestApplyClipStyles:
    """Tests for apply_clip_styles_to_parent method."""

    def test_applies_clip_styles_when_not_set(self, mock_win32_apis):
        """Test that clip styles are applied when not already set."""
        from auroraview.integration.qt.platforms.win import (
            WS_CLIPCHILDREN,
            WS_CLIPSIBLINGS,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()

        # Current style doesn't have clip styles
        mock_win32_apis["GetWindowLong"].return_value = 0
        mock_win32_apis["SetWindowLong"].return_value = 1

        result = backend.apply_clip_styles_to_parent(12345)

        assert result is True
        mock_win32_apis["GetWindowLong"].assert_called_once()
        mock_win32_apis["SetWindowLong"].assert_called_once()
        # Verify the new style includes clip styles
        call_args = mock_win32_apis["SetWindowLong"].call_args
        new_style = call_args[0][2]
        assert new_style & WS_CLIPCHILDREN
        assert new_style & WS_CLIPSIBLINGS

    def test_skips_when_already_set(self, mock_win32_apis):
        """Test that no changes are made when styles already set."""
        from auroraview.integration.qt.platforms.win import (
            WS_CLIPCHILDREN,
            WS_CLIPSIBLINGS,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()

        # Current style already has clip styles
        mock_win32_apis["GetWindowLong"].return_value = WS_CLIPCHILDREN | WS_CLIPSIBLINGS

        result = backend.apply_clip_styles_to_parent(12345)

        assert result is True
        # SetWindowLong should not be called
        mock_win32_apis["SetWindowLong"].assert_not_called()

    def test_returns_false_on_exception(self, mock_win32_apis):
        """Test that False is returned on exception."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = WindowsPlatformBackend()

        mock_win32_apis["GetWindowLong"].side_effect = Exception("Test error")

        result = backend.apply_clip_styles_to_parent(12345)

        assert result is False


class TestPrepareHwndForContainer:
    """Tests for prepare_hwnd_for_container method."""

    def test_removes_frame_styles(self, mock_win32_apis):
        """Test that frame/border styles are removed."""
        from auroraview.integration.qt.platforms.win import (
            WS_CAPTION,
            WS_CHILD,
            WS_POPUP,
            WS_THICKFRAME,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()

        # Current style has popup and caption
        mock_win32_apis["GetWindowLong"].return_value = WS_POPUP | WS_CAPTION | WS_THICKFRAME
        mock_win32_apis["SetWindowLong"].return_value = 1

        result = backend.prepare_hwnd_for_container(12345)

        assert result is True
        # Verify styles were modified correctly
        style_calls = mock_win32_apis["SetWindowLong"].call_args_list
        # First call is for GWL_STYLE, second for GWL_EXSTYLE
        style_call = style_calls[0]
        new_style = style_call[0][2]
        assert new_style & WS_CHILD  # WS_CHILD added
        assert not (new_style & WS_POPUP)  # WS_POPUP removed

    def test_returns_false_on_exception(self, mock_win32_apis):
        """Test that False is returned on exception."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = WindowsPlatformBackend()

        mock_win32_apis["GetWindowLong"].side_effect = Exception("Test error")

        result = backend.prepare_hwnd_for_container(12345)

        assert result is False


class TestHideWindowForInit:
    """Tests for hide_window_for_init method."""

    def test_adds_layered_style(self, mock_win32_apis):
        """Test that WS_EX_LAYERED is added."""
        from auroraview.integration.qt.platforms.win import (
            WS_EX_LAYERED,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()

        mock_win32_apis["GetWindowLong"].return_value = 0
        mock_win32_apis["SetWindowLong"].return_value = 1

        result = backend.hide_window_for_init(12345)

        assert result is True
        # Verify WS_EX_LAYERED was added
        style_call = mock_win32_apis["SetWindowLong"].call_args
        new_ex_style = style_call[0][2]
        assert new_ex_style & WS_EX_LAYERED
        # Verify alpha was set to 0
        mock_win32_apis["user32"].SetLayeredWindowAttributes.assert_called_once()

    def test_returns_false_on_exception(self, mock_win32_apis):
        """Test that False is returned on exception."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = WindowsPlatformBackend()

        mock_win32_apis["GetWindowLong"].side_effect = Exception("Test error")

        result = backend.hide_window_for_init(12345)

        assert result is False


class TestShowWindowAfterInit:
    """Tests for show_window_after_init method."""

    def test_removes_layered_style(self, mock_win32_apis):
        """Test that WS_EX_LAYERED is removed."""
        from auroraview.integration.qt.platforms.win import (
            WS_EX_LAYERED,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()

        mock_win32_apis["GetWindowLong"].return_value = WS_EX_LAYERED
        mock_win32_apis["SetWindowLong"].return_value = 1

        result = backend.show_window_after_init(12345)

        assert result is True
        # Verify alpha was set to 255 first
        mock_win32_apis["user32"].SetLayeredWindowAttributes.assert_called_once()
        # Verify WS_EX_LAYERED was removed
        style_call = mock_win32_apis["SetWindowLong"].call_args
        new_ex_style = style_call[0][2]
        assert not (new_ex_style & WS_EX_LAYERED)

    def test_returns_false_on_exception(self, mock_win32_apis):
        """Test that False is returned on exception."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = WindowsPlatformBackend()

        mock_win32_apis["user32"].SetLayeredWindowAttributes.side_effect = Exception("Test error")

        result = backend.show_window_after_init(12345)

        assert result is False


class TestEnsureNativeChildStyle:
    """Tests for ensure_native_child_style method."""

    def test_applies_child_style_when_missing(self, mock_win32_apis):
        """Test that WS_CHILD is applied when missing."""
        from auroraview.integration.qt.platforms.win import (
            WS_CHILD,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()
        mock_container = MagicMock()
        mock_container.winId.return_value = 67890

        # Current style doesn't have WS_CHILD
        mock_win32_apis["GetWindowLong"].return_value = 0
        mock_win32_apis["SetWindowLong"].return_value = 1

        # Should not raise
        backend.ensure_native_child_style(12345, mock_container)

        # Verify WS_CHILD was added
        style_call = mock_win32_apis["SetWindowLong"].call_args
        new_style = style_call[0][2]
        assert new_style & WS_CHILD
        # Verify SetParent was called
        mock_win32_apis["user32"].SetParent.assert_called_once()

    def test_skips_when_child_style_set(self, mock_win32_apis):
        """Test that nothing happens when WS_CHILD already set."""
        from auroraview.integration.qt.platforms.win import (
            WS_CHILD,
            WindowsPlatformBackend,
        )

        backend = WindowsPlatformBackend()
        mock_container = MagicMock()
        mock_container.winId.return_value = 67890

        # Current style already has WS_CHILD
        mock_win32_apis["GetWindowLong"].return_value = WS_CHILD

        backend.ensure_native_child_style(12345, mock_container)

        # SetWindowLong and SetParent should not be called
        mock_win32_apis["SetWindowLong"].assert_not_called()
        mock_win32_apis["user32"].SetParent.assert_not_called()

    def test_handles_invalid_container(self, mock_win32_apis):
        """Test that invalid container is handled gracefully."""
        from auroraview.integration.qt.platforms.win import WindowsPlatformBackend

        backend = WindowsPlatformBackend()
        mock_container = MagicMock()
        mock_container.winId.return_value = 0  # Invalid HWND

        # Should not raise
        backend.ensure_native_child_style(12345, mock_container)
