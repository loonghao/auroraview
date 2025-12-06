"""Integration tests for WebViewPool pre-warming functionality.

This module tests actual WebView2 pre-warming without mocks,
verifying the real performance benefits.
"""

import sys

import pytest

# Skip all tests if not on Windows (WebView2 is Windows-only)
pytestmark = pytest.mark.skipif(sys.platform != "win32", reason="WebView2 requires Windows")


class TestWebViewPoolIntegration:
    """Integration tests for WebViewPool with actual WebView2."""

    def setup_method(self):
        """Reset pool state before each test."""
        from auroraview.integration.qt.pool import WebViewPool

        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        from auroraview.integration.qt.pool import WebViewPool

        try:
            WebViewPool.cleanup()
        except Exception:
            pass
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    @pytest.mark.slow
    def test_actual_prewarm(self):
        """Test actual WebView2 pre-warming."""
        from auroraview.integration.qt.pool import WebViewPool

        # Pre-warm
        result = WebViewPool.prewarm()

        assert result is True
        assert WebViewPool.has_prewarmed() is True
        assert WebViewPool.get_prewarm_time() is not None
        assert WebViewPool.get_prewarm_time() > 0

    @pytest.mark.slow
    def test_prewarm_time_reasonable(self):
        """Test that pre-warm time is within reasonable bounds."""
        from auroraview.integration.qt.pool import WebViewPool

        WebViewPool.prewarm()

        prewarm_time = WebViewPool.get_prewarm_time()
        # Pre-warming should typically take less than 5 seconds
        assert prewarm_time < 5.0

    @pytest.mark.slow
    def test_cleanup_releases_resources(self):
        """Test that cleanup properly releases resources."""
        from auroraview.integration.qt.pool import WebViewPool

        WebViewPool.prewarm()
        assert WebViewPool.has_prewarmed() is True

        WebViewPool.cleanup()

        assert WebViewPool.has_prewarmed() is False
        assert WebViewPool.get_prewarm_time() is None

    @pytest.mark.slow
    def test_prewarm_again_after_cleanup(self):
        """Test that we can pre-warm again after cleanup."""
        from auroraview.integration.qt.pool import WebViewPool

        # First cycle
        WebViewPool.prewarm()
        WebViewPool.cleanup()

        # Second cycle
        result = WebViewPool.prewarm()

        assert result is True
        assert WebViewPool.has_prewarmed() is True


class TestWebViewPoolErrorHandling:
    """Tests for error handling in WebViewPool."""

    def setup_method(self):
        """Reset pool state before each test."""
        from auroraview.integration.qt.pool import WebViewPool

        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        from auroraview.integration.qt.pool import WebViewPool

        try:
            WebViewPool.cleanup()
        except Exception:
            pass
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_cleanup_when_not_prewarmed(self):
        """Test that cleanup is safe when not pre-warmed."""
        from auroraview.integration.qt.pool import WebViewPool

        # Should not raise
        WebViewPool.cleanup()
        assert WebViewPool.has_prewarmed() is False


class TestWebViewPoolExports:
    """Tests for WebViewPool exports from qt integration module."""

    def test_webviewpool_exported_from_qt_integration(self):
        """Test that WebViewPool is exported from auroraview.integration.qt."""
        from auroraview.integration.qt import WebViewPool

        assert WebViewPool is not None

    def test_webviewpool_in_all(self):
        """Test that WebViewPool is in __all__."""
        from auroraview.integration import qt

        assert "WebViewPool" in qt.__all__

    def test_webviewpool_has_required_methods(self):
        """Test that WebViewPool has all required class methods."""
        from auroraview.integration.qt import WebViewPool

        assert hasattr(WebViewPool, "prewarm")
        assert hasattr(WebViewPool, "has_prewarmed")
        assert hasattr(WebViewPool, "get_prewarm_time")
        assert hasattr(WebViewPool, "cleanup")

        # Verify they are callable
        assert callable(WebViewPool.prewarm)
        assert callable(WebViewPool.has_prewarmed)
        assert callable(WebViewPool.get_prewarm_time)
        assert callable(WebViewPool.cleanup)
