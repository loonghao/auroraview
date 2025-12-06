"""Tests for platform abstraction base classes.

This module tests the abstract base class and null implementation
for platform-specific window operations.
"""

from abc import ABC

import pytest

from auroraview.integration.qt.platforms.base import (
    NullPlatformBackend,
    PlatformBackend,
)


class TestPlatformBackendInterface:
    """Tests for PlatformBackend abstract base class."""

    def test_is_abstract_class(self):
        """Test that PlatformBackend is an abstract class."""
        assert issubclass(PlatformBackend, ABC)

    def test_cannot_instantiate_directly(self):
        """Test that PlatformBackend cannot be instantiated directly."""
        with pytest.raises(TypeError):
            PlatformBackend()

    def test_has_required_abstract_methods(self):
        """Test that PlatformBackend defines all required abstract methods."""
        abstract_methods = PlatformBackend.__abstractmethods__
        expected_methods = {
            "apply_clip_styles_to_parent",
            "prepare_hwnd_for_container",
            "hide_window_for_init",
            "show_window_after_init",
            "ensure_native_child_style",
        }
        assert abstract_methods == expected_methods

    def test_apply_clip_styles_to_parent_signature(self):
        """Test apply_clip_styles_to_parent method signature."""
        import inspect

        sig = inspect.signature(PlatformBackend.apply_clip_styles_to_parent)
        params = list(sig.parameters.keys())
        assert "self" in params
        assert "parent_hwnd" in params

    def test_prepare_hwnd_for_container_signature(self):
        """Test prepare_hwnd_for_container method signature."""
        import inspect

        sig = inspect.signature(PlatformBackend.prepare_hwnd_for_container)
        params = list(sig.parameters.keys())
        assert "self" in params
        assert "hwnd" in params

    def test_hide_window_for_init_signature(self):
        """Test hide_window_for_init method signature."""
        import inspect

        sig = inspect.signature(PlatformBackend.hide_window_for_init)
        params = list(sig.parameters.keys())
        assert "self" in params
        assert "hwnd" in params

    def test_show_window_after_init_signature(self):
        """Test show_window_after_init method signature."""
        import inspect

        sig = inspect.signature(PlatformBackend.show_window_after_init)
        params = list(sig.parameters.keys())
        assert "self" in params
        assert "hwnd" in params

    def test_ensure_native_child_style_signature(self):
        """Test ensure_native_child_style method signature."""
        import inspect

        sig = inspect.signature(PlatformBackend.ensure_native_child_style)
        params = list(sig.parameters.keys())
        assert "self" in params
        assert "hwnd" in params
        assert "container" in params


class TestNullPlatformBackend:
    """Tests for NullPlatformBackend no-op implementation."""

    @pytest.fixture
    def backend(self):
        """Create a NullPlatformBackend instance."""
        return NullPlatformBackend()

    def test_is_platform_backend_subclass(self):
        """Test that NullPlatformBackend is a PlatformBackend subclass."""
        assert issubclass(NullPlatformBackend, PlatformBackend)

    def test_can_instantiate(self, backend):
        """Test that NullPlatformBackend can be instantiated."""
        assert backend is not None
        assert isinstance(backend, PlatformBackend)

    def test_apply_clip_styles_to_parent_returns_false(self, backend):
        """Test that apply_clip_styles_to_parent returns False."""
        result = backend.apply_clip_styles_to_parent(12345)
        assert result is False

    def test_prepare_hwnd_for_container_returns_false(self, backend):
        """Test that prepare_hwnd_for_container returns False."""
        result = backend.prepare_hwnd_for_container(12345)
        assert result is False

    def test_hide_window_for_init_returns_false(self, backend):
        """Test that hide_window_for_init returns False."""
        result = backend.hide_window_for_init(12345)
        assert result is False

    def test_show_window_after_init_returns_false(self, backend):
        """Test that show_window_after_init returns False."""
        result = backend.show_window_after_init(12345)
        assert result is False

    def test_ensure_native_child_style_returns_none(self, backend):
        """Test that ensure_native_child_style returns None (no-op)."""
        result = backend.ensure_native_child_style(12345, None)
        assert result is None

    def test_all_methods_are_no_ops(self, backend):
        """Test that all methods are no-ops that don't raise exceptions."""
        # These should all complete without raising
        backend.apply_clip_styles_to_parent(0)
        backend.prepare_hwnd_for_container(0)
        backend.hide_window_for_init(0)
        backend.show_window_after_init(0)
        backend.ensure_native_child_style(0, None)

    def test_methods_accept_any_hwnd(self, backend):
        """Test that methods accept any HWND value."""
        for hwnd in [0, 1, 12345, 0xFFFFFFFF, -1]:
            assert backend.apply_clip_styles_to_parent(hwnd) is False
            assert backend.prepare_hwnd_for_container(hwnd) is False
            assert backend.hide_window_for_init(hwnd) is False
            assert backend.show_window_after_init(hwnd) is False


class TestCustomPlatformBackend:
    """Tests for creating custom PlatformBackend implementations."""

    def test_can_create_custom_implementation(self):
        """Test that custom implementations can be created."""

        class CustomBackend(PlatformBackend):
            def apply_clip_styles_to_parent(self, parent_hwnd: int) -> bool:
                return True

            def prepare_hwnd_for_container(self, hwnd: int) -> bool:
                return True

            def hide_window_for_init(self, hwnd: int) -> bool:
                return True

            def show_window_after_init(self, hwnd: int) -> bool:
                return True

            def ensure_native_child_style(self, hwnd: int, container) -> None:
                pass

        backend = CustomBackend()
        assert isinstance(backend, PlatformBackend)
        assert backend.apply_clip_styles_to_parent(123) is True
