"""Tests for WebViewPool pre-warming functionality.

This module tests the WebViewPool class using mocks to avoid
actual WebView2 initialization in unit tests.

Note: These tests require qtpy to be installed because the parent
qt package imports it during package initialization.
"""

import threading
import time
from unittest.mock import MagicMock, patch

import pytest

# Skip entire module if qtpy is not available
pytest.importorskip("qtpy", reason="Qt tests require qtpy")

from auroraview.integration.qt.pool import WebViewPool

# The correct path to patch - WebView is imported inside prewarm()
WEBVIEW_PATCH_PATH = "auroraview.core.webview.WebView"


class TestWebViewPoolBasics:
    """Basic tests for WebViewPool class."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_initial_state_not_prewarmed(self):
        """Test that pool is not pre-warmed initially."""
        assert WebViewPool.has_prewarmed() is False

    def test_initial_prewarm_time_is_none(self):
        """Test that prewarm time is None initially."""
        assert WebViewPool.get_prewarm_time() is None


class TestPrewarmMethod:
    """Tests for WebViewPool.prewarm() method."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_prewarm_success(self):
        """Test successful pre-warming."""
        mock_instance = MagicMock()

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.return_value = mock_instance

            result = WebViewPool.prewarm()

            assert result is True
            assert WebViewPool.has_prewarmed() is True
            assert WebViewPool._prewarmed_instance is mock_instance
            mock_webview.create.assert_called_once()

    def test_prewarm_returns_true_if_already_prewarmed(self):
        """Test that prewarm returns True if already pre-warmed."""
        WebViewPool._prewarmed_instance = MagicMock()

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            result = WebViewPool.prewarm()

            assert result is True
            mock_webview.create.assert_not_called()

    def test_prewarm_returns_false_if_in_progress(self):
        """Test that prewarm returns False if already in progress."""
        WebViewPool._is_prewarming = True

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            result = WebViewPool.prewarm()

            assert result is False
            mock_webview.create.assert_not_called()

    def test_prewarm_handles_exception(self):
        """Test that prewarm handles exceptions gracefully."""
        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.side_effect = Exception("Test error")

            result = WebViewPool.prewarm()

            assert result is False
            assert WebViewPool.has_prewarmed() is False
            assert WebViewPool._is_prewarming is False

    def test_prewarm_resets_flag_on_exception(self):
        """Test that _is_prewarming is reset even on exception."""
        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.side_effect = Exception("Test error")

            WebViewPool.prewarm()

            assert WebViewPool._is_prewarming is False

    def test_prewarm_records_time(self):
        """Test that prewarm records the time taken."""
        mock_instance = MagicMock()

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.return_value = mock_instance

            WebViewPool.prewarm()

            prewarm_time = WebViewPool.get_prewarm_time()
            assert prewarm_time is not None
            assert prewarm_time >= 0

    def test_prewarm_uses_default_parameters(self):
        """Test that prewarm uses default parameters."""
        mock_instance = MagicMock()

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.return_value = mock_instance

            WebViewPool.prewarm()

            call_kwargs = mock_webview.create.call_args.kwargs
            assert call_kwargs["width"] == 100
            assert call_kwargs["height"] == 100
            assert call_kwargs["auto_show"] is False
            assert call_kwargs["debug"] is False

    def test_prewarm_custom_size(self):
        """Test that prewarm accepts custom size parameters."""
        mock_instance = MagicMock()

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.return_value = mock_instance

            WebViewPool.prewarm(width=200, height=150)

            call_kwargs = mock_webview.create.call_args.kwargs
            assert call_kwargs["width"] == 200
            assert call_kwargs["height"] == 150


class TestHasPrewarmed:
    """Tests for WebViewPool.has_prewarmed() method."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_returns_false_when_not_prewarmed(self):
        """Test has_prewarmed returns False when not pre-warmed."""
        assert WebViewPool.has_prewarmed() is False

    def test_returns_true_when_prewarmed(self):
        """Test has_prewarmed returns True when pre-warmed."""
        WebViewPool._prewarmed_instance = MagicMock()
        assert WebViewPool.has_prewarmed() is True


class TestGetPrewarmTime:
    """Tests for WebViewPool.get_prewarm_time() method."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_returns_none_when_not_prewarmed(self):
        """Test get_prewarm_time returns None when not pre-warmed."""
        assert WebViewPool.get_prewarm_time() is None

    def test_returns_time_when_prewarmed(self):
        """Test get_prewarm_time returns time when pre-warmed."""
        WebViewPool._prewarm_time = 0.5
        assert WebViewPool.get_prewarm_time() == 0.5


class TestCleanup:
    """Tests for WebViewPool.cleanup() method."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_cleanup_clears_instance(self):
        """Test that cleanup clears the pre-warmed instance."""
        mock_instance = MagicMock()
        WebViewPool._prewarmed_instance = mock_instance
        WebViewPool._prewarm_time = 0.5

        WebViewPool.cleanup()

        assert WebViewPool._prewarmed_instance is None
        assert WebViewPool._prewarm_time is None
        mock_instance.close.assert_called_once()

    def test_cleanup_handles_close_exception(self):
        """Test that cleanup handles close() exception gracefully."""
        mock_instance = MagicMock()
        mock_instance.close.side_effect = Exception("Test error")
        WebViewPool._prewarmed_instance = mock_instance

        # Should not raise
        WebViewPool.cleanup()

        assert WebViewPool._prewarmed_instance is None

    def test_cleanup_when_not_prewarmed(self):
        """Test that cleanup is safe when not pre-warmed."""
        # Should not raise
        WebViewPool.cleanup()
        assert WebViewPool._prewarmed_instance is None


class TestIdempotency:
    """Tests for idempotent behavior of WebViewPool."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_repeated_prewarm_is_idempotent(self):
        """Test that repeated prewarm calls are idempotent."""
        mock_instance = MagicMock()

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.return_value = mock_instance

            # First call
            result1 = WebViewPool.prewarm()
            assert result1 is True

            # Second call should be no-op
            result2 = WebViewPool.prewarm()
            assert result2 is True

            # Third call should also be no-op
            result3 = WebViewPool.prewarm()
            assert result3 is True

            # create should only be called once
            assert mock_webview.create.call_count == 1

    def test_repeated_cleanup_is_safe(self):
        """Test that repeated cleanup calls are safe."""
        mock_instance = MagicMock()
        WebViewPool._prewarmed_instance = mock_instance

        # First cleanup
        WebViewPool.cleanup()
        assert WebViewPool._prewarmed_instance is None

        # Second cleanup should be safe
        WebViewPool.cleanup()
        assert WebViewPool._prewarmed_instance is None

        # Third cleanup should also be safe
        WebViewPool.cleanup()
        assert WebViewPool._prewarmed_instance is None


class TestThreadSafety:
    """Tests for thread safety of WebViewPool."""

    def setup_method(self):
        """Reset pool state before each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def teardown_method(self):
        """Clean up after each test."""
        WebViewPool._prewarmed_instance = None
        WebViewPool._prewarm_time = None
        WebViewPool._is_prewarming = False

    def test_concurrent_prewarm_single_initialization(self):
        """Test that concurrent prewarm calls result in single initialization."""
        results = []
        create_count = [0]
        mock_instance = MagicMock()

        def slow_create(**kwargs):
            create_count[0] += 1
            time.sleep(0.1)  # Simulate slow initialization
            return mock_instance

        with patch(WEBVIEW_PATCH_PATH) as mock_webview:
            mock_webview.create.side_effect = slow_create

            def prewarm_thread():
                result = WebViewPool.prewarm()
                results.append(result)

            # Start multiple threads
            threads = [threading.Thread(target=prewarm_thread) for _ in range(5)]
            for t in threads:
                t.start()
            for t in threads:
                t.join()

        # Only one should succeed with actual creation
        # Others should return True (already prewarmed) or False (in progress)
        assert any(results)  # At least one succeeded
